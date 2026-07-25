// DocStore — port of macOS DocStore.swift. Filesystem body storage: one `<uuid>.md` per doc.

use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn file_path(docs_dir: &Path, id: Uuid) -> std::path::PathBuf {
    docs_dir.join(format!("{}.md", id))
}

pub fn write(docs_dir: &Path, id: Uuid, markdown: &str) {
    let _ = fs::create_dir_all(docs_dir);
    let _ = fs::write(file_path(docs_dir, id), markdown);
}

pub fn read(docs_dir: &Path, id: Uuid) -> Option<String> {
    fs::read_to_string(file_path(docs_dir, id)).ok()
}

pub fn delete(docs_dir: &Path, id: Uuid) {
    let _ = fs::remove_file(file_path(docs_dir, id));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read_round_trip() {
        let dir = std::env::temp_dir().join(format!("ds-{}", uuid::Uuid::new_v4()));
        let id = Uuid::new_v4();
        write(&dir, id, "# hi\n");
        assert_eq!(read(&dir, id).as_deref(), Some("# hi\n"));
        delete(&dir, id);
        assert_eq!(read(&dir, id), None);
        let _ = fs::remove_dir_all(&dir);
    }
}
