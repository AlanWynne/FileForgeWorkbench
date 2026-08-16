//! COLS command implementation.
//!
//! Manages COLS_Line display artifacts — non-editable column ruler overlays
//! that help users identify column positions in fixed-width data.

use crate::types::{ColsLine, ColsToggleResult};

/// The standard COLS ruler pattern.
const COLS_RULER: &str =
    "----+----1----+----2----+----3----+----4----+----5----+----6----+----7----+----8";

/// Manages all COLS_Line display artifacts for a session.
#[derive(Debug, Clone)]
pub struct ColsManager {
    /// Active COLS_Lines ordered by anchor position.
    cols_lines: Vec<ColsLine>,
    /// Next unique ID for new COLS_Lines.
    next_id: u64,
}

impl ColsManager {
    /// Create with no active COLS_Lines.
    pub fn new() -> Self {
        Self {
            cols_lines: Vec::new(),
            next_id: 1,
        }
    }

    /// Insert a COLS_Line at the given anchor position (or toggle off if already present).
    ///
    /// If a COLS_Line already exists at the same anchor position, it is removed (toggle).
    /// Otherwise, a new COLS_Line is inserted.
    pub fn toggle_at(&mut self, anchor_line: u64) -> ColsToggleResult {
        if let Some(idx) = self
            .cols_lines
            .iter()
            .position(|c| c.anchor_line == anchor_line)
        {
            let removed = self.cols_lines.remove(idx);
            ColsToggleResult::Removed(removed.id)
        } else {
            let id = self.next_id;
            self.next_id += 1;
            let cols_line = ColsLine { anchor_line, id };
            // Insert maintaining sorted order by anchor_line
            let insert_pos = self
                .cols_lines
                .partition_point(|c| c.anchor_line < anchor_line);
            self.cols_lines.insert(insert_pos, cols_line.clone());
            ColsToggleResult::Inserted(cols_line)
        }
    }

    /// Insert a COLS_Line above a specific document line (from line command).
    pub fn insert_above(&mut self, doc_line: u64) {
        // Only insert if not already present at this position
        if !self.cols_lines.iter().any(|c| c.anchor_line == doc_line) {
            let id = self.next_id;
            self.next_id += 1;
            let cols_line = ColsLine {
                anchor_line: doc_line,
                id,
            };
            let insert_pos = self
                .cols_lines
                .partition_point(|c| c.anchor_line < doc_line);
            self.cols_lines.insert(insert_pos, cols_line);
        }
    }

    /// Remove all COLS_Lines (RESET command).
    pub fn reset_all(&mut self) {
        self.cols_lines.clear();
    }

    /// Query all active COLS_Lines.
    pub fn active_cols_lines(&self) -> &[ColsLine] {
        &self.cols_lines
    }

    /// Format the COLS ruler string.
    pub fn format_ruler() -> &'static str {
        COLS_RULER
    }
}

impl Default for ColsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_cols_line() {
        // Validates: Requirement 4.1
        let mut mgr = ColsManager::new();
        let result = mgr.toggle_at(5);
        assert!(matches!(result, ColsToggleResult::Inserted(_)));
        assert_eq!(mgr.active_cols_lines().len(), 1);
        assert_eq!(mgr.active_cols_lines()[0].anchor_line, 5);
    }

    #[test]
    fn toggle_removes_existing() {
        // Validates: Requirement 4.4
        let mut mgr = ColsManager::new();
        mgr.toggle_at(5);
        let result = mgr.toggle_at(5);
        assert!(matches!(result, ColsToggleResult::Removed(_)));
        assert!(mgr.active_cols_lines().is_empty());
    }

    #[test]
    fn multiple_cols_at_different_positions() {
        // Validates: Requirement 4.8
        let mut mgr = ColsManager::new();
        mgr.toggle_at(5);
        mgr.toggle_at(10);
        mgr.toggle_at(3);
        assert_eq!(mgr.active_cols_lines().len(), 3);
        // Should be sorted by anchor_line
        assert_eq!(mgr.active_cols_lines()[0].anchor_line, 3);
        assert_eq!(mgr.active_cols_lines()[1].anchor_line, 5);
        assert_eq!(mgr.active_cols_lines()[2].anchor_line, 10);
    }

    #[test]
    fn reset_clears_all() {
        // Validates: Requirement 4.5, 4.10
        let mut mgr = ColsManager::new();
        mgr.toggle_at(5);
        mgr.toggle_at(10);
        mgr.reset_all();
        assert!(mgr.active_cols_lines().is_empty());
    }

    #[test]
    fn format_ruler_pattern() {
        // Validates: Requirement 4.2
        let ruler = ColsManager::format_ruler();
        assert!(ruler.contains("----+----1"));
        assert!(ruler.contains("----+----8"));
    }

    #[test]
    fn insert_above_adds_at_position() {
        // Validates: Requirement 4.7
        let mut mgr = ColsManager::new();
        mgr.insert_above(7);
        assert_eq!(mgr.active_cols_lines().len(), 1);
        assert_eq!(mgr.active_cols_lines()[0].anchor_line, 7);
    }

    #[test]
    fn insert_above_does_not_duplicate() {
        let mut mgr = ColsManager::new();
        mgr.insert_above(7);
        mgr.insert_above(7);
        assert_eq!(mgr.active_cols_lines().len(), 1);
    }
}
