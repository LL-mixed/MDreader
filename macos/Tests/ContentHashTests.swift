import XCTest
@testable import MDreader

final class ContentHashTests: XCTestCase {
    func testEmptyString() {
        XCTAssertEqual(
            ContentHash.sha256Hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        )
    }

    func testKnownVectorAbc() {
        XCTAssertEqual(
            ContentHash.sha256Hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        )
    }

    func testStableForSameInput() {
        XCTAssertEqual(ContentHash.sha256Hex("hello"), ContentHash.sha256Hex("hello"))
    }

    func testDifferentForDifferentInput() {
        XCTAssertNotEqual(ContentHash.sha256Hex("a"), ContentHash.sha256Hex("b"))
    }

    func testOutputIs64LowerHexChars() {
        let hex = ContentHash.sha256Hex("some markdown content")
        XCTAssertEqual(hex.count, 64)
        XCTAssertTrue(hex.allSatisfy { "0123456789abcdef".contains($0) })
    }
}
