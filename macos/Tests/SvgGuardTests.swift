import XCTest
@testable import MDreader

final class SvgGuardTests: XCTestCase {
    private func g(_ md: String) -> Guarded { SvgGuard.protect(md) }

    func testMarkdownWithoutSvgIsReturnedUnchanged() {
        let src = "# Title\n\ntext **bold**\n\n```kotlin\nfun x() = 1\n```\n"
        let result = g(src)
        XCTAssertEqual(result.markdown, src)
        XCTAssertTrue(result.svgs.isEmpty)
    }

    func testSingleOneLineSvgIsExtracted() {
        let src = "before\n<svg id=\"a\"><rect/></svg>\nafter"
        let result = g(src)
        XCTAssertEqual(result.svgs, ["<svg id=\"a\"><rect/></svg>"])
        XCTAssertFalse(result.markdown.contains("<svg"))
        XCTAssertEqual(result.markdown.components(separatedBy: "\n")[1], SvgGuard.placeholder(0))
    }

    func testLargeSvgWithBlankLinesIsKeptIntact() {
        let svg = "<svg viewBox=\"0 0 1400 1800\"><defs><linearGradient id=\"g1\"><stop/></linearGradient></defs>\n\n<g>\n<text>1940s</text>\n\n<text>2020s</text>\n</g>\n\n<!-- comment -->\n<text>x</text></svg>"
        let src = "intro\n\n\(svg)\n\noutro"
        let result = g(src)
        XCTAssertEqual(result.svgs, [svg])
        XCTAssertFalse(result.markdown.contains("<svg"))
        XCTAssertFalse(result.markdown.contains("<rect"))
        XCTAssertFalse(result.markdown.contains("</svg>"))
        XCTAssertTrue(result.markdown.contains("intro"))
        XCTAssertTrue(result.markdown.contains("outro"))
    }

    func testMultipleSvgsGetSequentialPlaceholders() {
        let src = "<svg>A</svg>\nmid\n<svg>B</svg>"
        let result = g(src)
        XCTAssertEqual(result.svgs, ["<svg>A</svg>", "<svg>B</svg>"])
        XCTAssertTrue(result.markdown.contains(SvgGuard.placeholder(0)))
        XCTAssertTrue(result.markdown.contains(SvgGuard.placeholder(1)))
    }

    func testSvgInsideFencedCodeBlockIsNotExtracted() {
        let src = "```xml\n<svg>kept as code</svg>\n```\n<svg>real one</svg>"
        let result = g(src)
        XCTAssertEqual(result.svgs, ["<svg>real one</svg>"])
        XCTAssertTrue(result.markdown.contains("<svg>kept as code</svg>"))
        XCTAssertFalse(result.markdown.contains("<svg>real one</svg>"))
    }

    func testTildeFenceAlsoProtectsInnerSvg() {
        let src = "~~~\n<svg>code</svg>\n~~~\n<svg>real</svg>"
        let result = g(src)
        XCTAssertEqual(result.svgs, ["<svg>real</svg>"])
        XCTAssertTrue(result.markdown.contains("<svg>code</svg>"))
    }

    func testPlaceholderFormatIsMarkerIndexEnd() {
        let result = g("<svg>x</svg>")
        let expected = "\u{01}" + "0" + "\u{02}"
        XCTAssertEqual(SvgGuard.placeholder(0), expected)
        XCTAssertTrue(result.markdown.contains(SvgGuard.placeholder(0)))
    }

    func testTextAfterClosedSvgOnSameLineIsPreserved() {
        let src = "line\n<svg><rect/></svg>\ntail"
        let result = g(src)
        XCTAssertTrue(result.markdown.contains("tail"))
        XCTAssertEqual(result.svgs, ["<svg><rect/></svg>"])
    }
}
