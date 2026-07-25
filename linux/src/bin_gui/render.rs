// GUI-only rendering: WebKitGTK custom scheme + JS bridge.
// (The pure-logic render modules — fence/mermaid_fence/svg_guard/preprocess/
// outline — now live in the `mdreader_core` lib in tui/.)

#[cfg(feature = "gui")]
pub mod webview;
