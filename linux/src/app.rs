// Per-document top-level window: header bar (zoom + theme) + Paned split with a sidebar
// (库/大纲: library list + outline) and the WebKitGTK content. Mirrors macOS ContentView +
// SidebarView + LibraryView + OutlineView. State is shared via Rc<RefCell<State>>.

use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gio::prelude::{FileExt, FileMonitorExt};
use gio::{File, FileMonitorFlags};
use gtk::gdk::{BUTTON_SECONDARY, ModifierType};
use gtk::pango::{AttrList, EllipsizeMode};
use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, EventControllerKey,
    EventControllerScroll, EventControllerScrollFlags, GestureClick, HeaderBar, Label, ListBox,
    ListBoxRow, Orientation, Paned, Popover, ScrolledWindow, SearchEntry, Stack, StackSwitcher,
};
use uuid::Uuid;

use mdreader_core::context::{AppContext, InitialDoc};
use mdreader_core::render::outline::OutlineItem;
use mdreader_core::render::webview;
use mdreader_core::store::content_hash::sha256_hex;
use mdreader_core::store::doc_info::DocInfo;
use mdreader_core::store::{
    cache::DocRepository, session_store::SessionStore, settings_store::SettingsStore,
    theme_store::ThemeStore, zoom_store::ZoomStore,
};
use mdreader_core::util::date_buckets::{self, DayBucket};
use mdreader_core::util::markdown_ext;
use mdreader_core::util::theme::{resolve_dark, ThemePref};
use mdreader_core::util::titles;
use mdreader_core::util::zoom as zoom_util;

struct State {
    app: Application,
    ctx: Arc<AppContext>,
    window: ApplicationWindow,
    title_label: Label,
    webview: Option<webkit6::WebView>,
    zoom_label: Label,
    btn_out: Button,
    btn_in: Button,
    btn_reset: Button,
    btn_edit: Button,
    btn_refresh: Button,
    library_list: ListBox,
    outline_list: ListBox,
    library_empty: Label,
    outline_empty: Label,
    markdown: String,
    dark: bool,
    zoom: f64,
    current_doc_id: Option<Uuid>,
    current_hash: String,
    base_dir: Option<PathBuf>,
    source: Option<String>,
    file_monitor: Option<gio::FileMonitor>,
    reload_debounce: Option<glib::source::SourceId>,
    title: String,
    outline: Vec<OutlineItem>,
    active: Option<usize>,
    query: String,
}

