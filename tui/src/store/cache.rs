// DocRepository — port of macOS DocRepository.swift.
// SQLite metadata (one table) + `<uuid>.md` body files + SHA-256 content dedup + refresh-from-source.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection};
use uuid::Uuid;

use super::content_hash::sha256_hex;
use super::doc_info::DocInfo;
use super::doc_store;

pub const DEFAULT_TITLE: &str = "未命名文档";

const SCHEMA: &str = "\
CREATE TABLE IF NOT EXISTS cached_docs (\
    id TEXT PRIMARY KEY,\
    title TEXT NOT NULL,\
    content_hash TEXT NOT NULL,\
    source_uri TEXT,\
    char_count INTEGER NOT NULL,\
    size_bytes INTEGER NOT NULL,\
    cached_at INTEGER NOT NULL,\
    opened_at INTEGER NOT NULL,\
    favorite INTEGER NOT NULL DEFAULT 0\
);\
CREATE INDEX IF NOT EXISTS idx_content_hash ON cached_docs(content_hash);\
CREATE INDEX IF NOT EXISTS idx_opened_at ON cached_docs(opened_at);";

pub struct DocRepository {
    db: Mutex<Connection>,
    docs_dir: PathBuf,
}

impl DocRepository {
    /// Open (or create) the cache under `data_dir` (`cache.db` + `docs/`).
    pub fn open(data_dir: &Path) -> Result<Self, String> {
        let docs_dir = data_dir.join("docs");
        std::fs::create_dir_all(&docs_dir).map_err(|e| e.to_string())?;
        let conn = Connection::open(data_dir.join("cache.db")).map_err(|e| e.to_string())?;
        conn.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
        Ok(DocRepository {
            db: Mutex::new(conn),
            docs_dir,
        })
    }

    /// Insert or dedup. Returns the doc id (stable for identical content).
    pub fn cache(&self, title: &str, markdown: &str, source_uri: Option<&str>) -> Uuid {
        let hash = sha256_hex(markdown);
        let now = now_millis();
        let id = {
            let db = self.db.lock().unwrap();
            if let Ok(existing) = db.query_row(
                "SELECT id FROM cached_docs WHERE content_hash = ?1",
                params![hash],
                |r| r.get::<_, String>(0),
            ) {
                // Backfill source_uri when this open provides one the cached row lacks
                // (e.g. first cached via drop as None, now reopened from a file). COALESCE keeps
                // any existing source when this open has none, so a later drop never wipes a
                // recorded file path — relative image/SVG refs need the source dir to resolve.
                let _ = db.execute(
                    "UPDATE cached_docs SET opened_at = ?1, source_uri = COALESCE(?2, source_uri) WHERE id = ?3",
                    params![now, source_uri, existing],
                );
                Uuid::parse_str(&existing).unwrap_or_else(|_| Uuid::new_v4())
            } else {
                let resolved = if title.is_empty() {
                    DEFAULT_TITLE.to_string()
                } else {
                    title.to_string()
                };
                let id = Uuid::new_v4();
                let _ = db.execute(
                    "INSERT INTO cached_docs \
                     (id, title, content_hash, source_uri, char_count, size_bytes, cached_at, opened_at, favorite) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)",
                    params![
                        id.to_string(),
                        resolved,
                        hash,
                        source_uri,
                        markdown.chars().count() as i64,
                        markdown.len() as i64,
                        now,
                        now,
                    ],
                );
                id
            }
        };
        doc_store::write(&self.docs_dir, id, markdown);
        id
    }

    /// All docs, newest-opened first.
    pub fn all(&self) -> Vec<DocInfo> {
        let guard = self.db.lock().unwrap();
        let Ok(mut stmt) = guard.prepare(
            "SELECT id, title, content_hash, source_uri, opened_at, favorite, char_count \
             FROM cached_docs ORDER BY opened_at DESC",
        ) else {
            return vec![];
        };
        let rows = stmt.query_map([], |r| {
            Ok(DocRow {
                id: r.get::<_, String>(0)?,
                title: r.get::<_, String>(1)?,
                content_hash: r.get::<_, String>(2)?,
                source_uri: r.get::<_, Option<String>>(3)?,
                opened_at: r.get::<_, i64>(4)?,
                favorite: r.get::<_, i64>(5)?,
                char_count: r.get::<_, i64>(6)?,
            })
        });
        let Ok(rows) = rows else {
            return vec![];
        };
        rows.flatten()
            .filter_map(|r| {
                Uuid::parse_str(&r.id).ok().map(|id| DocInfo {
                    id,
                    title: r.title,
                    content_hash: r.content_hash,
                    source_uri: r.source_uri,
                    opened_at: r.opened_at,
                    favorite: r.favorite != 0,
                    char_count: r.char_count,
                })
            })
            .collect()
    }

