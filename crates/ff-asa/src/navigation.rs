//! Preview navigation — page location and viewport control.
//!
//! Provides commands for navigating the preview by page number,
//! including LOCATE PAGE, UP PAGE, DOWN PAGE, and status bar queries.

use crate::error::AsaError;
use crate::page_index::PageIndex;

/// Navigate to a specific page in the preview.
///
/// Returns the document line number for the start of the requested page.
///
/// # Errors
///
/// Returns `AsaError::PageNotFound` if the page number is out of range.
// Validates: Requirement 10.1
pub fn locate_page(page_index: &PageIndex, page_number: u32) -> Result<usize, AsaError> {
    page_index
        .document_line_for_page(page_number)
        .ok_or(AsaError::PageNotFound {
            page: page_number as usize,
            total: page_index.page_count(),
        })
}

/// Navigate to the next page relative to current position.
///
/// Returns the document line for the next page, or None if already at last page.
// Validates: Requirement 10.4
pub fn next_page(page_index: &PageIndex, current_page: u32) -> Option<usize> {
    page_index.document_line_for_page(current_page + 1)
}

/// Navigate to the previous page relative to current position.
///
/// Returns the document line for the previous page, or None if already at first page.
// Validates: Requirement 10.4
pub fn previous_page(page_index: &PageIndex, current_page: u32) -> Option<usize> {
    if current_page <= 1 {
        return None;
    }
    page_index.document_line_for_page(current_page - 1)
}

/// Navigate to the first page.
// Validates: Requirement 10.6
pub fn first_page(page_index: &PageIndex) -> Option<usize> {
    page_index.document_line_for_page(1)
}

/// Navigate to the last page.
// Validates: Requirement 10.6
pub fn last_page(page_index: &PageIndex) -> Option<usize> {
    let count = page_index.page_count();
    if count == 0 {
        return None;
    }
    page_index.document_line_for_page(count as u32)
}

/// Format the status bar page indicator text.
///
/// Returns text like `"Preview: Page 3 of 47"`.
// Validates: Requirement 10.3
pub fn format_page_indicator(current_page: u32, total_pages: usize) -> String {
    format!("Preview: Page {} of {}", current_page, total_pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::AsaControl;

    fn build_test_index() -> PageIndex {
        let controls = vec![
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::Space,
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::PageEject,
            AsaControl::Space,
        ];
        PageIndex::build(&controls, 60, true)
    }

    #[test]
    // Validates: Requirement 10.1
    fn locate_page_returns_correct_line() {
        let index = build_test_index();
        assert_eq!(locate_page(&index, 1).unwrap(), 0);
        assert_eq!(locate_page(&index, 2).unwrap(), 3);
        assert_eq!(locate_page(&index, 3).unwrap(), 5);
    }

    #[test]
    // Validates: Requirement 10.2
    fn locate_page_out_of_range_returns_error() {
        let index = build_test_index();
        let err = locate_page(&index, 99).unwrap_err();
        assert!(err.to_string().contains("page 99 not found"));
        assert!(err.to_string().contains("3 pages"));
    }

    #[test]
    // Validates: Requirement 10.4
    fn next_page_navigates_forward() {
        let index = build_test_index();
        assert_eq!(next_page(&index, 1), Some(3));
        assert_eq!(next_page(&index, 2), Some(5));
        assert_eq!(next_page(&index, 3), None);
    }

    #[test]
    // Validates: Requirement 10.4
    fn previous_page_navigates_backward() {
        let index = build_test_index();
        assert_eq!(previous_page(&index, 3), Some(3));
        assert_eq!(previous_page(&index, 2), Some(0));
        assert_eq!(previous_page(&index, 1), None);
    }

    #[test]
    // Validates: Requirement 10.6
    fn first_and_last_page_navigation() {
        let index = build_test_index();
        assert_eq!(first_page(&index), Some(0));
        assert_eq!(last_page(&index), Some(5));
    }

    #[test]
    fn first_page_on_empty_index_returns_none() {
        let index = PageIndex::new();
        assert_eq!(first_page(&index), None);
        assert_eq!(last_page(&index), None);
    }

    #[test]
    // Validates: Requirement 10.3
    fn format_page_indicator_produces_correct_text() {
        assert_eq!(format_page_indicator(3, 47), "Preview: Page 3 of 47");
        assert_eq!(format_page_indicator(1, 1), "Preview: Page 1 of 1");
    }
}