/// Open a window for `initial`.
pub fn open_window(ctx: &Arc<AppContext>, app: &Application, initial: InitialDoc) {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(1100)
        .default_height(700)
        .build();

    // --- sidebar ---
    let search = SearchEntry::new();
    search.set_placeholder_text(Some("搜索"));
    let library_list = ListBox::new();
    library_list.set_selection_mode(gtk::SelectionMode::Single);
    library_list.set_activate_on_single_click(true);
    library_list.add_css_class("navigation-sidebar");
    let library_empty = Label::new(Some("还没有缓存的文档\n打开或拖入 .md 即自动缓存"));
    library_empty.set_halign(Align::Center);
    library_empty.set_margin_top(24);
    library_empty.add_css_class("dim-label");
    let library_box = GtkBox::new(Orientation::Vertical, 0);
    library_box.append(&search);
    let lib_scroll = scrolled(&library_list);
    library_box.append(&lib_scroll);
    library_box.append(&library_empty);

    let outline_list = ListBox::new();
    outline_list.add_css_class("navigation-sidebar");
    outline_list.set_selection_mode(gtk::SelectionMode::Single);
    let outline_empty = Label::new(Some("无大纲"));
    outline_empty.add_css_class("dim-label");
    outline_empty.set_margin_top(24);
    let outline_box = GtkBox::new(Orientation::Vertical, 0);
    outline_box.append(&outline_empty);
    outline_box.append(&scrolled(&outline_list));

    let stack = Stack::new();
    stack.set_transition_duration(0);
    stack.add_titled(&library_box, Some("library"), "库");
    stack.add_titled(&outline_box, Some("outline"), "大纲");
    let switcher = StackSwitcher::builder().stack(&stack).build();
    let sidebar = GtkBox::new(Orientation::Vertical, 0);
    sidebar.append(&switcher);
    sidebar.append(&stack);
    sidebar.set_size_request(260, -1);

    // --- header bar (zoom + edit/export + theme) ---
    let zoom_label = Label::builder().label("100%").width_chars(5).build();
    let btn_out = Button::from_icon_name("zoom-out-symbolic");
    let btn_in = Button::from_icon_name("zoom-in-symbolic");
    let btn_reset = Button::with_label("1:1");
    let btn_edit = Button::from_icon_name("document-edit-symbolic");
    btn_edit.set_tooltip_text(Some("用外部编辑器打开原文件"));
    let btn_pdf = Button::from_icon_name("document-save-symbolic");
    btn_pdf.set_tooltip_text(Some("导出 PDF"));
    let btn_theme = Button::from_icon_name("weather-clear-night-symbolic");
    // Sidebar show/hide (mac parity: NavigationSplitView's built-in sidebar toggle).
    let btn_sidebar = Button::from_icon_name("sidebar-show-symbolic");
    btn_sidebar.set_tooltip_text(Some("显示/隐藏侧栏"));
    let btn_refresh = Button::from_icon_name("view-refresh-symbolic");
    btn_refresh.set_tooltip_text(Some("从原文件刷新（F5 / Ctrl+R）"));
    // Doc title shown in the header center (the HeaderBar doubles as the window titlebar).
    let title_label = Label::new(None);
    title_label.set_ellipsize(EllipsizeMode::End);
    title_label.set_max_width_chars(28);
    // Primary app menu (About / Preferences / Quit) — GNOME-native hamburger. The actions live on
    // the GApplication (see main.rs setup_app_menu) and resolve from any window of the app.
    let menu = gio::Menu::new();
    menu.append(Some("关于 MDreader"), Some("app.about"));
    menu.append(Some("首选项"), Some("app.preferences"));
    menu.append(Some("退出 MDreader"), Some("app.quit"));
    let menu_btn = gtk::MenuButton::new();
    menu_btn.set_menu_model(Some(&menu));
    menu_btn.set_icon_name("open-menu-symbolic");

    let header = HeaderBar::new();
    header.set_title_widget(Some(&title_label));
    header.pack_start(&btn_sidebar);
    header.pack_start(&btn_refresh);
    header.pack_start(&btn_out);
    header.pack_start(&zoom_label);
    header.pack_start(&btn_in);
    header.pack_start(&btn_reset);
    // pack_end fills right-to-left; first call is rightmost → [pdf][edit][theme][menu].
    header.pack_end(&menu_btn);
    header.pack_end(&btn_theme);
    header.pack_end(&btn_edit);
    header.pack_end(&btn_pdf);

    // resolve initial content
    let ResolvedDoc { content: md, title, base, dark, zoom, doc_id, hash, source } =
        resolve_initial(ctx, &initial);

    let state = Rc::new(RefCell::new(State {
        app: app.clone(),
        ctx: ctx.clone(),
        window: window.clone(),
        title_label: title_label.clone(),
        webview: None,
        zoom_label: zoom_label.clone(),
        btn_out: btn_out.clone(),
        btn_in: btn_in.clone(),
        btn_reset: btn_reset.clone(),
        btn_edit: btn_edit.clone(),
        btn_refresh: btn_refresh.clone(),
        library_list: library_list.clone(),
        outline_list: outline_list.clone(),
        library_empty: library_empty.clone(),
        outline_empty: outline_empty.clone(),
        markdown: md.clone(),
        dark,
        zoom,
        current_doc_id: doc_id,
        current_hash: hash.clone(),
        base_dir: base.clone(),
        source,
        file_monitor: None,
        reload_debounce: None,
        title: title.clone(),
        outline: Vec::new(),
        active: None,
        query: String::new(),
    }));

    // webview + bridge
    let wv = webview::new_webview(
        &md,
        dark,
        base.as_deref(),
        webview::Handlers {
            on_drop: {
                let s = state.clone();
                Box::new(move |name, text| {
                    if !markdown_ext::is_markdown(name) {
                        return;
                    }
                    apply_dropped_text(&s, text, name);
                })
            },
            on_outline: {
                let s = state.clone();
                Box::new(move |items| on_outline(&s, items))
            },
            on_active: {
                let s = state.clone();
                Box::new(move |i| on_active(&s, i))
            },
        },
    );
    webview::set_zoom(&wv, zoom);
    state.borrow_mut().webview = Some(wv.clone());
    install_scroll_zoom(&state, &wv);
    install_drop_target(&state, &wv);

    // layout
    let content_scroll = scrolled(&wv);
    let sidebar_scroll = scrolled(&sidebar);
    sidebar_scroll.set_propagate_natural_width(true);
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar_scroll));
    paned.set_end_child(Some(&content_scroll));
    paned.set_position(290);
    paned.set_vexpand(true);

    // HeaderBar is the window titlebar — one row of controls (GNOME-native). Putting it in the
    // content instead makes GTK draw a second set of window buttons. The Paned is the window body.
    window.set_titlebar(Some(&header));
    window.set_child(Some(&paned));
    window.set_title(Some(&title));
    state.borrow().title_label.set_label(&title);

    // --- wiring ---
    let s = state.clone();
    btn_out.connect_clicked(move |_| zoom_by(&s, false));
    let s = state.clone();
    btn_in.connect_clicked(move |_| zoom_by(&s, true));
    let s = state.clone();
    btn_reset.connect_clicked(move |_| zoom_reset(&s));
    let s = state.clone();
    btn_theme.connect_clicked(move |_| toggle_theme(&s));
    let s = state.clone();
    btn_edit.connect_clicked(move |_| edit_current(&s));
    let s = state.clone();
    btn_refresh.connect_clicked(move |_| reload_current(&s, true));
    let s = state.clone();
    btn_pdf.connect_clicked(move |_| export_current_pdf(&s));
    let sb = sidebar_scroll.clone();
    btn_sidebar.connect_clicked(move |_| {
        sb.set_visible(!sb.is_visible());
    });
    let s = state.clone();
    search.connect_search_changed(move |e| {
        s.borrow_mut().query = e.text().to_string();
        refresh_library(&s);
    });
    let s = state.clone();
    library_list.connect_row_activated(move |_l, row| {
        if let Some(id) = parse_row_id(row) {
            open_cached(&s, id);
        }
    });
    let s = state.clone();
    outline_list.connect_row_activated(move |_l, row| {
        let name = row.widget_name();
        if let Ok(i) = name.as_str().parse::<i32>() {
            if let Some(wv) = s.borrow().webview.clone() {
                webview::scroll_to_heading(&wv, i);
            }
        }
    });

    install_zoom_shortcuts(&state, &window);

    // Cancel any pending auto-reload and drop the file monitor on close, so a debounce timer
    // can never fire into a freed State.
    {
        let s = state.clone();
        window.connect_close_request(move |_| {
            if let Some(id) = s.borrow_mut().reload_debounce.take() {
                id.remove();
            }
            let _ = s.borrow_mut().file_monitor.take();
            gtk::Inhibit(false)
        });
    }

    refresh_library(&state);
    update_zoom_label(&state);
    update_edit_sensitivity(&state);
    update_refresh_sensitivity(&state);
    arm_file_monitor(&state);
    // React to OS color-scheme changes (System mode) and to our own apply_global_theme_pref
    // (Light/Dark selection): docs without a per-doc override re-follow the resolved default.
    // Weak ref so closing this window disconnects it instead of leaking the State.
    {
        if let Some(settings) = gtk::Settings::default() {
            let weak = Rc::downgrade(&state);
            settings.connect_notify_local(
                Some("gtk-application-prefer-dark-theme"),
                move |_, _| {
                    // set_prefer_dark_theme emits notify synchronously, so calling reapply inline
                    // recurses on the property-set stack and stalls the main loop (window paints
                    // then ANRs on input). Defer to idle: the re-entrant set either no-ops (value
                    // unchanged) or schedules another idle that terminates once dark stabilizes.
                    if let Some(s) = weak.upgrade() {
                        let _ = glib::source::idle_add_local_once(move || {
                            reapply_theme_if_unpinned(&s);
                        });
                    }
                },
            );
        }
    }
    window.present();
    // Sync chrome to the active doc's dark flag on idle (after present, so the notify chain
    // can't block the window from mapping). This is the per-doc resolved value, NOT the global
    // pref — a doc pinned Light under a Dark default must still paint chrome Light to match its
    // body, or the sidebar/outline ends up Dark while the body is Light.
    {
        let s = state.clone();
        let _ = glib::source::idle_add_local_once(move || {
            let dark = s.borrow().dark;
            apply_dark(&s, dark);
        });
    }
}

