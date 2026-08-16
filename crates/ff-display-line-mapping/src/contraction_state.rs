//! The central `ContractionState` struct implementing the display-line mapping.
//!
//! Starts in One-to-One mode with O(1) memory. Lazily transitions to
//! Full Tracking mode on the first non-trivial operation (hide, fold, wrap).
//!
//! Addresses: Requirements 1–10

use std::collections::HashMap;

use crate::partitioning::FenwickTree;
use crate::traits::DisplayLineMapping;
use crate::types::{
    DisplayLine, DisplayLineCountChange, DocLine, DocPosition, ListenerHandle, SubLine,
};

/// Full per-line tracking data, lazily allocated on first non-trivial operation.
#[derive(Debug, Clone)]
struct FullTrackingData {
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
struct ListenerEntry {
    handle: ListenerHandle,
    callback: Box<dyn Fn(DisplayLineCountChange) + Send + Sync>,
}

/// The central state machine tracking the document-to-display line mapping.
///
/// Starts in One_To_One_Mode with O(1) memory. Lazily transitions to
/// Full Tracking Mode on the first non-trivial operation (hide, fold, wrap).
///
/// Addresses: Requirements 1–10
pub struct ContractionState {
    /// Total number of document lines tracked.
    line_count: usize,

    /// Whether we are in optimized one-to-one mode.
    one_to_one: bool,

    /// Whether this instance uses 64-bit indexing (large document mode).
    large_document: bool,

    /// Full tracking data, None in one-to-one mode.
    data: Option<FullTrackingData>,

    /// Per-line fold display text (sparse, independent of mode).
    fold_text: HashMap<usize, String>,

    /// Registered change listeners.
    listeners: Vec<ListenerEntry>,

    /// Next listener handle ID.
    next_handle_id: u64,
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
    fn ensure_data(&mut self) {
        if self.data.is_none() {
            self.data = Some(FullTrackingData::new(self.line_count));
            self.one_to_one = false;
        }
    }

    /// Notify listeners of a display line count change.
    fn notify_change(&self, old_count: usize, new_count: usize) {
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
            // Hidden line — return the base (display line before this line)
            DisplayLine(base)
        } else {
            DisplayLine(base + effective_height - 1)
        }
    }

