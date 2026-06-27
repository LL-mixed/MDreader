// Compile-time build metadata, mirroring macOS BuildInfo.swift (gitHash/buildTime/author,
// fallback "dev"). Populated by build.rs via `cargo:rustc-env` and read here with option_env!.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Short git hash at build time ("dev" if unset or unavailable).
pub fn git_hash() -> &'static str {
    match option_env!("GIT_HASH") {
        Some(v) if !v.is_empty() => v,
        _ => "dev",
    }
}

/// UTC build timestamp ("dev" if unset or unavailable).
pub fn build_time() -> &'static str {
    match option_env!("BUILD_TIME") {
        Some(v) if !v.is_empty() => v,
        _ => "dev",
    }
}

/// "v{VERSION} ({GIT_HASH})" — the compact version line shown in About.
pub fn version_line() -> String {
    format!("v{} ({})", VERSION, git_hash())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_line_is_prefixed_and_carries_hash() {
        let v = version_line();
        assert!(v.starts_with('v'), "{v} should start with 'v'");
        assert!(v.contains(git_hash()), "{v} should contain the git hash");
    }

    #[test]
    fn fields_are_non_empty() {
        // VERSION is a cargo-guaranteed non-empty const; only the option_env! fallbacks are
        // environment-dependent and worth asserting.
        assert!(!git_hash().is_empty());
        assert!(!build_time().is_empty());
    }
}
