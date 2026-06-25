import Foundation

enum DocStore {
    static func fileURL(docsDir: URL, id: UUID) -> URL {
        docsDir.appendingPathComponent("\(id.uuidString).md")
    }

    static func write(docsDir: URL, id: UUID, markdown: String) {
        try? FileManager.default.createDirectory(at: docsDir, withIntermediateDirectories: true)
        try? markdown.write(to: fileURL(docsDir: docsDir, id: id), atomically: true, encoding: .utf8)
    }

    static func read(docsDir: URL, id: UUID) -> String? {
        try? String(contentsOf: fileURL(docsDir: docsDir, id: id), encoding: .utf8)
    }

    static func delete(docsDir: URL, id: UUID) {
        try? FileManager.default.removeItem(at: fileURL(docsDir: docsDir, id: id))
    }
}
