//! Style buffer: parallel array of style-slot indices matching document text length.

use crate::types::{BytePosition, HighlightSpan, StyleSlotIndex};

/// Parallel array of style-slot indices matching document text length.
/// Provides O(1) positional access.
/// Addresses: Requirement 2
pub struct StyleBuffer {
    data: Vec<u8>,
}

impl StyleBuffer {
    /// Create a style buffer of the given length, initialized to DEFAULT (0).
    /// Addresses: Requirement 2, criterion 2.5
    pub fn new(length: usize) -> Self {
        Self {
            data: vec![StyleSlotIndex::DEFAULT.0; length],
        }
    }

    /// Get the style at a byte position. O(1).
    /// Addresses: Requirement 2, criterion 2.3
    pub fn get(&self, position: BytePosition) -> StyleSlotIndex {
        self.data
            .get(position.0)
            .copied()
            .map(StyleSlotIndex)
            .unwrap_or(StyleSlotIndex::DEFAULT)
    }

    /// Set the style for a byte range [start, end).
    /// Addresses: Requirement 2, criterion 2.2
    pub fn set_range(&mut self, start: BytePosition, end: BytePosition, style: StyleSlotIndex) {
        let start_idx = start.0.min(self.data.len());
        let end_idx = end.0.min(self.data.len());
        for byte in &mut self.data[start_idx..end_idx] {
            *byte = style.0;
        }
    }

    /// Insert default style values at a position (for text insertion).
    /// Addresses: Requirement 2, criterion 2.7
    pub fn insert(&mut self, position: BytePosition, count: usize) {
        let pos = position.0.min(self.data.len());
        self.data.splice(
            pos..pos,
            std::iter::repeat_n(StyleSlotIndex::DEFAULT.0, count),
        );
    }

    /// Remove style values at a position (for text deletion).
    /// Addresses: Requirement 2, criterion 2.8
    pub fn delete(&mut self, position: BytePosition, count: usize) {
        let pos = position.0.min(self.data.len());
        let end = (pos + count).min(self.data.len());
        self.data.drain(pos..end);
    }

    /// Get the buffer length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get styled spans: coalesce adjacent positions with same style.
    /// Addresses: Requirement 2, criterion 2.4
    pub fn spans(&self, start: BytePosition, end: BytePosition) -> Vec<HighlightSpan> {
        let start_idx = start.0.min(self.data.len());
        let end_idx = end.0.min(self.data.len());

        if start_idx >= end_idx {
            return Vec::new();
        }

        let mut spans = Vec::new();
        let mut span_start = start_idx;
        let mut current_style = self.data[start_idx];

        for i in (start_idx + 1)..end_idx {
            if self.data[i] != current_style {
                spans.push(HighlightSpan {
                    start: BytePosition(span_start),
                    end: BytePosition(i),
                    style: StyleSlotIndex(current_style),
                });
                span_start = i;
                current_style = self.data[i];
            }
        }

        // Final span
        spans.push(HighlightSpan {
            start: BytePosition(span_start),
            end: BytePosition(end_idx),
            style: StyleSlotIndex(current_style),
        });

        spans
    }

    /// Get raw access to the underlying data (for StyleContext).
    #[allow(dead_code)]
    pub(crate) fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable raw access to the underlying data (for StyleContext).
    pub(crate) fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_initialized_to_default() {
        // Validates: Requirement 2, criterion 2.5
        let buf = StyleBuffer::new(10);
        assert_eq!(buf.len(), 10);
        for i in 0..10 {
            assert_eq!(buf.get(BytePosition(i)), StyleSlotIndex::DEFAULT);
        }
    }

