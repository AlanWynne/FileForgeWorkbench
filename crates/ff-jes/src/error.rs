//! Error types for the ff-jes crate.

use crate::model::{InitiatorId, JobId, JobStatus};

/// All errors produced by the ff-jes crate.
///
/// Follows `[jes] operation: description` format per design spec.
#[derive(Debug, thiserror::Error)]
pub enum JesError {
    /// Job submission failed (parse error, validation error, queue full).
    #[error("[jes] submission failed: {0}")]
    SubmissionFailed(String),

    /// FFJCL validation error (syntax, missing fields, unresolvable DSN).
    #[error("[jes] validation error at line {line}: {message}")]
    ValidationError { line: usize, message: String },

    /// Scheduler error (dispatch failure, no eligible jobs).
    #[error("[jes] scheduler error: {0}")]
    SchedulerError(String),

    /// Initiator failure (unrecoverable worker error).
    #[error("[jes] initiator {id} failed: {reason}")]
    InitiatorFailed { id: InitiatorId, reason: String },

    /// Dataset catalog resolution failed during job execution.
    #[error("[jes] catalog resolution failed for DSN '{dsn}': {reason}")]
    CatalogResolutionFailed { dsn: String, reason: String },

    /// Purge operation failed.
    #[error("[jes] purge failed for job {job_id}: {reason}")]
    PurgeError { job_id: JobId, reason: String },

    /// Provider is unavailable or returned an error.
    #[error("[jes] provider '{provider}' unavailable: {reason}")]
    ProviderUnavailable { provider: String, reason: String },

    /// Job not found in queue.
    #[error("[jes] job {0} not found")]
    JobNotFound(JobId),

    /// Invalid state transition (e.g., hold on active job).
    #[error(
        "[jes] invalid state transition for job {job_id}: cannot {action} from {current_status}"
    )]
    InvalidJobState {
        job_id: JobId,
        action: String,
        current_status: JobStatus,
    },

    /// Cancellation timed out — force-kill was required.
    #[error("[jes] cancellation timeout for job {0}")]
    CancellationTimeout(JobId),

    /// FFJCL parse error.
    #[error("[jes] FFJCL parse error at line {line}: {message}")]
    FfjclParseError { line: usize, message: String },

    /// Log access error.
    #[error("[jes] log access error for job {job_id}: {reason}")]
    LogAccessError { job_id: JobId, reason: String },

    /// Queue persistence error.
    #[error("[jes] queue persistence error: {0}")]
    QueuePersistenceError(String),

    /// Configuration error.
    #[error("[jes] configuration error: {0}")]
    ConfigError(String),

    /// I/O error (spool, persistence).
    #[error("[jes] I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal error (unexpected state).
    #[error("[jes] internal error: {0}")]
    Internal(String),
}
