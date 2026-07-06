import Foundation

/// Global default theme preference. Persisted in `AppSettings`. The per-doc override (ThemeStore)
/// only ever stores an explicit Light/Dark, never "system" — once a user toggles a doc, it sticks.
enum ThemePref: String, Codable, CaseIterable {
    case system, light, dark
}

struct AppSettings: Codable, Equatable {
    var editorCommand: String = ""
    var themePref: ThemePref = .system

    init() {}

    /// Tolerant decode: a config from an older version (no `themePref`) or a newer one (unknown
    /// value) must not break launch — each field falls back to its default.
    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        editorCommand = (try? c.decode(String.self, forKey: .editorCommand)) ?? ""
        if let p = try? c.decode(ThemePref.self, forKey: .themePref) {
            themePref = p
        } else {
            themePref = .system
        }
    }
}

final class SettingsStore: ObservableObject {
    @Published var settings: AppSettings {
        didSet { save() }
    }
    let fileURL: URL

    init(directory: URL? = nil) {
        let dir = directory ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".mdreader")
        self.fileURL = dir.appendingPathComponent("config.json")
        if let data = try? Data(contentsOf: fileURL),
           let decoded = try? JSONDecoder().decode(AppSettings.self, from: data) {
            settings = decoded
        } else {
            settings = AppSettings()
        }
    }

    private func save() {
        try? FileManager.default.createDirectory(at: fileURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        guard let data = try? JSONEncoder().encode(settings) else { return }
        try? data.write(to: fileURL, options: .atomic)
    }
}
