// MDreader — native Linux Markdown reader (GTK4 + WebKitGTK6).

mod app;
mod config;
mod render;
mod store;
mod util;

use std::sync::{Arc, Mutex};

use gio::prelude::*;
use gtk::Application;

use app::{AppContext, InitialDoc};
use store::cache::DocRepository;
use store::session_store::SessionStore;
use store::zoom_store::ZoomStore;

const APP_ID: &str = "com.mdreader.MDreader";

fn main() {
    gio::resources_register_include!("render.gresource").expect("failed to register gresource");
    render::webview::register_scheme();
    load_css();

    let ctx = Arc::new(AppContext {
        repo: Arc::new(DocRepository::open(&config::data_dir()).expect("failed to open cache")),
        zoom_store: Arc::new(Mutex::new(ZoomStore::open(&config::config_dir()))),
        session_store: Arc::new(Mutex::new(SessionStore::open(&config::config_dir()))),
    });

    let app = Application::new(Some(APP_ID), gio::ApplicationFlags::HANDLES_OPEN);

    {
        let ctx = Arc::clone(&ctx);
        app.connect_activate(move |app| {
            // Session restore: reopen the last doc if it still exists.
            let initial = match ctx.session_store.lock().unwrap().last_doc_id() {
                Some(id) if ctx.repo.all().iter().any(|d| d.id == id) => InitialDoc::Cached(id),
                other => {
                    if other.is_some() {
                        ctx.session_store.lock().unwrap().set_last_doc_id(None);
                    }
                    InitialDoc::Sample
                }
            };
            app::open_window(&ctx, app, initial);
        });
    }
    {
        let ctx = Arc::clone(&ctx);
        app.connect_open(move |app, files, _hint| {
            for f in files {
                open_file(app, &ctx, f);
            }
        });
    }

    app.run();
}

fn open_file(app: &Application, ctx: &Arc<AppContext>, file: &gio::File) {
    let Some(path) = file.path() else {
        eprintln!("mdreader: skipping non-local file: {}", file.uri());
        return;
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("mdreader: failed to read {}: {e}", path.display());
            return;
        }
    };
    let title = util::titles::from_path(&path.to_string_lossy());
    app::open_window(
        ctx,
        app,
        InitialDoc::File {
            content,
            title,
            base: path.parent().map(|p| p.to_path_buf()),
            source: path.to_str().map(|s| s.to_string()),
        },
    );
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        ".dim-label { opacity: 0.55; } .favorite-star { color: @theme_selected_bg_color; }",
    );
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
