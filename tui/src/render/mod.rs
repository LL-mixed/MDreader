// Rendering layer (pure logic — no GUI/WebView).
// - fence: CommonMark fence matcher
// - mermaid_fence: rewrite non-standard fence tags to `mermaid`
// - outline: decode of the JS onOutline payload
// - preprocess: resolve relative images to data URIs / inline SVG
// - svg_guard: lift top-level SVGs out of markdown

pub mod fence;
pub mod mermaid_fence;
pub mod outline;
pub mod preprocess;
pub mod svg_guard;
