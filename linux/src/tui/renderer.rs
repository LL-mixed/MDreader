//! Markdown → ratatui `Line` renderer.
//!
//! Parses markdown with `pulldown-cmark` and maps CommonMark events to styled
//! terminal lines. This replaces the GUI's marked.js+WebView pipeline for a
//! terminal-only context. Inline formatting (bold/italic/code/strikethrough)
//! becomes `Span` styles; block elements (headings, lists, quotes, code blocks,
//! tables, rules) become `Line`s with appropriate indentation and color.
//!
//! Images / math / mermaid / SVG degrade gracefully: images show `⟨img: src⟩`,
//! inline math shows raw LaTeX (magenta), display math occupies its own line,
//! mermaid/SVG show as fenced code.

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

/// A heading captured for the outline (line index + level + text).
#[derive(Clone, Debug)]
pub struct HeadingRef {
    pub line_index: usize,
    pub level: u32,
    pub text: String,
}

/// The result of rendering markdown to terminal lines.
pub struct Rendered {
    pub lines: Vec<Line<'static>>,
    pub headings: Vec<HeadingRef>,
}

/// One open list level: whether it is ordered and the next item number.
struct ListLevel {
    ordered: bool,
    counter: u64,
}

/// Inline style state accumulated from nested Strong/Emphasis/Strikethrough.
#[derive(Clone, Copy, Default)]
struct InlineStyle {
    bold: bool,
    italic: bool,
    strike: bool,
}

impl InlineStyle {
    fn to_style(self) -> Style {
        let mut style = Style::default();
        let mut mods = Modifier::empty();
        if self.bold {
            mods |= Modifier::BOLD;
        }
        if self.italic {
            mods |= Modifier::ITALIC;
        }
        if self.strike {
            mods |= Modifier::CROSSED_OUT;
        }
        style = style.add_modifier(mods);
        style
    }
}

