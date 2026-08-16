//! # ff-external-mod — External File Modification Detection
//!
//! This crate detects when files open in the workbench are modified, renamed, or
//! deleted by external processes (other editors, build tools, version control, shell
//! scripts). It subscribes to VFS file-watcher events, tracks per-document modification
//! times, and coordinates reload/notification decisions with configurable policies.
//!
//! ## Architecture
//!
//! The crate operates at Wave 8 (File I/O and Session) and depends on:
//! - `ff-vfs` for all filesystem interaction (FFW-ARCH-001)
//! - `ff-document-model` for document state queries
//! - `ff-config` for configuration access
//! - `ff-command` for command framework integration
//! - `ff-logging` for structured diagnostics
//!
//! ## Key Components
//!
//! - [`ExternalModificationDetector`](detector) — central coordinator for detection logic
//! - [`MtimeTracker`](mtime_tracker) — per-document mtime snapshot management
//! - [`ExternalChange`](change_event::ExternalChange) — change event representation
//! - [`ReloadPolicy`](reload_policy::ReloadPolicy) — configurable reload strategy
//! - [`BatchCoalescer`](batch_coalescer) — debounce window grouping
//! - [`FocusGainedChecker`](focus_check) — focus-gained mtime revalidation
//! - [`ExternalModConfig`](config::ExternalModConfig) — typed configuration
//! - [`ExternalModError`](error::ExternalModError) — unified error type
//!
//! ## Design Constraints
//!
//! - **FFW-ARCH-001**: All filesystem interaction flows through `ff-vfs` — no `std::fs`
//!   or `tokio::fs` calls for watching or stat operations.
//! - **GUI Independence**: Core detection logic is GUI-independent; user prompts are
//!   abstracted behind the [`ExternalModDialogProvider`](prompt::ExternalModDialogProvider) trait.

// ─── Public Modules ─────────────────────────────────────────────────────────

pub mod batch_coalescer;
pub mod change_event;
pub mod config;
pub mod detector;
pub mod error;
pub mod focus_check;
pub mod mtime_tracker;
pub mod prompt;
pub mod reload_policy;
pub mod types;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use change_event::{ChangeType, ExternalChange};
pub use config::ExternalModConfig;
pub use error::ExternalModError;
pub use prompt::{
    BatchAction, BatchNotification, ExternalModDialogProvider, PromptAction, PromptOptions,
    PromptResponse,
};
pub use reload_policy::{PolicyAction, ReloadPolicy, ReloadPolicyEngine};
pub use types::{DocumentId, MtimeComparison, MtimeSnapshot};
