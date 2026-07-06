import Foundation

/// Per-content-hash dark override map, persisted to `~/.mdreader/theme.json`. Symmetric with
/// `ZoomStore`. A doc lands here only when the user toggles its theme; absence means "follow the
/// global default" (AppSettings.themePref).
final class ThemeStore {
    let fileURL: URL
    private(set) var map: [String: Bool] = [:]

    init(directory: URL? = nil) {
        let dir = directory ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".mdreader")
        self.fileURL = dir.appendingPathComponent("theme.json")
        load()
    }

    private func load() {
        guard let data = try? Data(contentsOf: fileURL),
              let decoded = try? JSONDecoder().decode([String: Bool].self, from: data) else { return }
        map = decoded
    }

    func isDark(forHash hash: String) -> Bool? {
        map[hash]
    }

    func setDark(_ dark: Bool, forHash hash: String) {
        map[hash] = dark
        save()
    }

    private func save() {
        try? FileManager.default.createDirectory(at: fileURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        guard let data = try? JSONEncoder().encode(map) else { return }
        try? data.write(to: fileURL, options: .atomic)
    }
}
