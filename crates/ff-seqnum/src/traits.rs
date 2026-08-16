//! Trait interfaces for upstream crate integration.
//!
//! These traits decouple `ff-seqnum` from concrete implementations of
//! the document model, language service, undo system, and command registry.
//! At runtime the actual implementations are injected by the shell layer.

use crate::types::ColumnRange;

/// Read-only access to document line content.
///
/// Implemented by the document model crate to provide line data
/// for detection and overlay operations.
pub trait DocumentAccess {
    /// Returns the total number of lines in the document.
    fn line_count(&self) -> usize;

    /// Returns the content of the line at the given 0-based index.
    /// Returns `None` if the index is out of bounds.
    fn line_content(&self, index: usize) -> Option<&str>;
}

/// Mutable access to document line content.
///
/// Extends `DocumentAccess` with the ability to replace column ranges
/// within lines — used by the strip and number engines.
pub trait DocumentMutate: DocumentAccess {
    /// Replace the byte content of the specified column range on the given line
    /// with the provided replacement string.
    ///
    /// If the line is shorter than `range.start_offset()`, it is left unchanged.
    /// If the line is shorter than `range.end_offset()`, it is padded with spaces
    /// to accommodate the replacement.
    fn replace_columns(&mut self, line_index: usize, range: &ColumnRange, content: &str);
}

/// Language profile information for sequence number processing.
///
/// Provides the column definitions and auto-unnum flag from the active
/// language profile TOML.
pub trait LanguageProfile {
    /// Returns the front sequence column range, if defined (e.g., `"1-6"` for COBOL).
    fn sequence_cols_front(&self) -> Option<ColumnRange>;

    /// Returns the back sequence column range, if defined (e.g., `"73-80"`).
    fn sequence_cols_back(&self) -> Option<ColumnRange>;

    /// Returns whether auto-unnum is enabled. Defaults to `true` when absent.
    fn auto_unnum(&self) -> bool;

    /// Returns the language identifier (e.g., `"cobol"`, `"jcl"`).
    fn language_id(&self) -> &str;
}

/// Records column changes for undo/redo integration.
///
/// Used by UNNUM and NUMBER commands to create a single undoable
/// Sequence_Transaction that wraps all line modifications.
pub trait UndoRecorder {
    /// Begin a new sequence transaction with the given description.
    fn begin_sequence_transaction(&mut self, description: &str);

    /// Record a column change: stores the original content before modification.
    fn record_column_change(
        &mut self,
        line_index: usize,
        range: &ColumnRange,
        original_content: String,
    );

    /// Commit the current transaction to the undo stack.
    fn commit(&mut self);

    /// Abort the current transaction (discard recorded changes).
    fn abort(&mut self);
}

/// Command registry for registering sequence number commands.
pub trait CommandRegistry {
    /// Register a command with the given ID, description, and valid modes.
    fn register_command(
        &mut self,
        command_id: &str,
        description: &str,
        valid_in_edit: bool,
        valid_in_browse: bool,
    );
}
