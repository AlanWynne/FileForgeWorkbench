//! Derived editable surface for the Theme Editor (CR-CH-056 Task 31.1,
//! theme-and-appearance Requirement 20.11).
//!
//! The editor's editable surface is DERIVED from the egui `Style` / `Visuals`
//! chrome fields (the [`ff_theme::ChromeStyle`]) PLUS the retained domain groups
//! (syntax, gutter, file_tree, decorations, indicators), rather than a
//! hand-maintained fixed list of 14 `ui`/`editor` tokens. Each editable control
//! is a [`EditableToken`] carrying a label plus a get/set pair that reads and
//! writes a single `ColourRGBA` on a [`ThemePalette`].
//!
//! The CHROME tokens edit the Theme's egui-chrome surface. In the CR-CH-056
//! Phase-3 model the chrome `egui::Style` is DERIVED from the flat authoring
//! groups (`ui` / `tab_bar` / `editor`) via
//! [`ff_theme::ChromeStyle::from_palette_parts`] and the on-disk format persists
//! those flat groups (the file format is frozen until Phase 4). So a chrome
//! token reads/writes the flat authoring field that PRODUCES the egui chrome
//! field it is labelled for, which is exactly what round-trips (edit -> working
//! copy -> serialise -> load, Requirement 20.11): the shell re-derives
//! `chrome_style` after each edit (`ThemePalette::rederive_chrome_style`) so the
//! live preview reflects the egui chrome, and the loader re-derives it on load.
//! The DOMAIN tokens read and write the first-class domain groups egui does not
//! model. Adding or changing a chrome authoring field or a domain group colour
//! surfaces here by extending the generated table below -- there is no per-field
//! edit scattered across the render.
//!
//! Validates: theme-and-appearance Requirement 20.11, 23.1, 23.3.

use ff_theme::{ColourRGBA, ThemePalette};

/// A single editable colour control in the Theme Editor's derived surface.
///
/// `get` reads the control's current colour from a palette; `set` writes a new
/// colour into the palette. The pair is the only coupling to the palette shape,
/// so the render loop stays generic over the whole surface.
#[derive(Clone, Copy)]
pub struct EditableToken {
    label: &'static str,
    getter: fn(&ThemePalette) -> ColourRGBA,
    setter: fn(&mut ThemePalette, ColourRGBA),
}

impl std::fmt::Debug for EditableToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditableToken")
            .field("label", &self.label)
            .finish()
    }
}

impl PartialEq for EditableToken {
    fn eq(&self, other: &Self) -> bool {
        // Two tokens are the same control when they carry the same label (labels
        // are unique across the generated surface, asserted by a test).
        self.label == other.label
    }
}

impl Eq for EditableToken {}

impl EditableToken {
    /// Human-readable English label for the token (e.g. "Chrome: panel fill").
    ///
    /// This is the token's STABLE IDENTITY (used by `PartialEq`, tests, and the
    /// positional `all()` ordering); it is NOT localised. The DISPLAYED label is
    /// resolved at the render site via [`EditableToken::label_key`] + the
    /// `ff_i18n::t()` seam (localization Req 4.1), with this English string as
    /// the Identity_Base fallback.
    pub fn label(self) -> &'static str {
        self.label
    }

    /// The stable `ff-i18n` message key for this token's displayed label
    /// (localization Req 4.1, task 7.5). The key is a pure function of the
    /// English identity `label`, so it never drifts from the surface table. The
    /// render site calls `ff_i18n::t(token.label_key())`; a missing catalogue
    /// key falls back to the English `label()` string (resolved by the caller).
    pub fn label_key(self) -> &'static str {
        token_label_key(self.label)
    }

    /// Read the token's current colour from a palette.
    pub fn get(self, p: &ThemePalette) -> ColourRGBA {
        (self.getter)(p)
    }

    /// Write the token's colour into a palette.
    pub fn set(self, p: &mut ThemePalette, c: ColourRGBA) {
        (self.setter)(p, c)
    }

    /// The full derived editable surface in display order: the egui chrome
    /// authoring fields first, then each retained domain group.
    pub fn all() -> &'static [EditableToken] {
        SURFACE
    }
}

/// Generate an [`EditableToken`] for a `ColourRGBA` field accessed through
/// `$path` (relative to `p`). Used for BOTH the flat chrome authoring groups
/// (`ui` / `tab_bar` / `editor`) and the domain groups: both are `ColourRGBA`
/// fields that serialise, so both round-trip (edit -> serialise -> load).
macro_rules! colour_token {
    ($label:literal, $($path:tt)+) => {
        EditableToken {
            label: $label,
            getter: |p: &ThemePalette| p.$($path)+,
            setter: |p: &mut ThemePalette, c: ColourRGBA| {
                p.$($path)+ = c;
            },
        }
    };
}

