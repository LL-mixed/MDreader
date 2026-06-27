// Markdown extension whitelist — mirrors macOS ReaderModel.openText's allowed set
// {md, markdown, mdown, mkd, mkdown}. Gates drag-drop and GApplication::open so non-markdown
// files are ignored rather than blindly cached.

const MARKDOWN_EXTS: &[&str] = &["md", "markdown", "mdown", "mkd", "mkdown"];

/// True if `path`'s extension is a recognized markdown extension (case-insensitive).
pub fn is_markdown(path: &str) -> bool {
    match ext_of(path) {
        Some(e) => MARKDOWN_EXTS.iter().any(|&allowed| e == allowed),
        None => false,
    }
}

fn ext_of(path: &str) -> Option<String> {
    let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let dot = file.rfind('.')?;
    let e = &file[dot + 1..];
    if e.is_empty() {
        return None;
    }
    Some(e.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_all_canonical_exts_case_insensitively() {
        for e in ["file.md", "a.MARKDOWN", "b.Mdown", "c.mkd", "d.mkdown", "e.MD"] {
            assert!(is_markdown(e), "{e} should be markdown");
        }
    }

    #[test]
    fn rejects_non_markdown() {
        assert!(!is_markdown("note.txt"));
        assert!(!is_markdown("doc.html"));
        assert!(!is_markdown("readme"));
        assert!(!is_markdown(".gitignore"));
    }

    #[test]
    fn handles_paths_and_backslashes() {
        assert!(is_markdown("/home/u/docs/note.md"));
        assert!(is_markdown("C:\\users\\u\\note.markdown"));
    }

    #[test]
    fn trailing_dot_or_empty_ext_is_not_markdown() {
        assert!(!is_markdown("file."));
        assert!(!is_markdown("name"));
    }
}
