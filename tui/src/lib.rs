//! mdreader_core — the shared pure-logic layer of MDreader.
//!
//! Owns the data layer (SQLite cache, JSON stores), markdown preprocessing,
//! config paths, and build info. Consumed by:
//! - the `mdreader-tui` binary (this crate's own bin, cross-platform terminal)
//! - the `mdreader` GTK GUI binary in `linux/` (via path dependency)
//!
//! Everything here is free of GUI dependencies (no gtk/WebKit/glib/gio).

pub mod build_info;
pub mod config;
pub mod context;
pub mod render;
pub mod store;
pub mod tui;
pub mod util;

#[cfg(test)]
mod shared_spec_tests;
