//! Fold-level storage: per-line fold level and flags synchronized with document line count.

use crate::types::{FoldFlags, FoldLevel, LineNumber};

/// Per-line fold level and flags storage.
/// Addresses: Requirement 8, criterion 8.5
pub struct FoldData {
    /// (level, flags) for each line.
    data: Vec<(FoldLevel, FoldFlags)>,
}

impl FoldData {
    /// Create fold data for the given number of lines, initialized to level 0, no flags.
    pub fn new(line_count: usize) -> Self {
        Self {
            data: vec![(FoldLevel::MIN, FoldFlags::NONE); line_count],
        }
    }

    /// Get the fold level and flags for a specific line.
    /// Addresses: Requirement 8, criterion 8.5
    pub fn fold_level_at(&self, line: LineNumber) -> (FoldLevel, FoldFlags) {
        self.data
            .get(line.0)
            .copied()
            .unwrap_or((FoldLevel::MIN, FoldFlags::NONE))
    }

    /// Set the fold level and flags for a specific line.
    pub fn set_level(&mut self, line: LineNumber, level: FoldLevel, flags: FoldFlags) {
        if line.0 < self.data.len() {
            self.data[line.0] = (level, flags);
        }
    }

    /// Get fold levels for a range of lines (bulk query).
    /// Addresses: Requirement 15, criterion 15.6
    pub fn fold_level_range(
        &self,
        start_line: LineNumber,
        end_line: LineNumber,
    ) -> Vec<(LineNumber, FoldLevel, FoldFlags)> {
        let start = start_line.0.min(self.data.len());
        let end = end_line.0.min(self.data.len());
        (start..end)
            .map(|i| {
                let (level, flags) = self.data[i];
                (LineNumber(i), level, flags)
            })
            .collect()
    }

    /// Insert default entries for new lines.
    pub fn insert_lines(&mut self, at: LineNumber, count: usize) {
        let pos = at.0.min(self.data.len());
        self.data.splice(
            pos..pos,
            std::iter::repeat_n((FoldLevel::MIN, FoldFlags::NONE), count),
        );
    }

    /// Remove entries for deleted lines.
    pub fn delete_lines(&mut self, at: LineNumber, count: usize) {
        let pos = at.0.min(self.data.len());
        let end = (pos + count).min(self.data.len());
        self.data.drain(pos..end);
    }

    /// Get total number of lines tracked.
    pub fn line_count(&self) -> usize {
        self.data.len()
    }

