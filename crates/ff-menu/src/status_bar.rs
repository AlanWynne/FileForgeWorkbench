//! Status bar manager — segment lifecycle, ordering, and layout.
//!
//! The status bar is a horizontal bar at the bottom of the Primary_Window,
//! divided into configurable segments that display real-time workbench state.

use crate::error::MenuError;
use crate::status_segment::{validate_segment_id, SegmentAlignment, StatusSegment};

/// Manages the status bar segment registry and layout.
///
/// Segments are ordered by alignment group (Left, Center, Right),
/// then by priority within each group (lower priority renders first).
#[derive(Debug, Clone)]
pub struct StatusBar {
    /// Registered segments ordered by alignment and priority.
    segments: Vec<StatusSegment>,
}

impl StatusBar {
    /// Creates a new empty status bar.
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Registers a new segment. Returns an error if the segment ID already exists
    /// or if the ID format is invalid.
    ///
    /// # Errors
    ///
    /// - `MenuError::DuplicateSegmentId` if a segment with the same ID exists
    /// - `MenuError::InvalidSegmentId` if the ID format is invalid
    pub fn register_segment(&mut self, segment: StatusSegment) -> Result<(), MenuError> {
        validate_segment_id(&segment.id)?;

        if self.segments.iter().any(|s| s.id == segment.id) {
            return Err(MenuError::DuplicateSegmentId { id: segment.id });
        }

        self.segments.push(segment);
        self.sort_segments();
        Ok(())
    }

    /// Unregisters a segment by ID. Returns true if the segment was found and removed.
    pub fn unregister_segment(&mut self, segment_id: &str) -> bool {
        let len_before = self.segments.len();
        self.segments.retain(|s| s.id != segment_id);
        self.segments.len() < len_before
    }

    /// Returns all registered segments in display order.
    pub fn segments(&self) -> &[StatusSegment] {
        &self.segments
    }

    /// Returns only visible segments in display order.
    pub fn visible_segments(&self) -> Vec<&StatusSegment> {
        self.segments.iter().filter(|s| s.visible).collect()
    }

    /// Returns a mutable reference to a segment by ID.
    pub fn get_segment_mut(&mut self, segment_id: &str) -> Option<&mut StatusSegment> {
        self.segments.iter_mut().find(|s| s.id == segment_id)
    }

    /// Returns a reference to a segment by ID.
    pub fn get_segment(&self, segment_id: &str) -> Option<&StatusSegment> {
        self.segments.iter().find(|s| s.id == segment_id)
    }

    /// Returns all unique segment IDs.
    pub fn segment_ids(&self) -> Vec<&str> {
        self.segments.iter().map(|s| s.id.as_str()).collect()
    }

    /// Returns the number of registered segments.
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Sorts segments by alignment group, then by priority within each group.
    fn sort_segments(&mut self) {
        self.segments.sort_by(|a, b| {
            let align_ord = alignment_order(a.alignment).cmp(&alignment_order(b.alignment));
            align_ord.then_with(|| a.priority.cmp(&b.priority))
        });
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns a numeric ordering value for segment alignment groups.
fn alignment_order(alignment: SegmentAlignment) -> u8 {
    match alignment {
        SegmentAlignment::Left => 0,
        SegmentAlignment::Center => 1,
        SegmentAlignment::Right => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_status_bar_is_empty() {
        let bar = StatusBar::new();
        assert_eq!(bar.segment_count(), 0);
        assert!(bar.segments().is_empty());
    }

    #[test]
    fn register_segment_adds_to_bar() {
        let mut bar = StatusBar::new();
        let segment = StatusSegment::new("editor_mode", SegmentAlignment::Left, 0).unwrap();
        assert!(bar.register_segment(segment).is_ok());
        assert_eq!(bar.segment_count(), 1);
    }

    #[test]
    fn duplicate_segment_id_rejected() {
        let mut bar = StatusBar::new();
        let s1 = StatusSegment::new("mode", SegmentAlignment::Left, 0).unwrap();
        let s2 = StatusSegment::new("mode", SegmentAlignment::Right, 10).unwrap();

        assert!(bar.register_segment(s1).is_ok());
        let result = bar.register_segment(s2);
        assert!(result.is_err());
        assert_eq!(bar.segment_count(), 1);
    }

    #[test]
    fn segments_ordered_by_alignment_then_priority() {
        let mut bar = StatusBar::new();
        bar.register_segment(
            StatusSegment::new("right_high", SegmentAlignment::Right, 10).unwrap(),
        )
        .unwrap();
        bar.register_segment(StatusSegment::new("left_low", SegmentAlignment::Left, 0).unwrap())
            .unwrap();
        bar.register_segment(StatusSegment::new("right_low", SegmentAlignment::Right, 5).unwrap())
            .unwrap();
        bar.register_segment(StatusSegment::new("center", SegmentAlignment::Center, 0).unwrap())
            .unwrap();
        bar.register_segment(StatusSegment::new("left_high", SegmentAlignment::Left, 10).unwrap())
            .unwrap();

        let ids: Vec<&str> = bar.segments().iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["left_low", "left_high", "center", "right_low", "right_high"]
        );
    }

    #[test]
    fn unregister_segment_removes_by_id() {
        let mut bar = StatusBar::new();
        bar.register_segment(StatusSegment::new("seg1", SegmentAlignment::Left, 0).unwrap())
            .unwrap();
        bar.register_segment(StatusSegment::new("seg2", SegmentAlignment::Left, 1).unwrap())
            .unwrap();

        assert!(bar.unregister_segment("seg1"));
        assert_eq!(bar.segment_count(), 1);
        assert_eq!(bar.segments()[0].id, "seg2");
    }

    #[test]
    fn unregister_nonexistent_returns_false() {
        let mut bar = StatusBar::new();
        assert!(!bar.unregister_segment("nonexistent"));
    }

    #[test]
    fn visible_segments_filters_hidden() {
        let mut bar = StatusBar::new();
        bar.register_segment(StatusSegment::new("visible", SegmentAlignment::Left, 0).unwrap())
            .unwrap();
        let mut hidden = StatusSegment::new("hidden", SegmentAlignment::Left, 1).unwrap();
        hidden.visible = false;
        bar.register_segment(hidden).unwrap();

        let visible = bar.visible_segments();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].id, "visible");
    }
}