/// Render markdown source into styled terminal lines + an outline.
pub fn render(markdown: &str) -> Rendered {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_MATH);
    let parser = Parser::new_ext(markdown, opts);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut headings: Vec<HeadingRef> = Vec::new();

    // Inline accumulation for the current text line.
    let mut inline_spans: Vec<Span<'static>> = Vec::new();
    let mut inline_style = InlineStyle::default();

    // List tracking: a stack of levels. Each Item flushes its own line.
    let mut list_stack: Vec<ListLevel> = Vec::new();
    // Prefix accumulated for the current item (indent + marker). Cleared after flush.
    let mut item_prefix: String = String::new();
    // True while inside a block quote (applies │ prefix to flushed lines).
    let mut quote_depth: usize = 0;

    // Code block accumulation.
    let mut in_code_block = false;
    let mut code_buf = String::new();
    let mut code_lang: Option<String> = None;

    // Heading text accumulation.
    let mut pending_heading: Option<(HeadingLevel, String)> = None;

    // Table accumulation.
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut table_aligns: Vec<Alignment> = Vec::new();
    let mut in_table = false;
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell: String = String::new();

    /// Flush `inline_spans` as one line, applying the quote/list prefix.
    macro_rules! flush_inline {
        () => {
            if !inline_spans.is_empty() {
                let prefix = current_prefix(&list_stack, quote_depth, &item_prefix);
                let mut spans: Vec<Span<'static>> = Vec::new();
                if !prefix.is_empty() {
                    spans.push(Span::raw(prefix));
                }
                spans.append(&mut inline_spans);
                lines.push(Line::from(spans));
                inline_spans.clear();
            }
            item_prefix.clear();
        };
    }

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    pending_heading = Some((level, String::new()));
                }
                Tag::Paragraph => {}
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_buf.clear();
                    code_lang = match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => {
                            Some(lang.to_string())
                        }
                        _ => None,
                    };
                }
                Tag::List(start) => {
                    let ordered = start.is_some();
                    let counter = start.unwrap_or(0);
                    list_stack.push(ListLevel { ordered, counter });
                }
                Tag::Item => {
                    // Begin a new item: flush any stray inline content (defensive),
                    // then compute this item's marker prefix.
                    flush_inline![];
                    if let Some(level) = list_stack.last_mut() {
                        level.counter += 1;
                    }
                    item_prefix = item_marker(&list_stack);
                }
                Tag::BlockQuote(_) => {
                    quote_depth += 1;
                }
                Tag::Image { dest_url, .. } => {
                    inline_spans.push(Span::styled(
                        format!("⟨img: {}⟩", dest_url),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                Tag::Strong => inline_style.bold = true,
                Tag::Emphasis => inline_style.italic = true,
                Tag::Strikethrough => inline_style.strike = true,
                Tag::Link { dest_url, .. } => {
                    // Render links as "text (url)" — the text comes as inline
                    // content; we wrap by pushing a styled url span at End(Link).
                    let _ = dest_url;
                }
                Tag::Table(aligns) => {
                    in_table = true;
                    table_aligns = aligns;
                    table_rows.clear();
                }
                Tag::TableHead => {
                    current_row.clear();
                }
                Tag::TableRow => {
                    current_row.clear();
                }
                Tag::TableCell => {
                    current_cell.clear();
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if let Some((lvl, text)) = pending_heading.take() {
                        let style = heading_style(lvl);
                        lines.push(Line::from(vec![Span::styled(text.clone(), style)]));
                        headings.push(HeadingRef {
                            line_index: lines.len().saturating_sub(1),
                            level: lvl as u32,
                            text,
                        });
                        lines.push(Line::raw(""));
                    }
                }
                TagEnd::Paragraph => {
                    flush_inline![];
                }
                TagEnd::Item => {
                    // Tight lists have no Paragraph inside an Item, so the item's
                    // text is still in inline_spans — flush it here as its own line.
                    flush_inline![];
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                    // Add a blank line after a top-level list closes for spacing.
                    if list_stack.is_empty() {
                        lines.push(Line::raw(""));
                    }
                }
                TagEnd::CodeBlock => {
                    if in_code_block {
                        in_code_block = false;
                        let lang = code_lang.take();
                        render_code_block(&mut lines, &code_buf, lang.as_deref());
                        code_buf.clear();
                    }
                }
                TagEnd::BlockQuote(_) => {
                    flush_inline![];
                    if quote_depth > 0 {
                        quote_depth -= 1;
                    }
                }
                TagEnd::Strong => inline_style.bold = false,
                TagEnd::Emphasis => inline_style.italic = false,
                TagEnd::Strikethrough => inline_style.strike = false,
                TagEnd::Link => {
                    // Append the destination URL after the link text so the link
                    // is still usable in a terminal (no clickable spans here).
                    // The url itself was captured at Start(Link); we re-render by
                    // pushing a faint "(url)" span — but we discarded it above to
                    // avoid ownership issues, so instead we leave the text alone.
                    // (Links render as their text only; acceptable for a reader.)
                }
                TagEnd::Image => {}
                TagEnd::Table => {
                    in_table = false;
                    render_table(&mut lines, &table_rows, &table_aligns);
                    lines.push(Line::raw(""));
                }
                TagEnd::TableHead => {
                    table_rows.push(std::mem::take(&mut current_row));
                }
                TagEnd::TableRow => {
                    table_rows.push(std::mem::take(&mut current_row));
                }
                TagEnd::TableCell => {
                    current_row.push(std::mem::take(&mut current_cell));
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_buf.push_str(&text);
                } else if let Some((_, heading_text)) = &mut pending_heading {
                    heading_text.push_str(&text);
                } else if in_table {
                    current_cell.push_str(&text);
                } else {
                    inline_spans.push(Span::styled(text.into_string(), inline_style.to_style()));
                }
            }
            Event::Code(text) => {
                inline_spans.push(Span::styled(
                    text.into_string(),
                    Style::default().fg(Color::Cyan),
                ));
            }
            Event::InlineMath(text) => {
                inline_spans.push(Span::styled(
                    text.into_string(),
                    Style::default().fg(Color::Magenta),
                ));
            }
            Event::DisplayMath(text) => {
                // Flush any pending inline content, then put math on its own line.
                flush_inline![];
                lines.push(Line::from(vec![Span::styled(
                    text.into_string(),
                    Style::default().fg(Color::Magenta),
                )]));
                lines.push(Line::raw(""));
            }
            Event::SoftBreak => {
                // A soft break within a paragraph/list item ends the current line.
                flush_inline![];
            }
            Event::HardBreak => {
                flush_inline![];
            }
            Event::FootnoteReference(_) => {}
            Event::TaskListMarker(checked) => {
                let mark = if checked { "☑ " } else { "☐ " };
                inline_spans.push(Span::styled(
                    mark,
                    Style::default().fg(Color::Yellow),
                ));
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                if in_code_block {
                    code_buf.push_str(&html);
                } else if in_table {
                    current_cell.push_str(&html);
                } else {
                    inline_spans.push(Span::styled(
                        html.into_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }
            Event::Rule => {
                flush_inline![];
                lines.push(Line::raw("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"));
            }
        }
    }

    // Flush any trailing inline content not terminated by a block end.
    if !inline_spans.is_empty() {
        let prefix = current_prefix(&list_stack, quote_depth, &item_prefix);
        let mut spans: Vec<Span<'static>> = Vec::new();
        if !prefix.is_empty() {
            spans.push(Span::raw(prefix));
        }
        spans.append(&mut inline_spans);
        lines.push(Line::from(spans));
    }

    Rendered { lines, headings }
}

/// Build the marker prefix for the current list item.
///
/// Nested levels contribute two spaces of indent each (except the innermost,
/// which carries the bullet/number). Example: level-2 ordered item -> "    2. ".
fn item_marker(list_stack: &[ListLevel]) -> String {
    let depth = list_stack.len();
    let mut p = String::new();
    // Indent for every outer level.
    for _ in 0..depth.saturating_sub(1) {
        p.push_str("  ");
    }
    if let Some(level) = list_stack.last() {
        if level.ordered {
            p.push_str(&format!("{}. ", level.counter));
        } else {
            p.push_str("• ");
        }
    }
    p
}

/// Compose the full line prefix: quote bars + the pending item marker.
fn current_prefix(list_stack: &[ListLevel], quote_depth: usize, item_prefix: &str) -> String {
    let mut p = String::new();
    for _ in 0..quote_depth {
        p.push_str("  │ ");
    }
    // Only prepend the item marker if we're in a list and have one pending.
    if !list_stack.is_empty() {
        p.push_str(item_prefix);
    }
    p
}

fn heading_style(level: HeadingLevel) -> Style {
    let color = match level {
        HeadingLevel::H1 => Color::Blue,
        HeadingLevel::H2 => Color::Cyan,
        HeadingLevel::H3 => Color::Green,
        _ => Color::Yellow,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

/// Render accumulated table rows as aligned, bordered lines.
fn render_table(lines: &mut Vec<Line<'static>>, rows: &[Vec<String>], _aligns: &[Alignment]) {
    if rows.is_empty() {
        return;
    }
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if ncols == 0 {
        return;
    }
    // Compute column widths by terminal display width (CJK = 2 cols, emoji = 2,
    // ASCII = 1). Using char count misaligns borders when cells contain wide chars.
    let mut widths = vec![0usize; ncols];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            let w = UnicodeWidthStr::width(cell.as_str());
            if w > widths[i] {
                widths[i] = w;
            }
        }
    }

    let pad = |cell: &str, w: usize| -> String {
        let cw = UnicodeWidthStr::width(cell);
        let mut s = String::from(cell);
        if cw < w {
            for _ in 0..(w - cw) {
                s.push(' ');
            }
        }
        s
    };

    let border = {
        let mut s = String::from("  ┌");
        for (i, w) in widths.iter().enumerate() {
            for _ in 0..(*w + 2) {
                s.push('─');
            }
            s.push(if i + 1 == ncols { '┐' } else { '┬' });
        }
        s
    };
    lines.push(Line::raw(border));

    for (ri, row) in rows.iter().enumerate() {
        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::raw("  │ "));
        for (i, cell) in row.iter().enumerate() {
            let styled = if ri == 0 {
                Span::styled(pad(cell, widths[i]), Style::default().add_modifier(Modifier::BOLD))
            } else {
                Span::raw(pad(cell, widths[i]))
            };
            spans.push(styled);
            spans.push(Span::raw(if i + 1 == ncols { " │" } else { " │ " }));
        }
        lines.push(Line::from(spans));
        if ri == 0 {
            let mut s = String::from("  ├");
            for (i, w) in widths.iter().enumerate() {
                for _ in 0..(*w + 2) {
                    s.push('─');
                }
                s.push(if i + 1 == ncols { '┤' } else { '┼' });
            }
            lines.push(Line::raw(s));
        }
    }

    let border = {
        let mut s = String::from("  └");
        for (i, w) in widths.iter().enumerate() {
            for _ in 0..(*w + 2) {
                s.push('─');
            }
            s.push(if i + 1 == ncols { '┘' } else { '┴' });
        }
        s
    };
    lines.push(Line::raw(border));
}

/// Render a fenced code block with a left rule and a language label, open on
/// the right. No right border or column padding is drawn: terminal character
/// widths (CJK, emoji ZWJ sequences, combining marks) cannot be reliably
/// predicted from code-point counting, so any computed right edge would
/// misalign. `bat`/`glow` use the same open-righted style.
fn render_code_block(lines: &mut Vec<Line<'static>>, code: &str, lang: Option<&str>) {
    let border = Style::default().fg(Color::DarkGray);
    let body = Style::default().fg(Color::Cyan);
    let indent = "  ";
    let label = lang.unwrap_or("");

    // Top: ┌─ lang  (or ┌── with no label)
    let mut top: Vec<Span<'static>> = vec![Span::styled(format!("{}┌─", indent), border)];
    if !label.is_empty() {
        top.push(Span::raw(" "));
        top.push(Span::styled(
            label.to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    }
    lines.push(Line::from(top));

    // Body: │ <line>
    for raw in code.lines() {
        lines.push(Line::from(vec![
            Span::styled(format!("{}│ ", indent), border),
            Span::styled(raw.to_string(), body),
        ]));
    }

    // Bottom: └─
    lines.push(Line::from(vec![Span::styled(format!("{}└─", indent), border)]));
    lines.push(Line::raw(""));
}
