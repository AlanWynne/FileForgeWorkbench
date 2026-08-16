//! Print preview panel state and logic.
//!
//! GUI-independent state management for the paginated print preview panel.
//! Provides page navigation, zoom, and source mapping without knowledge
//! of the UI framework.

/// GUI-independent state for the print preview panel.
///
/// Drives rendering without knowledge of the UI framework.
// Validates: Requirement 6.1–6.8
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewPanelState {
    /// Currently displayed page number (1-based).
    pub current_page: u32,
    /// Total page count.
    pub total_pages: u32,
    /// Current zoom level as a percentage (50–200, default 100).
    pub zoom_percent: u32,
    /// Whether the panel is currently visible/docked.
    pub is_visible: bool,
    /// Page width in characters for layout calculation.
    pub page_width: u16,
    /// Page depth in lines for layout calculation.
    pub page_depth: u16,
}

impl PreviewPanelState {
    /// Create a new panel state with default settings.
    pub fn new(total_pages: u32, page_width: u16, page_depth: u16) -> Self {
        Self {
            current_page: if total_pages > 0 { 1 } else { 0 },
            total_pages,
            zoom_percent: 100,
            is_visible: false,
            page_width,
            page_depth,
        }
    }

    /// Navigate to a specific page. Returns false if page is out of range.
    // Validates: Requirement 6.4
    pub fn go_to_page(&mut self, page: u32) -> bool {
        if page >= 1 && page <= self.total_pages {
            self.current_page = page;
            true
        } else {
            false
        }
    }

    /// Navigate to the next page. Returns false if already at last page.
    pub fn next_page(&mut self) -> bool {
        if self.current_page < self.total_pages {
            self.current_page += 1;
            true
        } else {
            false
        }
    }

    /// Navigate to the previous page. Returns false if already at first page.
    pub fn previous_page(&mut self) -> bool {
        if self.current_page > 1 {
            self.current_page -= 1;
            true
        } else {
            false
        }
    }

    /// Navigate to the first page.
    pub fn first_page(&mut self) {
        self.current_page = 1;
    }

    /// Navigate to the last page.
    pub fn last_page(&mut self) {
        self.current_page = self.total_pages;
    }

    /// Set zoom level, clamped to [50, 200].
    // Validates: Requirement 6.7
    pub fn set_zoom(&mut self, percent: u32) {
        self.zoom_percent = percent.clamp(50, 200);
    }

    /// Format the page header text.
    // Validates: Requirement 6.3
    pub fn header_text(&self) -> String {
        format!("Page {} of {}", self.current_page, self.total_pages)
    }

    /// Whether the "previous page" action is available.
    pub fn can_go_previous(&self) -> bool {
        self.current_page > 1
    }

    /// Whether the "next page" action is available.
    pub fn can_go_next(&self) -> bool {
        self.current_page < self.total_pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 6.4
    fn go_to_page_within_range_succeeds() {
        let mut state = PreviewPanelState::new(10, 132, 60);
        assert!(state.go_to_page(5));
        assert_eq!(state.current_page, 5);
    }

    #[test]
    fn go_to_page_out_of_range_fails() {
        let mut state = PreviewPanelState::new(10, 132, 60);
        assert!(!state.go_to_page(0));
        assert!(!state.go_to_page(11));
        assert_eq!(state.current_page, 1);
    }

    #[test]
    fn next_page_stops_at_last() {
        let mut state = PreviewPanelState::new(3, 132, 60);
        assert!(state.next_page());
        assert_eq!(state.current_page, 2);
        assert!(state.next_page());
        assert_eq!(state.current_page, 3);
        assert!(!state.next_page());
        assert_eq!(state.current_page, 3);
    }

    #[test]
    fn previous_page_stops_at_first() {
        let mut state = PreviewPanelState::new(3, 132, 60);
        state.go_to_page(3);
        assert!(state.previous_page());
        assert_eq!(state.current_page, 2);
        assert!(state.previous_page());
        assert_eq!(state.current_page, 1);
        assert!(!state.previous_page());
    }

    #[test]
    // Validates: Requirement 6.7
    fn set_zoom_clamps_to_range() {
        let mut state = PreviewPanelState::new(5, 132, 60);
        state.set_zoom(30);
        assert_eq!(state.zoom_percent, 50);
        state.set_zoom(300);
        assert_eq!(state.zoom_percent, 200);
        state.set_zoom(150);
        assert_eq!(state.zoom_percent, 150);
    }

    #[test]
    // Validates: Requirement 6.3
    fn header_text_format() {
        let state = PreviewPanelState::new(47, 132, 60);
        assert_eq!(state.header_text(), "Page 1 of 47");
    }

    #[test]
    fn can_go_navigation_flags() {
        let mut state = PreviewPanelState::new(3, 132, 60);
        assert!(!state.can_go_previous());
        assert!(state.can_go_next());
        state.go_to_page(3);
        assert!(state.can_go_previous());
        assert!(!state.can_go_next());
    }
}
