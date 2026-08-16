//! Theme palette structures organised by colour group.
//!
//! The `ThemePalette` is the fully-resolved, immutable palette for the
//! active theme and mode. It composes all colour groups, style slots,
//! font configuration, design tokens, and element colours.

use serde::{Deserialize, Serialize};

use crate::colour::ColourRGBA;
use crate::design_tokens::DesignTokens;
use crate::element::ElementColourMap;
use crate::font::FontConfig;
use crate::mode::VisualMode;
use crate::style_slot::StyleSlotTable;
use crate::token::ColourToken;

/// Colours for the editor content area.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorColours {
    /// Editor content background.
    pub background: ColourRGBA,
    /// Editor content foreground (default text).
    pub foreground: ColourRGBA,
    /// Accent colour for active/focused elements.
    pub accent: ColourRGBA,
    /// Muted/disabled text colour.
    pub muted: ColourRGBA,
    /// Modified document indicator.
    pub modified_indicator: ColourRGBA,
    /// Current line background highlight.
    pub current_line_background: ColourRGBA,
    /// Secondary selection background.
    pub selection_secondary_background: ColourRGBA,
}

/// Colours for syntax-highlighted tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxColours {
    /// Keyword token colour.
    pub keyword: ColourRGBA,
    /// Comment token colour.
    pub comment: ColourRGBA,
    /// String literal colour.
    pub string: ColourRGBA,
    /// Numeric literal colour.
    pub number: ColourRGBA,
    /// Operator colour.
    pub operator: ColourRGBA,
    /// Type name colour.
    pub type_name: ColourRGBA,
    /// Function name colour.
    pub function: ColourRGBA,
    /// Macro colour.
    pub macro_name: ColourRGBA,
    /// Preprocessor directive colour.
    pub preprocessor: ColourRGBA,
    /// Default text colour for unclassified tokens.
    pub default_text: ColourRGBA,
}

/// Colours for file tree panel entries by category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTreeColours {
    /// Non-editable binary file.
    pub binary: ColourRGBA,
    /// FileForge structured file.
    pub structured: ColourRGBA,
    /// Standard text file.
    pub text: ColourRGBA,
    /// Unknown file type.
    pub unknown: ColourRGBA,
    /// Directory entry.
    pub directory: ColourRGBA,
    /// Symbolic link entry.
    pub symlink: ColourRGBA,
}

/// Colours for the tab bar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabBarColours {
    /// Active tab background.
    pub active_bg: ColourRGBA,
    /// Inactive tab background.
    pub inactive_bg: ColourRGBA,
    /// Active tab text.
    pub active_text: ColourRGBA,
    /// Inactive tab text.
    pub inactive_text: ColourRGBA,
    /// Modified indicator in tab.
    pub modified_indicator: ColourRGBA,
    /// Close button colour.
    pub close_button: ColourRGBA,
    /// Drop target highlight for drag-and-drop.
    pub drop_target: ColourRGBA,
}

/// Colours for editor chrome elements (line numbers, margins, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChromeColours {
    /// Cursor row border.
    pub cursor_row_border: ColourRGBA,
    /// Cursor column indicator.
    pub cursor_column_indicator: ColourRGBA,
    /// Line number gutter foreground.
    pub line_number_fg: ColourRGBA,
    /// Line number gutter background.
    pub line_number_bg: ColourRGBA,
    /// Fold margin background.
    pub fold_margin_bg: ColourRGBA,
    /// Fold margin foreground.
    pub fold_margin_fg: ColourRGBA,
    /// Margin separator line.
    pub margin_separator: ColourRGBA,
}

/// Colours for text decorations and markers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecorationColours {
    /// Search match highlight.
    pub search_highlight: ColourRGBA,
    /// Error underline.
    pub error_underline: ColourRGBA,
    /// Warning underline.
    pub warning_underline: ColourRGBA,
    /// Info underline.
    pub info_underline: ColourRGBA,
    /// Added change marker.
    pub change_added: ColourRGBA,
    /// Modified change marker.
    pub change_modified: ColourRGBA,
    /// Deleted change marker.
    pub change_deleted: ColourRGBA,
    /// Bookmark indicator.
    pub bookmark: ColourRGBA,
}

