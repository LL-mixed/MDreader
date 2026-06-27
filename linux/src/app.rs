// Per-document top-level window: header bar (zoom + theme) + Paned split with a sidebar
// (库/大纲: library list + outline) and the WebKitGTK content. Mirrors macOS ContentView +
// SidebarView + LibraryView + OutlineView. State is shared via Rc<RefCell<State>>.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gtk::gdk::{BUTTON_SECONDARY, ModifierType};
use gtk::pango::EllipsizeMode;
use gtk::prelude::*;
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, EventControllerKey, GestureClick,
    HeaderBar, Label, ListBox, ListBoxRow, Orientation, Paned, Popover, ScrolledWindow,
    SearchEntry, Stack, StackSwitcher,
};
use uuid::Uuid;

use crate::render::outline::OutlineItem;
use crate::render::webview;
use crate::store::content_hash::sha256_hex;
use crate::store::doc_info::DocInfo;
use crate::store::{cache::DocRepository, session_store::SessionStore, zoom_store::ZoomStore};
use crate::util::date_buckets::{self, DayBucket};
use crate::util::titles;

/// Process-wide stores shared across windows.
pub struct AppContext {
    pub repo: Arc<DocRepository>,
    pub zoom_store: Arc<Mutex<ZoomStore>>,
    pub session_store: Arc<Mutex<SessionStore>>,
}

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

struct State {
    app: Application,
    ctx: Arc<AppContext>,
    window: ApplicationWindow,
    webview: Option<webkit6::WebView>,
    zoom_label: Label,
    library_list: ListBox,
    outline_list: ListBox,
    library_empty: Label,
    markdown: String,
    dark: bool,
    zoom: f64,
    current_doc_id: Option<Uuid>,
    current_hash: String,
    base_dir: Option<PathBuf>,
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

    // --- header bar (zoom + theme) ---
    let zoom_label = Label::builder().label("100%").width_chars(5).build();
    let btn_out = Button::from_icon_name("zoom-out-symbolic");
    let btn_in = Button::from_icon_name("zoom-in-symbolic");
    let btn_reset = Button::with_label("1:1");
    let btn_theme = Button::from_icon_name("weather-clear-night-symbolic");
    let header = HeaderBar::new();
    header.pack_start(&btn_out);
    header.pack_start(&zoom_label);
    header.pack_start(&btn_in);
    header.pack_start(&btn_reset);
    header.pack_end(&btn_theme);

    // resolve initial content
    let (md, title, base, dark, zoom, doc_id, hash) = resolve_initial(ctx, &initial);

