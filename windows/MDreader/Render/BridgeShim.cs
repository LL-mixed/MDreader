namespace MDreader.Render;

/// <summary>
/// Builds the <c>window.mdreaderNative</c> bridge shim JS.
/// </summary>
/// <remarks>
/// Mirrors <c>linux/src/render/webview.rs::bridge_shim</c>: exposes
/// <c>window.mdreaderNative</c> reading synchronous values from
/// <c>window.__mdrPayload</c>, while async callbacks post to
/// <c>window.chrome.webview</c> (WebView2) instead of
/// <c>window.webkit.messageHandlers</c> (WKWebView / WebKitGTK).
/// render.js stays unchanged across all four platforms.
/// </remarks>
public static class BridgeShim
{
    public static string Build(string payloadJson)
    {
        return
            "(function(){\n" +
            "  window.__mdrPayload = " + payloadJson + ";\n" +
            "  window.mdreaderNative = {\n" +
            "    getMarkdown: function(){ return window.__mdrPayload.md; },\n" +
            "    getDark: function(){ return window.__mdrPayload.dark; },\n" +
            "    getSvg: function(i){ return (window.__mdrPayload.svgs||[])[i] || ''; },\n" +
            "    markRendered: function(){ window.chrome.webview.postMessage({event:'markRendered'}); },\n" +
            "    onOutline: function(j){ window.chrome.webview.postMessage({event:'onOutline',json:j}); },\n" +
            "    onActiveHeading: function(i){ window.chrome.webview.postMessage({event:'onActiveHeading',index:i}); }\n" +
            "  };\n" +
            "})();";
    }
}
