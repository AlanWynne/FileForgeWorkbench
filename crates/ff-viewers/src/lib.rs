//! # ff-viewers — Extensible File Viewer Framework for FileForgeWorkbench
//!
//! This crate provides the complete viewer framework: a `FileViewer` trait that
//! all viewer implementations fulfil, a thread-safe `ViewerRegistry` mapping
//! viewer keys to implementations, the `PREVIEW` command for activation, built-in
//! viewers for common file types, and plugin extensibility for custom viewers.
//!
//! ## Design Principles
//!
//! - **View-Only Rendering** — viewers NEVER modify document content. The
//!   `render` method receives `&[u8]` (immutable), enforced at the type level.
//! - **Plugin-Extensible** — plugins register viewers at runtime via the plugin bridge.
//! - **Command-Driven** — all viewer operations are invoked through `PREVIEW`.
//! - **GUI-Independent** — viewer logic is decoupled from the rendering shell.
//!
//! ## Architecture
//!
//! ```text
//! ViewerRegistry ← central viewer storage + lookup
//! ├── FileViewer trait ← contract for all viewers
//! ├── Built-in Viewers ← asa-report, hex, image, csv-table
//! ├── ContentSelector ← auto-detection + matching
//! ├── RefreshController ← debounced change notification
//! ├── PreviewCommand ← PREVIEW command handler
//! ├── ViewerPanel ← DockablePanel host for viewer output
//! ├── PluginBridge ← plugin registration/deregistration
//! └── ReadOnlyGuard ← mutation rejection during Viewer_Mode
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_viewers::{ViewerRegistry, ViewerKey, built_in};
//!
//! let registry = ViewerRegistry::new();
//! built_in::register_built_in_viewers(&registry).unwrap();
//!
//! let key = ViewerKey::new("csv-table").unwrap();
//! assert!(registry.contains(&key));
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Error types for the viewer framework.
pub mod error;

/// ViewerKey — validated, unique identifier for a viewer.
pub mod key;

/// FileViewer trait definition.
pub mod trait_def;

/// ViewerRegistry — central registry of FileViewer implementations.
pub mod registry;

/// Built-in viewer implementations.
pub mod built_in;

/// PREVIEW command handler.
pub mod command;

/// Content selection and viewer matching logic.
pub mod selection;

/// ViewerPanel — DockablePanel implementation for viewer output.
pub mod panel;

/// Plugin viewer bridge — plugin registration/deregistration.
pub mod plugin_bridge;

/// RefreshController — debounced change notification for viewer refresh.
pub mod refresh;

/// Viewer configuration — TOML `[viewers]` section parsing.
pub mod config;

/// Read-only enforcement — mutation rejection during Viewer_Mode.
pub mod readonly;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use config::ViewerConfig;
pub use error::ViewerError;
pub use key::ViewerKey;
pub use panel::{ViewerPanel, ViewerPosition};
pub use readonly::ReadOnlyGuard;
pub use refresh::RefreshController;
pub use registry::{ViewerInfo, ViewerRegistry, ViewerSource};
pub use selection::{ContentMatch, ContentSelector, MatchConfidence, MatchMethod};
pub use trait_def::FileViewer;
