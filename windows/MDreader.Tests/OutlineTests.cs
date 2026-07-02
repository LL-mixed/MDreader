using MDreader.Render;
using Xunit;

namespace MDreader.Tests;

public class OutlineTests
{
    [Fact]
    public void ParsesTypicalPayload()
    {
        var json = "[{\"index\":0,\"level\":1,\"text\":\"Title\"}," +
                   "{\"index\":1,\"level\":2,\"text\":\"Section\"}," +
                   "{\"index\":2,\"level\":3,\"text\":\"子标题\"}]";
        var items = Outline.Parse(json)!;
        Assert.Equal(3, items.Count);
        Assert.Equal("Title", items[0].Text);
        Assert.Equal(2u, items[1].Level);
        Assert.Equal("子标题", items[2].Text);
    }

    [Fact]
    public void EmptyArray()
    {
        var items = Outline.Parse("[]");
        Assert.NotNull(items);
        Assert.Empty(items);
    }

    [Fact]
    public void InvalidJsonReturnsNull() => Assert.Null(Outline.Parse("not json"));
}
