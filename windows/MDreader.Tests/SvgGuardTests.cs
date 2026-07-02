using MDreader.Util;
using Xunit;

namespace MDreader.Tests;

public class SvgGuardTests
{
    private static Guarded G(string md) => SvgGuard.Protect(md);

    [Fact]
    public void MarkdownWithoutSvgIsUnchanged()
    {
        var src = "# Title\n\ntext **bold**\n\n```kotlin\nfun x() = 1\n```\n";
        var result = G(src);
        Assert.Equal(src, result.Markdown);
        Assert.Empty(result.Svgs);
    }

    [Fact]
    public void SingleOneLineSvgIsExtracted()
    {
        var src = "before\n<svg id=\"a\"><rect/></svg>\nafter";
        var result = G(src);
        Assert.Equal(new[] { "<svg id=\"a\"><rect/></svg>" }, result.Svgs);
        Assert.DoesNotContain("<svg", result.Markdown);
        var lines = result.Markdown.Split('\n');
        Assert.Equal(SvgGuard.Placeholder(0), lines[1]);
    }

    [Fact]
    public void LargeSvgWithBlankLinesIsKeptIntact()
    {
        var svg = "<svg viewBox=\"0 0 1400 1800\"><defs><linearGradient id=\"g1\"><stop/></linearGradient></defs>\n\n<g>\n<text>1940s</text>\n\n<text>2020s</text>\n</g>\n\n<!-- comment -->\n<text>x</text></svg>";
        var src = $"intro\n\n{svg}\n\noutro";
        var result = G(src);
        Assert.Equal(new[] { svg }, result.Svgs);
        Assert.DoesNotContain("<svg", result.Markdown);
        Assert.DoesNotContain("<rect", result.Markdown);
        Assert.DoesNotContain("</svg>", result.Markdown);
        Assert.Contains("intro", result.Markdown);
        Assert.Contains("outro", result.Markdown);
    }

    [Fact]
    public void MultipleSvgsGetSequentialPlaceholders()
    {
        var src = "<svg>A</svg>\nmid\n<svg>B</svg>";
        var result = G(src);
        Assert.Equal(new[] { "<svg>A</svg>", "<svg>B</svg>" }, result.Svgs);
        Assert.Contains(SvgGuard.Placeholder(0), result.Markdown);
        Assert.Contains(SvgGuard.Placeholder(1), result.Markdown);
    }

    [Fact]
    public void SvgInsideFencedCodeBlockIsNotExtracted()
    {
        var src = "```xml\n<svg>kept as code</svg>\n```\n<svg>real one</svg>";
        var result = G(src);
        Assert.Equal(new[] { "<svg>real one</svg>" }, result.Svgs);
        Assert.Contains("<svg>kept as code</svg>", result.Markdown);
        Assert.DoesNotContain("<svg>real one</svg>", result.Markdown);
    }

    [Fact]
    public void TildeFenceAlsoProtectsInnerSvg()
    {
        var src = "~~~\n<svg>code</svg>\n~~~\n<svg>real</svg>";
        var result = G(src);
        Assert.Equal(new[] { "<svg>real</svg>" }, result.Svgs);
        Assert.Contains("<svg>code</svg>", result.Markdown);
    }

    [Fact]
    public void PlaceholderFormatIsMarkerIndexEnd()
    {
        var result = G("<svg>x</svg>");
        var expected = $"{SvgGuard.Marker}0{SvgGuard.End}";
        Assert.Equal(expected, SvgGuard.Placeholder(0));
        Assert.Contains(SvgGuard.Placeholder(0), result.Markdown);
    }

    [Fact]
    public void TextAfterClosedSvgOnSameLineIsPreserved()
    {
        var src = "line\n<svg><rect/></svg>\ntail";
        var result = G(src);
        Assert.Contains("tail", result.Markdown);
        Assert.Equal(new[] { "<svg><rect/></svg>" }, result.Svgs);
    }
}
