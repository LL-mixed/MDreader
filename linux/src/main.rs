// MDreader — native Linux Markdown reader (GTK4 + WebKitGTK6).

mod app;
mod build_info;
mod config;
mod render;
mod store;
mod util;

use std::sync::{Arc, Mutex};

use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;
use gtk::{
    AboutDialog, Align, Application, Box as GtkBox, Entry, Label, License, Orientation,
    Window as GtkWindow,
};

use app::{AppContext, InitialDoc};
use store::cache::DocRepository;
use store::session_store::SessionStore;
use store::settings_store::SettingsStore;
use store::zoom_store::ZoomStore;

const APP_ID: &str = "com.mdreader.MDreader";

fn main() {
    // WebKit's GPU-compositing path calls abort() when it can't create a GBM EGL display (headless
    // / NVIDIA-EGL / X-forwarded boxes). A Markdown reader doesn't need GPU compositing, so disable
    // it up-front — the app runs everywhere without the user setting env vars.
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    gio::resources_register_include!("render.gresource").expect("failed to register gresource");
    render::webview::register_scheme();

    let ctx = Arc::new(AppContext {
        repo: Arc::new(DocRepository::open(&config::data_dir()).expect("failed to open cache")),
        zoom_store: Arc::new(Mutex::new(ZoomStore::open(&config::config_dir()))),
        session_store: Arc::new(Mutex::new(SessionStore::open(&config::config_dir()))),
        settings: Arc::new(Mutex::new(SettingsStore::open(&config::config_dir()))),
    });

    let app = Application::new(Some(APP_ID), gio::ApplicationFlags::HANDLES_OPEN);

    // App-wide GTK setup must run AFTER gtk is initialized — CssProvider::new /
    // IconTheme::default assert it. ::startup fires once, after init, before the first window.
    app.connect_startup(|_| {
        load_css();
        register_icon();
    });

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

    setup_app_menu(&app, &ctx);

    app.run();
}

fn open_file(app: &Application, ctx: &Arc<AppContext>, file: &gio::File) {
    let Some(path) = file.path() else {
        eprintln!("mdreader: skipping non-local file: {}", file.uri());
        return;
    };
    if !util::markdown_ext::is_markdown(&path.to_string_lossy()) {
        eprintln!("mdreader: not a markdown file: {}", path.display());
        return;
    }
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

/// Register the bundled app icon so it resolves without a system install: the gresource holds it
/// under an icon-theme layout (icons/<size>/apps/<id>.png); point the default theme at that path
/// and name it as the default window icon.
fn register_icon() {
    gtk::IconTheme::default().add_resource_path("/com/mdreader/MDreader/icons");
    gtk::Window::set_default_icon_name("com.mdreader.MDreader");
}

/// App menu (About / Preferences / Quit) with keyboard accelerators. Mirrors macOS's
/// `.commands{ appInfo }` + the `Settings` scene.
fn setup_app_menu(app: &Application, ctx: &Arc<AppContext>) {
    let about = SimpleAction::new("about", None);
    {
        let app = app.clone();
        about.connect_activate(move |_, _| show_about(&app));
    }
    app.add_action(&about);

    let preferences = SimpleAction::new("preferences", None);
    {
        let app = app.clone();
        let ctx = ctx.clone();
        preferences.connect_activate(move |_, _| show_preferences(&app, &ctx));
    }
    app.add_action(&preferences);

    let quit = SimpleAction::new("quit", None);
    {
        let app = app.clone();
        quit.connect_activate(move |_, _| app.quit());
    }
    app.add_action(&quit);

    // The menu itself is surfaced as a primary MenuButton in each window's header bar
    // (GNOME-native hamburger); the actions here provide the targets + keyboard shortcuts.
    app.set_accels_for_action("app.quit", &["<Primary>q"]);
    app.set_accels_for_action("app.preferences", &["<Primary>comma"]);
}

/// Native About dialog: name, version+git hash, description+build time, authors, MIT license.
fn show_about(app: &Application) {
    let dlg = AboutDialog::new();
    dlg.set_program_name(Some("MDreader"));
    dlg.set_version(Some(&build_info::version_line()));
    dlg.set_comments(Some(&format!(
        "{}\n构建时间：{}",
        build_info::DESCRIPTION, build_info::build_time()
    )));
    dlg.set_license_type(License::MitX11);
    let authors: Vec<&str> = build_info::AUTHORS
        .split(':')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if !authors.is_empty() {
        dlg.set_authors(&authors);
    }
    dlg.set_logo_icon_name(Some("com.mdreader.MDreader"));
    if let Some(win) = app.active_window() {
        dlg.set_transient_for(Some(&win));
        dlg.set_modal(true);
    }
    dlg.present();
}

/// Preferences window: external-editor command (bound live to SettingsStore).
fn show_preferences(app: &Application, ctx: &Arc<AppContext>) {
    let win = GtkWindow::new();
    win.set_title(Some("首选项"));
    win.set_default_size(420, 150);
    win.set_destroy_with_parent(true);

    let vbox = GtkBox::new(Orientation::Vertical, 6);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);

    let title = Label::new(Some("外部编辑器命令"));
    title.set_halign(Align::Start);
    let hint = Label::new(Some("打开原文件时调用的命令；留空则用 xdg-open（例如：code、typora、gedit）"));
    hint.set_halign(Align::Start);
    hint.add_css_class("dim-label");
    hint.set_wrap(true);

    let entry = Entry::new();
    entry.set_placeholder_text(Some("code / typora / gedit …"));
    let cur = ctx.settings.lock().unwrap().editor_command().to_string();
    entry.set_text(&cur);
    {
        let ctx = ctx.clone();
        entry.connect_changed(move |e| {
            ctx.settings
                .lock()
                .unwrap()
                .set_editor_command(e.text().to_string());
        });
    }

    vbox.append(&title);
    vbox.append(&entry);
    vbox.append(&hint);
    win.set_child(Some(&vbox));

    if let Some(parent) = app.active_window() {
        win.set_transient_for(Some(&parent));
    }
    win.present();
}
