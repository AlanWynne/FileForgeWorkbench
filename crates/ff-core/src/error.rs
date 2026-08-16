//! # Error — Unified Error Type for ff-core
//!
//! This module defines `CoreError`, the unified error type for the `ff-core` crate.
//! All fallible operations within the crate return `Result<T, CoreError>`.
//!
//! Error messages follow the project-wide format convention:
//! `[core] operation: description`

/// Unified error type for the `ff-core` crate.
///
/// All error variants carry enough context to diagnose the problem without
/// requiring additional investigation. The `#[non_exhaustive]` attribute
/// allows future variants to be added without breaking downstream code.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CoreError {
    /// A critical subsystem failed to initialize during startup.
    #[error("[core] startup: critical subsystem '{name}' failed to initialize: {reason}")]
    CriticalSubsystemFailure {
        /// Name of the subsystem that failed.
        name: String,
        /// Reason for the failure.
        reason: String,
    },

    /// A non-critical subsystem failed to initialize during startup.
    /// The application continues with reduced functionality.
    #[error("[core] startup: non-critical subsystem '{name}' failed: {reason}")]
    NonCriticalSubsystemFailure {
        /// Name of the subsystem that failed.
        name: String,
        /// Reason for the failure.
        reason: String,
    },

    /// A service type was registered more than once.
    #[error("[core] registry: service type '{type_name}' is already registered")]
    DuplicateServiceRegistration {
        /// The type name of the duplicate service.
        type_name: String,
    },

    /// A registration was attempted after the registry was frozen.
    #[error("[core] registry: cannot register '{type_name}' — registry is frozen")]
    RegistryFrozen {
        /// The type name that was rejected.
        type_name: String,
    },

    /// A subsystem exceeded its shutdown grace period.
    #[error("[core] shutdown: subsystem '{name}' exceeded {grace_seconds}s grace period")]
    ShutdownTimeout {
        /// Name of the subsystem that timed out.
        name: String,
        /// The grace period in seconds that was exceeded.
        grace_seconds: u64,
    },

    /// A plugin hot-restart failed. The plugin is left in an unloaded state.
    #[error("[core] hot-restart: plugin '{plugin_name}' failed to reload: {reason}")]
    HotRestartFailed {
        /// The name of the plugin that failed to hot-restart.
        plugin_name: String,
        /// The reason for the failure.
        reason: String,
    },

    /// Tokio runtime failed to initialize.
    #[error("[core] runtime: failed to create Tokio runtime: {reason}")]
    RuntimeCreationFailed {
        /// The reason the runtime could not be created.
        reason: String,
    },

    /// Tokio runtime encountered a fatal error (e.g., all worker threads panicked).
    #[error("[core] runtime: fatal runtime error — all worker threads panicked")]
    RuntimeFatal,

    /// An I/O error occurred.
    #[error("[core] io: {0}")]
    Io(#[from] std::io::Error),
}
