//! TUI application state + ratatui rendering + event loop.

use std::io;
use std::sync::Arc;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Terminal;

use crate::context::AppContext;
use crate::render::mermaid_fence;
use crate::store::doc_info::DocInfo;
use crate::util::date_buckets;
use crate::util::theme::resolve_dark;

use super::renderer::{render, HeadingRef};

/// Which pane the left sidebar is showing.
#[derive(PartialEq, Eq, Clone, Copy)]
enum SideTab {
    Library,
    Outline,
}

/// The document currently displayed (title + rendered lines + outline).
struct CurrentDoc {
    title: String,
    lines: Vec<Line<'static>>,
    headings: Vec<HeadingRef>,
}

pub struct App {
    ctx: Arc<AppContext>,
    pub docs: Vec<DocInfo>,
    selected_doc: Option<usize>,   // index into docs
    side_tab: SideTab,
    pub list_state: ListState,
    current: Option<CurrentDoc>,
    scroll: usize,
}

impl App {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        let docs = ctx.repo.all();
        let mut app = App {
            ctx,
            docs,
            selected_doc: None,
            side_tab: SideTab::Library,
            list_state: ListState::default(),
            current: None,
            scroll: 0,
        };
        app.list_state.select(Some(0));
        app
    }

    /// Open a cached doc by index into `docs`.
    pub fn open_doc(&mut self, idx: usize) {
        let doc = match self.docs.get(idx) {
            Some(d) => d.clone(),
            None => return,
        };
        // refresh from source if possible
        let _ = self.ctx.repo.refresh_from_source(doc.id);
        let content = self.ctx.repo.load_content(doc.id).unwrap_or_default();
        let title = doc.title.clone();
        let normalized = mermaid_fence::normalize(&content);
        let rendered = render(&normalized);
        self.current = Some(CurrentDoc {
            title,
            lines: rendered.lines,
            headings: rendered.headings,
        });
        self.selected_doc = Some(idx);
        self.scroll = 0;
        self.side_tab = SideTab::Outline;
        // persist session
        self.ctx.session_store.lock().unwrap().set_last_doc_id(Some(doc.id));
    }

    fn move_selection(&mut self, delta: i32) {
        let len = match self.side_tab {
            SideTab::Library => self.docs.len(),
            SideTab::Outline => self.current.as_ref().map(|d| d.headings.len()).unwrap_or(0),
        };
        if len == 0 {
            return;
        }
        let cur = self.list_state.selected().unwrap_or(0);
        let next = (cur as i32 + delta).rem_euclid(len as i32) as usize;
        self.list_state.select(Some(next));
    }

    fn scroll_content(&mut self, delta: i32) {
        if self.current.is_none() {
            return;
        }
        let total = self.current.as_ref().unwrap().lines.len() as i32;
        let next = self.scroll as i32 + delta;
        self.scroll = next.max(0).min(total.saturating_sub(1)) as usize;
    }

    fn jump_to_heading(&mut self) {
        let idx = match self.list_state.selected() {
            Some(i) => i,
            None => return,
        };
        if self.side_tab == SideTab::Outline {
            if let Some(doc) = &self.current {
                if let Some(h) = doc.headings.get(idx) {
                    self.scroll = h.line_index;
                }
            }
        } else {
            self.open_doc(idx);
        }
    }

    fn toggle_theme(&mut self) {
        let doc = match &self.current {
            Some(d) => d,
            None => return,
        };
        // compute current dark state and flip
        let hash = crate::store::content_hash::sha256_hex(
            &doc.lines.iter().map(|l| l.spans.iter().map(|s| s.content.to_string()).collect::<String>()).collect::<String>(),
        );
        let per_doc = self.ctx.theme_store.lock().unwrap().dark_for(&hash);
        let system_dark = false; // TUI can't easily detect system theme; treat as light
        let pref = self.ctx.settings.lock().unwrap().theme_pref();
        let is_dark = resolve_dark(per_doc, pref, system_dark);
        self.ctx.theme_store.lock().unwrap().set_dark(!is_dark, &hash);
    }
}

