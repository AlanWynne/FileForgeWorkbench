//! # ff-vfs — Virtual File System Abstraction Layer
//!
//! This crate implements the overriding architectural principle **FFW-ARCH-001**: all content
//! access throughout the FileForgeWorkbench platform flows through this single abstraction layer.
//! No consuming crate ever calls `std::fs` or `tokio::fs` directly.
//!
//! ## Key Components
//!
//! - [`VfsError`] — unified error type abstracting provider-specific errors
//! - `ResourceUri` — unified resource identifier in the format `vfs://provider/path`
//! - `VfsProvider` — trait that all storage backends implement
//! - `ProviderRegistry` — thread-safe registry of provider instances keyed by scheme
//! - `Vfs` — top-level facade providing async file operations
//! - Watch and search abstractions for provider-agnostic eventing and content search

pub mod error;
pub mod provider;
pub mod search;
pub mod types;
pub mod uri;
pub mod watch;

pub mod registry;

pub mod vfs;

pub mod subsystem;

pub use error::VfsError;
pub use provider::{VfsFile, VfsProvider};
pub use registry::ProviderRegistry;
pub use search::{fallback_search, SearchOptions, SearchQuery, VfsSearchResult};
pub use subsystem::VfsSubsystem;
pub use types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsEntryType,
    VfsMetadata, WatchOptions, WriteMode,
};
pub use uri::ResourceUri;
pub use vfs::Vfs;
pub use watch::{WatchEvent, WatchHandle};
