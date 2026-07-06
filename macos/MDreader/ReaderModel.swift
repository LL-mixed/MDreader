import Foundation

final class ReaderModel: ObservableObject {
    static let sampleMarkdown: String = {
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "md", subdirectory: "shared"),
              let text = try? String(contentsOf: url, encoding: .utf8) else {
            return "# MDreader\n\n无法加载样例文档。"
        }
        return text
    }()

    @Published var markdown: String = ReaderModel.sampleMarkdown
    @Published var isDark: Bool = false
    @Published var title: String = "MDreader"
    @Published var docs: [DocInfo] = []
    @Published var query: String = ""
    @Published var selectedDocID: UUID?
    @Published var outline: [OutlineItem] = []
    @Published var activeHeadingIndex: Int? = nil
    @Published var scrollRequest: Int? = nil
    @Published var zoom: Double = 1.0
    @Published var currentSourceURL: URL? = nil {
        willSet {
            // Tear down any watcher bound to the previous source before swapping in
            // a new one (or nil). The new watcher is rebuilt by the open paths.
            if newValue?.path != currentSourceURL?.path { teardownWatcher() }
        }
    }
    @Published var exportRequest: Int = 0
    @Published var returnRequest: Int = 0
    @Published var navigatedAway: Bool = false

    /// UUID of the cached doc backing whatever is currently displayed, regardless
    /// of how it was opened (file, drag-drop, or library). `nil` for the bundled
    /// sample or drag-dropped text with no source. Drives source-change tracking.
    private(set) var currentDocID: UUID?
    var repository: DocRepository?
    var zoomStore: ZoomStore?
    var themeStore: ThemeStore?
    var sessionStore: SessionStore?
    var settings: SettingsStore?
    /// Live OS color scheme, pushed in by ContentView; used only when the global pref is `.system`
    /// and the current doc has no per-doc override.
    var systemDark: Bool = false

    /// Per-window source watcher; observes `currentSourceURL` for external edits.
    private var sourceWatcher: SourceFileWatcher?

    /// Builds the watcher for a given path. Overridable so tests can disable live
    /// file watching (the production default arms a real `DispatchSource`).
    var sourceWatcherFactory: (String) -> SourceFileWatcher? = { SourceFileWatcher(path: $0) }

    init(repository: DocRepository? = nil) {
        self.repository = repository
    }

    deinit {
        // `SourceFileWatcher` hops to its own queue on cancel, so this is safe from
        // any thread (e.g. the SwiftUI teardown path).
        sourceWatcher?.cancel()
    }

    func loadSample() {
        markdown = Self.sampleMarkdown
        title = "MDreader"
        currentDocID = nil
        currentSourceURL = nil // willSet tears down the watcher
        resetOutline()
        restoreZoom()
        restoreTheme()
    }

    func refreshDocs() {
        docs = repository?.all() ?? []
    }

    var filteredDocs: [DocInfo] {
        let q = query.lowercased()
        guard !q.isEmpty else { return docs }
        return docs.filter { $0.title.lowercased().contains(q) }
    }

    func open(_ url: URL) {
        guard let text = try? String(contentsOf: url, encoding: .utf8) else { return }
        applyText(text, named: url.lastPathComponent, sourceURI: url.path, sourceURL: url)
    }

    func openText(_ text: String, named: String, sourceURI: String? = nil) {
        let ext = (named as NSString).pathExtension.lowercased()
        let allowed: Set<String> = ["md", "markdown", "mdown", "mkd", "mkdown"]
        guard allowed.contains(ext) else { return }
        applyText(text, named: named, sourceURI: sourceURI, sourceURL: nil)
    }

    private func applyText(_ text: String, named: String, sourceURI: String?, sourceURL: URL?) {
        markdown = text
        title = (named as NSString).deletingPathExtension
        let cachedID = repository?.cache(title: title, markdown: text, sourceURI: sourceURI)
        currentDocID = cachedID
        if let cachedID { sessionStore?.setLastDocID(cachedID) }
        // Set the URL only after currentDocID is known, so a watcher built from it
        // can resolve the cache UUID. The willSet on currentSourceURL tears down
        // the previous watcher; rebuildWatcherIfNeeded() arms the new one.
        currentSourceURL = sourceURL
        refreshDocs()
        resetOutline()
        restoreZoom()
        rebuildWatcherIfNeeded()
        restoreTheme()
    }

    func openCached(_ doc: DocInfo) {
        let refreshed = repository?.refreshFromSource(id: doc.id) ?? false
        guard let text = repository?.loadContent(id: doc.id) else { return }
        markdown = text
        title = doc.title
        selectedDocID = doc.id
        currentDocID = doc.id
        currentSourceURL = doc.sourceURI.flatMap {
            FileManager.default.fileExists(atPath: $0) ? URL(fileURLWithPath: $0) : nil
        }
        resetOutline()
        restoreZoom()
        restoreTheme()
        sessionStore?.setLastDocID(doc.id)
        if refreshed { refreshDocs() }
        rebuildWatcherIfNeeded()
    }

    /// Manually forces a re-read of the original file for `doc`, then displays it.
    func refreshDoc(_ doc: DocInfo) {
        repository?.refreshFromSource(id: doc.id)
        refreshDocs()
        if let latest = docs.first(where: { $0.id == doc.id }) {
            openCached(latest)
        }
    }

    /// Restores the last-opened document on a normal launch; clears the record if
    /// the stored doc no longer exists. Invoked only when no explicit doc is requested.
    func restoreLastDoc() {
        guard let id = sessionStore?.lastDocID else { return }
        if let doc = docs.first(where: { $0.id == id }) {
            openCached(doc)
        } else {
            sessionStore?.setLastDocID(nil)
        }
    }

    func deleteDoc(id: UUID) {
        repository?.delete(id: id)
        if selectedDocID == id { selectedDocID = nil }
        if currentDocID == id {
            currentDocID = nil
            currentSourceURL = nil // willSet tears down the watcher
        }
        refreshDocs()
    }

    func toggleFavorite(id: UUID) {
        guard let doc = docs.first(where: { $0.id == id }) else { return }
        repository?.setFavorite(id: id, favorite: !doc.favorite)
        refreshDocs()
    }

    func jumpToHeading(_ index: Int) {
        scrollRequest = index
    }

    func zoomIn() { zoom = min(zoom * 1.1, 3.0); saveZoom() }
    func zoomOut() { zoom = max(zoom / 1.1, 0.3); saveZoom() }
    func resetZoom() { zoom = 1.0; saveZoom() }

    func exportPDF() {
        exportRequest += 1
    }

    func goBackToDocument() {
        returnRequest += 1
    }

    func editCurrent() {
        guard let url = currentSourceURL,
              let cmd = settings?.settings.editorCommand,
              !cmd.isEmpty else { return }
        let process = Process()
        process.launchPath = "/bin/sh"
        process.arguments = ["-c", "open -a \"\(cmd)\" \"\(url.path)\""]
        try? process.run()
    }

    var canEdit: Bool {
        currentSourceURL != nil && !(settings?.settings.editorCommand.isEmpty ?? true)
    }

    // MARK: - Source change tracking

    /// Arms a watcher on `currentSourceURL` (if any). Called from the open paths.
    private func rebuildWatcherIfNeeded() {
        teardownWatcher()
        guard let url = currentSourceURL else { return }
        guard let watcher = sourceWatcherFactory(url.path) else { return }
        watcher.onChange = { [weak self] in self?.sourceDidChange() }
        watcher.onCancel = { [weak self] in
            // Source vanished (deleted/moved away). Drop the watcher so we don't
            // leak; the displayed snapshot stays as-is.
            DispatchQueue.main.async { self?.sourceWatcher = nil }
        }
        sourceWatcher = watcher
    }

    private func teardownWatcher() {
        sourceWatcher?.cancel()
        sourceWatcher = nil
    }

    /// Invoked on the watcher's queue when the current source file changes on disk.
    /// Re-reads via the (idempotent, hash-gated) `refreshFromSource`, then hops to
    /// main to update the displayed content without disturbing scroll/zoom/outline.
    private func sourceDidChange() {
        guard let id = currentDocID, let repo = repository else { return }
        guard repo.refreshFromSource(id: id) else { return } // unchanged / unreadable
        DispatchQueue.main.async { [weak self] in
            guard let self, self.currentDocID == id,
                  let text = repo.loadContent(id: id) else { return }
            self.markdown = text
            self.resetOutline()
            self.refreshDocs()
        }
    }

    private func restoreZoom() {
        let hash = ContentHash.sha256Hex(markdown)
        zoom = zoomStore?.zoom(for: hash) ?? 1.0
    }

    private func saveZoom() {
        guard let store = zoomStore else { return }
        let hash = ContentHash.sha256Hex(markdown)
        store.setZoom(zoom, for: hash)
    }

    // MARK: - Theme

    /// Pure decision rule (unit-tested): a per-doc override wins; otherwise the global default,
    /// where `.system` follows the OS color scheme.
    static func resolveDark(perDoc: Bool?, pref: ThemePref, systemDark: Bool) -> Bool {
        if let d = perDoc { return d }
        switch pref {
        case .system: return systemDark
        case .light: return false
        case .dark: return true
        }
    }

    private var currentThemePref: ThemePref {
        settings?.settings.themePref ?? .system
    }

    /// Recompute isDark from persisted state. Call after any content change (open/drop/sample) so
    /// the displayed theme matches the decision rule.
    func restoreTheme() {
        let hash = ContentHash.sha256Hex(markdown)
        let perDoc = themeStore?.isDark(forHash: hash)
        isDark = Self.resolveDark(perDoc: perDoc, pref: currentThemePref, systemDark: systemDark)
    }

    /// Toolbar toggle: flip this doc's theme and persist the override so it sticks on reopen.
    func toggleTheme() {
        let newDark = !isDark
        let hash = ContentHash.sha256Hex(markdown)
        if !hash.isEmpty {
            themeStore?.setDark(newDark, forHash: hash)
        }
        isDark = newDark
    }

    /// Push the live OS color scheme into the model; docs without a per-doc override re-follow it.
    func setSystemDark(_ dark: Bool) {
        systemDark = dark
        reapplyThemeIfUnpinned()
    }

    /// When the global pref changes, docs WITHOUT an override re-follow; docs WITH one keep it.
    func reapplyThemeIfUnpinned() {
        let hash = ContentHash.sha256Hex(markdown)
        if themeStore?.isDark(forHash: hash) != nil { return }
        isDark = Self.resolveDark(perDoc: nil, pref: currentThemePref, systemDark: systemDark)
    }

    private func resetOutline() {
        outline = []
        activeHeadingIndex = nil
        scrollRequest = nil
    }
}
