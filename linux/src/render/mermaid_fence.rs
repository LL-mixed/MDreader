// MermaidFenceNormalizer — port of macOS MermaidFenceNormalizer.swift.
// Rewrites non-standard fence tags (sequence/gantt/flow/…) or untagged fences whose first body
// line starts with a mermaid keyword into ```mermaid, preserving indent/marker/attrs.

use super::fence::{fence_match, trim_trailing_whitespace, FenceMatch};
use regex::Regex;
use std::sync::OnceLock;

const ALIAS: &[&str] = &[
    "mermaid", "sequence", "sequencediagram", "flow", "flowchart", "gantt", "class", "classdiagram",
    "state", "statediagram", "er", "erdiagram", "journey", "pie", "gitgraph", "mindmap", "timeline",
    "requirement", "requirementdiagram", "c4context", "c4container", "c4component", "packet", "kanban",
];

fn is_alias(tag_lower: &str) -> bool {
    ALIAS.iter().any(|a| *a == tag_lower)
}

fn keyword_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(graph|flowchart|sequenceDiagram|classDiagram|stateDiagram(-v2)?|erDiagram|gantt|pie|journey|gitGraph|requirementDiagram|requirement|C4Context|C4Container|C4Component|C4Dynamic|C4Deployment|mindmap|timeline|quadrantChart|xychart-beta|sankey-beta|block-beta|architecture-beta|packet|kanban)\b",
        ).unwrap()
    })
}

pub fn normalize(markdown: &str) -> String {
    if markdown.is_empty() {
        return markdown.to_string();
    }
    let mut lines: Vec<String> = markdown.split('\n').map(|s| s.to_string()).collect();
    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = trim_trailing_whitespace(&lines[i]);
        if let Some(fm) = fence_match(trimmed) {
            let marker_run = fm.marker.clone();
            let tag = fm.tag.clone();
            let first_body_line = if i + 1 < lines.len() {
                Some(lines[i + 1].clone())
            } else {
                None
            };
            if should_tag_as_mermaid(&tag, first_body_line.as_deref())
                && !tag.eq_ignore_ascii_case("mermaid")
            {
                lines[i] = rebuild_fence(&fm, "mermaid");
            }
            i = index_after_fence_body(&lines, i + 1, &marker_run);
        } else {
            i += 1;
        }
    }
    lines.join("\n")
}

fn should_tag_as_mermaid(tag: &str, first_body_line: Option<&str>) -> bool {
    if !tag.is_empty() {
        return is_alias(&tag.to_lowercase());
    }
    let first = match first_body_line {
        Some(s) => s,
        None => return false,
    };
    keyword_re().is_match(first.trim())
}

fn rebuild_fence(fm: &FenceMatch, new_tag: &str) -> String {
    let mut s = format!("{}{}{}", fm.indent, fm.marker, new_tag);
    if !fm.attrs.is_empty() {
        s.push(' ');
        s.push_str(&fm.attrs);
    }
    s
}

fn index_after_fence_body(lines: &[String], start: usize, marker: &str) -> usize {
    let mut j = start;
    while j < lines.len() {
        let trimmed = trim_trailing_whitespace(&lines[j]);
        if let Some(fm) = fence_match(trimmed) {
            if let (Some(mf), Some(cf)) = (marker.chars().next(), fm.marker.chars().next()) {
                if mf == cf && fm.marker.len() >= marker.len() && fm.tag.is_empty() {
                    return j + 1;
                }
            }
        }
        j += 1;
    }
    lines.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(md: &str) -> String {
        normalize(md)
    }

    #[test]
    fn standard_mermaid_fence_unchanged() {
        let src = "```mermaid\nflowchart LR\n  A --> B\n```";
        assert_eq!(n(src), src);
    }

    #[test]
    fn sequence_fence_rewritten() {
        assert_eq!(
            n("```sequence\nsequenceDiagram\n  A->>B: hi\n```"),
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```"
        );
    }

    #[test]
    fn alias_tag_case_insensitive() {
        assert_eq!(
            n("```Sequence\nsequenceDiagram\n  A->>B: hi\n```"),
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```"
        );
    }

    #[test]
    fn gantt_and_flow_aliases_rewrite() {
        assert_eq!(n("```gantt\ntitle X\n```"), "```mermaid\ntitle X\n```");
        assert_eq!(n("```flow\nflowchart TD\n```"), "```mermaid\nflowchart TD\n```");
    }

    #[test]
    fn tilde_fences_preserve_marker() {
        assert_eq!(n("~~~sequence\nsequenceDiagram\n```"), "~~~mermaid\nsequenceDiagram\n```");
    }

    #[test]
    fn untagged_block_with_keyword_rewrites() {
        assert_eq!(
            n("```\nsequenceDiagram\n  A->>B: hi\n```"),
            "```mermaid\nsequenceDiagram\n  A->>B: hi\n```"
        );
    }

    #[test]
    fn untagged_block_without_keyword_left_alone() {
        let src = "```\njust some plain text\nnot a diagram\n```";
        assert_eq!(n(src), src);
    }

    #[test]
    fn tagged_real_code_never_rewritten() {
        let src = "```kotlin\nflowchart fun build() = 1\n```";
        assert_eq!(n(src), src);
        let text = "```text\ngraph this is prose\n```";
        assert_eq!(n(text), text);
    }

    #[test]
    fn language_attribute_preserved() {
        assert_eq!(
            n("```sequence {#d}\nflowchart LR\n  A --> B\n```"),
            "```mermaid {#d}\nflowchart LR\n  A --> B\n```"
        );
    }

    #[test]
    fn leading_indent_up_to_three_spaces_preserved() {
        assert_eq!(n("  ```sequence\nflowchart LR\n  ```"), "  ```mermaid\nflowchart LR\n  ```");
    }

    #[test]
    fn fence_looking_lines_inside_code_block_not_rewritten() {
        let src = "```kotlin\nval s = \"```sequence\"\n```";
        assert_eq!(n(src), src);
    }

    #[test]
    fn multiple_mixed_blocks_handled_independently() {
        let src = "# Doc\n\n```sequence\nsequenceDiagram\n  A->>B: x\n```\n\n```kotlin\nfun main() {}\n```\n\n```gantt\ntitle T\n```";
        let expected = "# Doc\n\n```mermaid\nsequenceDiagram\n  A->>B: x\n```\n\n```kotlin\nfun main() {}\n```\n\n```mermaid\ntitle T\n```";
        assert_eq!(n(src), expected);
    }

    #[test]
    fn unterminated_block_rewrites_and_runs_to_eof() {
        assert_eq!(n("```sequence\nflowchart LR\n  A --> B"), "```mermaid\nflowchart LR\n  A --> B");
    }

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(n(""), "");
    }

    #[test]
    fn close_fence_shorter_than_opener_is_not_close() {
        let src = "````text\n```sequence\n````";
        assert_eq!(n(src), src);
    }
}
