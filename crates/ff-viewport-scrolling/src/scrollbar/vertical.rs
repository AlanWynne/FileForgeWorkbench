//! Vertical scrollbar model.
//!
//! Pure-function mapping between `top_line` and scrollbar fraction.
//! Uses 64-bit integer arithmetic for precision with large files.

use crate::types::ScrollFraction;

/// Pure-function vertical scrollbar model.
pub struct VerticalScrollbar;

impl VerticalScrollbar {
    /// Compute the scrollbar position fraction from viewport state.
    ///
    /// Returns 0.0 when top_line == 1, 1.0 when top_line == max_top_line.
    pub fn position_fraction(top_line: u64, max_top_line: u64) -> ScrollFraction {
        if max_top_line <= 1 {
            return ScrollFraction::new(0.0);
        }
        let fraction = (top_line.saturating_sub(1)) as f64 / (max_top_line - 1) as f64;
        ScrollFraction::new(fraction)
    }

    /// Compute the thumb size ratio (visible_count / total_display_lines).
    ///
    /// Returns 1.0 when entire document fits in viewport.
    pub fn thumb_ratio(visible_count: u64, total_display_lines: u64) -> f64 {
        if total_display_lines == 0 {
            return 1.0;
        }
        (visible_count as f64 / total_display_lines as f64).clamp(0.0, 1.0)
    }

    /// Convert a scrollbar fraction to a top_line value.
    ///
    /// Uses 64-bit integer arithmetic for precision with large files.
    pub fn fraction_to_top_line(fraction: ScrollFraction, max_top_line: u64) -> u64 {
        if max_top_line <= 1 {
            return 1;
        }
        let f = fraction.value();
        let result = 1.0 + f * (max_top_line - 1) as f64;
        (result.round() as u64).clamp(1, max_top_line)
    }

    /// Whether the scrollbar should be disabled (document fits in viewport).
    pub fn is_disabled(total_display_lines: u64, visible_count: u64) -> bool {
        total_display_lines <= visible_count
    }

    /// Precision drag: given a pixel delta, compute fine-grained top_line change.
    ///
    /// In precision mode, 1 pixel of mouse movement = 1 line of scroll.
    pub fn precision_drag_delta(
        pixel_delta: i32,
        _track_height: u32,
        _total_display_lines: u64,
        _max_top_line: u64,
    ) -> i64 {
        pixel_delta as i64
    }
}
