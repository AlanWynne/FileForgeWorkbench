//! # ff-undo-redo — Undo/Redo Transaction System for FileForgeWorkbench
//!
//! This crate implements the full transaction system for undo and redo operations.
//! It owns the undo/redo stacks, manages transaction boundaries and coalescing,
//! tracks save-point semantics for the dirty flag, supports bulk transaction
//! optimisations, provides tentative action support for IME composition,
//! manages selection history for cursor restoration, and persists undo state
//! for crash recovery.
//!
//! ## Architecture
//!
//! - **GUI-independent** — zero GUI dependencies; pure data structures and logic
//! - **Per-document isolation** — each document has its own undo manager
//! - **Command-driven** — bridges `ff-command` and `ff-document-model` via traits
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_undo_redo::{DocumentUndoManager, UndoConfig};
//!
//! let mut mgr = DocumentUndoManager::new(UndoConfig::default());
//! mgr.record_insert(0, b"Hello");
//! assert!(mgr.can_undo());
//! mgr.undo().unwrap();
//! assert!(mgr.can_redo());
//! ```

pub mod bulk;
pub mod coalesce;
pub mod config;
pub mod container;
pub mod edit_op;
pub mod error;
pub mod manager;
pub mod notify;
pub mod record_id;
pub mod recovery;
pub mod save_point;
pub mod scrap;
pub mod selection;
pub mod stack;
pub mod tentative;
pub mod transaction;
pub mod undo_manager_trait;
pub mod validate;

// --- Public API Re-exports ---

pub use bulk::{BulkScope, BulkTransaction, IndexTransaction, RuleTransaction, TransformRule};
pub use config::UndoConfig;
pub use container::UndoableState;
pub use edit_op::EditOperation;
pub use error::UndoError;
pub use manager::DocumentUndoManager;
pub use notify::{ListenerId, UndoNotifier};
pub use record_id::{LogicalRecordId, RecordIdMap};
pub use selection::{CaretPosition, SelectionState, SelectionType};
pub use transaction::Transaction;
pub use undo_manager_trait::{EditTarget, WorkbenchUndoManager};
