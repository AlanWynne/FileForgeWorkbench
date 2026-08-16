//! Compile-time colour token identifiers.
//!
//! Using an enum ensures misspelled token names produce compilation errors
//! rather than runtime failures. Every colour in the theme palette has a
//! corresponding `ColourToken` variant.

use serde::{Deserialize, Serialize};

/// Semantic colour token identifiers covering all palette groups.
///
/// This enum provides compile-time safety for colour lookups. Rendering
/// code uses these tokens to obtain colours from the palette, and any
/// typo or invalid token name will be caught by the compiler.
///
/// # Groups
///
/// Tokens are organised by their palette group:
/// - `Editor*` — editor content area colours
/// - `Syntax*` — syntax highlighting colours
/// - `FileTree*` — file tree panel colours
/// - `TabBar*` — tab bar colours
/// - `Chrome*` — editor chrome (line numbers, margins, etc.)
/// - `Decoration*` — text decorations and markers
/// - `Indicator*` — match highlights and indicators
/// - `Ui*` — general UI component colours
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ColourToken {
    // ── Editor group ─────────────────────────────────────────────────────
    /// Editor content background.
    EditorBackground,
    /// Editor content foreground (default text).
    EditorForeground,
    /// Editor accent colour (active selections, focused elements).
    EditorAccent,
    /// Muted/disabled text colour.
    EditorMuted,
    /// Modified document indicator colour.
    EditorModifiedIndicator,
    /// Current line background highlight.
    EditorCurrentLineBackground,
    /// Secondary selection background (additional selections).
    EditorSelectionSecondaryBackground,

    // ── Syntax group ─────────────────────────────────────────────────────
    /// Keyword token colour.
    SyntaxKeyword,
    /// Comment token colour.
    SyntaxComment,
    /// String literal token colour.
    SyntaxString,
    /// Numeric literal token colour.
    SyntaxNumber,
    /// Operator token colour.
    SyntaxOperator,
    /// Type name token colour.
    SyntaxType,
    /// Function name token colour.
    SyntaxFunction,
    /// Macro token colour.
    SyntaxMacro,
    /// Preprocessor directive token colour.
    SyntaxPreprocessor,
    /// Default text colour for unclassified tokens.
    SyntaxDefault,

    // ── File Tree group ──────────────────────────────────────────────────
    /// Binary file colour.
    FileTreeBinary,
    /// Structured file colour (FileForge format).
    FileTreeStructured,
    /// Text file colour.
    FileTreeText,
    /// Unknown file type colour.
    FileTreeUnknown,
    /// Directory colour.
    FileTreeDirectory,
    /// Symbolic link colour.
    FileTreeSymlink,

    // ── Tab Bar group ────────────────────────────────────────────────────
    /// Active tab background.
    TabBarActiveBackground,
    /// Inactive tab background.
    TabBarInactiveBackground,
    /// Active tab text colour.
    TabBarActiveText,
    /// Inactive tab text colour.
    TabBarInactiveText,
    /// Modified indicator in tab.
    TabBarModifiedIndicator,
    /// Close button colour.
    TabBarCloseButton,
    /// Drop target highlight for drag-and-drop.
    TabBarDropTargetHighlight,

    // ── Chrome group ─────────────────────────────────────────────────────
    /// Cursor row border colour.
    ChromeCursorRowBorder,
    /// Cursor column indicator colour.
    ChromeCursorColumnIndicator,
    /// Line number gutter foreground.
    ChromeLineNumberForeground,
    /// Line number gutter background.
    ChromeLineNumberBackground,
    /// Fold margin background.
    ChromeFoldMarginBackground,
    /// Fold margin foreground.
    ChromeFoldMarginForeground,
    /// Margin separator line colour.
    ChromeMarginSeparator,

    // ── Decorations group ────────────────────────────────────────────────
    /// Search match highlight colour.
    DecorationSearchHighlight,
    /// Error underline colour.
    DecorationErrorUnderline,
    /// Warning underline colour.
    DecorationWarningUnderline,
    /// Info underline colour.
    DecorationInfoUnderline,
    /// Added change marker colour.
    DecorationChangeAdded,
    /// Modified change marker colour.
    DecorationChangeModified,
    /// Deleted change marker colour.
    DecorationChangeDeleted,
    /// Bookmark indicator colour.
    DecorationBookmark,

    // ── Indicators group ─────────────────────────────────────────────────
    /// Find match highlight colour.
    IndicatorFindMatch,
    /// Brace match highlight colour.
    IndicatorBraceMatch,
    /// Brace mismatch highlight colour.
    IndicatorBraceMismatch,
    /// Hotspot underline colour.
    IndicatorHotspotUnderline,

    // ── UI group ─────────────────────────────────────────────────────────
    /// Panel background colour.
    UiPanelBackground,
    /// Panel foreground colour.
    UiPanelForeground,
    /// Panel border colour.
    UiPanelBorder,
    /// Button background colour.
    UiButtonBackground,
    /// Button foreground colour.
    UiButtonForeground,
    /// Button hover state colour.
    UiButtonHover,
    /// Input field background.
    UiInputBackground,
    /// Input field border.
    UiInputBorder,
    /// Input field foreground.
    UiInputForeground,
    /// Scrollbar track colour.
    UiScrollbarTrack,
    /// Scrollbar thumb colour.
    UiScrollbarThumb,
    /// Tooltip background.
    UiTooltipBackground,
    /// Tooltip foreground.
    UiTooltipForeground,
    /// Menu bar top-level item text colour.
    UiMenuBarForeground,
    /// Primary menu / screen heading background colour (blue in Legacy).
    UiPrimaryMenuBackground,
}

