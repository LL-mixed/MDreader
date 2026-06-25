import SwiftUI

struct ContentView: View {
    @State private var isDark = false

    private let sampleMarkdown: String = {
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "md", subdirectory: "shared"),
              let text = try? String(contentsOf: url, encoding: .utf8) else {
            return "# MDreader\n\n无法加载样例文档。"
        }
        return text
    }()

    var body: some View {
        MarkdownWebView(markdown: sampleMarkdown, isDark: isDark)
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        isDark.toggle()
                    } label: {
                        Label(isDark ? "浅色" : "深色", systemImage: isDark ? "sun.max" : "moon")
                    }
                }
            }
    }
}
