//! Smooth scrolling engine.
//!
//! Manages pixel-level smooth scrolling state and target computation.
//! The viewport model computes targets; the GUI shell performs animation.

use crate::types::{PixelOffset, ScrollFraction, ScrollMode};

/// Target for smooth scroll animation, exposed to the GUI shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmoothScrollTarget {
    /// Target top_line after animation completes.
    pub target_line: u64,
    /// Target pixel offset within that line.
    pub target_pixel_offset: PixelOffset,
    /// Total pixel distance to animate.
    pub pixel_distance: i64,
}

/// Manages pixel-level smooth scrolling state and target computation.
#[derive(Debug, Clone)]
pub struct SmoothScrollEngine {
    /// Whether smooth scrolling is currently active.
    enabled: bool,
    /// Current sub-line pixel offset [0, line_height).
    pixel_offset: PixelOffset,
    /// Target top_line for ongoing animation (None if idle).
    target_top_line: Option<u64>,
    /// Target pixel offset for ongoing animation.
    target_pixel_offset: Option<PixelOffset>,
}

impl SmoothScrollEngine {
    /// Create a new smooth scroll engine (disabled by default).
    pub fn new() -> Self {
        Self {
            enabled: false,
            pixel_offset: PixelOffset(0),
            target_top_line: None,
            target_pixel_offset: None,
        }
    }

    /// Whether smooth scrolling is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable smooth scrolling.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset();
        }
    }

    /// Current pixel offset.
    pub fn pixel_offset(&self) -> PixelOffset {
        self.pixel_offset
    }

    /// Set pixel offset (clamped to [0, line_height)).
    pub fn set_pixel_offset(&mut self, offset: u32, line_height: u32) {
        if line_height > 0 {
            self.pixel_offset = PixelOffset(offset % line_height);
        }
    }

    /// Compute the target pixel position for a scroll-to-line command.
    pub fn compute_scroll_target(
        &self,
        current_top_line: u64,
        target_line: u64,
        line_height: u32,
    ) -> SmoothScrollTarget {
        let distance = (target_line as i64 - current_top_line as i64) * line_height as i64
            - self.pixel_offset.0 as i64;

        SmoothScrollTarget {
            target_line,
            target_pixel_offset: PixelOffset(0),
            pixel_distance: distance,
        }
    }

    /// Get the pixel-accurate scrollbar fraction.
    pub fn pixel_accurate_fraction(
        &self,
        top_line: u64,
        max_top_line: u64,
        line_height: u32,
    ) -> ScrollFraction {
        if max_top_line <= 1 || line_height == 0 {
            return ScrollFraction::new(0.0);
        }
        let line_fraction = (top_line.saturating_sub(1)) as f64 / (max_top_line - 1) as f64;
        let pixel_contribution =
            self.pixel_offset.0 as f64 / (line_height as f64 * (max_top_line - 1) as f64);
        ScrollFraction::new(line_fraction + pixel_contribution)
    }

    /// Get the current scroll mode based on engine state.
    pub fn scroll_mode(&self) -> ScrollMode {
        if self.enabled {
            ScrollMode::Smooth
        } else {
            ScrollMode::Line
        }
    }

    /// Reset to line-level scrolling (pixel_offset = 0).
    pub fn reset(&mut self) {
        self.pixel_offset = PixelOffset(0);
        self.target_top_line = None;
        self.target_pixel_offset = None;
    }

    /// Whether an animation is currently in progress.
    pub fn is_animating(&self) -> bool {
        self.target_top_line.is_some()
    }

    /// Set the animation target.
    pub fn set_target(&mut self, target_line: u64, target_offset: PixelOffset) {
        self.target_top_line = Some(target_line);
        self.target_pixel_offset = Some(target_offset);
    }

    /// Clear the animation target (animation complete).
    pub fn clear_target(&mut self) {
        self.target_top_line = None;
        self.target_pixel_offset = None;
    }
}

impl Default for SmoothScrollEngine {
    fn default() -> Self {
        Self::new()
    }
}