impl ColourToken {
    /// Returns the TOML key path for this token (e.g., `"editor.background"`).
    pub fn key_path(&self) -> &'static str {
        match self {
            Self::EditorBackground => "editor.background",
            Self::EditorForeground => "editor.foreground",
            Self::EditorAccent => "editor.accent",
            Self::EditorMuted => "editor.muted",
            Self::EditorModifiedIndicator => "editor.modified_indicator",
            Self::EditorCurrentLineBackground => "editor.current_line_background",
            Self::EditorSelectionSecondaryBackground => "editor.selection_secondary_background",
            Self::SyntaxKeyword => "syntax.keyword",
            Self::SyntaxComment => "syntax.comment",
            Self::SyntaxString => "syntax.string",
            Self::SyntaxNumber => "syntax.number",
            Self::SyntaxOperator => "syntax.operator",
            Self::SyntaxType => "syntax.type",
            Self::SyntaxFunction => "syntax.function",
            Self::SyntaxMacro => "syntax.macro",
            Self::SyntaxPreprocessor => "syntax.preprocessor",
            Self::SyntaxDefault => "syntax.default",
            Self::FileTreeBinary => "file_tree.binary",
            Self::FileTreeStructured => "file_tree.structured",
            Self::FileTreeText => "file_tree.text",
            Self::FileTreeUnknown => "file_tree.unknown",
            Self::FileTreeDirectory => "file_tree.directory",
            Self::FileTreeSymlink => "file_tree.symlink",
            Self::TabBarActiveBackground => "tab_bar.active_background",
            Self::TabBarInactiveBackground => "tab_bar.inactive_background",
            Self::TabBarActiveText => "tab_bar.active_text",
            Self::TabBarInactiveText => "tab_bar.inactive_text",
            Self::TabBarModifiedIndicator => "tab_bar.modified_indicator",
            Self::TabBarCloseButton => "tab_bar.close_button",
            Self::TabBarDropTargetHighlight => "tab_bar.drop_target_highlight",
            Self::ChromeCursorRowBorder => "chrome.cursor_row_border",
            Self::ChromeCursorColumnIndicator => "chrome.cursor_column_indicator",
            Self::ChromeLineNumberForeground => "chrome.line_number_foreground",
            Self::ChromeLineNumberBackground => "chrome.line_number_background",
            Self::ChromeFoldMarginBackground => "chrome.fold_margin_background",
            Self::ChromeFoldMarginForeground => "chrome.fold_margin_foreground",
            Self::ChromeMarginSeparator => "chrome.margin_separator",
            Self::DecorationSearchHighlight => "decorations.search_highlight",
            Self::DecorationErrorUnderline => "decorations.error_underline",
            Self::DecorationWarningUnderline => "decorations.warning_underline",
            Self::DecorationInfoUnderline => "decorations.info_underline",
            Self::DecorationChangeAdded => "decorations.change_added",
            Self::DecorationChangeModified => "decorations.change_modified",
            Self::DecorationChangeDeleted => "decorations.change_deleted",
            Self::DecorationBookmark => "decorations.bookmark",
            Self::IndicatorFindMatch => "indicators.find_match",
            Self::IndicatorBraceMatch => "indicators.brace_match",
            Self::IndicatorBraceMismatch => "indicators.brace_mismatch",
            Self::IndicatorHotspotUnderline => "indicators.hotspot_underline",
            Self::UiPanelBackground => "ui.panel_background",
            Self::UiPanelForeground => "ui.panel_foreground",
            Self::UiPanelBorder => "ui.panel_border",
            Self::UiButtonBackground => "ui.button_background",
            Self::UiButtonForeground => "ui.button_foreground",
            Self::UiButtonHover => "ui.button_hover",
            Self::UiInputBackground => "ui.input_background",
            Self::UiInputBorder => "ui.input_border",
            Self::UiInputForeground => "ui.input_foreground",
            Self::UiScrollbarTrack => "ui.scrollbar_track",
            Self::UiScrollbarThumb => "ui.scrollbar_thumb",
            Self::UiTooltipBackground => "ui.tooltip_background",
            Self::UiTooltipForeground => "ui.tooltip_foreground",
            Self::UiMenuBarForeground => "ui.menu_bar_foreground",
            Self::UiPrimaryMenuBackground => "ui.primary_menu_background",
        }
    }
}
