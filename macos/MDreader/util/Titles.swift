import Foundation

enum Titles {
    private static let markdownExtensions: Set<String> = ["md", "markdown", "mdown", "mkd", "mkdown"]

    static func fromPath(_ path: String) -> String {
        if path.isEmpty { return "" }
        let slashIdx = [path.lastIndex(of: "/"), path.lastIndex(of: "\\")].compactMap { $0 }.max()
        let nameStart: String.Index
        if let s = slashIdx {
            nameStart = path.index(after: s)
        } else {
            nameStart = path.startIndex
        }
        let name = String(path[nameStart...])
        guard let dot = name.lastIndex(of: ".") else { return name }
        let dotOffset = name.distance(from: name.startIndex, to: dot)
        if dotOffset <= 0 { return name }
        let ext = String(name[name.index(after: dot)...]).lowercased()
        if markdownExtensions.contains(ext) {
            return String(name[..<dot])
        }
        return name
    }
}
