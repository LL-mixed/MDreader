using System.IO;
using MDreader.Store;
using Xunit;

namespace MDreader.Tests;

public class CacheTests
{
    private static (DocRepository repo, string dir) MakeRepo()
    {
        var dir = Path.Combine(Path.GetTempPath(), "mdreader-cache-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        return (DocRepository.Open(dir), dir);
    }

    [Fact]
    public void CacheInsertsOnceAndWritesFile()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            var id = repo.Cache("Doc", "# Hi", null);
            repo.Cache("Doc Again", "# Hi", null);
            Assert.Equal(1, repo.All().Count);
            Assert.Equal("# Hi", DocStore.Read(repo.DocsDir, id));
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void CacheDifferentContentSeparateRows()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            repo.Cache("A", "aaa", null);
            repo.Cache("B", "bbb", null);
            Assert.Equal(2, repo.All().Count);
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void EmptyTitleGetsDefault()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            repo.Cache("", "x", null);
            Assert.Equal(DocRepository.DefaultTitle, repo.All()[0].Title);
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void DeleteRemovesRowAndFile()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            repo.Cache("T", "body", null);
            var id = repo.All()[0].Id;
            repo.Delete(id);
            Assert.Empty(repo.All());
            Assert.Null(DocStore.Read(repo.DocsDir, id));
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void CacheReturnsStableIdForSameContent()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            var id1 = repo.Cache("Doc", "# Hi", null);
            var id2 = repo.Cache("Doc Again", "# Hi", null);
            Assert.Equal(id1, id2);
            Assert.Equal(id1, repo.All()[0].Id);
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void CacheDedupBackfillsMissingSourceUri()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            repo.Cache("Doc", "# Hi", null);
            repo.Cache("Doc Again", "# Hi", "/path/to/doc.md");
            Assert.Equal("/path/to/doc.md", repo.All()[0].SourceUri);
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void CacheDedupKeepsExistingSourceUri()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            repo.Cache("Doc", "# Hi", "/real/file.md");
            repo.Cache("Doc Again", "# Hi", null);
            Assert.Equal("/real/file.md", repo.All()[0].SourceUri);
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void RefreshFromSourceUpdatesChangedContent()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            var (src, srcDir) = WriteSource("note.md", "# v1");
            try
            {
                var id = repo.Cache("note", "# v1", src);
                Assert.Equal("# v1", repo.LoadContent(id));
                File.WriteAllText(src, "# v2");
                Assert.True(repo.RefreshFromSource(id));
                Assert.Equal("# v2", repo.LoadContent(id));
            }
            finally { TryDelete(srcDir); }
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void RefreshFromSourceNoopWhenUnchanged()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            var (src, srcDir) = WriteSource("note.md", "# same");
            try
            {
                var id = repo.Cache("note", "# same", src);
                Assert.False(repo.RefreshFromSource(id));
                Assert.Equal("# same", repo.LoadContent(id));
            }
            finally { TryDelete(srcDir); }
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void RefreshFromSourceFalseWhenNoSource()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            var id = repo.Cache("note", "# x", null);
            Assert.False(repo.RefreshFromSource(id));
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void RefreshFromSourceFalseWhenSourceMissing()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            var id = repo.Cache("note", "# x", "/nonexistent/path-1234567890.md");
            Assert.False(repo.RefreshFromSource(id));
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void SetFavoritePersists()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            var id = repo.Cache("Fav", "x", null);
            repo.SetFavorite(id, true);
            Assert.True(repo.All()[0].Favorite);
            repo.SetFavorite(id, false);
            Assert.False(repo.All()[0].Favorite);
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    [Fact]
    public void SearchMatchesCaseInsensitively()
    {
        var (repo, dir) = MakeRepo();
        try
        {
            repo.Cache("Kotlin Notes", "a", null);
            repo.Cache("Rust Guide", "b", null);
            Assert.Equal(1, repo.Search("kotlin").Count);
            Assert.Equal(1, repo.Search("rust").Count);
            Assert.Equal(1, repo.Search("notes").Count);
            Assert.Equal(0, repo.Search("xyz").Count);
        }
        finally { repo.Dispose(); TryDelete(dir); }
    }

    private static (string path, string dir) WriteSource(string name, string body)
    {
        var dir = Path.Combine(Path.GetTempPath(), "mdreader-src-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        var path = Path.Combine(dir, name);
        File.WriteAllText(path, body);
        return (path, dir);
    }

    private static void TryDelete(string dir)
    {
        try { Directory.Delete(dir, true); } catch { /* best effort */ }
    }
}
