using MDreader.Util;
using Xunit;

namespace MDreader.Tests;

public class FenceTests
{
    [Fact]
    public void MatchesBacktickFenceWithTag()
    {
        var m = Fence.MatchLine("```kotlin")!;
        Assert.Equal("```", m.Marker);
        Assert.Equal("kotlin", m.Tag);
        Assert.Equal("", m.Indent);
        Assert.Equal("", m.Attrs);
    }

    [Fact]
    public void MatchesTildeFenceWithIndentAndAttrs()
    {
        var m = Fence.MatchLine("  ~~~mermaid {#d}")!;
        Assert.Equal("  ", m.Indent);
        Assert.Equal("~~~", m.Marker);
        Assert.Equal("mermaid", m.Tag);
        Assert.Equal("{#d}", m.Attrs);
    }

    [Fact]
    public void MatchesBareCloser()
    {
        var m = Fence.MatchLine("````")!;
        Assert.Equal("````", m.Marker);
        Assert.Equal("", m.Tag);
    }

    [Fact]
    public void RejectsNonFenceLine()
    {
        Assert.Null(Fence.MatchLine("just text"));
        Assert.Null(Fence.MatchLine("# heading"));
        // Only two backticks is not a fence.
        Assert.Null(Fence.MatchLine("``kotlin"));
    }
}
