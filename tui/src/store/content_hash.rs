// ContentHash — port of macOS ContentHash.swift. SHA-256 of UTF-8 bytes as lowercase hex.

use sha2::{Digest, Sha256};

/// (Used by the cache layer in LM4; exercised by tests now.)
#[allow(dead_code)]
pub fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        write!(out, "{:02x}", b).unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(
            sha256_hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn known_vector_abc() {
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn stable_for_same_input() {
        assert_eq!(sha256_hex("hello"), sha256_hex("hello"));
    }

    #[test]
    fn different_for_different_input() {
        assert_ne!(sha256_hex("a"), sha256_hex("b"));
    }

    #[test]
    fn output_is_64_lower_hex_chars() {
        let hex = sha256_hex("some markdown content");
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| "0123456789abcdef".contains(c)));
    }
}
