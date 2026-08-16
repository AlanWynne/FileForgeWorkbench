//! Core data types for the JES subsystem.
//!
//! Defines Job, JobId, JobStatus, InitiatorId, InitiatorStatus, and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::ffjcl::FfjclDefinition;
pub use crate::log::JobLog;

// ─── JobId ──────────────────────────────────────────────────────────────────

/// A unique, monotonically increasing job identifier.
///
/// Never reused within the same workbench session.
/// Display format: `JOB00001`, `JOB00002`, etc.
///
/// Validates: Requirement 2 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JobId(pub u64);

impl JobId {
    /// Creates a new JobId from a raw numeric value.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw numeric value.
    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JOB{:05}", self.0)
    }
}

// ─── JobStatus ──────────────────────────────────────────────────────────────

/// The lifecycle state of a job.
///
/// Validates: Requirement 3 AC 10, Requirement 6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum JobStatus {
    /// Job is in the input queue awaiting dispatch.
    Queued,
    /// Job is held — not eligible for scheduling.
    Held,
    /// Job is currently executing on an initiator.
    Active,
    /// Job completed successfully.
    Completed,
    /// Job terminated abnormally.
    Failed,
    /// Job was cancelled by user before or during execution.
    Cancelled,
}

impl JobStatus {
    /// Returns true if this is a terminal state (no further transitions possible).
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Returns true if this job is eligible for scheduling.
    pub fn is_eligible(self) -> bool {
        self == Self::Queued
    }
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "QUEUED"),
            Self::Held => write!(f, "HELD"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Failed => write!(f, "FAILED"),
            Self::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

// ─── InitiatorId ────────────────────────────────────────────────────────────

/// Unique identifier for an initiator in the pool.
///
/// Validates: Requirement 4 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InitiatorId(pub u32);

impl InitiatorId {
    /// Creates a new InitiatorId.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw numeric value.
    pub fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for InitiatorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "INIT{:02}", self.0)
    }
}

// ─── InitiatorStatus ────────────────────────────────────────────────────────

/// The lifecycle state of an initiator worker.
///
/// Validates: Requirement 4 AC 3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InitiatorStatus {
    /// Initiator is idle and available for work.
    Idle,
    /// Initiator is starting up.
    Starting,
    /// Initiator is executing a job.
    Active,
    /// Initiator is draining — finishing current job but accepting no new work.
    Draining,
    /// Initiator is shutting down.
    Stopping,
    /// Initiator has been stopped (inactive).
    Stopped,
    /// Initiator encountered an unrecoverable error.
    Failed,
}

impl fmt::Display for InitiatorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "IDLE"),
            Self::Starting => write!(f, "STARTING"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Draining => write!(f, "DRAINING"),
            Self::Stopping => write!(f, "STOPPING"),
            Self::Stopped => write!(f, "STOPPED"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

// ─── Disposition ────────────────────────────────────────────────────────────

/// Dataset disposition for DD statements.
///
/// Validates: Requirement 11 AC 1–3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    /// Create a new dataset.
    New,
    /// Open an existing dataset exclusively.
    Old,
    /// Open an existing dataset for shared access.
    Shr,
    /// Open existing or create if not found.
    Mod,
}

impl Disposition {
    /// Parses a disposition string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "NEW" => Some(Self::New),
            "OLD" => Some(Self::Old),
            "SHR" => Some(Self::Shr),
            "MOD" => Some(Self::Mod),
            _ => None,
        }
    }
}

impl fmt::Display for Disposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New => write!(f, "NEW"),
            Self::Old => write!(f, "OLD"),
            Self::Shr => write!(f, "SHR"),
            Self::Mod => write!(f, "MOD"),
        }
    }
}

// ─── StepStatus ─────────────────────────────────────────────────────────────

/// The execution status of a single job step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Step has not yet started.
    Pending,
    /// Step is currently executing.
    Running,
    /// Step completed successfully.
    Completed,
    /// Step failed.
    Failed,
    /// Step was bypassed due to condition code check.
    Bypassed,
    /// Step was cancelled.
    Cancelled,
}

// ─── Job ────────────────────────────────────────────────────────────────────

/// A complete job record with all lifecycle metadata.
///
/// Validates: Requirements 2, 3, 5, 6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier.
    pub id: JobId,
    /// Job name from the FFJCL JOB statement.
    pub name: String,
    /// Current lifecycle status.
    pub status: JobStatus,
    /// Priority (higher = dispatched sooner). Default: 0.
    pub priority: u32,
    /// Owner/submitter identity.
    pub owner: String,
    /// Submission timestamp (UTC).
    pub submit_time: DateTime<Utc>,
    /// Start timestamp (set when ACTIVE).
    pub start_time: Option<DateTime<Utc>>,
    /// End timestamp (set on terminal status).
    pub end_time: Option<DateTime<Utc>>,
    /// Assigned initiator ID (set when ACTIVE).
    pub initiator_id: Option<InitiatorId>,
    /// Current step name (updated during execution).
    pub current_step: Option<String>,
    /// Final return code (set on COMPLETED).
    pub return_code: Option<i32>,
    /// Failure reason (set on FAILED).
    pub failure_reason: Option<String>,
    /// Failing step name (set on FAILED).
    pub failing_step: Option<String>,
    /// Cancellation requester (set on CANCELLED).
    pub cancelled_by: Option<String>,
    /// Cancellation timestamp.
    pub cancel_time: Option<DateTime<Utc>>,
    /// Source provider identifier.
    pub provider_id: String,
    /// The parsed FFJCL job definition.
    pub definition: FfjclDefinition,
}

