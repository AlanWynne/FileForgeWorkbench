//! # ff-tabs — Multi-Tab Editor Subsystem for FileForgeWorkbench
//!
//! This crate implements the multi-tab editor subsystem. It owns the tab data
//! model, tab collection management, per-tab state isolation, MRU ordering,
//! pinned tabs, duplicate detection, tab context menu logic, session
//! serialisation, and the rendering contract for the GUI shell's tab bar.
//!
//! ## Capabilities
//!
//! - **Tab Collection**: Ordered storage of tabs within Tab_Groups
//! - **Per-Tab State**: Independent viewport, cursor, selections, folds, bookmarks per tab
//! - **MRU Stack**: Most-recently-used ordering for Ctrl+Tab navigation
//! - **Pinned Tabs**: Pin/unpin with positional enforcement and bulk-close immunity
//! - **Duplicate Detection**: ResourceUri deduplication across all Tab_Groups
//! - **Context Menu**: Conditional menu model for right-click operations
//! - **Drag-and-Drop**: Reorder state machine for within-group and cross-group moves
//! - **Tab Bar Model**: Rendering contract (titles, indicators, overflow)
//! - **Session Persistence**: Serialise/deserialise full tab state
//! - **Command Registration**: All tab operations as registered commands
//!
//! ## Architecture
//!
//! This is a Wave 8 (File I/O and Session) crate depending on:
//! - `ff-command` (command registration and dispatch)
//! - `ff-document-model` (DocumentHandle)
//! - `ff-layout` (TabGroupId, split requests)
//! - `ff-vfs` (ResourceUri)
//! - `ff-config` (configuration settings)
//! - `ff-undo-redo` (TransactionStack reference)
//! - `ff-logging` (diagnostics)
//!
//! ## GUI Independence
//!
//! All tab logic is GUI-independent. The shell layer renders tab headers
//! using the model exposed by this crate but does not own it.

// ─── Public Modules ─────────────────────────────────────────────────────────

pub mod commands;
pub mod context_menu;
pub mod drag_drop;
pub mod duplicate_detection;
pub mod error;
pub mod keyboard_nav;
pub mod mru_stack;
pub mod overflow;
pub mod pinned;
pub mod session;
pub mod split_view;
pub mod tab;
pub mod tab_bar;
pub mod tab_collection;
pub mod tab_id;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use error::TabsError;
pub use tab_id::TabId;
