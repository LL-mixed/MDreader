package com.mdreader.util

import java.security.MessageDigest

/** SHA-256 content hashing used to deduplicate cached documents. */
object ContentHash {

    private val HEX = "0123456789abcdef".toCharArray()

    /** Returns the lowercase hex SHA-256 digest of [text] (UTF-8). */
    fun sha256Hex(text: String): String {
        val digest = MessageDigest.getInstance("SHA-256")
            .digest(text.toByteArray(Charsets.UTF_8))
        val sb = StringBuilder(digest.size * 2)
        for (b in digest) {
            val v = b.toInt() and 0xff
            sb.append(HEX[v ushr 4])
            sb.append(HEX[v and 0x0f])
        }
        return sb.toString()
    }
}