    fn doc_from_display(&self, display_line: DisplayLine) -> DocPosition {
        if self.line_count == 0 {
            return DocPosition {
                doc_line: DocLine(0),
                sub_line: SubLine(0),
            };
        }

        if self.one_to_one {
            let clamped = display_line.0.min(self.line_count.saturating_sub(1));
            return DocPosition {
                doc_line: DocLine(clamped),
                sub_line: SubLine(0),
            };
        }

        let data = self
            .data
            .as_ref()
            .expect("data must exist in non-one-to-one mode");
        let total_displayed = data.partitioning.total() as usize;

        if total_displayed == 0 {
            // All lines hidden — return first line
            return DocPosition {
                doc_line: DocLine(0),
                sub_line: SubLine(0),
            };
        }

        // Clamp to valid range
        let target = if display_line.0 >= total_displayed {
            total_displayed.saturating_sub(1)
        } else {
            display_line.0
        };

        let doc_idx = data.partitioning.find_prefix(target as i64);
        let base = data.partitioning.prefix_sum(doc_idx) as usize;
        let sub = target - base;

        DocPosition {
            doc_line: DocLine(doc_idx),
            sub_line: SubLine(sub),
        }
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
        // Validate range
        if start.0 > end.0 || end.0 >= self.line_count {
            return false;
        }

        if self.one_to_one && visible {
            // Already all visible, nothing to change
            return false;
        }

        if !visible || !self.one_to_one {
            self.ensure_data();
        }

        let old_displayed = self.lines_displayed();
        let data = self
            .data
            .as_mut()
            .expect("data must exist after ensure_data");
        let mut changed = false;

        for i in start.0..=end.0 {
            let was_visible = data.visibility[i];
            if was_visible != visible {
                data.visibility[i] = visible;
                changed = true;
                let height = data.heights[i] as i64;
                if visible {
                    // Showing: add height to the Fenwick tree
                    data.partitioning.set(i, height);
                } else {
                    // Hiding: set effective height to 0
                    data.partitioning.set(i, 0);
                }
            }
        }

        if changed {
            let new_displayed = self.lines_displayed();
            self.notify_change(old_displayed, new_displayed);
        }

        changed
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
        if doc_line.0 >= self.line_count {
            return false;
        }

        if self.one_to_one && expanded {
            // Already all expanded
            return false;
        }

        if !expanded {
            self.ensure_data();
        }

        if let Some(data) = self.data.as_mut() {
            let was_expanded = data.expanded[doc_line.0];
            if was_expanded != expanded {
                data.expanded[doc_line.0] = expanded;
                return true;
            }
        }

        false
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
        if doc_line.0 >= self.line_count || height == 0 {
            return false;
        }

        if self.one_to_one && height == 1 {
            return false;
        }

        if height != 1 {
            self.ensure_data();
        }

        if let Some(data) = self.data.as_mut() {
            let old_height = data.heights[doc_line.0];
            if old_height == height {
                return false;
            }

            let old_displayed = data.partitioning.total() as usize;
            data.heights[doc_line.0] = height;

            // Only update Fenwick tree if the line is visible
            if data.visibility[doc_line.0] {
                data.partitioning.set(doc_line.0, height as i64);
            }

            let new_displayed = data.partitioning.total() as usize;
            self.notify_change(old_displayed, new_displayed);
            return true;
        }

        false
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
        if count == 0 {
            return;
        }

        let old_displayed = self.lines_displayed();
        let insert_at = doc_line.0.min(self.line_count);

        if self.one_to_one {
            self.line_count += count;
        } else {
            self.line_count += count;
            let data = self
                .data
                .as_mut()
                .expect("data must exist in non-one-to-one mode");

            // Insert into per-line arrays
            for i in 0..count {
                data.visibility.insert(insert_at + i, true);
                data.expanded.insert(insert_at + i, true);
                data.heights.insert(insert_at + i, 1);
            }

            // Insert into Fenwick tree
            data.partitioning.insert(insert_at, count, 1);
        }

        // Adjust fold_text keys
        let keys_to_adjust: Vec<usize> = self
            .fold_text
            .keys()
            .filter(|&&k| k >= insert_at)
            .copied()
            .collect();
        for key in keys_to_adjust.into_iter().rev() {
            if let Some(val) = self.fold_text.remove(&key) {
                self.fold_text.insert(key + count, val);
            }
        }

        let new_displayed = self.lines_displayed();
        self.notify_change(old_displayed, new_displayed);
    }

    fn delete_lines(&mut self, doc_line: DocLine, count: usize) {
        if count == 0 || doc_line.0 >= self.line_count {
            return;
        }

        let actual_count = count.min(self.line_count - doc_line.0);
        let old_displayed = self.lines_displayed();
        let delete_at = doc_line.0;

        if self.one_to_one {
            self.line_count -= actual_count;
        } else {
            let data = self
                .data
                .as_mut()
                .expect("data must exist in non-one-to-one mode");

            // Remove from per-line arrays
            data.visibility.drain(delete_at..delete_at + actual_count);
            data.expanded.drain(delete_at..delete_at + actual_count);
            data.heights.drain(delete_at..delete_at + actual_count);

            // Remove from Fenwick tree
            data.partitioning.remove(delete_at, actual_count);

            self.line_count -= actual_count;
        }

        // Remove fold_text entries in the deleted range and adjust keys after
        let keys_to_remove: Vec<usize> = self
            .fold_text
            .keys()
            .filter(|&&k| k >= delete_at && k < delete_at + actual_count)
            .copied()
            .collect();
        for key in &keys_to_remove {
            self.fold_text.remove(key);
        }

        let keys_to_adjust: Vec<usize> = self
            .fold_text
            .keys()
            .filter(|&&k| k >= delete_at + actual_count)
            .copied()
            .collect();
        for key in keys_to_adjust.into_iter().rev() {
            if let Some(val) = self.fold_text.remove(&key) {
                self.fold_text.insert(key - actual_count, val);
            }
        }

        let new_displayed = self.lines_displayed();
        self.notify_change(old_displayed, new_displayed);
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
