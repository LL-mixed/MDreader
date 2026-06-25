import SwiftUI

@main
struct MDreaderApp: App {
    @StateObject private var model = ReaderModel()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(model)
                .onOpenURL { url in model.open(url) }
        }
        .defaultSize(width: 900, height: 640)
    }
}
