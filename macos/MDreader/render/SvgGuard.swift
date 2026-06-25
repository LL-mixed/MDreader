import Foundation

struct Guarded {
    let markdown: String
    let svgs: [String]
}

enum SvgGuard {
    static let marker: Character = "\u{01}"
    static let end: Character = "\u{02}"

    private static let svgRegex: NSRegularExpression = {
        try! NSRegularExpression(pattern: "<svg\\b[\\s\\S]*?</svg>", options: [])
    }()

    static func placeholder(_ index: Int) -> String {
        "\(marker)\(index)\(end)"
    }

    static func protect(_ markdown: String) -> Guarded {
        if !markdown.contains("<svg") {
            return Guarded(markdown: markdown, svgs: [])
        }
        var svgs: [String] = []
        let lines = markdown.components(separatedBy: "\n")
        var out = ""
        var i = 0
        var inFence = false
        var fenceMarker = ""
        while i < lines.count {
            let line = lines[i]
            if let fm = Fence.match(trimTrailingWhitespace(line)) {
                let markerRun = fm.marker
                if !inFence {
                    inFence = true
                    fenceMarker = markerRun
                } else if !markerRun.isEmpty && !fenceMarker.isEmpty &&
                    markerRun.first == fenceMarker.first &&
                    markerRun.count >= fenceMarker.count {
                    inFence = false
                    fenceMarker = ""
                }
                out += line + "\n"
                i += 1
                continue
            }
            if inFence {
                out += line + "\n"
                i += 1
                continue
            }
            if line.contains("<svg") {
                var buf = line
                var j = i
                if !line.contains("</svg>") {
                    j = i + 1
                    while j < lines.count {
                        buf += "\n" + lines[j]
                        if lines[j].contains("</svg>") { break }
                        j += 1
                    }
                }
                let replaced = extractSvgs(buf, into: &svgs)
                out += replaced + "\n"
                i = j + 1
                continue
            }
            out += line + "\n"
            i += 1
        }
        if out.hasSuffix("\n") { out.removeLast() }
        return Guarded(markdown: out, svgs: svgs)
    }

    private static func extractSvgs(_ text: String, into svgs: inout [String]) -> String {
        let ns = text as NSString
        let range = NSRange(location: 0, length: ns.length)
        let matches = svgRegex.matches(in: text, range: range)
        var result = ""
        var cursor = 0
        for m in matches {
            let mr = m.range
            result += ns.substring(with: NSRange(location: cursor, length: mr.location - cursor))
            svgs.append(ns.substring(with: mr))
            result += placeholder(svgs.count - 1)
            cursor = mr.location + mr.length
        }
        result += ns.substring(with: NSRange(location: cursor, length: ns.length - cursor))
        return result
    }
}
