import SwiftUI
import AppKit
import WebKit
import UniformTypeIdentifiers

final class ZoomWebView: WKWebView {
    var onCommandScroll: ((CGFloat) -> Void)?

    override func scrollWheel(with event: NSEvent) {
        if event.modifierFlags.contains(.command), event.deltaY != 0 {
            onCommandScroll?(event.deltaY)
            return
        }
        super.scrollWheel(with: event)
    }
}

struct MarkdownWebView: NSViewRepresentable {
    let markdown: String
    let isDark: Bool
    var baseDir: URL? = nil
    var zoom: Double = 1.0
    var scrollRequest: Int? = nil
    var exportRequest: Int = 0
    var onDropText: ((String, String) -> Void)? = nil
    var onOutline: (([OutlineItem]) -> Void)? = nil
    var onActiveHeading: ((Int) -> Void)? = nil
    var onCommandScroll: ((CGFloat) -> Void)? = nil

    func makeCoordinator() -> Coordinator {
        Coordinator(markdown: markdown, isDark: isDark)
    }

    func makeNSView(context: Context) -> ZoomWebView {
        let config = WKWebViewConfiguration()
        let ucc = config.userContentController
        ucc.add(context.coordinator, name: "mdreaderNative")
        ucc.addUserScript(WKUserScript(
            source: context.coordinator.bridgeScript(),
            injectionTime: .atDocumentStart,
            forMainFrameOnly: true
        ))
        ucc.addUserScript(WKUserScript(
            source: Self.dropScript(),
            injectionTime: .atDocumentEnd,
            forMainFrameOnly: true
        ))
        let webView = ZoomWebView(frame: .zero, configuration: config)
        webView.onCommandScroll = onCommandScroll
        context.coordinator.webView = webView
        context.coordinator.onDropText = onDropText
        context.coordinator.onOutline = onOutline
        context.coordinator.onActiveHeading = onActiveHeading
        context.coordinator.baseDir = baseDir

        let renderDir = Bundle.main.resourceURL!.appendingPathComponent("shared/render")
        let indexURL = renderDir.appendingPathComponent("index.html")
        webView.loadFileURL(indexURL, allowingReadAccessTo: renderDir)
        return webView
    }

    func updateNSView(_ webView: ZoomWebView, context: Context) {
        let coord = context.coordinator
        coord.markdown = markdown
        coord.isDark = isDark
        coord.baseDir = baseDir
        coord.onDropText = onDropText
        coord.onOutline = onOutline
        coord.onActiveHeading = onActiveHeading
        webView.onCommandScroll = onCommandScroll

        if webView.pageZoom != zoom {
            webView.pageZoom = zoom
        }

        if let req = scrollRequest, req != coord.lastScrollRequest {
            coord.lastScrollRequest = req
            webView.evaluateJavaScript("window.MDreader && window.MDreader.scrollToHeading(\(req))")
        }

        if exportRequest != coord.lastExportRequest {
            coord.lastExportRequest = exportRequest
            Self.exportPDF(webView)
        }

        guard coord.hasRendered else { return }
        guard coord.lastMarkdown != markdown || coord.lastDark != isDark else { return }
        coord.lastMarkdown = markdown
        coord.lastDark = isDark
        let js = "window.__mdrPayload = \(coord.payloadJSON()); if (window.MDreader) { window.MDreader.render(); }"
        webView.evaluateJavaScript(js)
    }

    static func exportPDF(_ webView: WKWebView) {
        webView.createPDF { result in
            DispatchQueue.main.async {
                guard case .success(let data) = result else { return }
                let panel = NSSavePanel()
                panel.allowedContentTypes = [UTType.pdf]
                panel.nameFieldStringValue = "MDreader.pdf"
                guard panel.runModal() == .OK, let url = panel.url else { return }
                try? data.write(to: url)
            }
        }
    }

    static func dropScript() -> String {
        """
        (function(){
          document.addEventListener('dragover', function(e){ e.preventDefault(); });
          document.addEventListener('drop', function(e){
            e.preventDefault();
            var f = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];
            if (!f) return;
            if (!/\\.(md|markdown|mdown|mkd|mkdown)$/i.test(f.name || '')) return;
            var reader = new FileReader();
            reader.onload = function(){
              window.webkit.messageHandlers.mdreaderNative.postMessage({event:'dropFile', name:f.name, text:reader.result});
            };
            reader.readAsText(f);
          });
        })();
        """
    }

