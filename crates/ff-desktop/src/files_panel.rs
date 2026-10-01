//! # Files Panel (adapter)
//!
//! The Virtual Catalog Files Panel (tree + content area + context-menu and
//! inline-form types) was extracted into the standalone `ff-files-panel`
//! crate (decomposition Wave 7, Task 24). This module is now a thin re-export
//! so every existing `crate::files_panel::*` reference (shell/render,
//! shell/update), including `ContentEntry` and
//! `FilesPanelState::create_dataset_file`, resolves unchanged against the
//! extracted crate's public API.
//!
//! The shell-entangled dialog state machine (opening dialogs from a
//! `FilesPanelAction`, the `TabKind::FilesPanel` render arm, resolve-and-open
//! into an editor tab, the `render_dialogs` match over `FilesDialogState`, and
//! the session registry save) stays in `shell/render.rs` and `shell/update.rs`
//! and calls into the re-exported items here.

pub use ff_files_panel::*;
