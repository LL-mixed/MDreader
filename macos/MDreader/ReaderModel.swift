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
    var repository: DocRepository?

    init(repository: DocRepository? = nil) {
        self.repository = repository
    }

    func loadSample() {
        markdown = Self.sampleMarkdown
        title = "MDreader"
        resetOutline()
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
        openText(text, named: url.lastPathComponent, sourceURI: url.path)
    }

    func openText(_ text: String, named: String, sourceURI: String? = nil) {
        markdown = text
        title = (named as NSString).deletingPathExtension
        repository?.cache(title: title, markdown: text, sourceURI: sourceURI)
        refreshDocs()
        resetOutline()
    }

    func openCached(_ doc: DocInfo) {
        guard let text = repository?.loadContent(id: doc.id) else { return }
        markdown = text
        title = doc.title
        selectedDocID = doc.id
        resetOutline()
    }

    func deleteDoc(id: UUID) {
        repository?.delete(id: id)
        if selectedDocID == id { selectedDocID = nil }
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

    private func resetOutline() {
        outline = []
        activeHeadingIndex = nil
        scrollRequest = nil
    }
}
