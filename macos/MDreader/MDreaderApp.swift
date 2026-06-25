import SwiftUI
import SwiftData

@main
struct MDreaderApp: App {
    @StateObject private var model: ReaderModel
    @StateObject private var settingsStore: SettingsStore

    init() {
        let container: ModelContainer
        do {
            container = try ModelContainer(for: CachedDoc.self)
        } catch {
            fatalError("Failed to create ModelContainer: \(error)")
        }
        let repo = DocRepository(container: container)
        let settings = SettingsStore()
        let zoomStore = ZoomStore()
        let readerModel = ReaderModel(repository: repo)
        readerModel.zoomStore = zoomStore
        readerModel.settings = settings
        _model = StateObject(wrappedValue: readerModel)
        _settingsStore = StateObject(wrappedValue: settings)
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(model)
                .environmentObject(settingsStore)
                .onOpenURL { url in model.open(url) }
                .onAppear { model.refreshDocs() }
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
