import Foundation

final class ReaderModel: ObservableObject {
    @Published var markdown: String = ""
    @Published var isDark: Bool = false
    @Published var title: String = "MDreader"

    static let sampleMarkdown: String = {
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "md", subdirectory: "shared"),
              let text = try? String(contentsOf: url, encoding: .utf8) else {
            return "# MDreader\n\n无法加载样例文档。"
        }
        return text
    }()

    func loadSample() {
        markdown = Self.sampleMarkdown
        title = "MDreader"
    }

    func open(_ url: URL) {
        guard let text = try? String(contentsOf: url, encoding: .utf8) else { return }
        markdown = text
        title = url.deletingPathExtension().lastPathComponent
    }
}
