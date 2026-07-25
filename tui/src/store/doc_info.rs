// DocInfo — port of macOS DocInfo.swift. An immutable value snapshot of a cached document,
// decoupled from the SQLite row so the UI holds plain data.

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocInfo {
    pub id: Uuid,
    pub title: String,
    pub content_hash: String,
    pub source_uri: Option<String>,
    pub opened_at: i64, // epoch millis
    pub favorite: bool,
    pub char_count: i64,
}
