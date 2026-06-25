import SwiftUI
import SwiftData

@main
struct MDreaderApp: App {
    @StateObject private var model: ReaderModel

    init() {
        let container: ModelContainer
        do {
            container = try ModelContainer(for: CachedDoc.self)
        } catch {
            fatalError("Failed to create ModelContainer: \(error)")
        }
        let repo = DocRepository(container: container)
        _model = StateObject(wrappedValue: ReaderModel(repository: repo))
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(model)
                .onOpenURL { url in model.open(url) }
        }
        .defaultSize(width: 900, height: 640)
    }
}
