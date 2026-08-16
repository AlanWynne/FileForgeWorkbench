//! Hex viewport adapter.
//!
//! Adapts the hex row model to the viewport-and-scrolling system.
//! Provides row-based scrolling, page calculations, and scroll clamping.

use crate::layout::HexLayout;
use crate::types::BytesPerRow;

/// Adapts the hex row model to the viewport-and-scrolling system.
///
/// Provides row-based scrolling, page calculations, and scroll clamping
/// for the hex display mode viewport.
#[derive(Debug, Clone)]
pub struct HexViewportAdapter {
    /// Current top visible row.
    top_row: u64,
    /// Number of visible rows in the viewport.
    visible_rows: u64,
    /// Total rows in the document (computed from document length / bytes_per_row).
    total_rows: u64,
}

impl HexViewportAdapter {
    /// Create a new viewport adapter.
    pub fn new(total_rows: u64, visible_rows: u64) -> Self {
        Self {
            top_row: 0,
            visible_rows,
            total_rows,
        }
    }

    /// Recalculate total rows when document length or bytes_per_row changes.
    pub fn recalculate(&mut self, document_byte_length: u64, bytes_per_row: BytesPerRow) {
        self.total_rows = if document_byte_length == 0 {
            1
        } else {
            document_byte_length.div_ceil(bytes_per_row.as_u64())
        };
        // Clamp top_row if it's now out of range
        self.clamp_top_row();
    }

    /// Scroll down by one page.
    pub fn page_down(&mut self) {
        self.top_row = self.top_row.saturating_add(self.visible_rows);
        self.clamp_top_row();
    }

    /// Scroll up by one page.
    pub fn page_up(&mut self) {
        self.top_row = self.top_row.saturating_sub(self.visible_rows);
    }

    /// Scroll to ensure the given row is visible.
    pub fn ensure_row_visible(&mut self, row: u64) {
        if row < self.top_row {
            self.top_row = row;
        } else if row >= self.top_row + self.visible_rows {
            self.top_row = row.saturating_sub(self.visible_rows - 1);
        }
        self.clamp_top_row();
    }

    /// Set viewport size (on resize).
    pub fn set_visible_rows(&mut self, count: u64) {
        self.visible_rows = count.max(1);
        self.clamp_top_row();
    }

    /// Get scrollbar position as a fraction [0.0, 1.0].
    pub fn scrollbar_fraction(&self) -> f64 {
        let max_top = self.max_top_row();
        if max_top == 0 {
            0.0
        } else {
            self.top_row as f64 / max_top as f64
        }
    }

    /// Set top row from scrollbar fraction.
    pub fn scroll_to_fraction(&mut self, fraction: f64) {
        let fraction = fraction.clamp(0.0, 1.0);
        let max_top = self.max_top_row();
        self.top_row = (fraction * max_top as f64).round() as u64;
        self.clamp_top_row();
    }

    /// Current top row.
    pub fn top_row(&self) -> u64 {
        self.top_row
    }

    /// Total row count.
    pub fn total_rows(&self) -> u64 {
        self.total_rows
    }

    /// Visible row count.
    pub fn visible_rows(&self) -> u64 {
        self.visible_rows
    }

    /// Whether horizontal scrolling is needed.
    pub fn needs_horizontal_scroll(&self, layout: &HexLayout, viewport_width: usize) -> bool {
        layout.total_row_width() > viewport_width
    }

    /// Set the top row directly (e.g., from scrollbar drag).
    pub fn set_top_row(&mut self, row: u64) {
        self.top_row = row;
        self.clamp_top_row();
    }

    /// Maximum valid top_row value.
    fn max_top_row(&self) -> u64 {
        self.total_rows.saturating_sub(self.visible_rows)
    }

