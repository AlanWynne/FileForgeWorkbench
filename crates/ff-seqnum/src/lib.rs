//! # ff-seqnum — Sequence Number Detection, Stripping, and Numbering
//!
//! This crate implements the sequence number subsystem for FileForgeWorkbench.
//! It handles the detection, stripping, re-insertion, and display overlay of
//! legacy sequence numbers found in mainframe source files (COBOL, JCL, FORTRAN,
//! PL/I) where fixed column ranges carry punched-card-era sequence data that is
//! not part of the source logic.
//!
//! ## Design Principles
//!
//! - **GUI-independent** — all logic operates on the document model via traits
//! - **Stripping is the default** — auto-strip on open when language profile enables it
//! - **Re-insertion is explicit** — NUMBER command with confirmation
//! - **Command-framework integrated** — UNNUM, NUMBER, NUMBER SHOW are registered commands
//! - **BOUNDS-aware** — sequence operations never alter active BOUNDS state
//! - **Undo-safe** — UNNUM and NUMBER are single-transaction undoable operations
//!
//! ## Architecture
//!
//! The crate is organized into layers:
//! 1. **Detection** — heuristic sampling to find sequence numbers
//! 2. **Strip** — column clearing with side-table storage
//! 3. **Number** — sequence generation and column insertion
//! 4. **Display** — NUMBER SHOW overlay data model
//! 5. **Save** — restore/strip on save pipeline hook
//! 6. **Command** — command registration and argument dispatch

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Error types for the sequence numbers subsystem.
pub mod error;

/// Core types: ColumnRange, SequenceFormat, DetectionResult.
pub mod types;

/// Configuration: SeqNumConfig, LanguageOverride.
pub mod config;

/// Trait interfaces for upstream crate integration.
pub mod traits;

/// Sequence number detection engine.
pub mod detector;

/// Per-document state tracking and side-table.
pub mod state;

/// Strip engine — core column clearing logic.
pub mod strip;

/// Auto-strip on file open orchestration.
pub mod auto_strip;

/// NUMBER SHOW display mode.
pub mod number_show;

/// Sequence number generation engine.
pub mod number;

/// UNNUM command implementation.
pub mod unnum;

/// NUMBER command implementation.
pub mod number_cmd;

/// Built-in language profile constants.
pub mod profiles;

/// Configuration resolution with per-language overrides.
pub mod profile_config;

/// Save-time preservation and restoration.
pub mod save_handler;

/// Visual indicators and status bar integration.
pub mod indicators;

/// BOUNDS interaction enforcement.
pub mod bounds;

/// Undo/Redo integration.
pub mod undo_integration;

/// Command registration and dispatch integration.
pub mod commands;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use auto_strip::{auto_strip_on_open, AutoStripResult};
pub use commands::{
    register_commands, validate_mode, NUMBER_COMMAND_ID, NUMBER_SHOW_COMMAND_ID, UNNUM_COMMAND_ID,
};
pub use config::{LanguageOverride, SeqNumConfig};
pub use detector::{FullDetectionResult, SequenceDetector};
pub use error::SeqNumError;
pub use indicators::{
    format_indicator_text, get_indicator, should_highlight_columns, SeqNumIndicator,
};
pub use number::{
    apply_numbering, auto_number_line, generate_sequence, validate_number_params, NumberResult,
};
pub use number_cmd::{
    execute_number, get_confirmation_prompt, parse_number_args, NumberCommandResult, NumberVariant,
};
pub use number_show::{get_overlay_content, toggle_show_mode, OverlayEntry};
pub use profile_config::{resolve_config, ResolvedSequenceConfig};
pub use profiles::{CobolProfile, FortranProfile, JclProfile, NoSequenceProfile, PliProfile};
pub use save_handler::{
    apply_restoration_to_line, prepare_save_content, LineRestoration, SaveContentDecision,
};
pub use state::{AutoNumberState, SeqNumState, SeqNumStatusIndicator, SideTable, SideTableEntry};
pub use strip::{
    extract_columns, restore_from_side_table, strip_columns, strip_document, strip_line_range,
    StripResult,
};
pub use traits::{CommandRegistry, DocumentAccess, DocumentMutate, LanguageProfile, UndoRecorder};
pub use types::{ColumnRange, DetectedFormat, DetectionResult, SequenceFormat};
pub use undo_integration::{
    record_number_transaction, record_unnum_transaction, should_record_undo, ColumnChange,
};
pub use unnum::{execute_unnum, parse_unnum_args, UnnumResult, UnnumVariant};
