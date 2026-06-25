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
}
