package com.mdreader.render

/**
 * Normalizes Mermaid fenced blocks so the WebView renderer — which only
 * recognises the `mermaid` language tag — renders every diagram regardless of
 * the fence tag an author used.
 *
 * Many docs write the diagram type as the fence tag (```sequence, ```gantt,
 * ```flowchart), a legacy convention that predates Mermaid's standard
 * ```mermaid tag. Without normalization those blocks reach the syntax
 * highlighter as unknown code and render as plain text. This rewrites their
 * opening fence to ```mermaid (the body and closing fence are left untouched)
 * so the renderer handles them uniformly.
 *
 * A block is treated as Mermaid when its opening fence carries the `mermaid`
 * tag or a known alias, or — for UNTAGGED fences only — its first content line
 * opens with a Mermaid diagram-type keyword. Tagged non-Mermaid code
 * (```kotlin, ```js, ```text) is never rewritten, so real code is safe, and
 * fences nested inside another code block are skipped.
 *
 * Pure and side-effect free: unit-testable on the JVM without a WebView. This
 * is the single source of truth for Mermaid detection; render.js only has to
 * handle the `mermaid` tag.
 */
object MermaidFenceNormalizer {

    // CommonMark-ish opening fence: up to 3 leading spaces, a run of ` or ~,
    // an optional language tag, and an optional trailing attribute block.
    private val fenceRegex =
        Regex("""^([ \t]{0,3})(`{3,}|~{3,})[ \t]*([\w-]+)?[ \t]*(\{.*\})?[ \t]*$""")

    // Fence tags that unambiguously denote a Mermaid diagram (besides `mermaid`).
    private val alias = setOf(
        "mermaid", "sequence", "sequencediagram", "flow", "flowchart", "gantt",
        "class", "classdiagram", "state", "statediagram", "er", "erdiagram",
        "journey", "pie", "gitgraph", "mindmap", "timeline", "requirement",
        "requirementdiagram", "c4context", "c4container", "c4component", "packet", "kanban",
    )

    // First-line keywords that introduce a Mermaid diagram. Used only as a
    // fallback for UNTAGGED fences, so tagged code is never mistaken for one.
    private val keyword = Regex(
        """^(graph|flowchart|sequenceDiagram|classDiagram|stateDiagram(-v2)?|erDiagram|gantt|pie|journey|gitGraph|requirementDiagram|requirement|C4Context|C4Container|C4Component|C4Dynamic|C4Deployment|mindmap|timeline|quadrantChart|xychart-beta|sankey-beta|block-beta|architecture-beta|packet|kanban)\b""",
    )

    /**
     * Returns [markdown] with every Mermaid block's opening fence tagged
     * `mermaid`. Lines that are not Mermaid fences are returned unchanged.
     */
    fun normalize(markdown: String): String {
        if (markdown.isEmpty()) return markdown
        val lines = markdown.split("\n").toMutableList()
        var i = 0
        while (i < lines.size) {
            val match = fenceRegex.matchEntire(lines[i].trimEnd())
            if (match != null) {
                val marker = match.groupValues[2]
                val tag = match.groupValues[3]
                if (shouldTagAsMermaid(tag, lines.getOrNull(i + 1)) &&
                    !tag.equals("mermaid", ignoreCase = true)
                ) {
                    lines[i] = rebuildFence(match, newTag = "mermaid")
                }
                // Advance past the fenced body so tags inside code are never
                // rewritten and a close fence is not mistaken for an opener.
                i = indexAfterFenceBody(lines, startIndex = i + 1, marker = marker)
            } else {
                i++
            }
        }
        return lines.joinToString("\n")
    }

    private fun shouldTagAsMermaid(tag: String, firstBodyLine: String?): Boolean {
        if (tag.isNotEmpty()) return alias.contains(tag.lowercase())
        // Untagged fence: only treat as Mermaid when the first body line opens
        // with a diagram-type keyword.
        return firstBodyLine != null && keyword.containsMatchIn(firstBodyLine.trim())
    }

    private fun rebuildFence(match: MatchResult, newTag: String): String {
        val indent = match.groupValues[1]
        val marker = match.groupValues[2]
        val attrs = match.groupValues[4]
        return buildString {
            append(indent).append(marker).append(newTag)
            if (attrs.isNotEmpty()) append(' ').append(attrs)
        }
    }

    // Returns the index just past the fenced block starting at [startIndex]
    // (after its closing fence), or [lines].size if the block runs to EOF.
    private fun indexAfterFenceBody(lines: List<String>, startIndex: Int, marker: String): Int {
        var j = startIndex
        while (j < lines.size) {
            val m = fenceRegex.matchEntire(lines[j].trimEnd())
            if (m != null) {
                val thisMarker = m.groupValues[2]
                val thisTag = m.groupValues[3]
                if (thisMarker.startsWith(marker[0]) &&
                    thisMarker.length >= marker.length &&
                    thisTag.isEmpty()
                ) {
                    return j + 1
                }
            }
            j++
        }
        return lines.size
    }
}
