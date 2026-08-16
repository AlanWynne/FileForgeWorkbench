//! # ff-logging — Structured Logging Subsystem for FileForgeWorkbench
//!
//! This crate provides the foundational logging subsystem for the entire
//! FileForgeWorkbench workspace. Every other crate (platform-core,
//! command-framework, plugin-architecture, workflow-engine, document-model,
//! and all plugins) depends on `ff-logging` for diagnostic output.
//!
//! ## Design Principles
//!
//! - **File-based output only** — no console dependency; all records go to persistent log files
//! - **Thread-safe** — safe to call from any thread (including Tokio workers) without blocking
//! - **Graceful degradation** — falls back to a no-op sink if file I/O is unavailable
//! - **Configurable** — log levels, file paths, rotation, and retention are user-adjustable
//! - **Plugin-accessible** — plugins receive a logging handle via `PluginContext`
//!
//! ## GUI-Independent Process Model (Requirement 7)
//!
//! `ff-logging` is designed for GUI-independent operation. It never writes to
//! stdout or stderr, never allocates a console window, and never spawns child
//! processes. All diagnostic output is routed exclusively to the log file.
//!
//! ### `#![windows_subsystem = "windows"]` Attribute
//!
//! The `#![windows_subsystem = "windows"]` attribute **belongs on the desktop
//! binary crate** (`fileforge-desktop`), NOT on `ff-logging`. This is because:
//!
//! - `ff-logging` is a **library crate** — the `windows_subsystem` attribute is
//!   only meaningful on binary crates (it controls how the OS loader treats the
//!   executable).
//! - The attribute suppresses automatic console window allocation on Windows
//!   when the application is launched from a shortcut, Start Menu entry, or
//!   file association (Requirement 7, AC 7.1).
//! - The binary crate must include `#![windows_subsystem = "windows"]` at the
//!   top of its `main.rs` to ensure no console window appears.
//!
//! `ff-logging` upholds its part of the contract by:
//! - Writing all output exclusively to the log file (AC 7.4)
//! - Never producing output on stdout or stderr (AC 7.5)
//! - Never calling `AllocConsole` or spawning child processes (AC 7.6)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ff_logging::{init_default, shutdown, LogLevel, log};
//!
//! let status = init_default();
//! log(LogLevel::Info, "my_module", "Application started");
//! shutdown();
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Log level enum, ordering, and parsing.
pub mod level;

/// Log record data type with formatting, truncation, and escaping.
pub mod record;

/// Log record formatting and parsing utilities.
pub mod format;

/// Configuration types, TOML deserialization, defaults, and validation.
pub mod config;

/// Error types for the logging subsystem.
pub mod error;

/// `PluginLogHandle` trait and concrete implementation for plugin integration.
pub mod plugin_handle;

/// Log macros: `log_trace!`, `log_debug!`, `log_info!`, `log_warn!`, `log_error!`.
#[macro_use]
pub mod macros;

/// Integration patterns and helpers for platform-core subsystems.
pub mod integration;

// ─── Internal Modules (pub(crate)) ──────────────────────────────────────────

/// Bounded MPSC channel, overflow handling, and drop counter.
pub(crate) mod channel;

/// Writer thread, buffered I/O, and flush strategies.
pub(crate) mod writer;

/// File rotation logic, naming, and size tracking.
pub(crate) mod rotation;

/// `FileSink` and `NoopSink` implementations.
pub(crate) mod sink;

/// Initialization sequence, directory creation, and fallback logic.
pub(crate) mod init;

/// Graceful shutdown, flush timeout, and shutdown signaling.
pub(crate) mod shutdown;

// ─── Public API Re-exports ──────────────────────────────────────────────────

/// The severity level for log records.
pub use level::LogLevel;

/// Error returned when parsing an unrecognized log level string.
pub use level::ParseLogLevelError;

/// A single structured log entry.
pub use record::LogRecord;

/// Configuration for the logging subsystem.
pub use config::LogConfig;

/// Error types that can occur within the logging subsystem.
pub use error::LoggingError;

/// Status returned by initialization functions.
pub use init::LoggingStatus;

/// Trait for plugin logging handles.
pub use plugin_handle::PluginLogHandle;

// ─── Public API Functions (re-exported from internal modules) ────────────────

pub use init::{
    current_level, dropped_count, init, init_default, install_panic_hook, is_fallback,
    is_logging_available, log, log_lazy,
};

pub use shutdown::shutdown;

pub use plugin_handle::create_plugin_handle;

pub use integration::log_subsystem_error;
