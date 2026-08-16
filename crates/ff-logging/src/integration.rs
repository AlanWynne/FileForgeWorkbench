//! Integration patterns for platform-core subsystems.
//!
//! This module provides helper functions and documents the standard logging
//! patterns used by all platform-core subsystems when integrating with
//! `ff-logging`. Every subsystem follows a consistent error-reporting pattern
//! to produce a unified, searchable log stream.
//!
//! # Subsystem Error Logging Pattern (Requirement 9, AC 9.1)
//!
//! When any platform-core subsystem encounters a recoverable error, it SHALL
//! write a WARN-level or ERROR-level log record containing:
//! - The **subsystem name** (e.g., `"file_engine"`, `"command_executor"`)
//! - The **operation** that was attempted (e.g., `"open"`, `"execute"`)
//! - The **error description** (the Display output of the error value)
//!
//! Use [`log_subsystem_error`] for this standard pattern.
//!
//! # Integration Patterns by Subsystem
//!
//! ## Command Executor (Requirement 9, AC 9.4)
//!
//! The command executor logs at **TRACE** level for every command processed
//! through the command-framework:
//!
//! ```rust,no_run
//! use ff_logging::{log, LogLevel};
//!
//! // Before executing a command:
//! # let command_id = "cmd";
//! # let params = "p";
//! log(
//!     LogLevel::Trace,
//!     "command_executor",
//!     &format!("executing command '{}' with params: {}", command_id, params),
//! );
//! ```
//!
//! ## File Engine (Requirement 9, AC 9.3)
//!
//! The file engine logs at **DEBUG** level when opening or processing files
//! through the virtual-file-system layer:
//!
//! ```rust,no_run
//! use ff_logging::{log, LogLevel};
//!
//! // When opening a file:
//! # let resource_uri = "file.txt";
//! # let file_size = 0usize;
//! log(
//!     LogLevel::Debug,
//!     "file_engine",
//!     &format!("opening {} ({} bytes)", resource_uri, file_size),
//! );
//! ```
//!
//! ## Macro Engine (Requirement 9, AC 9.2)
//!
//! The macro engine uses multiple levels depending on the execution phase:
//!
//! - **DEBUG** before execution begins (script filename)
//! - **INFO** after successful execution (script filename + duration)
//! - **ERROR** on execution failure (script filename + error message)
//!
//! ```rust,no_run
//! use ff_logging::{log, log_subsystem_error, LogLevel};
//! use std::time::Instant;
//!
//! # let script_name = "script.lua";
//! # let duration_ms = 0u64;
//! # let error = std::io::Error::new(std::io::ErrorKind::Other, "err");
//! // Before execution:
//! log(LogLevel::Debug, "macro_engine", &format!("executing script '{}'", script_name));
//!
//! // After successful execution:
//! log(
//!     LogLevel::Info,
//!     "macro_engine",
//!     &format!("script '{}' completed in {}ms", script_name, duration_ms),
//! );
//!
//! // On failure:
//! log_subsystem_error(LogLevel::Error, "macro_engine", "execute_script", &error);
//! ```
//!
//! # Zero-Cost Level Filtering (Requirement 9, AC 9.5)
//!
//! All log calls perform an atomic level check before any string formatting
//! or allocation occurs. When the configured minimum level is above the
//! record's level, the call returns immediately with negligible overhead
//! (a single atomic load). This means TRACE-level calls in the command
//! executor have zero cost when the configured level is DEBUG or higher.

use crate::init::log;
use crate::level::LogLevel;

/// Log a subsystem error using the standard platform-core pattern.
///
/// This is the canonical way for platform-core subsystems to report
/// recoverable errors. The formatted record includes the subsystem name
/// as the module path, and combines the operation and error description
/// into a structured message.
///
/// # Arguments
///
/// * `level` - The severity level (typically `LogLevel::Warn` or `LogLevel::Error`)
/// * `subsystem` - The subsystem name (e.g., `"file_engine"`, `"macro_engine"`)
/// * `operation` - The operation that was attempted (e.g., `"open"`, `"execute"`)
/// * `error` - The error value (anything implementing `Display`)
///
/// # Example
///
/// ```rust,no_run
/// use ff_logging::{log_subsystem_error, LogLevel};
///
/// // In the file engine, when opening a file fails:
/// log_subsystem_error(
///     LogLevel::Error,
///     "file_engine",
///     "open",
///     &std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"),
/// );
/// // Produces: "2025-01-20T14:30:22.123+00:00 ERROR [file_engine] open: access denied\n"
/// ```
///
/// # Requirement Coverage
///
/// Implements the error logging pattern from Requirement 9, AC 9.1.
pub fn log_subsystem_error(
    level: LogLevel,
    subsystem: &str,
    operation: &str,
    error: &dyn std::fmt::Display,
) {
    log(level, subsystem, &format!("{operation}: {error}"));
}