/// Run the TUI event loop. Restores the terminal on exit.
pub fn run(app: App) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = app;
    let result = main_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    result
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if !event::poll(std::time::Duration::from_millis(250))? {
            continue;
        }
        let ev = match event::read()? {
            Event::Key(k) if k.kind == KeyEventKind::Press => k,
            _ => continue,
        };

        match (ev.code, ev.modifiers) {
            (KeyCode::Char('q'), _) => return Ok(()),
            (KeyCode::Tab, _) => {
                app.side_tab = match app.side_tab {
                    SideTab::Library => SideTab::Outline,
                    SideTab::Outline => SideTab::Library,
                };
                app.list_state.select(Some(0));
            }
            (KeyCode::Char('j') | KeyCode::Down, _) => {
                if app.side_tab == SideTab::Outline && app.current.is_some()
                    && (ev.modifiers.contains(KeyModifiers::SHIFT)
                        || app.list_state.selected().map(|i| i >= app.current.as_ref().unwrap().headings.len().saturating_sub(1)).unwrap_or(true))
                {
                    app.scroll_content(1);
                } else {
                    app.move_selection(1);
                }
            }
            (KeyCode::Char('k') | KeyCode::Up, _) => app.move_selection(-1),
            (KeyCode::Char('J'), _) => app.scroll_content(3),
            (KeyCode::Char('K'), _) => app.scroll_content(-3),
            (KeyCode::PageDown, _) => app.scroll_content(10),
            (KeyCode::PageUp, _) => app.scroll_content(-10),
            (KeyCode::Char('G'), _) => {
                if let Some(d) = &app.current {
                    app.scroll = d.lines.len().saturating_sub(1);
                }
            }
            (KeyCode::Char('g'), _) => app.scroll = 0,
            (KeyCode::Enter, _) => app.jump_to_heading(),
            (KeyCode::Char('r'), _) => {
                if let Some(idx) = app.selected_doc {
                    app.open_doc(idx);
                }
            }
            (KeyCode::Char('t'), _) => app.toggle_theme(),
            (KeyCode::Char('?'), _) => { /* TODO: help overlay */ }
            _ => {}
        }
    }
}

fn draw(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();

    // Layout: top bar (title) + main split (sidebar | content) + bottom bar (help)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Top bar: doc title + theme indicator
    let title = app
        .current
        .as_ref()
        .map(|d| d.title.as_str())
        .unwrap_or("MDreader — 文档库");
    let top = Paragraph::new(format!(" {} ", title))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(top, chunks[0]);

    // Main split: sidebar (30%) | content (70%)
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    // Sidebar: tabs (库/大纲) + list
    let sidebar_block = Block::default().borders(Borders::RIGHT);
    let sidebar_inner = sidebar_block.inner(main[0]);
    f.render_widget(sidebar_block, main[0]);

    let tab_titles = vec![
        Span::raw(if app.side_tab == SideTab::Library { "[库] " } else { " 库  " }),
        Span::raw(if app.side_tab == SideTab::Outline { "[大纲]" } else { " 大纲" }),
    ];
    let tabs = Tabs::new(tab_titles);
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(sidebar_inner);
    f.render_widget(tabs, sidebar_chunks[0]);

    // List content depends on active tab (clone items out to release the borrow
    // before we mutably borrow list_state for rendering).
    let items: Vec<ListItem> = match app.side_tab {
        SideTab::Library => build_library_items(app),
        SideTab::Outline => build_outline_items(app),
    };
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    let mut list_state = app.list_state.clone();
    f.render_stateful_widget(list, sidebar_chunks[1], &mut list_state);
    app.list_state = list_state;

    // Content area
    let content_block = Block::default();
    let content_inner = content_block.inner(main[1]);
    f.render_widget(content_block, main[1]);

    if let Some(doc) = &app.current {
        let visible: Vec<Line> = doc
            .lines
            .iter()
            .skip(app.scroll)
            .take(content_inner.height as usize)
            .cloned()
            .collect();
        let para = Paragraph::new(visible);
        f.render_widget(para, content_inner);
    } else {
        let para = Paragraph::new("按 Tab 切换到库，选择一个文档按 Enter 打开。\n\n或运行: mdreader-tui <file.md>")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(para, content_inner);
    }

    // Bottom bar: help
    let help = " Tab:库/大纲  j/k:选择  Enter:打开/跳转  J/K:滚动  r:刷新  t:主题  q:退出 ";
    let bottom = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    f.render_widget(bottom, chunks[2]);
}

fn build_library_items(app: &App) -> Vec<ListItem> {
    let mut items = Vec::new();
    // group by date bucket (simplified: flat list with bucket headers)
    let now = chrono::Local::now().timestamp_millis();
    let mut last_bucket: Option<date_buckets::DayBucket> = None;
    for doc in &app.docs {
        let bucket = date_buckets::bucket(doc.opened_at, now);
        if last_bucket != Some(bucket) {
            let label = match bucket {
                date_buckets::DayBucket::Today => "── 今天 ──",
                date_buckets::DayBucket::Yesterday => "── 昨天 ──",
                date_buckets::DayBucket::Earlier => "── 更早 ──",
            };
            items.push(ListItem::new(Line::from(vec![Span::styled(
                label,
                Style::default().fg(Color::DarkGray),
            )])));
            last_bucket = Some(bucket);
        }
        let fav = if doc.favorite { "★ " } else { "  " };
        items.push(ListItem::new(format!("{}{}", fav, doc.title)));
    }
    if items.is_empty() {
        items.push(ListItem::new("（空）打开一个 .md 文件试试"));
    }
    items
}

fn build_outline_items(app: &App) -> Vec<ListItem> {
    match &app.current {
        Some(doc) => {
            if doc.headings.is_empty() {
                vec![ListItem::new("（无标题）")]
            } else {
                doc.headings
                    .iter()
                    .map(|h| {
                        let indent = "  ".repeat((h.level.saturating_sub(1)) as usize);
                        ListItem::new(format!("{}{}", indent, h.text))
                    })
                    .collect()
            }
        }
        None => vec![ListItem::new("（未打开文档）")],
    }
}
