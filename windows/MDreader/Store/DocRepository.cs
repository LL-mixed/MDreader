using System.Globalization;
using System.IO;
using System.Text;
using MDreader.Util;
using Microsoft.Data.Sqlite;

namespace MDreader.Store;

/// <summary>
/// SQLite metadata (one table) + &lt;uuid&gt;.md body files + SHA-256 content dedup
/// + refresh-from-source. Port of linux cache.rs / macOS DocRepository.swift.
/// </summary>
public sealed class DocRepository : IDisposable
{
    public const string DefaultTitle = "未命名文档";

    private const string Schema =
        "CREATE TABLE IF NOT EXISTS cached_docs (" +
        "id TEXT PRIMARY KEY," +
        "title TEXT NOT NULL," +
        "content_hash TEXT NOT NULL," +
        "source_uri TEXT," +
        "char_count INTEGER NOT NULL," +
        "size_bytes INTEGER NOT NULL," +
        "cached_at INTEGER NOT NULL," +
        "opened_at INTEGER NOT NULL," +
        "favorite INTEGER NOT NULL DEFAULT 0);" +
        "CREATE INDEX IF NOT EXISTS idx_content_hash ON cached_docs(content_hash);" +
        "CREATE INDEX IF NOT EXISTS idx_opened_at ON cached_docs(opened_at);";

    private readonly SqliteConnection _conn;
    private readonly object _gate = new();
    private readonly string _docsDir;

    private DocRepository(SqliteConnection conn, string docsDir)
    {
        _conn = conn;
        _docsDir = docsDir;
    }

    /// <summary>Directory holding the <c>&lt;uuid&gt;.md</c> body files.</summary>
    public string DocsDir => _docsDir;

    /// <summary>Open (or create) the cache under <paramref name="dataDir"/>.</summary>
    public static DocRepository Open(string dataDir)
    {
        var docsDir = Path.Combine(dataDir, "docs");
        Directory.CreateDirectory(docsDir);
        var conn = new SqliteConnection("Data Source=" + Path.Combine(dataDir, "cache.db"));
        conn.Open();
        using (var cmd = new SqliteCommand(Schema, conn))
        {
            cmd.ExecuteNonQuery();
        }
        return new DocRepository(conn, docsDir);
    }

    /// <summary>Insert or dedup. Returns the doc id (stable for identical content).</summary>
    public Guid Cache(string title, string markdown, string? sourceUri)
    {
        var hash = ContentHash.Sha256Hex(markdown);
        var now = NowMillis();
        Guid id;
        lock (_gate)
        {
            var existing = QueryFirstOrDefault<string>(
                "SELECT id FROM cached_docs WHERE content_hash = @h",
                ("@h", hash));
            if (existing != null)
            {
                // Backfill source_uri when this open provides one the cached row lacks.
                // COALESCE keeps any existing source so a later drop never wipes a recorded
                // path — relative image/SVG refs need the source dir to resolve.
                Exec(
                    "UPDATE cached_docs SET opened_at = @o, source_uri = COALESCE(@s, source_uri) WHERE id = @id",
                    ("@o", now), ("@s", (object?)sourceUri), ("@id", existing));
                id = Guid.TryParse(existing, out var g) ? g : Guid.NewGuid();
            }
            else
            {
                var resolved = title.Length == 0 ? DefaultTitle : title;
                id = Guid.NewGuid();
                Exec(
                    "INSERT INTO cached_docs " +
                    "(id, title, content_hash, source_uri, char_count, size_bytes, cached_at, opened_at, favorite) " +
                    "VALUES (@id, @t, @h, @s, @cc, @sb, @c, @o, 0)",
                    ("@id", id.ToString()),
                    ("@t", resolved),
                    ("@h", hash),
                    ("@s", (object?)sourceUri),
                    ("@cc", (long)CharCount(markdown)),
                    ("@sb", (long)Encoding.UTF8.GetByteCount(markdown)),
                    ("@c", now),
                    ("@o", now));
            }
        }
        DocStore.Write(_docsDir, id, markdown);
        return id;
    }

