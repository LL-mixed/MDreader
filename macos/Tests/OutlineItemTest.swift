import XCTest
@testable import MDreader

final class OutlineItemTest: XCTestCase {
    func testDecodeOutlineJson() throws {
        let json = #"[{"index":0,"level":1,"text":"Title"},{"index":1,"level":2,"text":"Sub"}]"#
        let data = try XCTUnwrap(json.data(using: .utf8))
        let items = try JSONDecoder().decode([OutlineItem].self, from: data)
        XCTAssertEqual(items.count, 2)
        XCTAssertEqual(items[0].index, 0)
        XCTAssertEqual(items[0].level, 1)
        XCTAssertEqual(items[1].text, "Sub")
    }
}
