package com.mdreader.data

import android.content.Context
import android.net.Uri
import com.mdreader.data.db.CachedDocDao
import com.mdreader.data.db.CachedDocEntity
import com.mdreader.util.ContentHash
import kotlinx.coroutines.flow.Flow

/**
 * The single source of truth for cached documents. Caches by SHA-256 content
 * hash: reopening the same content only bumps [openedAt] instead of duplicating
 * storage. Full text is persisted to internal files via [DocStore].
 */
class DocRepository(
    private val dao: CachedDocDao,
    private val appContext: Context,
    private val now: () -> Long = { System.currentTimeMillis() },
) {

    /** Caches [markdown] (deduplicating by content) and returns the stored row. */
    suspend fun cache(title: String, markdown: String, sourceUri: String?): CachedDocEntity {
        val hash = ContentHash.sha256Hex(markdown)
        val timestamp = now()
        dao.findByHash(hash)?.let { existing ->
            dao.touchOpenedAt(existing.id, timestamp)
            return existing.copy(openedAt = timestamp)
        }
        val entity = CachedDocEntity(
            title = title.ifBlank { DEFAULT_TITLE },
            contentHash = hash,
            sourceUri = sourceUri,
            charCount = markdown.length,
            sizeBytes = markdown.toByteArray(Charsets.UTF_8).size,
            cachedAt = timestamp,
            openedAt = timestamp,
            favorite = false,
        )
        val id = dao.insert(entity)
        DocStore.write(appContext, id, markdown)
        return entity.copy(id = id)
    }

    /** Opens a cached document: refreshes [openedAt] and returns title + content. */
    suspend fun openDocument(id: Long): Pair<CachedDocEntity, String>? {
        val entity = dao.getById(id) ?: return null
        dao.touchOpenedAt(id, now())
        return entity to (DocStore.read(appContext, id) ?: "")
    }

    /**
     * Re-reads the original file backing [id]. If it still exists and its content
     * differs from the cached snapshot, updates the cached content + metadata and
     * returns true. Returns false when there is no source, the source is unreadable
     * (e.g. a content URI whose permission lapsed), or the content is unchanged.
     * Mirrors macOS `DocRepository.refreshFromSource` / Linux `refresh_from_source`.
     */
    suspend fun refreshFromSource(id: Long): Boolean {
        val entity = dao.getById(id) ?: return false
        val uri = entity.sourceUri?.let { runCatching { Uri.parse(it) }.getOrNull() } ?: return false
        val text = MarkdownSources.readText(appContext, uri) ?: return false
        val hash = ContentHash.sha256Hex(text)
        if (hash == entity.contentHash) return false
        dao.updateContent(
            id = id,
            hash = hash,
            charCount = text.length,
            sizeBytes = text.toByteArray(Charsets.UTF_8).size,
            timestamp = now(),
        )
        DocStore.write(appContext, id, text)
        return true
    }

    fun observeAll(): Flow<List<CachedDocEntity>> = dao.observeAll()

    fun search(query: String): Flow<List<CachedDocEntity>> = dao.search(query)

    /** Loads a document's text and refreshes its [openedAt]. */
    suspend fun loadContent(id: Long): String? {
        dao.touchOpenedAt(id, now())
        return DocStore.read(appContext, id)
    }

    suspend fun setFavorite(id: Long, favorite: Boolean) = dao.setFavorite(id, favorite)

    suspend fun delete(id: Long) {
        dao.deleteById(id)
        DocStore.delete(appContext, id)
    }

    companion object {
        const val DEFAULT_TITLE = "未命名文档"
    }
}
