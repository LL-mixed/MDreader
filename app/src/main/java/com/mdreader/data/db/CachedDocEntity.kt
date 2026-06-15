package com.mdreader.data.db

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

/**
 * Metadata for a cached markdown document. The full text lives in internal
 * storage (see DocStore) keyed by [id]; this row holds only what the library
 * UI needs and the [contentHash] used to deduplicate by content.
 */
@Entity(
    tableName = "cached_docs",
    indices = [Index(value = ["content_hash"], unique = true)],
)
data class CachedDocEntity(
    @PrimaryKey(autoGenerate = true) val id: Long = 0,
    @ColumnInfo(name = "title") val title: String,
    @ColumnInfo(name = "content_hash") val contentHash: String,
    @ColumnInfo(name = "source_uri") val sourceUri: String?,
    @ColumnInfo(name = "char_count") val charCount: Int,
    @ColumnInfo(name = "size_bytes") val sizeBytes: Int,
    @ColumnInfo(name = "cached_at") val cachedAt: Long,
    @ColumnInfo(name = "opened_at") val openedAt: Long,
    @ColumnInfo(name = "favorite") val favorite: Boolean = false,
)
