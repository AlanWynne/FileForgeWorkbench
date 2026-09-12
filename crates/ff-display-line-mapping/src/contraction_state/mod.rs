//! The central `ContractionState` struct implementing the display-line mapping.
//!
//! Starts in One-to-One mode with O(1) memory. Lazily transitions to
//! Full Tracking mode on the first non-trivial operation (hide, fold, wrap).
//!
//! Addresses: Requirements 1-10

use std::collections::HashMap;

use crate::partitioning::FenwickTree;
use crate::traits::DisplayLineMapping;
use crate::types::{
    DisplayLine, DisplayLineCountChange, DocLine, DocPosition, ListenerHandle, SubLine,
};

/// Full per-line tracking data, lazily allocated on first non-trivial operation.
#[derive(Debug, Clone)]
pub(super) struct FullTrackingData {
    /// Per-line visibility. `true` = visible, `false` = hidden.
    visibility: Vec<bool>,
    /// Per-line fold expanded state. `true` = expanded.
    expanded: Vec<bool>,
    /// Per-line display heights.
    heights: Vec<u32>,
    /// Fenwick tree storing effective heights (height if visible, 0 if hidden).
    partitioning: FenwickTree,
}

impl FullTrackingData {
    /// Create full tracking data for `n` lines, all visible/expanded/height-1.
    fn new(n: usize) -> Self {
        Self {
            visibility: vec![true; n],
            expanded: vec![true; n],
            heights: vec![1; n],
            partitioning: FenwickTree::new(n, 1),
        }
    }
}

/// A registered listener with its callback and handle.
pub(super) struct ListenerEntry {
    handle: ListenerHandle,
    callback: Box<dyn Fn(DisplayLineCountChange) + Send + Sync>,
}

/// The central state machine tracking the document-to-display line mapping.
///
/// Starts in One_To_One_Mode with O(1) memory. Lazily transitions to
/// Full Tracking Mode on the first non-trivial operation (hide, fold, wrap).
///
/// Addresses: Requirements 1-10
pub struct ContractionState {
    /// Total number of document lines tracked.
    pub(super) line_count: usize,

    /// Whether we are in optimized one-to-one mode.
    pub(super) one_to_one: bool,

    /// Whether this instance uses 64-bit indexing (large document mode).
    pub(super) large_document: bool,

    /// Full tracking data, None in one-to-one mode.
    pub(super) data: Option<FullTrackingData>,

    /// Per-line fold display text (sparse, independent of mode).
    pub(super) fold_text: HashMap<usize, String>,

    /// Registered change listeners.
    pub(super) listeners: Vec<ListenerEntry>,

    /// Next listener handle ID.
    pub(super) next_handle_id: u64,
}

impl std::fmt::Debug for ContractionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractionState")
            .field("line_count", &self.line_count)
            .field("one_to_one", &self.one_to_one)
            .field("large_document", &self.large_document)
            .finish_non_exhaustive()
    }
}

// SAFETY: ContractionState is Send + Sync because:
// - All fields are Send + Sync (Vec, HashMap, Box<dyn Fn + Send + Sync>)
// - The listeners contain Send + Sync trait objects
unsafe impl Send for ContractionState {}
unsafe impl Sync for ContractionState {}

impl ContractionState {
    /// Create a new ContractionState in one-to-one mode for a document
    /// with the given number of lines.
    ///
    /// Addresses: Requirement 9 AC 1
    pub fn new(line_count: usize) -> Self {
        Self {
            line_count,
            one_to_one: true,
            large_document: false,
            data: None,
            fold_text: HashMap::new(),
            listeners: Vec::new(),
            next_handle_id: 1,
        }
    }

    /// Create a ContractionState with large-document (64-bit) mode enabled.
    ///
    /// Addresses: Requirement 8 AC 1, AC 2, AC 5
    pub fn new_large(line_count: usize) -> Self {
        Self {
            line_count,
            one_to_one: true,
            large_document: true,
            data: None,
            fold_text: HashMap::new(),
            listeners: Vec::new(),
            next_handle_id: 1,
        }
    }

    /// Check whether the state is currently in one-to-one mode.
    ///
    /// Addresses: Requirement 9 AC 4
    pub fn is_one_to_one(&self) -> bool {
        self.one_to_one
    }

    /// Check whether this is a large-document (64-bit) instance.
    ///
    /// Addresses: Requirement 8 AC 5
    pub fn is_large_document(&self) -> bool {
        self.large_document
    }

    /// Lazily allocate full tracking data on first non-trivial operation.
    ///
    /// Addresses: Requirement 9 AC 2, AC 7
    pub(super) fn ensure_data(&mut self) {
        if self.data.is_none() {
            self.data = Some(FullTrackingData::new(self.line_count));
            self.one_to_one = false;
        }
    }

    /// Notify listeners of a display line count change.
    pub(super) fn notify_change(&self, old_count: usize, new_count: usize) {
        if old_count != new_count {
            let change = DisplayLineCountChange {
                old_count,
                new_count,
            };
            for entry in &self.listeners {
                (entry.callback)(change);
            }
        }
    }
}

mod ops;

impl DisplayLineMapping for ContractionState {
    fn display_from_doc(&self, doc_line: DocLine) -> DisplayLine {
        if self.one_to_one {
            return DisplayLine(doc_line.0.min(self.line_count.saturating_sub(1)));
        }

        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        let idx = doc_line.0.min(self.line_count.saturating_sub(1));
        // prefix_sum(idx) gives the cumulative height of all lines before idx
        // But we need to sum effective heights (0 for hidden lines).
        // Our Fenwick tree stores effective heights, so prefix_sum(idx) is correct.
        let sum = data.partitioning.prefix_sum(idx);
        DisplayLine(sum as usize)
    }

