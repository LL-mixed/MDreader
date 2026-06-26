import XCTest
@testable import MDreader

final class SessionStoreTest: XCTestCase {
    private func makeDir() throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    func testStartsNil() throws {
        let store = SessionStore(directory: try makeDir())
        XCTAssertNil(store.lastDocID)
    }

    func testSetAndGetSameInstance() throws {
        let store = SessionStore(directory: try makeDir())
        let id = UUID()
        store.setLastDocID(id)
        XCTAssertEqual(store.lastDocID, id)
    }

    func testPersistsAcrossInstances() throws {
        let dir = try makeDir()
        let id = UUID()
        let writer = SessionStore(directory: dir)
        writer.setLastDocID(id)
        let reader = SessionStore(directory: dir)
        XCTAssertEqual(reader.lastDocID, id)
    }

    func testClearWithNilPersists() throws {
        let dir = try makeDir()
        let store = SessionStore(directory: dir)
        store.setLastDocID(UUID())
        store.setLastDocID(nil)
        XCTAssertNil(store.lastDocID)
        let reader = SessionStore(directory: dir)
        XCTAssertNil(reader.lastDocID)
    }
}
