//! Extracted inherent helpers for the heavier ContractionState operations.
//!
//! These are `pub(super)` helpers that the thin `DisplayLineMapping` trait
//! methods in `mod.rs` delegate to (a trait impl cannot span files, so the
//! heavy logic lives here and the trait methods are thin wrappers).

use crate::traits::DisplayLineMapping;
use crate::types::{DisplayLine, DocLine, DocPosition, SubLine};

use super::ContractionState;

impl ContractionState {
    pub(super) fn doc_from_display_impl(&self, display_line: DisplayLine) -> DocPosition {
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
            // All lines hidden -- return first line
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

    pub(super) fn set_visible_impl(&mut self, start: DocLine, end: DocLine, visible: bool) -> bool {
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

    pub(super) fn set_expanded_impl(&mut self, doc_line: DocLine, expanded: bool) -> bool {
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

    pub(super) fn set_height_impl(&mut self, doc_line: DocLine, height: u32) -> bool {
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

    pub(super) fn insert_lines_impl(&mut self, doc_line: DocLine, count: usize) {
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

    pub(super) fn delete_lines_impl(&mut self, doc_line: DocLine, count: usize) {
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
}
