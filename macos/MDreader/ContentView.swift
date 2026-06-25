import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var model: ReaderModel

    var body: some View {
        NavigationSplitView {
            SidebarView()
        } detail: {
            MarkdownWebView(
                markdown: model.markdown,
                isDark: model.isDark,
                scrollRequest: model.scrollRequest,
                onDropText: { model.openText($0, named: $1) },
                onOutline: { model.outline = $0 },
                onActiveHeading: { model.activeHeadingIndex = $0 }
            )
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
}
