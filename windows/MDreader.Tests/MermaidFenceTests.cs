using MDreader.Util;
using Xunit;

namespace MDreader.Tests;

public class MermaidFenceTests
{
    private static string N(string md) => MermaidFenceNormalizer.Normalize(md);

    [Fact]
    public void StandardMermaidFenceUnchanged()
    {
        var src = "```mermaid\nflowchart LR\n  A --> B\n```";
        Assert.Equal(src, N(src));
    }

    [Fact]
    public void SequenceFenceRewritten() =>
        Assert.Equal(
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```",
            N("```sequence\nsequenceDiagram\n  A->>B: hi\n```"));

    [Fact]
    public void AliasTagCaseInsensitive() =>
        Assert.Equal(
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```",
            N("```Sequence\nsequenceDiagram\n  A->>B: hi\n```"));

    [Fact]
    public void GanttAndFlowAliasesRewrite()
    {
        Assert.Equal("```mermaid\ntitle X\n```", N("```gantt\ntitle X\n```"));
        Assert.Equal("```mermaid\nflowchart TD\n```", N("```flow\nflowchart TD\n```"));
    }

    [Fact]
    public void TildeFencesPreserveMarker() =>
        Assert.Equal("~~~mermaid\nsequenceDiagram\n```", N("~~~sequence\nsequenceDiagram\n```"));

    [Fact]
    public void UntaggedBlockWithKeywordRewrites() =>
        Assert.Equal(
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```",
            N("```\nsequenceDiagram\n  A->>B: hi\n```"));

    [Fact]
    public void UntaggedBlockWithoutKeywordLeftAlone()
    {
        var src = "```\njust some plain text\nnot a diagram\n```";
        Assert.Equal(src, N(src));
    }

    [Fact]
    public void TaggedRealCodeNeverRewritten()
    {
        var src = "```kotlin\nflowchart fun build() = 1\n```";
        Assert.Equal(src, N(src));
        var text = "```text\ngraph this is prose\n```";
        Assert.Equal(text, N(text));
    }

    [Fact]
    public void LanguageAttributePreserved() =>
        Assert.Equal(
            "```mermaid {#d}\nflowchart LR\n  A --> B\n```",
            N("```sequence {#d}\nflowchart LR\n  A --> B\n```"));

    [Fact]
    public void LeadingIndentUpToThreeSpacesPreserved() =>
        Assert.Equal("  ```mermaid\nflowchart LR\n  ```", N("  ```sequence\nflowchart LR\n  ```"));

    [Fact]
    public void FenceLookingLinesInsideCodeBlockNotRewritten()
    {
        var src = "```kotlin\nval s = \"```sequence\"\n```";
        Assert.Equal(src, N(src));
    }

    [Fact]
    public void MultipleMixedBlocksHandledIndependently()
    {
        var src = "# Doc\n\n```sequence\nsequenceDiagram\n  A->>B: x\n```\n\n```kotlin\nfun main() {}\n```\n\n```gantt\ntitle T\n```";
        var expected = "# Doc\n\n```mermaid\nsequenceDiagram\n  A->>B: x\n```\n\n```kotlin\nfun main() {}\n```\n\n```mermaid\ntitle T\n```";
        Assert.Equal(expected, N(src));
    }

    [Fact]
    public void UnterminatedBlockRewritesAndRunsToEof() =>
        Assert.Equal("```mermaid\nflowchart LR\n  A --> B", N("```sequence\nflowchart LR\n  A --> B"));

    [Fact]
    public void EmptyInputReturnsEmpty() => Assert.Equal("", N(""));

    [Fact]
    public void CloseFenceShorterThanOpenerIsNotClose()
    {
        var src = "````text\n```sequence\n````";
        Assert.Equal(src, N(src));
    }
}
