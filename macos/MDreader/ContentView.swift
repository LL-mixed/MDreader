import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var model: ReaderModel

    var body: some View {
        MarkdownWebView(markdown: model.markdown, isDark: model.isDark, onDropText: { model.openText($0, named: $1) })
            .navigationTitle(model.title)
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        model.isDark.toggle()
                    } label: {
                        Label(model.isDark ? "浅色" : "深色", systemImage: model.isDark ? "sun.max" : "moon")
                    }
                }
            }
    }
}
