// MDreader TUI — terminal Markdown reader. Shares the data layer (cache,
// session, theme) with the GTK GUI via `mdreader_core`. Builds without GTK.

use std::sync::{Arc, Mutex};

use mdreader_core::context::AppContext;
use mdreader_core::store::cache::DocRepository;
use mdreader_core::store::session_store::SessionStore;
use mdreader_core::store::settings_store::SettingsStore;
use mdreader_core::store::theme_store::ThemeStore;
use mdreader_core::store::zoom_store::ZoomStore;
use mdreader_core::tui::app::App;
use mdreader_core::{config, util};

fn main() {
    let ctx = Arc::new(AppContext {
        repo: Arc::new(
            DocRepository::open(&config::data_dir())
                .expect("failed to open cache db"),
        ),
        zoom_store: Arc::new(Mutex::new(ZoomStore::open(&config::config_dir()))),
        theme_store: Arc::new(Mutex::new(ThemeStore::open(&config::config_dir()))),
        session_store: Arc::new(Mutex::new(SessionStore::open(&config::config_dir()))),
        settings: Arc::new(Mutex::new(SettingsStore::open(&config::config_dir()))),
    });

    // If a file argument is given, cache it and open directly.
    let args: Vec<String> = std::env::args().collect();
    if let Some(path) = args.get(1) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let title = util::titles::from_path(path);
            let cached = ctx.repo.cache(&title, &content, Some(path));
            // the App will open it by finding it in the library list; for a direct
            // open we set it up by refreshing the list and selecting the new doc.
            let mut app = App::new(ctx);
            // find the just-cached doc in the list and open it
            if let Some(idx) = app.docs.iter().position(|d| d.id == cached) {
                app.list_state.select(Some(idx));
                app.open_doc(idx);
            }
            if let Err(e) = mdreader_core::tui::app::run(app) {
                eprintln!("mdreader-tui: {e}");
                std::process::exit(1);
            }
            return;
        }
    }

    // Session restore: reopen the last doc if it still exists.
    let mut app = App::new(ctx.clone());
    if let Some(id) = ctx.session_store.lock().unwrap().last_doc_id() {
        if let Some(idx) = app.docs.iter().position(|d| d.id == id) {
            app.open_doc(idx);
        }
    }

    if let Err(e) = mdreader_core::tui::app::run(app) {
        eprintln!("mdreader-tui: {e}");
        std::process::exit(1);
    }
}
