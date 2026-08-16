//! Zoom operations, results, and events.
//!
//! This module defines the outcome types for zoom operations and the
//! event/metrics structs used to coordinate with downstream consumers.

use crate::state::ZoomState;
use crate::types::ZoomOffset;

/// The outcome of a zoom operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoomResult {
    /// The zoom operation was applied successfully.
    Applied {
        /// The new zoom offset value after the operation.
        new_offset: i32,
    },
    /// The zoom operation was rejected because the offset is already at a limit.
    AtLimit {
        /// The limit value (min or max) that was reached.
        limit: i32,
        /// A human-readable message describing the limit.
        message: String,
    },
}

/// Font metrics computed from zoom state.
///
/// Provides the data bridge between zoom state and viewport/rendering.
/// The rendering layer uses these values to determine layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomFontMetrics {
    /// The base font size from the theme (in points).
    pub base_font_size: u32,
    /// The effective font size after applying zoom offset (in points).
    pub effective_font_size: u32,
    /// The current zoom offset value.
    pub zoom_offset: i32,
}

impl ZoomFontMetrics {
    /// Compute font metrics from a base size and zoom state.
    pub fn compute(base_size: u32, state: &ZoomState) -> Self {
        Self {
            base_font_size: base_size,
            effective_font_size: state.effective_font_size(base_size),
            zoom_offset: state.offset().value(),
        }
    }

    /// Estimate the number of visible lines given a viewport height and line height.
    ///
    /// This is a simplified calculation — the rendering engine may apply
    /// additional adjustments for partial lines.
    pub fn visible_lines(&self, viewport_height_px: f32, line_height_px: f32) -> u32 {
        if line_height_px <= 0.0 {
            return 0;
        }
        (viewport_height_px / line_height_px).floor() as u32
    }
}

/// Event emitted after a zoom offset mutation on an editor instance.
///
/// Downstream consumers (viewport, status bar, rendering) observe this
/// event to trigger re-layout and indicator updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomChangeEvent {
    /// Identifier for the editor instance that changed.
    pub editor_instance_id: u64,
    /// The zoom offset before the change.
    pub old_offset: i32,
    /// The zoom offset after the change.
    pub new_offset: i32,
    /// The effective font size after the change (in points).
    pub effective_font_size: u32,
    /// Whether the viewport needs re-layout (true when offset actually changed).
    pub requires_relayout: bool,
}

impl ZoomChangeEvent {
    /// Construct a change event from before/after state.
    ///
    /// `requires_relayout` is true only when the offset actually changed.
    pub fn from_state_change(
        editor_instance_id: u64,
        old_offset: ZoomOffset,
        new_offset: ZoomOffset,
        base_size: u32,
    ) -> Self {
        Self {
            editor_instance_id,
            old_offset: old_offset.value(),
            new_offset: new_offset.value(),
            effective_font_size: new_offset.effective_font_size(base_size),
            requires_relayout: old_offset != new_offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ZoomConfig;

    // Validates: Requirement 1.2 — effective font size computation via metrics
    #[test]
    fn font_metrics_computes_effective_size() {
        let config = ZoomConfig::default();
        let state = ZoomState::from_persisted(3, &config);
        let metrics = ZoomFontMetrics::compute(12, &state);
        assert_eq!(metrics.base_font_size, 12);
        assert_eq!(metrics.effective_font_size, 15);
        assert_eq!(metrics.zoom_offset, 3);
    }

    // Validates: Requirement 1.8 — larger offset → fewer visible lines
    #[test]
    fn visible_lines_decreases_with_larger_font() {
        // With a 600px viewport:
        // At 12pt (line_height ~16px): 600/16 = 37 lines
        // At 15pt (line_height ~20px): 600/20 = 30 lines
        let metrics_small = ZoomFontMetrics {
            base_font_size: 12,
            effective_font_size: 12,
            zoom_offset: 0,
        };
        let metrics_large = ZoomFontMetrics {
            base_font_size: 12,
            effective_font_size: 15,
            zoom_offset: 3,
        };
        let lines_small = metrics_small.visible_lines(600.0, 16.0);
        let lines_large = metrics_large.visible_lines(600.0, 20.0);
        assert!(lines_small > lines_large);
    }

    // Validates: Requirement 1.6 — requires_relayout when offset changes
    #[test]
    fn change_event_requires_relayout_when_offset_changed() {
        let old = ZoomOffset::new(0, -10, 60);
        let new = ZoomOffset::new(3, -10, 60);
        let event = ZoomChangeEvent::from_state_change(1, old, new, 12);
        assert!(event.requires_relayout);
        assert_eq!(event.effective_font_size, 15);
    }

    // Validates: Requirement 1.6 — no relayout when offset unchanged
    #[test]
    fn change_event_no_relayout_when_offset_same() {
        let old = ZoomOffset::new(5, -10, 60);
        let new = ZoomOffset::new(5, -10, 60);
        let event = ZoomChangeEvent::from_state_change(1, old, new, 12);
        assert!(!event.requires_relayout);
    }

    #[test]
    fn visible_lines_zero_when_line_height_zero() {
        let metrics = ZoomFontMetrics {
            base_font_size: 12,
            effective_font_size: 12,
            zoom_offset: 0,
        };
        assert_eq!(metrics.visible_lines(600.0, 0.0), 0);
    }
}
