package com.mdreader.render

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Unit tests for [MermaidFenceNormalizer]. The normalizer is pure logic that
 * decides which fenced blocks are Mermaid diagrams and rewrites their opening
 * fence to the `mermaid` tag the WebView renderer recognises. These run on the
 * JVM with no WebView.
 */
class MermaidFenceNormalizerTest {

    private fun n(md: String) = MermaidFenceNormalizer.normalize(md)

    @Test fun `standard mermaid fence is left unchanged`() {
        val src = "```mermaid\nflowchart LR\n  A --> B\n```"
        assertEquals(src, n(src))
    }

    @Test fun `sequence fence is rewritten to mermaid`() {
        val src = "```sequence\nsequenceDiagram\n  A->>B: hi\n```"
        assertEquals(
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```",
            n(src),
        )
    }

    @Test fun `alias tag is case-insensitive`() {
        assertEquals(
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```",
            n("```Sequence\nsequenceDiagram\n  A->>B: hi\n```"),
        )
    }

    @Test fun `gantt and flow aliases rewrite`() {
        assertEquals("```mermaid\ntitle X\n```", n("```gantt\ntitle X\n```"))
        assertEquals("```mermaid\nflowchart TD\n```", n("```flow\nflowchart TD\n```"))
    }

    @Test fun `tilde fences rewrite and preserve marker`() {
        assertEquals("~~~mermaid\nsequenceDiagram\n```", n("~~~sequence\nsequenceDiagram\n```"))
    }

    @Test fun `untagged block with mermaid keyword rewrites`() {
        assertEquals(
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```",
            n("```\nsequenceDiagram\n  A->>B: hi\n```"),
        )
    }

    @Test fun `untagged block without keyword is left alone`() {
        val src = "```\njust some plain text\nnot a diagram\n```"
        assertEquals(src, n(src))
    }

    @Test fun `tagged real code is never rewritten even if content looks mermaid-ish`() {
        // ```kotlin whose body mentions flowchart must stay kotlin.
        val src = "```kotlin\nflowchart fun build() = 1\n```"
        assertEquals(src, n(src))
        // ```text starting with a keyword stays text.
        val text = "```text\ngraph this is prose\n```"
        assertEquals(text, n(text))
    }

    @Test fun `language attribute block is preserved`() {
        assertEquals(
            "```mermaid {#d}\nflowchart LR\n  A --> B\n```",
            n("```sequence {#d}\nflowchart LR\n  A --> B\n```"),
        )
    }

    @Test fun `leading indentation up to three spaces is preserved`() {
        assertEquals(
            "  ```mermaid\nflowchart LR\n  ```",
            n("  ```sequence\nflowchart LR\n  ```"),
        )
    }

    @Test fun `fence-looking lines inside a code block are not rewritten`() {
        // A ```sequence line that is the body of a ```kotlin block must survive.
        val src = "```kotlin\nval s = \"```sequence\"\n```"
        assertEquals(src, n(src))
    }

    @Test fun `multiple mixed blocks are handled independently`() {
        val src = """
            # Doc

            ```sequence
            sequenceDiagram
              A->>B: x
            ```

            ```kotlin
            fun main() {}
            ```

            ```gantt
            title T
            ```
        """.trimIndent()
        assertEquals(
            """
            # Doc

            ```mermaid
            sequenceDiagram
              A->>B: x
            ```

            ```kotlin
            fun main() {}
            ```

            ```mermaid
            title T
            ```
            """.trimIndent(),
            n(src),
        )
    }

    @Test fun `unterminated mermaid block rewrites opener and runs to EOF`() {
        assertEquals(
            "```mermaid\nflowchart LR\n  A --> B",
            n("```sequence\nflowchart LR\n  A --> B"),
        )
    }

    @Test fun `empty input returns empty`() {
        assertEquals("", n(""))
    }

    @Test fun `close fence shorter than opener is not treated as close`() {
        // 4-backtick opener must be closed by >= 4 backticks; a 3-backtick line
        // is body, so a later ```sequence inside is body too and stays as-is.
        val src = "````text\n```sequence\n````"
        assertEquals(src, n(src))
    }
}