    /// Apply FOLD_HEADER auto-marking based on level relationships.
    /// A line gets FOLD_HEADER if its level > next line's level and line has visible content.
    /// Addresses: Requirement 8, criterion 8.4
    pub fn apply_fold_headers(&mut self, line_has_content: &[bool]) {
        if self.data.is_empty() {
            return;
        }
        let len = self.data.len();
        for i in 0..len {
            self.data[i].1.remove(FoldFlags::FOLD_HEADER);

            let level = self.data[i].0;
            if i + 1 < len {
                let next_level = self.data[i + 1].0;
                let has_content = line_has_content.get(i).copied().unwrap_or(false);
                if level > next_level && has_content {
                    self.data[i].1.insert(FoldFlags::FOLD_HEADER);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initializes_to_zero_level_no_flags() {
        let fd = FoldData::new(5);
        assert_eq!(fd.line_count(), 5);
        for i in 0..5 {
            let (level, flags) = fd.fold_level_at(LineNumber(i));
            assert_eq!(level, FoldLevel::MIN);
            assert_eq!(flags, FoldFlags::NONE);
        }
    }

    #[test]
    fn set_and_get_level() {
        // Validates: Requirement 8, criterion 8.5
        let mut fd = FoldData::new(5);
        fd.set_level(LineNumber(2), FoldLevel::new(3), FoldFlags::FOLD_HEADER);
        let (level, flags) = fd.fold_level_at(LineNumber(2));
        assert_eq!(level.value(), 3);
        assert!(flags.contains(FoldFlags::FOLD_HEADER));
    }

    #[test]
    fn fold_level_range_returns_bulk_data() {
        // Validates: Requirement 15, criterion 15.6
        let mut fd = FoldData::new(5);
        fd.set_level(LineNumber(1), FoldLevel::new(1), FoldFlags::NONE);
        fd.set_level(LineNumber(2), FoldLevel::new(2), FoldFlags::FOLD_HEADER);
        fd.set_level(LineNumber(3), FoldLevel::new(1), FoldFlags::NONE);

        let range = fd.fold_level_range(LineNumber(1), LineNumber(4));
        assert_eq!(range.len(), 3);
        assert_eq!(
            range[0],
            (LineNumber(1), FoldLevel::new(1), FoldFlags::NONE)
        );
        assert_eq!(
            range[1],
            (LineNumber(2), FoldLevel::new(2), FoldFlags::FOLD_HEADER)
        );
        assert_eq!(
            range[2],
            (LineNumber(3), FoldLevel::new(1), FoldFlags::NONE)
        );
    }

    #[test]
    fn insert_lines_adds_entries() {
        let mut fd = FoldData::new(3);
        fd.set_level(LineNumber(0), FoldLevel::new(1), FoldFlags::NONE);
        fd.insert_lines(LineNumber(1), 2);
        assert_eq!(fd.line_count(), 5);
        assert_eq!(fd.fold_level_at(LineNumber(0)).0.value(), 1);
        assert_eq!(fd.fold_level_at(LineNumber(1)).0.value(), 0);
        assert_eq!(fd.fold_level_at(LineNumber(2)).0.value(), 0);
    }

    #[test]
    fn delete_lines_removes_entries() {
        let mut fd = FoldData::new(5);
        fd.set_level(LineNumber(2), FoldLevel::new(5), FoldFlags::NONE);
        fd.delete_lines(LineNumber(1), 2);
        assert_eq!(fd.line_count(), 3);
        // Line 2 (was at index 3) is now at index 1
        assert_eq!(fd.fold_level_at(LineNumber(1)).0.value(), 0);
    }

    #[test]
    fn apply_fold_headers_marks_correctly() {
        // Validates: Requirement 8, criterion 8.4
        let mut fd = FoldData::new(4);
        fd.set_level(LineNumber(0), FoldLevel::new(1), FoldFlags::NONE);
        fd.set_level(LineNumber(1), FoldLevel::new(2), FoldFlags::NONE);
        fd.set_level(LineNumber(2), FoldLevel::new(1), FoldFlags::NONE);
        fd.set_level(LineNumber(3), FoldLevel::new(0), FoldFlags::NONE);

        // Lines 0, 1, 2 have content; line 3 is whitespace-only
        let content = vec![true, true, true, false];
        fd.apply_fold_headers(&content);

        // Line 1 (level 2 > level 1 at line 2) should be FOLD_HEADER
        assert!(fd
            .fold_level_at(LineNumber(1))
            .1
            .contains(FoldFlags::FOLD_HEADER));
        // Line 2 (level 1 > level 0 at line 3) should be FOLD_HEADER
        assert!(fd
            .fold_level_at(LineNumber(2))
            .1
            .contains(FoldFlags::FOLD_HEADER));
        // Line 0 (level 1 < level 2 at line 1) should NOT be FOLD_HEADER
        assert!(!fd
            .fold_level_at(LineNumber(0))
            .1
            .contains(FoldFlags::FOLD_HEADER));
    }

    #[test]
    fn apply_fold_headers_whitespace_only_lines_never_get_header() {
        // Validates: Requirement 8, criterion 8.4
        let mut fd = FoldData::new(3);
        fd.set_level(LineNumber(0), FoldLevel::new(2), FoldFlags::NONE);
        fd.set_level(LineNumber(1), FoldLevel::new(1), FoldFlags::NONE);
        fd.set_level(LineNumber(2), FoldLevel::new(0), FoldFlags::NONE);

        // Line 0 is whitespace-only
        let content = vec![false, true, true];
        fd.apply_fold_headers(&content);

        // Line 0 has higher level than line 1 but is whitespace-only
        assert!(!fd
            .fold_level_at(LineNumber(0))
            .1
            .contains(FoldFlags::FOLD_HEADER));
        // Line 1 has higher level than line 2 and has content
        assert!(fd
            .fold_level_at(LineNumber(1))
            .1
            .contains(FoldFlags::FOLD_HEADER));
    }

    #[test]
    fn fold_level_at_out_of_range_returns_defaults() {
        let fd = FoldData::new(3);
        let (level, flags) = fd.fold_level_at(LineNumber(100));
        assert_eq!(level, FoldLevel::MIN);
        assert_eq!(flags, FoldFlags::NONE);
    }
}
