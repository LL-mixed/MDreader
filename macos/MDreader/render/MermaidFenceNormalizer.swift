import Foundation

enum MermaidFenceNormalizer {
    private static let alias: Set<String> = [
        "mermaid", "sequence", "sequencediagram", "flow", "flowchart", "gantt",
        "class", "classdiagram", "state", "statediagram", "er", "erdiagram",
        "journey", "pie", "gitgraph", "mindmap", "timeline", "requirement",
        "requirementdiagram", "c4context", "c4container", "c4component", "packet", "kanban",
    ]

    private static let keyword: NSRegularExpression = {
        let pattern = "^(graph|flowchart|sequenceDiagram|classDiagram|stateDiagram(-v2)?|erDiagram|gantt|pie|journey|gitGraph|requirementDiagram|requirement|C4Context|C4Container|C4Component|C4Dynamic|C4Deployment|mindmap|timeline|quadrantChart|xychart-beta|sankey-beta|block-beta|architecture-beta|packet|kanban)\\b"
        return try! NSRegularExpression(pattern: pattern, options: [])
    }()

    static func normalize(_ markdown: String) -> String {
        if markdown.isEmpty { return markdown }
        var lines = markdown.components(separatedBy: "\n")
        var i = 0
        while i < lines.count {
            if let fm = Fence.match(trimTrailingWhitespace(lines[i])) {
                let markerRun = fm.marker
                let tag = fm.tag
                let firstBodyLine = (i + 1 < lines.count) ? lines[i + 1] : nil
                if shouldTagAsMermaid(tag: tag, firstBodyLine: firstBodyLine),
                   tag.caseInsensitiveCompare("mermaid") != .orderedSame {
                    lines[i] = rebuildFence(fm, newTag: "mermaid")
                }
                i = indexAfterFenceBody(lines, startIndex: i + 1, marker: markerRun)
            } else {
                i += 1
            }
        }
        return lines.joined(separator: "\n")
    }

    private static func shouldTagAsMermaid(tag: String, firstBodyLine: String?) -> Bool {
        if !tag.isEmpty { return alias.contains(tag.lowercased()) }
        guard let first = firstBodyLine else { return false }
        let trimmed = first.trimmingCharacters(in: .whitespacesAndNewlines)
        let range = NSRange(location: 0, length: trimmed.utf16.count)
        return keyword.firstMatch(in: trimmed, range: range) != nil
    }

    private static func rebuildFence(_ fm: FenceMatch, newTag: String) -> String {
        var s = fm.indent + fm.marker + newTag
        if !fm.attrs.isEmpty { s += " " + fm.attrs }
        return s
    }

    private static func indexAfterFenceBody(_ lines: [String], startIndex: Int, marker: String) -> Int {
        var j = startIndex
        while j < lines.count {
            if let fm = Fence.match(trimTrailingWhitespace(lines[j])) {
                if let mf = marker.first, let cf = fm.marker.first,
                   mf == cf && fm.marker.count >= marker.count && fm.tag.isEmpty {
                    return j + 1
                }
            }
            j += 1
        }
        return lines.count
    }
}
