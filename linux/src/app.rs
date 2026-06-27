// Top-level document window: each opened document gets its own GtkApplicationWindow
// (GNOME-native multi-window model). LM3-LM4: window + webview + cache-on-open.
// LM5 adds the sidebar/toolbar.

use std::path::PathBuf;
use std::sync::Arc;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

use crate::render::webview;
use crate::store::cache::DocRepository;

/// Open a window rendering `content`. The opened file is cached (dedup by SHA-256);
/// a .md dropped onto the page opens + caches a new window.
pub fn open_doc_window(
    app: &Application,
    repo: &Arc<DocRepository>,
    content: &str,
    dark: bool,
    base_dir: Option<PathBuf>,
    title: &str,
) {
    let win = ApplicationWindow::builder()
        .application(app)
        .title(title)
        .default_width(1000)
        .default_height(640)
        .build();

    let app_for_drop = app.clone();
    let repo_for_drop = Arc::clone(repo);
    let wv = webview::new_webview(
        content,
        dark,
        base_dir.as_deref(),
        Box::new(move |name, text| {
            repo_for_drop.cache(name, &text, None);
            open_doc_window(&app_for_drop, &repo_for_drop, &text, dark, None, name);
        }),
    );

    win.set_child(Some(&wv));
    win.present();
}
