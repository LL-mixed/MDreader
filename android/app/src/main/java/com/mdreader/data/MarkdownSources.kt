package com.mdreader.data

import android.content.ContentResolver
import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import com.mdreader.util.Titles

/** A markdown document loaded from some source, ready to render. */
data class LoadedDocument(val title: String, val markdown: String)

/** Reads markdown content from external (intent) or internal sources. */
object MarkdownSources {

    /** Reads the markdown text from [uri], or null if it cannot be read. */
    fun readText(context: Context, uri: Uri): String? = try {
        context.contentResolver.openInputStream(uri)?.use { input ->
            input.bufferedReader().readText()
        }
    } catch (e: Exception) {
        null
    }

    /**
     * Best-effort display name for [uri]: the provider's DISPLAY_NAME if
     * available, otherwise derived from the URI path. The markdown extension
     * is stripped for a cleaner title.
     */
    fun displayName(context: Context, uri: Uri): String {
        val queried = queryDisplayName(context.contentResolver, uri)
        if (!queried.isNullOrBlank()) return Titles.fromPath(queried)
        return Titles.fromPath(uri.path.orEmpty())
            .ifEmpty { Titles.fromPath(uri.lastPathSegment.orEmpty()) }
    }

    private fun queryDisplayName(resolver: ContentResolver, uri: Uri): String? = try {
        resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
            if (c.moveToFirst() && !c.isNull(0)) c.getString(0) else null
        }
    } catch (e: Exception) {
        null
    }
}
