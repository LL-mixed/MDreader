package com.mdreader.data

import android.content.Context
import java.io.File

/**
 * Stores the full markdown text of cached documents in app-internal storage,
 * one file per document id. Keeps large, variable-size content out of the DB.
 */
object DocStore {

    private fun file(context: Context, id: Long): File =
        File(File(context.filesDir, "docs"), "$id.md")

    fun write(context: Context, id: Long, markdown: String) {
        val target = file(context, id)
        target.parentFile?.mkdirs()
        target.writeText(markdown, Charsets.UTF_8)
    }

    fun read(context: Context, id: Long): String? =
        file(context, id).takeIf { it.exists() }?.readText(Charsets.UTF_8)

    fun delete(context: Context, id: Long): Boolean =
        file(context, id).delete()
}
