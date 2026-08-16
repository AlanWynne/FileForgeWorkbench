//! # ff-clipboard — Clipboard Operations Subsystem for FileForgeWorkbench
//!
//! This crate provides the unified clipboard subsystem:
//!
//! - **Platform-independent clipboard access** via [`ClipboardProvider`] trait
//! - **Copy/Cut/Paste operations** with mode-aware behaviour (Stream, Line, Rectangular)
//! - **COPY command routing** — disambiguation between in-document, clipboard-paste,
//!   file-insert, and shell-capture modes
//! - **Multi-caret clipboard distribution** — segment-per-caret matching
//! - **Rectangular clipboard handling** — column-block paste semantics
//! - **Line-copy mode** — copy entire line when no selection exists
//! - **Clipboard history ring** — bounded ring buffer of recent entries
//! - **Context menu state** — enabled/disabled computation for Cut/Copy/Paste
//! - **Configuration integration** — clipboard-related settings with defaults
//!
//! ## GUI Independence
//!
//! This crate has zero GUI dependencies. It operates on abstract document model
//! types and produces results that callers use to perform document mutations and
//! undo recording.
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_clipboard::{
//!     ClipboardEngine, ClipboardEntry, ClipboardMode, ClipboardConfig,
//!     InMemoryClipboardProvider, CopyHandler, PasteHandler,
//! };
//!
//! // Create an engine with an in-memory provider (for testing)
//! let provider = InMemoryClipboardProvider::new();
//! let mut engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());
//!
//! // Copy text to clipboard
//! CopyHandler::copy_stream(&mut engine, "hello world").unwrap();
//!
//! // Read it back
//! let entry = engine.read().unwrap();
//! assert_eq!(entry.text(), "hello world");
//! assert_eq!(entry.mode(), ClipboardMode::Stream);
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Clipboard configuration with typed access and validation.
pub mod config;

/// Context menu clipboard state computation.
pub mod context_menu;

/// Copy handler — copy operations for all selection types.
pub mod copy;

/// Cut handler — cut operations (copy + delete) with failure safety.
pub mod cut;

/// Clipboard engine — read/write orchestration with metadata tracking.
pub mod engine;

/// Clipboard entry types — structured content with mode and segments.
pub mod entry;

/// Error types for the clipboard subsystem.
pub mod error;

/// File-insert handler — VFS file reading and content preparation.
pub mod file_insert;

/// Clipboard history ring — bounded ring buffer of recent entries.
pub mod history;

/// Paste handler — mode-aware paste preparation.
pub mod paste;

/// Clipboard provider trait — platform-independent clipboard access.
pub mod provider;

/// COPY command router — disambiguation logic.
pub mod router;

/// Shell-capture handler — document-insertion contract for captured output.
pub mod shell_capture;

/// Line splitter — splits text into logical lines (LF/CRLF/CR).
pub mod splitter;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use config::ClipboardConfig;
pub use context_menu::ClipboardContextMenuState;
pub use copy::CopyHandler;
pub use cut::{CutHandler, CutResult};
pub use engine::ClipboardEngine;
pub use entry::{ClipboardEntry, ClipboardMetadata, ClipboardMode};
pub use error::ClipboardError;
pub use file_insert::{FileInsertHandler, FileInsertResult};
pub use history::ClipboardHistoryRing;
pub use paste::{PasteHandler, PasteMode, PasteResult};
pub use provider::{ClipboardProvider, InMemoryClipboardProvider};
pub use router::{CopyCommandMode, CopyCommandRouter, TargetPosition};
pub use shell_capture::{ShellCaptureHandler, ShellCaptureResult, ShellInsertResult};
pub use splitter::{LineEnding, LineSplitResult, LineSplitter};

// ─── Thread Safety Assertions ───────────────────────────────────────────────

fn _assert_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<ClipboardEntry>();
    assert_sync::<ClipboardEntry>();
    assert_send::<ClipboardMode>();
    assert_sync::<ClipboardMode>();
    assert_send::<ClipboardConfig>();
    assert_sync::<ClipboardConfig>();
    assert_send::<ClipboardContextMenuState>();
    assert_sync::<ClipboardContextMenuState>();
    assert_send::<ClipboardError>();
    assert_sync::<ClipboardError>();
    assert_send::<InMemoryClipboardProvider>();
    assert_sync::<InMemoryClipboardProvider>();
    assert_send::<CopyCommandMode>();
    assert_sync::<CopyCommandMode>();
}
