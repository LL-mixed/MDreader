// XDG base-directory paths. Linux equivalent of macOS's App Support / ~/.mdreader.

use std::path::PathBuf;

/// Cache + body storage: `$XDG_DATA_HOME/MDreader` (→ `~/.local/share/MDreader`).
pub fn data_dir() -> PathBuf {
    if let Ok(x) = std::env::var("XDG_DATA_HOME") {
        if !x.is_empty() {
            return PathBuf::from(x).join("MDreader");
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return PathBuf::from(h).join(".local").join("share").join("MDreader");
        }
    }
    PathBuf::from(".mdreader")
}

/// Settings / session / per-doc zoom: `$XDG_CONFIG_HOME/mdreader` (→ `~/.config/mdreader`).
pub fn config_dir() -> PathBuf {
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        if !x.is_empty() {
            return PathBuf::from(x).join("mdreader");
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return PathBuf::from(h).join(".config").join("mdreader");
        }
    }
    PathBuf::from(".mdreader-config")
}

/// `$XDG_DATA_HOME` root (→ `~/.local/share`). The on-disk icon theme lives under
/// `$XDG_DATA_HOME/icons/hicolor/<size>/apps/`; unlike `data_dir()`, this is NOT namespaced to
/// MDreader because the icon-theme spec requires that fixed layout. GTK4/GNOME read this on-disk
/// theme for the taskbar icon — not the in-process GResource that `add_resource_path` touches.
pub fn data_home() -> PathBuf {
    if let Ok(x) = std::env::var("XDG_DATA_HOME") {
        if !x.is_empty() {
            return PathBuf::from(x);
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return PathBuf::from(h).join(".local").join("share");
        }
    }
    PathBuf::from(".local").join("share")
}
