//! # ff-session — Startup Sequence and Session Management
//!
//! This crate orchestrates the **application startup sequence**, **session state
//! persistence and restoration**, **command-line argument handling**, **exit sequence**,
//! and **crash recovery** for the FileForgeWorkbench platform.
//!
//! ## Responsibilities
//!
//! - Define and execute the 10-phase Startup_Sequence from process launch to interactive UI
//! - Orchestrate configuration loading, plugin initialisation, layout restoration, and file
//!   opening in correct dependency order
//! - Persist and restore complete Session_State: open tabs, per-tab state, window geometry,
//!   panel layout, recent files
//! - Process command-line arguments with proper precedence over session restore
//! - Execute a safe Exit_Sequence: unsaved-change prompts, session save, plugin shutdown
//! - Detect abnormal termination and offer crash recovery from Recovery_Files
//! - Guarantee graceful degradation — no single corrupt or missing file prevents startup
//!
//! ## Architecture Position
//!
//! ```text
//! Wave 8 — File I/O and Session
//!
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Shell Layer: ff-desktop (egui)                   │
//! │   (renders first frame when Phase 8 signals ready)           │
//! ├─────────────────────────────────────────────────────────────┤
//! │  THIS CRATE: ff-session ← Wave 8                             │
//! │  (startup sequence, session persistence, CLI, exit, recovery)│
//! ├─────────────────────────────────────────────────────────────┤
//! │  ff-config (settings)   │  ff-plugin (lifecycle)             │
//! │  ff-layout (panels)     │  ff-tabs (tab collection)          │
//! │  ff-file-ops (open)     │  ff-undo-redo (recovery files)     │
//! │  ff-logging (diagnostics)│  ff-core (event bus, platform)    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Startup sequence orchestration — 10-phase ordered flow.
pub mod startup;

/// Session state data model and schema versioning.
pub mod session_state;

/// Session file TOML serialisation and persistence.
pub mod session_file;

/// Command-line argument parsing and validation.
pub mod cli;

/// User Data Directory initialisation and management.
pub mod user_data_dir;

/// Recent Files list with bounded storage and deduplication.
pub mod recent_files;

/// Window geometry persistence and display validation.
pub mod window_geometry;

/// Exit sequence orchestration.
pub mod exit_sequence;

/// Session restore decision logic.
pub mod session_restore;

/// Crash recovery detection and restore flow.
pub mod crash_recovery;

/// Graceful degradation tracking.
pub mod degraded_mode;

/// Session configuration key definitions.
pub mod config_keys;

/// Workspace model -- WorkspaceState, load/save for `.ffwb-workspace` files.
pub mod workspace;

/// Error types for the ff-session crate.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use cli::CliArgs;
pub use config_keys::SessionConfig;
pub use crash_recovery::{RecoverableDocument, RecoveryAction, RecoveryResult};
pub use degraded_mode::{DegradedModeTracker, DegradedSubsystem, Subsystem};
pub use error::SessionError;
pub use exit_sequence::{DirtyDocument, ExitAction, ExitDecision, ShutdownStep};
pub use recent_files::RecentFilesList;
pub use session_file::SessionFile;
pub use session_restore::{determine_restore_mode, FileOpenTargets, RestoreMode};
pub use session_state::{
    LayoutSnapshot, RecentFileEntry, SelectionRange, SessionState, TabState, WindowGeometryState,
    CURRENT_SCHEMA_VERSION,
};
pub use startup::{
    execute_startup_sequence, PhaseOutcome, PhaseResult, StartupPhase, StartupResult,
};
pub use user_data_dir::UserDataDir;
pub use window_geometry::{clamp_to_display, is_visible_on, restore_geometry, DisplayBounds};
pub use workspace::{load_workspace, save_workspace, WorkspaceRecentFile, WorkspaceState};
