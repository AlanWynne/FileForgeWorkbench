//! # ff-command-semantics — ISPF-Inspired Command Execution Pipeline
//!
//! This crate is the command execution pipeline for FileForgeWorkbench.
//! It accepts raw command-line text, parses it into structured tokens,
//! resolves scope, validates preconditions, executes transactionally,
//! and reports results via short status messages.
//!
//! ## Architecture
//!
//! - **Primary Command Parser** — tokenises command-line text into command name + arguments
//! - **Line Command Parser** — interprets prefix-area strings into kind + count descriptors
//! - **Scope Resolution** — priority-ordered algorithm for determining target lines/columns
//! - **Command Engine** — orchestrates the 10-step execution pipeline
//! - **Session State** — tracks pending line commands, last command, tags, cursor
//! - **Status Messages** — ≤200 character feedback messages
//! - **Configuration** — runtime-configurable behaviours
//! - **HELP Command** — context-sensitive online documentation
//!
//! ## GUI Independence
//!
//! This crate has zero GUI dependencies. It provides pure command parsing,
//! scope resolution, and execution orchestration logic.

pub mod config;
pub mod engine;
pub mod error;
pub mod help;
pub mod line_parser;
pub mod parser;
pub mod scope;
pub mod session;
pub mod status;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use config::{CommandConfig, InvalidLineCommandPolicy};
pub use engine::CommandEngine;
pub use error::{CommandSemanticsError, ParseError, ScopeError};
pub use help::{HelpEngine, HelpTopic};
pub use line_parser::{LineCommandDescriptor, LineCommandKind, LineCommandParser};
pub use parser::{CommandToken, ParsedCommand, PrimaryCommandParser, QuoteStyle};
pub use scope::{ColumnBounds, ResolvedScope, ScopeFilter, ScopeLines, ScopeResolver, ScopeSource};
pub use session::{PendingLineCommand, SessionState};
pub use status::{StatusKind, StatusMessage};
