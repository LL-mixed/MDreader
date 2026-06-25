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

        let renderDir = Bundle.main.resourceURL!.appendingPathComponent("shared/render")
        let indexURL = renderDir.appendingPathComponent("index.html")
        webView.loadFileURL(indexURL, allowingReadAccessTo: renderDir)
        return webView
    }

    func updateNSView(_ webView: ZoomWebView, context: Context) {
        let coord = context.coordinator
        coord.markdown = markdown
        coord.isDark = isDark
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

    final class Coordinator: NSObject, WKScriptMessageHandler {
        weak var webView: WKWebView?
        var markdown: String
        var isDark: Bool
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
            let guarded = SvgGuard.protect(MermaidFenceNormalizer.normalize(markdown))
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
