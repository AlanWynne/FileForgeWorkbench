//! Error types for the ff-select crate.
//!
//! All errors follow the `[record-criteria] operation: description` format
//! per Error Message Standards.

/// Errors originating from the ff-select crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CriteriaError {
    /// A referenced field does not exist in the active Record_Structure.
    #[error("[record-criteria] evaluate: field '{field}' not found in active structure")]
    FieldNotFound {
        /// The field name that was not found.
        field: String,
    },

    /// The regex pattern in a MATCHES_REGEX criterion is invalid.
    #[error(
        "[record-criteria] evaluate: invalid regex pattern '{pattern}' in row {row}: {detail}"
    )]
    InvalidRegex {
        /// Row index where the invalid regex was found.
        row: usize,
        /// The invalid pattern string.
        pattern: String,
        /// Description of the regex error.
        detail: String,
    },

    /// A criterion value cannot be parsed as the expected numeric type.
    #[error("[record-criteria] evaluate: cannot parse '{value}' as numeric for field '{field}'")]
    NumericParseFailed {
        /// The field name.
        field: String,
        /// The value that failed to parse.
        value: String,
    },

    /// Group open/close structure is invalid.
    #[error("[record-criteria] validate: unmatched group at row {row} — {detail}")]
    UnmatchedGroup {
        /// Row index where the group mismatch was detected.
        row: usize,
        /// Description of the mismatch.
        detail: String,
    },

    /// A named CriteriaSet was not found in the catalog.
    #[error("[record-criteria] load: criteria set '{name}' not found in {location}")]
    CriteriaNotFound {
        /// The criteria set name that was not found.
        name: String,
        /// The location that was searched.
        location: String,
    },

    /// The .criteria.json file could not be parsed.
    #[error("[record-criteria] load: failed to parse '{path}' — {detail}")]
    ParseFailed {
        /// Path to the file that failed to parse.
        path: String,
        /// Description of the parse error.
        detail: String,
    },

    /// I/O error accessing the criteria catalog.
    #[error("[record-criteria] io: {operation} failed for '{path}' — {detail}")]
    Io {
        /// The operation that failed (e.g., "read", "write", "create_dir").
        operation: String,
        /// Path involved in the operation.
        path: String,
        /// Description of the underlying I/O error.
        detail: String,
    },

    /// The Criteria_Store configuration file is corrupt.
    #[error("[record-criteria] store: criteria store at '{path}' is corrupt — {detail}")]
    StoreCorrupt {
        /// Path to the corrupt store file.
        path: String,
        /// Description of the corruption.
        detail: String,
    },

    /// Invalid CRITERIA command argument.
    #[error(
        "[record-criteria] command: invalid argument '{arg}' — expected SET, CLEAR, SHOW, or SAVE"
    )]
    InvalidCommandArg {
        /// The invalid argument string.
        arg: String,
    },

    /// FileForge_Mode is not active (criteria require structured records).
    #[error("[record-criteria] command: FileForge_Mode is not active — criteria require a structure definition")]
    FileForgeNotActive,

    /// Configuration key has invalid value.
    #[error("[record-criteria] config: key '{key}' has invalid value '{value}' — using default")]
    InvalidConfig {
        /// The configuration key.
        key: String,
        /// The invalid value.
        value: String,
    },

    /// Maximum criteria rows exceeded.
    #[error("[record-criteria] validate: criteria set has {count} rows, maximum is {max}")]
    MaxRowsExceeded {
        /// The actual count.
        count: usize,
        /// The maximum allowed.
        max: usize,
    },

    /// Name collision when saving.
    #[error(
        "[record-criteria] save: a criteria set named '{name}' already exists — use overwrite"
    )]
    NameCollision {
        /// The colliding name.
        name: String,
    },
}
