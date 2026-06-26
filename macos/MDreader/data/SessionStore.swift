import Foundation

/// Persists the last-opened document id for session restore on next launch.
/// Stored at `~/.mdreader/session.json`; symmetric with `ZoomStore`.
final class SessionStore {
    let fileURL: URL
    private(set) var lastDocID: UUID?

    init(directory: URL? = nil) {
        let dir = directory ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".mdreader")
        self.fileURL = dir.appendingPathComponent("session.json")
        load()
    }

    private struct Snapshot: Codable {
        var lastDocID: UUID?
    }

    private func load() {
        guard let data = try? Data(contentsOf: fileURL),
              let snap = try? JSONDecoder().decode(Snapshot.self, from: data) else { return }
        lastDocID = snap.lastDocID
    }

    func setLastDocID(_ id: UUID?) {
        lastDocID = id
        save()
    }

    private func save() {
        try? FileManager.default.createDirectory(at: fileURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        guard let data = try? JSONEncoder().encode(Snapshot(lastDocID: lastDocID)) else { return }
        try? data.write(to: fileURL, options: .atomic)
    }
}
