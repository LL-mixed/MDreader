using System.IO;
using MDreader.Store;
using Xunit;

namespace MDreader.Tests;

public class SessionStoreTests
{
    private static string Tmp()
    {
        var dir = Path.Combine(Path.GetTempPath(), "mdreader-sess-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        return dir;
    }

    [Fact]
    public void RoundTripPersistsId()
    {
        var dir = Tmp();
        try
        {
            var id = Guid.NewGuid();
            var s = new SessionStore(dir);
            Assert.Null(s.LastDocID);
            s.SetLastDocID(id);
            var s2 = new SessionStore(dir);
            Assert.Equal(id, s2.LastDocID);
        }
        finally { TryDelete(dir); }
    }

    [Fact]
    public void ClearingWritesNull()
    {
        var dir = Tmp();
        try
        {
            var id = Guid.NewGuid();
            var s = new SessionStore(dir);
            s.SetLastDocID(id);
            s.SetLastDocID(null);
            var s2 = new SessionStore(dir);
            Assert.Null(s2.LastDocID);
        }
        finally { TryDelete(dir); }
    }

    private static void TryDelete(string dir)
    {
        try { Directory.Delete(dir, true); } catch { }
    }
}
