package com.mdreader.render

import java.util.Locale

/**
 * Stylesheet variant applied to the rendered document.
 * The [bodyClass] value matches a CSS rule in assets/render/render.css.
 */
enum class RenderTheme(val bodyClass: String) {
    LIGHT("light"),
    DARK("dark");

    companion object {
        fun fromDark(isDark: Boolean): RenderTheme =
            if (isDark) DARK else LIGHT
    }
}

/**
 * Builds a self-contained HTML document that renders Markdown through the
 * vendored WebView assets. The markdown payload is embedded as a JSON string
 * literal assigned to `window.MD_SOURCE`; assets/render/render.js then parses
 * it with marked.js, renders $...$/$$...$$ math with KaTeX, and highlights code
 * with highlight.js.
 *
 * Pure and side-effect free, so it can be unit-tested on the JVM without a
 * WebView. The payload is JSON-escaped so it is safe inside a <script> block.
 */
object MarkdownHtmlBuilder {

    private const val PLACEHOLDER_THEME = "__THEME__"
    private const val PLACEHOLDER_SOURCE = "__SOURCE__"

    private val TEMPLATE = """
        <!DOCTYPE html>
        <html lang="zh">
        <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
        <link rel="stylesheet" href="render.css">
        <link rel="stylesheet" href="katex/katex.min.css">
        <script src="marked.min.js"></script>
        <script src="highlight.min.js"></script>
        <script src="katex/katex.min.js"></script>
        </head>
        <body class="$PLACEHOLDER_THEME">
        <article id="content"></article>
        <script>window.MD_SOURCE = $PLACEHOLDER_SOURCE;</script>
        <script src="render.js"></script>
        </body>
        </html>
    """.trimIndent()

    /** Returns the full HTML document for [markdown] rendered under [theme]. */
    fun build(markdown: String, theme: RenderTheme): String =
        TEMPLATE
            .replace(PLACEHOLDER_THEME, theme.bodyClass)
            .replace(PLACEHOLDER_SOURCE, "\"" + jsonEscape(markdown) + "\"")

    /**
     * Escapes [value] as a JSON string body (without surrounding quotes).
     * Also escapes '<' as a unicode escape so an embedded closing script tag in
     * the markdown can never terminate the inline script block.
     *
     * Uses integer code comparisons (not char-literal escapes) so the source
     * never contains literal control characters.
     */
    internal fun jsonEscape(value: String): String {
        val sb = StringBuilder(value.length + 8)
        for (c in value) {
            val code = c.code
            sb.append(
                when (code) {
                    0x5C -> "\\\\"      // backslash
                    0x22 -> "\\\""      // double quote
                    0x0A -> "\\n"       // newline
                    0x0D -> "\\r"       // carriage return
                    0x09 -> "\\t"       // tab
                    0x08 -> "\\b"       // backspace
                    0x0C -> "\\f"       // form feed
                    0x3C -> "\\u003c"   // '<'
                    else -> if (code < 0x20) {
                        String.format(Locale.ROOT, "\\u%04x", code)
                    } else {
                        c.toString()
                    }
                }
            )
        }
        return sb.toString()
    }
}