    /// Clamp top_row to valid range [0, max_top_row].
    fn clamp_top_row(&mut self) {
        let max = self.max_top_row();
        if self.top_row > max {
            self.top_row = max;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 9 AC 1
    #[test]
    fn recalculate_computes_correct_total_rows() {
        let mut vp = HexViewportAdapter::new(0, 20);
        vp.recalculate(256, BytesPerRow::Sixteen);
        assert_eq!(vp.total_rows(), 16);

        vp.recalculate(257, BytesPerRow::Sixteen);
        assert_eq!(vp.total_rows(), 17);

        vp.recalculate(0, BytesPerRow::Sixteen);
        assert_eq!(vp.total_rows(), 1);
    }

    // Validates: Requirement 9 AC 2
    #[test]
    fn page_down_advances_by_visible_rows() {
        let mut vp = HexViewportAdapter::new(100, 20);
        assert_eq!(vp.top_row(), 0);

        vp.page_down();
        assert_eq!(vp.top_row(), 20);

        vp.page_down();
        assert_eq!(vp.top_row(), 40);
    }

    // Validates: Requirement 9 AC 3
    #[test]
    fn page_up_moves_back_by_visible_rows_clamped_to_zero() {
        let mut vp = HexViewportAdapter::new(100, 20);
        vp.set_top_row(30);

        vp.page_up();
        assert_eq!(vp.top_row(), 10);

        vp.page_up();
        assert_eq!(vp.top_row(), 0);

        // Already at 0, should stay
        vp.page_up();
        assert_eq!(vp.top_row(), 0);
    }

    // Validates: Requirement 9 AC 5
    #[test]
    fn ensure_row_visible_scrolls_down_when_below_viewport() {
        let mut vp = HexViewportAdapter::new(100, 20);
        vp.ensure_row_visible(25);
        // Row 25 should be visible: top_row should be 6 (25 - 19)
        assert_eq!(vp.top_row(), 6);
    }

    // Validates: Requirement 9 AC 5
    #[test]
    fn ensure_row_visible_scrolls_up_when_above_viewport() {
        let mut vp = HexViewportAdapter::new(100, 20);
        vp.set_top_row(30);
        vp.ensure_row_visible(10);
        assert_eq!(vp.top_row(), 10);
    }

    // Validates: Requirement 9 AC 5
    #[test]
    fn ensure_row_visible_no_change_when_already_visible() {
        let mut vp = HexViewportAdapter::new(100, 20);
        vp.set_top_row(10);
        vp.ensure_row_visible(15);
        assert_eq!(vp.top_row(), 10); // unchanged
    }

    // Validates: Requirement 9 AC 4
    #[test]
    fn scrollbar_fraction_maps_position_correctly() {
        let mut vp = HexViewportAdapter::new(100, 20);
        assert_eq!(vp.scrollbar_fraction(), 0.0);

        vp.set_top_row(80); // max_top = 100 - 20 = 80
        assert_eq!(vp.scrollbar_fraction(), 1.0);

        vp.set_top_row(40);
        assert_eq!(vp.scrollbar_fraction(), 0.5);
    }

    // Validates: Requirement 9 AC 4
    #[test]
    fn scroll_to_fraction_sets_top_row() {
        let mut vp = HexViewportAdapter::new(100, 20);
        vp.scroll_to_fraction(0.5);
        assert_eq!(vp.top_row(), 40); // 0.5 * 80 = 40

        vp.scroll_to_fraction(1.0);
        assert_eq!(vp.top_row(), 80); // max
    }

    // Validates: Requirement 9 AC 7
    #[test]
    fn recalculate_clamps_top_row_when_total_shrinks() {
        let mut vp = HexViewportAdapter::new(100, 20);
        vp.set_top_row(70);
        // Shrink total rows
        vp.recalculate(480, BytesPerRow::Sixteen); // 480/16 = 30 rows
                                                   // max_top = 30 - 20 = 10
        assert_eq!(vp.top_row(), 10);
    }

    // Validates: Requirement 9 AC 8
    #[test]
    fn page_down_clamped_at_max_top_row() {
        let mut vp = HexViewportAdapter::new(25, 20);
        // max_top = 25 - 20 = 5
        vp.page_down();
        assert_eq!(vp.top_row(), 5); // clamped

        vp.page_down();
        assert_eq!(vp.top_row(), 5); // still clamped
    }

    // Validates: Requirement 9 AC 6
    #[test]
    fn needs_horizontal_scroll_based_on_layout_width() {
        let layout = HexLayout::new(256, BytesPerRow::Sixteen);
        let vp = HexViewportAdapter::new(16, 20);

        let row_width = layout.total_row_width();
        assert!(!vp.needs_horizontal_scroll(&layout, row_width + 10));
        assert!(vp.needs_horizontal_scroll(&layout, row_width - 1));
    }
}
