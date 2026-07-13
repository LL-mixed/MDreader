// Rendering layer.
// - fence / svg_guard / mermaid_fence / preprocess / outline: pure logic, shared
//   by both GUI and TUI.
// - webview: WebKitGTK custom scheme + JS bridge — GUI-only, behind `gui`.

pub mod fence;
pub mod mermaid_fence;
pub mod outline;
pub mod preprocess;
pub mod svg_guard;

#[cfg(feature = "gui")]
pub mod webview;