/// Colours for indicators and match highlights.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndicatorColours {
    /// Find match highlight.
    pub find_match: ColourRGBA,
    /// Brace match highlight.
    pub brace_match: ColourRGBA,
    /// Brace mismatch highlight.
    pub brace_mismatch: ColourRGBA,
    /// Hotspot underline.
    pub hotspot_underline: ColourRGBA,
    /// Up to 32 user-defined indicator colours (indexed 0–31).
    pub user_defined: [ColourRGBA; 32],
}

/// Colours for general UI components (panels, buttons, inputs, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiColours {
    /// Panel background.
    pub panel_bg: ColourRGBA,
    /// Panel foreground (field labels).
    pub panel_fg: ColourRGBA,
    /// Panel border.
    pub panel_border: ColourRGBA,
    /// Button background.
    pub button_bg: ColourRGBA,
    /// Button foreground.
    pub button_fg: ColourRGBA,
    /// Button hover state.
    pub button_hover: ColourRGBA,
    /// Input field background.
    pub input_bg: ColourRGBA,
    /// Input field border.
    pub input_border: ColourRGBA,
    /// Input field foreground.
    pub input_fg: ColourRGBA,
    /// Scrollbar track.
    pub scrollbar_track: ColourRGBA,
    /// Scrollbar thumb.
    pub scrollbar_thumb: ColourRGBA,
    /// Tooltip background.
    pub tooltip_bg: ColourRGBA,
    /// Tooltip foreground.
    pub tooltip_fg: ColourRGBA,
    /// Menu bar top-level item text colour.
    pub menu_bar_fg: ColourRGBA,
    /// Primary menu / screen heading background (blue in Legacy, panel_bg in other themes).
    pub primary_menu_bg: ColourRGBA,
}

/// The complete resolved palette for the active theme and mode.
///
/// This struct is immutable after construction and shareable via `Arc`
/// for thread-safe, lock-free read access by all rendering subsystems.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemePalette {
    /// Theme name.
    pub name: String,
    /// Active visual mode.
    pub mode: VisualMode,
    /// Editor content area colours.
    pub editor: EditorColours,
    /// Syntax highlighting colours.
    pub syntax: SyntaxColours,
    /// File tree panel colours.
    pub file_tree: FileTreeColours,
    /// Tab bar colours.
    pub tab_bar: TabBarColours,
    /// Editor chrome colours.
    pub chrome: ChromeColours,
    /// Text decoration colours.
    pub decorations: DecorationColours,
    /// Indicator colours.
    pub indicators: IndicatorColours,
    /// UI component colours.
    pub ui: UiColours,
    /// Style slot table (256 entries).
    pub style_slots: StyleSlotTable,
    /// Font configuration.
    pub fonts: FontConfig,
    /// Design system tokens.
    pub design: DesignTokens,
    /// Element colour map.
    pub elements: ElementColourMap,
}

