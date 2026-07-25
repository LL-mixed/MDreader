//! MDreader TUI — cross-platform terminal Markdown reader (macOS/Linux).
//!
//! Self-contained crate: bundles its own data layer (SQLite cache + JSON stores)
//! so it runs anywhere without depending on a GUI app's data directory. The
//! markdown rendering pipeline uses pulldown-cmark → ratatui styled lines.

use std::sync::{Arc, Mutex};

use mdreader_core::context::AppContext;
use mdreader_core::store::cache::DocRepository;
use mdreader_core::store::session_store::SessionStore;
use mdreader_core::store::settings_store::SettingsStore;
use mdreader_core::store::theme_store::ThemeStore;
use mdreader_core::store::zoom_store::ZoomStore;
use mdreader_core::tui::app::App;
use mdreader_core::util::titles;
use mdreader_core::{config, tui};

fn main() {
    let ctx = Arc::new(AppContext {
        repo: Arc::new(
            DocRepository::open(&config::data_dir()).expect("failed to open cache db"),
        ),
        zoom_store: Arc::new(Mutex::new(ZoomStore::open(&config::config_dir()))),
        theme_store: Arc::new(Mutex::new(ThemeStore::open(&config::config_dir()))),
        session_store: Arc::new(Mutex::new(SessionStore::open(&config::config_dir()))),
        settings: Arc::new(Mutex::new(SettingsStore::open(&config::config_dir()))),
    });

    let args: Vec<String> = std::env::args().collect();
    if let Some(path) = args.get(1) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let title = titles::from_path(path);
            let cached = ctx.repo.cache(&title, &content, Some(path));
            let mut app = App::new(ctx);
            if let Some(idx) = app.docs.iter().position(|d| d.id == cached) {
                app.list_state.select(Some(idx));
                app.open_doc(idx);
            }
            run(app);
            return;
        }
    }

    let mut app = App::new(ctx.clone());
    if let Some(id) = ctx.session_store.lock().unwrap().last_doc_id() {
        if let Some(idx) = app.docs.iter().position(|d| d.id == id) {
            app.open_doc(idx);
        }
    }
    run(app);
}

fn run(app: App) {
    if let Err(e) = tui::app::run(app) {
        eprintln!("mdreader-tui: {e}");
        std::process::exit(1);
    }
}
