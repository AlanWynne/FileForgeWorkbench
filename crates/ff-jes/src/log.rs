//! Job log types and assembly.
//!
//! Defines the structured log format for job execution records.

use chrono::{DateTime, Utc};

use crate::model::{Job, JobId};

/// Log entry classification.
///
/// Validates: Requirement 7 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Informational JES message.
    Info,
    /// Warning (non-fatal).
    Warning,
    /// Error (fatal to step or job).
    Error,
    /// Application output (SYSOUT).
    Output,
    /// Allocation/resolution message.
    Allocation,
}

/// A single log entry with timestamp and content.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp of the log line.
    pub timestamp: DateTime<Utc>,
    /// Log level/category.
    pub level: LogLevel,
    /// The log message text.
    pub message: String,
}

impl LogEntry {
    /// Creates a new log entry with the current timestamp.
    pub fn now(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message: message.into(),
        }
    }
}

/// Log for a single step's execution.
#[derive(Debug, Clone)]
pub struct StepLog {
    /// Step name.
    pub step_name: String,
    /// Standard output (SYSOUT).
    pub sysout: Vec<LogEntry>,
    /// Error output.
    pub syserr: Vec<LogEntry>,
    /// Step return code.
    pub return_code: Option<i32>,
    /// Step start time.
    pub start_time: Option<DateTime<Utc>>,
    /// Step end time.
    pub end_time: Option<DateTime<Utc>>,
}

/// Complete execution log for a job.
///
/// Validates: Requirement 7
pub struct JobLog {
    /// The job this log belongs to.
    pub job_id: JobId,
    /// JES-style scheduling messages.
    pub jes_messages: Vec<LogEntry>,
    /// Allocation messages (DSN resolution per DD).
    pub allocation_messages: Vec<LogEntry>,
    /// Per-step execution logs.
    pub step_logs: Vec<StepLog>,
    /// Final JES completion messages.
    pub completion_messages: Vec<LogEntry>,
}

impl JobLog {
    /// Creates a minimal job log from a job record.
    pub fn for_job(job: &Job) -> Self {
        let mut jes_messages = Vec::new();

        jes_messages.push(LogEntry::now(
            LogLevel::Info,
            format!("JES2 JOB LOG -- SYSTEM {} -- NODE LOCAL", job.name),
        ));
        jes_messages.push(LogEntry::now(
            LogLevel::Info,
            format!("{} SUBMITTED BY {}", job.id, job.owner),
        ));

        if let Some(start) = job.start_time {
            jes_messages.push(LogEntry {
                timestamp: start,
                level: LogLevel::Info,
                message: format!("{} STARTED", job.id),
            });
        }

        let completion_messages = if let Some(end) = job.end_time {
            vec![LogEntry {
                timestamp: end,
                level: LogLevel::Info,
                message: format!(
                    "{} ENDED - STATUS={} RC={}",
                    job.id,
                    job.status,
                    job.return_code
                        .map(|rc| rc.to_string())
                        .unwrap_or_else(|| "N/A".to_string())
                ),
            }]
        } else {
            vec![]
        };

        let step_logs = job
            .definition
            .steps
            .iter()
            .map(|s| StepLog {
                step_name: s.name.clone(),
                sysout: vec![],
                syserr: vec![],
                return_code: None,
                start_time: None,
                end_time: None,
            })
            .collect();

        Self {
            job_id: job.id,
            jes_messages,
            allocation_messages: vec![],
            step_logs,
            completion_messages,
        }
    }
}
