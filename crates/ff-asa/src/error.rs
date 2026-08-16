//! Error types for the ASA report preview subsystem.

/// Errors originating from the `ff-asa` crate.
///
/// Formatted per project error message standards:
/// `[asa-preview] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AsaError {
    /// Page number is out of valid range.
    #[error("[asa-preview] navigate: page {page} not found — report has {total} pages")]
    PageNotFound {
        /// The requested page number.
        page: usize,
        /// The total number of pages in the report.
        total: usize,
    },

    /// Overprint line has no preceding base line to merge with.
    #[error("[asa-preview] merge: overprint at line {line} has no preceding base line")]
    OverprintNoBaseLine {
        /// The 0-based document line number.
        line: usize,
    },

    /// Unrecognised ASA control character encountered.
    #[error("[asa-preview] parse: unrecognised control character '{ch}' at line {line}")]
    UnrecognisedControl {
        /// The unrecognised character.
        ch: char,
        /// The 0-based document line number.
        line: usize,
    },

    /// Export I/O failure.
    #[error("[asa-preview] export: I/O error writing to {path}: {source}")]
    ExportIo {
        /// The target file path.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Export path is invalid or inaccessible.
    #[error("[asa-preview] export: invalid path {path}: {reason}")]
    InvalidExportPath {
        /// The target file path.
        path: String,
        /// Why the path is invalid.
        reason: String,
    },

    /// Configuration value is invalid.
    #[error("[asa-preview] config: invalid value for {key}: {reason}")]
    InvalidConfig {
        /// The configuration key.
        key: String,
        /// Why the value is invalid.
        reason: String,
    },

    /// Strip/restore operation failed due to inconsistent state.
    #[error("[asa-preview] {operation}: control map has {map_size} entries but document has {doc_lines} lines")]
    ControlMapMismatch {
        /// The operation being attempted (e.g. "restore").
        operation: String,
        /// Number of entries in the control map.
        map_size: usize,
        /// Number of lines in the document.
        doc_lines: usize,
    },
}
