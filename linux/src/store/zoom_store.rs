// ZoomStore — port of macOS ZoomStore.swift. Per-content-hash zoom map, persisted to JSON.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ZoomStore {
    path: PathBuf,
    map: HashMap<String, f64>,
}

impl ZoomStore {
    pub fn open(dir: &Path) -> Self {
        let path = dir.join("zoom.json");
        let map = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<HashMap<String, f64>>(&s).ok())
            .unwrap_or_default();
        ZoomStore { path, map }
    }

    pub fn zoom_for(&self, hash: &str) -> Option<f64> {
        self.map.get(hash).copied()
    }

    pub fn set_zoom(&mut self, zoom: f64, hash: &str) {
        self.map.insert(hash.to_string(), zoom);
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
        let dir = std::env::temp_dir().join(format!("mdreader-zoom-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn round_trip_persists_per_hash() {
        let dir = tmp();
        {
            let mut z = ZoomStore::open(&dir);
            assert_eq!(z.zoom_for("aaa"), None);
            z.set_zoom(1.5, "aaa");
            z.set_zoom(0.75, "bbb");
            assert_eq!(z.zoom_for("aaa"), Some(1.5));
        }
        // reopen -> persisted
        let z = ZoomStore::open(&dir);
        assert_eq!(z.zoom_for("aaa"), Some(1.5));
        assert_eq!(z.zoom_for("bbb"), Some(0.75));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_is_empty() {
        let dir = tmp();
        let z = ZoomStore::open(&dir);
        assert_eq!(z.zoom_for("x"), None);
        let _ = fs::remove_dir_all(&dir);
    }
}
