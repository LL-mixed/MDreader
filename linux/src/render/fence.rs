// CommonMark fence-line matcher — port of macOS Fence.swift.
// Matches only when the ENTIRE line is a fence opener/closer.

use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceMatch {
    pub indent: String,
    pub marker: String,
    pub tag: String,
    pub attrs: String,
}

fn fence_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^([ \t]{0,3})(`{3,}|~{3,})[ \t]*([\w-]+)?[ \t]*(\{.*\})?[ \t]*$").unwrap()
    })
}

/// Returns the fence components iff the whole line is a fence line.
pub fn fence_match(line: &str) -> Option<FenceMatch> {
    let caps = fence_re().captures(line)?;
    let whole = caps.get(0)?;
    if whole.start() != 0 || whole.end() != line.len() {
        return None;
    }
    Some(FenceMatch {
        indent: caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default(),
        marker: caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default(),
        tag: caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default(),
        attrs: caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_default(),
    })
}

/// Port of Swift `trimTrailingWhitespace` — strips trailing Unicode whitespace.
pub fn trim_trailing_whitespace(s: &str) -> &str {
    s.trim_end()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_backtick_fence_with_tag() {
        let m = fence_match("```kotlin").unwrap();
        assert_eq!(m.marker, "```");
        assert_eq!(m.tag, "kotlin");
        assert_eq!(m.indent, "");
        assert_eq!(m.attrs, "");
    }

    #[test]
    fn matches_tilde_fence_with_indent_and_attrs() {
        let m = fence_match("  ~~~mermaid {#d}").unwrap();
        assert_eq!(m.indent, "  ");
        assert_eq!(m.marker, "~~~");
        assert_eq!(m.tag, "mermaid");
        assert_eq!(m.attrs, "{#d}");
    }

    #[test]
    fn matches_bare_closer() {
        let m = fence_match("````").unwrap();
        assert_eq!(m.marker, "````");
        assert_eq!(m.tag, "");
    }

    #[test]
    fn rejects_non_fence_line() {
        assert!(fence_match("just text").is_none());
        assert!(fence_match("# heading").is_none());
        // only two backticks is not a fence
        assert!(fence_match("``kotlin").is_none());
    }
}
