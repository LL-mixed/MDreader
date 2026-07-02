using System.IO;
using MDreader.Store;
using Xunit;

namespace MDreader.Tests;

public class ZoomStoreTests
{
    private static string Tmp()
    {
        var dir = Path.Combine(Path.GetTempPath(), "mdreader-zoom-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        return dir;
    }

    [Fact]
    public void RoundTripPersistsPerHash()
    {
        var dir = Tmp();
        try
        {
            var z = new ZoomStore(dir);
            Assert.Null(z.ZoomFor("aaa"));
            z.SetZoom(1.5, "aaa");
            z.SetZoom(0.75, "bbb");
            Assert.Equal(1.5, z.ZoomFor("aaa"));
            // reopen -> persisted
            var z2 = new ZoomStore(dir);
            Assert.Equal(1.5, z2.ZoomFor("aaa"));
            Assert.Equal(0.75, z2.ZoomFor("bbb"));
        }
        finally { TryDelete(dir); }
    }

    [Fact]
    public void MissingFileIsEmpty()
    {
        var dir = Tmp();
        try
        {
            var z = new ZoomStore(dir);
            Assert.Null(z.ZoomFor("x"));
        }
        finally { TryDelete(dir); }
    }

    private static void TryDelete(string dir)
    {
        try { Directory.Delete(dir, true); } catch { }
    }
}
