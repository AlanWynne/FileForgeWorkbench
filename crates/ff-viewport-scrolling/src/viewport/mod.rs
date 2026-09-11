//! Core viewport state model.
//!
//! `ViewportModel` is the central state container for the visible portion of a
//! document. It is GUI-independent and owned by the editor session.

use crate::cursor::CursorModel;
use crate::display_mapper::DisplayLineMapper;
use crate::events::{ViewportChanged, ViewportObserver};
use crate::types::{PixelOffset, ScrollMode};

/// The core viewport state. GUI-independent, owned by the editor session.
pub struct ViewportModel {
    /// First visible display line (1-based).
    pub(super) top_line: u64,
    /// Number of display lines that fit vertically.
    pub(super) visible_count: u64,
    /// Horizontal scroll position in pixels.
    pub(super) horizontal_offset: u64,
    /// Total display lines in the document.
    pub(super) total_display_lines: u64,
    /// Current scroll mode (Line or Smooth).
    pub(super) scroll_mode: ScrollMode,
    /// Sub-line pixel offset for smooth scrolling.
    pub(super) pixel_offset: PixelOffset,
    /// Line height in pixels (for smooth scroll calculations).
    pub(super) line_height: u32,
    /// Viewport width in pixels.
    pub(super) viewport_width: u64,
    /// Maximum horizontal extent (longest line width minus viewport width).
    pub(super) max_horizontal_extent: u64,
    /// Whether word-wrap is enabled (disables horizontal scrollbar).
    pub(super) word_wrap_enabled: bool,
    /// Display line mapper (optional).
    pub(super) display_mapper: Option<Box<dyn DisplayLineMapper>>,
    /// Registered observers for viewport changes.
    pub(super) observers: Vec<(u64, Box<dyn ViewportObserver>)>,
    /// Next observer ID.
    pub(super) next_observer_id: u64,
    /// Lines per mouse wheel tick.
    pub(super) lines_per_wheel_tick: u32,
}

impl ViewportModel {
    pub fn new() -> Self {
        Self {
            top_line: 1,
            visible_count: 1,
            horizontal_offset: 0,
            total_display_lines: 1,
            scroll_mode: ScrollMode::default(),
            pixel_offset: PixelOffset(0),
            line_height: 16,
            viewport_width: 800,
            max_horizontal_extent: 0,
            word_wrap_enabled: false,
            display_mapper: None,
            observers: Vec::new(),
            next_observer_id: 1,
            lines_per_wheel_tick: 3,
        }
    }

    /// Create with a known document line count.
    pub fn with_line_count(total_display_lines: u64) -> Self {
        let mut model = Self::new();
        model.total_display_lines = total_display_lines.max(1);
        model
    }

    // === Accessors ======================================================

    /// Current top_line (1-based).
    pub fn top_line(&self) -> u64 {
        self.top_line
    }

    /// Current visible_count.
    pub fn visible_count(&self) -> u64 {
        self.visible_count
    }

    /// Current horizontal_offset in pixels.
    pub fn horizontal_offset(&self) -> u64 {
        self.horizontal_offset
    }

    /// Total display lines in the document.
    pub fn total_display_lines(&self) -> u64 {
        self.total_display_lines
    }

    /// Maximum valid top_line: max(1, total_display_lines - visible_count + 1).
    pub fn max_top_line(&self) -> u64 {
        if self.total_display_lines <= self.visible_count {
            1
        } else {
            self.total_display_lines - self.visible_count + 1
        }
    }

    /// Current scroll mode (Line or Smooth).
    pub fn scroll_mode(&self) -> ScrollMode {
        self.scroll_mode
    }

    /// Current sub-line pixel offset (0 in Line mode).
    pub fn pixel_offset(&self) -> PixelOffset {
        self.pixel_offset
    }

    /// Line height in pixels.
    pub fn line_height(&self) -> u32 {
        self.line_height
    }

