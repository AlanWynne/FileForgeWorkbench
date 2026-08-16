//! # ff-wrap — Per-Editor-Instance Line Wrap Management
//!
//! This crate provides the line wrap toggle subsystem for FileForgeWorkbench.
//! It controls whether and how long document lines are visually broken across
//! multiple display rows to fit within a configured boundary width.
//!
//! ## Core Types
//!
//! - [`WrapMode`] — enum with variants None, Word, Character
//! - [`WrapBoundary`] — viewport-width dynamic or fixed-column static wrapping
//! - [`WrapColumn`] — validated column number newtype (1–10000)
//! - [`WrapIndentMode`] — continuation line indent mode (Fixed/Same/Indent/DeepIndent)
//! - [`WrapVisualFlags`] — visual markers at wrap break points
//! - [`WrapConfig`] — validated configuration from `[view.wrap]` TOML namespace
//! - [`WrapState`] — per-editor-instance mutable wrap settings
//! - [`WrapSnapshot`] — serialisable state for session persistence
//! - [`WrapOperation`] — command operations (On/Off/Toggle/Word/Char/Col)
//!
//! ## Architecture
//!
//! The crate has zero GUI dependencies. It operates on abstract types and emits
//! results that the rendering layer consumes to trigger re-layout. The WRAP
//! command and its sub-commands produce `WrapOperation` values that are applied
//! to a `WrapState` via the command handler.
//!
//! Line wrapping is a **display-only state change**: it never modifies document
//! content, never produces UndoRecords, and is never recorded in command history.

pub mod boundary;
pub mod commands;
pub mod config;
pub mod error;
pub mod indent;
pub mod indicator;
pub mod layout;
pub mod mode;
pub mod persistence;
pub mod scrollbar;
pub mod state;
pub mod visual_flags;

// Public API re-exports
pub use boundary::{WrapBoundary, WrapColumn};
pub use commands::{
    execute_wrap_operation, format_status_message, parse_wrap_args, WrapCommandResult,
    WrapOperation, WRAP_COMMAND_ID,
};
pub use config::{ConfigWarning, RawWrapConfig, WrapConfig};
pub use error::WrapError;
pub use indent::WrapIndentMode;
pub use indicator::{format_indicator, next_mode_in_cycle};
pub use layout::{
    compute_breaks, compute_char_breaks, compute_height_from_width, compute_sub_line_count,
    compute_sub_lines, compute_word_breaks, first_non_whitespace_col, resolve_indent_offset,
    SubLineInfo,
};
pub use mode::WrapMode;
pub use persistence::WrapSnapshot;
pub use scrollbar::{
    scrollbar_visibility, should_reset_horizontal_offset, should_show_horizontal_scrollbar,
    ScrollbarVisibility,
};
pub use state::{WrapModeChange, WrapState};
pub use visual_flags::{compute_markers, WrapMarkerLocation, WrapMarkerPosition, WrapVisualFlags};
