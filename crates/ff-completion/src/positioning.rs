//! Popup positioning model.
//!
//! Computes the popup's anchor position, dimensions, and direction (above/below)
//! relative to the command field and viewport. This module is GUI-independent —
//! it produces coordinates that the shell renderer consumes.

/// The application window viewport rectangle.
#[derive(Debug, Clone, Copy)]
pub struct ViewportRect {
    /// X origin of the viewport.
    pub x: f32,
    /// Y origin of the viewport.
    pub y: f32,
    /// Width of the viewport in logical pixels.
    pub width: f32,
    /// Height of the viewport in logical pixels.
    pub height: f32,
}

/// The command field rectangle (for positioning relative to the field).
#[derive(Debug, Clone, Copy)]
pub struct FieldRect {
    /// X coordinate of the field's left edge.
    pub x: f32,
    /// Y coordinate of the field's top edge.
    pub y: f32,
    /// Width of the field.
    pub width: f32,
    /// Height of the field.
    pub height: f32,
}

impl FieldRect {
    /// Returns the Y coordinate of the bottom edge.
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns the Y coordinate of the top edge.
    pub fn top(&self) -> f32 {
        self.y
    }
}

/// Configuration for popup positioning calculations.
#[derive(Debug, Clone, Copy)]
pub struct PopupConfig {
    /// Maximum number of visible items.
    pub max_items: usize,
    /// Maximum popup width in logical pixels.
    pub max_width: f32,
    /// Height of a single item row in logical pixels.
    pub item_height: f32,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            max_items: 10,
            max_width: 400.0,
            item_height: 20.0,
        }
    }
}

/// The anchor coordinates for popup placement.
#[derive(Debug, Clone, Copy)]
pub struct PopupAnchor {
    /// X coordinate — horizontal position at the start of the prefix.
    pub x: f32,
    /// Y coordinate — vertical position of the popup's top edge.
    pub y: f32,
}

/// Computed popup geometry — position, size, and direction.
#[derive(Debug, Clone, Copy)]
pub struct PopupBounds {
    /// X coordinate of the popup's left edge.
    pub x: f32,
    /// Y coordinate of the popup's top edge.
    pub y: f32,
    /// Width of the popup.
    pub width: f32,
    /// Height of the popup.
    pub height: f32,
    /// Whether the popup is positioned above the field (flipped).
    pub flipped: bool,
}

impl PopupBounds {
    /// Returns the right edge X coordinate.
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the bottom edge Y coordinate.
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Returns true if this popup overlaps the given field rectangle.
    pub fn overlaps_field(&self, field: &FieldRect) -> bool {
        self.x < field.x + field.width
            && self.x + self.width > field.x
            && self.y < field.y + field.height
            && self.y + self.height > field.y
    }
}

/// Computes the popup position based on anchor, item count, config, and viewport.
///
/// The algorithm:
/// 1. Default placement: below the field
/// 2. If below extends past viewport bottom: flip above
/// 3. If both directions extend past viewport: choose direction with more space, clip
/// 4. Width is bounded by max_width and viewport width
/// 5. Popup never overlaps the command field
///
/// # Arguments
///
/// * `anchor_x` — horizontal position of the prefix start
/// * `field` — the command field rectangle
/// * `item_count` — number of candidates to display
/// * `longest_label_width` — width of the longest label (for sizing)
/// * `config` — popup sizing constraints
/// * `viewport` — the application window bounds
pub fn compute_popup_position(
    anchor_x: f32,
    field: &FieldRect,
    item_count: usize,
    longest_label_width: f32,
    config: &PopupConfig,
    viewport: &ViewportRect,
) -> PopupBounds {
    // Calculate dimensions
    let visible_items = item_count.min(config.max_items);
    let height = visible_items as f32 * config.item_height;
    let width = longest_label_width.max(100.0).min(config.max_width);

    // Clamp width to viewport
    let width = width.min(viewport.width);

    // Calculate X position (clamped to viewport)
    let x = anchor_x
        .max(viewport.x)
        .min((viewport.x + viewport.width) - width);

    // Calculate Y position — try below first
    let below_y = field.bottom();
    let below_fits = below_y + height <= viewport.y + viewport.height;

    // Try above
    let above_y = field.top() - height;
    let above_fits = above_y >= viewport.y;

    let (y, flipped) = if below_fits {
        // Default: below the field
        (below_y, false)
    } else if above_fits {
        // Flip: above the field
        (above_y, true)
    } else {
        // Best-fit: choose direction with more available space
        let space_below = (viewport.y + viewport.height) - field.bottom();
        let space_above = field.top() - viewport.y;

        if space_below >= space_above {
            // More space below — clip to available space
            (below_y, false)
        } else {
            // More space above — clip to available space
            let clipped_height = space_above.min(height);
            let y = field.top() - clipped_height;
            (y, true)
        }
    };

    // Final height adjustment for best-fit clipping
    let final_height = if !below_fits && !above_fits {
        let space_below = (viewport.y + viewport.height) - field.bottom();
        let space_above = field.top() - viewport.y;
        if !flipped {
            height.min(space_below)
        } else {
            height.min(space_above)
        }
    } else {
        height
    };

    PopupBounds {
        x,
        y,
        width,
        height: final_height,
        flipped,
    }
}

