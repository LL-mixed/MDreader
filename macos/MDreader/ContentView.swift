import SwiftUI
import AppKit

struct ContentView: View {
    let initialDocID: UUID?
    @Environment(\.mdRepository) private var repository
    @Environment(\.mdZoomStore) private var zoomStore
    @Environment(\.mdThemeStore) private var themeStore
    @Environment(\.mdSessionStore) private var sessionStore
    @EnvironmentObject private var settingsStore: SettingsStore
    @Environment(\.colorScheme) private var systemColorScheme
    @StateObject private var model = ReaderModel()

    var body: some View {
        NavigationSplitView {
            SidebarView()
        } detail: {
            ZStack(alignment: .topLeading) {
                MarkdownWebView(
                    markdown: model.markdown,
                    isDark: model.isDark,
                    baseDir: model.currentSourceURL?.deletingLastPathComponent(),
                    zoom: model.zoom,
                    scrollRequest: model.scrollRequest,
                    exportRequest: model.exportRequest,
                    returnRequest: model.returnRequest,
                    onDropText: { model.openText($0, named: $1) },
                    onOutline: { model.outline = $0 },
                    onActiveHeading: { model.activeHeadingIndex = $0 },
                    onCommandScroll: { delta in
                        if delta > 0 { model.zoomIn() } else { model.zoomOut() }
                    },
                    onNavigatedAway: { model.navigatedAway = $0 }
                )

                if model.navigatedAway {
                Button {
                    model.goBackToDocument()
                } label: {
                    Label("返回文档", systemImage: "chevron.left")
                }
                .buttonStyle(.borderedProminent)
                .padding(10)
                }
            }
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

                    Button { model.toggleTheme() } label: {
                        Label(model.isDark ? "浅色" : "深色", systemImage: model.isDark ? "sun.max" : "moon")
                    }
                }
            }
        }
        // Chrome tracks the rendered doc's dark flag (model.isDark), so the window chrome stays in
        // sync with the body — pinned docs recolor the whole window, not just the WebView.
        .preferredColorScheme(model.isDark ? .dark : .light)
        .environmentObject(model)
        .onAppear {
            model.repository = repository
            model.zoomStore = zoomStore
            model.themeStore = themeStore
            model.sessionStore = sessionStore
            model.settings = settingsStore
            model.systemDark = (systemColorScheme == .dark)
            model.refreshDocs()
            if let id = initialDocID, let doc = model.docs.first(where: { $0.id == id }) {
                model.openCached(doc)
            } else if initialDocID == nil {
                model.restoreLastDoc()
            }
            configureTabbing()
        }
        .onChange(of: systemColorScheme) { _, newScheme in
            model.setSystemDark(newScheme == .dark)
        }
        .onChange(of: settingsStore.settings.themePref) { _, _ in
            model.reapplyThemeIfUnpinned()
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
