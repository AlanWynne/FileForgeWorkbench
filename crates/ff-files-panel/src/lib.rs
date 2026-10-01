//! # ff-files-panel -- Virtual Catalog Files Panel
//!
//! Renders the unified virtual file catalog explorer opened by POM option 1.
//! Provides a split layout: left catalog tree + right content area, plus the
//! context-menu item sets and inline dialog-form structs.
//!
//! Extracted from `ff-desktop::files_panel` in decomposition Wave 7 (Task 24).
//! The pure data model, tree/content render, context-menu item sets and
//! inline-form structs live here; the shell-entangled dialog state machine
//! (opening dialogs, resolve-and-open into an editor tab, session registry
//! save) stays in `ff-desktop` and calls into this crate's public API. This
//! crate therefore has NO dependency on `ff-desktop`.
//!
//! Validates: Requirement 1.1-1.7, 4.1, 4.3, 10.1-10.6

#![allow(dead_code)]

mod content;
mod forms;
mod menus;
mod resolve;
mod state;
mod tree;

// `content` exposes only the `pub(crate)` `render_content_area` helper (called
// by `tree::render`); it has no public surface to re-export.
pub use forms::*;
pub use menus::*;
pub use resolve::*;
pub use state::*;
pub use tree::*;

#[cfg(test)]
mod tests;
