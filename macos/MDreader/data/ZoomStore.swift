import Foundation

final class ZoomStore {
    let fileURL: URL
    private(set) var map: [String: Double] = [:]

    init(directory: URL? = nil) {
        let dir = directory ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".mdreader")
        self.fileURL = dir.appendingPathComponent("zoom.json")
        load()
    }

    private func load() {
        guard let data = try? Data(contentsOf: fileURL),
              let decoded = try? JSONDecoder().decode([String: Double].self, from: data) else { return }
        map = decoded
    }

    func zoom(for hash: String) -> Double? {
        map[hash]
    }

    func setZoom(_ zoom: Double, for hash: String) {
        map[hash] = zoom
        save()
    }

    private func save() {
        try? FileManager.default.createDirectory(at: fileURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        guard let data = try? JSONEncoder().encode(map) else { return }
        try? data.write(to: fileURL, options: .atomic)
    }
}