    /// Case-insensitive substring search over titles.
    pub fn search(&self, query: &str) -> Vec<DocInfo> {
        let q = query.to_lowercase();
        self.all()
            .into_iter()
            .filter(|d| d.title.to_lowercase().contains(&q))
            .collect()
    }

    /// Bump openedAt and return the cached body.
    pub fn load_content(&self, id: Uuid) -> Option<String> {
        {
            let db = self.db.lock().unwrap();
            let _ = db.execute(
                "UPDATE cached_docs SET opened_at = ?1 WHERE id = ?2",
                params![now_millis(), id.to_string()],
            );
        }
        doc_store::read(&self.docs_dir, id)
    }

    pub fn set_favorite(&self, id: Uuid, favorite: bool) {
        let db = self.db.lock().unwrap();
        let _ = db.execute(
            "UPDATE cached_docs SET favorite = ?1 WHERE id = ?2",
            params![favorite as i64, id.to_string()],
        );
    }

    pub fn delete(&self, id: Uuid) {
        {
            let db = self.db.lock().unwrap();
            let _ = db.execute("DELETE FROM cached_docs WHERE id = ?1", params![id.to_string()]);
        }
        doc_store::delete(&self.docs_dir, id);
    }

    /// Re-read the original file backing `id`; if it exists and differs from the cached snapshot,
    /// update the cached content + metadata. Returns true when a refresh happened.
    pub fn refresh_from_source(&self, id: Uuid) -> bool {
        let (source_uri, current_hash) = {
            let db = self.db.lock().unwrap();
            let res = db.query_row(
                "SELECT source_uri, content_hash FROM cached_docs WHERE id = ?1",
                params![id.to_string()],
                |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, String>(1)?)),
            );
            match res {
                Ok(v) => v,
                Err(_) => return false,
            }
        };
        let Some(path) = source_uri else {
            return false;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return false;
        };
        let hash = sha256_hex(&text);
        if hash == current_hash {
            return false;
        }
        let now = now_millis();
        {
            let db = self.db.lock().unwrap();
            let _ = db.execute(
                "UPDATE cached_docs SET content_hash = ?1, char_count = ?2, size_bytes = ?3, opened_at = ?4 WHERE id = ?5",
                params![hash, text.chars().count() as i64, text.len() as i64, now, id.to_string()],
            );
        }
        doc_store::write(&self.docs_dir, id, &text);
        true
    }
}

