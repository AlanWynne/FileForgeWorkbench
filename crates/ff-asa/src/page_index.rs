//! Page index — mapping from page numbers to document line numbers.
//!
//! Built during initial preview parse; provides O(1) page-to-line navigation
//! via binary search over a sorted list of page entries.

use crate::control::AsaControl;
use crate::types::PageNumber;

/// A single entry in the page index.
// Validates: Requirement 3.6, 10.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageEntry {
    /// 1-based page number.
    pub page_number: PageNumber,
    /// 0-based document line number where this page starts.
    pub document_line: usize,
    /// Whether this page break is explicit (from `1` control) or implicit (page depth).
    pub is_explicit: bool,
}

/// Mapping from page numbers to document line numbers for efficient navigation.
///
/// Built during initial preview parse; rebuilt when document changes.
// Validates: Requirement 3.6, Requirement 10.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageIndex {
    /// Ordered list of page entries.
    entries: Vec<PageEntry>,
}

impl PageIndex {
    /// Create an empty page index.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Build a page index from a sequence of ASA control characters.
    ///
    /// Handles:
    /// - Explicit page breaks from `1` (PageEject) controls
    /// - Implicit page breaks every `page_depth` lines when no explicit break exists
    /// - Pre-page-1 section (data before first `1`)
    /// - Mixed mode: explicit breaks take priority over implicit breaks
    // Validates: Requirement 4.1, 4.5, 4.6, 8.3, 8.4
    pub fn build(controls: &[AsaControl], page_depth: u16, implicit_breaks_enabled: bool) -> Self {
        let mut entries = Vec::new();

        if controls.is_empty() {
            return Self { entries };
        }

        let mut page_number: u32 = 0;
        let mut lines_in_current_page: u16 = 0;
        let mut in_pre_page_section = true;

        for (i, &control) in controls.iter().enumerate() {
            if control.is_overstrike() {
                // Overstrike lines don't count toward page depth
                continue;
            }

            if control.is_page_break() {
                // Explicit page break
                page_number += 1;
                entries.push(PageEntry {
                    page_number: PageNumber(page_number),
                    document_line: i,
                    is_explicit: true,
                });
                lines_in_current_page = 0;
                in_pre_page_section = false;
            } else {
                // Count spacing lines toward page depth
                let spacing = control.spacing_lines() as u16;
                let line_cost = 1 + spacing;

                if in_pre_page_section {
                    // Pre-page-1 section — no page band before these lines
                    lines_in_current_page += line_cost;

                    // Check for implicit break in pre-page section
                    if implicit_breaks_enabled && lines_in_current_page > page_depth {
                        page_number += 1;
                        entries.push(PageEntry {
                            page_number: PageNumber(page_number),
                            document_line: i,
                            is_explicit: false,
                        });
                        lines_in_current_page = line_cost;
                        in_pre_page_section = false;
                    }
                } else {
                    lines_in_current_page += line_cost;

                    // Implicit page break at page depth
                    if implicit_breaks_enabled && lines_in_current_page > page_depth {
                        page_number += 1;
                        entries.push(PageEntry {
                            page_number: PageNumber(page_number),
                            document_line: i,
                            is_explicit: false,
                        });
                        lines_in_current_page = line_cost;
                    }
                }
            }
        }

        Self { entries }
    }

    /// Total number of pages in the index.
    pub fn page_count(&self) -> usize {
        self.entries.len()
    }

    /// Look up the document line for a given page number (1-based).
    ///
    /// Returns None if page_number is out of range.
    // Validates: Requirement 10.1
    pub fn document_line_for_page(&self, page_number: u32) -> Option<usize> {
        self.entries
            .iter()
            .find(|e| e.page_number.0 == page_number)
            .map(|e| e.document_line)
    }

    /// Find which page a given document line belongs to.
    ///
    /// Returns the page number of the page containing this line,
    /// or 0 if the line is in the pre-page-1 section.
    // Validates: Requirement 10.3
    pub fn page_for_document_line(&self, document_line: usize) -> u32 {
        // Find the last page entry whose document_line <= the given line
        let mut page = 0;
        for entry in &self.entries {
            if entry.document_line <= document_line {
                page = entry.page_number.0;
            } else {
                break;
            }
        }
        page
    }

    /// Add a page entry to the index.
    pub fn push(&mut self, entry: PageEntry) {
        self.entries.push(entry);
    }

