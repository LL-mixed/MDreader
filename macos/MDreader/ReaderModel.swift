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
    var repository: DocRepository?

    init(repository: DocRepository? = nil) {
        self.repository = repository
    }

    func loadSample() {
        markdown = Self.sampleMarkdown
        title = "MDreader"
    }

    func open(_ url: URL) {
        guard let text = try? String(contentsOf: url, encoding: .utf8) else { return }
        openText(text, named: url.lastPathComponent, sourceURI: url.path)
    }

    func openText(_ text: String, named: String, sourceURI: String? = nil) {
        markdown = text
        title = (named as NSString).deletingPathExtension
        repository?.cache(title: title, markdown: text, sourceURI: sourceURI)
    }
}
