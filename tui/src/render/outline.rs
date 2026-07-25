// Outline item — decode of render.js's onOutline payload [{index, level, text}].
// Mirrors macOS OutlineItem.swift.

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OutlineItem {
    pub index: usize,
    pub level: u32,
    pub text: String,
}

/// Decode the JSON string render.js posts via onOutline.
/// (Wired into the sidebar in LM5; exercised by tests now.)
#[allow(dead_code)]
pub fn parse_outline(json: &str) -> Option<Vec<OutlineItem>> {
    serde_json::from_str(json).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_payload() {
        let json = r#"[
            {"index":0,"level":1,"text":"Title"},
            {"index":1,"level":2,"text":"Section"},
            {"index":2,"level":3,"text":"子标题"}
        ]"#;
        let items = parse_outline(json).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].text, "Title");
        assert_eq!(items[1].level, 2);
        assert_eq!(items[2].text, "子标题");
    }

    #[test]
    fn empty_array() {
        let items = parse_outline("[]").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(parse_outline("not json").is_none());
    }
}
