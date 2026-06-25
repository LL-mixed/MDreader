package com.mdreader.render

/**
 * Guards inline `<svg>…</svg>` blocks from marked's HTML-block parsing.
 *
 * marked follows CommonMark's HTML-block rule that ends an HTML block at the
 * first blank line. Inline SVGs almost always contain blank lines (between
 * `<defs>`, `<g>`, `<text>` groups), so without guarding marked terminates
 * the block mid-SVG and re-parses the remainder as markdown, corrupting the
 * diagram — on real documents a ~15 KB SVG collapsed to ~2.3 KB in the DOM
 * and rendered at 0×0.
 *
 * [guard] lifts every top-level SVG out of the markdown and replaces it with
 * a compact, blank-line-free placeholder. The renderer re-injects the
 * originals after `marked.parse` via `SourceBridge.getSvg` (see render.js),
 * so each SVG enters the DOM intact and verbatim.
 *
 * SVGs nested inside fenced code blocks are left in place: marked renders
 * those verbatim, and guarding them would corrupt the code display.
 *
 * Pure and side-effect free: unit-testable on the JVM without a WebView.
 * The fence handling mirrors [MermaidFenceNormalizer] so fenced code is
 * respected by both passes.
 */
object SvgGuard {
    private val fenceRegex =
        Regex("""^([ \t]{0,3})(`{3,}|~{3,})[ \t]*([\w-]+)?[ \t]*(\{.*\})?[ \t]*$""")
    private val svgRegex = Regex("""<svg\b[\s\S]*?</svg>""")

    const val MARKER = '\u0001'
    const val END = '\u0002'

    data class Guarded(val markdown: String, val svgs: List<String>)

    /** Placeholder injected for the [index]-th SVG; matched by render.js. */
    fun placeholder(index: Int): String = "$MARKER${index}$END"

    /**
     * Returns [markdown] with every top-level (non-fenced) `<svg>…</svg>` run
     * replaced by a placeholder, together with the extracted SVG texts in
     * insertion order. Returns the input unchanged (with an empty list) when
     * it contains no `<svg` token at all.
     */
    fun guard(markdown: String): Guarded {
        if (!markdown.contains("<svg")) return Guarded(markdown, emptyList())
        val svgs = mutableListOf<String>()
        val lines = markdown.split('\n')
        val out = StringBuilder()
        var i = 0
        var inFence = false
        var fenceMarker = ""
        while (i < lines.size) {
            val line = lines[i]
            val fm = fenceRegex.matchEntire(line.trimEnd())
            if (fm != null) {
                val marker = fm.groupValues[2]
                if (!inFence) {
                    inFence = true
                    fenceMarker = marker
                } else if (marker.isNotEmpty() && fenceMarker.isNotEmpty() &&
                    marker[0] == fenceMarker[0] && marker.length >= fenceMarker.length
                ) {
                    inFence = false
                    fenceMarker = ""
                }
                out.append(line).append('\n')
                i++
                continue
            }
            if (inFence) {
                out.append(line).append('\n')
                i++
                continue
            }
            // Top-level: collect an SVG block from the first line containing
            // `<svg` up to the next line that closes it, then extract every
            // <svg>…</svg> run inside (usually one).
            if ("<svg" in line) {
                val buf = StringBuilder(line)
                var j = i
                if ("</svg>" !in line) {
                    j = i + 1
                    while (j < lines.size) {
                        buf.append('\n').append(lines[j])
                        if ("</svg>" in lines[j]) break
                        j++
                    }
                }
                val replaced = svgRegex.replace(buf.toString()) {
                    svgs.add(it.value)
                    placeholder(svgs.size - 1)
                }
                out.append(replaced).append('\n')
                i = j + 1
                continue
            }
            out.append(line).append('\n')
            i++
        }
        return Guarded(out.toString().removeSuffix("\n"), svgs)
    }
}