    /// Viewport width in pixels.
    pub fn viewport_width(&self) -> u64 {
        self.viewport_width
    }

    /// Maximum horizontal extent.
    pub fn max_horizontal_extent(&self) -> u64 {
        self.max_horizontal_extent
    }

    /// Lines per mouse wheel tick.
    pub fn lines_per_wheel_tick(&self) -> u32 {
        self.lines_per_wheel_tick
    }

    /// Whether the vertical scrollbar should be disabled.
    pub fn is_vertical_scrollbar_disabled(&self) -> bool {
        self.total_display_lines <= self.visible_count
    }

    /// Whether the horizontal scrollbar should be disabled.
    pub fn is_horizontal_scrollbar_disabled(&self) -> bool {
        self.word_wrap_enabled || self.max_horizontal_extent == 0
    }

    // === Geometry Configuration =========================================

    /// Update the visible line count (called when GUI window resizes).
    /// Clamps top_line if it now exceeds max_top_line.
    pub fn set_visible_count(&mut self, count: u64) {
        self.visible_count = count.max(1);
        self.clamp_top_line();
    }

    /// Update the total display line count.
    /// Clamps top_line if it now exceeds max_top_line.
    pub fn set_total_display_lines(&mut self, total: u64) {
        self.total_display_lines = total.max(1);
        self.clamp_top_line();
    }

    /// Set the line height in pixels (for smooth scroll calculations).
    pub fn set_line_height(&mut self, height: u32) {
        if height > 0 {
            self.line_height = height;
        }
    }

    /// Set the viewport width in pixels.
    pub fn set_viewport_width(&mut self, width: u64) {
        self.viewport_width = width;
    }

    /// Set the maximum horizontal extent (longest line - viewport width).
    pub fn set_max_horizontal_extent(&mut self, extent: u64) {
        self.max_horizontal_extent = extent;
        if self.horizontal_offset > extent {
            self.horizontal_offset = extent;
        }
    }

    /// Set whether word-wrap is enabled.
    pub fn set_word_wrap_enabled(&mut self, enabled: bool) {
        self.word_wrap_enabled = enabled;
        if enabled {
            self.horizontal_offset = 0;
        }
    }

    /// Set lines per mouse wheel tick.
    pub fn set_lines_per_wheel_tick(&mut self, lines: u32) {
        self.lines_per_wheel_tick = lines.max(1);
    }

    /// Attach a DisplayLineMapper for wrapped/folded content.
    pub fn set_display_mapper(&mut self, mapper: Option<Box<dyn DisplayLineMapper>>) {
        if let Some(ref m) = mapper {
            self.total_display_lines = m.total_display_lines();
        }
        self.display_mapper = mapper;
        self.clamp_top_line();
    }

    /// Set scroll mode (Line or Smooth).
    pub fn set_scroll_mode(&mut self, mode: ScrollMode) {
        self.scroll_mode = mode;
        if mode == ScrollMode::Line {
            self.pixel_offset = PixelOffset(0);
        }
    }

    // === Vertical Scrolling =============================================

    // === Internal Helpers ===============================================

    /// Clamp top_line to [1, max_top_line].
    pub(super) fn clamp_top_line(&mut self) {
        let max = self.max_top_line();
        self.top_line = self.top_line.clamp(1, max);
    }

    /// Emit a ViewportChanged event to all observers.
    pub(super) fn emit_event(&self, cursor: &CursorModel, cursor_triggered: bool) {
        let event = ViewportChanged {
            top_line: self.top_line,
            cursor_line: cursor.cursor_line(),
            cursor_column: cursor.cursor_column(),
            horizontal_offset: self.horizontal_offset,
            cursor_triggered,
        };
        for (_, observer) in &self.observers {
            observer.on_viewport_changed(&event);
        }
    }
}

mod scrolling;

impl Default for ViewportModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollbarFeedback {
    /// Current top_line during drag.
    pub current_line: u64,
    /// Total document lines.
    pub total_lines: u64,
}
