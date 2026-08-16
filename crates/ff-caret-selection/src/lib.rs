//! # ff-caret-selection — Caret Appearance and Selection Display for FileForgeWorkbench
//!
//! This crate provides the visual presentation layer for carets, selections,
//! caret-line highlighting, virtual space rendering, and modified-line markers.
//! It consumes the logical selection model from `ff-edit-operations` and exposes
//! a GUI-independent query API that shell renderers use to paint these visual elements.
//!
//! ## Design Principles
//!
//! - **GUI Independence** — stores configuration and exposes query methods; actual
//!   drawing is performed by the shell layer
//! - **Timer-Agnostic Blink** — the blink model stores only period and last-reset;
//!   the GUI shell owns the clock
//! - **Theme-Configurable** — all visual settings are configurable through the theme system
//! - **Multi-Caret Aware** — correctly assigns primary vs additional colours per caret
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_caret_selection::{
//!     CaretShape, CaretStyle, CaretWidth, BlinkState, CaretSelectionConfig,
//! };
//! use ff_edit_operations::EditMode;
//!
//! // Configure caret shape
//! let shape = CaretShape::new(CaretStyle::Line, CaretWidth::new(2));
//! assert_eq!(shape.effective_style(EditMode::Insert), CaretStyle::Line);
//! assert_eq!(shape.effective_style(EditMode::Overstrike), CaretStyle::Block);
//!
//! // Blink model
//! let mut blink = BlinkState::new(530);
//! blink.reset(0);
//! assert!(blink.is_visible(100));   // first half of cycle
//! assert!(!blink.is_visible(300));  // second half of cycle
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Caret shape and style types (Invisible, Line, Block).
pub mod caret_style;

/// Caret colour model (primary, additional, inverse text).
pub mod caret_colour;

/// RGBA colour type.
pub mod colour;

/// Caret blink model (period, visibility query, reset).
pub mod blink;

/// Caret line highlight configuration (None, Frame, Fill).
pub mod caret_line;

/// Selection display configuration (visibility, layer, EOL fill).
pub mod selection_display;

/// Selection element colours (Primary, Additional, Secondary, Inactive).
pub mod selection_colours;

/// Virtual space display logic.
pub mod virtual_space;

/// Rectangular selection display.
pub mod rectangular;

/// Multi-caret display coordination.
pub mod multi_caret;

/// Modified line marker rendering.
pub mod modified_marker;

/// Configuration aggregate and theme integration.
pub mod config;

/// Keyboard focus integration.
pub mod focus;

/// Error types.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use blink::BlinkState;
pub use caret_colour::CaretColours;
pub use caret_line::{CaretLineConfig, CaretLineLayer, CaretLineMode};
pub use caret_style::{CaretShape, CaretStyle, CaretWidth};
pub use colour::ColourRGBA;
pub use config::CaretSelectionConfig;
pub use error::CaretSelectionError;
pub use focus::FocusState;
pub use modified_marker::ModifiedMarkerConfig;
pub use multi_caret::{CaretRenderInfo, MultiCaretDisplay, SelectionRenderInfo};
pub use rectangular::RectangularSelectionDisplay;
pub use selection_colours::{SelectionColourSet, SelectionContext};
pub use selection_display::{SelectionDisplayConfig, SelectionLayer};
pub use virtual_space::{Rect, VirtualSpaceRenderer};
