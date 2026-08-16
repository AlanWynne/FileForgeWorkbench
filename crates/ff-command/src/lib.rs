//! # ff-command — Command Framework for FileForgeWorkbench
//!
//! This crate is the **central dispatch mechanism** for all user-facing
//! operations in the FileForgeWorkbench platform. It provides:
//!
//! - A global **Command Registry** for registering and discovering commands
//! - A single **Command Dispatch** entry point through which all state changes flow
//! - Rich **Command Metadata** for menus, palettes, and help systems
//! - Automatic **Undo/Redo Integration** with the transaction system
//! - A **Keyboard Shortcut Registry** with conflict detection and reserved shortcuts
//! - A **Scripting Bridge** for Lua macro command invocation
//! - A **Command History** log for retrieval and audit
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │  Invocation Sources (keyboard, menu, macro, ...) │
//! └──────────────────────┬──────────────────────────┘
//!                        │
//!                        ▼
//! ┌──────────────────────────────────────────────────┐
//! │  CommandDispatch::execute_command()                │
//! │  • Validates command existence                    │
//! │  • Checks enabled predicate                      │
//! │  • Constructs ExecutionContext                    │
//! │  • Routes to CommandHandler                      │
//! │  • Manages undo records                          │
//! │  • Records history                               │
//! └──────────────────────────────────────────────────┘
//! ```
//!
//! ## Position in Architecture
//!
//! `ff-command` is a **Wave 2 (Platform Architecture)** crate. It depends
//! only on `ff-logging` for diagnostics and is consumed by virtually every
//! higher-level crate.

// ─── Public Modules ─────────────────────────────────────────────────────────

/// `CommandId` newtype with validation.
pub mod id;

/// `CommandParams` typed key-value map.
pub mod params;

/// `ExecutionContext` — ambient state for command execution.
pub mod context;

/// `CommandResult` and `UndoRecord` trait.
pub mod result;

/// `CommandMetadata` — descriptive information for commands.
pub mod metadata;

/// `CommandHandler` and `AsyncCommandHandler` traits.
pub mod handler;

/// `CommandRegistry` — thread-safe global registry.
pub mod registry;

/// `CommandDispatch` — single execution entry point.
pub mod dispatch;

/// Undo/Redo integration — stack management.
pub mod undo_bridge;

/// Keyboard shortcut management.
pub mod shortcut;

/// Scripting bridge for Lua macro invocation.
pub mod scripting;

/// Command history — bounded, persistent log.
pub mod history;

/// Error types for the command framework.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use context::ExecutionContext;
pub use dispatch::{CommandDispatch, ContextProvider};
pub use error::{CommandError, ScriptingError};
pub use handler::{AsyncCommandHandler, CommandHandler};
pub use history::{CommandHistory, HistoryEntry};
pub use id::CommandId;
pub use metadata::CommandMetadata;
pub use params::{CommandParams, ParamValue};
pub use registry::CommandRegistry;
pub use result::{CommandResult, UndoRecord};
pub use scripting::{LuaParams, LuaValue, ScriptingBridge, ScriptingCommandInfo};
pub use shortcut::{KeyChord, KeyCode, Modifiers, ShortcutBinding, ShortcutRegistry};
pub use undo_bridge::{DefaultUndoManager, UndoManager};