    fn display_from_doc_sub(&self, doc_line: DocLine, sub_line: SubLine) -> DisplayLine {
        if self.one_to_one {
            return DisplayLine(doc_line.0.min(self.line_count.saturating_sub(1)));
        }

        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        let idx = doc_line.0.min(self.line_count.saturating_sub(1));
        let height = data.heights[idx] as usize;
        let clamped_sub = sub_line.0.min(height.saturating_sub(1));
        let base = data.partitioning.prefix_sum(idx) as usize;
        DisplayLine(base + clamped_sub)
    }

    fn display_last_from_doc(&self, doc_line: DocLine) -> DisplayLine {
        if self.one_to_one {
            return DisplayLine(doc_line.0.min(self.line_count.saturating_sub(1)));
        }

        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        let idx = doc_line.0.min(self.line_count.saturating_sub(1));
        let base = data.partitioning.prefix_sum(idx) as usize;
        let effective_height = data.partitioning.get(idx) as usize;
        if effective_height == 0 {
            // Hidden line -- return the base (display line before this line)
            DisplayLine(base)
        } else {
            DisplayLine(base + effective_height - 1)
        }
    }

    fn doc_from_display(&self, display_line: DisplayLine) -> DocPosition {
        self.doc_from_display_impl(display_line)
    }

    fn lines_in_doc(&self) -> usize {
        self.line_count
    }

    fn lines_displayed(&self) -> usize {
        if self.one_to_one {
            return self.line_count;
        }
        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        data.partitioning.total() as usize
    }

    fn set_visible(&mut self, start: DocLine, end: DocLine, visible: bool) -> bool {
        self.set_visible_impl(start, end, visible)
    }

    fn get_visible(&self, doc_line: DocLine) -> bool {
        if self.one_to_one {
            return true;
        }
        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        if doc_line.0 >= self.line_count {
            return false;
        }
        data.visibility[doc_line.0]
    }

    fn hidden_lines(&self) -> bool {
        if self.one_to_one {
            return false;
        }
        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        data.visibility.iter().any(|&v| !v)
    }

    fn show_all(&mut self) {
        let old_displayed = self.lines_displayed();
        self.data = None;
        self.one_to_one = true;
        self.fold_text.clear();
        let new_displayed = self.line_count;
        self.notify_change(old_displayed, new_displayed);
    }

    fn set_expanded(&mut self, doc_line: DocLine, expanded: bool) -> bool {
        self.set_expanded_impl(doc_line, expanded)
    }

    fn get_expanded(&self, doc_line: DocLine) -> bool {
        if self.one_to_one {
            return true;
        }
        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        if doc_line.0 >= self.line_count {
            return true;
        }
        data.expanded[doc_line.0]
    }

    fn expand_all(&mut self) -> bool {
        if self.one_to_one {
            return false;
        }
        let data = self
            .data
            .as_mut()
            .expect("data must exist in non-one-to-one mode");
        let mut changed = false;
        for exp in data.expanded.iter_mut() {
            if !*exp {
                *exp = true;
                changed = true;
            }
        }
        changed
    }

    fn contracted_next(&self, start_line: DocLine) -> Option<DocLine> {
        if self.one_to_one {
            return None;
        }
        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        for i in start_line.0..self.line_count {
            if !data.expanded[i] {
                return Some(DocLine(i));
            }
        }
        None
    }

    fn set_fold_display_text(&mut self, doc_line: DocLine, text: Option<&str>) -> bool {
        if doc_line.0 >= self.line_count {
            return false;
        }
        match text {
            Some(t) => {
                let existing = self.fold_text.get(&doc_line.0);
                if existing.map(|s| s.as_str()) == Some(t) {
                    return false;
                }
                self.fold_text.insert(doc_line.0, t.to_string());
                true
            }
            None => self.fold_text.remove(&doc_line.0).is_some(),
        }
    }

    fn get_fold_display_text(&self, doc_line: DocLine) -> Option<&str> {
        self.fold_text.get(&doc_line.0).map(|s| s.as_str())
    }

    fn set_height(&mut self, doc_line: DocLine, height: u32) -> bool {
        self.set_height_impl(doc_line, height)
    }

    fn get_height(&self, doc_line: DocLine) -> u32 {
        if self.one_to_one {
            return 1;
        }
        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        if doc_line.0 >= self.line_count {
            return 1;
        }
        data.heights[doc_line.0]
    }

    fn insert_lines(&mut self, doc_line: DocLine, count: usize) {
        self.insert_lines_impl(doc_line, count)
    }

    fn delete_lines(&mut self, doc_line: DocLine, count: usize) {
        self.delete_lines_impl(doc_line, count)
    }

    fn on_display_count_change(
        &mut self,
        callback: Box<dyn Fn(DisplayLineCountChange) + Send + Sync>,
    ) -> ListenerHandle {
        let handle = ListenerHandle(self.next_handle_id);
        self.next_handle_id += 1;
        self.listeners.push(ListenerEntry { handle, callback });
        handle
    }

    fn remove_listener(&mut self, handle: ListenerHandle) {
        self.listeners.retain(|entry| entry.handle != handle);
    }
}
