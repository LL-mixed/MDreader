import Foundation

struct FenceMatch {
    let indent: String
    let marker: String
    let tag: String
    let attrs: String
}

enum Fence {
    private static let regex: NSRegularExpression = {
        let pattern = "^([ \\t]{0,3})(`{3,}|~{3,})[ \\t]*([\\w-]+)?[ \\t]*(\\{.*\\})?[ \\t]*$"
        return try! NSRegularExpression(pattern: pattern, options: [])
    }()

    static func match(_ line: String) -> FenceMatch? {
        let range = NSRange(location: 0, length: line.utf16.count)
        guard let m = regex.firstMatch(in: line, range: range),
              m.range.length == line.utf16.count else { return nil }
        let ns = line as NSString
        func grp(_ n: Int) -> String {
            let r = m.range(at: n)
            return r.location == NSNotFound ? "" : ns.substring(with: r)
        }
        return FenceMatch(indent: grp(1), marker: grp(2), tag: grp(3), attrs: grp(4))
    }
}

func trimTrailingWhitespace(_ s: String) -> String {
    var s = s
    while let last = s.last, last.isWhitespace {
        s.removeLast()
    }
    return s
}
