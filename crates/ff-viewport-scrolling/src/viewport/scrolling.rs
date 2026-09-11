//! Scrolling, wheel, scrollbar, and cursor-movement operations for
//! `ViewportModel`. Split from the model for file-size; operates on the
//! same struct via pub(super) fields/helpers.

use crate::caret_policy::CaretPolicyEngine;
use crate::cursor::CursorModel;
use crate::events::ViewportObserver;
use crate::snapshot::ViewportSnapshot;
use crate::types::{PixelOffset, ScrollFraction, ScrollMode};

use super::{ScrollbarFeedback, ViewportModel};

impl ViewportModel {
    pub fn scroll_page_down(&mut self, cursor: &mut CursorModel) {
        let max = self.max_top_line();
        self.top_line = (self.top_line + self.visible_count).min(max);
        cursor.set_position(self.top_line, cursor.cursor_column());
        self.emit_event(cursor, false);
    }

    /// Scroll up by one page (visible_count lines).
    pub fn scroll_page_up(&mut self, cursor: &mut CursorModel) {
        self.top_line = self.top_line.saturating_sub(self.visible_count).max(1);
        cursor.set_position(self.top_line, cursor.cursor_column());
        self.emit_event(cursor, false);
    }

    /// Scroll down by one line.
    pub fn scroll_line_down(&mut self, cursor: &CursorModel) {
        let max = self.max_top_line();
        if self.top_line < max {
            self.top_line += 1;
        }
        self.emit_event(cursor, false);
    }

    /// Scroll up by one line.
    pub fn scroll_line_up(&mut self, cursor: &CursorModel) {
        if self.top_line > 1 {
            self.top_line -= 1;
        }
        self.emit_event(cursor, false);
    }

    /// Scroll to a specific line (clamped to [1, max_top_line]).
    pub fn scroll_to_line(&mut self, line: u64, cursor: &CursorModel) {
        let max = self.max_top_line();
        self.top_line = line.clamp(1, max);
        self.emit_event(cursor, false);
    }

    /// Scroll to the top of the document.
    pub fn scroll_to_top(&mut self, cursor: &CursorModel) {
        self.top_line = 1;
        self.emit_event(cursor, false);
    }

    /// Scroll to the bottom of the document.
    pub fn scroll_to_bottom(&mut self, cursor: &CursorModel) {
        self.top_line = self.max_top_line();
        self.emit_event(cursor, false);
    }

    /// Handle mouse wheel vertical scroll.
    pub fn scroll_wheel_vertical(&mut self, ticks: i32, cursor: &CursorModel) {
        let lines = (ticks as i64) * (self.lines_per_wheel_tick as i64);
        let new_top = (self.top_line as i64 + lines).clamp(1, self.max_top_line() as i64);
        self.top_line = new_top as u64;
        self.emit_event(cursor, false);
    }

    /// Handle mouse wheel horizontal scroll.
    pub fn scroll_wheel_horizontal(
        &mut self,
        ticks: i32,
        pixels_per_tick: u32,
        cursor: &CursorModel,
    ) {
        let delta = (ticks as i64) * (pixels_per_tick as i64);
        let new_offset =
            (self.horizontal_offset as i64 + delta).clamp(0, self.max_horizontal_extent as i64);
        self.horizontal_offset = new_offset as u64;
        self.emit_event(cursor, false);
    }

    // === Horizontal Scrolling ===========================================

    /// Set horizontal offset (clamped to [0, max_horizontal_extent]).
    pub fn set_horizontal_offset(&mut self, offset: u64, cursor: &CursorModel) {
        self.horizontal_offset = offset.min(self.max_horizontal_extent);
        self.emit_event(cursor, false);
    }

    /// Scroll horizontally to ensure a column pixel position is visible.
    pub fn ensure_column_visible(&mut self, column_pixel_position: u64) {
        if column_pixel_position < self.horizontal_offset {
            self.horizontal_offset = column_pixel_position;
        } else if column_pixel_position >= self.horizontal_offset + self.viewport_width {
            self.horizontal_offset = column_pixel_position
                .saturating_sub(self.viewport_width)
                .saturating_add(1)
                .min(self.max_horizontal_extent);
        }
    }

