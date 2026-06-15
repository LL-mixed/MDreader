package com.mdreader.render

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Unit tests for MarkdownHtmlBuilder. Inputs and expected values that contain
 * quotes or backslashes are built from char codes so the test source never
 * relies on backslash escape literals.
 */
class MarkdownHtmlBuilderTest {

    private val backslash = 0x5C.toChar()
    private val quote = 0x22.toChar()

    @Test
    fun darkThemeUsesDarkBodyClass() {
        val html = MarkdownHtmlBuilder.build("# Title", RenderTheme.DARK)
        assertTrue(html.contains("class=" + quote + "dark"))
        assertFalse(html.contains("class=" + quote + "light"))
    }

    @Test
    fun lightThemeUsesLightBodyClass() {
        val html = MarkdownHtmlBuilder.build("# Title", RenderTheme.LIGHT)
        assertTrue(html.contains("class=" + quote + "light"))
    }

    @Test
    fun referencesVendoredAssets() {
        val html = MarkdownHtmlBuilder.build("body", RenderTheme.LIGHT)
        assertTrue(html.contains("marked.min.js"))
        assertTrue(html.contains("highlight.min.js"))
        assertTrue(html.contains("render.css"))
        assertTrue(html.contains("katex/katex.min.js"))
        assertTrue(html.contains("katex/katex.min.css"))
        assertTrue(html.contains("render.js"))
    }

    @Test
    fun embedsSourceAndLoadsRenderer() {
        val html = MarkdownHtmlBuilder.build("hello math", RenderTheme.LIGHT)
        assertTrue(html.contains("window.MD_SOURCE ="))
        assertTrue(html.contains("\"hello math\""))
        assertTrue(html.contains("render.js"))
    }

    @Test
    fun markdownIsEmbeddedAsJsonString() {
        val html = MarkdownHtmlBuilder.build("hello world", RenderTheme.LIGHT)
        assertTrue(html.contains("window.MD_SOURCE ="))
        assertTrue(html.contains("hello world"))
    }

    @Test
    fun quotesAndNewlinesAreEscaped() {
        // input:  a " b <LF> c
        val input = "a" + quote + "b" + 0x0A.toChar() + "c"
        // expected: a \" b \n c   (literal backslash sequences)
        val expected = "a" + backslash + quote + "b" + backslash + "n" + "c"
        assertEquals(expected, MarkdownHtmlBuilder.jsonEscape(input))
    }

    @Test
    fun backslashIsEscaped() {
        val singleBackslash = backslash.toString()
        assertEquals(singleBackslash + singleBackslash, MarkdownHtmlBuilder.jsonEscape(singleBackslash))
    }

    @Test
    fun scriptClosingTagIsEscaped() {
        val escaped = MarkdownHtmlBuilder.jsonEscape("</script>")
        assertFalse("raw closing script tag must not survive", escaped.contains("</script>"))
        // '<' becomes a unicode escape: backslash + u003c
        assertTrue(escaped.contains(backslash + "u003c"))
    }

    @Test
    fun controlCharsBecomeUnicodeEscapes() {
        val bell = 0x07.toChar().toString()
        val expected = backslash + "u0007"
        assertEquals(expected, MarkdownHtmlBuilder.jsonEscape(bell))
    }

    @Test
    fun fromDarkMapsCorrectly() {
        assertEquals(RenderTheme.DARK, RenderTheme.fromDark(isDark = true))
        assertEquals(RenderTheme.LIGHT, RenderTheme.fromDark(isDark = false))
    }
}
