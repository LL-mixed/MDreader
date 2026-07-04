// ThemeStore — per-content-hash dark override map, persisted to JSON. Mirrors ZoomStore exactly.
// A doc lands here only when the user toggles its theme; absence means "follow the global default".

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ThemeStore {
    path: PathBuf,
    map: HashMap<String, bool>,
}

impl ThemeStore {
    pub fn open(dir: &Path) -> Self {
        let path = dir.join("theme.json");
        let map = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<HashMap<String, bool>>(&s).ok())
            .unwrap_or_default();
        ThemeStore { path, map }
    }

    pub fn dark_for(&self, hash: &str) -> Option<bool> {
        self.map.get(hash).copied()
    }

    pub fn set_dark(&mut self, dark: bool, hash: &str) {
        self.map.insert(hash.to_string(), dark);
        self.save();
    }

    fn save(&self) {
        let _ = fs::create_dir_all(self.path.parent().unwrap_or(Path::new(".")));
        if let Ok(data) = serde_json::to_string(&self.map) {
            let _ = fs::write(&self.path, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mdreader-theme-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn round_trip_persists_per_hash() {
        let dir = tmp();
        {
            let mut t = ThemeStore::open(&dir);
            assert_eq!(t.dark_for("aaa"), None);
            t.set_dark(true, "aaa");
            t.set_dark(false, "bbb");
            assert_eq!(t.dark_for("aaa"), Some(true));
        }
        let t = ThemeStore::open(&dir);
        assert_eq!(t.dark_for("aaa"), Some(true));
        assert_eq!(t.dark_for("bbb"), Some(false));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_is_empty() {
        let dir = tmp();
        let t = ThemeStore::open(&dir);
        assert_eq!(t.dark_for("x"), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn overwrite_updates_value() {
        let dir = tmp();
        let mut t = ThemeStore::open(&dir);
        t.set_dark(true, "h");
        t.set_dark(false, "h");
        assert_eq!(t.dark_for("h"), Some(false));
        let _ = fs::remove_dir_all(&dir);
    }
}
