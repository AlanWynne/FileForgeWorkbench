//! Virtual space display logic.
//!
//! Computes caret and selection positions in virtual space — the region
//! beyond the end of a line's actual content.

/// A screen-space rectangle for rendering (logical pixels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left edge X coordinate.
    pub x: f32,
    /// Top edge Y coordinate.
    pub y: f32,
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
}

/// Stateless helper for virtual space position calculations.
///
/// Virtual space is the region beyond line-end where the caret can be placed
/// without actual characters existing. The caret uses the same style/width/colour
/// as in real text.
///
/// Addresses: Requirement 7, criteria 7.1–7.6
#[derive(Debug, Clone, Copy)]
pub struct VirtualSpaceRenderer;

impl VirtualSpaceRenderer {
    /// Computes the horizontal caret X position in virtual space.
    ///
    /// The position is: `line_end_x + virtual_space * space_width`.
    /// When `virtual_space` is 0, returns `line_end_x` exactly.
    ///
    /// Addresses: Requirement 7, criterion 7.1
    pub fn horizontal_offset(&self, line_end_x: f32, virtual_space: u64, space_width: f32) -> f32 {
        line_end_x + (virtual_space as f32 * space_width)
    }

    /// Computes the selection highlight rectangle in virtual space.
    ///
    /// Returns the rectangle spanning from `vs_start` to `vs_end` columns
    /// of virtual space, positioned after `line_end_x`.
    ///
    /// Addresses: Requirement 7, criteria 7.3, 7.4
    pub fn selection_rect_in_virtual_space(
        &self,
        line_end_x: f32,
        vs_start: u64,
        vs_end: u64,
        space_width: f32,
        line_height: f32,
    ) -> Rect {
        let x = line_end_x + (vs_start as f32 * space_width);
        let width = (vs_end.saturating_sub(vs_start)) as f32 * space_width;
        Rect {
            x,
            y: 0.0, // Y is set by the caller based on line position
            width,
            height: line_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_offset_zero_virtual_space_returns_line_end() {
        // Validates: Requirement 7.1
        let renderer = VirtualSpaceRenderer;
        let result = renderer.horizontal_offset(100.0, 0, 8.0);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn horizontal_offset_computes_correctly() {
        // Validates: Requirement 7.1
        let renderer = VirtualSpaceRenderer;
        let result = renderer.horizontal_offset(100.0, 5, 8.0);
        assert_eq!(result, 140.0); // 100 + 5*8
    }

    #[test]
    fn horizontal_offset_with_large_virtual_space() {
        let renderer = VirtualSpaceRenderer;
        let result = renderer.horizontal_offset(50.0, 100, 10.0);
        assert_eq!(result, 1050.0); // 50 + 100*10
    }

    #[test]
    fn selection_rect_zero_length_returns_zero_width() {
        let renderer = VirtualSpaceRenderer;
        let rect = renderer.selection_rect_in_virtual_space(100.0, 5, 5, 8.0, 20.0);
        assert_eq!(rect.width, 0.0);
    }

    #[test]
    fn selection_rect_computes_correct_dimensions() {
        // Validates: Requirement 7.3
        let renderer = VirtualSpaceRenderer;
        let rect = renderer.selection_rect_in_virtual_space(100.0, 2, 7, 8.0, 20.0);
        assert_eq!(rect.x, 116.0); // 100 + 2*8
        assert_eq!(rect.width, 40.0); // (7-2)*8
        assert_eq!(rect.height, 20.0);
    }

    #[test]
    fn selection_rect_at_start_of_virtual_space() {
        let renderer = VirtualSpaceRenderer;
        let rect = renderer.selection_rect_in_virtual_space(200.0, 0, 3, 10.0, 16.0);
        assert_eq!(rect.x, 200.0);
        assert_eq!(rect.width, 30.0); // 3*10
    }
}
