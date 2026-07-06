import SwiftUI
import SwiftData

@main
struct MDreaderApp: App {
    @StateObject private var settingsStore: SettingsStore
    let repository: DocRepository
    let zoomStore: ZoomStore
    let themeStore: ThemeStore
    let sessionStore: SessionStore

    init() {
        let container: ModelContainer
        do {
            container = try ModelContainer(for: CachedDoc.self)
        } catch {
            fatalError("Failed to create ModelContainer: \(error)")
        }
        let settings = SettingsStore()
        repository = DocRepository(container: container)
        zoomStore = ZoomStore()
        themeStore = ThemeStore()
        sessionStore = SessionStore()
        _settingsStore = StateObject(wrappedValue: settings)
    }

    var body: some Scene {
        WindowGroup(for: UUID.self) { $docID in
            ContentView(initialDocID: docID)
                .environmentObject(settingsStore)
                .environment(\.mdRepository, repository)
                .environment(\.mdZoomStore, zoomStore)
                .environment(\.mdThemeStore, themeStore)
                .environment(\.mdSessionStore, sessionStore)
        }
        .defaultSize(width: 1000, height: 640)
        .commands {
            CommandGroup(replacing: .appInfo) {
                Button("关于 MDreader") {
                    AboutWindow.show()
                }
            }
        }
        Settings {
            SettingsView()
                .environmentObject(settingsStore)
        }
    }
}