/// Resolved content for a freshly opened window. A named struct (not an 8-tuple) so callers
/// can't silently transpose same-typed fields.
struct ResolvedDoc {
    content: String,
    title: String,
    base: Option<PathBuf>,
    dark: bool,
    zoom: f64,
    doc_id: Option<Uuid>,
    hash: String,
    source: Option<String>,
}

fn resolve_initial(ctx: &Arc<AppContext>, initial: &InitialDoc) -> ResolvedDoc {
    let sample = || ResolvedDoc {
        content: webview::bundled_sample(),
        title: "MDreader".to_string(),
        base: None,
        dark: compute_dark_for(ctx, ""),
        zoom: 1.0,
        doc_id: None,
        hash: String::new(),
        source: None,
    };
    match initial {
        InitialDoc::Sample => sample(),
        InitialDoc::File { content, title, base, source } => {
            let hash = sha256_hex(content);
            let id = ctx.repo.cache(title, content, source.as_deref());
            ctx.session_store.lock().unwrap().set_last_doc_id(Some(id));
            let zoom = ctx.zoom_store.lock().unwrap().zoom_for(&hash).unwrap_or(1.0);
            ResolvedDoc {
                content: content.clone(),
                title: title.clone(),
                base: base.clone(),
                dark: compute_dark_for(ctx, &hash),
                zoom,
                doc_id: Some(id),
                hash,
                source: source.clone(),
            }
        }
        InitialDoc::Cached(id) => {
            let _ = ctx.repo.refresh_from_source(*id);
            let doc = ctx.repo.all().into_iter().find(|d| d.id == *id);
            if let Some(d) = doc {
                if let Some(text) = ctx.repo.load_content(*id) {
                    let base = d.source_uri.as_ref().and_then(parent_of);
                    ctx.session_store.lock().unwrap().set_last_doc_id(Some(*id));
                    let zoom = ctx.zoom_store.lock().unwrap().zoom_for(&d.content_hash).unwrap_or(1.0);
                    return ResolvedDoc {
                        content: text,
                        title: d.title,
                        base,
                        dark: compute_dark_for(ctx, &d.content_hash),
                        zoom,
                        doc_id: Some(*id),
                        hash: d.content_hash,
                        source: d.source_uri,
                    };
                }
            }
            sample()
        }
    }
}