    #[test]
    fn empty_buffer_has_zero_length() {
        let buf = StyleBuffer::new(0);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn set_range_assigns_style() {
        // Validates: Requirement 2, criterion 2.2
        let mut buf = StyleBuffer::new(10);
        buf.set_range(BytePosition(2), BytePosition(5), StyleSlotIndex(3));
        assert_eq!(buf.get(BytePosition(1)), StyleSlotIndex::DEFAULT);
        assert_eq!(buf.get(BytePosition(2)), StyleSlotIndex(3));
        assert_eq!(buf.get(BytePosition(3)), StyleSlotIndex(3));
        assert_eq!(buf.get(BytePosition(4)), StyleSlotIndex(3));
        assert_eq!(buf.get(BytePosition(5)), StyleSlotIndex::DEFAULT);
    }

    #[test]
    fn get_out_of_range_returns_default() {
        let buf = StyleBuffer::new(5);
        assert_eq!(buf.get(BytePosition(100)), StyleSlotIndex::DEFAULT);
    }

    #[test]
    fn insert_grows_buffer() {
        // Validates: Requirement 2, criterion 2.7
        let mut buf = StyleBuffer::new(5);
        buf.set_range(BytePosition(0), BytePosition(5), StyleSlotIndex(1));
        buf.insert(BytePosition(2), 3);
        assert_eq!(buf.len(), 8);
        assert_eq!(buf.get(BytePosition(0)), StyleSlotIndex(1));
        assert_eq!(buf.get(BytePosition(1)), StyleSlotIndex(1));
        // Inserted positions have default style
        assert_eq!(buf.get(BytePosition(2)), StyleSlotIndex::DEFAULT);
        assert_eq!(buf.get(BytePosition(3)), StyleSlotIndex::DEFAULT);
        assert_eq!(buf.get(BytePosition(4)), StyleSlotIndex::DEFAULT);
        // Shifted existing styles
        assert_eq!(buf.get(BytePosition(5)), StyleSlotIndex(1));
        assert_eq!(buf.get(BytePosition(6)), StyleSlotIndex(1));
        assert_eq!(buf.get(BytePosition(7)), StyleSlotIndex(1));
    }

    #[test]
    fn delete_shrinks_buffer() {
        // Validates: Requirement 2, criterion 2.8
        let mut buf = StyleBuffer::new(10);
        buf.set_range(BytePosition(0), BytePosition(3), StyleSlotIndex(1));
        buf.set_range(BytePosition(3), BytePosition(6), StyleSlotIndex(2));
        buf.set_range(BytePosition(6), BytePosition(10), StyleSlotIndex(3));
        buf.delete(BytePosition(3), 3);
        assert_eq!(buf.len(), 7);
        assert_eq!(buf.get(BytePosition(0)), StyleSlotIndex(1));
        assert_eq!(buf.get(BytePosition(2)), StyleSlotIndex(1));
        assert_eq!(buf.get(BytePosition(3)), StyleSlotIndex(3));
        assert_eq!(buf.get(BytePosition(6)), StyleSlotIndex(3));
    }

    #[test]
    fn spans_coalesces_adjacent_same_style() {
        // Validates: Requirement 2, criterion 2.4
        let mut buf = StyleBuffer::new(10);
        buf.set_range(BytePosition(0), BytePosition(4), StyleSlotIndex(1));
        buf.set_range(BytePosition(4), BytePosition(7), StyleSlotIndex(2));
        buf.set_range(BytePosition(7), BytePosition(10), StyleSlotIndex(1));

        let spans = buf.spans(BytePosition(0), BytePosition(10));
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].start, BytePosition(0));
        assert_eq!(spans[0].end, BytePosition(4));
        assert_eq!(spans[0].style, StyleSlotIndex(1));
        assert_eq!(spans[1].start, BytePosition(4));
        assert_eq!(spans[1].end, BytePosition(7));
        assert_eq!(spans[1].style, StyleSlotIndex(2));
        assert_eq!(spans[2].start, BytePosition(7));
        assert_eq!(spans[2].end, BytePosition(10));
        assert_eq!(spans[2].style, StyleSlotIndex(1));
    }

    #[test]
    fn spans_empty_range_returns_empty() {
        let buf = StyleBuffer::new(10);
        let spans = buf.spans(BytePosition(5), BytePosition(5));
        assert!(spans.is_empty());
    }

    #[test]
    fn spans_single_style_returns_one_span() {
        let buf = StyleBuffer::new(10);
        let spans = buf.spans(BytePosition(0), BytePosition(10));
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].start, BytePosition(0));
        assert_eq!(spans[0].end, BytePosition(10));
        assert_eq!(spans[0].style, StyleSlotIndex::DEFAULT);
    }

    #[test]
    fn insert_at_end() {
        let mut buf = StyleBuffer::new(5);
        buf.insert(BytePosition(5), 3);
        assert_eq!(buf.len(), 8);
    }

    #[test]
    fn delete_at_end() {
        let mut buf = StyleBuffer::new(5);
        buf.delete(BytePosition(3), 2);
        assert_eq!(buf.len(), 3);
    }
}
