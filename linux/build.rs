// Embeds ../../shared/render/** (+ sample.md) into the binary via GResource so the
// physical single source of truth (shared/) is bundled unchanged. The manifest is
// generated at build time by walking shared/render, so new files (e.g. KaTeX fonts)
// are picked up automatically — no hand-maintained list.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let shared = manifest
        .join("..")
        .join("shared")
        .canonicalize()
        .expect("linux/build.rs: ../shared not found (shared/ must be sibling of linux/)");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Walk shared/render; register each file under prefix /com/mdreader/MDreader/render/
    // using its bare relative path, so the resource URI of each file equals the path
    // render.js/index.html reference relatively (resource path == relative ref).
    let mut entries: Vec<String> = Vec::new();
    walk(&shared.join("render"), &shared, &mut entries);
    if shared.join("sample.md").exists() {
        entries.push("sample.md".to_string());
    }
    entries.sort();

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<gresources>\n");
    xml.push_str("  <gresource prefix=\"/com/mdreader/MDreader\">\n");
    for e in &entries {
        xml.push_str(&format!("    <file>{}</file>\n", e));
    }
    xml.push_str("  </gresource>\n");
    xml.push_str("</gresources>\n");

    let xml_path = out_dir.join("render.gresource.xml");
    fs::write(&xml_path, &xml).unwrap();

    glib_build_tools::compile_resources(
        &[shared.to_str().unwrap()],
        xml_path.to_str().unwrap(),
        "render.gresource",
    );

    println!("cargo:rerun-if-changed={}", shared.join("render").display());
    println!("cargo:rerun-if-changed={}", shared.join("sample.md").display());
    println!("cargo:rerun-if-changed=build.rs");
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            walk(&p, root, out);
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}
