//! Error types for the IDCAMS emulator.
//!
//! Defines the primary error enum covering parse failures, execution failures,
//! rollback issues, and input problems.

/// Primary error type for the ff-idcams crate.
#[derive(Debug, thiserror::Error)]
pub enum IdcamsError {
    /// A parse error occurred while processing control statements.
    #[error("parse error at position {position}: {message} ({code})")]
    ParseError {
        /// The IDC message code (e.g., IDC0001E, IDC0002E).
        code: String,
        /// Human-readable description of the parse failure.
        message: String,
        /// Character position in the input where the error was detected.
        position: usize,
    },

    /// An execution error occurred while processing a command.
    #[error("execution error: {message} ({code})")]
    ExecutionError {
        /// The IDC message code for this execution failure.
        code: String,
        /// Human-readable description of the execution failure.
        message: String,
    },

    /// A rollback (compensating action) failed, leaving potential inconsistency.
    #[error("rollback failed: {message}")]
    RollbackFailed {
        /// Description of what rollback step failed.
        message: String,
    },

    /// A required downstream service is not available.
    #[error("service unavailable: {service_name}")]
    ServiceUnavailable {
        /// Name of the unavailable service.
        service_name: String,
    },

    /// An error occurred reading or processing input.
    #[error("input error: {message}")]
    InputError {
        /// Description of the input problem.
        message: String,
    },
}

/// Errors returned by the CatalogService trait.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CatalogError {
    /// The requested dataset was not found in the catalog.
    #[error("dataset not found: {0}")]
    NotFound(String),

    /// A dataset with this name already exists.
    #[error("duplicate dataset name: {0}")]
    DuplicateName(String),

    /// The entry type does not match the expected type.
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        /// The expected type.
        expected: String,
        /// The actual type found.
        found: String,
    },

    /// The requested attribute cannot be modified.
    #[error("attribute not modifiable: {0}")]
    AttributeNotModifiable(String),

    /// A general I/O or internal error.
    #[error("catalog error: {0}")]
    Internal(String),
}

/// Errors returned by the VsamService trait.
#[derive(Debug, Clone, thiserror::Error)]
pub enum VsamError {
    /// The specified dataset was not found.
    #[error("dataset not found: {0}")]
    NotFound(String),

    /// The dataset is not a VSAM dataset.
    #[error("not a VSAM dataset: {0}")]
    NotVsam(String),

    /// The target is not a valid alternate index.
    #[error("not a valid AIX: {0}")]
    NotAnAix(String),

    /// Duplicate key encountered during write.
    #[error("duplicate key: {0}")]
    DuplicateKey(String),

    /// Integrity check failed.
    #[error("integrity error: {0}")]
    IntegrityError(String),

    /// A general VSAM error.
    #[error("VSAM error: {0}")]
    Internal(String),
}

/// Errors returned by the AllocatorService trait.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AllocatorError {
    /// The requested DD name could not be resolved.
    #[error("DD not found: {0}")]
    DdNotFound(String),

    /// Access to the DD failed.
    #[error("DD access failure: {0}")]
    AccessFailure(String),
}

/// A convenient Result type alias for IdcamsError.
pub type Result<T> = std::result::Result<T, IdcamsError>;