/// Map a token's stable English identity `label` to its `ff-i18n` message key.
///
/// A pure, total function over the surface table's labels. Every label in
/// `SURFACE` has a case here; an unmapped label (only possible if a new token is
/// added to `SURFACE` without a matching key) falls back to a stable "unknown"
/// key so the render still resolves (and the English `label()` remains the
/// Identity_Base fallback). A unit test asserts every `SURFACE` label is mapped.
///
/// Validates: localization Requirement 4.1 (task 7.5).
fn token_label_key(label: &str) -> &'static str {
    match label {
        // --- egui chrome -----------------------------------------------------
        "Chrome: panel fill" => "theme-editor-token-chrome-panel-fill",
        "Chrome: window fill" => "theme-editor-token-chrome-window-fill",
        "Chrome: panel border" => "theme-editor-token-chrome-panel-border",
        "Chrome: extreme (input) bg" => "theme-editor-token-chrome-extreme-input-bg",
        "Chrome: raised (button) bg" => "theme-editor-token-chrome-raised-button-bg",
        "Chrome: hovered bg" => "theme-editor-token-chrome-hovered-bg",
        "Chrome: menu bar text" => "theme-editor-token-chrome-menu-bar-text",
        "Chrome: input foreground" => "theme-editor-token-chrome-input-foreground",
        "Chrome: input border" => "theme-editor-token-chrome-input-border",
        "Chrome: focus ring" => "theme-editor-token-chrome-focus-ring",
        "Chrome: accent" => "theme-editor-token-chrome-accent",
        "Chrome: title band bg" => "theme-editor-token-chrome-title-band-bg",
        "Chrome: tab active bg" => "theme-editor-token-chrome-tab-active-bg",
        "Chrome: tab inactive bg" => "theme-editor-token-chrome-tab-inactive-bg",
        "Chrome: tab active text" => "theme-editor-token-chrome-tab-active-text",
        "Chrome: tab inactive text" => "theme-editor-token-chrome-tab-inactive-text",
        // --- domain: syntax --------------------------------------------------
        "Syntax: keyword" => "theme-editor-token-syntax-keyword",
        "Syntax: comment" => "theme-editor-token-syntax-comment",
        "Syntax: string" => "theme-editor-token-syntax-string",
        "Syntax: number" => "theme-editor-token-syntax-number",
        "Syntax: operator" => "theme-editor-token-syntax-operator",
        "Syntax: type" => "theme-editor-token-syntax-type",
        "Syntax: function" => "theme-editor-token-syntax-function",
        "Syntax: macro" => "theme-editor-token-syntax-macro",
        "Syntax: preprocessor" => "theme-editor-token-syntax-preprocessor",
        "Syntax: default text" => "theme-editor-token-syntax-default-text",
        // --- domain: editor content -----------------------------------------
        "Editor: foreground" => "theme-editor-token-editor-foreground",
        "Editor: muted" => "theme-editor-token-editor-muted",
        "Editor: current line bg" => "theme-editor-token-editor-current-line-bg",
        // --- domain: gutter --------------------------------------------------
        "Gutter: line number fg" => "theme-editor-token-gutter-line-number-fg",
        "Gutter: line number bg" => "theme-editor-token-gutter-line-number-bg",
        "Gutter: fold margin bg" => "theme-editor-token-gutter-fold-margin-bg",
        "Gutter: fold margin fg" => "theme-editor-token-gutter-fold-margin-fg",
        "Gutter: margin separator" => "theme-editor-token-gutter-margin-separator",
        "Gutter: cursor row border" => "theme-editor-token-gutter-cursor-row-border",
        "Gutter: cursor column" => "theme-editor-token-gutter-cursor-column",
        // --- domain: file tree -----------------------------------------------
        "File tree: binary" => "theme-editor-token-file-tree-binary",
        "File tree: structured" => "theme-editor-token-file-tree-structured",
        "File tree: text" => "theme-editor-token-file-tree-text",
        "File tree: unknown" => "theme-editor-token-file-tree-unknown",
        "File tree: directory" => "theme-editor-token-file-tree-directory",
        "File tree: symlink" => "theme-editor-token-file-tree-symlink",
        // --- domain: decorations ---------------------------------------------
        "Decoration: search highlight" => "theme-editor-token-decoration-search-highlight",
        "Decoration: error underline" => "theme-editor-token-decoration-error-underline",
        "Decoration: warning underline" => "theme-editor-token-decoration-warning-underline",
        "Decoration: info underline" => "theme-editor-token-decoration-info-underline",
        "Decoration: change added" => "theme-editor-token-decoration-change-added",
        "Decoration: change modified" => "theme-editor-token-decoration-change-modified",
        "Decoration: change deleted" => "theme-editor-token-decoration-change-deleted",
        "Decoration: bookmark" => "theme-editor-token-decoration-bookmark",
        // --- domain: indicators ----------------------------------------------
        "Indicator: find match" => "theme-editor-token-indicator-find-match",
        "Indicator: brace match" => "theme-editor-token-indicator-brace-match",
        "Indicator: brace mismatch" => "theme-editor-token-indicator-brace-mismatch",
        "Indicator: hotspot underline" => "theme-editor-token-indicator-hotspot-underline",
        _ => "theme-editor-token-unknown",
    }
}