    let state = Rc::new(RefCell::new(State {
        app: app.clone(),
        ctx: ctx.clone(),
        window: window.clone(),
        webview: None,
        zoom_label: zoom_label.clone(),
        library_list: library_list.clone(),
        outline_list: outline_list.clone(),
        library_empty: library_empty.clone(),
        markdown: md.clone(),
        dark,
        zoom,
        current_doc_id: doc_id,
        current_hash: hash.clone(),
        base_dir: base.clone(),
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
                let ctx = ctx.clone();
                let app = app.clone();
                Box::new(move |name, text| {
                    open_window(
                        &ctx,
                        &app,
                        InitialDoc::File {
                            content: text.to_string(),
                            title: titles::from_path(name),
                            base: None,
                            source: None,
                        },
                    );
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

    // layout
    let content_scroll = scrolled(&wv);
    let sidebar_scroll = scrolled(&sidebar);
    sidebar_scroll.set_propagate_natural_width(true);
    let paned = Paned::new(Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar_scroll));
    paned.set_end_child(Some(&content_scroll));
    paned.set_position(290);
    paned.set_vexpand(true);

    let outer = GtkBox::new(Orientation::Vertical, 0);
    outer.append(&header);
    outer.append(&paned);
    window.set_title(Some(&title));
    window.set_child(Some(&outer));

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

    refresh_library(&state);
    update_zoom_label(&state);
    window.present();
}

fn resolve_initial(
    ctx: &Arc<AppContext>,
    initial: &InitialDoc,
) -> (String, String, Option<PathBuf>, bool, f64, Option<Uuid>, String) {
    match initial {
        InitialDoc::Sample => (
            webview::bundled_sample(),
            "MDreader".to_string(),
            None,
            false,
            1.0,
            None,
            String::new(),
        ),
        InitialDoc::File { content, title, base, source } => {
            let hash = sha256_hex(content);
            let id = ctx.repo.cache(title, content, source.as_deref());
            let _ = ctx.session_store.lock().unwrap().set_last_doc_id(Some(id));
            let z = ctx.zoom_store.lock().unwrap().zoom_for(&hash).unwrap_or(1.0);
            (content.clone(), title.clone(), base.clone(), false, z, Some(id), hash)
        }
        InitialDoc::Cached(id) => {
            let _ = ctx.repo.refresh_from_source(*id);
            let doc = ctx.repo.all().into_iter().find(|d| d.id == *id);
            if let Some(d) = doc {
                if let Some(text) = ctx.repo.load_content(*id) {
                    let base = d.source_uri.as_ref().and_then(parent_of);
                    let _ = ctx.session_store.lock().unwrap().set_last_doc_id(Some(*id));
                    let z = ctx.zoom_store.lock().unwrap().zoom_for(&d.content_hash).unwrap_or(1.0);
                    return (text, d.title, base, false, z, Some(*id), d.content_hash);
                }
            }
            (webview::bundled_sample(), "MDreader".to_string(), None, false, 1.0, None, String::new())
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
        if !modifier.contains(ModifierType::CONTROL_MASK) {
            return gtk::Inhibit(false);
        }
        match k.name().unwrap_or_default().as_str() {
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
    let (list, items) = {
        let s = state.borrow();
        (s.outline_list.clone(), s.outline.clone())
    };
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for item in &items {
        let row = ListBoxRow::new();
        row.set_widget_name(&item.index.to_string());
        let lbl = Label::new(Some(&item.text));
        lbl.set_halign(Align::Start);
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
    let (docs, query) = {
        let s = state.borrow();
        let docs = if s.query.trim().is_empty() {
            s.ctx.repo.all()
        } else {
            s.ctx.repo.search(&s.query)
        };
        (docs, s.query.clone())
    };
    let _ = query;
    let list = state.borrow().library_list.clone();
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let now = now_millis();
    state.borrow().library_empty.set_visible(docs.is_empty());

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
            list.append(&make_doc_row(state, doc));
        }
    }
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
    let is_fav = state
        .borrow()
        .ctx
        .repo
        .all()
        .iter()
        .find(|d| d.id == id)
        .map(|d| d.favorite)
        .unwrap_or(false);
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
        refresh_library(&s);
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
    let _ = ctx.session_store.lock().unwrap().set_last_doc_id(Some(id));
    let base = doc.source_uri.as_ref().and_then(parent_of);
    let zoom = ctx.zoom_store.lock().unwrap().zoom_for(&doc.content_hash).unwrap_or(1.0);

    let (dark, wv, window, title) = {
        let mut s = state.borrow_mut();
        s.markdown = text.clone();
        s.title = doc.title.clone();
        s.base_dir = base.clone();
        s.current_doc_id = Some(id);
        s.current_hash = doc.content_hash.clone();
        s.zoom = zoom;
        (s.dark, s.webview.clone(), s.window.clone(), doc.title.clone())
    };
    window.set_title(Some(&title));
    if let Some(wv) = wv {
        webview::set_zoom(&wv, zoom);
        webview::render(&wv, &text, dark, base.as_deref());
    }
    update_zoom_label(state);
}

fn zoom_by(state: &Rc<RefCell<State>>, up: bool) {
    let factor = 1.1f64;
    let z = state.borrow().zoom;
    let new = if up { (z * factor).min(3.0) } else { (z / factor).max(0.3) };
    apply_zoom(state, new);
}

fn zoom_reset(state: &Rc<RefCell<State>>) {
    apply_zoom(state, 1.0);
}

fn apply_zoom(state: &Rc<RefCell<State>>, zoom: f64) {
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
    update_zoom_label(state);
}

fn update_zoom_label(state: &Rc<RefCell<State>>) {
    let pct = (state.borrow().zoom * 100.0).round() as i32;
    state.borrow().zoom_label.set_label(&format!("{pct}%"));
}

fn toggle_theme(state: &Rc<RefCell<State>>) {
    let (dark, md, base, wv) = {
        let mut s = state.borrow_mut();
        s.dark = !s.dark;
        (s.dark, s.markdown.clone(), s.base_dir.clone(), s.webview.clone())
    };
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(dark);
    }
    if let Some(wv) = wv {
        webview::render(&wv, &md, dark, base.as_deref());
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
