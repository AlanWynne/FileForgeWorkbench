//! Per-editor-instance wrap state.
//!
//! Each open document tab owns one `WrapState`. The wrap mode and settings
//! are independent across all editor instances.

use crate::boundary::WrapBoundary;
use crate::config::WrapConfig;
use crate::indent::WrapIndentMode;
use crate::mode::WrapMode;
use crate::visual_flags::WrapVisualFlags;

/// Result of a wrap mode change operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapModeChange {
    /// The mode before the change.
    pub old_mode: WrapMode,
    /// The mode after the change.
    pub new_mode: WrapMode,
}

/// Per-editor-instance wrap state.
///
/// Each open document tab owns one `WrapState`. The wrap mode and settings
/// are independent across all editor instances.
///
/// Addresses: Requirement 2 (Per-Document Wrap State)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapState {
    /// The current wrap mode for this editor instance.
    mode: WrapMode,

    /// The current wrap boundary (viewport or fixed column).
    boundary: WrapBoundary,

    /// The wrap indent mode for continuation lines.
    indent_mode: WrapIndentMode,

    /// The fixed indent amount (characters) when indent_mode is Fixed.
    indent_amount: u8,

    /// Visual flag style for continuation markers.
    visual_flags: WrapVisualFlags,

    /// The last active wrap mode before switching to None.
    /// Used by WRAP TOGGLE to restore the previous mode.
    last_active_mode: WrapMode,
}

impl WrapState {
    /// Create a new wrap state initialised from configuration defaults.
    ///
    /// Addresses: Requirement 2 AC 1, AC 2
    pub fn from_config(config: &WrapConfig) -> Self {
        Self {
            mode: config.default_mode,
            boundary: config.wrap_column,
            indent_mode: config.indent_mode,
            indent_amount: config.indent_amount,
            visual_flags: config.visual_flags,
            last_active_mode: WrapMode::Word,
        }
    }

    /// Get the current wrap mode.
    pub fn mode(&self) -> WrapMode {
        self.mode
    }

    /// Get the current wrap boundary.
    pub fn boundary(&self) -> WrapBoundary {
        self.boundary
    }

    /// Get the wrap indent mode.
    pub fn indent_mode(&self) -> WrapIndentMode {
        self.indent_mode
    }

    /// Get the fixed indent amount.
    pub fn indent_amount(&self) -> u8 {
        self.indent_amount
    }

    /// Get the visual flags setting.
    pub fn visual_flags(&self) -> WrapVisualFlags {
        self.visual_flags
    }

    /// Whether wrap is currently active (mode is not None).
    pub fn is_active(&self) -> bool {
        self.mode.is_active()
    }

    /// Set the wrap mode. Records previous active mode for toggle restore.
    ///
    /// Returns the mode change information.
    pub fn set_mode(&mut self, mode: WrapMode) -> WrapModeChange {
        let old_mode = self.mode;
        if self.mode.is_active() {
            self.last_active_mode = self.mode;
        }
        self.mode = mode;
        WrapModeChange {
            old_mode,
            new_mode: mode,
        }
    }

    /// Set the wrap boundary.
    pub fn set_boundary(&mut self, boundary: WrapBoundary) {
        self.boundary = boundary;
    }

    /// Get the last active mode (for TOGGLE restoration).
    pub fn last_active_mode(&self) -> WrapMode {
        self.last_active_mode
    }

    /// Compute the effective wrap width in columns for the current state.
    ///
    /// Returns `viewport_width` when boundary is `Viewport`, or the column
    /// value when boundary is `Column(n)`.
    pub fn effective_wrap_width(&self, viewport_width: u16) -> u16 {
        self.boundary.effective_column(viewport_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::WrapColumn;

    #[test]
    fn from_config_default_produces_none_mode() {
        // Validates: Requirement 2.2
        let config = WrapConfig::default();
        let state = WrapState::from_config(&config);
        assert_eq!(state.mode(), WrapMode::None);
    }

    #[test]
    fn from_config_word_default_produces_word_mode() {
        // Validates: Requirement 12.5
        let config = WrapConfig {
            default_mode: WrapMode::Word,
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        assert_eq!(state.mode(), WrapMode::Word);
        assert!(state.is_active());
    }

    #[test]
    fn two_instances_are_independent() {
        // Validates: Requirement 2.3
        let config = WrapConfig::default();
        let mut state_a = WrapState::from_config(&config);
        let state_b = WrapState::from_config(&config);

        state_a.set_mode(WrapMode::Word);
        assert_eq!(state_a.mode(), WrapMode::Word);
        assert_eq!(state_b.mode(), WrapMode::None);
    }

    #[test]
    fn set_mode_records_last_active_mode() {
        let config = WrapConfig::default();
        let mut state = WrapState::from_config(&config);

        // Start from None — last_active_mode defaults to Word
        assert_eq!(state.last_active_mode(), WrapMode::Word);

        // Setting to Character from None doesn't update last_active_mode
        // because current mode (None) is not active
        state.set_mode(WrapMode::Character);
        assert_eq!(state.last_active_mode(), WrapMode::Word);

        // Now switching from Character (active) to None updates last_active_mode
        state.set_mode(WrapMode::None);
        assert_eq!(state.last_active_mode(), WrapMode::Character);
    }

    #[test]
    fn set_mode_returns_change_info() {
        let config = WrapConfig::default();
        let mut state = WrapState::from_config(&config);

        let change = state.set_mode(WrapMode::Word);
        assert_eq!(change.old_mode, WrapMode::None);
        assert_eq!(change.new_mode, WrapMode::Word);
    }

    #[test]
    fn effective_wrap_width_viewport() {
        // Validates: Requirement 4.2
        let config = WrapConfig::default();
        let state = WrapState::from_config(&config);
        assert_eq!(state.effective_wrap_width(120), 120);
    }

    #[test]
    fn effective_wrap_width_column() {
        // Validates: Requirement 4.3
        let config = WrapConfig {
            wrap_column: WrapBoundary::Column(WrapColumn::new(80).unwrap()),
            ..WrapConfig::default()
        };
        let state = WrapState::from_config(&config);
        assert_eq!(state.effective_wrap_width(120), 80);
    }

    #[test]
    fn is_active_delegates_to_mode() {
        let config = WrapConfig::default();
        let mut state = WrapState::from_config(&config);
        assert!(!state.is_active());
        state.set_mode(WrapMode::Word);
        assert!(state.is_active());
    }
}
