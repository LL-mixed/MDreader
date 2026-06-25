package com.mdreader.util

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Test

class ContentHashTest {

    @Test
    fun emptyString() {
        assertEquals(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ContentHash.sha256Hex(""),
        )
    }

    @Test
    fun knownVectorAbc() {
        assertEquals(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ContentHash.sha256Hex("abc"),
        )
    }

    @Test
    fun stableForSameInput() {
        assertEquals(ContentHash.sha256Hex("hello"), ContentHash.sha256Hex("hello"))
    }

    @Test
    fun differentForDifferentInput() {
        assertNotEquals(ContentHash.sha256Hex("a"), ContentHash.sha256Hex("b"))
    }

    @Test
    fun outputIs64LowerHexChars() {
        val hex = ContentHash.sha256Hex("some markdown content")
        assertEquals(64, hex.length)
        assertFalse(hex.any { it !in '0'..'9' && it !in 'a'..'f' })
    }
}
