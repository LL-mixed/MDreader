// Persistence + content-addressing layer.
// - content_hash: SHA-256 hex
// - doc_info: immutable value snapshot of a cached doc (UI-facing)
// - doc_store: filesystem body storage (<uuid>.md)
// - cache: DocRepository — SQLite metadata + SHA-256 dedup + refresh-from-source
// - zoom_store: per-content-hash zoom map (JSON)
// - session_store: last-opened doc id (JSON)

pub mod cache;
pub mod content_hash;
pub mod doc_info;
pub mod doc_store;
pub mod session_store;
pub mod zoom_store;