fn parent_of(p: &String) -> Option<PathBuf> {
    std::path::Path::new(p).parent().map(|x| x.to_path_buf())
}

fn scrolled(child: &impl IsA<gtk::Widget>) -> ScrolledWindow {
    let sw = ScrolledWindow::new();
    sw.set_vexpand(true);
    sw.set_child(Some(child));
    sw
}

fn parse_row_id(row: &ListBoxRow) -> Option<Uuid> {
    Uuid::parse_str(row.widget_name().as_str()).ok()
}

fn install_zoom_shortcuts(state: &Rc<RefCell<State>>, window: &ApplicationWindow) {
    let key = EventControllerKey::new();
    let s = state.clone();
    key.connect_key_pressed(move |_, k, _code, modifier| {
        let name = k.name().unwrap_or_default();
        let ctrl = modifier.contains(ModifierType::CONTROL_MASK);
        // F5 / Ctrl+R: reload the current doc from its source file.
        if name == "F5" || (ctrl && (name == "r" || name == "R")) {
            reload_current(&s, true);
            return gtk::Inhibit(true);
        }
        if !ctrl {
            return gtk::Inhibit(false);
        }
        match name.as_str() {
            "plus" | "equal" | "KP_Add" => {
                zoom_by(&s, true);
                gtk::Inhibit(true)
            }
            "minus" | "underscore" | "KP_Subtract" => {
                zoom_by(&s, false);
                gtk::Inhibit(true)
            }
            "0" | "KP_0" => {
                zoom_reset(&s);
                gtk::Inhibit(true)
            }
            _ => gtk::Inhibit(false),
        }
    });
    window.add_controller(key);
}

fn on_outline(state: &Rc<RefCell<State>>, items: Vec<OutlineItem>) {
    state.borrow_mut().outline = items;
    refresh_outline(state);
}

fn on_active(state: &Rc<RefCell<State>>, index: i32) {
    state.borrow_mut().active = if index < 0 { None } else { Some(index as usize) };
    select_active_outline(state);
}

fn refresh_outline(state: &Rc<RefCell<State>>) {
    let (list, items, zoom, empty) = {
        let s = state.borrow();
        (s.outline_list.clone(), s.outline.clone(), s.zoom, s.outline_empty.clone())
    };
    // Placeholder is shown only when there are no headings (mac parity: OutlineView empty state
    // is mutually exclusive with content).
    empty.set_visible(items.is_empty());
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for item in &items {
        let row = ListBoxRow::new();
        row.set_widget_name(&item.index.to_string());
        let lbl = Label::new(Some(&item.text));
        lbl.set_halign(Align::Start);
        // Outline text scales with zoom (mac parity: OutlineRow font size 13 * zoom).
        let attrs = AttrList::new();
        attrs.insert(gtk::pango::AttrFloat::new_scale(zoom));
        lbl.set_attributes(Some(&attrs));
        lbl.set_margin_start((item.level.saturating_sub(1) as i32) * 12 + 6);
        lbl.set_ellipsize(EllipsizeMode::End);
        row.set_child(Some(&lbl));
        list.append(&row);
    }
    select_active_outline(state);
}

fn select_active_outline(state: &Rc<RefCell<State>>) {
    let active = state.borrow().active;
    let list = state.borrow().outline_list.clone();
    let mut target: Option<ListBoxRow> = None;
    let mut i = 0usize;
    let mut child = list.first_child();
    while let Some(w) = child {
        if let Some(row) = w.downcast_ref::<ListBoxRow>() {
            if active == Some(i) {
                target = Some(row.clone());
            }
            i += 1;
        }
        child = w.next_sibling();
    }
    list.select_row(target.as_ref());
}

fn refresh_library(state: &Rc<RefCell<State>>) {
    let (docs, query, current) = {
        let s = state.borrow();
        let docs = if s.query.trim().is_empty() {
            s.ctx.repo.all()
        } else {
            s.ctx.repo.search(&s.query)
        };
        (docs, s.query.clone(), s.current_doc_id)
    };
    let _ = query;
    let list = state.borrow().library_list.clone();
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let now = now_millis();
    state.borrow().library_empty.set_visible(docs.is_empty());

    let mut selected: Option<ListBoxRow> = None;
    for bucket in DayBucket::all() {
        let group: Vec<&DocInfo> = docs
            .iter()
            .filter(|d| date_buckets::bucket(d.opened_at, now) == bucket)
            .collect();
        if group.is_empty() {
            continue;
        }
        let header = ListBoxRow::new();
        header.set_selectable(false);
        header.set_activatable(false);
        let hlbl = Label::new(Some(bucket.title()));
        hlbl.set_halign(Align::Start);
        hlbl.set_margin_start(6);
        hlbl.add_css_class("dim-label");
        header.set_child(Some(&hlbl));
        list.append(&header);
        for doc in group {
            let row = make_doc_row(state, doc);
            if Some(doc.id) == current {
                selected = Some(row.clone());
            }
            list.append(&row);
        }
    }
    list.select_row(selected.as_ref());
}

