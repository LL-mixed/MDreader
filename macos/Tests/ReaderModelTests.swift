import XCTest
import SwiftData
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

    func testZoomInCapsAtMax() {
        let model = ReaderModel()
        model.zoom = 2.9
        model.zoomIn()
        XCTAssertEqual(model.zoom, 3.0, accuracy: 0.001)
    }

    func testZoomOutFloorsAtMin() {
        let model = ReaderModel()
        model.zoom = 0.32
        model.zoomOut()
        XCTAssertEqual(model.zoom, 0.3, accuracy: 0.001)
    }

    func testResetZoom() {
        let model = ReaderModel()
        model.zoom = 2.0
        model.resetZoom()
        XCTAssertEqual(model.zoom, 1.0)
    }

    private func makeRepo() throws -> (DocRepository, SessionStore) {
        let config = ModelConfiguration(isStoredInMemoryOnly: true)
        let container = try ModelContainer(for: CachedDoc.self, configurations: config)
        let docsDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: docsDir, withIntermediateDirectories: true)
        let sessionDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        return (DocRepository(container: container, docsDir: docsDir), SessionStore(directory: sessionDir))
    }

    func testRestoreLastDocReopensStoredDoc() throws {
        let (repo, session) = try makeRepo()
        let model = ReaderModel(repository: repo)
        model.sessionStore = session

        let id = repo.cache(title: "Hello", markdown: "# Hello", sourceURI: nil)
        session.setLastDocID(id)
        model.refreshDocs()

        model.restoreLastDoc()
        XCTAssertEqual(model.selectedDocID, id)
        XCTAssertEqual(model.title, "Hello")
    }

    func testRestoreLastDocClearsWhenDocMissing() {
        let sessionDir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        let session = SessionStore(directory: sessionDir)
        session.setLastDocID(UUID())
        let model = ReaderModel()
        model.sessionStore = session
        model.restoreLastDoc()
        XCTAssertNil(session.lastDocID)
    }

    func testOpenCachedRecordsLastDocID() throws {
        let (repo, session) = try makeRepo()
        repo.cache(title: "Cached", markdown: "# Body", sourceURI: nil)

        let model = ReaderModel(repository: repo)
        model.sessionStore = session
        model.refreshDocs()
        let doc = try XCTUnwrap(model.docs.first)
        model.openCached(doc)
        XCTAssertEqual(session.lastDocID, doc.id)
    }

    func testOpenURLRecordsLastDocID() throws {
        let (repo, session) = try makeRepo()
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let url = dir.appendingPathComponent("note.md")
        try "# Note".write(to: url, atomically: true, encoding: .utf8)

        let model = ReaderModel(repository: repo)
        model.sessionStore = session
        model.open(url)
        XCTAssertEqual(session.lastDocID, repo.all().first?.id)
    }

    private func writeSource(_ name: String, _ body: String) throws -> URL {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        let url = dir.appendingPathComponent(name)
        try body.write(to: url, atomically: true, encoding: .utf8)
        return url
    }

    func testOpenCachedAutoRefreshesFromSource() throws {
        let (repo, session) = try makeRepo()
        let url = try writeSource("note.md", "# v1")
        let id = repo.cache(title: "note", markdown: "# v1", sourceURI: url.path)
        try "# v2".write(to: url, atomically: true, encoding: .utf8)

        let model = ReaderModel(repository: repo)
        model.sessionStore = session
        model.refreshDocs()
        let doc = try XCTUnwrap(model.docs.first(where: { $0.id == id }))
        model.openCached(doc)
        XCTAssertEqual(model.markdown, "# v2")
    }

    func testRefreshDocReloadsFromSource() throws {
        let (repo, session) = try makeRepo()
        let url = try writeSource("note.md", "# v1")
        let id = repo.cache(title: "note", markdown: "# v1", sourceURI: url.path)

        let model = ReaderModel(repository: repo)
        model.sessionStore = session
        model.refreshDocs()
        let doc = try XCTUnwrap(model.docs.first(where: { $0.id == id }))
        model.openCached(doc)
        XCTAssertEqual(model.markdown, "# v1")

        try "# v2".write(to: url, atomically: true, encoding: .utf8)
        model.refreshDoc(doc)
        XCTAssertEqual(model.markdown, "# v2")
    }
}
