//! # ff-zoom — Per-Editor-Instance Zoom Management
//!
//! This crate provides the zoom offset model for FileForgeWorkbench. Each editor
//! instance maintains a signed integer point-size offset applied to the base editor
//! font. The zoom offset affects only the editor content area and is a display-only
//! transformation — it never modifies document content.
//!
//! ## Core Types
//!
//! - [`ZoomOffset`] — signed integer newtype representing the point offset
//! - [`ZoomConfig`] — validated configuration (step, min/max range, default offset)
//! - [`ZoomState`] — per-editor-instance mutable zoom state
//! - [`ZoomResult`] — outcome of a zoom operation (applied or at-limit)
//! - [`ZoomChangeEvent`] — notification emitted after offset mutations
//! - [`ZoomIndicatorState`] — data model for the status bar indicator
//! - [`ZoomSessionEntry`] — serialisable snapshot for session persistence
//!
//! ## Architecture
//!
//! The crate has zero GUI dependencies. It operates on abstract types and emits
//! events that the rendering layer consumes to trigger re-layout. Keyboard
//! shortcuts and mouse wheel handling produce `ZoomOperation` values that are
//! applied to a `ZoomState` via its methods.

pub mod commands;
pub mod config;
pub mod error;
pub mod indicator;
pub mod operations;
pub mod persistence;
pub mod state;
pub mod types;

// Public API re-exports
pub use config::{ConfigWarning, RawZoomConfig, ZoomConfig};
pub use error::ZoomError;
pub use indicator::{ZoomIndicatorState, ZoomQuickPickOption};
pub use operations::{ZoomChangeEvent, ZoomFontMetrics, ZoomResult};
pub use persistence::ZoomSessionEntry;
pub use state::ZoomState;
pub use types::ZoomOffset;
