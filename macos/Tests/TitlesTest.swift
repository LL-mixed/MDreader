import XCTest
@testable import MDreader

final class TitlesTest: XCTestCase {
    func testStripsMarkdownExtension() {
        XCTAssertEqual(Titles.fromPath("readme.md"), "readme")
    }

    func testIgnoresExtensionCase() {
        XCTAssertEqual(Titles.fromPath("/a/b/Notes.MARKDOWN"), "Notes")
    }

    func testHandlesMultipleDots() {
        XCTAssertEqual(Titles.fromPath("a.b.md"), "a.b")
    }

    func testPreservesNonMarkdownExtension() {
        XCTAssertEqual(Titles.fromPath("archive.txt"), "archive.txt")
    }

    func testNoExtensionReturnedAsIs() {
        XCTAssertEqual(Titles.fromPath("noext"), "noext")
    }

    func testEmptyPathReturnsEmpty() {
        XCTAssertEqual(Titles.fromPath(""), "")
    }

    func testHandlesMdown() {
        XCTAssertEqual(Titles.fromPath("WeChat Files/doc.mdown"), "doc")
    }

    func testHandlesBackslashSeparator() {
        XCTAssertEqual(Titles.fromPath("C:\\Users\\me\\file.md"), "file")
    }
}
