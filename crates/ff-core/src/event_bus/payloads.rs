//! Locally-defined event payload value types.
//!
//! Defined in ff-core to avoid circular dependencies / layer violations;
//! re-exported from the parent module.

use std::collections::HashMap;

// === Locally-Defined Event Payload Types ====================================

/// Opaque document identifier. Defined here to avoid layer violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(pub u64);

/// Opaque operation identifier for progress tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId(pub u64);

/// Parameters passed to a command at dispatch time.
/// Defined locally in ff-core to avoid circular dependency with ff-command.
#[derive(Debug, Clone, Default)]
pub struct CommandParams(pub HashMap<String, ParamValue>);

/// A single parameter value within CommandParams.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    /// A string value.
    String(String),
    /// A 64-bit signed integer value.
    Integer(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// A nested map of string keys to parameter values.
    Map(HashMap<String, ParamValue>),
}

/// Outcome of a dispatched command, used in event payloads.
/// Simplified status type defined locally to avoid circular dependency with ff-command.
#[derive(Debug, Clone)]
pub struct CommandOutcome {
    /// Whether the command completed successfully.
    pub success: bool,
    /// Optional human-readable message describing the outcome.
    pub message: Option<String>,
}

/// Severity level for GUI notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSeverity {
    /// Informational message -- no action required.
    Info,
    /// Warning -- something unexpected but non-fatal occurred.
    Warning,
    /// Error -- an operation failed.
    Error,
}

/// Progress information for long-running operations.
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Human-readable label describing the operation in progress.
    pub label: String,
    /// Completion fraction in [0.0, 1.0], or `None` for indeterminate progress.
    pub fraction: Option<f32>,
    /// Whether the user can cancel this operation.
    pub cancellable: bool,
}
