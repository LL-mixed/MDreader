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

    func testThemePrefDefaultsToSystem() throws {
        let store = SettingsStore(directory: try tmpDir())
        XCTAssertEqual(store.settings.themePref, .system)
    }

    func testThemePrefWriteAndReload() throws {
        let dir = try tmpDir()
        let store1 = SettingsStore(directory: dir)
        store1.settings.themePref = .dark
        let store2 = SettingsStore(directory: dir)
        XCTAssertEqual(store2.settings.themePref, .dark)
    }

    func testOldConfigWithoutThemePrefStillLoads() throws {
        // A config predating themePref must not break decode (and yields .system), so existing
        // users don't lose their editorCommand on upgrade.
        let dir = try tmpDir()
        let url = dir.appendingPathComponent("config.json")
        try #"{"editorCommand":"Typora"}"#.write(to: url, atomically: true, encoding: .utf8)
        let store = SettingsStore(directory: dir)
        XCTAssertEqual(store.settings.editorCommand, "Typora")
        XCTAssertEqual(store.settings.themePref, .system)
    }
}
