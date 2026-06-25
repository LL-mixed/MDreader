package com.mdreader.render

/**
 * A heading in the rendered document, used to build the outline (table of
 * contents). [index] is the heading's position in document order — the same
 * index the renderer uses to scroll into view — so it always aligns with the
 * DOM regardless of how the markdown was authored (ATX, Setext, or raw HTML).
 */
data class OutlineItem(
    val index: Int,
    val level: Int,
    val text: String,
)
