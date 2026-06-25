import XCTest
@testable import MDreader

final class ReaderModelTests: XCTestCase {
    func testLoadSampleSetsDefaultContent() {
        let model = ReaderModel()
        model.loadSample()
        XCTAssertFalse(model.markdown.isEmpty)
        XCTAssertEqual(model.title, "MDreader")
    }

    func testOpenReadsFileContentAndTitle() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let url = dir.appendingPathComponent("note.md")
        let body = "# Title\n\nbody text"
        try body.write(to: url, atomically: true, encoding: .utf8)
        defer { try? FileManager.default.removeItem(at: dir) }

        let model = ReaderModel()
        model.open(url)
        XCTAssertEqual(model.markdown, body)
        XCTAssertEqual(model.title, "note")
    }
}
