package com.mdreader.render

import android.annotation.SuppressLint
import android.view.ViewGroup
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView

/**
 * Renders [markdown] as styled HTML inside a WebView.
 *
 * The document HTML is rebuilt only when [markdown] or [isDark] change (tracked
 * via the view tag), so recomposition does not reload the page and cause flicker.
 */
@SuppressLint("SetJavaScriptEnabled")
@Composable
fun MarkdownView(
    markdown: String,
    isDark: Boolean,
    modifier: Modifier = Modifier,
) {
    val html = remember(markdown, isDark) {
        MarkdownHtmlBuilder.build(markdown, RenderTheme.fromDark(isDark))
    }

    AndroidView(
        modifier = modifier,
        factory = { context ->
            WebView(context).apply {
                layoutParams = ViewGroup.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT,
                    ViewGroup.LayoutParams.MATCH_PARENT,
                )
                settings.javaScriptEnabled = true
                settings.loadWithOverviewMode = true
                settings.cacheMode = WebSettings.LOAD_NO_CACHE
                isVerticalScrollBarEnabled = true
                webViewClient = WebViewClient()
            }
        },
        update = { webView ->
            // Reload only when the rendered document actually changed.
            if (webView.tag != html) {
                webView.loadDataWithBaseURL(
                    /* baseUrl = */ "file:///android_asset/",
                    /* data = */ html,
                    /* mimeType = */ "text/html",
                    /* encoding = */ "utf-8",
                    /* historyUrl = */ null,
                )
                webView.tag = html
            }
        },
    )
}
