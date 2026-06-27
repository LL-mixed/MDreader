// WebKitGTK rendering layer.
// - fence / svg_guard / mermaid_fence / preprocess: native preprocessing ports of macOS logic.
// - outline: decode of render.js's onOutline payload.
// - webview: custom `mdreader://` scheme (serves bundled resources from the GResource) and the
//   `mdreaderNative` JS bridge (mirrors macOS MarkdownWebView.bridgeScript).

pub mod fence;
pub mod mermaid_fence;
pub mod outline;
pub mod preprocess;
pub mod svg_guard;
pub mod webview;