impl Job {
    /// Creates a new job in QUEUED state.
    pub fn new(id: JobId, definition: FfjclDefinition, owner: &str) -> Self {
        let priority = definition.priority.unwrap_or(0);
        let name = definition.job_name.clone();
        Self {
            id,
            name,
            status: JobStatus::Queued,
            priority,
            owner: owner.to_string(),
            submit_time: Utc::now(),
            start_time: None,
            end_time: None,
            initiator_id: None,
            current_step: None,
            return_code: None,
            failure_reason: None,
            failing_step: None,
            cancelled_by: None,
            cancel_time: None,
            provider_id: "desktop".to_string(),
            definition,
        }
    }

    /// Returns true if the job is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    /// Returns true if the job is eligible for scheduling.
    pub fn is_eligible(&self) -> bool {
        self.status.is_eligible()
    }

    /// Calculates elapsed runtime from start_time to end_time (or now if active).
    pub fn elapsed(&self) -> Option<chrono::Duration> {
        let start = self.start_time?;
        let end = self.end_time.unwrap_or_else(Utc::now);
        Some(end - start)
    }
}

// ─── JobStatusUpdate ────────────────────────────────────────────────────────

/// A status transition update for a job record.
///
/// Validates: Requirements 3, 6, 10
#[derive(Debug, Clone)]
pub enum JobStatusUpdate {
    /// Job dispatched to an initiator.
    Dispatched {
        initiator_id: InitiatorId,
        start_time: DateTime<Utc>,
    },
    /// Job step progress update.
    StepProgress { step_name: String },
    /// Job completed successfully.
    Completed {
        end_time: DateTime<Utc>,
        return_code: i32,
    },
    /// Job failed.
    Failed {
        end_time: DateTime<Utc>,
        reason: String,
        failing_step: Option<String>,
    },
    /// Job cancelled.
    Cancelled {
        cancel_time: DateTime<Utc>,
        cancelled_by: String,
    },
    /// Job held.
    Held,
    /// Job released from hold.
    Released,
}

// ─── JobFilter ──────────────────────────────────────────────────────────────

/// Query predicates for filtering jobs in the queue/monitor.
///
/// Validates: Requirement 9 AC 4, 5
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Filter by owner/user.
    pub owner: Option<String>,
    /// Filter by job name (prefix match).
    pub name: Option<String>,
    /// Filter by job ID.
    pub id: Option<JobId>,
    /// Filter by status (multiple allowed).
    pub statuses: Option<Vec<JobStatus>>,
    /// Filter by provider.
    pub provider_id: Option<String>,
    /// Sort field.
    pub sort: Option<JobSortField>,
    /// Sort ascending (true) or descending (false).
    pub ascending: bool,
}

/// Fields by which jobs can be sorted in the monitor.
///
/// Validates: Requirement 3 AC 8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobSortField {
    Name,
    Id,
    Owner,
    SubmitTime,
    Priority,
    Status,
    ReturnCode,
}

// ─── JobEvent ───────────────────────────────────────────────────────────────

/// Events emitted when job state changes.
///
/// Validates: Requirement 12 AC 4, Requirement 15 AC 5
#[derive(Debug, Clone)]
pub struct JobEvent {
    /// The job that changed.
    pub job_id: JobId,
    /// The new status.
    pub new_status: JobStatus,
    /// Previous status.
    pub previous_status: Option<JobStatus>,
    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// Provider that owns this job.
    pub provider_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_id_display_formats_correctly() {
        // Validates: Requirement 2 AC 2
        assert_eq!(JobId::new(1).to_string(), "JOB00001");
        assert_eq!(JobId::new(42).to_string(), "JOB00042");
        assert_eq!(JobId::new(99999).to_string(), "JOB99999");
    }

    #[test]
    fn job_id_ordering_is_monotonic() {
        // Validates: Requirement 2 AC 2
        assert!(JobId::new(1) < JobId::new(2));
        assert!(JobId::new(100) < JobId::new(200));
    }

    #[test]
    fn job_status_terminal_states() {
        // Validates: Requirement 6 AC 1–3
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Held.is_terminal());
        assert!(!JobStatus::Active.is_terminal());
    }

    #[test]
    fn job_status_eligible_only_queued() {
        // Validates: Requirement 3 AC 4
        assert!(JobStatus::Queued.is_eligible());
        assert!(!JobStatus::Held.is_eligible());
        assert!(!JobStatus::Active.is_eligible());
        assert!(!JobStatus::Completed.is_eligible());
    }

    #[test]
    fn job_status_display() {
        assert_eq!(JobStatus::Queued.to_string(), "QUEUED");
        assert_eq!(JobStatus::Active.to_string(), "ACTIVE");
        assert_eq!(JobStatus::Completed.to_string(), "COMPLETED");
    }

    #[test]
    fn initiator_id_display_formats_correctly() {
        // Validates: Requirement 4 AC 2
        assert_eq!(InitiatorId::new(1).to_string(), "INIT01");
        assert_eq!(InitiatorId::new(10).to_string(), "INIT10");
    }

    #[test]
    fn disposition_from_str_case_insensitive() {
        assert_eq!(Disposition::parse("NEW"), Some(Disposition::New));
        assert_eq!(Disposition::parse("old"), Some(Disposition::Old));
        assert_eq!(Disposition::parse("SHR"), Some(Disposition::Shr));
        assert_eq!(Disposition::parse("mod"), Some(Disposition::Mod));
        assert_eq!(Disposition::parse("INVALID"), None);
    }
}
