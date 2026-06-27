// SettingsStore — port of macOS Settings.swift. Persists user preferences (currently just the
// external-editor command) as JSON under the XDG config dir. Mirrors ZoomStore/SessionStore:
// open(dir) -> Self; the caller wraps it in Arc<Mutex> for shared mutation.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// User-editable preferences. `editorCommand` mirrors macOS's Codable key name.
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq, Debug)]
pub struct AppSettings {
    #[serde(rename = "editorCommand", default)]
    pub editor_command: String,
}

pub struct SettingsStore {
    path: PathBuf,
    settings: AppSettings,
}

impl SettingsStore {
    pub fn open(dir: &Path) -> Self {
        let path = dir.join("config.json");
        let settings = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
            .unwrap_or_default();
        SettingsStore { path, settings }
    }

    pub fn editor_command(&self) -> &str {
        &self.settings.editor_command
    }

    pub fn set_editor_command(&mut self, command: String) {
        self.settings.editor_command = command;
        self.save();
    }

    fn save(&self) {
        let _ = fs::create_dir_all(self.path.parent().unwrap_or(Path::new(".")));
        if let Ok(data) = serde_json::to_string(&self.settings) {
            let _ = fs::write(&self.path, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mdreader-settings-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_defaults_to_empty_editor() {
        let dir = tmp();
        let s = SettingsStore::open(&dir);
        assert_eq!(s.editor_command(), "");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn round_trip_persists_editor_command() {
        let dir = tmp();
        {
            let mut s = SettingsStore::open(&dir);
            s.set_editor_command("Typora".to_string());
            assert_eq!(s.editor_command(), "Typora");
        }
        let s = SettingsStore::open(&dir);
        assert_eq!(s.editor_command(), "Typora");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn tolerant_of_corrupt_json() {
        let dir = tmp();
        fs::write(dir.join("config.json"), "{not valid").unwrap();
        let s = SettingsStore::open(&dir);
        assert_eq!(s.editor_command(), "");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let dir = tmp();
        fs::write(dir.join("config.json"), r#"{"editorCommand":"Code","futureKey":7}"#).unwrap();
        let s = SettingsStore::open(&dir);
        assert_eq!(s.editor_command(), "Code");
        let _ = fs::remove_dir_all(&dir);
    }
}
