import SwiftUI
import AppKit

struct ContentView: View {
    let initialDocID: UUID?
    @Environment(\.mdRepository) private var repository
    @Environment(\.mdZoomStore) private var zoomStore
    @EnvironmentObject private var settingsStore: SettingsStore
    @StateObject private var model = ReaderModel()

    var body: some View {
        NavigationSplitView {
            SidebarView()
        } detail: {
            MarkdownWebView(
                markdown: model.markdown,
                isDark: model.isDark,
                zoom: model.zoom,
                scrollRequest: model.scrollRequest,
                exportRequest: model.exportRequest,
                onDropText: { model.openText($0, named: $1) },
                onOutline: { model.outline = $0 },
                onActiveHeading: { model.activeHeadingIndex = $0 },
                onCommandScroll: { delta in
                    if delta > 0 { model.zoomIn() } else { model.zoomOut() }
                }
            )
            .navigationTitle(model.title)
            .toolbar {
                ToolbarItemGroup(placement: .primaryAction) {
                    Button { model.zoomOut() } label: { Image(systemName: "minus") }
                        .keyboardShortcut("-", modifiers: .command)
                        .disabled(model.zoom <= 0.3)
                    Text("\(Int((model.zoom * 100).rounded()))%")
                        .monospacedDigit()
                        .frame(minWidth: 44)
                        .foregroundStyle(.secondary)
                    Button { model.zoomIn() } label: { Image(systemName: "plus") }
                        .keyboardShortcut("+", modifiers: .command)
                        .disabled(model.zoom >= 3.0)
                    Button { model.resetZoom() } label: { Image(systemName: "1.magnifyingglass") }
                        .keyboardShortcut("0", modifiers: .command)
                        .disabled(model.zoom == 1.0)

                    Divider()

                    Button { model.exportPDF() } label: {
                        Label("导出 PDF", systemImage: "square.and.arrow.down")
                    }
                    Button { model.editCurrent() } label: {
                        Label("编辑", systemImage: "pencil")
                    }
                    .disabled(!model.canEdit)

                    Divider()

                    Button { model.isDark.toggle() } label: {
                        Label(model.isDark ? "浅色" : "深色", systemImage: model.isDark ? "sun.max" : "moon")
                    }
                }
            }
        }
        .environmentObject(model)
        .onAppear {
            model.repository = repository
            model.zoomStore = zoomStore
            model.settings = settingsStore
            model.refreshDocs()
            if let id = initialDocID, let doc = model.docs.first(where: { $0.id == id }) {
                model.openCached(doc)
            }
            configureTabbing()
        }
        .onOpenURL { url in model.open(url) }
    }

    private func configureTabbing() {
        for window in NSApp.windows {
            window.tabbingMode = .preferred
            window.tabbingIdentifier = "com.mdreader.main"
        }
    }
}
