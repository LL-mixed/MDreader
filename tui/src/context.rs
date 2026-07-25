//! Process-wide store bundle shared across windows/TUI sessions. Pure data —
//! no GUI types — so both the GTK app and the TUI can hold it.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::store::{
    cache::DocRepository, session_store::SessionStore, settings_store::SettingsStore,
    theme_store::ThemeStore, zoom_store::ZoomStore,
};

/// Process-wide stores shared across windows.
pub struct AppContext {
    pub repo: Arc<DocRepository>,
    pub zoom_store: Arc<Mutex<ZoomStore>>,
    pub theme_store: Arc<Mutex<ThemeStore>>,
    pub session_store: Arc<Mutex<SessionStore>>,
    pub settings: Arc<Mutex<SettingsStore>>,
}

/// What to show when a window/TUI session opens.
pub enum InitialDoc {
    Sample,
    File {
        content: String,
        title: String,
        base: Option<PathBuf>,
        source: Option<String>,
    },
    Cached(Uuid),
}
