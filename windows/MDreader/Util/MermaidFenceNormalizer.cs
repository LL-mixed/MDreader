using System.Text.RegularExpressions;

namespace MDreader.Util;

/// <summary>
/// MermaidFenceNormalizer — port of linux mermaid_fence.rs / macOS
/// MermaidFenceNormalizer.swift. Rewrites non-standard fence tags
/// (sequence/gantt/flow/…) or untagged fences whose first body line starts with
/// a mermaid keyword into <c>```mermaid</c>, preserving indent/marker/attrs.
/// </summary>
public static class MermaidFenceNormalizer
{
    private static readonly HashSet<string> Alias = new(StringComparer.OrdinalIgnoreCase)
    {
        "mermaid", "sequence", "sequencediagram", "flow", "flowchart", "gantt",
        "class", "classdiagram", "state", "statediagram", "er", "erdiagram",
        "journey", "pie", "gitgraph", "mindmap", "timeline",
        "requirement", "requirementdiagram",
        "c4context", "c4container", "c4component", "packet", "kanban",
    };

    private static readonly Regex KeywordRe = new(
        @"^(graph|flowchart|sequenceDiagram|classDiagram|stateDiagram(-v2)?|erDiagram|gantt|pie|journey|gitGraph|requirementDiagram|requirement|C4Context|C4Container|C4Component|C4Dynamic|C4Deployment|mindmap|timeline|quadrantChart|xychart-beta|sankey-beta|block-beta|architecture-beta|packet|kanban)\b",
        RegexOptions.Compiled);

    public static string Normalize(string markdown)
    {
        if (markdown.Length == 0) return markdown;
        var lines = markdown.Split('\n');
        int i = 0;
        while (i < lines.Length)
        {
            var trimmed = Fence.TrimTrailingWhitespace(lines[i]);
            var fm = Fence.MatchLine(trimmed);
            if (fm != null)
            {
                var markerRun = fm.Marker;
                var tag = fm.Tag;
                string? firstBody = (i + 1 < lines.Length) ? lines[i + 1] : null;
                if (ShouldTagAsMermaid(tag, firstBody)
                    && !tag.Equals("mermaid", StringComparison.OrdinalIgnoreCase))
                {
                    lines[i] = RebuildFence(fm, "mermaid");
                }
                i = IndexAfterFenceBody(lines, i + 1, markerRun);
            }
            else
            {
                i++;
            }
        }
        return string.Join("\n", lines);
    }

    private static bool ShouldTagAsMermaid(string tag, string? firstBodyLine)
    {
        if (tag.Length > 0) return Alias.Contains(tag);
        if (firstBodyLine == null) return false;
        return KeywordRe.IsMatch(firstBodyLine.Trim());
    }

    private static string RebuildFence(FenceMatch fm, string newTag)
    {
        var s = fm.Indent + fm.Marker + newTag;
        if (fm.Attrs.Length > 0)
        {
            s += " ";
            s += fm.Attrs;
        }
        return s;
    }

    private static int IndexAfterFenceBody(string[] lines, int start, string marker)
    {
        int j = start;
        while (j < lines.Length)
        {
            var trimmed = Fence.TrimTrailingWhitespace(lines[j]);
            var fm = Fence.MatchLine(trimmed);
            if (fm != null
                && marker.Length > 0 && fm.Marker.Length > 0
                && marker[0] == fm.Marker[0]
                && fm.Marker.Length >= marker.Length
                && fm.Tag.Length == 0)
            {
                return j + 1;
            }
            j++;
        }
        return lines.Length;
    }
}
