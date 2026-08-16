//! # ff-file-ops — File Operations for FileForgeWorkbench
//!
//! This crate implements all user-facing file operations: New, Open, Save,
//! Save As, Revert, and Recent Files. It also provides the underlying
//! persistence mechanisms — atomic rename-on-write, backup copies, read-only
//! detection, and unsaved-changes guards.
//!
//! ## Architecture
//!
//! - All file I/O goes through the VFS abstraction (`ff-vfs`) — no `std::fs` calls
//! - All operations are registered commands via `ff-command`
//! - GUI dialogs (File Picker, Unsaved Changes) are abstracted behind traits
//! - Large operations delegate to background I/O with progress reporting
//!
//! ## Key Components
//!
//! - [`SaveStrategy`] — Atomic, DeleteFirst, or Direct write strategies
//! - [`FileOpenOptions`] / [`FileSaveOptions`] — Operation configuration
//! - [`RecentFilesList`] — Bounded MRU list with persistence
//! - [`ReadOnlyStatus`] — Source-aware read-only detection
//! - [`UnsavedChangesAction`] — Dialog response enumeration
//! - [`FileOpsError`] — Unified error type for all operations
//! - [`commands::ids`] — Command ID constants for registration

// ─── Public Modules ─────────────────────────────────────────────────────────

pub mod backup;
pub mod commands;
pub mod config;
pub mod error;
pub mod guard;
pub mod new;
pub mod open;
pub mod options;
pub mod persistence;
pub mod read_only;
pub mod recent_files;
pub mod resource_uri;
pub mod revert;
pub mod save;
pub mod save_as;
pub mod save_strategy;
pub mod traits;
pub mod unsaved_guard;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use backup::{create_backup, BackupConfig, BackupLocation};
pub use commands::{
    aliases, all_command_metadata, events, file_menu_layout, ids, is_revert_enabled,
    is_save_enabled, shortcuts, FileCommandMetadata, MenuEntry,
};
pub use config::{defaults, keys};
pub use error::FileOpsError;
pub use guard::{check_unsaved_changes, determine_guard_action, GuardAction, GuardResult};
pub use new::{create_new_file, NewFileResult};
pub use open::{determine_read_only_status, is_duplicate_open, load_resource, OpenResult};
pub use options::{
    FileOpenOptions, FilePickerMode, FilePickerOptions, FileSaveOptions, SaveResult,
};
pub use persistence::{
    cleanup_temp_files, select_strategy, AtomicWriteStrategy, DeleteFirstStrategy,
    DirectWriteStrategy, PersistenceStrategy,
};
pub use read_only::{
    matches_read_only_pattern, read_only_indicator, toggle_read_only, ReadOnlyStatus,
};
pub use recent_files::RecentFilesList;
pub use resource_uri::{backup_uri_alongside, filename_from_uri, parent_path, temp_uri_for};
pub use revert::{is_revert_available, needs_revert_confirmation, reload_from_vfs, RevertResult};
pub use save::{check_external_modification, execute_save, should_save_async, SaveState};
pub use save_as::{execute_save_as, target_exists};
pub use save_strategy::SaveStrategy;
pub use traits::{DialogProvider, DocumentAccess, EventEmitter, TabManager, UntitledCounter};
pub use unsaved_guard::UnsavedChangesAction;
