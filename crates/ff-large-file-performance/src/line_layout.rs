//! LineLayout — per-line layout data structure.
//!
//! Contains measured x-positions, sub-line break points, style runs,
//! and validity state. Cached by LineLayoutCache for instant re-rendering.
//!
//! Adapted from Scintilla's `LineLayout`.

use crate::types::{CharOffset, ChunkRange, StyleSlot, ValidLevel, XPosition};

/// Per-line layout data: measured x-positions, sub-line breaks, style runs,
/// and validity state.
///
/// Cached by `LineLayoutCache` for instant re-rendering.
#[derive(Debug, Clone)]
pub struct LineLayout {
    /// The document line number this layout represents.
    pub line_number: u64,
    /// Character content length (for reuse validation).
    pub text_length: u64,
    /// Measured x-positions for each character (may be partial for long lines).
    pub positions: Vec<XPosition>,
    /// For long lines: the measured chunk range (None = full line measured).
    pub measured_range: Option<ChunkRange>,
    /// Sub-line break points for wrapped lines (character offsets where wraps occur).
    pub sub_line_breaks: Vec<CharOffset>,
    /// Style slot assignments per character run: (start_offset, style_slot).
    pub style_runs: Vec<(CharOffset, StyleSlot)>,
    /// Wrap indent in pixels (for continuation lines).
    pub wrap_indent: XPosition,
    /// Current validity level.
    pub validity: ValidLevel,
    /// Whether this line contains the caret (prioritised for retention).
    pub contains_caret: bool,
}

impl LineLayout {
    /// Create a new LineLayout for the given line.
    pub fn new(line_number: u64, text_length: u64) -> Self {
        Self {
            line_number,
            text_length,
            positions: Vec::new(),
            measured_range: None,
            sub_line_breaks: Vec::new(),
            style_runs: Vec::new(),
            wrap_indent: XPosition(0.0),
            validity: ValidLevel::Invalid,
            contains_caret: false,
        }
    }

    /// Check if this layout is reusable for the given line.
    ///
    /// Returns true if line number matches, text length matches, and
    /// validity level permits reuse.
    pub fn is_reusable_for(&self, line_number: u64, text_length: u64) -> bool {
        self.line_number == line_number
            && self.text_length == text_length
            && self.validity > ValidLevel::Invalid
    }

    /// Get the x-position for a character offset within this layout.
    pub fn x_position_at(&self, offset: CharOffset) -> Option<XPosition> {
        let idx = offset.0 as usize;
        self.positions.get(idx).copied()
    }

    /// Get the character offset nearest to an x-position (for hit-testing).
    pub fn offset_at_x(&self, x: XPosition) -> CharOffset {
        if self.positions.is_empty() {
            return CharOffset(0);
        }
        // Find the first position >= x
        for (i, pos) in self.positions.iter().enumerate() {
            if pos.0 >= x.0 {
                return CharOffset(i as u64);
            }
        }
        CharOffset(self.positions.len() as u64)
    }

    /// Number of sub-lines (1 for unwrapped, >1 for wrapped).
    pub fn sub_line_count(&self) -> usize {
        self.sub_line_breaks.len() + 1
    }

    /// Estimated memory consumption of this entry in bytes.
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.positions.len() * std::mem::size_of::<XPosition>()
            + self.sub_line_breaks.len() * std::mem::size_of::<CharOffset>()
            + self.style_runs.len()
                * (std::mem::size_of::<CharOffset>() + std::mem::size_of::<StyleSlot>())
            + self.text_length as usize // approximate text storage
    }

    /// Downgrade validity to the given level (validity can only decrease).
    pub fn downgrade_to(&mut self, level: ValidLevel) {
        if level < self.validity {
            self.validity = level;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_layout_is_invalid() {
        let layout = LineLayout::new(5, 100);
        assert_eq!(layout.validity, ValidLevel::Invalid);
        assert_eq!(layout.line_number, 5);
        assert_eq!(layout.text_length, 100);
    }

    #[test]
    fn reusable_for_matching_line() {
        // Validates: Requirement 3 AC 9
        let mut layout = LineLayout::new(10, 50);
        layout.validity = ValidLevel::Lines;
        assert!(layout.is_reusable_for(10, 50));
    }

    #[test]
    fn not_reusable_for_different_line() {
        // Validates: Requirement 3 AC 9
        let mut layout = LineLayout::new(10, 50);
        layout.validity = ValidLevel::Lines;
        assert!(!layout.is_reusable_for(11, 50));
    }

    #[test]
    fn not_reusable_for_different_length() {
        // Validates: Requirement 3 AC 9
        let mut layout = LineLayout::new(10, 50);
        layout.validity = ValidLevel::Lines;
        assert!(!layout.is_reusable_for(10, 51));
    }

    #[test]
    fn not_reusable_when_invalid() {
        // Validates: Requirement 3 AC 9
        let layout = LineLayout::new(10, 50);
        assert!(!layout.is_reusable_for(10, 50));
    }

    #[test]
    fn x_position_at_returns_correct_value() {
        let mut layout = LineLayout::new(0, 3);
        layout.positions = vec![XPosition(8.0), XPosition(16.0), XPosition(24.0)];
        assert_eq!(layout.x_position_at(CharOffset(1)).unwrap().0, 16.0);
    }

    #[test]
    fn x_position_at_out_of_bounds_returns_none() {
        let layout = LineLayout::new(0, 3);
        assert!(layout.x_position_at(CharOffset(100)).is_none());
    }

    #[test]
    fn sub_line_count_unwrapped() {
        let layout = LineLayout::new(0, 10);
        assert_eq!(layout.sub_line_count(), 1);
    }

    #[test]
    fn sub_line_count_wrapped() {
        let mut layout = LineLayout::new(0, 100);
        layout.sub_line_breaks = vec![CharOffset(40), CharOffset(80)];
        assert_eq!(layout.sub_line_count(), 3);
    }

    #[test]
    fn downgrade_to_reduces_validity() {
        // Validates: Requirement 3 AC 5 — Property 5: Validity Transitions
        let mut layout = LineLayout::new(0, 10);
        layout.validity = ValidLevel::Lines;
        layout.downgrade_to(ValidLevel::Positions);
        assert_eq!(layout.validity, ValidLevel::Positions);
    }

    #[test]
    fn downgrade_to_does_not_upgrade() {
        // Validates: Requirement 3 AC 5
        let mut layout = LineLayout::new(0, 10);
        layout.validity = ValidLevel::Invalid;
        layout.downgrade_to(ValidLevel::Lines); // should not upgrade
        assert_eq!(layout.validity, ValidLevel::Invalid);
    }
}