    /// Rewrites relative image URLs (`![alt](rel.png)`) to absolute file:// URLs
    /// resolved against the document's directory, so WKWebView can load figures
    /// stored alongside the .md. Absolute (http/file/path) URLs are left as-is.
    static func resolveImages(_ markdown: String, baseDir: URL) -> String {
        let pattern = #"(!\[[^\]]*\]\()([^)]+)(\))"#
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return markdown }
        let ns = markdown as NSString
        let matches = regex.matches(in: markdown, range: NSRange(location: 0, length: ns.length))
        var result = ""
        var pos = 0
        for m in matches {
            guard m.numberOfRanges >= 4 else { continue }
            result += ns.substring(with: NSRange(location: pos, length: m.range.location - pos))
            let g1 = ns.substring(with: m.range(at: 1))
            let original = ns.substring(with: m.range(at: 2))
            let g3 = ns.substring(with: m.range(at: 3))
            var src = original
            if let spaceIdx = src.firstIndex(of: " ") {
                src = String(src[..<spaceIdx])
            }
            if src.hasPrefix("http://") || src.hasPrefix("https://") || src.hasPrefix("/") || src.hasPrefix("file:") || src.hasPrefix("#") {
                result += g1 + original + g3
            } else {
                let absURL = baseDir.appendingPathComponent(src).standardizedFileURL
                let ext = (src as NSString).pathExtension.lowercased()
                if ext == "svg" {
                    if let svgText = try? String(contentsOf: absURL, encoding: .utf8) {
                        result += "\n\n" + svgText + "\n\n"
                    } else {
                        result += g1 + original + g3
                    }
                } else {
                    let mime: String
                    switch ext {
                    case "png": mime = "image/png"
                    case "jpg", "jpeg": mime = "image/jpeg"
                    case "gif": mime = "image/gif"
                    case "webp": mime = "image/webp"
                    default: mime = "application/octet-stream"
                    }
                    if let data = try? Data(contentsOf: absURL) {
                        result += "\(g1)data:\(mime);base64,\(data.base64EncodedString())\(g3)"
                    } else {
                        result += g1 + original + g3
                    }
                }
            }
            pos = m.range.location + m.range.length
        }
        if pos < ns.length {
            result += ns.substring(from: pos)
        }
        return result
    }

    final class Coordinator: NSObject, WKScriptMessageHandler {
        weak var webView: WKWebView?
        var markdown: String
        var isDark: Bool
        var baseDir: URL? = nil
        var hasRendered = false
        var lastMarkdown: String
        var lastDark: Bool
        var lastScrollRequest: Int? = nil
        var lastExportRequest: Int = 0
        var onDropText: ((String, String) -> Void)?
        var onOutline: (([OutlineItem]) -> Void)?
        var onActiveHeading: ((Int) -> Void)?

        init(markdown: String, isDark: Bool) {
            self.markdown = markdown
            self.isDark = isDark
            self.lastMarkdown = markdown
            self.lastDark = isDark
        }

        func bridgeScript() -> String {
            """
            (function(){
              window.__mdrPayload = \(payloadJSON());
              window.mdreaderNative = {
                getMarkdown: function(){ return window.__mdrPayload.md; },
                getDark: function(){ return window.__mdrPayload.dark; },
                getSvg: function(i){ return (window.__mdrPayload.svgs || [])[i] || ""; },
                markRendered: function(){ window.webkit.messageHandlers.mdreaderNative.postMessage({event:"markRendered"}); },
                onOutline: function(j){ window.webkit.messageHandlers.mdreaderNative.postMessage({event:"onOutline",json:j}); },
                onActiveHeading: function(i){ window.webkit.messageHandlers.mdreaderNative.postMessage({event:"onActiveHeading",index:i}); }
              };
            })();
            """
        }

        func payloadJSON() -> String {
            let resolved = baseDir.map { MarkdownWebView.resolveImages(markdown, baseDir: $0) } ?? markdown
            let guarded = SvgGuard.protect(MermaidFenceNormalizer.normalize(resolved))
            let payload: [String: Any] = [
                "md": guarded.markdown,
                "dark": isDark,
                "svgs": guarded.svgs,
            ]
            guard let data = try? JSONSerialization.data(withJSONObject: payload),
                  let json = String(data: data, encoding: .utf8) else {
                return "{\"md\":\"\",\"dark\":false,\"svgs\":[]}"
            }
            return json
        }

        func userContentController(_ ucc: WKUserContentController, didReceive message: WKScriptMessage) {
            guard let body = message.body as? [String: Any],
                  let event = body["event"] as? String else { return }
            switch event {
            case "markRendered":
                hasRendered = true
            case "dropFile":
                if let text = body["text"] as? String {
                    let name = body["name"] as? String ?? "Untitled"
                    onDropText?(text, name)
                }
            case "onOutline":
                if let json = body["json"] as? String,
                   let data = json.data(using: .utf8),
                   let items = try? JSONDecoder().decode([OutlineItem].self, from: data) {
                    onOutline?(items)
                }
            case "onActiveHeading":
                if let index = body["index"] as? Int {
                    onActiveHeading?(index)
                }
            default:
                break
            }
        }
    }
}