struct DocRow {
    id: String,
    title: String,
    content_hash: String,
    source_uri: Option<String>,
    opened_at: i64,
    favorite: i64,
    char_count: i64,
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_repo() -> (DocRepository, PathBuf) {
        let dir = std::env::temp_dir().join(format!("mdreader-cache-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let repo = DocRepository::open(&dir).unwrap();
        (repo, dir)
    }

    #[test]
    fn cache_inserts_once_and_writes_file() {
        let (repo, dir) = make_repo();
        let id = repo.cache("Doc", "# Hi", None);
        repo.cache("Doc Again", "# Hi", None);
        assert_eq!(repo.all().len(), 1);
        assert_eq!(doc_store::read(&dir.join("docs"), id).as_deref(), Some("# Hi"));
    }

    #[test]
    fn cache_different_content_separate_rows() {
        let (repo, _dir) = make_repo();
        repo.cache("A", "aaa", None);
        repo.cache("B", "bbb", None);
        assert_eq!(repo.all().len(), 2);
    }

    #[test]
    fn empty_title_gets_default() {
        let (repo, _dir) = make_repo();
        repo.cache("", "x", None);
        assert_eq!(repo.all().first().unwrap().title, DEFAULT_TITLE);
    }

    #[test]
    fn delete_removes_row_and_file() {
        let (repo, dir) = make_repo();
        repo.cache("T", "body", None);
        let id = repo.all().first().unwrap().id;
        repo.delete(id);
        assert!(repo.all().is_empty());
        assert_eq!(doc_store::read(&dir.join("docs"), id), None);
    }

    #[test]
    fn cache_returns_stable_id_for_same_content() {
        let (repo, _dir) = make_repo();
        let id1 = repo.cache("Doc", "# Hi", None);
        let id2 = repo.cache("Doc Again", "# Hi", None);
        assert_eq!(id1, id2);
        assert_eq!(repo.all().first().unwrap().id, id1);
    }

    #[test]
    fn cache_dedup_backfills_missing_source_uri() {
        // A doc first cached without a source (e.g. dropped text) has source_uri = NULL. When the
        // same content is later opened from a file, the dedup hit must backfill source_uri —
        // otherwise relative image/SVG refs can't resolve on session restore (base = None).
        let (repo, _dir) = make_repo();
        repo.cache("Doc", "# Hi", None);
        repo.cache("Doc Again", "# Hi", Some("/path/to/doc.md"));
        let docs = repo.all();
        let doc = docs.first().unwrap();
        assert_eq!(doc.source_uri.as_deref(), Some("/path/to/doc.md"));
    }

    #[test]
    fn cache_dedup_keeps_existing_source_uri() {
        // A file-opened doc (source recorded) later re-cached via drop (no source) must NOT lose
        // its source_uri.
        let (repo, _dir) = make_repo();
        repo.cache("Doc", "# Hi", Some("/real/file.md"));
        repo.cache("Doc Again", "# Hi", None);
        let docs = repo.all();
        let doc = docs.first().unwrap();
        assert_eq!(doc.source_uri.as_deref(), Some("/real/file.md"));
    }

    fn write_source(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mdreader-src-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn refresh_from_source_updates_changed_content() {
        let (repo, _dir) = make_repo();
        let url = write_source("note.md", "# v1");
        let id = repo.cache("note", "# v1", Some(url.to_str().unwrap()));
        assert_eq!(repo.load_content(id).as_deref(), Some("# v1"));
        fs::write(&url, "# v2").unwrap();
        assert!(repo.refresh_from_source(id));
        assert_eq!(repo.load_content(id).as_deref(), Some("# v2"));
    }

    #[test]
    fn refresh_from_source_noop_when_unchanged() {
        let (repo, _dir) = make_repo();
        let url = write_source("note.md", "# same");
        let id = repo.cache("note", "# same", Some(url.to_str().unwrap()));
        assert!(!repo.refresh_from_source(id));
        assert_eq!(repo.load_content(id).as_deref(), Some("# same"));
    }

    #[test]
    fn refresh_from_source_false_when_no_source() {
        let (repo, _dir) = make_repo();
        let id = repo.cache("note", "# x", None);
        assert!(!repo.refresh_from_source(id));
    }

    #[test]
    fn refresh_from_source_false_when_source_missing() {
        let (repo, _dir) = make_repo();
        let id = repo.cache("note", "# x", Some("/nonexistent/path-1234567890.md"));
        assert!(!repo.refresh_from_source(id));
    }

    #[test]
    fn set_favorite_persists() {
        let (repo, _dir) = make_repo();
        let id = repo.cache("Fav", "x", None);
        repo.set_favorite(id, true);
        assert!(repo.all().first().unwrap().favorite);
        repo.set_favorite(id, false);
        assert!(!repo.all().first().unwrap().favorite);
    }

    #[test]
    fn search_matches_case_insensitively() {
        let (repo, _dir) = make_repo();
        repo.cache("Kotlin Notes", "a", None);
        repo.cache("Rust Guide", "b", None);
        assert_eq!(repo.search("kotlin").len(), 1);
        assert_eq!(repo.search("rust").len(), 1);
        assert_eq!(repo.search("notes").len(), 1);
        assert_eq!(repo.search("xyz").len(), 0);
    }
}
