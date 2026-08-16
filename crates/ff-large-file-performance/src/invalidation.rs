//! Cache invalidation coordinator.
//!
//! Receives edit/font/zoom/resize events, batches invalidations within a frame,
//! and dispatches to caches.

use crate::line_layout_cache::LineLayoutCache;
use crate::position_cache::PositionCache;
use crate::types::ValidLevel;

/// An invalidation event that affects cached measurements.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InvalidationEvent {
    /// A single line was edited (content changed, same line count).
    LineEdited { line_number: u64 },
    /// Lines were inserted or deleted (line count changed).
    LinesChanged { from_line: u64, lines_delta: i64 },
    /// A style change occurred on a specific line.
    StyleChanged { line_number: u64 },
    /// Font metrics changed (font family, size, weight, or style).
    FontChanged,
    /// Zoom level changed.
    ZoomChanged,
    /// Viewport width changed (affects sub-line breaks only).
    ViewportResized,
    /// Display-line visibility changed (fold/unfold/exclude).
    VisibilityChanged { line_number: u64 },
}

/// Coordinates cache invalidation across all subsystems.
///
/// Batches multiple invalidation events within a single frame into
/// coalesced operations.
pub struct InvalidationCoordinator {
    /// Pending invalidation events for the current frame.
    pending_events: Vec<InvalidationEvent>,
    /// Whether we are within a frame batch window.
    in_batch: bool,
    /// Metric: total invalidation events processed.
    invalidation_count: u64,
}

impl InvalidationCoordinator {
    /// Create a new InvalidationCoordinator.
    pub fn new() -> Self {
        Self {
            pending_events: Vec::new(),
            in_batch: false,
            invalidation_count: 0,
        }
    }

    /// Begin a batch window (call at start of frame).
    pub fn begin_batch(&mut self) {
        self.in_batch = true;
        self.pending_events.clear();
    }

    /// Submit an invalidation event to the current batch.
    pub fn submit(&mut self, event: InvalidationEvent) {
        self.invalidation_count += 1;
        if self.in_batch {
            self.pending_events.push(event);
        } else {
            // Immediate dispatch (single event)
            self.pending_events.push(event);
        }
    }

    /// End the batch window and dispatch coalesced invalidations to caches.
    pub fn flush(
        &mut self,
        position_cache: &PositionCache,
        line_layout_cache: &mut LineLayoutCache,
    ) {
        self.in_batch = false;

        // Coalesce: if FontChanged or ZoomChanged is present, do full clear
        let has_font_change = self.pending_events.iter().any(|e| {
            matches!(
                e,
                InvalidationEvent::FontChanged | InvalidationEvent::ZoomChanged
            )
        });

        if has_font_change {
            position_cache.clear();
            line_layout_cache.clear();
            self.pending_events.clear();
            return;
        }

        // Check for viewport resize
        let has_resize = self
            .pending_events
            .iter()
            .any(|e| matches!(e, InvalidationEvent::ViewportResized));

        if has_resize {
            // Downgrade sub-line breaks only — positions remain valid
            line_layout_cache.downgrade_all_to(ValidLevel::Positions);
        }

        // Process remaining events
        let events: Vec<_> = self.pending_events.drain(..).collect();
        for event in events {
            match event {
                InvalidationEvent::LineEdited { line_number } => {
                    line_layout_cache.invalidate_line(line_number);
                }
                InvalidationEvent::LinesChanged { from_line, .. } => {
                    line_layout_cache.invalidate_from(from_line);
                }
                InvalidationEvent::StyleChanged { line_number } => {
                    line_layout_cache.mark_check_style(line_number);
                }
                InvalidationEvent::VisibilityChanged { .. } => {
                    // Do NOT invalidate — cached data remains valid for hidden lines
                }
                InvalidationEvent::FontChanged
                | InvalidationEvent::ZoomChanged
                | InvalidationEvent::ViewportResized => {
                    // Already handled above
                }
            }
        }
    }

    /// Get the total invalidation event count.
    pub fn invalidation_count(&self) -> u64 {
        self.invalidation_count
    }
}

impl Default for InvalidationCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_layout::LineLayout;
    use crate::types::{CacheLevel, ValidLevel};

    fn make_cache() -> (PositionCache, LineLayoutCache) {
        let pc = PositionCache::new(64);
        let mut llc = LineLayoutCache::new(CacheLevel::Page, 20, 5);
        let mut l = LineLayout::new(5, 100);
        l.validity = ValidLevel::Lines;
        llc.insert(l);
        (pc, llc)
    }

    #[test]
    fn line_edit_invalidates_line() {
        // Validates: Requirement 9 AC 1
        let (pc, mut llc) = make_cache();
        let mut coord = InvalidationCoordinator::new();
        coord.begin_batch();
        coord.submit(InvalidationEvent::LineEdited { line_number: 5 });
        coord.flush(&pc, &mut llc);
        assert_eq!(llc.get(5).unwrap().validity, ValidLevel::Invalid);
    }

    #[test]
    fn font_change_clears_all() {
        // Validates: Requirement 9 AC 3
        let (pc, mut llc) = make_cache();
        let mut coord = InvalidationCoordinator::new();
        coord.begin_batch();
        coord.submit(InvalidationEvent::FontChanged);
        coord.flush(&pc, &mut llc);
        assert!(llc.is_empty());
        assert!(pc.is_empty());
    }

    #[test]
    fn zoom_change_clears_all() {
        // Validates: Requirement 9 AC 4
        let (pc, mut llc) = make_cache();
        let mut coord = InvalidationCoordinator::new();
        coord.begin_batch();
        coord.submit(InvalidationEvent::ZoomChanged);
        coord.flush(&pc, &mut llc);
        assert!(llc.is_empty());
    }

    #[test]
    fn viewport_resize_downgrades_to_positions() {
        // Validates: Requirement 9 AC 5
        let (pc, mut llc) = make_cache();
        let mut coord = InvalidationCoordinator::new();
        coord.begin_batch();
        coord.submit(InvalidationEvent::ViewportResized);
        coord.flush(&pc, &mut llc);
        assert_eq!(llc.get(5).unwrap().validity, ValidLevel::Positions);
    }

    #[test]
    fn visibility_change_does_not_invalidate() {
        // Validates: Requirement 9 AC 8
        let (pc, mut llc) = make_cache();
        let mut coord = InvalidationCoordinator::new();
        coord.begin_batch();
        coord.submit(InvalidationEvent::VisibilityChanged { line_number: 5 });
        coord.flush(&pc, &mut llc);
        // Line 5 should still be valid
        assert_eq!(llc.get(5).unwrap().validity, ValidLevel::Lines);
    }

    #[test]
    fn style_change_marks_check_style() {
        // Validates: Requirement 9 AC 6
        let (pc, mut llc) = make_cache();
        let mut coord = InvalidationCoordinator::new();
        coord.begin_batch();
        coord.submit(InvalidationEvent::StyleChanged { line_number: 5 });
        coord.flush(&pc, &mut llc);
        assert_eq!(llc.get(5).unwrap().validity, ValidLevel::CheckTextAndStyle);
    }

    #[test]
    fn invalidation_count_tracked() {
        // Validates: Requirement 9 AC 9
        let (pc, mut llc) = make_cache();
        let mut coord = InvalidationCoordinator::new();
        coord.submit(InvalidationEvent::LineEdited { line_number: 1 });
        coord.submit(InvalidationEvent::LineEdited { line_number: 2 });
        coord.flush(&pc, &mut llc);
        assert_eq!(coord.invalidation_count(), 2);
    }
}
