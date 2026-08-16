//! Aggregated settings struct for the whitespace-and-guides subsystem.

use crate::modes::{
    EdgeMode, IndentGuideMode, TabDrawMode, WhitespaceVisibility, WrapIndentMode, WrapVisualFlag,
    WrapVisualLocation,
};
use crate::types::EdgeProperties;

/// Aggregated snapshot of all effective whitespace-and-guides settings.
///
/// Rebuilt on hot-reload. Immutable after construction.
///
/// Addresses: Requirement 9 AC 9.2
#[derive(Debug, Clone, PartialEq)]
pub struct WhitespaceSettings {
    // -- Whitespace visibility --
    /// The current whitespace visibility mode.
    pub visibility: WhitespaceVisibility,
    /// The tab character drawing style.
    pub tab_draw_mode: TabDrawMode,
    /// The size of whitespace glyphs (min 1).
    pub whitespace_size: u8,

    // -- Indent guides --
    /// The current indent guide display mode.
    pub indent_guide_mode: IndentGuideMode,
    /// The column of the currently highlighted (active) guide, if any.
    pub active_guide_column: Option<u32>,

    // -- Edge column --
    /// The edge indicator mode.
    pub edge_mode: EdgeMode,
    /// The single-edge column position.
    pub edge_column: u32,
    /// Multi-edge entries for `MultiLine` mode.
    pub edge_columns: Vec<EdgeProperties>,

    // -- Wrap markers --
    /// Which wrap markers to display.
    pub wrap_visual_flags: WrapVisualFlag,
    /// Where wrap markers are positioned relative to text.
    pub wrap_visual_location: WrapVisualLocation,
    /// How continuation sub-lines are indented.
    pub wrap_indent_mode: WrapIndentMode,
    /// Fixed indent offset for `Fixed` mode.
    pub wrap_start_indent: u32,

    // -- Derived state --
    /// Whether word wrap is currently active.
    pub wrap_active: bool,
    /// The document's tab size.
    pub tab_size: u32,
    /// The document's indent size.
    pub indent_size: u32,
}

impl Default for WhitespaceSettings {
    fn default() -> Self {
        Self {
            visibility: WhitespaceVisibility::default(),
            tab_draw_mode: TabDrawMode::default(),
            whitespace_size: 1,
            indent_guide_mode: IndentGuideMode::default(),
            active_guide_column: None,
            edge_mode: EdgeMode::default(),
            edge_column: 80,
            edge_columns: Vec::new(),
            wrap_visual_flags: WrapVisualFlag::default(),
            wrap_visual_location: WrapVisualLocation::default(),
            wrap_indent_mode: WrapIndentMode::default(),
            wrap_start_indent: 0,
            wrap_active: false,
            tab_size: 4,
            indent_size: 4,
        }
    }
}

impl WhitespaceSettings {
    /// Check whether any whitespace glyphs would be rendered.
    pub fn is_whitespace_visible(&self) -> bool {
        self.visibility != WhitespaceVisibility::Invisible
    }

    /// Check whether indent guides would be rendered.
    pub fn has_indent_guides(&self) -> bool {
        self.indent_guide_mode != IndentGuideMode::None
    }

    /// Check whether any edge indicator is active.
    pub fn has_edge_indicator(&self) -> bool {
        self.edge_mode != EdgeMode::None
    }

    /// Check whether wrap markers can appear (requires wrap active + flags set).
    pub fn has_wrap_markers(&self) -> bool {
        self.wrap_active && self.wrap_visual_flags.bits() != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_have_correct_values() {
        // Validates: Requirement 1.2, 2.3, 2.6, 3.2, 5.2, 6.2, 7.2
        let settings = WhitespaceSettings::default();
        assert_eq!(settings.visibility, WhitespaceVisibility::Invisible);
        assert_eq!(settings.tab_draw_mode, TabDrawMode::LongArrow);
        assert_eq!(settings.whitespace_size, 1);
        assert_eq!(settings.indent_guide_mode, IndentGuideMode::None);
        assert_eq!(settings.active_guide_column, None);
        assert_eq!(settings.edge_mode, EdgeMode::None);
        assert_eq!(settings.edge_column, 80);
        assert!(settings.edge_columns.is_empty());
        assert_eq!(settings.wrap_visual_flags, WrapVisualFlag::NONE);
        assert_eq!(settings.wrap_visual_location, WrapVisualLocation::Default);
        assert_eq!(settings.wrap_indent_mode, WrapIndentMode::Fixed);
        assert_eq!(settings.wrap_start_indent, 0);
        assert!(!settings.wrap_active);
    }

    #[test]
    fn is_whitespace_visible_returns_false_for_invisible() {
        let settings = WhitespaceSettings::default();
        assert!(!settings.is_whitespace_visible());
    }

    #[test]
    fn is_whitespace_visible_returns_true_for_non_invisible() {
        let mut settings = WhitespaceSettings::default();
        settings.visibility = WhitespaceVisibility::VisibleAlways;
        assert!(settings.is_whitespace_visible());
    }

    #[test]
    fn has_indent_guides_returns_false_for_none() {
        let settings = WhitespaceSettings::default();
        assert!(!settings.has_indent_guides());
    }

    #[test]
    fn has_indent_guides_returns_true_for_real() {
        let mut settings = WhitespaceSettings::default();
        settings.indent_guide_mode = IndentGuideMode::Real;
        assert!(settings.has_indent_guides());
    }

    #[test]
    fn has_edge_indicator_returns_false_for_none() {
        let settings = WhitespaceSettings::default();
        assert!(!settings.has_edge_indicator());
    }

    #[test]
    fn has_edge_indicator_returns_true_for_line() {
        let mut settings = WhitespaceSettings::default();
        settings.edge_mode = EdgeMode::Line;
        assert!(settings.has_edge_indicator());
    }

    #[test]
    fn has_wrap_markers_requires_both_active_and_flags() {
        let mut settings = WhitespaceSettings::default();
        // Neither active nor flags
        assert!(!settings.has_wrap_markers());

        // Flags set but not active
        settings.wrap_visual_flags = WrapVisualFlag::END;
        assert!(!settings.has_wrap_markers());

        // Active but no flags
        settings.wrap_visual_flags = WrapVisualFlag::NONE;
        settings.wrap_active = true;
        assert!(!settings.has_wrap_markers());

        // Both active and flags
        settings.wrap_visual_flags = WrapVisualFlag::END;
        assert!(settings.has_wrap_markers());
    }
}
