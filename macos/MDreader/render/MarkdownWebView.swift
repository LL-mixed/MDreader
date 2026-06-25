import SwiftUI
import AppKit
import WebKit

struct MarkdownWebView: NSViewRepresentable {
    let markdown: String
    let isDark: Bool
    var onDropText: ((String, String) -> Void)? = nil

    func makeCoordinator() -> Coordinator {
        Coordinator(markdown: markdown, isDark: isDark)
    }

    func makeNSView(context: Context) -> WKWebView {
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
        let webView = WKWebView(frame: .zero, configuration: config)
        context.coordinator.webView = webView
        context.coordinator.onDropText = onDropText

        let renderDir = Bundle.main.resourceURL!.appendingPathComponent("shared/render")
        let indexURL = renderDir.appendingPathComponent("index.html")
        webView.loadFileURL(indexURL, allowingReadAccessTo: renderDir)
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        let coord = context.coordinator
        coord.markdown = markdown
        coord.isDark = isDark
        guard coord.hasRendered else { return }
        guard coord.lastMarkdown != markdown || coord.lastDark != isDark else { return }
        coord.lastMarkdown = markdown
        coord.lastDark = isDark
        let js = "window.__mdrPayload = \(coord.payloadJSON()); if (window.MDreader) { window.MDreader.render(); }"
        webView.evaluateJavaScript(js)
    }

    static func dropScript() -> String {
        """
        (function(){
          document.addEventListener('dragover', function(e){ e.preventDefault(); });
          document.addEventListener('drop', function(e){
            e.preventDefault();
            var f = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];
            if (!f) return;
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
        var onDropText: ((String, String) -> Void)?

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
            default:
                break
            }
        }
    }
}
