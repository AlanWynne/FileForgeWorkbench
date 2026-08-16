//! # ff-viewport-scrolling — GUI-Independent Viewport Management
//!
//! This crate implements the viewport and scrolling subsystem for
//! FileForgeWorkbench. It tracks the visible portion of a document, manages
//! scroll state, caret visibility policies, column affinity, smooth scrolling,
//! and scrollbar mapping.
//!
//! ## Design Principles
//!
//! - **GUI-independent** — no dependency on egui, winit, or any rendering
//!   framework. The viewport model computes positions; GUI shells render.
//! - **Command-driven** — all scroll operations are expressible as commands
//!   for integration with the command framework.
//! - **Configurable** — caret policies, scroll mode, and wheel speed are
//!   adjustable at runtime.
//! - **Testable** — pure-function scrollbar mapping and property-based tests
//!   validate all invariants.
//!
//! ## Architecture
//!
//! ```text
//! ViewportModel ← core state (top_line, visible_count, offsets)
//! CursorModel   ← cursor position + column affinity
//! CaretPolicyEngine ← viewport adjustment rules
//! VerticalScrollbar / HorizontalScrollbar ← pure-function mapping
//! SmoothScrollEngine ← pixel-level scrolling targets
//! ScrollCommand ← command definitions for framework integration
//! ViewportSnapshot ← serialisable state for persistence
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_viewport_scrolling::{ViewportModel, CursorModel, CaretPolicyEngine};
//!
//! let mut viewport = ViewportModel::with_line_count(1000);
//! viewport.set_visible_count(40);
//!
//! let mut cursor = CursorModel::new();
//! let policy = CaretPolicyEngine::default_policy();
//!
//! // Scroll down one page
//! viewport.scroll_page_down(&mut cursor);
//! assert_eq!(viewport.top_line(), 41);
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Core newtypes (DisplayLine, ScrollFraction, PixelOffset, etc.).
pub mod types;

/// Error types for the viewport-scrolling crate.
pub mod error;

/// Core viewport state model.
pub mod viewport;

/// Cursor position and column affinity tracking.
pub mod cursor;

/// Caret visibility policy engine.
pub mod caret_policy;

/// Scrollbar models (vertical and horizontal).
pub mod scrollbar;

/// Smooth scrolling engine.
pub mod smooth;

/// Scroll command definitions and handler.
pub mod commands;

/// Viewport state-change events and observer trait.
pub mod events;

/// Viewport state persistence (snapshot and restore).
pub mod snapshot;

/// Display line mapper trait for wrapping/folding integration.
pub mod display_mapper;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use caret_policy::{CaretPolicy, CaretPolicyConfig, CaretPolicyEngine};
pub use commands::{execute_scroll_command, ScrollCommand};
pub use cursor::{AffinityMode, CursorModel};
pub use display_mapper::DisplayLineMapper;
pub use error::ViewportError;
pub use events::{ViewportChanged, ViewportObserver};
pub use scrollbar::horizontal::HorizontalScrollbar;
pub use scrollbar::vertical::VerticalScrollbar;
pub use smooth::{SmoothScrollEngine, SmoothScrollTarget};
pub use snapshot::ViewportSnapshot;
pub use types::{ColumnOffset, DisplayLine, PixelOffset, ScrollFraction, ScrollMode, WheelTicks};
pub use viewport::{ScrollbarFeedback, ViewportModel};
