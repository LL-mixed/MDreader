import XCTest
@testable import MDreader

final class MermaidFenceNormalizerTests: XCTestCase {
    private func n(_ md: String) -> String { MermaidFenceNormalizer.normalize(md) }

    func testStandardMermaidFenceIsLeftUnchanged() {
        let src = "```mermaid\nflowchart LR\n  A --> B\n```"
        XCTAssertEqual(n(src), src)
    }

    func testSequenceFenceIsRewrittenToMermaid() {
        XCTAssertEqual(
            n("```sequence\nsequenceDiagram\n  A->>B: hi\n```"),
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```"
        )
    }

    func testAliasTagIsCaseInsensitive() {
        XCTAssertEqual(
            n("```Sequence\nsequenceDiagram\n  A->>B: hi\n```"),
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```"
        )
    }

    func testGanttAndFlowAliasesRewrite() {
        XCTAssertEqual(n("```gantt\ntitle X\n```"), "```mermaid\ntitle X\n```")
        XCTAssertEqual(n("```flow\nflowchart TD\n```"), "```mermaid\nflowchart TD\n```")
    }

    func testTildeFencesRewriteAndPreserveMarker() {
        XCTAssertEqual(n("~~~sequence\nsequenceDiagram\n```"), "~~~mermaid\nsequenceDiagram\n```")
    }

    func testUntaggedBlockWithMermaidKeywordRewrites() {
        XCTAssertEqual(
            n("```\nsequenceDiagram\n  A->>B: hi\n```"),
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```"
        )
    }

    func testUntaggedBlockWithoutKeywordIsLeftAlone() {
        let src = "```\njust some plain text\nnot a diagram\n```"
        XCTAssertEqual(n(src), src)
    }

    func testTaggedRealCodeIsNeverRewritten() {
        let src = "```kotlin\nflowchart fun build() = 1\n```"
        XCTAssertEqual(n(src), src)
        let text = "```text\ngraph this is prose\n```"
        XCTAssertEqual(n(text), text)
    }

    func testLanguageAttributeBlockIsPreserved() {
        XCTAssertEqual(
            n("```sequence {#d}\nflowchart LR\n  A --> B\n```"),
            "```mermaid {#d}\nflowchart LR\n  A --> B\n```"
        )
    }

    func testLeadingIndentationUpToThreeSpacesIsPreserved() {
        XCTAssertEqual(
            n("  ```sequence\nflowchart LR\n  ```"),
            "  ```mermaid\nflowchart LR\n  ```"
        )
    }

    func testFenceLookingLinesInsideCodeBlockAreNotRewritten() {
        let src = "```kotlin\nval s = \"```sequence\"\n```"
        XCTAssertEqual(n(src), src)
    }

    func testMultipleMixedBlocksAreHandledIndependently() {
        let src = "# Doc\n\n```sequence\nsequenceDiagram\n  A->>B: x\n```\n\n```kotlin\nfun main() {}\n```\n\n```gantt\ntitle T\n```"
        let expected = "# Doc\n\n```mermaid\nsequenceDiagram\n  A->>B: x\n```\n\n```kotlin\nfun main() {}\n```\n\n```mermaid\ntitle T\n```"
        XCTAssertEqual(n(src), expected)
    }

    func testUnterminatedMermaidBlockRewritesOpenerAndRunsToEOF() {
        XCTAssertEqual(
            n("```sequence\nflowchart LR\n  A --> B"),
            "```mermaid\nflowchart LR\n  A --> B"
        )
    }

    func testEmptyInputReturnsEmpty() {
        XCTAssertEqual(n(""), "")
    }

    func testCloseFenceShorterThanOpenerIsNotTreatedAsClose() {
        let src = "````text\n```sequence\n````"
        XCTAssertEqual(n(src), src)
    }
}
