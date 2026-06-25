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
                zoom: model.zoom,
                scrollRequest: model.scrollRequest,
                onDropText: { model.openText($0, named: $1) },
                onOutline: { model.outline = $0 },
                onActiveHeading: { model.activeHeadingIndex = $0 }
            )
            .navigationTitle(model.title)
            .toolbar {
                ToolbarItemGroup(placement: .primaryAction) {
                    Button {
                        model.zoomOut()
                    } label: {
                        Image(systemName: "minus")
                    }
                    .keyboardShortcut("-", modifiers: .command)
                    .disabled(model.zoom <= 0.3)

                    Text("\(Int((model.zoom * 100).rounded()))%")
                        .monospacedDigit()
                        .frame(minWidth: 44)
                        .foregroundStyle(.secondary)

                    Button {
                        model.zoomIn()
                    } label: {
                        Image(systemName: "plus")
                    }
                    .keyboardShortcut("+", modifiers: .command)
                    .disabled(model.zoom >= 3.0)

                    Button {
                        model.resetZoom()
                    } label: {
                        Image(systemName: "1.magnifyingglass")
                    }
                    .keyboardShortcut("0", modifiers: .command)
                    .disabled(model.zoom == 1.0)

                    Divider()

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
