using System.IO;
using MDreader.Render;
using Xunit;

namespace MDreader.Tests;

public class PreprocessTests
{
    private static string TmpDir()
    {
        var dir = Path.Combine(Path.GetTempPath(), "mdreader-preprocess-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(dir);
        return dir;
    }

    [Fact]
    public void NoBaseDirReturnsUnchanged()
    {
        var md = "![x](rel.png)";
        Assert.Equal(md, Preprocess.ResolveImages(md, null));
    }

    [Fact]
    public void AbsoluteAndRemoteUrlsLeftAlone()
    {
        var md = "![a](http://e/a.png) ![b](https://e/b.png) ![c](/abs/c.png) ![d](#anchor)";
        Assert.Equal(md, Preprocess.ResolveImages(md, "/tmp"));
    }

    [Fact]
    public void RasterImageIsInlinedAsDataUri()
    {
        var dir = TmpDir();
        try
        {
            var png = new byte[] { 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A };
            File.WriteAllBytes(Path.Combine(dir, "pixel.png"), png);
            var result = Preprocess.ResolveImages("![p](pixel.png)", dir);
            Assert.Contains("![p](data:image/png;base64,", result);
            Assert.DoesNotContain("pixel.png)", result);
        }
        finally
        {
            Directory.Delete(dir, true);
        }
    }

    [Fact]
    public void SvgImageIsInlinedAsRawSvg()
    {
        var dir = TmpDir();
        try
        {
            File.WriteAllText(Path.Combine(dir, "d.svg"), "<svg><rect/></svg>");
            var result = Preprocess.ResolveImages("![d](d.svg)", dir);
            Assert.Contains("\n\n<svg><rect/></svg>\n\n", result);
        }
        finally
        {
            Directory.Delete(dir, true);
        }
    }

    [Fact]
    public void MissingFileFallsBackToOriginalSrc()
    {
        var dir = TmpDir();
        try
        {
            Assert.Equal("![m](nope.png)", Preprocess.ResolveImages("![m](nope.png)", dir));
        }
        finally
        {
            Directory.Delete(dir, true);
        }
    }

    [Fact]
    public void TitleAfterSrcIsPreservedOnRemote()
    {
        var md = "![a](http://e/a.png \"t\")";
        Assert.Equal(md, Preprocess.ResolveImages(md, "/tmp"));
    }
}
