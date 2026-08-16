//! Error types for the undo-redo-transactions crate.
//!
//! All errors follow the `[undo] operation: description` format per project error standards.

/// Errors produced by the undo-redo-transactions crate.
///
/// # Variants
///
/// Each variant maps to a specific failure mode documented in the requirements.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UndoError {
    /// Undo stack is empty — nothing to undo.
    #[error("[undo] undo: nothing to undo")]
    NothingToUndo,

    /// Redo stack is empty — nothing to redo.
    #[error("[undo] redo: nothing to redo")]
    NothingToRedo,

    /// Undo is disabled (max_levels == 0).
    #[error("[undo] operation: undo is disabled (max_levels=0)")]
    UndoDisabled,

    /// Operation not available in current mode (Browse/View).
    #[error("[undo] {operation}: not available in {mode} mode")]
    NotAvailableInMode {
        /// The operation that was attempted.
        operation: String,
        /// The mode that prevented the operation.
        mode: String,
    },

    /// Transaction rollback failed.
    #[error("[undo] rollback: failed to reverse operation at position {position}")]
    RollbackFailed {
        /// The byte position where the rollback failed.
        position: u64,
    },

    /// No active transaction to end or abort.
    #[error("[undo] end_transaction: no transaction in progress")]
    NoActiveTransaction,

    /// Recovery file I/O error.
    #[error("[undo] recovery: {operation} failed — {source}")]
    RecoveryIo {
        /// The I/O operation that failed.
        operation: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// Recovery file is corrupted or incompatible.
    #[error("[undo] recovery: file is corrupted or incompatible")]
    RecoveryCorrupted,

    /// History validation failed.
    #[error(
        "[undo] validate: history inconsistent with document length {expected}, computed {actual}"
    )]
    ValidationFailed {
        /// The expected document length.
        expected: u64,
        /// The computed document length from history replay.
        actual: u64,
    },

    /// Tentative mode is not active when an operation requires it.
    #[error("[undo] tentative: no tentative mode active")]
    TentativeNotActive,

    /// Tentative mode is already active.
    #[error("[undo] tentative: tentative mode already active")]
    TentativeAlreadyActive,

    /// No active bulk transaction.
    #[error("[undo] bulk: no bulk transaction in progress")]
    NoBulkTransaction,

    /// Bulk transaction already in progress.
    #[error("[undo] bulk: bulk transaction already in progress")]
    BulkAlreadyActive,

    /// Document not registered.
    #[error("[undo] routing: document '{document_id}' not registered")]
    DocumentNotRegistered {
        /// The document ID that was not found.
        document_id: String,
    },

    /// No active document set for routing.
    #[error("[undo] routing: no active document set")]
    NoActiveDocument,

    /// Serialization/deserialization error.
    #[error("[undo] serialization: {0}")]
    Serialization(String),
}