/// Recomputes popup position after a viewport resize.
///
/// This is equivalent to calling `compute_popup_position` with updated viewport dimensions.
pub fn recompute_on_resize(
    anchor_x: f32,
    field: &FieldRect,
    item_count: usize,
    longest_label_width: f32,
    config: &PopupConfig,
    new_viewport: &ViewportRect,
) -> PopupBounds {
    compute_popup_position(
        anchor_x,
        field,
        item_count,
        longest_label_width,
        config,
        new_viewport,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard_viewport() -> ViewportRect {
        ViewportRect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }

    fn top_field() -> FieldRect {
        FieldRect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 30.0,
        }
    }

    fn bottom_field() -> FieldRect {
        FieldRect {
            x: 0.0,
            y: 570.0,
            width: 800.0,
            height: 30.0,
        }
    }

    fn standard_config() -> PopupConfig {
        PopupConfig {
            max_items: 10,
            max_width: 400.0,
            item_height: 20.0,
        }
    }

    // Validates: Requirement 3.2 (below by default)
    #[test]
    fn positions_below_field_by_default() {
        let bounds = compute_popup_position(
            10.0,
            &top_field(),
            5,
            200.0,
            &standard_config(),
            &standard_viewport(),
        );
        assert!(!bounds.flipped);
        assert_eq!(bounds.y, 30.0); // bottom of field
        assert!(bounds.bottom() <= 600.0);
    }

    // Validates: Requirement 3.3 (flip above when no space below)
    #[test]
    fn flips_above_when_no_space_below() {
        let bounds = compute_popup_position(
            10.0,
            &bottom_field(),
            10,
            200.0,
            &standard_config(),
            &standard_viewport(),
        );
        assert!(bounds.flipped);
        assert!(bounds.y >= 0.0);
        assert!(bounds.bottom() <= bottom_field().top());
    }

    // Validates: Requirement 3.5 (no overlap with field)
    #[test]
    fn popup_does_not_overlap_field() {
        let field = FieldRect {
            x: 100.0,
            y: 100.0,
            width: 600.0,
            height: 30.0,
        };
        let bounds = compute_popup_position(
            100.0,
            &field,
            5,
            200.0,
            &standard_config(),
            &standard_viewport(),
        );
        assert!(!bounds.overlaps_field(&field));
    }

    // Validates: Requirement 3.6 (width bounded by max_width)
    #[test]
    fn width_bounded_by_max_width() {
        let bounds = compute_popup_position(
            10.0,
            &top_field(),
            5,
            9999.0, // very wide labels
            &standard_config(),
            &standard_viewport(),
        );
        assert!(bounds.width <= 400.0); // max_width
    }

    // Validates: Requirement 3.7 (max visible items)
    #[test]
    fn height_limited_to_max_items() {
        let config = PopupConfig {
            max_items: 5,
            max_width: 400.0,
            item_height: 20.0,
        };
        let bounds = compute_popup_position(
            10.0,
            &top_field(),
            100, // many candidates
            200.0,
            &config,
            &standard_viewport(),
        );
        assert_eq!(bounds.height, 100.0); // 5 items * 20px
    }

    // Validates: Requirement 3.8 (reposition on resize)
    #[test]
    fn recompute_on_resize_recalculates() {
        let small_viewport = ViewportRect {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 300.0,
        };
        let bounds = recompute_on_resize(
            10.0,
            &top_field(),
            5,
            200.0,
            &standard_config(),
            &small_viewport,
        );
        assert!(bounds.right() <= 300.0);
        assert!(bounds.bottom() <= 300.0);
    }

    // Validates: Requirement 3.4 (best-fit when both directions overflow)
    #[test]
    fn best_fit_chooses_direction_with_more_space() {
        let small_viewport = ViewportRect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 100.0,
        };
        let mid_field = FieldRect {
            x: 0.0,
            y: 40.0,
            width: 800.0,
            height: 20.0,
        };
        // Space above: 40px, Space below: 40px, need: 200px (10 items * 20)
        let bounds = compute_popup_position(
            10.0,
            &mid_field,
            10,
            200.0,
            &standard_config(),
            &small_viewport,
        );
        // Should clip to available space
        assert!(bounds.y >= 0.0);
        assert!(bounds.bottom() <= 100.0);
    }

    #[test]
    fn x_position_clamped_to_viewport() {
        let bounds = compute_popup_position(
            700.0, // near right edge
            &top_field(),
            5,
            200.0,
            &standard_config(),
            &standard_viewport(),
        );
        assert!(bounds.right() <= 800.0);
    }

    #[test]
    fn within_viewport_bounds() {
        let bounds = compute_popup_position(
            50.0,
            &top_field(),
            5,
            200.0,
            &standard_config(),
            &standard_viewport(),
        );
        assert!(bounds.x >= 0.0);
        assert!(bounds.y >= 0.0);
        assert!(bounds.right() <= 800.0);
        assert!(bounds.bottom() <= 600.0);
    }
}
