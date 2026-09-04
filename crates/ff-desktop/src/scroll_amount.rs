//! SCROLL ===> field value model.
//!
//! Represents the current scroll amount displayed in the SCROLL ===> field.
//! Validates: Requirement 19.1, 19.2, 19.3, 19.10

/// The active scroll amount for a panel.
///
/// Validates: Requirement 19.10 -- HALF, CSR, MAX, DATA supported in addition
/// to PAGE and numeric values for all scroll commands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScrollAmount {
    /// Scroll one full page (default).
    #[default]
    Page,
    /// Scroll half a page.
    Half,
    /// Scroll to the cursor position.
    Csr,
    /// Scroll to the maximum extent.
    Max,
    /// Scroll one data screen (same as Page for most panels).
    Data,
    /// Scroll a specific number of lines/columns.
    Lines(u64),
}

impl ScrollAmount {
    /// Parse a scroll amount from a string (case-insensitive).
    ///
    /// Returns `None` if the string is not a recognised scroll amount.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "PAGE" | "" => Some(ScrollAmount::Page),
            "HALF" => Some(ScrollAmount::Half),
            "CSR" => Some(ScrollAmount::Csr),
            "MAX" => Some(ScrollAmount::Max),
            "DATA" => Some(ScrollAmount::Data),
            other => other.parse::<u64>().ok().map(ScrollAmount::Lines),
        }
    }

    /// Return the display string for the SCROLL ===> field.
    pub fn display(&self) -> &str {
        match self {
            ScrollAmount::Page => "PAGE",
            ScrollAmount::Half => "HALF",
            ScrollAmount::Csr => "CSR",
            ScrollAmount::Max => "MAX",
            ScrollAmount::Data => "DATA",
            ScrollAmount::Lines(_) => "n",
        }
    }

    /// Return the display string including numeric value when applicable.
    pub fn display_string(&self) -> String {
        match self {
            ScrollAmount::Lines(n) => n.to_string(),
            other => other.display().to_string(),
        }
    }

    /// Convert to a line count for scroll operations.
    ///
    /// `page_lines` is the number of visible lines in the current viewport.
    #[allow(dead_code)]
    pub fn to_line_count(&self, page_lines: u64) -> u64 {
        match self {
            ScrollAmount::Page | ScrollAmount::Data => page_lines,
            ScrollAmount::Half => (page_lines / 2).max(1),
            ScrollAmount::Csr => 1,
            ScrollAmount::Max => u64::MAX,
            ScrollAmount::Lines(n) => *n,
        }
    }
}

// === Split screen state =====================================================

/// State for the split-screen mode (PF2/PF9/PF3).
///
/// Validates: Requirement 19.11, 19.12, 19.13, 19.14
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SplitScreenState {
    /// The line at which the screen was split (0-based).
    pub split_line: usize,
    /// Which half currently has focus (0 = top, 1 = bottom).
    pub active_half: usize,
    /// Scroll offset for the top half.
    pub top_scroll: usize,
    /// Scroll offset for the bottom half.
    pub bottom_scroll: usize,
    /// Cursor line in the top half.
    pub top_cursor: usize,
    /// Cursor line in the bottom half.
    pub bottom_cursor: usize,
}

impl SplitScreenState {
    /// Create a new split at the given line.
    pub fn new(split_line: usize) -> Self {
        Self {
            split_line,
            active_half: 0,
            top_scroll: 0,
            bottom_scroll: split_line,
            top_cursor: 0,
            bottom_cursor: split_line,
        }
    }

    /// Swap focus between the two halves.
    ///
    /// Validates: Requirement 19.12
    pub fn swap_focus(&mut self) {
        self.active_half = 1 - self.active_half;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_page() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::parse("PAGE"), Some(ScrollAmount::Page));
        assert_eq!(ScrollAmount::parse("page"), Some(ScrollAmount::Page));
        assert_eq!(ScrollAmount::parse(""), Some(ScrollAmount::Page));
    }

    #[test]
    fn parse_half() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::parse("HALF"), Some(ScrollAmount::Half));
        assert_eq!(ScrollAmount::parse("half"), Some(ScrollAmount::Half));
    }

    #[test]
    fn parse_csr() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::parse("CSR"), Some(ScrollAmount::Csr));
    }

    #[test]
    fn parse_max() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::parse("MAX"), Some(ScrollAmount::Max));
    }

    #[test]
    fn parse_data() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::parse("DATA"), Some(ScrollAmount::Data));
    }

    #[test]
    fn parse_numeric() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::parse("10"), Some(ScrollAmount::Lines(10)));
        assert_eq!(ScrollAmount::parse("1"), Some(ScrollAmount::Lines(1)));
    }

    #[test]
    fn parse_invalid_returns_none() {
        // Validates: Requirement 19.2 -- invalid value rejected
        assert_eq!(ScrollAmount::parse("BOGUS"), None);
        assert_eq!(ScrollAmount::parse("abc"), None);
    }

    #[test]
    fn display_string_numeric() {
        // Validates: Requirement 19.1
        assert_eq!(ScrollAmount::Lines(25).display_string(), "25");
    }

    #[test]
    fn display_string_named() {
        // Validates: Requirement 19.1
        assert_eq!(ScrollAmount::Half.display_string(), "HALF");
        assert_eq!(ScrollAmount::Page.display_string(), "PAGE");
    }

    #[test]
    fn to_line_count_page() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::Page.to_line_count(24), 24);
    }

    #[test]
    fn to_line_count_half() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::Half.to_line_count(24), 12);
        assert_eq!(ScrollAmount::Half.to_line_count(1), 1); // clamp to 1
    }

    #[test]
    fn to_line_count_csr() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::Csr.to_line_count(24), 1);
    }

    #[test]
    fn to_line_count_max() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::Max.to_line_count(24), u64::MAX);
    }

    #[test]
    fn to_line_count_lines() {
        // Validates: Requirement 19.10
        assert_eq!(ScrollAmount::Lines(7).to_line_count(24), 7);
    }

    #[test]
    fn default_is_page() {
        // Validates: Requirement 19.1 -- default scroll amount is PAGE
        assert_eq!(ScrollAmount::default(), ScrollAmount::Page);
    }

    #[test]
    fn split_screen_new_sets_split_line() {
        // Validates: Requirement 19.11
        let s = SplitScreenState::new(12);
        assert_eq!(s.split_line, 12);
        assert_eq!(s.active_half, 0);
        assert_eq!(s.bottom_scroll, 12);
    }

    #[test]
    fn split_screen_swap_focus() {
        // Validates: Requirement 19.12
        let mut s = SplitScreenState::new(12);
        assert_eq!(s.active_half, 0);
        s.swap_focus();
        assert_eq!(s.active_half, 1);
        s.swap_focus();
        assert_eq!(s.active_half, 0);
    }
}
