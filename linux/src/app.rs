// Top-level document window: each opened document gets its own GtkApplicationWindow
// (GNOME-native multi-window model). LM3: window + webview; LM5 adds the sidebar/toolbar.

use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

use crate::render::webview;

/// Open a window rendering `content`. `base_dir` is the opened file's directory (for relative
/// image resolution); `title` is the window title.
pub fn open_doc_window(
    app: &Application,
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
    let wv = webview::new_webview(
        content,
        dark,
        base_dir.as_deref(),
        Box::new(move |name, text| {
            // A dropped .md opens a new window (no source path -> no image base dir).
            open_doc_window(&app_for_drop, &text, dark, None, name);
        }),
    );

    win.set_child(Some(&wv));
    win.present();
}
