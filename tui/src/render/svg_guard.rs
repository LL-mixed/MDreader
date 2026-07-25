// SvgGuard — port of macOS SvgGuard.swift.
// Lifts top-level `<svg>…</svg>` blocks out of markdown into `\u{1}<index>\u{2}` placeholders
// (restored by JS via getSvg), because marked truncates large inline SVGs at blank lines.
// Fence-aware: SVGs inside ```/~~~ code blocks are left untouched.

use super::fence::{fence_match, trim_trailing_whitespace};
use regex::Regex;
use std::sync::OnceLock;

pub const MARKER: char = '\u{0001}';
pub const END: char = '\u{0002}';

pub struct Guarded {
    pub markdown: String,
    pub svgs: Vec<String>,
}

pub fn placeholder(index: usize) -> String {
    format!("{}{}{}", MARKER, index, END)
}

fn svg_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<svg\b[\s\S]*?</svg>").unwrap())
}

pub fn protect(markdown: &str) -> Guarded {
    if !markdown.contains("<svg") {
        return Guarded { markdown: markdown.to_string(), svgs: vec![] };
    }
    let mut svgs: Vec<String> = Vec::new();
    let lines: Vec<&str> = markdown.split('\n').collect();
    let mut out = String::with_capacity(markdown.len());
    let mut i = 0usize;
    let mut in_fence = false;
    let mut fence_marker = String::new();
    while i < lines.len() {
        let line = lines[i];
        if let Some(fm) = fence_match(trim_trailing_whitespace(line)) {
            if !in_fence {
                in_fence = true;
                fence_marker = fm.marker.clone();
            } else if !fm.marker.is_empty()
                && !fence_marker.is_empty()
                && fm.marker.chars().next() == fence_marker.chars().next()
                && fm.marker.len() >= fence_marker.len()
            {
                in_fence = false;
                fence_marker.clear();
            }
            out.push_str(line);
            out.push('\n');
            i += 1;
            continue;
        }
        if in_fence {
            out.push_str(line);
            out.push('\n');
            i += 1;
            continue;
        }
        if line.contains("<svg") {
            let mut buf = line.to_string();
            let mut j = i;
            if !line.contains("</svg>") {
                j = i + 1;
                while j < lines.len() {
                    buf.push('\n');
                    buf.push_str(lines[j]);
                    if lines[j].contains("</svg>") {
                        break;
                    }
                    j += 1;
                }
            }
            let replaced = extract_svgs(&buf, &mut svgs);
            out.push_str(&replaced);
            out.push('\n');
            i = j + 1;
            continue;
        }
        out.push_str(line);
        out.push('\n');
        i += 1;
    }
    if out.ends_with('\n') {
        out.pop();
    }
    Guarded { markdown: out, svgs }
}

fn extract_svgs(text: &str, svgs: &mut Vec<String>) -> String {
    let mut result = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for m in svg_re().find_iter(text) {
        result.push_str(&text[cursor..m.start()]);
        svgs.push(m.as_str().to_string());
        result.push_str(&placeholder(svgs.len() - 1));
        cursor = m.end();
    }
    result.push_str(&text[cursor..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(md: &str) -> Guarded {
        protect(md)
    }

    #[test]
    fn markdown_without_svg_is_unchanged() {
        let src = "# Title\n\ntext **bold**\n\n```kotlin\nfun x() = 1\n```\n";
        let result = g(src);
        assert_eq!(result.markdown, src);
        assert!(result.svgs.is_empty());
    }

    #[test]
    fn single_one_line_svg_is_extracted() {
        let src = "before\n<svg id=\"a\"><rect/></svg>\nafter";
        let result = g(src);
        assert_eq!(result.svgs, vec!["<svg id=\"a\"><rect/></svg>"]);
        assert!(!result.markdown.contains("<svg"));
        let lines: Vec<&str> = result.markdown.split('\n').collect();
        assert_eq!(lines[1], placeholder(0));
    }

    #[test]
    fn large_svg_with_blank_lines_is_kept_intact() {
        let svg = "<svg viewBox=\"0 0 1400 1800\"><defs><linearGradient id=\"g1\"><stop/></linearGradient></defs>\n\n<g>\n<text>1940s</text>\n\n<text>2020s</text>\n</g>\n\n<!-- comment -->\n<text>x</text></svg>";
        let src = format!("intro\n\n{}\n\noutro", svg);
        let result = g(&src);
        assert_eq!(result.svgs, vec![svg]);
        assert!(!result.markdown.contains("<svg"));
        assert!(!result.markdown.contains("<rect"));
        assert!(!result.markdown.contains("</svg>"));
        assert!(result.markdown.contains("intro"));
        assert!(result.markdown.contains("outro"));
    }

    #[test]
    fn multiple_svgs_get_sequential_placeholders() {
        let src = "<svg>A</svg>\nmid\n<svg>B</svg>";
        let result = g(src);
        assert_eq!(result.svgs, vec!["<svg>A</svg>", "<svg>B</svg>"]);
        assert!(result.markdown.contains(&placeholder(0)));
        assert!(result.markdown.contains(&placeholder(1)));
    }

    #[test]
    fn svg_inside_fenced_code_block_is_not_extracted() {
        let src = "```xml\n<svg>kept as code</svg>\n```\n<svg>real one</svg>";
        let result = g(src);
        assert_eq!(result.svgs, vec!["<svg>real one</svg>"]);
        assert!(result.markdown.contains("<svg>kept as code</svg>"));
        assert!(!result.markdown.contains("<svg>real one</svg>"));
    }

    #[test]
    fn tilde_fence_also_protects_inner_svg() {
        let src = "~~~\n<svg>code</svg>\n~~~\n<svg>real</svg>";
        let result = g(src);
        assert_eq!(result.svgs, vec!["<svg>real</svg>"]);
        assert!(result.markdown.contains("<svg>code</svg>"));
    }

    #[test]
    fn placeholder_format_is_marker_index_end() {
        let result = g("<svg>x</svg>");
        let expected = format!("{}{}{}", MARKER, 0, END);
        assert_eq!(placeholder(0), expected);
        assert!(result.markdown.contains(&placeholder(0)));
    }

    #[test]
    fn text_after_closed_svg_on_same_line_is_preserved() {
        let src = "line\n<svg><rect/></svg>\ntail";
        let result = g(src);
        assert!(result.markdown.contains("tail"));
        assert_eq!(result.svgs, vec!["<svg><rect/></svg>"]);
    }
}
