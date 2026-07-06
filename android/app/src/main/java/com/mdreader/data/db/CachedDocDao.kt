package com.mdreader.data.db

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface CachedDocDao {

    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insert(entity: CachedDocEntity): Long

    @Query("SELECT * FROM cached_docs WHERE content_hash = :hash LIMIT 1")
    suspend fun findByHash(hash: String): CachedDocEntity?

    @Query("UPDATE cached_docs SET opened_at = :timestamp WHERE id = :id")
    suspend fun touchOpenedAt(id: Long, timestamp: Long)

    @Query("SELECT * FROM cached_docs ORDER BY opened_at DESC")
    fun observeAll(): Flow<List<CachedDocEntity>>

    @Query("SELECT * FROM cached_docs WHERE title LIKE '%' || :query || '%' ORDER BY opened_at DESC")
    fun search(query: String): Flow<List<CachedDocEntity>>

    @Query("SELECT * FROM cached_docs WHERE id = :id LIMIT 1")
    suspend fun getById(id: Long): CachedDocEntity?

    /** Updates the cached content metadata after a source refresh. */
    @Query(
        "UPDATE cached_docs SET content_hash = :hash, char_count = :charCount, " +
            "size_bytes = :sizeBytes, opened_at = :timestamp WHERE id = :id"
    )
    suspend fun updateContent(id: Long, hash: String, charCount: Int, sizeBytes: Int, timestamp: Long)

    @Query("UPDATE cached_docs SET favorite = :favorite WHERE id = :id")
    suspend fun setFavorite(id: Long, favorite: Boolean)

    @Query("DELETE FROM cached_docs WHERE id = :id")
    suspend fun deleteById(id: Long)
}
