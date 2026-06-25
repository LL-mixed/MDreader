import Foundation

struct AppSettings: Codable, Equatable {
    var editorCommand: String = ""
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
