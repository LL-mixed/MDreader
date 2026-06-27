// Image resolution — port of macOS MarkdownWebView.resolveImages.
// Rewrites relative `![alt](rel)` URLs against the file's directory: leaves http(s)://, /abs,
// file:, # untouched; inlines `.svg` as raw SVG text (SvgGuard then lifts it); base64-encodes
// raster types into data: URIs. Runs before the markdown reaches JS.

use base64::Engine;
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

fn img_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(!\[[^\]]*\]\()([^)]+)(\))").unwrap())
}

pub fn resolve_images(markdown: &str, base_dir: Option<&Path>) -> String {
    let Some(base) = base_dir else {
        return markdown.to_string();
    };
    let re = img_re();
    let mut result = String::with_capacity(markdown.len());
    let mut last = 0usize;
    for caps in re.captures_iter(markdown) {
        let m = caps.get(0).unwrap();
        let g1 = caps.get(1).unwrap().as_str();
        let original = caps.get(2).unwrap().as_str();
        let g3 = caps.get(3).unwrap().as_str();
        result.push_str(&markdown[last..m.start()]);

        // src = original up to the first space (Swift: firstIndex(of: " "))
        let src = match original.find(' ') {
            Some(idx) => &original[..idx],
            None => original,
        };

        if src.starts_with("http://")
            || src.starts_with("https://")
            || src.starts_with("/")
            || src.starts_with("file:")
            || src.starts_with("#")
        {
            result.push_str(g1);
            result.push_str(original);
            result.push_str(g3);
        } else {
            let abs = base.join(src);
            let ext = Path::new(src)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
                .unwrap_or_default();
            if ext == "svg" {
                match std::fs::read_to_string(&abs) {
                    Ok(svg) => {
                        result.push_str("\n\n");
                        result.push_str(&svg);
                        result.push_str("\n\n");
                    }
                    Err(_) => {
                        result.push_str(g1);
                        result.push_str(original);
                        result.push_str(g3);
                    }
                }
            } else {
                let mime = match ext.as_str() {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "webp" => "image/webp",
                    _ => "application/octet-stream",
                };
                match std::fs::read(&abs) {
                    Ok(data) => {
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                        result.push_str(g1);
                        result.push_str(&format!("data:{};base64,{}", mime, b64));
                        result.push_str(g3);
                    }
                    Err(_) => {
                        result.push_str(g1);
                        result.push_str(original);
                        result.push_str(g3);
                    }
                }
            }
        }
        last = m.end();
    }
    result.push_str(&markdown[last..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mdreader-preprocess-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn no_base_dir_returns_unchanged() {
        let md = "![x](rel.png)";
        assert_eq!(resolve_images(md, None), md);
    }

    #[test]
    fn absolute_and_remote_urls_left_alone() {
        let md = "![a](http://e/a.png) ![b](https://e/b.png) ![c](/abs/c.png) ![d](#anchor)";
        assert_eq!(resolve_images(md, Some(Path::new("/tmp"))), md);
    }

    #[test]
    fn raster_image_is_inlined_as_data_uri() {
        let dir = tmp_dir();
        // 1x1 transparent PNG
        let png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        ];
        fs::write(dir.join("pixel.png"), png).unwrap();
        let out = resolve_images("![p](pixel.png)", Some(&dir));
        assert!(out.contains("![p](data:image/png;base64,"));
        assert!(!out.contains("pixel.png)"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn svg_image_is_inlined_as_raw_svg() {
        let dir = tmp_dir();
        fs::write(dir.join("d.svg"), "<svg><rect/></svg>").unwrap();
        let out = resolve_images("![d](d.svg)", Some(&dir));
        assert!(out.contains("\n\n<svg><rect/></svg>\n\n"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_falls_back_to_original_src() {
        let dir = tmp_dir();
        let out = resolve_images("![m](nope.png)", Some(&dir));
        assert_eq!(out, "![m](nope.png)");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn title_after_src_is_preserved_on_remote() {
        let md = "![a](http://e/a.png \"t\")";
        assert_eq!(resolve_images(md, Some(Path::new("/tmp"))), md);
    }
}
