//! Caret line highlight model.
//!
//! Defines the configuration for highlighting the line containing the
//! primary caret — either as a background fill or a frame (border/outline).

use crate::colour::ColourRGBA;
use serde::{Deserialize, Serialize};

/// Caret-line highlight mode.
///
/// Addresses: Requirement 4, criteria 4.1–4.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CaretLineMode {
    /// No caret-line highlighting.
    None,
    /// Border/outline around the caret line.
    #[default]
    Frame,
    /// Solid background fill on the caret line.
    Fill,
}

/// Controls how a colour overlay is composited with underlying content.
///
/// Addresses: Requirement 4, criterion 4.6; Requirement 5, criteria 5.5–5.8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CaretLineLayer {
    /// Opaque background drawn under text.
    #[default]
    Base,
    /// Translucent overlay alpha-blended over text.
    OverText,
}

/// Complete caret-line highlight configuration.
///
/// Addresses: Requirement 4, criteria 4.1–4.13
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaretLineConfig {
    /// Highlight mode: None, Frame, or Fill.
    mode: CaretLineMode,
    /// Frame border width in pixels (stored raw, clamped on use).
    frame_width: u32,
    /// Compositing layer mode.
    layer: CaretLineLayer,
    /// Whether highlight shows when pane is unfocused.
    always_show: bool,
    /// Whether highlight applies only to the wrapped sub-line containing the caret.
    sub_line: bool,
    /// Background/frame colour (element: CaretLineBack).
    colour: ColourRGBA,
}

impl CaretLineConfig {
    /// Creates a new caret line config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the highlight mode.
    pub fn mode(&self) -> CaretLineMode {
        self.mode
    }

    /// Sets the highlight mode.
    pub fn set_mode(&mut self, mode: CaretLineMode) {
        self.mode = mode;
    }

    /// Returns the effective frame width, clamped to [1, line_height / 3].
    ///
    /// Addresses: Requirement 4, criterion 4.5
    pub fn effective_frame_width(&self, line_height: u32) -> u32 {
        let max = (line_height / 3).max(1);
        self.frame_width.clamp(1, max)
    }

    /// Returns the raw configured frame width (before clamping).
    pub fn frame_width(&self) -> u32 {
        self.frame_width
    }

    /// Sets the frame width.
    pub fn set_frame_width(&mut self, width: u32) {
        self.frame_width = width;
    }

    /// Returns whether the caret-line should be shown given the focus state.
    ///
    /// When `always_show` is true, returns true regardless of focus.
    /// When `always_show` is false, returns true only when the pane is focused.
    ///
    /// Addresses: Requirement 4, criterion 4.8
    pub fn should_show(&self, pane_focused: bool) -> bool {
        if self.mode == CaretLineMode::None {
            return false;
        }
        self.always_show || pane_focused
    }

    /// Returns whether the highlight applies only to the wrapped sub-line.
    ///
    /// Addresses: Requirement 4, criterion 4.10
    pub fn applies_to_subline(&self) -> bool {
        self.sub_line
    }

    /// Returns the compositing layer mode.
    pub fn layer(&self) -> CaretLineLayer {
        self.layer
    }

    /// Sets the compositing layer mode.
    pub fn set_layer(&mut self, layer: CaretLineLayer) {
        self.layer = layer;
    }

    /// Returns the always-show flag.
    pub fn always_show(&self) -> bool {
        self.always_show
    }

    /// Sets the always-show flag.
    pub fn set_always_show(&mut self, always_show: bool) {
        self.always_show = always_show;
    }

    /// Returns the sub-line flag.
    pub fn sub_line(&self) -> bool {
        self.sub_line
    }

    /// Sets the sub-line flag.
    pub fn set_sub_line(&mut self, sub_line: bool) {
        self.sub_line = sub_line;
    }

    /// Returns the highlight colour.
    pub fn colour(&self) -> ColourRGBA {
        self.colour
    }

    /// Sets the highlight colour.
    pub fn set_colour(&mut self, colour: ColourRGBA) {
        self.colour = colour;
    }
}

impl Default for CaretLineConfig {
    fn default() -> Self {
        Self {
            mode: CaretLineMode::Frame,
            frame_width: 1,
            layer: CaretLineLayer::Base,
            always_show: false,
            sub_line: false,
            colour: ColourRGBA::rgba(0, 0, 0, 30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_frame() {
        // Validates: Requirement 4.2
        let config = CaretLineConfig::default();
        assert_eq!(config.mode(), CaretLineMode::Frame);
    }

    #[test]
    fn default_frame_width_is_one() {
        // Validates: Requirement 4.3
        let config = CaretLineConfig::default();
        assert_eq!(config.frame_width(), 1);
    }

    #[test]
    fn default_layer_is_base() {
        // Validates: Requirement 4.7
        let config = CaretLineConfig::default();
        assert_eq!(config.layer(), CaretLineLayer::Base);
    }

    #[test]
    fn default_always_show_is_false() {
        // Validates: Requirement 4.9
        let config = CaretLineConfig::default();
        assert!(!config.always_show());
    }

    #[test]
    fn default_sub_line_is_false() {
        // Validates: Requirement 4.11
        let config = CaretLineConfig::default();
        assert!(!config.sub_line());
    }

    #[test]
    fn effective_frame_width_clamps_to_one_minimum() {
        // Validates: Requirement 4.5
        let mut config = CaretLineConfig::default();
        config.set_frame_width(0);
        assert_eq!(config.effective_frame_width(30), 1);
    }

    #[test]
    fn effective_frame_width_clamps_to_line_height_third() {
        // Validates: Requirement 4.5
        let mut config = CaretLineConfig::default();
        config.set_frame_width(50);
        // line_height = 30, max = 30/3 = 10
        assert_eq!(config.effective_frame_width(30), 10);
    }

    #[test]
    fn effective_frame_width_passes_through_valid_value() {
        let mut config = CaretLineConfig::default();
        config.set_frame_width(5);
        // line_height = 30, max = 10, value 5 is valid
        assert_eq!(config.effective_frame_width(30), 5);
    }

    #[test]
    fn effective_frame_width_with_small_line_height() {
        let mut config = CaretLineConfig::default();
        config.set_frame_width(3);
        // line_height = 6, max = 6/3 = 2, value 3 clamps to 2
        assert_eq!(config.effective_frame_width(6), 2);
    }

    #[test]
    fn should_show_returns_false_when_mode_is_none() {
        let mut config = CaretLineConfig::default();
        config.set_mode(CaretLineMode::None);
        assert!(!config.should_show(true));
        assert!(!config.should_show(false));
    }

    #[test]
    fn should_show_returns_true_when_focused() {
        // Validates: Requirement 4.8
        let config = CaretLineConfig::default();
        assert!(config.should_show(true));
    }

    #[test]
    fn should_show_returns_false_when_unfocused_and_not_always_show() {
        // Validates: Requirement 4.8
        let config = CaretLineConfig::default();
        assert!(!config.should_show(false));
    }

    #[test]
    fn should_show_returns_true_when_unfocused_and_always_show() {
        // Validates: Requirement 4.8
        let mut config = CaretLineConfig::default();
        config.set_always_show(true);
        assert!(config.should_show(false));
    }

    #[test]
    fn applies_to_subline_reflects_flag() {
        // Validates: Requirement 4.10
        let mut config = CaretLineConfig::default();
        assert!(!config.applies_to_subline());
        config.set_sub_line(true);
        assert!(config.applies_to_subline());
    }
}
