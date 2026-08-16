//! # ff-connector-local-fs — Local Filesystem VFS Provider
//!
//! This crate is the **primary VFS provider** for FileForgeWorkbench. It implements
//! the `VfsProvider` trait from `ff-vfs` to provide full read/write/create/delete/
//! rename/list/stat/watch access to the host operating system's native filesystem.
//!
//! ## Features
//!
//! - Registers under URI scheme `"local"` (e.g., `vfs://local/home/user/file.txt`)
//! - Cross-platform path handling (Windows drive letters, UNC, Unix paths)
//! - OS-native file watching via the `notify` crate (inotify, ReadDirectoryChangesW, FSEvents)
//! - Path resolution: tilde expansion, environment variable substitution, canonicalization
//! - Large file streaming and memory-mapped I/O
//! - Unified error mapping from OS-specific errors to `VfsError`
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │  ff-vfs — ProviderRegistry, routing             │
//! ├─────────────────────────────────────────────────┤
//! │  ff-connector-local-fs (this crate)             │
//! │  LocalFsProvider → PathResolver, FileWatcher,   │
//! │                     StreamingManager, ErrorMapper│
//! ├─────────────────────────────────────────────────┤
//! │  OS filesystem (Tokio async I/O, notify)        │
//! └─────────────────────────────────────────────────┘
//! ```

pub mod config;
pub mod error;
pub mod metadata;
pub mod path;
pub mod provider;
pub mod streaming;
pub mod watcher;

// Public API re-exports
pub use config::LocalFsConfig;
pub use error::map_io_error;
pub use metadata::{FileMetadata, FilePermissions, ResourceType};
pub use path::{NativePath, PathResolver};
pub use provider::LocalFsProvider;
pub use streaming::{AtomicWriter, ChunkedReader, StreamingManager};
pub use watcher::{FileWatcher, WatchId};
