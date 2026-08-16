//! Rectangular selection display logic.
//!
//! Computes column-band rendering geometry for rectangular (column) selections.

/// Computes display geometry for rectangular (column) selections.
///
/// A rectangular selection is rendered as a vertical column band spanning
/// the same left-right column range across multiple lines.
///
/// Addresses: Requirement 8, criteria 8.1–8.5
#[derive(Debug, Clone, Copy)]
pub struct RectangularSelectionDisplay;

impl RectangularSelectionDisplay {
    /// Computes the pixel extents (left_x, right_x) for a column band on a single line.
    ///
    /// If the line content is shorter than `right_col`, the band extends into
    /// virtual space.
    ///
    /// Addresses: Requirement 8, criteria 8.1, 8.3
    pub fn column_band_for_line(
        &self,
        left_col: u64,
        right_col: u64,
        line_content_len: u64,
        space_width: f32,
    ) -> (f32, f32) {
        let left_x = left_col as f32 * space_width;
        let right_x = right_col as f32 * space_width;
        // The band extends into virtual space if right_col > line_content_len
        // This is handled naturally since we compute from column positions
        let _ = line_content_len; // Used conceptually — band extends regardless
        (left_x, right_x)
    }

    /// Returns the pixel position for a thin (zero-width) rectangular selection.
    ///
    /// A thin selection is rendered as a vertical line at the column position
    /// on each affected line.
    ///
    /// Addresses: Requirement 8, criterion 8.4
    pub fn thin_selection_x(&self, column: u64, space_width: f32) -> f32 {
        column as f32 * space_width
    }

    /// Computes the caret X position for a rectangular selection on a given line.
    ///
    /// The caret is placed at the caret-column edge of the selection.
    ///
    /// Addresses: Requirement 8, criterion 8.5
    pub fn caret_x_for_line(&self, caret_column: u64, space_width: f32) -> f32 {
        caret_column as f32 * space_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_band_computes_pixel_extents() {
        // Validates: Requirement 8.1
        let display = RectangularSelectionDisplay;
        let (left, right) = display.column_band_for_line(3, 10, 80, 8.0);
        assert_eq!(left, 24.0); // 3 * 8
        assert_eq!(right, 80.0); // 10 * 8
    }

    #[test]
    fn column_band_extends_into_virtual_space() {
        // Validates: Requirement 8.3
        let display = RectangularSelectionDisplay;
        // Line is only 5 chars long, but selection goes to column 10
        let (left, right) = display.column_band_for_line(3, 10, 5, 8.0);
        assert_eq!(left, 24.0);
        assert_eq!(right, 80.0); // extends past line content
    }

    #[test]
    fn thin_selection_returns_single_x_position() {
        // Validates: Requirement 8.4
        let display = RectangularSelectionDisplay;
        let x = display.thin_selection_x(7, 8.0);
        assert_eq!(x, 56.0); // 7 * 8
    }

    #[test]
    fn caret_x_for_line_computes_position() {
        // Validates: Requirement 8.5
        let display = RectangularSelectionDisplay;
        let x = display.caret_x_for_line(15, 8.0);
        assert_eq!(x, 120.0); // 15 * 8
    }

    #[test]
    fn column_band_at_zero_column() {
        let display = RectangularSelectionDisplay;
        let (left, right) = display.column_band_for_line(0, 5, 100, 10.0);
        assert_eq!(left, 0.0);
        assert_eq!(right, 50.0);
    }
}