fn make_doc_row(state: &Rc<RefCell<State>>, doc: &DocInfo) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_widget_name(&doc.id.to_string());

    let title_line = GtkBox::new(Orientation::Horizontal, 4);
    title_line.set_margin_start(8);
    title_line.set_margin_end(8);
    title_line.set_margin_top(5);
    title_line.set_margin_bottom(5);
    if doc.favorite {
        let star = Label::new(Some("★"));
        title_line.append(&star);
    }
    let title_lbl = Label::new(Some(&doc.title));
    title_lbl.set_halign(Align::Start);
    title_lbl.set_hexpand(true);
    title_lbl.set_ellipsize(EllipsizeMode::End);
    let time_lbl = Label::new(Some(&date_buckets::format(doc.opened_at)));
    time_lbl.add_css_class("dim-label");
    title_line.append(&title_lbl);
    title_line.append(&time_lbl);
    row.set_child(Some(&title_line));

    // right-click context menu
    let click = GestureClick::new();
    click.set_button(BUTTON_SECONDARY);
    let s = state.clone();
    let id = doc.id;
    let row_ref = row.clone();
    click.connect_pressed(move |_g, _n, x, y| {
        let popover = build_context_menu(&s, id);
        popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.set_parent(&row_ref);
        popover.popup();
    });
    row.add_controller(click);
    row
}

fn build_context_menu(state: &Rc<RefCell<State>>, id: Uuid) -> Popover {
    let popover = Popover::new();
    let vbox = GtkBox::new(Orientation::Vertical, 0);
    let mk = |label: &str| {
        let b = Button::with_label(label);
        b.add_css_class("flat");
        b
    };
    let b_open = mk("在新窗口打开");
    let b_refresh = mk("从原文件刷新");
    let doc = state.borrow().ctx.repo.all().into_iter().find(|d| d.id == id);
    let is_fav = doc.as_ref().map(|d| d.favorite).unwrap_or(false);
    // Refresh is only meaningful when the original file still exists (mac parity: canRefresh).
    let can_refresh = doc
        .as_ref()
        .and_then(|d| d.source_uri.as_ref())
        .map(|p| std::path::Path::new(p).exists())
        .unwrap_or(false);
    b_refresh.set_sensitive(can_refresh);
    let b_fav = mk(if is_fav { "取消收藏" } else { "收藏" });
    let b_del = mk("删除");
    b_del.add_css_class("destructive-action");

    {
        let ctx = state.borrow().ctx.clone();
        let app = state.borrow().app.clone();
        b_open.connect_clicked(move |_| {
            open_window(&ctx, &app, InitialDoc::Cached(id));
        });
    }
    let s = state.clone();
    b_refresh.connect_clicked(move |_| {
        s.borrow().ctx.repo.refresh_from_source(id);
        // If refreshing the doc currently in view, re-render it (mac parity: refreshDoc → openCached).
        if s.borrow().current_doc_id == Some(id) {
            open_cached(&s, id);
        } else {
            refresh_library(&s);
        }
    });
    let s = state.clone();
    b_fav.connect_clicked(move |_| {
        let fav = s
            .borrow()
            .ctx
            .repo
            .all()
            .iter()
            .find(|d| d.id == id)
            .map(|d| d.favorite)
            .unwrap_or(false);
        s.borrow().ctx.repo.set_favorite(id, !fav);
        refresh_library(&s);
    });
    let s = state.clone();
    b_del.connect_clicked(move |_| {
        s.borrow().ctx.repo.delete(id);
        refresh_library(&s);
    });

    for b in [&b_open, &b_refresh, &b_fav, &b_del] {
        vbox.append(b);
    }
    popover.set_child(Some(&vbox));
    popover
}

fn open_cached(state: &Rc<RefCell<State>>, id: Uuid) {
    let ctx = state.borrow().ctx.clone();
    let _ = ctx.repo.refresh_from_source(id);
    let text = match ctx.repo.load_content(id) {
        Some(t) => t,
        None => return,
    };
    let doc = match ctx.repo.all().into_iter().find(|d| d.id == id) {
        Some(d) => d,
        None => return,
    };
    ctx.session_store.lock().unwrap().set_last_doc_id(Some(id));
    let base = doc.source_uri.as_ref().and_then(parent_of);
    let zoom = ctx.zoom_store.lock().unwrap().zoom_for(&doc.content_hash).unwrap_or(1.0);

    let (wv, window, title) = {
        let mut s = state.borrow_mut();
        s.markdown = text.clone();
        s.title = doc.title.clone();
        s.base_dir = base.clone();
        s.current_doc_id = Some(id);
        s.current_hash = doc.content_hash.clone();
        s.source = doc.source_uri.clone();
        s.zoom = zoom;
        (s.webview.clone(), s.window.clone(), doc.title.clone())
    };
    // Theme: a stored per-doc override wins, else follow the global default (system/light/dark).
    let dark = compute_dark(state);
    apply_dark(state, dark);
    window.set_title(Some(&title));
    state.borrow().title_label.set_label(&title);
    if let Some(wv) = wv {
        webview::set_zoom(&wv, zoom);
        webview::render(&wv, &text, dark, base.as_deref());
    }
    // Defer the rebuild: open_cached can run inside a row_activated emission
    // (activate_on_single_click); mutating the emitting ListBox mid-emission is fragile.
    {
        let s = state.clone();
        let _ = glib::idle_add_local_once(move || refresh_library(&s));
    }
    update_zoom_label(state);
    update_edit_sensitivity(state);
    update_refresh_sensitivity(state);
    arm_file_monitor(state);
}