impl ThemePalette {
    /// Look up a colour by its compile-time token identifier.
    ///
    /// This is the primary colour access method used by rendering code.
    /// The exhaustive match ensures all tokens are handled — adding a new
    /// token variant without updating this method produces a compile error.
    pub fn colour(&self, token: ColourToken) -> ColourRGBA {
        match token {
            ColourToken::EditorBackground => self.editor.background,
            ColourToken::EditorForeground => self.editor.foreground,
            ColourToken::EditorAccent => self.editor.accent,
            ColourToken::EditorMuted => self.editor.muted,
            ColourToken::EditorModifiedIndicator => self.editor.modified_indicator,
            ColourToken::EditorCurrentLineBackground => self.editor.current_line_background,
            ColourToken::EditorSelectionSecondaryBackground => {
                self.editor.selection_secondary_background
            }
            ColourToken::SyntaxKeyword => self.syntax.keyword,
            ColourToken::SyntaxComment => self.syntax.comment,
            ColourToken::SyntaxString => self.syntax.string,
            ColourToken::SyntaxNumber => self.syntax.number,
            ColourToken::SyntaxOperator => self.syntax.operator,
            ColourToken::SyntaxType => self.syntax.type_name,
            ColourToken::SyntaxFunction => self.syntax.function,
            ColourToken::SyntaxMacro => self.syntax.macro_name,
            ColourToken::SyntaxPreprocessor => self.syntax.preprocessor,
            ColourToken::SyntaxDefault => self.syntax.default_text,
            ColourToken::FileTreeBinary => self.file_tree.binary,
            ColourToken::FileTreeStructured => self.file_tree.structured,
            ColourToken::FileTreeText => self.file_tree.text,
            ColourToken::FileTreeUnknown => self.file_tree.unknown,
            ColourToken::FileTreeDirectory => self.file_tree.directory,
            ColourToken::FileTreeSymlink => self.file_tree.symlink,
            ColourToken::TabBarActiveBackground => self.tab_bar.active_bg,
            ColourToken::TabBarInactiveBackground => self.tab_bar.inactive_bg,
            ColourToken::TabBarActiveText => self.tab_bar.active_text,
            ColourToken::TabBarInactiveText => self.tab_bar.inactive_text,
            ColourToken::TabBarModifiedIndicator => self.tab_bar.modified_indicator,
            ColourToken::TabBarCloseButton => self.tab_bar.close_button,
            ColourToken::TabBarDropTargetHighlight => self.tab_bar.drop_target,
            ColourToken::ChromeCursorRowBorder => self.chrome.cursor_row_border,
            ColourToken::ChromeCursorColumnIndicator => self.chrome.cursor_column_indicator,
            ColourToken::ChromeLineNumberForeground => self.chrome.line_number_fg,
            ColourToken::ChromeLineNumberBackground => self.chrome.line_number_bg,
            ColourToken::ChromeFoldMarginBackground => self.chrome.fold_margin_bg,
            ColourToken::ChromeFoldMarginForeground => self.chrome.fold_margin_fg,
            ColourToken::ChromeMarginSeparator => self.chrome.margin_separator,
            ColourToken::DecorationSearchHighlight => self.decorations.search_highlight,
            ColourToken::DecorationErrorUnderline => self.decorations.error_underline,
            ColourToken::DecorationWarningUnderline => self.decorations.warning_underline,
            ColourToken::DecorationInfoUnderline => self.decorations.info_underline,
            ColourToken::DecorationChangeAdded => self.decorations.change_added,
            ColourToken::DecorationChangeModified => self.decorations.change_modified,
            ColourToken::DecorationChangeDeleted => self.decorations.change_deleted,
            ColourToken::DecorationBookmark => self.decorations.bookmark,
            ColourToken::IndicatorFindMatch => self.indicators.find_match,
            ColourToken::IndicatorBraceMatch => self.indicators.brace_match,
            ColourToken::IndicatorBraceMismatch => self.indicators.brace_mismatch,
            ColourToken::IndicatorHotspotUnderline => self.indicators.hotspot_underline,
            ColourToken::UiPanelBackground => self.ui.panel_bg,
            ColourToken::UiPanelForeground => self.ui.panel_fg,
            ColourToken::UiPanelBorder => self.ui.panel_border,
            ColourToken::UiButtonBackground => self.ui.button_bg,
            ColourToken::UiButtonForeground => self.ui.button_fg,
            ColourToken::UiButtonHover => self.ui.button_hover,
            ColourToken::UiInputBackground => self.ui.input_bg,
            ColourToken::UiInputBorder => self.ui.input_border,
            ColourToken::UiInputForeground => self.ui.input_fg,
            ColourToken::UiScrollbarTrack => self.ui.scrollbar_track,
            ColourToken::UiScrollbarThumb => self.ui.scrollbar_thumb,
            ColourToken::UiTooltipBackground => self.ui.tooltip_bg,
            ColourToken::UiTooltipForeground => self.ui.tooltip_fg,
            ColourToken::UiMenuBarForeground => self.ui.menu_bar_fg,
            ColourToken::UiPrimaryMenuBackground => self.ui.primary_menu_bg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults;

    #[test]
    fn palette_colour_lookup_returns_correct_value() {
        // Validates: Requirement 8.7, 8.8
        let palette = defaults::dark_palette();
        let bg = palette.colour(ColourToken::EditorBackground);
        assert_eq!(bg, palette.editor.background);
    }

    #[test]
    fn palette_token_lookup_is_exhaustive() {
        // Validates: Requirement 8.8 — compile-time token safety
        // This test passes if it compiles — the exhaustive match in
        // colour() ensures every token variant is handled.
        let palette = defaults::dark_palette();
        let _bg = palette.colour(ColourToken::EditorBackground);
        let _fg = palette.colour(ColourToken::EditorForeground);
        let _ui = palette.colour(ColourToken::UiTooltipForeground);
    }
}
