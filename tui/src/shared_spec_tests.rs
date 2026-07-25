//! Data-driven tests driven by the shared spec files under `shared/specs`. The
//! input→output contract is defined ONCE in those JSON files and consumed by
//! Android, macOS, and Linux alike — so a spec edit propagates to all platforms.
//!
//! Wired into the build via `#[cfg(test)] mod shared_spec_tests;` in main.rs.
//! `cargo test` runs from the crate root (linux/), so the specs resolve via
//! `CARGO_MANIFEST_DIR/../shared/specs/`.

#[cfg(test)]
use serde_json::Value;

/// Loads `shared/specs/<name>.json`'s `cases` array.
#[cfg(test)]
fn load_cases(name: &str) -> Vec<Value> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("shared/specs")
        .join(format!("{name}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("spec not found at {}: {e}", path.display()));
    let json: Value = serde_json::from_str(&text).expect("invalid spec JSON");
    json["cases"]
        .as_array()
        .expect("spec has no 'cases' array")
        .clone()
}

#[cfg(test)]
mod tests {
    use super::load_cases;
    use crate::render::mermaid_fence::normalize;
    use crate::store::content_hash::sha256_hex;
    use crate::util::titles::from_path;

    #[test]
    fn content_hash_matches_spec() {
        for c in load_cases("content_hash") {
            let input = c["input"].as_str().unwrap();
            let expected = c["expected"].as_str().unwrap();
            assert_eq!(expected, sha256_hex(input), "sha256({input:?})");
        }
    }

    #[test]
    fn titles_matches_spec() {
        for c in load_cases("titles") {
            let input = c["input"].as_str().unwrap();
            let expected = c["expected"].as_str().unwrap();
            assert_eq!(expected, from_path(input), "Titles.fromPath({input:?})");
        }
    }

    #[test]
    fn mermaid_fence_matches_spec() {
        for c in load_cases("mermaid_fence") {
            let name = c["name"].as_str().unwrap_or("?");
            let input = c["input"].as_str().unwrap();
            let expected = c["expected"].as_str().unwrap();
            assert_eq!(expected, normalize(input), "mermaid case '{name}'");
        }
    }
}
