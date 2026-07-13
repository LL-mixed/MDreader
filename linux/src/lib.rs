//! MDreader core library — pure-logic layer shared by the GTK GUI and the TUI.
//!
//! Everything here is free of GUI dependencies (no gtk/WebKit/glib/gio). The GUI
//! binary (`src/bin/mdreader.rs`) additionally pulls in `render::webview` and
//! `app` behind the `gui` feature; the TUI binary (`src/bin/tui.rs`) builds with
//! `--no-default-features` and uses only this lib.

pub mod build_info;
pub mod config;
pub mod context;
pub mod render;
pub mod store;
pub mod tui;
pub mod util;

#[cfg(test)]
mod shared_spec_tests;
