import XCTest
@testable import MDreader

final class SettingsStoreTest: XCTestCase {
    private func tmpDir() throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    func testDefaultsWhenAbsent() throws {
        let store = SettingsStore(directory: try tmpDir())
        XCTAssertEqual(store.settings.editorCommand, "")
    }

    func testWriteAndReload() throws {
        let dir = try tmpDir()
        let store1 = SettingsStore(directory: dir)
        store1.settings.editorCommand = "code"
        let store2 = SettingsStore(directory: dir)
        XCTAssertEqual(store2.settings.editorCommand, "code")
    }
}