/// Replace the current window's content with dropped text (mac parity: openText/applyText — a
/// drop replaces the current doc rather than spawning a new window; "open in new window" stays
/// available from the library's context menu).
fn apply_dropped_text(state: &Rc<RefCell<State>>, content: &str, name: &str) {
    let title = titles::from_path(name);
    let hash = sha256_hex(content);
    let ctx = state.borrow().ctx.clone();
    let id = ctx.repo.cache(&title, content, None);
    ctx.session_store.lock().unwrap().set_last_doc_id(Some(id));
    let zoom = ctx.zoom_store.lock().unwrap().zoom_for(&hash).unwrap_or(1.0);

    let (wv, window) = {
        let mut s = state.borrow_mut();
        s.markdown = content.to_string();
        s.title = title.clone();
        s.base_dir = None;
        s.current_doc_id = Some(id);
        s.current_hash = hash.clone();
        s.source = None;
        s.zoom = zoom;
        s.outline.clear();
        s.active = None;
        (s.webview.clone(), s.window.clone())
    };
    let dark = compute_dark(state);
    apply_dark(state, dark);
    window.set_title(Some(&title));
    state.borrow().title_label.set_label(&title);
    if let Some(wv) = wv {
        webview::set_zoom(&wv, zoom);
        webview::render(&wv, content, dark, None);
    }
    refresh_outline(state);
    refresh_library(state);
    update_zoom_label(state);
    update_edit_sensitivity(state);
    update_refresh_sensitivity(state);
    arm_file_monitor(state);
}

fn zoom_by(state: &Rc<RefCell<State>>, up: bool) {
    let new = zoom_util::step(state.borrow().zoom, up);
    apply_zoom(state, new);
}

fn zoom_reset(state: &Rc<RefCell<State>>) {
    apply_zoom(state, 1.0);
}

fn apply_zoom(state: &Rc<RefCell<State>>, zoom: f64) {
    let zoom = zoom_util::clamp(zoom);
    let (wv, hash, ctx) = {
        let mut s = state.borrow_mut();
        s.zoom = zoom;
        if s.current_hash.is_empty() {
            s.current_hash = sha256_hex(&s.markdown);
        }
        (s.webview.clone(), s.current_hash.clone(), s.ctx.clone())
    };
    ctx.zoom_store.lock().unwrap().set_zoom(zoom, &hash);
    if let Some(wv) = wv {
        webview::set_zoom(&wv, zoom);
    }
    refresh_outline(state); // outline font scales with zoom
    update_zoom_label(state);
}

fn update_zoom_label(state: &Rc<RefCell<State>>) {
    let s = state.borrow();
    let z = s.zoom;
    let pct = (z * 100.0).round() as i32;
    s.zoom_label.set_label(&format!("{pct}%"));
    s.btn_out.set_sensitive(z > zoom_util::MIN_ZOOM);
    s.btn_in.set_sensitive(z < zoom_util::MAX_ZOOM);
    s.btn_reset.set_sensitive((z - 1.0).abs() > 1e-9);
}

fn toggle_theme(state: &Rc<RefCell<State>>) {
    let (dark, hash, ctx, md, base, wv) = {
        let mut s = state.borrow_mut();
        s.dark = !s.dark;
        (
            s.dark,
            s.current_hash.clone(),
            s.ctx.clone(),
            s.markdown.clone(),
            s.base_dir.clone(),
            s.webview.clone(),
        )
    };
    // Persist the override so this doc keeps its theme on reopen. The sample and content-less
    // drops (empty hash) aren't persisted — they follow the global default instead.
    if !hash.is_empty() {
        ctx.theme_store.lock().unwrap().set_dark(dark, &hash);
    }
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(dark);
    }
    if let Some(wv) = wv {
        webview::render(&wv, &md, dark, base.as_deref());
    }
}

/// Current OS color scheme as seen through GTK's `gtk-application-prefer-dark-theme`. On GNOME this
/// tracks the system dark mode; on desktops that don't set it, it defaults to light.
fn system_dark() -> bool {
    gtk::Settings::default()
        .map(|s| s.is_gtk_application_prefer_dark_theme())
        .unwrap_or(false)
}

