//! # ff-background-io — Async File Loading/Saving for FileForgeWorkbench
//!
//! This crate provides the async file loading and saving infrastructure that keeps
//! the GUI responsive during all file I/O operations. It implements chunked streaming
//! reads with progress reporting, cancellable operations, background save with
//! temp-file + atomic rename for data integrity, and large-file streaming (>100 MB).
//!
//! All file operations flow through the **Virtual File System abstraction**
//! (FFW-ARCH-001) — background-io uses the VFS provider async interface and never
//! calls `std::fs`, `tokio::fs`, or any platform-specific I/O directly.
//!
//! ## Key Components
//!
//! - [`IoError`] — error type wrapping `VfsError` with operation phase context
//! - [`ProgressState`] — current state of an I/O operation (bytes, percentage, ETA)
//! - [`IoPhase`] — current phase of an I/O operation (reading, writing, etc.)
//! - [`IoCancellationToken`] — cooperative cancellation signal
//! - [`IoTaskHandle`] — handle for querying progress, cancelling, and awaiting tasks
//! - [`IoConfig`] — configuration for chunk size, thresholds, concurrency
//! - [`ChunkSize`] — validated chunk size (4 KB – 1 MB)
//! - [`LargeFileThreshold`] — validated large-file threshold (10 MB – 4096 MB)
//! - [`TaskId`] — unique identifier for background I/O tasks
//! - [`TaskState`] — lifecycle state (queued, in-progress, complete, failed, cancelled)

pub mod cancellation;
pub mod config;
pub mod error;
pub mod handle;
pub mod load;
pub mod progress;
pub mod retry;
pub mod save;
pub mod service;
pub mod subsystem;
pub mod types;

// Re-exports for the public API surface
pub use cancellation::IoCancellationToken;
pub use config::IoConfig;
pub use error::IoError;
pub use handle::IoTaskHandle;
pub use load::{ChunkCallback, LoadOptions};
pub use progress::{IoPhase, ProgressState, RateCalculator};
pub use retry::RetryPolicy;
pub use save::{DocumentChunkSource, SaveOptions};
pub use service::BackgroundIoService;
pub use subsystem::BackgroundIoSubsystem;
pub use types::{
    ChunkSize, IoSuccess, IoTaskEntry, IoTaskType, LargeFileThreshold, TaskId, TaskState,
};