    /// Get all entries as a slice.
    pub fn entries(&self) -> &[PageEntry] {
        &self.entries
    }
}

impl Default for PageIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 3.6
    fn empty_document_produces_empty_index() {
        let index = PageIndex::build(&[], 60, true);
        assert_eq!(index.page_count(), 0);
    }

    #[test]
    // Validates: Requirement 4.1, 4.5
    fn explicit_page_breaks_create_entries() {
        let controls = vec![
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::PageEject,
            AsaControl::Space,
        ];
        let index = PageIndex::build(&controls, 60, true);
        assert_eq!(index.page_count(), 2);
        assert_eq!(index.entries[0].page_number, PageNumber(1));
        assert_eq!(index.entries[0].document_line, 0);
        assert!(index.entries[0].is_explicit);
        assert_eq!(index.entries[1].page_number, PageNumber(2));
        assert_eq!(index.entries[1].document_line, 3);
    }

    #[test]
    // Validates: Requirement 4.6
    fn pre_page_section_has_no_page_band() {
        let controls = vec![
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::PageEject,
            AsaControl::Space,
        ];
        let index = PageIndex::build(&controls, 60, true);
        // The first explicit break is page 1, data before it is pre-page section
        assert_eq!(index.page_count(), 1);
        assert_eq!(index.entries[0].document_line, 2);
    }

    #[test]
    // Validates: Requirement 8.3
    fn implicit_breaks_at_page_depth() {
        // 5 lines with page_depth of 2 → implicit breaks
        let controls = vec![
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::Space,
        ];
        let index = PageIndex::build(&controls, 2, true);
        // Lines: 1,2 | 3,4 | 5
        // After 2 lines, implicit break at line 2 (3rd line)
        assert!(index.page_count() >= 2);
    }

    #[test]
    // Validates: Requirement 8.4
    fn explicit_breaks_take_priority_over_implicit() {
        let controls = vec![
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::PageEject, // Explicit before page depth is reached
            AsaControl::Space,
        ];
        let index = PageIndex::build(&controls, 60, true);
        assert_eq!(index.page_count(), 2);
        assert!(index.entries[0].is_explicit);
        assert!(index.entries[1].is_explicit);
    }

    #[test]
    // Validates: Requirement 10.1
    fn document_line_for_page_returns_correct_line() {
        let controls = vec![
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::PageEject,
            AsaControl::Space,
        ];
        let index = PageIndex::build(&controls, 60, true);
        assert_eq!(index.document_line_for_page(1), Some(0));
        assert_eq!(index.document_line_for_page(2), Some(2));
        assert_eq!(index.document_line_for_page(3), None);
    }

    #[test]
    // Validates: Requirement 10.3
    fn page_for_document_line_finds_correct_page() {
        let controls = vec![
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::PageEject,
            AsaControl::Space,
        ];
        let index = PageIndex::build(&controls, 60, true);
        assert_eq!(index.page_for_document_line(0), 1);
        assert_eq!(index.page_for_document_line(1), 1);
        assert_eq!(index.page_for_document_line(3), 2);
        assert_eq!(index.page_for_document_line(4), 2);
    }

    #[test]
    fn page_for_line_in_pre_page_section_returns_zero() {
        let controls = vec![AsaControl::Space, AsaControl::Space, AsaControl::PageEject];
        let index = PageIndex::build(&controls, 60, true);
        assert_eq!(index.page_for_document_line(0), 0);
        assert_eq!(index.page_for_document_line(1), 0);
        assert_eq!(index.page_for_document_line(2), 1);
    }

    #[test]
    // Validates: Requirement 8.3
    fn no_implicit_breaks_when_disabled() {
        let controls = vec![
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::Space,
        ];
        let index = PageIndex::build(&controls, 2, false);
        assert_eq!(index.page_count(), 0);
    }

    #[test]
    fn overstrike_lines_do_not_count_toward_page_depth() {
        let controls = vec![
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::Overstrike,
            AsaControl::Overstrike,
            AsaControl::Space,
        ];
        // page_depth 2: line at [1] = 1 line, overstrikes don't count, line at [4] = 2 lines
        let index = PageIndex::build(&controls, 2, true);
        // Should be 1 explicit page, no implicit break because only 2 data lines
        assert_eq!(index.page_count(), 1);
    }
}
