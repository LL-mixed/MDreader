package com.mdreader.util

import org.junit.Assert.assertEquals
import org.junit.Test

class TitlesTest {

    @Test
    fun stripsMarkdownExtension() {
        assertEquals("readme", Titles.fromPath("readme.md"))
    }

    @Test
    fun ignoresExtensionCase() {
        assertEquals("Notes", Titles.fromPath("/a/b/Notes.MARKDOWN"))
    }

    @Test
    fun handlesMultipleDots() {
        assertEquals("a.b", Titles.fromPath("a.b.md"))
    }

    @Test
    fun preservesNonMarkdownExtension() {
        assertEquals("archive.txt", Titles.fromPath("archive.txt"))
    }

    @Test
    fun noExtensionReturnedAsIs() {
        assertEquals("noext", Titles.fromPath("noext"))
    }

    @Test
    fun emptyPathReturnsEmpty() {
        assertEquals("", Titles.fromPath(""))
    }

    @Test
    fun handlesMdown() {
        assertEquals("doc", Titles.fromPath("WeChat Files/doc.mdown"))
    }

    @Test
    fun handlesBackslashSeparator() {
        val bs = 0x5C.toChar()
        val path = "C:" + bs + "Users" + bs + "me" + bs + "file.md"
        assertEquals("file", Titles.fromPath(path))
    }
}
