//! # ff-shell — Shell Command Subsystem for FileForgeWorkbench
#![warn(clippy::all)]
#![deny(clippy::correctness)]
//!
//! This crate provides the operating-system shell integration layer for the
//! FileForgeWorkbench platform. It enables users to:
//!
//! - Execute OS commands asynchronously with output capture (stdout/stderr)
//! - Host interactive terminal sessions as dockable panels with ANSI/VT100 emulation
//! - Insert command output into documents at `A`/`B` target positions (document capture mode)
//! - Pipe document content (full or selection) as stdin to external commands
//! - Manage working directory, environment variables, and shell profiles
//! - Enforce security controls (`shell.mode`) independently of macro security
//! - Handle process lifecycle: spawning, timeout, cancellation, signal delivery
//! - Stream output incrementally to the Output Panel with scrollback history
//!
//! # Architecture
//!
//! ```text
//! Wave 9 — Desktop Integration
//!
//! ┌─────────────────────────────────────────────────────┐
//! │  ff-shell (THIS CRATE)                               │
//! │  Shell engine, process management, terminal emulation│
//! ├─────────────────────────────────────────────────────┤
//! │  ff-command │ ff-config │ ff-layout │ ff-workflow     │
//! │  ff-document-model │ ff-logging                      │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! # Position in Workspace
//!
//! `ff-shell` is a Wave 9 (Desktop Integration) crate. It depends on
//! `ff-command` (command framework), `ff-config` (configuration),
//! `ff-layout` (docking), `ff-workflow` (async/cancellation), and
//! `ff-document-model` (line insertion for capture mode).

pub mod capture;
pub mod commands;
pub mod config;
pub mod engine;
pub mod environment;
pub mod error;
pub mod executor;
pub mod panel;
pub mod pipe;
pub mod platform;
pub mod process;
pub mod profile;
pub mod terminal;
pub mod working_dir;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use capture::{CaptureHandler, CapturePosition, CaptureResult, CaptureTarget};
pub use config::{ShellConfig, ShellConfigProvider, ShellMode, WorkingDirectoryMode};
pub use engine::ShellEngine;
pub use environment::EnvironmentBuilder;
pub use error::ShellError;
pub use panel::output_panel::{OutputEntry, OutputLine, OutputPanel, OutputStream};
pub use panel::terminal_panel::TerminalPanel;
pub use pipe::StdinPiper;
pub use platform::PlatformDetector;
pub use process::{ExitStatus, ProcessId, ProcessState};
pub use profile::{ProfileResolver, ShellProfile};
pub use terminal::cell::{Cell, CellAttributes, TerminalColor};
pub use terminal::emulator::TerminalEmulator;
pub use terminal::grid::TerminalGrid;
pub use terminal::manager::{SessionId, TerminalManager, TerminalSession};
pub use terminal::pty::PtyHandle;
pub use working_dir::WorkingDirResolver;
