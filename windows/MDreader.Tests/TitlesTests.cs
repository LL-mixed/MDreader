using MDreader.Util;
using Xunit;

namespace MDreader.Tests;

public class TitlesTests
{
    [Fact] public void StripsMarkdownExtension() => Assert.Equal("readme", Titles.FromPath("readme.md"));
    [Fact] public void IgnoresExtensionCase() => Assert.Equal("Notes", Titles.FromPath("/a/b/Notes.MARKDOWN"));
    [Fact] public void HandlesMultipleDots() => Assert.Equal("a.b", Titles.FromPath("a.b.md"));
    [Fact] public void PreservesNonMarkdownExtension() => Assert.Equal("archive.txt", Titles.FromPath("archive.txt"));
    [Fact] public void NoExtensionReturnedAsIs() => Assert.Equal("noext", Titles.FromPath("noext"));
    [Fact] public void EmptyPathReturnsEmpty() => Assert.Equal("", Titles.FromPath(""));
    [Fact] public void HandlesMdown() => Assert.Equal("doc", Titles.FromPath("WeChat Files/doc.mdown"));
    [Fact] public void HandlesBackslashSeparator() => Assert.Equal("file", Titles.FromPath("C:\\Users\\me\\file.md"));
}
