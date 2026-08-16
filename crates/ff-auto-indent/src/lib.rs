//! # ff-auto-indent — Language-Aware Automatic Indentation Engine
//!
//! This crate provides the auto-indentation subsystem for FileForgeWorkbench.
//! It computes indentation adjustments triggered by newline insertion, provides
//! explicit indent/unindent commands, handles block comment auto-continuation,
//! and supports smart indent patterns defined per-language in TOML files.
//!
//! ## Design Principles
//!
//! - **GUI-independent** — operates purely on document model line content
//! - **Language-aware** — uses regex patterns from language TOML definitions
//! - **Three modes** — None, Maintain, Smart (with fallback logic)
//! - **Transaction-safe** — all modifications grouped for single-step undo
//! - **Hot-reload** — configuration changes apply without document close/reopen
//!
//! ## Architecture
//!
//! The auto-indent engine is triggered by:
//! 1. Newline insertion (via `ff-edit-operations`) → `compute_newline_indent()`
//! 2. Character typed (closing delimiter) → `compute_decrease_on_type()`
//! 3. Indent/Unindent commands (via `ff-command`) → `indent_lines()` / `unindent_lines()`

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Auto-indent mode enum and resolution logic.
pub mod mode;

/// Indent configuration (indent_size, tab_size, use_tabs) and accessors.
pub mod config;

/// Maintain-indent engine — copies reference line whitespace.
pub mod maintain;

/// Smart-indent engine — pattern-based increase/decrease logic.
pub mod smart;

/// Indent pattern compilation, caching, and matching.
pub mod patterns;

/// Block expansion logic (Enter between braces).
pub mod block;

/// Comment continuation (block and line comments).
pub mod comment;

/// Indent/Unindent command handler and registration.
pub mod indent_cmd;

/// IndentDecision type and related result structures.
pub mod decision;

/// Error types for the auto-indent subsystem.
pub mod error;

/// Core types: IndentLevel newtype and related helpers.
pub mod types;

/// Top-level auto-indent service facade.
pub mod service;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use comment::{CommentConfig, CommentTableRaw};
pub use config::IndentConfig;
pub use decision::{BlockExpansion, CommentContinuation, CommentKind, IndentDecision};
pub use error::AutoIndentError;
pub use indent_cmd::{indent_lines, unindent_lines, IndentLineEdit};
pub use mode::{resolve_effective_mode, AutoIndentMode};
pub use patterns::{IndentPatterns, IndentTableRaw};
pub use service::AutoIndentService;
pub use smart::IndentContext;
pub use types::IndentLevel;
