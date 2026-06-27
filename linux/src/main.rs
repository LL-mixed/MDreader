// MDreader — native Linux Markdown reader (GTK4 + WebKitGTK6).

mod app;
mod config;
mod render;
mod store;

use std::sync::Arc;

use gio::prelude::*;
use gtk::Application;

use store::cache::DocRepository;

const APP_ID: &str = "com.mdreader.MDreader";

fn main() {
    gio::resources_register_include!("render.gresource").expect("failed to register gresource");
    render::webview::register_scheme();

    let repo = Arc::new(DocRepository::open(&config::data_dir()).expect("failed to open cache"));

    // HANDLES_OPEN so launching with file args routes to `open` (one window per file).
    let app = Application::new(Some(APP_ID), gio::ApplicationFlags::HANDLES_OPEN);

    {
        let repo = Arc::clone(&repo);
        app.connect_activate(move |app| {
            app::open_doc_window(app, &repo, &render::webview::bundled_sample(), false, None, "MDreader");
        });
    }
    {
        let repo = Arc::clone(&repo);
        app.connect_open(move |app, files, _hint| {
            for f in files {
                open_file(app, &repo, f);
            }
        });
    }

    app.run();
}

fn open_file(app: &Application, repo: &Arc<DocRepository>, file: &gio::File) {
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
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("MDreader")
        .to_string();
    let _id = repo.cache(&title, &content, path.to_str());
    let base = path.parent().map(|p| p.to_path_buf());
    app::open_doc_window(app, repo, &content, false, base, &title);
}
