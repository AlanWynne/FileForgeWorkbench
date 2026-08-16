//! # ff-asa — ASA Carriage Control Interpretation and Print Preview
//!
//! This crate implements the ASA (ANSI) carriage control interpretation and
//! print preview subsystem for FileForgeWorkbench. It transforms mainframe
//! spool files into a visual representation simulating how the report would
//! have appeared on a line printer — with page breaks, line spacing, overprint
//! merging (bold/underline), and green-bar paper simulation.
//!
//! ## Design Principles
//!
//! - **GUI-independent** — all ASA parsing, merging, pagination, and export logic
//!   operates on the document model without GUI framework dependency
//! - **Command-framework integrated** — PREVIEW activation, export commands, and
//!   page navigation are registered with the command framework
//! - **Custom-viewer compliant** — registered as `Custom_Viewer` with Viewer_Key
//!   `"asa-report"` through the custom-file-viewers framework
//! - **Read-only display** — Preview_Mode is a rendering transformation only
//! - **Sequence-aware** — operates on post-strip content when sequence number
//!   stripping is active
//!
//! ## Architecture
//!
//! The crate is organized into:
//! 1. **Control** — ASA control character parsing and classification
//! 2. **Detection** — heuristic first-column analysis for ASA content
//! 3. **Merge** — overstrike line merging engine
//! 4. **Page Index** — page number to document line mapping
//! 5. **Preview** — GUI-independent preview element generation
//! 6. **Strip** — transparent ASA strip/restore for editing
//! 7. **Shading** — line band shading computation
//! 8. **Printer** — printer profile definitions
//! 9. **Navigation** — page navigation logic
//! 10. **Export** — plain text export
//! 11. **Panel** — print preview panel state
//! 12. **Config** — configuration and defaults

// ─── Public Modules ─────────────────────────────────────────────────────────

/// ASA carriage control character parsing and classification.
pub mod control;

/// Configuration for the ASA preview subsystem.
pub mod config;

/// ASA auto-detection engine.
pub mod detection;

/// Error types for the ASA report preview subsystem.
pub mod error;

/// Plain text export.
pub mod export_text;

/// Overstrike line merging engine.
pub mod merge;

/// Preview navigation — page location and viewport control.
pub mod navigation;

/// Page index — page number to document line mapping.
pub mod page_index;

/// Print preview panel state and logic.
pub mod panel;

/// Preview rendering — GUI-independent display model.
pub mod preview;

/// Printer profiles and page dimension configuration.
pub mod printer;

/// Line band shading computation.
pub mod shading;

/// ASA strip/restore engine.
pub mod strip;

/// Core data types and newtypes.
pub mod types;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use config::{AsaPreviewConfig, ExportPageSeparator};
pub use control::{AsaControl, ASA_VALID_CHARS};
pub use detection::{
    detect_asa, detect_by_recfm, is_asa_by_recfm, DetectionConfig, DetectionResult,
};
pub use error::AsaError;
pub use export_text::{count_page_separators, export_text, TextExportOptions};
pub use merge::{merge_overstrikes, CharStyle, MergeResult, MergedLine, StyledChar};
pub use navigation::{
    first_page, format_page_indicator, last_page, locate_page, next_page, previous_page,
};
pub use page_index::{PageEntry, PageIndex};
pub use panel::PreviewPanelState;
pub use preview::{build_preview, PreviewElement, PreviewState};
pub use printer::{PageOverflow, PrinterProfile};
pub use shading::{compute_band_groups, is_tinted_group};
pub use strip::{restore_asa, strip_asa, AsaControlMap};
pub use types::{PageDepth, PageNumber, PageWidth};

/// The viewer key used for registration and command dispatch.
pub const VIEWER_KEY: &str = "asa-report";
