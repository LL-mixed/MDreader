//! Terminal UI (TUI) for MDreader — a ratatui-based markdown reader that runs
//! in the terminal, sharing the same data layer (cache/session/theme) as the GUI.
//!
//! The markdown rendering pipeline reuses the lib's pure-logic preprocessing
//! (`mermaid_fence::normalize`) and then parses with `pulldown-cmark` to produce
//! styled `ratatui::text::Line`s.

pub mod app;
pub mod renderer;
