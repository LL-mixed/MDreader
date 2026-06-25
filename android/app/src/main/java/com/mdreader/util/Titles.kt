package com.mdreader.util

/** Pure helpers for deriving human-friendly titles from file paths. */
object Titles {

    private val markdownExtensions = setOf("md", "markdown", "mdown", "mkd", "mkdown")

    /**
     * Returns a display title from a file path or URI path: takes the segment
     * after the last separator and strips a known markdown extension.
     * Non-markdown extensions are preserved.
     */
    fun fromPath(path: String): String {
        if (path.isEmpty()) return ""
        val slash = maxOf(path.lastIndexOf('/'), path.lastIndexOf('\\'))
        val name = if (slash >= 0) path.substring(slash + 1) else path
        val dot = name.lastIndexOf('.')
        if (dot <= 0) return name
        val ext = name.substring(dot + 1).lowercase()
        return if (ext in markdownExtensions) name.substring(0, dot) else name
    }
}