    /// <summary>All docs, newest-opened first.</summary>
    public IReadOnlyList<DocInfo> All(bool favoritesOnly = false)
    {
        var list = new List<DocInfo>();
        lock (_gate)
        {
            using var cmd = new SqliteCommand(
                "SELECT id, title, content_hash, source_uri, opened_at, favorite, char_count " +
                "FROM cached_docs " + (favoritesOnly ? "WHERE favorite = 1 " : "") +
                "ORDER BY opened_at DESC",
                _conn);
            using var r = cmd.ExecuteReader();
            while (r.Read())
            {
                if (!Guid.TryParse(r.GetString(0), out var id)) continue;
                list.Add(new DocInfo(
                    id,
                    r.GetString(1),
                    r.GetString(2),
                    r.IsDBNull(3) ? null : r.GetString(3),
                    r.GetInt64(4),
                    r.GetInt64(5) != 0,
                    r.GetInt64(6)));
            }
        }
        return list;
    }

    /// <summary>Case-insensitive substring search over titles.</summary>
    public IReadOnlyList<DocInfo> Search(string query, bool favoritesOnly = false)
    {
        var q = query.ToLowerInvariant();
        return All(favoritesOnly)
            .Where(d => d.Title.ToLowerInvariant().Contains(q, StringComparison.Ordinal))
            .ToList();
    }

    /// <summary>Bump openedAt and return the cached body.</summary>
    public string? LoadContent(Guid id)
    {
        lock (_gate)
        {
            Exec("UPDATE cached_docs SET opened_at = @o WHERE id = @id",
                ("@o", NowMillis()), ("@id", id.ToString()));
        }
        return DocStore.Read(_docsDir, id);
    }

    public void SetFavorite(Guid id, bool favorite)
    {
        lock (_gate)
        {
            Exec("UPDATE cached_docs SET favorite = @f WHERE id = @id",
                ("@f", favorite ? 1L : 0L), ("@id", id.ToString()));
        }
    }

    public void Delete(Guid id)
    {
        lock (_gate)
        {
            Exec("DELETE FROM cached_docs WHERE id = @id", ("@id", id.ToString()));
        }
        DocStore.Delete(_docsDir, id);
    }

    /// <summary>
    /// Re-read the original file backing <paramref name="id"/>; if it exists and differs
    /// from the cached snapshot, update the cached content + metadata. Returns true when a
    /// refresh happened.
    /// </summary>
    public bool RefreshFromSource(Guid id)
    {
        string? sourceUri;
        string currentHash;
        lock (_gate)
        {
            using var cmd = new SqliteCommand(
                "SELECT source_uri, content_hash FROM cached_docs WHERE id = @id", _conn);
            cmd.Parameters.AddWithValue("@id", id.ToString());
            using var r = cmd.ExecuteReader();
            if (!r.Read()) return false;
            sourceUri = r.IsDBNull(0) ? null : r.GetString(0);
            currentHash = r.GetString(1);
        }
        if (sourceUri == null) return false;
        string text;
        try { text = File.ReadAllText(sourceUri); }
        catch { return false; }
        var hash = ContentHash.Sha256Hex(text);
        if (hash == currentHash) return false;
        var now = NowMillis();
        lock (_gate)
        {
            Exec(
                "UPDATE cached_docs SET content_hash=@h, char_count=@cc, size_bytes=@sb, opened_at=@o WHERE id=@id",
                ("@h", hash),
                ("@cc", (long)CharCount(text)),
                ("@sb", (long)Encoding.UTF8.GetByteCount(text)),
                ("@o", now),
                ("@id", id.ToString()));
        }
        DocStore.Write(_docsDir, id, text);
        return true;
    }

    public void Dispose()
    {
        lock (_gate) _conn.Dispose();
    }

    private void Exec(string sql, params (string Name, object? Value)[] args)
    {
        using var cmd = new SqliteCommand(sql, _conn);
        foreach (var (n, v) in args) cmd.Parameters.AddWithValue(n, v ?? System.DBNull.Value);
        cmd.ExecuteNonQuery();
    }

    private T? QueryFirstOrDefault<T>(string sql, params (string Name, object? Value)[] args)
    {
        using var cmd = new SqliteCommand(sql, _conn);
        foreach (var (n, v) in args) cmd.Parameters.AddWithValue(n, v ?? System.DBNull.Value);
        using var r = cmd.ExecuteReader();
        if (!r.Read()) return default;
        var v = r.GetValue(0);
        return v is System.DBNull ? default : (T)v;
    }

    private static long NowMillis() => DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

    // Approximate char count (UTF-16 code units); differs from Rust's scalar-value count only
    // for non-BMP characters, which don't surface in any assertion.
    private static int CharCount(string s) => s.Length;
}