/// Effective dark for a doc not yet attached to a window (the resolve_initial path): a stored
/// per-doc override by hash wins, else the global default resolves against the system scheme.
fn compute_dark_for(ctx: &Arc<AppContext>, hash: &str) -> bool {
    let per_doc = ctx.theme_store.lock().unwrap().dark_for(hash);
    let pref = ctx.settings.lock().unwrap().theme_pref();
    resolve_dark(per_doc, pref, system_dark())
}

/// Effective dark for the current doc of an existing window.
fn compute_dark(state: &Rc<RefCell<State>>) -> bool {
    let (per_doc, pref) = {
        let s = state.borrow();
        let hash = s.current_hash.clone();
        let per_doc = s.ctx.theme_store.lock().unwrap().dark_for(&hash);
        let pref = s.ctx.settings.lock().unwrap().theme_pref();
        (per_doc, pref)
    };
    resolve_dark(per_doc, pref, system_dark())
}

/// Push the global theme preference into GTK's chrome via `gtk-application-prefer-dark-theme`.
/// Light/Dark force the value; System snaps it to the current OS scheme so the chrome matches what
/// an unpinned doc will compute. Each window's `notify` handler then re-renders any doc without a
/// per-doc override.
pub fn apply_global_theme_pref(pref: ThemePref) {
    let Some(settings) = gtk::Settings::default() else { return; };
    match pref {
        ThemePref::Light => settings.set_gtk_application_prefer_dark_theme(false),
        ThemePref::Dark => settings.set_gtk_application_prefer_dark_theme(true),
        ThemePref::System => settings.set_gtk_application_prefer_dark_theme(system_dark()),
    }
}

/// Set the window's dark flag AND sync GTK's chrome to it. The guard reads the *live*
/// `gtk-application-prefer-dark-theme` (not s.dark) so an external write that moved the chrome
/// (prefs dropdown / OS scheme) is pulled back to the doc's dark flag, while no-op sets don't
/// spam notify. This is what keeps the sidebar/outline (which follows chrome) aligned with the
/// rendered body (which follows s.dark), including pinned docs under a different global default.
fn apply_dark(state: &Rc<RefCell<State>>, dark: bool) {
    state.borrow_mut().dark = dark;
    if let Some(settings) = gtk::Settings::default() {
        if settings.is_gtk_application_prefer_dark_theme() != dark {
            settings.set_gtk_application_prefer_dark_theme(dark);
        }
    }
}

/// Re-resolve this window's theme after a global-pref or OS-scheme change. The chrome is always
/// (re)glued to the resolved dark — a pinned doc keeps its body theme, but an external prefer-dark
/// write (prefs dropdown / OS) may have moved its chrome and must be pulled back. Unpinned docs
/// additionally re-render when the resolved value actually changes.
fn reapply_theme_if_unpinned(state: &Rc<RefCell<State>>) {
    let pinned = {
        let s = state.borrow();
        let hash = s.current_hash.clone();
        let pinned = s.ctx.theme_store.lock().unwrap().dark_for(&hash);
        pinned
    };
    let new_dark = compute_dark(state);
    let prev_dark = state.borrow().dark;
    apply_dark(state, new_dark);
    if pinned.is_some() || prev_dark == new_dark {
        return;
    }
    let (md, base, wv) = {
        let s = state.borrow();
        (s.markdown.clone(), s.base_dir.clone(), s.webview.clone())
    };
    if let Some(wv) = wv {
        webview::render(&wv, &md, new_dark, base.as_deref());
    }
}

/// Enable the "edit" button only when the current doc has a backing source file.
fn update_edit_sensitivity(state: &Rc<RefCell<State>>) {
    let (has_source, btn) = {
        let s = state.borrow();
        (s.source.is_some(), s.btn_edit.clone())
    };
    btn.set_sensitive(has_source);
}

/// Enable the "refresh" button only when the current doc has a backing source file.
fn update_refresh_sensitivity(state: &Rc<RefCell<State>>) {
    let (has_source, btn) = {
        let s = state.borrow();
        (s.source.is_some(), s.btn_refresh.clone())
    };
    btn.set_sensitive(has_source);
}

/// Reload the current doc from its original file (header button / F5 / Ctrl+R). Reuses open_cached,
/// which runs refresh_from_source then re-renders. When `force` is true (a user-initiated reload)
/// the doc is re-rendered unconditionally — the user may have changed external resources the hash
/// doesn't cover (images, etc.). When false (auto-reload from the file monitor) the render is
/// skipped on identical content so noisy monitor events stay cheap.
fn reload_current(state: &Rc<RefCell<State>>, force: bool) {
    let (id, ctx) = {
        let s = state.borrow();
        (s.current_doc_id, s.ctx.clone())
    };
    let Some(id) = id else { return; };
    if force || ctx.repo.refresh_from_source(id) {
        open_cached(state, id);
    }
}

