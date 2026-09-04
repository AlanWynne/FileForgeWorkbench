//! # ff-edit-operations — Text Editing Behaviour Layer for FileForgeWorkbench
//!
//! This crate implements all text editing behaviour for the FileForgeWorkbench
//! editor. It sits between the low-level document buffer (`ff-document-model`)
//! and the user-facing command dispatch (`ff-command`), providing:
//!
//! - **Edit mode management** — Insert, Overstrike, and Browse mode state machine
//! - **Character insertion and deletion** — single character, word, line, range operations
//! - **Selection model** — stream, rectangular, and multi-caret selection with position adjustment
//! - **Multi-caret coordination** — simultaneous editing at multiple positions
//! - **Edit boundaries (BOUNDS)** — ISPF-heritage column-range protection
//! - **Line manipulation** — transpose, duplicate, case change
//! - **Transaction recording** — defining undo boundaries and grouping multi-caret operations
//! - **Clipboard integration** — edit-side cut/copy/paste semantics for all selection types
//!
//! ## GUI Independence
//!
//! This crate has zero GUI dependencies — it operates on abstract types and produces
//! transaction records for `ff-undo-redo-transactions`. All operations are driven by
//! the command framework via `ff-command`.
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_edit_operations::{EditModeManager, EditMode, SelectionPosition, SelectionRange, SelectionContainer};
//!
//! // Create an edit mode manager (defaults to Insert)
//! let mut mode = EditModeManager::new();
//! assert_eq!(mode.mode(), EditMode::Insert);
//!
//! // Toggle to Overstrike
//! mode.toggle();
//! assert_eq!(mode.mode(), EditMode::Overstrike);
//!
//! // Create a selection container
//! let mut selection = SelectionContainer::new();
//! assert_eq!(selection.count(), 1);
//!
//! // Add a caret at line 5, column 10
//! selection.add(SelectionRange::collapsed(SelectionPosition::new(5, 10)));
//! assert!(selection.is_multi_caret());
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Error types for the edit-operations crate.
pub mod error;

/// Selection position type with virtual space support.
pub mod position;

/// Selection range type (anchor + caret pair).
pub mod range;

/// Selection container — holds all active SelectionRanges.
pub mod selection;

/// Edit mode management — Insert, Overstrike, Browse.
pub mod mode;

/// Edit boundaries (BOUNDS) — column-range protection.
pub mod bounds;

/// Rectangular (column) selection support.
pub mod rectangular;

/// Clipboard content type and edit-side semantics.
pub mod clipboard;

/// Multi-caret coordination.
pub mod multi_caret;

/// Transaction recording — EditorTransaction, LineSnapshot, modified line tracking.
pub mod transaction;

/// Edit profile -- CAPS, NULLS, STATS, LOCK, HILITE settings.
pub mod profile;

/// Edit profile persistence -- TOML serialisation/deserialisation.
pub mod profile_persistence;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use error::EditError;

pub use position::SelectionPosition;

pub use range::SelectionRange;

pub use selection::{DocumentModification, SelectionContainer};

pub use mode::{EditMode, EditModeManager};

pub use bounds::{BoundsEnforcer, EditBounds};

pub use rectangular::{RectDirection, RectangularSelection, SelectionKind};

pub use clipboard::ClipboardContent;

pub use multi_caret::{MultiCaretCoordinator, SingleEditResult};

pub use transaction::{EditorTransaction, LineSnapshot, ModifiedLineTracker, UndoGroup};

pub use profile::{
    CapsMode, EditProfile, HiliteMode, NullsMode, ProfileError, ProfileLock, StatsMode,
};

pub use profile_persistence::{deserialize_profile, serialize_profile, ProfilePersistError};

// ─── Thread Safety Assertions ───────────────────────────────────────────────
// All core types must be Send + Sync for use across threads.

fn _assert_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<SelectionPosition>();
    assert_sync::<SelectionPosition>();
    assert_send::<SelectionRange>();
    assert_sync::<SelectionRange>();
    assert_send::<SelectionContainer>();
    assert_sync::<SelectionContainer>();
    assert_send::<EditModeManager>();
    assert_sync::<EditModeManager>();
    assert_send::<BoundsEnforcer>();
    assert_sync::<BoundsEnforcer>();
    assert_send::<RectangularSelection>();
    assert_sync::<RectangularSelection>();
    assert_send::<ClipboardContent>();
    assert_sync::<ClipboardContent>();
    assert_send::<EditorTransaction>();
    assert_sync::<EditorTransaction>();
    assert_send::<ModifiedLineTracker>();
    assert_sync::<ModifiedLineTracker>();
    assert_send::<UndoGroup>();
    assert_sync::<UndoGroup>();
    assert_send::<EditError>();
    assert_sync::<EditError>();

    assert_send::<EditProfile>();
    assert_sync::<EditProfile>();
}