    // === Scrollbar Interaction ===========================================

    /// Get the vertical scrollbar fraction for the current state.
    pub fn vertical_scrollbar_fraction(&self) -> ScrollFraction {
        let max = self.max_top_line();
        if max <= 1 {
            return ScrollFraction::new(0.0);
        }
        let fraction = (self.top_line - 1) as f64 / (max - 1) as f64;
        ScrollFraction::new(fraction)
    }

    /// Get the vertical scrollbar thumb ratio.
    pub fn vertical_scrollbar_thumb_ratio(&self) -> f64 {
        if self.total_display_lines == 0 {
            return 1.0;
        }
        let ratio = self.visible_count as f64 / self.total_display_lines as f64;
        ratio.clamp(0.0, 1.0)
    }

    /// Apply a vertical scrollbar drag to a fraction position.
    pub fn apply_scrollbar_drag(&mut self, fraction: ScrollFraction, cursor: &CursorModel) {
        let max = self.max_top_line();
        if max <= 1 {
            self.top_line = 1;
        } else {
            let f = fraction.value();
            self.top_line = (1.0 + f * (max - 1) as f64).round() as u64;
            self.top_line = self.top_line.clamp(1, max);
        }
        self.emit_event(cursor, false);
    }

    /// Apply a precision scrollbar drag (Shift+drag).
    pub fn apply_precision_drag(
        &mut self,
        pixel_delta: i32,
        track_height: u32,
        cursor: &CursorModel,
    ) {
        if track_height == 0 {
            return;
        }
        let max = self.max_top_line();
        // Precision mode: 1 pixel = 1 line
        let new_top = (self.top_line as i64 + pixel_delta as i64).clamp(1, max as i64);
        self.top_line = new_top as u64;
        self.emit_event(cursor, false);
    }

    /// Get scrollbar feedback data for tooltip.
    pub fn scrollbar_feedback(&self) -> ScrollbarFeedback {
        ScrollbarFeedback {
            current_line: self.top_line,
            total_lines: self.total_display_lines,
        }
    }

    /// Get horizontal scrollbar fraction.
    pub fn horizontal_scrollbar_fraction(&self) -> ScrollFraction {
        if self.max_horizontal_extent == 0 {
            return ScrollFraction::new(0.0);
        }
        let fraction = self.horizontal_offset as f64 / self.max_horizontal_extent as f64;
        ScrollFraction::new(fraction)
    }

    /// Apply a horizontal scrollbar drag.
    pub fn apply_horizontal_drag(&mut self, fraction: ScrollFraction, cursor: &CursorModel) {
        let offset = (fraction.value() * self.max_horizontal_extent as f64).round() as u64;
        self.horizontal_offset = offset.min(self.max_horizontal_extent);
        self.emit_event(cursor, false);
    }

    // === Cursor-Viewport Coordination ===================================

    /// Move cursor down and adjust viewport if needed.
    pub fn move_cursor_down(
        &mut self,
        cursor: &mut CursorModel,
        target_line_length: u64,
        total_lines: u64,
        policy: &CaretPolicyEngine,
    ) {
        cursor.move_down(target_line_length, total_lines);
        let new_top = policy.compute_vertical_scroll(
            cursor.cursor_line(),
            self.top_line,
            self.visible_count,
            self.max_top_line(),
        );
        self.top_line = new_top;
        self.emit_event(cursor, true);
    }

    /// Move cursor up and adjust viewport if needed.
    pub fn move_cursor_up(
        &mut self,
        cursor: &mut CursorModel,
        target_line_length: u64,
        policy: &CaretPolicyEngine,
    ) {
        cursor.move_up(target_line_length);
        let new_top = policy.compute_vertical_scroll(
            cursor.cursor_line(),
            self.top_line,
            self.visible_count,
            self.max_top_line(),
        );
        self.top_line = new_top;
        self.emit_event(cursor, true);
    }

