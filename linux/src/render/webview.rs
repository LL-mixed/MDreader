// WebKitGTK webview + custom `mdreader://` URI scheme + JS bridge.
//
// 1. Serve bundled `shared/render/**` from the GResource under a custom scheme, so relative
//    references resolve same-origin — WebKitGTK cannot load `resource://` for web content
//    (unlike WKWebView's bundle), so we provide our own equivalent.
// 2. Inject the `mdreaderNative` bridge (port of macOS's bridgeScript): synchronous reads
//    (getMarkdown/getDark/getSvg) come from a pre-populated `__mdrPayload`; async callbacks post
//    to a registered message handler.
// 3. Port of macOS's dropScript so dropping a .md onto the page opens it.

use std::path::Path;

use webkit6::prelude::*;
use webkit6::{
    UserContentInjectedFrames, UserScript, UserScriptInjectionTime, URISchemeRequest, WebContext,
    WebView,
};

use super::outline::{parse_outline, OutlineItem};

pub const SCHEME: &str = "mdreader";
const PREFIX: &str = "/com/mdreader/MDreader";
/// Triple-slash so the path is `render/index.html` (empty authority) and relative refs resolve
/// under the same scheme, e.g. `render.css` -> mdreader:///render/render.css.
pub const INDEX_URI: &str = "mdreader:///render/index.html";
pub const MSG_HANDLER: &str = "mdreaderNative";

/// Host-side callbacks invoked from the JS bridge.
pub struct Handlers {
    pub on_drop: Box<dyn Fn(&str, &str) + 'static>,
    pub on_outline: Box<dyn Fn(Vec<OutlineItem>) + 'static>,
    pub on_active: Box<dyn Fn(i32) + 'static>,
}

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

/// Build a webview wired with the bridge + drop handler + message callbacks, then load the page.
pub fn new_webview(md: &str, dark: bool, base_dir: Option<&Path>, handlers: Handlers) -> WebView {
    let wv = WebView::new();
    let payload = build_payload(md, dark, base_dir);
    if let Some(ucm) = wv.user_content_manager() {
        ucm.register_script_message_handler(MSG_HANDLER, None);
        ucm.add_script(&UserScript::new(
            &bridge_shim(&payload),
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::Start,
            &[],
            &[],
        ));
        ucm.add_script(&UserScript::new(
            drop_script(),
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::End,
            &[],
            &[],
        ));
        ucm.connect_script_message_received(Some(MSG_HANDLER), move |_ucm, value| {
            let Some(ev) = value.object_get_property("event") else {
                return;
            };
            let event = ev.to_str().to_string();
            match event.as_str() {
                "dropFile" => {
                    let name = value
                        .object_get_property("name")
                        .map(|v| v.to_str().to_string())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| "Untitled".to_string());
                    let text = value
                        .object_get_property("text")
                        .map(|v| v.to_str().to_string())
                        .unwrap_or_default();
                    (handlers.on_drop)(&name, &text);
                }
                "onOutline" => {
                    if let Some(jv) = value.object_get_property("json") {
                        if let Some(items) = parse_outline(&jv.to_str()) {
                            (handlers.on_outline)(items);
                        }
                    }
                }
                "onActiveHeading" => {
                    if let Some(iv) = value.object_get_property("index") {
                        (handlers.on_active)(iv.to_int32());
                    }
                }
                _ => {}
            }
        });
    }
    wv.load_uri(INDEX_URI);
    wv
}

/// Push new markdown/dark/base into an already-loaded webview and re-render.
pub fn render(webview: &WebView, md: &str, dark: bool, base_dir: Option<&Path>) {
    let payload = build_payload(md, dark, base_dir);
    let js = format!(
        "window.__mdrPayload = {payload}; if (window.MDreader) {{ window.MDreader.render(); }}"
    );
    webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
}

/// Whole-page zoom (mirrors macOS pageZoom), range handled by the caller.
pub fn set_zoom(webview: &WebView, zoom: f64) {
    webview.set_zoom_level(zoom);
}

/// Scroll the document to outline heading `index`.
pub fn scroll_to_heading(webview: &WebView, index: i32) {
    let js = format!("if (window.MDreader) {{ window.MDreader.scrollToHeading({index}); }}");
    webview.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
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

/// The bundled sample document (used when launched with no file).
pub fn bundled_sample() -> String {
    bundled_sample_impl().unwrap_or_else(|| "# MDreader\n\n(no sample found)".to_string())
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

/// Port of macOS dropScript: open a dropped .md file by reading it as text in-page.
fn drop_script() -> &'static str {
    r#"
(function(){
  document.addEventListener('dragover', function(e){ e.preventDefault(); });
  document.addEventListener('drop', function(e){
    e.preventDefault();
    var f = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];
    if (!f) return;
    if (!/\.(md|markdown|mdown|mkd|mkdown)$/i.test(f.name || '')) return;
    var reader = new FileReader();
    reader.onload = function(){
      window.webkit.messageHandlers.mdreaderNative.postMessage({event:'dropFile', name:f.name, text:reader.result});
    };
    reader.readAsText(f);
  });
})();
"#
}

fn bundled_sample_impl() -> Option<String> {
    let bytes =
        gio::resources_lookup_data(&format!("{PREFIX}/sample.md"), gio::ResourceLookupFlags::empty())
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
