import XCTest
import SwiftData
@testable import MDreader

final class DocRepositoryTests: XCTestCase {
    private func makeRepo() throws -> (DocRepository, URL) {
        let config = ModelConfiguration(isStoredInMemoryOnly: true)
        let container = try ModelContainer(for: CachedDoc.self, configurations: config)
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return (DocRepository(container: container, docsDir: dir), dir)
    }

    func testCacheInsertsOnceAndWritesFile() throws {
        let (repo, dir) = try makeRepo()
        repo.cache(title: "Doc", markdown: "# Hi", sourceURI: nil)
        repo.cache(title: "Doc Again", markdown: "# Hi", sourceURI: nil)
        XCTAssertEqual(repo.all().count, 1)
        let doc = try XCTUnwrap(repo.all().first)
        XCTAssertEqual(DocStore.read(docsDir: dir, id: doc.id), "# Hi")
    }

    func testCacheDifferentContentSeparateRows() throws {
        let (repo, _) = try makeRepo()
        repo.cache(title: "A", markdown: "aaa", sourceURI: nil)
        repo.cache(title: "B", markdown: "bbb", sourceURI: nil)
        XCTAssertEqual(repo.all().count, 2)
    }

    func testEmptyTitleGetsDefault() throws {
        let (repo, _) = try makeRepo()
        repo.cache(title: "", markdown: "x", sourceURI: nil)
        XCTAssertEqual(try XCTUnwrap(repo.all().first).title, DocRepository.defaultTitle)
    }

    func testDeleteRemovesRowAndFile() throws {
        let (repo, dir) = try makeRepo()
        repo.cache(title: "T", markdown: "body", sourceURI: nil)
        let doc = try XCTUnwrap(repo.all().first)
        let id = doc.id
        repo.delete(id: id)
        XCTAssertTrue(repo.all().isEmpty)
        XCTAssertNil(DocStore.read(docsDir: dir, id: id))
    }

    func testCacheReturnsStableIDForSameContent() throws {
        let (repo, _) = try makeRepo()
        let id1 = repo.cache(title: "Doc", markdown: "# Hi", sourceURI: nil)
        let id2 = repo.cache(title: "Doc Again", markdown: "# Hi", sourceURI: nil)
        XCTAssertEqual(id1, id2)
        XCTAssertEqual(try XCTUnwrap(repo.all().first).id, id1)
    }

    private func writeSource(_ name: String, _ body: String) throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let url = dir.appendingPathComponent(name)
        try body.write(to: url, atomically: true, encoding: .utf8)
        return url
    }

    func testRefreshFromSourceUpdatesChangedContent() throws {
        let (repo, _) = try makeRepo()
        let url = try writeSource("note.md", "# v1")
        let id = repo.cache(title: "note", markdown: "# v1", sourceURI: url.path)
        XCTAssertEqual(repo.loadContent(id: id), "# v1")

        try "# v2".write(to: url, atomically: true, encoding: .utf8)
        let refreshed = repo.refreshFromSource(id: id)
        XCTAssertTrue(refreshed)
        XCTAssertEqual(repo.loadContent(id: id), "# v2")
    }

    func testRefreshFromSourceNoopWhenUnchanged() throws {
        let (repo, _) = try makeRepo()
        let url = try writeSource("note.md", "# same")
        let id = repo.cache(title: "note", markdown: "# same", sourceURI: url.path)
        XCTAssertFalse(repo.refreshFromSource(id: id))
        XCTAssertEqual(repo.loadContent(id: id), "# same")
    }

    func testRefreshFromSourceFalseWhenNoSource() throws {
        let (repo, _) = try makeRepo()
        let id = repo.cache(title: "note", markdown: "# x", sourceURI: nil)
        XCTAssertFalse(repo.refreshFromSource(id: id))
    }

    func testRefreshFromSourceFalseWhenSourceMissing() throws {
        let (repo, _) = try makeRepo()
        let id = repo.cache(title: "note", markdown: "# x", sourceURI: "/nonexistent/path-\(UUID().uuidString).md")
        XCTAssertFalse(repo.refreshFromSource(id: id))
    }
}
