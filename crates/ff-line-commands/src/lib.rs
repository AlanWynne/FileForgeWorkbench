//! # ff-line-commands — ISPF Line Command Engine for FileForgeWorkbench
//!
//! This crate implements the ISPF line command engine: prefix-area command
//! parsing, block pairing, pending state management, compatibility validation,
//! and execution logic for all line commands.
//!
//! ## Supported Commands
//!
//! - **Delete** (D, Dn, DD) — remove lines, undoable
//! - **Insert** (I, In) — add blank lines, undoable
//! - **Repeat** (R, Rn, RR) — duplicate lines, undoable
//! - **Copy** (C, CC + A/B) — copy lines to target, undoable
//! - **Move** (M, MM + A/B) — move lines to target, undoable
//! - **Exclude** (X, Xn, XX) — hide lines, session-state only
//! - **Tag/Untag** (T, TT, U, UU) — mark lines, session-state only
//! - **Shift Right** (>, >n, >>) — indent content, undoable
//! - **Shift Left** (<, <n, <<) — de-indent content, undoable
//! - **Bounds-Aware Shift** (), )), (, (( — shift within column bounds, undoable
//!
//! ## GUI Independence
//!
//! This crate has zero GUI dependencies. The prefix-area visual representation
//! is the responsibility of the UI layer.
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_line_commands::{LineCommandParser, LineCommandConfig, PendingCommandStore};
//! use ff_line_commands::command::{LineCommandKind, LineCommandCategory, classify};
//!
//! // Parse a line command
//! let cmd = LineCommandParser::parse("D5", 3).unwrap();
//! assert_eq!(cmd.kind, LineCommandKind::DeleteCount(5));
//! assert_eq!(classify(&cmd.kind), LineCommandCategory::Immediate);
//!
//! // Create a pending store
//! let store = PendingCommandStore::new();
//! assert!(store.is_empty());
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Line command types, enums, and classification logic.
pub mod command;

/// Line command parser — string to typed command conversion.
pub mod parser;

/// Pending command store — per-session state management.
pub mod pending;

/// Block pair validator — pair matching and normalization.
pub mod block_pair;

/// Command compatibility matrix — primary vs line command rules.
pub mod compatibility;

/// Resolution engine — determines executable commands from pending state.
pub mod resolution;

/// Execution engine — performs document mutations.
pub mod execution;

/// Command framework registration and handlers.
pub mod commands;

/// Configuration values for the line commands subsystem.
pub mod config;

/// Error types for the crate.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use block_pair::BlockPairValidator;
pub use command::{
    BlockCommandKind, BlockPair, ExecutableCommand, LineCommandCategory, LineCommandKind,
    ParsedLineCommand, SourceOperation, SourceTarget, TargetPosition,
};
pub use commands::handlers::{all_command_ids, command_ids, is_session_state, is_undoable};
pub use compatibility::CommandCompatibilityMatrix;
pub use config::LineCommandConfig;
pub use error::LineCommandError;
pub use execution::clipboard_copy::collect_clipboard_text;
pub use execution::ExecutionEngine;
pub use parser::LineCommandParser;
pub use pending::{PendingCommand, PendingCommandStore, PendingReason};
pub use resolution::{ResolutionEngine, ResolutionResult};

// ─── Thread Safety Assertions ───────────────────────────────────────────────

fn _assert_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<ParsedLineCommand>();
    assert_sync::<ParsedLineCommand>();
    assert_send::<LineCommandKind>();
    assert_sync::<LineCommandKind>();
    assert_send::<LineCommandConfig>();
    assert_sync::<LineCommandConfig>();
    assert_send::<LineCommandError>();
    assert_sync::<LineCommandError>();
    assert_send::<PendingCommandStore>();
    assert_sync::<PendingCommandStore>();
    assert_send::<ExecutableCommand>();
    assert_sync::<ExecutableCommand>();
    assert_send::<BlockPair>();
    assert_sync::<BlockPair>();
    assert_send::<SourceTarget>();
    assert_sync::<SourceTarget>();
}
