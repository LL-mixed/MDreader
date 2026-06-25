package com.mdreader.render

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Unit tests for [SvgGuard]. The guard is pure logic that lifts top-level
 * `<svg>…</svg>` blocks out of markdown so marked's HTML-block rule (which
 * ends a block at the first blank line) cannot truncate a large SVG. These
 * run on the JVM with no WebView.
 */
class SvgGuardTest {

    private fun g(md: String) = SvgGuard.guard(md)

    @Test fun `markdown without svg is returned unchanged`() {
        val src = "# Title\n\ntext **bold**\n\n```kotlin\nfun x() = 1\n```\n"
        val result = g(src)
        assertEquals(src, result.markdown)
        assertTrue(result.svgs.isEmpty())
    }

    @Test fun `single one_line svg is extracted`() {
        val src = "before\n<svg id=\"a\"><rect/></svg>\nafter"
        val result = g(src)
        assertEquals(listOf("<svg id=\"a\"><rect/></svg>"), result.svgs)
        assertFalse(result.markdown.contains("<svg"))
        assertEquals(SvgGuard.placeholder(0), result.markdown.split('\n')[1])
    }

    @Test fun `large svg containing blank lines is kept intact`() {
        // Reproduces the real failure: marked ended the HTML block at the first
        // blank line inside the SVG, corrupting the diagram. Guarding must lift
        // the whole thing, blank lines included.
        val svg = buildString {
            append("<svg viewBox=\"0 0 1400 1800\">")
            append("<defs><linearGradient id=\"g1\"><stop/></linearGradient></defs>")
            append("\n\n") // blank line that used to truncate the block
            append("<g>\n<text>1940s</text>\n\n<text>2020s</text>\n</g>")
            append("\n\n<!-- comment -->\n<text>x</text>")
            append("</svg>")
        }
        val src = "intro\n\n$svg\n\noutro"
        val result = g(src)
        assertEquals(listOf(svg), result.svgs)
        // The guarded markdown must not carry the raw SVG or any of its tags.
        assertFalse(result.markdown.contains("<svg"))
        assertFalse(result.markdown.contains("<rect"))
        assertFalse(result.markdown.contains("</svg>"))
        // Surrounding markdown is preserved.
        assertTrue(result.markdown.contains("intro"))
        assertTrue(result.markdown.contains("outro"))
    }

    @Test fun `multiple svgs get sequential placeholders`() {
        val src = "<svg>A</svg>\nmid\n<svg>B</svg>"
        val result = g(src)
        assertEquals(listOf("<svg>A</svg>", "<svg>B</svg>"), result.svgs)
        assertTrue(result.markdown.contains(SvgGuard.placeholder(0)))
        assertTrue(result.markdown.contains(SvgGuard.placeholder(1)))
    }

    @Test fun `svg inside fenced code block is not extracted`() {
        val src = "```xml\n<svg>kept as code</svg>\n```\n<svg>real one</svg>"
        val result = g(src)
        // Only the top-level SVG is lifted; the fenced one stays verbatim.
        assertEquals(listOf("<svg>real one</svg>"), result.svgs)
        assertTrue(result.markdown.contains("<svg>kept as code</svg>"))
        assertFalse(result.markdown.contains("<svg>real one</svg>"))
    }

    @Test fun `tilde fence also protects inner svg`() {
        val src = "~~~\n<svg>code</svg>\n~~~\n<svg>real</svg>"
        val result = g(src)
        assertEquals(listOf("<svg>real</svg>"), result.svgs)
        assertTrue(result.markdown.contains("<svg>code</svg>"))
    }

    @Test fun `placeholder format is marker_index_end`() {
        val result = g("<svg>x</svg>")
        assertEquals('\u0001'.toString() + "0" + '\u0002'.toString(), SvgGuard.placeholder(0))
        assertTrue(result.markdown.contains(SvgGuard.placeholder(0)))
    }

    @Test fun `text after a closed svg on the same line is preserved`() {
        val src = "line\n<svg><rect/></svg>\ntail"
        val result = g(src)
        assertTrue(result.markdown.contains("tail"))
        assertEquals(listOf("<svg><rect/></svg>"), result.svgs)
    }
}
