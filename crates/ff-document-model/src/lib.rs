//! # ff-document-model — Foundational Text Storage Layer
//!
//! This crate provides the core document model for FileForgeWorkbench:
//!
//! - **Gap buffer** text storage with O(1) amortized editing
//! - **Line index** with O(log n) bidirectional lookups
//! - **Streaming file loading** via the VFS with progressive availability
//! - **Encoding-aware character navigation** (UTF-8, CRLF atomic)
//! - **Document lifecycle** with shared ownership via `Arc<RwLock<Document>>`
//! - **Watcher notifications** for incremental view updates
//! - **Viewport management** with clamped scroll arithmetic
//! - **Save-point tracking** for modification state
//!
//! ## Architecture
//!
//! The document model sits in Wave 4 (Core Editor) and depends on:
//! - `ff-vfs` for all file access (FFW-ARCH-001)
//! - `ff-logging` for structured diagnostic output
//!
//! It is consumed by higher-level crates: `ff-edit-operations`,
//! `ff-undo-redo-transactions`, `ff-display-line-mapping`, and others.

// ─── Public Modules ─────────────────────────────────────────────────────────

pub mod command;
pub mod document;
pub mod encoding_nav;
pub mod error;
pub mod gap_buffer;
pub mod handle;
pub mod line_end;
pub mod line_index;
pub mod save_point;
pub mod sparse_line_index;
pub mod streaming;
pub mod text_buffer;
pub mod types;
pub mod viewport;
pub mod watcher;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use command::{CommandResult, DeleteCommand, DocumentCommand, InsertCommand};
pub use document::Document;
pub use error::DocumentError;
pub use handle::{new_document, new_document_with_capacity, wrap_document, DocumentHandle};
pub use line_end::LineEndMode;
pub use types::{
    BytePosition, CharacterExtracted, DeleteResult, Direction, InsertResult, LineNumber,
    LoadingProgress, SplitView,
};
pub use viewport::Viewport;
pub use watcher::{DocumentWatcher, WatcherHandle};
