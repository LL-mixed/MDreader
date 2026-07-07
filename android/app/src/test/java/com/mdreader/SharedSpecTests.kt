package com.mdreader

import com.mdreader.render.MermaidFenceNormalizer
import com.mdreader.util.ContentHash
import com.mdreader.util.Titles
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Data-driven tests driven by the shared spec files under shared/specs. The
 * input→output contract is defined ONCE in those JSON files and consumed by
 * Android, macOS, and Linux alike — so a spec edit propagates to all platforms.
 * Per-platform behavioral assertions live here; the data lives in shared/specs.
 */
class SharedSpecTests {

    @Test
    fun contentHashMatchesSpec() {
        SpecLoader.cases("content_hash").forEach { c ->
            val input = c.getString("input")
            val expected = c.getString("expected")
            assertEquals("sha256($input) mismatch", expected, ContentHash.sha256Hex(input))
        }
    }

    @Test
    fun titlesMatchesSpec() {
        SpecLoader.cases("titles").forEach { c ->
            val input = c.getString("input")
            val expected = c.getString("expected")
            assertEquals("Titles.fromPath($input) mismatch", expected, Titles.fromPath(input))
        }
    }

    @Test
    fun mermaidFenceMatchesSpec() {
        SpecLoader.cases("mermaid_fence").forEach { c ->
            val name = c.optString("name", "?")
            val input = c.getString("input")
            val expected = c.getString("expected")
            assertEquals("mermaid case '$name' mismatch", expected, MermaidFenceNormalizer.normalize(input))
        }
    }
}
