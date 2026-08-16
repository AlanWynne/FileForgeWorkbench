//! # ff-hex — Hexadecimal Display and Editing Subsystem
//!
//! This crate implements the **hex display mode** for FileForgeWorkbench.
//! It provides a complete hexadecimal viewing and editing subsystem: toggling
//! between text and hex modes, three-pane layout rendering (offset/hex/ASCII),
//! in-place byte editing, hex search integration, cursor synchronisation,
//! hex dump export, goto-offset navigation, and undo/redo participation.
//!
//! ## Design Principles
//!
//! - **GUI-independent** — no dependency on egui, winit, or any rendering
//!   framework. The hex model computes positions and formatted text; GUI shells
//!   render.
//! - **Command-driven** — `HEX ON`, `HEX OFF`, `HEX DUMP`, `GOTO X'...'`
//!   all integrate with the command framework.
//! - **Testable** — pure-function layout computation and property-based tests
//!   validate all invariants.
//!
//! ## Architecture
//!
//! ```text
//! HexModeController ← top-level orchestrator
//! ├── HexLayout      ← row geometry, pane widths, formatting
//! ├── HexCursor      ← position, pane, nibble, navigation
//! ├── HexEditState   ← byte modification engine
//! ├── HexSearchBridge ← FIND X'...' integration
//! ├── HexViewportAdapter ← row-based scrolling
//! ├── ModifiedByteTracker ← save-state diffing
//! ├── HexConfig      ← settings binding
//! └── HexSessionState ← per-file persistence
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_hex::{HexModeController, HexConfig, HexInput, ArrowDirection, VecByteReader, ByteReader};
//!
//! let mut ctrl = HexModeController::new(HexConfig::default());
//! let doc = VecByteReader::new(vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]);
//!
//! // Activate hex mode
//! ctrl.activate(0, doc.byte_length()).unwrap();
//!
//! // Navigate down one row
//! ctrl.handle_input(HexInput::Arrow(ArrowDirection::Down), &doc).unwrap();
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Core types: ByteOffset, NibblePosition, HexPane, HexMode, BytesPerRow, etc.
pub mod types;

/// Error types for the ff-hex crate.
pub mod error;

/// Hex view layout model: row geometry, pane widths, formatting.
pub mod layout;

/// Hex cursor: position, pane focus, nibble, and navigation.
pub mod cursor;

/// Hex editing engine: byte modification with validation.
pub mod editing;

/// Modified byte tracking: save-state diffing.
pub mod modified_tracker;

/// Hex search integration: FIND X'...' pattern validation and highlighting.
pub mod search;

/// Hex viewport adapter: row-based scrolling.
pub mod viewport_adapter;

/// Hex dump export: formatting and output.
pub mod dump;

/// Goto offset command: parsing and validation.
pub mod goto;

/// Hex display configuration.
pub mod config;

/// Hex mode session state persistence.
pub mod session;

/// Hex view model: pre-computed renderable data.
pub mod view_model;

/// Hex mode controller: top-level orchestrator.
pub mod controller;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use config::HexConfig;
pub use controller::{ByteReader, HexModeController, VecByteReader};
pub use cursor::HexCursor;
pub use dump::{HexDumpExporter, HexDumpRange, HexDumpTarget};
pub use editing::{HexEditAction, HexEditState};
pub use error::HexError;
pub use goto::{HexGotoHandler, ParsedOffset};
pub use layout::HexLayout;
pub use modified_tracker::ModifiedByteTracker;
pub use search::{HexMatchHighlight, HexSearchBridge};
pub use session::HexSessionState;
pub use types::{
    ArrowDirection, AutoActivateBinary, ByteOffset, BytesPerRow, HexDigitCase, HexInput, HexMode,
    HexPane, NibblePosition,
};
pub use view_model::{HexByteMetadata, HexCursorRenderState, HexRow, HexViewModel};
pub use viewport_adapter::HexViewportAdapter;
