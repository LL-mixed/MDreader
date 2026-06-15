package com.mdreader.render

import android.annotation.SuppressLint
import android.view.ViewGroup
import android.webkit.JavascriptInterface
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
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
private class SourceBridge(initialMarkdown: String, initialDark: Boolean) {
    @Volatile var markdownSource: String = initialMarkdown
    @Volatile var darkMode: Boolean = initialDark
    @Volatile var renderedOnce: Boolean = false

    @JavascriptInterface fun getMarkdown(): String = markdownSource
    @JavascriptInterface fun getDark(): Boolean = darkMode
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
    AndroidView(
        modifier = modifier,
        factory = { context ->
            val bridge = SourceBridge(markdown, isDark)
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
                tag = bridge
                loadUrl("file:///android_asset/render/index.html")
            }
        },
        update = { webView ->
            val bridge = webView.tag as SourceBridge
            val changed = bridge.markdownSource != markdown || bridge.darkMode != isDark
            bridge.markdownSource = markdown
            bridge.darkMode = isDark
            // After the first render, re-render in place on content/theme change
            // (no shell reload, so no flicker).
            if (changed && bridge.renderedOnce) {
                webView.evaluateJavascript("window.MDreader && window.MDreader.render()", null)
            }
        },
    )
}