    /// Move cursor left and adjust horizontal offset if needed.
    pub fn move_cursor_left(&mut self, cursor: &mut CursorModel, policy: &CaretPolicyEngine) {
        cursor.move_left();
        let new_offset = policy.compute_horizontal_scroll(
            cursor.cursor_column(),
            self.horizontal_offset,
            self.viewport_width,
            self.max_horizontal_extent,
        );
        self.horizontal_offset = new_offset;
        self.emit_event(cursor, true);
    }

    /// Move cursor right and adjust horizontal offset if needed.
    pub fn move_cursor_right(
        &mut self,
        cursor: &mut CursorModel,
        line_length: u64,
        policy: &CaretPolicyEngine,
    ) {
        cursor.move_right(line_length);
        let new_offset = policy.compute_horizontal_scroll(
            cursor.cursor_column(),
            self.horizontal_offset,
            self.viewport_width,
            self.max_horizontal_extent,
        );
        self.horizontal_offset = new_offset;
        self.emit_event(cursor, true);
    }

    /// Set cursor position (e.g., click) and adjust viewport.
    pub fn set_cursor_position(
        &mut self,
        cursor: &mut CursorModel,
        line: u64,
        column: u64,
        policy: &CaretPolicyEngine,
    ) {
        cursor.set_position(line, column);
        let new_top = policy.compute_vertical_scroll(
            cursor.cursor_line(),
            self.top_line,
            self.visible_count,
            self.max_top_line(),
        );
        self.top_line = new_top;
        self.emit_event(cursor, true);
    }

    // === Smooth Scrolling ===============================================

    /// Get pixel-accurate scrollbar fraction (accounts for sub-line offset).
    pub fn pixel_accurate_scrollbar_fraction(&self) -> ScrollFraction {
        let max = self.max_top_line();
        if max <= 1 {
            return ScrollFraction::new(0.0);
        }
        let line_fraction = (self.top_line - 1) as f64 / (max - 1) as f64;
        if self.scroll_mode == ScrollMode::Smooth && self.line_height > 0 {
            let pixel_fraction =
                self.pixel_offset.0 as f64 / (self.line_height as f64 * (max - 1) as f64);
            ScrollFraction::new(line_fraction + pixel_fraction)
        } else {
            ScrollFraction::new(line_fraction)
        }
    }

    /// Set pixel offset for smooth scrolling (clamped to [0, line_height)).
    pub fn set_pixel_offset(&mut self, offset: u32) {
        if self.scroll_mode == ScrollMode::Smooth {
            self.pixel_offset = PixelOffset(offset % self.line_height);
        }
    }

    // === Snapshot and Restore ===========================================

    /// Create a serialisable snapshot of the current viewport state.
    pub fn snapshot(&self, cursor: &CursorModel) -> ViewportSnapshot {
        ViewportSnapshot {
            top_line: self.top_line,
            cursor_line: cursor.cursor_line(),
            cursor_column: cursor.cursor_column(),
            horizontal_offset: self.horizontal_offset,
            column_affinity: cursor.column_affinity(),
        }
    }

    /// Restore from a persisted snapshot, clamping to current document bounds.
    pub fn restore(&mut self, snapshot: &ViewportSnapshot, cursor: &mut CursorModel) {
        let max_top = self.max_top_line();
        self.top_line = snapshot.top_line.clamp(1, max_top);
        self.horizontal_offset = snapshot.horizontal_offset.min(self.max_horizontal_extent);

        let max_line = self.total_display_lines;
        let clamped_line = snapshot.cursor_line.clamp(1, max_line);
        let clamped_column = snapshot.cursor_column.max(1);
        cursor.set_position(clamped_line, clamped_column);

        self.emit_event(cursor, false);
    }

    // === Observers ======================================================

    /// Register a viewport observer. Returns the observer ID.
    pub fn add_observer(&mut self, observer: Box<dyn ViewportObserver>) -> u64 {
        let id = self.next_observer_id;
        self.next_observer_id += 1;
        self.observers.push((id, observer));
        id
    }

    /// Remove a viewport observer by ID.
    pub fn remove_observer(&mut self, id: u64) {
        self.observers.retain(|(obs_id, _)| *obs_id != id);
    }
}
