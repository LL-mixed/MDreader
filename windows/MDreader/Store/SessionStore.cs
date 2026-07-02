using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace MDreader.Store;

/// <summary>
/// Last-opened doc id for session restore. Port of linux session_store.rs /
/// macOS SessionStore.swift. Persisted as <c>session.json</c>.
/// </summary>
public sealed class SessionStore
{
    private sealed class Snapshot
    {
        [JsonPropertyName("lastDocID")]
        public Guid? LastDocID { get; set; }
    }

    private readonly string _path;

    public Guid? LastDocID { get; private set; }

    public SessionStore(string dir)
    {
        _path = Path.Combine(dir, "session.json");
        try
        {
            var snap = JsonSerializer.Deserialize<Snapshot>(File.ReadAllText(_path));
            LastDocID = snap?.LastDocID;
        }
        catch
        {
            LastDocID = null;
        }
    }

    public void SetLastDocID(Guid? id)
    {
        LastDocID = id;
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(_path)!);
            File.WriteAllText(_path, JsonSerializer.Serialize(new Snapshot { LastDocID = id }));
        }
        catch
        {
            // Best-effort persistence.
        }
    }
}
