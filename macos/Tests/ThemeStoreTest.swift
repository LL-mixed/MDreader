import XCTest
@testable import MDreader

final class ThemeStoreTest: XCTestCase {
    private func tmpDir() throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    func testSetAndGetDark() throws {
        let store = ThemeStore(directory: try tmpDir())
        store.setDark(true, forHash: "hashA")
        store.setDark(false, forHash: "hashB")
        XCTAssertEqual(store.isDark(forHash: "hashA"), true)
        XCTAssertEqual(store.isDark(forHash: "hashB"), false)
        XCTAssertNil(store.isDark(forHash: "hashC"))
    }

    func testPersistAcrossInstances() throws {
        let dir = try tmpDir()
        let store1 = ThemeStore(directory: dir)
        store1.setDark(true, forHash: "hashX")
        let store2 = ThemeStore(directory: dir)
        XCTAssertEqual(store2.isDark(forHash: "hashX"), true)
    }

    func testOverwriteDark() throws {
        let store = ThemeStore(directory: try tmpDir())
        store.setDark(true, forHash: "h")
        store.setDark(false, forHash: "h")
        XCTAssertEqual(store.isDark(forHash: "h"), false)
    }
}
