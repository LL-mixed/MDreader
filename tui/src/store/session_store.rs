// SessionStore — port of macOS SessionStore.swift. Last-opened doc id for session restore.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Default)]
struct Snapshot {
    #[serde(rename = "lastDocID", default, skip_serializing_if = "Option::is_none")]
    last_doc_id: Option<Uuid>,
}

pub struct SessionStore {
    path: PathBuf,
    last_doc_id: Option<Uuid>,
}

impl SessionStore {
    pub fn open(dir: &Path) -> Self {
        let path = dir.join("session.json");
        let last_doc_id = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Snapshot>(&s).ok())
            .and_then(|snap| snap.last_doc_id);
        SessionStore { path, last_doc_id }
    }

    pub fn last_doc_id(&self) -> Option<Uuid> {
        self.last_doc_id
    }

    pub fn set_last_doc_id(&mut self, id: Option<Uuid>) {
        self.last_doc_id = id;
        let _ = fs::create_dir_all(self.path.parent().unwrap_or(Path::new(".")));
        if let Ok(data) = serde_json::to_string(&Snapshot { last_doc_id: self.last_doc_id }) {
            let _ = fs::write(&self.path, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mdreader-sess-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn round_trip_persists_id() {
        let dir = tmp();
        let id = Uuid::new_v4();
        {
            let mut s = SessionStore::open(&dir);
            assert_eq!(s.last_doc_id(), None);
            s.set_last_doc_id(Some(id));
        }
        let s = SessionStore::open(&dir);
        assert_eq!(s.last_doc_id(), Some(id));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clearing_writes_null() {
        let dir = tmp();
        let id = Uuid::new_v4();
        {
            let mut s = SessionStore::open(&dir);
            s.set_last_doc_id(Some(id));
            s.set_last_doc_id(None);
        }
        let s = SessionStore::open(&dir);
        assert_eq!(s.last_doc_id(), None);
        let _ = fs::remove_dir_all(&dir);
    }
}
