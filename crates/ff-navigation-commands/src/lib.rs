//! # ff-navigation-commands — Navigation Commands for FileForgeWorkbench
//!
//! This crate implements all navigation, display-artifact, and line-reorder
//! commands for the FileForge editor workbench:
//!
//! - **LOCATE** — jump to a line number or named label
//! - **SORT** — reorder lines by column key (the only undoable command)
//! - **COLS** — display/toggle a column ruler overlay
//! - **BOUNDS/BNDS** — set/clear active column boundaries
//! - **UP/DOWN/LEFT/RIGHT/TOP/BOTTOM** — viewport scroll commands
//! - **PARA_UP/PARA_DOWN** — paragraph navigation
//! - **WORD_LEFT/WORD_RIGHT/WORD_END_RIGHT** — word navigation
//! - **WORD_PART_LEFT/WORD_PART_RIGHT** — sub-word (camelCase) navigation
//! - **Vertical caret movement** — line/page up/down with column affinity
//! - **DOC_START/DOC_END** — document start/end navigation
//!
//! ## Architecture
//!
//! This is a **Wave 5 (Command Engine)** crate that depends on:
//! - `ff-viewport-scrolling` (Wave 4) for viewport state delegation
//! - `ff-document-model` (Wave 4) for line content and character classification
//! - `ff-command` (Wave 2) for command registration
//! - `ff-undo-redo` (Wave 4) for SORT transaction wrapping
//!
//! All navigation commands except SORT are non-undoable: they modify only
//! viewport/session state, never document content.

// ─── Public Modules ─────────────────────────────────────────────────────────

/// LOCATE command — line number and label navigation.
pub mod locate;

/// SORT command — undoable line reorder.
pub mod sort;

/// COLS command — column ruler display artifacts.
pub mod cols;

/// BOUNDS/BNDS command — active bounds state and BNDS_Line.
pub mod bounds;

/// Viewport scroll commands — UP, DOWN, LEFT, RIGHT, TOP, BOTTOM.
pub mod scroll;

/// Paragraph navigation — PARA_UP, PARA_DOWN.
pub mod paragraph;

/// Word navigation — WORD_LEFT, WORD_RIGHT, WORD_END_RIGHT.
pub mod word;

/// Word-part (camelCase/sub-word) navigation — WORD_PART_LEFT, WORD_PART_RIGHT.
pub mod word_part;

/// Vertical caret movement with column affinity.
pub mod vertical_caret;

/// Document start/end navigation — DOC_START, DOC_END.
pub mod doc_nav;

/// Character classification engine for word navigation.
pub mod char_class;

/// Delegation command registrations.
pub mod delegation;

/// Command registration and metadata.
pub mod registration;

/// Configuration keys and defaults.
pub mod config;

/// Error types.
pub mod error;

/// Core data types shared across modules.
pub mod types;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use bounds::BoundsManager;
pub use char_class::{CharClassifier, CharacterClass};
pub use cols::ColsManager;
pub use config::{
    load_navigation_config, DEFAULT_BOUNDS_AFFECT_FIND, DEFAULT_HORIZONTAL_SCROLL_COLUMNS,
    DEFAULT_PAGE_OVERLAP_LINES, KEY_BOUNDS_AFFECT_FIND, KEY_HORIZONTAL_SCROLL_COLUMNS,
    KEY_PAGE_OVERLAP_LINES, KEY_WORD_CHARACTERS,
};
pub use delegation::{delegation_entries, DelegationEntry};
pub use doc_nav::DocStartEndNav;
pub use error::NavigationError;
pub use locate::{LabelRegistry, LocateCommand};
pub use paragraph::ParagraphNav;
pub use registration::{owned_command_metadata, CommandMode, NavCommandMetadata, UndoClass};
pub use scroll::ScrollCommands;
pub use sort::{SortCommand, SortUndoRecord};
pub use types::{
    ActiveBounds, ColsLine, ColsToggleResult, NavigationConfig, SelectionModifier, SortDirection,
    SortParams, SortScope, WordDirection, WordNavKind, WordPartBoundary,
};
pub use vertical_caret::VerticalCaretNav;
pub use word::WordNav;
pub use word_part::WordPartNav;

// ─── Send + Sync assertions ────────────────────────────────────────────────

#[cfg(test)]
mod thread_safety {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn types_are_send_sync() {
        assert_send::<BoundsManager>();
        assert_sync::<BoundsManager>();
        assert_send::<ColsManager>();
        assert_sync::<ColsManager>();
        assert_send::<CharClassifier>();
        assert_sync::<CharClassifier>();
        assert_send::<NavigationError>();
        assert_sync::<NavigationError>();
        assert_send::<ActiveBounds>();
        assert_sync::<ActiveBounds>();
        assert_send::<SortParams>();
        assert_sync::<SortParams>();
    }
}
