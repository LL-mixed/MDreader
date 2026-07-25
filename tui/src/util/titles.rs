// Titles — port of macOS Titles.swift. Derives a display title from a file path,
// stripping a trailing markdown extension.

const MARKDOWN_EXTS: &[&str] = &["md", "markdown", "mdown", "mkd", "mkdown"];

pub fn from_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let name_start = path.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
    let name = &path[name_start..];
    let Some(dot) = name.rfind('.') else {
        return name.to_string();
    };
    if dot == 0 {
        return name.to_string();
    }
    let ext = name[dot + 1..].to_lowercase();
    if MARKDOWN_EXTS.contains(&ext.as_str()) {
        return name[..dot].to_string();
    }
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markdown_extension() {
        assert_eq!(from_path("readme.md"), "readme");
    }

    #[test]
    fn ignores_extension_case() {
        assert_eq!(from_path("/a/b/Notes.MARKDOWN"), "Notes");
    }

    #[test]
    fn handles_multiple_dots() {
        assert_eq!(from_path("a.b.md"), "a.b");
    }

    #[test]
    fn preserves_non_markdown_extension() {
        assert_eq!(from_path("archive.txt"), "archive.txt");
    }

    #[test]
    fn no_extension_returned_as_is() {
        assert_eq!(from_path("noext"), "noext");
    }

    #[test]
    fn empty_path_returns_empty() {
        assert_eq!(from_path(""), "");
    }

    #[test]
    fn handles_mdown() {
        assert_eq!(from_path("WeChat Files/doc.mdown"), "doc");
    }

    #[test]
    fn handles_backslash_separator() {
        assert_eq!(from_path("C:\\Users\\me\\file.md"), "file");
    }
}
