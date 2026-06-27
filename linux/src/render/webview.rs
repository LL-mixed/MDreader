// WebKitGTK webview + custom `mdreader://` URI scheme + JS bridge.
//
// 1. Serve bundled `shared/render/**` from the GResource under a custom scheme, so relative
//    references resolve same-origin — WebKitGTK cannot load `resource://` for web content
//    (unlike WKWebView's bundle), so we provide our own equivalent.
// 2. Inject the `mdreaderNative` bridge (port of macOS's bridgeScript): synchronous reads
//    (getMarkdown/getDark/getSvg) come from a pre-populated `__mdrPayload`; async callbacks post
//    to a registered message handler.

use std::path::Path;

use webkit6::prelude::*;
use webkit6::{
    UserContentInjectedFrames, UserScript, UserScriptInjectionTime, URISchemeRequest, WebContext,
    WebView,
};

pub const SCHEME: &str = "mdreader";
const PREFIX: &str = "/com/mdreader/MDreader";
/// Triple-slash so the path is `render/index.html` (empty authority) and relative refs resolve
/// under the same scheme, e.g. `render.css` -> mdreader:///render/render.css.
pub const INDEX_URI: &str = "mdreader:///render/index.html";
pub const MSG_HANDLER: &str = "mdreaderNative";

/// Register the `mdreader://` scheme on the default web context. Call once at startup, before
/// any webview is created (WebView::new() uses the default context).
pub fn register_scheme() {
    let ctx = WebContext::default().expect("default WebContext");
    if let Some(sm) = ctx.security_manager() {
        sm.register_uri_scheme_as_local(SCHEME);
        sm.register_uri_scheme_as_secure(SCHEME);
        sm.register_uri_scheme_as_cors_enabled(SCHEME);
    }
    ctx.register_uri_scheme(SCHEME, serve);
}

/// Build a webview wired with the bridge, then load the renderer page.
pub fn new_webview() -> WebView {
    let wv = WebView::new();
    let md = bundled_sample().unwrap_or_else(|| "# MDreader\n\n(no sample found)".to_string());
    let payload = build_payload(&md, false, None);
    if let Some(ucm) = wv.user_content_manager() {
        ucm.register_script_message_handler(MSG_HANDLER, None);
        ucm.add_script(&UserScript::new(
            &bridge_shim(&payload),
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        ));
    }
    wv.load_uri(INDEX_URI);
    wv
}

/// Run the native preprocessing pipeline (resolve images -> normalize mermaid fences ->
/// guard SVGs) and return the JSON payload `{md, dark, svgs}`.
pub fn build_payload(md: &str, dark: bool, base_dir: Option<&Path>) -> String {
    let resolved = super::preprocess::resolve_images(md, base_dir);
    let normalized = super::mermaid_fence::normalize(&resolved);
    let guarded = super::svg_guard::protect(&normalized);
    serde_json::json!({
        "md": guarded.markdown,
        "dark": dark,
        "svgs": guarded.svgs,
    })
    .to_string()
}

/// The bridge shim: exposes `window.mdreaderNative` reading from `window.__mdrPayload`.
fn bridge_shim(payload_json: &str) -> String {
    let h = MSG_HANDLER;
    let mut s = String::new();
    s.push_str("(function(){\n");
    s.push_str("  window.__mdrPayload = ");
    s.push_str(payload_json);
    s.push_str(";\n");
    s.push_str("  window.mdreaderNative = {\n");
    s.push_str("    getMarkdown: function(){ return window.__mdrPayload.md; },\n");
    s.push_str("    getDark: function(){ return window.__mdrPayload.dark; },\n");
    s.push_str("    getSvg: function(i){ return (window.__mdrPayload.svgs||[])[i] || ''; },\n");
    s.push_str(&format!(
        "    markRendered: function(){{ window.webkit.messageHandlers.{}.postMessage({{event:'markRendered'}}); }},\n",
        h
    ));
    s.push_str(&format!(
        "    onOutline: function(j){{ window.webkit.messageHandlers.{}.postMessage({{event:'onOutline',json:j}}); }},\n",
        h
    ));
    s.push_str(&format!(
        "    onActiveHeading: function(i){{ window.webkit.messageHandlers.{}.postMessage({{event:'onActiveHeading',index:i}}); }}\n",
        h
    ));
    s.push_str("  };\n");
    s.push_str("})();");
    s
}

fn bundled_sample() -> Option<String> {
    let bytes = gio::resources_lookup_data(
        &format!("{PREFIX}/sample.md"),
        gio::ResourceLookupFlags::empty(),
    )
    .ok()?;
    String::from_utf8(bytes.as_ref().to_vec()).ok()
}

/// Serve one render resource from the GResource.
fn serve(request: &URISchemeRequest) {
    let uri = request.uri().unwrap_or_default();
    let raw = uri.split("://").nth(1).unwrap_or("");
    let path = raw.trim_start_matches('/');
    let path = if path.is_empty() { "render/index.html" } else { path };
    let res = format!("{PREFIX}/{path}");
    match gio::resources_lookup_data(&res, gio::ResourceLookupFlags::empty()) {
        Ok(bytes) => {
            let len = bytes.as_ref().len() as i64;
            let stream = gio::MemoryInputStream::from_bytes(&bytes);
            request.finish(&stream, len, Some(mime_for(path)));
        }
        Err(mut e) => {
            eprintln!("mdreader: resource lookup failed for {res}: {e}");
            request.finish_error(&mut e);
        }
    }
}

fn mime_for(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html",
        "js" | "mjs" => "text/javascript",
        "css" => "text/css",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "md" | "markdown" => "text/markdown",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}
