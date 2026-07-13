//! Markdown → ratatui `Line` renderer.
//!
//! Parses markdown with `pulldown-cmark` and maps CommonMark events to styled
//! terminal lines. This replaces the GUI's marked.js+WebView pipeline for a
//! terminal-only context. Inline formatting (bold/italic/code/strikethrough)
//! becomes `Span` styles; block elements (headings, lists, quotes, code blocks,
//! tables, rules) become `Line`s with appropriate indentation and color.
//!
//! Images / math / mermaid / SVG degrade gracefully: images show `⟨img: src⟩`,
//! math shows raw LaTeX, mermaid/SVG show as fenced code.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

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

/// Render markdown source into styled terminal lines + an outline.
pub fn render(markdown: &str) -> Rendered {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown, opts);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut headings: Vec<HeadingRef> = Vec::new();
    let mut inline_spans: Vec<Span<'static>> = Vec::new();
    let mut list_depth: usize = 0;
    let mut in_code_block: bool = false;
    let mut code_buf: String = String::new();
    let mut in_quote = false;
    let mut pending_heading: Option<(HeadingLevel, String)> = None;

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
                    // store language for potential future syntax hinting
                    if let CodeBlockKind::Fenced(lang) = kind {
                        code_buf.push_str(&format!("```{}\n", lang));
                    }
                }
                Tag::List(_) => list_depth += 1,
                Tag::BlockQuote(_) => in_quote = true,
                Tag::Image { dest_url, .. } => {
                    inline_spans.push(Span::styled(
                        format!("⟨img: {}⟩", dest_url),
                        Style::default().fg(Color::DarkGray),
                    ));
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
                    if !inline_spans.is_empty() {
                        let prefix = prefix_for(list_depth, in_quote);
                        let mut spans: Vec<Span<'static>> = Vec::new();
                        if !prefix.is_empty() {
                            spans.push(Span::raw(prefix));
                        }
                        spans.append(&mut inline_spans);
                        lines.push(Line::from(spans));
                    }
                    inline_spans.clear();
                }
                TagEnd::CodeBlock => {
                    if in_code_block {
                        in_code_block = false;
                        lines.push(
                            Span::raw("  ┌─ code ─────────────────────────────────────").into(),
                        );
                        for code_line in code_buf.lines() {
                            lines.push(Line::from(vec![Span::styled(
                                format!("  │ {}", code_line),
                                Style::default().fg(Color::Cyan),
                            )]));
                        }
                        lines.push(
                            Span::raw("  └─────────────────────────────────────────────").into(),
                        );
                        lines.push(Line::raw(""));
                        code_buf.clear();
                    }
                }
                TagEnd::List(_) => {
                    if list_depth > 0 {
                        list_depth -= 1;
                    }
                    if list_depth == 0 {
                        lines.push(Line::raw(""));
                    }
                }
                TagEnd::BlockQuote(_) => in_quote = false,
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_buf.push_str(&text);
                } else if let Some((_, heading_text)) = &mut pending_heading {
                    heading_text.push_str(&text);
                } else {
                    inline_spans.push(Span::raw(text.into_string()));
                }
            }
            Event::Code(text) => {
                inline_spans.push(Span::styled(
                    text.into_string(),
                    Style::default().fg(Color::Cyan),
                ));
            }
            Event::SoftBreak => {
                inline_spans.push(Span::raw("\n"));
            }
            Event::HardBreak => {
                let prefix = prefix_for(list_depth, in_quote);
                let mut spans: Vec<Span<'static>> = Vec::new();
                if !prefix.is_empty() {
                    spans.push(Span::raw(prefix));
                }
                spans.append(&mut inline_spans);
                lines.push(Line::from(spans));
                inline_spans.clear();
            }
            Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}
            Event::InlineMath(text) | Event::DisplayMath(text) => {
                inline_spans.push(Span::styled(
                    text.into_string(),
                    Style::default().fg(Color::Magenta),
                ));
            }
            Event::Html(html) => {
                inline_spans.push(Span::styled(
                    html.into_string(),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            Event::InlineHtml(html) => {
                inline_spans.push(Span::styled(
                    html.into_string(),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            Event::Rule => {
                // horizontal rule: a line of dashes
                let prefix = prefix_for(list_depth, in_quote);
                lines.push(Line::from(format!("{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", prefix)));
            }
        }
    }

    if !inline_spans.is_empty() {
        lines.push(Line::from(inline_spans));
    }

    Rendered { lines, headings }
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

fn prefix_for(list_depth: usize, in_quote: bool) -> String {
    let mut p = String::new();
    if in_quote {
        p.push_str("  │ ");
    }
    for _ in 0..list_depth.saturating_sub(1) {
        p.push_str("  ");
    }
    if list_depth > 0 && !in_quote {
        p.push_str("• ");
    }
    p
}
