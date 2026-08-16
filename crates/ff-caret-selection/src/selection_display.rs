//! Selection display configuration.
//!
//! Controls visibility, layer mode, and EOL fill for selection rendering.

use crate::caret_line::CaretLineLayer;
use serde::{Deserialize, Serialize};

/// Selection display layer, reusing the same semantics as `CaretLineLayer`.
pub type SelectionLayer = CaretLineLayer;

/// Configuration for how selections are rendered visually.
///
/// Addresses: Requirement 5, criteria 5.1–5.10
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionDisplayConfig {
    /// Whether selections are rendered visually.
    visible: bool,
    /// Compositing layer mode for selection background.
    layer: SelectionLayer,
    /// Whether selection background extends past line-end to right edge.
    eol_filled: bool,
}

impl SelectionDisplayConfig {
    /// Creates a new selection display config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether selections are visible.
    ///
    /// Addresses: Requirement 5, criterion 5.3
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the selection visibility flag.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns whether the selection extends to end-of-line.
    ///
    /// Addresses: Requirement 5, criterion 5.9
    pub fn extends_to_eol(&self) -> bool {
        self.eol_filled
    }

    /// Sets the EOL fill flag.
    pub fn set_eol_filled(&mut self, eol_filled: bool) {
        self.eol_filled = eol_filled;
    }

    /// Returns whether the selection layer is translucent (OverText).
    ///
    /// Addresses: Requirement 5, criteria 5.5, 5.8
    pub fn is_translucent(&self) -> bool {
        self.layer == SelectionLayer::OverText
    }

    /// Returns the compositing layer mode.
    pub fn layer(&self) -> SelectionLayer {
        self.layer
    }

    /// Sets the compositing layer mode.
    pub fn set_layer(&mut self, layer: SelectionLayer) {
        self.layer = layer;
    }
}

impl Default for SelectionDisplayConfig {
    fn default() -> Self {
        Self {
            visible: true,
            layer: SelectionLayer::Base,
            eol_filled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_visibility_is_true() {
        // Validates: Requirement 5.4
        let config = SelectionDisplayConfig::default();
        assert!(config.is_visible());
    }

    #[test]
    fn default_layer_is_base() {
        // Validates: Requirement 5.6
        let config = SelectionDisplayConfig::default();
        assert_eq!(config.layer(), SelectionLayer::Base);
        assert!(!config.is_translucent());
    }

    #[test]
    fn default_eol_filled_is_false() {
        // Validates: Requirement 5.10
        let config = SelectionDisplayConfig::default();
        assert!(!config.extends_to_eol());
    }

    #[test]
    fn is_translucent_returns_true_for_over_text_layer() {
        // Validates: Requirement 5.5
        let mut config = SelectionDisplayConfig::default();
        config.set_layer(SelectionLayer::OverText);
        assert!(config.is_translucent());
    }

    #[test]
    fn is_translucent_returns_false_for_base_layer() {
        let config = SelectionDisplayConfig::default();
        assert!(!config.is_translucent());
    }

    #[test]
    fn set_visible_toggles_visibility() {
        let mut config = SelectionDisplayConfig::default();
        config.set_visible(false);
        assert!(!config.is_visible());
        config.set_visible(true);
        assert!(config.is_visible());
    }

    #[test]
    fn set_eol_filled_toggles_flag() {
        let mut config = SelectionDisplayConfig::default();
        config.set_eol_filled(true);
        assert!(config.extends_to_eol());
    }
}
