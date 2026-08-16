//! Viewport position management with clamped scroll arithmetic.
//!
//! Maintains `top_line` (1-based) and provides scroll operations that
//! always clamp to valid ranges.

/// Viewport position manager.
///
/// Tracks the 1-based top_line and provides clamped scroll operations.
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Current top line (1-based).
    top_line: u64,
}

impl Viewport {
    /// Create a new viewport at line 1.
    pub fn new() -> Self {
        Self { top_line: 1 }
    }

    /// Get the current top line (1-based).
    pub fn top_line(&self) -> u64 {
        self.top_line
    }

    /// Scroll down by one page (visible_count lines).
    pub fn scroll_page_down(&mut self, visible_count: u64, line_count: u64) {
        let max = Self::compute_max_top_line(line_count, visible_count);
        self.top_line = (self.top_line + visible_count).min(max);
    }

    /// Scroll up by one page (visible_count lines).
    pub fn scroll_page_up(&mut self, visible_count: u64) {
        self.top_line = self.top_line.saturating_sub(visible_count).max(1);
    }

    /// Scroll down by count lines.
    pub fn scroll_line_down(&mut self, count: u64, line_count: u64, visible_count: u64) {
        let max = Self::compute_max_top_line(line_count, visible_count);
        self.top_line = (self.top_line + count).min(max);
    }

    /// Scroll up by count lines.
    pub fn scroll_line_up(&mut self, count: u64) {
        self.top_line = self.top_line.saturating_sub(count).max(1);
    }

    /// Set top_line to a specific value, clamped to valid range.
    pub fn set_top_line(&mut self, line: u64, line_count: u64, visible_count: u64) {
        let max = Self::compute_max_top_line(line_count, visible_count);
        self.top_line = line.max(1).min(max);
    }

    /// Maximum valid top_line for a given viewport height.
    /// Computed as max(1, line_count - visible_count + 1).
    pub fn compute_max_top_line(line_count: u64, visible_count: u64) -> u64 {
        if visible_count >= line_count {
            1
        } else {
            line_count - visible_count + 1
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_top_line_is_one() {
        let vp = Viewport::new();
        assert_eq!(vp.top_line(), 1);
    }

    #[test]
    fn scroll_page_down_advances_by_visible_count() {
        let mut vp = Viewport::new();
        vp.scroll_page_down(20, 100);
        assert_eq!(vp.top_line(), 21);
    }

    #[test]
    fn scroll_page_down_clamps_to_max() {
        let mut vp = Viewport::new();
        vp.set_top_line(90, 100, 20);
        vp.scroll_page_down(20, 100);
        // max_top_line = 100 - 20 + 1 = 81
        assert_eq!(vp.top_line(), 81);
    }

    #[test]
    fn scroll_page_up_from_beginning_stays_at_one() {
        let mut vp = Viewport::new();
        vp.scroll_page_up(20);
        assert_eq!(vp.top_line(), 1);
    }

    #[test]
    fn scroll_page_up_retreats_by_visible_count() {
        let mut vp = Viewport::new();
        vp.set_top_line(50, 100, 20);
        vp.scroll_page_up(20);
        assert_eq!(vp.top_line(), 30);
    }

    #[test]
    fn scroll_line_down_with_clamping() {
        let mut vp = Viewport::new();
        vp.scroll_line_down(5, 100, 20);
        assert_eq!(vp.top_line(), 6);

        // Scroll past max
        vp.set_top_line(80, 100, 20);
        vp.scroll_line_down(10, 100, 20);
        assert_eq!(vp.top_line(), 81); // max = 100 - 20 + 1 = 81
    }

    #[test]
    fn scroll_line_up_with_clamping() {
        let mut vp = Viewport::new();
        vp.set_top_line(10, 100, 20);
        vp.scroll_line_up(5);
        assert_eq!(vp.top_line(), 5);

        // Past beginning
        vp.scroll_line_up(10);
        assert_eq!(vp.top_line(), 1);
    }

    #[test]
    fn set_top_line_clamps_to_range() {
        let mut vp = Viewport::new();
        vp.set_top_line(0, 100, 20);
        assert_eq!(vp.top_line(), 1);

        vp.set_top_line(200, 100, 20);
        assert_eq!(vp.top_line(), 81);
    }

    #[test]
    fn max_top_line_single_page_document() {
        // Document with fewer lines than viewport
        assert_eq!(Viewport::compute_max_top_line(10, 20), 1);
    }

    #[test]
    fn idempotent_scroll_at_boundaries() {
        let mut vp = Viewport::new();
        // At top boundary
        vp.scroll_page_up(20);
        assert_eq!(vp.top_line(), 1);
        vp.scroll_page_up(20);
        assert_eq!(vp.top_line(), 1);

        // At bottom boundary
        vp.set_top_line(81, 100, 20);
        let before = vp.top_line();
        vp.scroll_page_down(20, 100);
        assert_eq!(vp.top_line(), before); // Already at max, no change
    }
}
