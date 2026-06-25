import XCTest
@testable import MDreader

final class ZoomStoreTest: XCTestCase {
    private func tmpDir() throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    func testSetAndGetZoom() throws {
        let store = ZoomStore(directory: try tmpDir())
        store.setZoom(1.5, for: "hashA")
        XCTAssertEqual(store.zoom(for: "hashA"), 1.5)
        XCTAssertNil(store.zoom(for: "hashB"))
    }

    func testPersistAcrossInstances() throws {
        let dir = try tmpDir()
        let store1 = ZoomStore(directory: dir)
        store1.setZoom(2.0, for: "hashX")
        let store2 = ZoomStore(directory: dir)
        XCTAssertEqual(store2.zoom(for: "hashX"), 2.0)
    }

    func testOverwriteZoom() throws {
        let store = ZoomStore(directory: try tmpDir())
        store.setZoom(1.0, for: "h")
        store.setZoom(2.5, for: "h")
        XCTAssertEqual(store.zoom(for: "h"), 2.5)
    }
}
