//! # ff-whitespace-guides — Whitespace Visibility and Structural Guides
//!
//! This crate provides the data model, configuration, and per-line metadata
//! computation for visual annotations in FileForgeWorkbench:
//!
//! - **Whitespace visibility** — dots for spaces, arrows/strikeouts for tabs
//! - **Indent guides** — vertical guide lines at each indentation level
//! - **Edge column indicator** — vertical line(s) or background shading at column boundaries
//! - **Wrap markers** — visual indicators at start/end of wrapped sub-lines
//!
//! ## GUI Independence
//!
//! This crate is fully GUI-independent. It defines settings, enums, per-line
//! metadata queries, and toggle commands. Actual rendering is delegated to the
//! GUI shell (e.g., `ff-desktop`).
//!
//! ## Architecture
//!
//! ```text
//! Shell Layer (egui) → reads settings + per-line queries → draws glyphs
//! THIS CRATE          → settings model, query API, toggle commands
//! Upstream            → ff-config (settings), ff-command (toggle commands)
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Mode enums for whitespace visibility, tab drawing, indent guides, edge, and wrap.
pub mod modes;

/// Per-line query functions for whitespace glyphs, indent guides, edge, and wrap markers.
pub mod query;

/// Indent level computation and blank-line scanning utilities.
pub mod indent;

/// Aggregated settings struct and configuration integration.
pub mod settings;

/// Toggle command implementations.
pub mod commands;

/// Configuration key constants for the `editor.*` namespace.
pub mod keys;

/// Resolved colour cache types.
pub mod colours;

/// Data types for query results (glyph positions, guide columns, etc.).
pub mod types;

/// Error types for the whitespace-guides subsystem.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use modes::{
    EdgeMode, IndentGuideMode, TabDrawMode, WhitespaceVisibility, WrapIndentMode, WrapVisualFlag,
    WrapVisualLocation,
};

pub use types::{
    EdgeInfo, EdgeProperties, GlyphPosition, IndentGuideInfo, WhitespaceGlyph, WrapIndentInfo,
    WrapMarkerInfo,
};

pub use settings::WhitespaceSettings;

pub use colours::ResolvedColours;

pub use error::WhitespaceGuidesError;