/// (Re)arm a file-changed monitor on the current doc's source. Re-invoked whenever the current
/// doc changes (open/switch/drop) or after an auto-reload, so the watch always tracks the live
/// inode — editors that save via atomic rename replace the inode, and recreating the monitor
/// re-attaches to the new one. The signal closure holds a Weak ref to avoid an Rc cycle (State
/// owns the monitor, which would otherwise own State).
fn arm_file_monitor(state: &Rc<RefCell<State>>) {
    // Drop the previous monitor and cancel any pending debounce first.
    let _ = state.borrow_mut().file_monitor.take();
    if let Some(id) = state.borrow_mut().reload_debounce.take() {
        id.remove();
    }

    let Some(path) = state.borrow().source.clone() else {
        return;
    };
    // Only watch when the file currently exists; refresh_from_source tolerates later deletion.
    if !std::path::Path::new(&path).exists() {
        return;
    }
    let file = File::for_path(&path);
    let Ok(monitor) = file.monitor_file(FileMonitorFlags::NONE, None::<&gio::Cancellable>) else {
        // Some filesystems (NFS, certain FUSE) don't support monitoring; fall back to manual only.
        return;
    };
    let weak = Rc::downgrade(state);
    monitor.connect_changed(move |_m, _file, _other, _event| {
        // React to every event and let refresh_from_source's hash compare decide whether a
        // reload is warranted — this sidesteps platform/editor differences in which
        // FileMonitorEvent variant they emit on save.
        if let Some(s) = weak.upgrade() {
            schedule_reload(&s);
        }
    });
    state.borrow_mut().file_monitor = Some(monitor);
}

/// Debounce monitor chatter: editors often emit several Changed events per save. Each event
/// resets a 300ms timer; the reload only fires after the file has settled.
fn schedule_reload(state: &Rc<RefCell<State>>) {
    if let Some(id) = state.borrow_mut().reload_debounce.take() {
        id.remove();
    }
    let s = state.clone();
    let id = glib::source::timeout_add_local_once(Duration::from_millis(300), move || {
        s.borrow_mut().reload_debounce = None;
        reload_current(&s, false);
    });
    state.borrow_mut().reload_debounce = Some(id);
}

/// Open the current doc's source file in the configured external editor; fall back to
/// `xdg-open` when no command is set (a UX improvement over macOS, which disables the button).
/// `editor_command` is a command (e.g. `code`, `typora`, `gedit`, or `code -n` with flags).
fn edit_current(state: &Rc<RefCell<State>>) {
    let (source, cmd) = {
        let s = state.borrow();
        let cmd = s.ctx.settings.lock().unwrap().editor_command().to_string();
        (s.source.clone(), cmd)
    };
    let Some(path) = source else { return; };
    let launcher = if cmd.trim().is_empty() { "xdg-open".to_string() } else { cmd };
    // Spawn with the path as a real argv element — NO shell — so path bytes ($, backtick,
    // newline, spaces) cannot be interpreted or break the launch. The command may carry flags
    // (e.g. `code -n`), so split it on whitespace into program + leading args, then the path.
    let mut parts = launcher.split_whitespace();
    let Some(program) = parts.next() else { return; };
    let mut command = Command::new(program);
    for arg in parts {
        command.arg(arg);
    }
    command.arg(&path);
    let _ = command.spawn();
}

/// Export the current page to PDF via the native print dialog (pre-set to "Print to File").
fn export_current_pdf(state: &Rc<RefCell<State>>) {
    let (wv, window) = {
        let s = state.borrow();
        (s.webview.clone(), s.window.clone())
    };
    if let Some(wv) = wv {
        webview::export_pdf(&wv, Some(window.upcast_ref::<gtk::Window>()));
    }
}

/// Ctrl+scroll zooms the page (mac parity: ⌘-scroll). Plain scroll pans the document.
fn install_scroll_zoom(state: &Rc<RefCell<State>>, wv: &webkit6::WebView) {
    let scroll = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
    let s = state.clone();
    scroll.connect_scroll(move |c, _dx, dy| {
        if c.current_event_state().contains(ModifierType::CONTROL_MASK) {
            zoom_by(&s, dy < 0.0);
            gtk::Inhibit(true)
        } else {
            gtk::Inhibit(false)
        }
    });
    wv.add_controller(scroll);
}

/// Native file drag-drop: receive a dropped `.md` as a `GFile`. This is the robust path on
/// WebKitGTK, where the in-page HTML5 drop event does not reliably expose dropped files to JS
/// (the macOS port relies on that page-level drop because WKWebView delivers it; Linux can't).
fn install_drop_target(state: &Rc<RefCell<State>>, wv: &webkit6::WebView) {
    let target = gtk::DropTarget::new(gio::File::static_type(), gtk::gdk::DragAction::COPY);
    let s = state.clone();
    target.connect_drop(move |_t, value, _x, _y| {
        let Ok(file) = value.get::<gio::File>() else { return false; };
        let Some(path) = file.path() else { return false; };
        let path_str = path.to_string_lossy().to_string();
        if !markdown_ext::is_markdown(&path_str) {
            return false;
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                apply_dropped_text(&s, &text, &path_str);
                true
            }
            Err(_) => false,
        }
    });
    wv.add_controller(target);
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