/// The derived editable surface. CHROME tokens map onto the flat authoring
/// fields that PRODUCE the egui `Visuals` chrome (Req 23.1, via
/// `ChromeStyle::from_palette_parts`); DOMAIN tokens map onto the retained
/// first-class groups (Req 23.3). This table IS the generated field list --
/// extending the chrome authoring surface or a domain group is a one-line table
/// edit here, not a change scattered through the render.
static SURFACE: &[EditableToken] = &[
    // --- egui chrome: window / panel / surface fills (ui + editor groups) --
    colour_token!("Chrome: panel fill", editor.background),
    colour_token!("Chrome: window fill", ui.panel_bg),
    colour_token!("Chrome: panel border", ui.panel_border),
    colour_token!("Chrome: extreme (input) bg", ui.input_bg),
    colour_token!("Chrome: raised (button) bg", ui.button_bg),
    colour_token!("Chrome: hovered bg", ui.button_hover),
    colour_token!("Chrome: menu bar text", ui.menu_bar_fg),
    colour_token!("Chrome: input foreground", ui.input_fg),
    colour_token!("Chrome: input border", ui.input_border),
    colour_token!("Chrome: focus ring", ui.focus_ring),
    colour_token!("Chrome: accent", editor.accent),
    // --- egui chrome: title band + tabs (ui + tab_bar groups) --------------
    colour_token!("Chrome: title band bg", ui.primary_menu_bg),
    colour_token!("Chrome: tab active bg", tab_bar.active_bg),
    colour_token!("Chrome: tab inactive bg", tab_bar.inactive_bg),
    colour_token!("Chrome: tab active text", tab_bar.active_text),
    colour_token!("Chrome: tab inactive text", tab_bar.inactive_text),
    // --- domain: syntax ----------------------------------------------------
    colour_token!("Syntax: keyword", syntax.keyword),
    colour_token!("Syntax: comment", syntax.comment),
    colour_token!("Syntax: string", syntax.string),
    colour_token!("Syntax: number", syntax.number),
    colour_token!("Syntax: operator", syntax.operator),
    colour_token!("Syntax: type", syntax.type_name),
    colour_token!("Syntax: function", syntax.function),
    colour_token!("Syntax: macro", syntax.macro_name),
    colour_token!("Syntax: preprocessor", syntax.preprocessor),
    colour_token!("Syntax: default text", syntax.default_text),
    // --- domain: editor content -------------------------------------------
    colour_token!("Editor: foreground", editor.foreground),
    colour_token!("Editor: muted", editor.muted),
    colour_token!("Editor: current line bg", editor.current_line_background),
    // --- domain: gutter ----------------------------------------------------
    colour_token!("Gutter: line number fg", gutter.line_number_fg),
    colour_token!("Gutter: line number bg", gutter.line_number_bg),
    colour_token!("Gutter: fold margin bg", gutter.fold_margin_bg),
    colour_token!("Gutter: fold margin fg", gutter.fold_margin_fg),
    colour_token!("Gutter: margin separator", gutter.margin_separator),
    colour_token!("Gutter: cursor row border", gutter.cursor_row_border),
    colour_token!("Gutter: cursor column", gutter.cursor_column_indicator),
    // --- domain: file tree -------------------------------------------------
    colour_token!("File tree: binary", file_tree.binary),
    colour_token!("File tree: structured", file_tree.structured),
    colour_token!("File tree: text", file_tree.text),
    colour_token!("File tree: unknown", file_tree.unknown),
    colour_token!("File tree: directory", file_tree.directory),
    colour_token!("File tree: symlink", file_tree.symlink),
    // --- domain: decorations ----------------------------------------------
    colour_token!("Decoration: search highlight", decorations.search_highlight),
    colour_token!("Decoration: error underline", decorations.error_underline),
    colour_token!(
        "Decoration: warning underline",
        decorations.warning_underline
    ),
    colour_token!("Decoration: info underline", decorations.info_underline),
    colour_token!("Decoration: change added", decorations.change_added),
    colour_token!("Decoration: change modified", decorations.change_modified),
    colour_token!("Decoration: change deleted", decorations.change_deleted),
    colour_token!("Decoration: bookmark", decorations.bookmark),
    // --- domain: indicators ------------------------------------------------
    colour_token!("Indicator: find match", indicators.find_match),
    colour_token!("Indicator: brace match", indicators.brace_match),
    colour_token!("Indicator: brace mismatch", indicators.brace_mismatch),
    colour_token!("Indicator: hotspot underline", indicators.hotspot_underline),
];

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 20.11 -- the editable surface is derived (chrome
    // fields + domain groups) and is larger than the former fixed 14-token list.
    #[test]
    fn derived_surface_is_larger_than_the_former_fixed_fourteen() {
        assert!(
            EditableToken::all().len() > 14,
            "the derived surface must exceed the former fixed 14-token list"
        );
    }

    // Validates: Requirement 20.11 -- every control has a unique, non-empty label.
    #[test]
    fn all_tokens_have_unique_non_empty_labels() {
        let tokens = EditableToken::all();
        for t in tokens {
            assert!(!t.label().is_empty(), "label must be non-empty");
        }
        let mut labels: Vec<&str> = tokens.iter().map(|t| t.label()).collect();
        let total = labels.len();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), total, "labels must be unique");
    }

    // Validates: Requirement 20.11 -- a chrome token round-trips through its
    // backing authoring field (edit -> read back).
    #[test]
    fn chrome_token_round_trips_through_authoring_field() {
        let mut p = ff_theme::defaults::dark_palette();
        let window_fill = EditableToken::all()
            .iter()
            .find(|t| t.label() == "Chrome: window fill")
            .copied()
            .expect("window fill token exists");
        let red = ColourRGBA::rgb(255, 0, 0);
        window_fill.set(&mut p, red);
        assert_eq!(window_fill.get(&p), red);
        assert_eq!(
            p.ui.panel_bg, red,
            "the edit is written into the backing authoring field"
        );
    }

    // Validates: Requirement 20.11, 23.3 -- a domain token round-trips through
    // its domain group.
    #[test]
    fn domain_token_round_trips_through_domain_group() {
        let mut p = ff_theme::defaults::dark_palette();
        let keyword = EditableToken::all()
            .iter()
            .find(|t| t.label() == "Syntax: keyword")
            .copied()
            .expect("syntax keyword token exists");
        let green = ColourRGBA::rgb(0, 255, 0);
        keyword.set(&mut p, green);
        assert_eq!(keyword.get(&p), green);
        assert_eq!(p.syntax.keyword, green);
    }

    // Validates: localization Requirement 4.1 (task 7.5) -- every token in the
    // surface maps to a dedicated i18n label key (never the "unknown" fallback),
    // and every key is unique. Guards against a new token being added to SURFACE
    // without a matching key in `token_label_key`.
    #[test]
    fn every_token_has_a_unique_dedicated_label_key() {
        let tokens = EditableToken::all();
        let mut keys: Vec<&str> = Vec::with_capacity(tokens.len());
        for t in tokens {
            let key = t.label_key();
            assert_ne!(
                key,
                "theme-editor-token-unknown",
                "token {:?} has no dedicated label key -- add it to token_label_key",
                t.label()
            );
            keys.push(key);
        }
        let total = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), total, "token label keys must be unique");
    }

    // Validates: Requirement 20.11 -- the surface covers both chrome and every
    // retained domain group.
    #[test]
    fn surface_covers_chrome_and_all_domain_groups() {
        let labels: Vec<&str> = EditableToken::all().iter().map(|t| t.label()).collect();
        assert!(labels.iter().any(|l| l.starts_with("Chrome:")));
        assert!(labels.iter().any(|l| l.starts_with("Syntax:")));
        assert!(labels.iter().any(|l| l.starts_with("Gutter:")));
        assert!(labels.iter().any(|l| l.starts_with("File tree:")));
        assert!(labels.iter().any(|l| l.starts_with("Decoration:")));
        assert!(labels.iter().any(|l| l.starts_with("Indicator:")));
    }
}
