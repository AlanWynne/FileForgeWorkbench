//! Public trait defining the full display-line-mapping API.
//!
//! Consumers depend on this trait rather than the concrete `ContractionState`,
//! enabling testability and decoupling.

use crate::types::{
    DisplayLine, DisplayLineCountChange, DocLine, DocPosition, ListenerHandle, SubLine,
};

/// Public trait defining the full display-line-mapping API.
///
/// Consumers (viewport, scrollbar, gutter, find) depend on this trait
/// rather than the concrete `ContractionState` implementation.
///
/// Addresses: Requirement 7 AC 10
pub trait DisplayLineMapping: Send + Sync {
    // --- Document-to-Display Conversion ---

    /// Returns the first display line index for the given document line.
    /// Equals the cumulative sum of display heights of all preceding visible lines.
    ///
    /// Addresses: Requirement 1 AC 1
    fn display_from_doc(&self, doc_line: DocLine) -> DisplayLine;

    /// Returns the display line index for a specific sub-line within a document line.
    /// Clamps sub_line to height - 1 if it exceeds the line's display height.
    ///
    /// Addresses: Requirement 1 AC 2
    fn display_from_doc_sub(&self, doc_line: DocLine, sub_line: SubLine) -> DisplayLine;

    /// Returns the last display line index occupied by the given document line.
    ///
    /// Addresses: Requirement 1 AC 3
    fn display_last_from_doc(&self, doc_line: DocLine) -> DisplayLine;

    // --- Display-to-Document Conversion ---

    /// Returns the document line and sub-line offset for a given display line.
    /// Always returns a visible line. Clamps out-of-range display lines.
    ///
    /// Addresses: Requirement 1 AC 4, AC 5, AC 6
    fn doc_from_display(&self, display_line: DisplayLine) -> DocPosition;

    // --- Line Counts ---

    /// Total number of document lines in the mapping.
    ///
    /// Addresses: Requirement 1 AC 7
    fn lines_in_doc(&self) -> usize;

    /// Total display line count (sum of heights of all visible lines).
    ///
    /// Addresses: Requirement 1 AC 8
    fn lines_displayed(&self) -> usize;

    // --- Visibility ---

    /// Set visibility for a range of document lines [start, end] inclusive.
    /// Returns true if any line's visibility actually changed.
    ///
    /// Addresses: Requirement 2 AC 1
    fn set_visible(&mut self, start: DocLine, end: DocLine, visible: bool) -> bool;

    /// Query visibility for a single document line.
    ///
    /// Addresses: Requirement 2 AC 2
    fn get_visible(&self, doc_line: DocLine) -> bool;

    /// Returns true if any document line is currently hidden.
    ///
    /// Addresses: Requirement 2 AC 5
    fn hidden_lines(&self) -> bool;

    /// Make all lines visible and reset to one-to-one mode.
    ///
    /// Addresses: Requirement 2 AC 6
    fn show_all(&mut self);

    // --- Fold State ---

    /// Set the expanded/collapsed state of a fold header line.
    /// Returns true if the state changed.
    ///
    /// Addresses: Requirement 3 AC 1
    fn set_expanded(&mut self, doc_line: DocLine, expanded: bool) -> bool;

    /// Query the expanded state of a document line.
    /// Returns true for non-fold-header lines and expanded fold headers.
    ///
    /// Addresses: Requirement 3 AC 2
    fn get_expanded(&self, doc_line: DocLine) -> bool;

    /// Set all fold headers to expanded state.
    /// Returns true if any fold state changed.
    ///
    /// Addresses: Requirement 3 AC 3
    fn expand_all(&mut self) -> bool;

    /// Find the next collapsed fold header at or after start_line.
    /// Returns None if no contracted fold exists beyond that point.
    ///
    /// Addresses: Requirement 3 AC 4
    fn contracted_next(&self, start_line: DocLine) -> Option<DocLine>;

    /// Set fold display text for a collapsed fold header.
    /// Returns true if the text changed.
    ///
    /// Addresses: Requirement 3 AC 7
    fn set_fold_display_text(&mut self, doc_line: DocLine, text: Option<&str>) -> bool;

    /// Get fold display text for a line. Returns None if not set.
    ///
    /// Addresses: Requirement 3 AC 8
    fn get_fold_display_text(&self, doc_line: DocLine) -> Option<&str>;

    // --- Wrap Height ---

    /// Set the display height (number of sub-lines) for a document line.
    /// Returns true if the height changed.
    ///
    /// Addresses: Requirement 4 AC 1
    fn set_height(&mut self, doc_line: DocLine, height: u32) -> bool;

    /// Get the current display height of a document line.
    ///
    /// Addresses: Requirement 4 AC 2
    fn get_height(&self, doc_line: DocLine) -> u32;

    // --- Incremental Updates ---

    /// Insert new document lines at the given position.
    /// New lines are initialized as visible with height 1.
    ///
    /// Addresses: Requirement 6 AC 1
    fn insert_lines(&mut self, doc_line: DocLine, count: usize);

    /// Remove document lines starting at the given position.
    ///
    /// Addresses: Requirement 6 AC 2
    fn delete_lines(&mut self, doc_line: DocLine, count: usize);

    // --- Change Notification ---

    /// Register a listener for display-line-count changes.
    /// Returns a handle for later removal.
    ///
    /// Addresses: Requirement 7 AC 9
    fn on_display_count_change(
        &mut self,
        callback: Box<dyn Fn(DisplayLineCountChange) + Send + Sync>,
    ) -> ListenerHandle;

    /// Remove a previously registered listener.
    fn remove_listener(&mut self, handle: ListenerHandle);
}
