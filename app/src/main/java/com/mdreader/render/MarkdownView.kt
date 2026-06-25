package com.mdreader.render

import android.annotation.SuppressLint
import android.util.Log
import android.view.ViewGroup
import android.webkit.ConsoleMessage
import android.webkit.JavascriptInterface
import android.webkit.WebChromeClient
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView

/**
 * Bridges the markdown source and theme from Kotlin to the renderer page's JS.
 * The markdown is passed as a runtime string value (not embedded in HTML/JS),
 * so no escaping is needed and injection is impossible.
 *
 * Backing properties are named distinctly from the @JavascriptInterface methods
 * to avoid a JVM signature clash between the property getters and the methods.
 */
private class SourceBridge(
    initialMarkdown: String,
    initialDark: Boolean,
    initialSvgs: List<String>,
) {
    @Volatile var markdownSource: String = initialMarkdown
    @Volatile var darkMode: Boolean = initialDark
    @Volatile var svgs: List<String> = initialSvgs
    @Volatile var renderedOnce: Boolean = false

    @JavascriptInterface fun getMarkdown(): String = markdownSource
    @JavascriptInterface fun getDark(): Boolean = darkMode

    /** Returns the [index]-th guarded SVG; render.js re-injects it after marked.parse. */
    @JavascriptInterface fun getSvg(index: Int): String = svgs.getOrElse(index) { "" }

    @JavascriptInterface fun markRendered() { renderedOnce = true }
}

/**
 * Renders [markdown] as styled HTML inside a WebView.
 *
 * The page is loaded with loadUrl from the asset shell (file:///android_asset/
 * render/index.html), which makes the page and all its sibling assets (scripts,
 * KaTeX fonts) same-origin and reliably loadable across WebView versions — a
 * property loadDataWithBaseURL does not guarantee on real devices. Content is
 * supplied at runtime via the [SourceBridge] and re-rendered when it changes.
 */
@SuppressLint("SetJavaScriptEnabled")
@Composable
fun MarkdownView(
    markdown: String,
    isDark: Boolean,
    modifier: Modifier = Modifier,
) {
    // Normalize non-standard Mermaid fences (```sequence, ```gantt, aliases,
    // and untagged keyword blocks) to the ```mermaid tag the renderer handles.
    // Pure logic — see MermaidFenceNormalizer and its JVM tests.
    val normalized = remember(markdown) { MermaidFenceNormalizer.normalize(markdown) }
    // Guard inline <svg>…</svg> blocks: marked's HTML-block rule ends a block
    // at the first blank line, which truncates large SVGs mid-way. The guard
    // lifts SVGs out; the renderer re-injects them after marked.parse via
    // getSvg. Pure logic — see SvgGuard and its JVM tests.
    val guarded = remember(normalized) { SvgGuard.guard(normalized) }

    AndroidView(
        modifier = modifier,
        factory = { context ->
            val bridge = SourceBridge(guarded.markdown, isDark, guarded.svgs)
            WebView(context).apply {
                layoutParams = ViewGroup.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT,
                    ViewGroup.LayoutParams.MATCH_PARENT,
                )
                settings.javaScriptEnabled = true
                settings.loadWithOverviewMode = true
                settings.cacheMode = WebSettings.LOAD_NO_CACHE
                isVerticalScrollBarEnabled = true
                addJavascriptInterface(bridge, "mdreaderNative")
                webViewClient = WebViewClient()
                // Forward JS console output to logcat (tag MDreaderWeb) so renderer
                // issues stay observable via `adb logcat` during development.
                webChromeClient = object : WebChromeClient() {
                    override fun onConsoleMessage(consoleMessage: ConsoleMessage?): Boolean {
                        Log.i(
                            "MDreaderWeb",
                            "${consoleMessage?.message()} (${consoleMessage?.sourceId()}:${consoleMessage?.lineNumber()})",
                        )
                        return true
                    }
                }
                tag = bridge
                loadUrl("file:///android_asset/render/index.html")
            }
        },
        update = { webView ->
            val bridge = webView.tag as SourceBridge
            val changed = bridge.markdownSource != guarded.markdown ||
                bridge.darkMode != isDark ||
                bridge.svgs != guarded.svgs
            bridge.markdownSource = guarded.markdown
            bridge.svgs = guarded.svgs
            bridge.darkMode = isDark
            // After the first render, re-render in place on content/theme change
            // (no shell reload, so no flicker).
            if (changed && bridge.renderedOnce) {
                webView.evaluateJavascript("window.MDreader && window.MDreader.render()", null)
            }
        },
    )
}
