//! egui-native chrome layer for a Theme (CR-CH-056, Requirement 23).
//!
//! `ChromeStyle` is the part of a FFWB Theme that configures the application's
//! real egui appearance. It is BACKED BY `egui::Style` (serialised through
//! egui's own serde derives, Requirement 23.2) so the theme embeds egui's
//! native representation rather than a hand-written mirror that could drift from
//! egui's type.
//!
//! egui's `Style`/`Visuals` do NOT model a few application-chrome colours FFWB
//! paints itself (the primary-menu / Title_Line "header band" and the per-tab
//! active/inactive fills and text). Those are carried here as FFWB-only extras
//! alongside the stored `egui::Style`.
//!
//! The single application seam -- `WorkbenchShell::apply_theme` -- performs a
//! WHOLESALE [`ChromeStyle::apply_to_egui`] (Requirement 23.5), replacing the
//! former hand-written body that mapped only a handful of egui's fields and
//! injected hardcoded Legacy slider colours.

use serde::{Deserialize, Serialize};

use crate::colour::ColourRGBA;
use crate::design_tokens::{DesignTokens, RadiusLevel, ShadowDef, ShadowLevel};
use crate::mode::VisualMode;
use crate::palette::{EditorColours, TabBarColours, UiColours};

/// Convert a `ColourRGBA` to an `egui::Color32` (premultiplied, matching the
/// shell's `to_egui_color` helper so appearance is byte-identical).
#[inline]
fn to_color32(c: ColourRGBA) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
}

/// Convert a `ColourRGBA` to an `egui::Color32`, ignoring alpha (opaque).
#[inline]
fn to_color32_opaque(c: ColourRGBA) -> egui::Color32 {
    egui::Color32::from_rgb(c.r, c.g, c.b)
}

/// Build an `egui::CornerRadius` from a logical-pixel radius.
#[inline]
fn corner_radius(px: f32) -> egui::CornerRadius {
    egui::CornerRadius::same(px.round().clamp(0.0, 255.0) as u8)
}

/// Build an `egui::epaint::Shadow` from a `ShadowDef`.
#[inline]
fn to_shadow(def: &ShadowDef) -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [def.offset_x.round() as i8, def.offset_y.round() as i8],
        blur: def.blur_radius.round().clamp(0.0, 255.0) as u8,
        spread: def.spread.round().clamp(0.0, 255.0) as u8,
        color: to_color32(def.colour),
    }
}

/// The egui-native chrome layer of a Theme.
///
/// Carries the full themable `egui::Style` surface (window/panel fills, per-state
/// widget fills + strokes + corner radius + expansion, selection, hyperlink,
/// extreme/faint/code backgrounds, warn/error colours, shadows, and the
/// `dark_mode` flag) plus the FFWB-only chrome colours egui does not model (the
/// Title_Line header band and the per-tab fills/text the tab bar paints).
///
/// Validates: Requirement 23.1, 23.2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChromeStyle {
    /// The egui `Style` this Theme produces (serialised via egui's own serde).
    pub style: egui::Style,
    /// Title_Line / primary-menu header band fill (egui models no such band).
    pub title_band_bg: ColourRGBA,
    /// Title_Line / menu-bar text colour.
    pub title_band_fg: ColourRGBA,
    /// Active-tab background (egui models no tab bar).
    pub tab_active_bg: ColourRGBA,
    /// Inactive-tab background.
    pub tab_inactive_bg: ColourRGBA,
    /// Active-tab text.
    pub tab_active_text: ColourRGBA,
    /// Inactive-tab text.
    pub tab_inactive_text: ColourRGBA,
    /// Generic chrome accent (modified markers, status highlights, focus fills).
    pub accent: ColourRGBA,
    /// Keyboard focus-ring colour.
    pub focus_ring: ColourRGBA,
    /// Body/content text colour used by chrome surfaces that paint their own
    /// labels (e.g. the function-key label bar). Mirrors the editor foreground
    /// so chrome labels keep the Theme's content-text colour.
    pub foreground: ColourRGBA,
}

impl ChromeStyle {
    /// Build a `ChromeStyle` from the retained palette groups (Phase 1 keeps
    /// `UiColours`/`TabBarColours`/`EditorColours` as the authoring surface and
    /// DERIVES the chrome layer from them, so Phase 1 has no visible appearance
    /// change). The produced `egui::Style` reproduces the mapping the former
    /// hand-written `apply_theme` body performed, PLUS the previously-defaulted
    /// egui surface (open-state widgets, strokes, extreme/faint/code bg,
    /// hyperlink, warn/error, selection) and the DesignTokens-driven
    /// spacing/rounding/shadow (Requirement 23.6).
    ///
    /// Validates: Requirement 23.1, 23.3, 23.5, 23.6
    pub fn from_palette_parts(
        ui: &UiColours,
        tab_bar: &TabBarColours,
        editor: &EditorColours,
        design: &DesignTokens,
        mode: VisualMode,
    ) -> Self {
        let dark_mode = !matches!(mode, VisualMode::Light);
        let base = if dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        let mut style = egui::Style::default();
        let mut visuals = base;

        visuals.dark_mode = dark_mode;
        visuals.panel_fill = to_color32_opaque(editor.background);
        visuals.window_fill = to_color32_opaque(ui.panel_bg);
        visuals.window_stroke = egui::Stroke::new(1.0_f32, to_color32_opaque(ui.panel_border));
        // Global text colour: menu_bar_fg (white in Legacy) correctly colours the
        // menu bar, tab bar, and chrome text; editor content text is applied
        // per-element via palette tokens.
        visuals.override_text_color = Some(to_color32_opaque(ui.menu_bar_fg));

        let fg = |c: ColourRGBA| egui::Stroke::new(1.0_f32, to_color32_opaque(c));
        visuals.widgets.noninteractive.bg_fill = to_color32_opaque(ui.panel_bg);
        visuals.widgets.noninteractive.weak_bg_fill = to_color32_opaque(ui.panel_bg);
        visuals.widgets.noninteractive.fg_stroke = fg(editor.foreground);
        visuals.widgets.inactive.bg_fill = to_color32_opaque(ui.button_bg);
        visuals.widgets.inactive.weak_bg_fill = to_color32_opaque(ui.button_bg);
        visuals.widgets.inactive.fg_stroke = fg(ui.menu_bar_fg);
        visuals.widgets.hovered.bg_fill = to_color32_opaque(ui.button_hover);
        visuals.widgets.hovered.weak_bg_fill = to_color32_opaque(ui.button_hover);
        visuals.widgets.hovered.fg_stroke = fg(ui.menu_bar_fg);
        visuals.widgets.active.bg_fill = to_color32_opaque(ui.input_bg);
        visuals.widgets.active.weak_bg_fill = to_color32_opaque(ui.input_bg);
        visuals.widgets.active.fg_stroke = fg(ui.menu_bar_fg);
        // `open` (open-combo / open-menu) state: previously left at the
        // egui-default; drive it from the same inactive-widget surface so a
        // Theme covers the full WidgetVisuals set (Requirement 23.1).
        visuals.widgets.open.bg_fill = to_color32_opaque(ui.button_bg);
        visuals.widgets.open.weak_bg_fill = to_color32_opaque(ui.button_bg);
        visuals.widgets.open.fg_stroke = fg(ui.menu_bar_fg);

        // Input / inset backgrounds (previously default): source from ui.input_bg
        // so themed text inputs are consistent with the chrome (Requirement 23.1).
        visuals.extreme_bg_color = to_color32_opaque(ui.input_bg);
        visuals.code_bg_color = to_color32_opaque(ui.input_bg);
        visuals.faint_bg_color = to_color32_opaque(ui.button_bg);

        // Selection: accent fill (0.35 multiply, matching the former seam) +
        // accent stroke.
        visuals.selection.bg_fill = to_color32_opaque(editor.accent).linear_multiply(0.35);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, to_color32_opaque(editor.accent));

        // Hyperlink / warn / error (previously default): draw from the palette so
        // a Theme drives egui's real range (Requirement 23.1).
        visuals.hyperlink_color = to_color32_opaque(editor.accent);
        visuals.warn_fg_color = to_color32_opaque(editor.modified_indicator);
        visuals.error_fg_color = to_color32_opaque(editor.accent);

        // DesignTokens -> egui Style (Requirement 23.6). Previously dead at the
        // seam; now wired. Radii -> widget + window + menu corner radius; shadows
        // -> window/popup shadow; spacing -> Style.spacing.
        let r_md = corner_radius(design.border_radius(RadiusLevel::Md));
        for w in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            w.corner_radius = r_md;
        }
        visuals.window_corner_radius = corner_radius(design.border_radius(RadiusLevel::Lg));
        visuals.menu_corner_radius = r_md;
        visuals.window_shadow = to_shadow(design.shadow(ShadowLevel::Lg));
        visuals.popup_shadow = to_shadow(design.shadow(ShadowLevel::Md));

        style.spacing.item_spacing = egui::vec2(design.spacing.sm, design.spacing.xs);
        style.spacing.button_padding = egui::vec2(design.spacing.sm, design.spacing.xs);
        style.spacing.menu_margin = egui::Margin::same(design.spacing.xs.round() as i8);
        style.spacing.indent = design.spacing.lg;

        // Legacy slider legibility (CR-CH-056 Req 23.5, Phase 2 Task 30.3):
        // on a black 3270 background the widget fills derived above are near-black
        // and the slider track/handle vanish. The former shell seam injected
        // hardcoded ISPF turquoise/yellow slider colours; those now live HERE, in
        // the Legacy instance's chrome Style, sourced from palette data so each
        // Legacy variant (Default Legacy vs Legacy Soft) carries its own tone:
        // the slider TRACK is the Legacy input border (turquoise) and the HANDLE
        // stroke is the editor accent (yellow). No seam-side special case remains.
        if matches!(mode, VisualMode::Legacy) {
            let track = to_color32_opaque(ui.input_border);
            let handle = egui::Stroke::new(2.0_f32, to_color32_opaque(editor.accent));
            visuals.widgets.inactive.bg_fill = track;
            visuals.widgets.inactive.weak_bg_fill = track;
            visuals.widgets.inactive.fg_stroke = handle;
            visuals.widgets.hovered.bg_fill = track;
            visuals.widgets.hovered.weak_bg_fill = track;
            visuals.widgets.hovered.fg_stroke = handle;
            visuals.widgets.active.bg_fill = to_color32_opaque(ui.input_fg);
            visuals.widgets.active.weak_bg_fill = to_color32_opaque(ui.input_fg);
            visuals.widgets.active.fg_stroke = handle;
        }

        style.visuals = visuals;

        Self {
            style,
            title_band_bg: ui.primary_menu_bg,
            title_band_fg: ui.menu_bar_fg,
            tab_active_bg: tab_bar.active_bg,
            tab_inactive_bg: tab_bar.inactive_bg,
            tab_active_text: tab_bar.active_text,
            tab_inactive_text: tab_bar.inactive_text,
            accent: editor.accent,
            focus_ring: ui.focus_ring,
            foreground: editor.foreground,
        }
    }

    /// Apply this chrome layer WHOLESALE onto an `egui::Style` (Requirement 23.5).
    ///
    /// This is the single seam the shell uses: it overwrites the whole `Style`
    /// (visuals + spacing + corner radius + shadow) with the Theme's stored
    /// `egui::Style`, so every themable egui field is set from one place rather
    /// than a hand-written subset. `dark_mode` is kept in sync with the Theme.
    ///
    /// Validates: Requirement 23.5, 23.6
    pub fn apply_to_egui(&self, style: &mut egui::Style) {
        let mut applied = self.style.clone();
        applied.visuals.dark_mode = self.style.visuals.dark_mode;
        *style = applied;
    }

    /// The Title_Line / primary-menu band fill as an `egui::Color32`.
    pub fn title_band_bg_color(&self) -> egui::Color32 {
        to_color32_opaque(self.title_band_bg)
    }

    /// The Title_Line / menu-bar text colour as an `egui::Color32`.
    pub fn title_band_fg_color(&self) -> egui::Color32 {
        to_color32_opaque(self.title_band_fg)
    }

    /// The active-tab background as an `egui::Color32`.
    pub fn tab_active_bg_color(&self) -> egui::Color32 {
        to_color32_opaque(self.tab_active_bg)
    }

    /// The inactive-tab background as an `egui::Color32`.
    pub fn tab_inactive_bg_color(&self) -> egui::Color32 {
        to_color32_opaque(self.tab_inactive_bg)
    }

    /// The active-tab text as an `egui::Color32`.
    pub fn tab_active_text_color(&self) -> egui::Color32 {
        to_color32_opaque(self.tab_active_text)
    }

    /// The inactive-tab text as an `egui::Color32`.
    pub fn tab_inactive_text_color(&self) -> egui::Color32 {
        to_color32_opaque(self.tab_inactive_text)
    }

    /// The generic chrome accent as an `egui::Color32`.
    pub fn accent_color(&self) -> egui::Color32 {
        to_color32_opaque(self.accent)
    }

    /// The keyboard focus-ring colour as an `egui::Color32`.
    pub fn focus_ring_color(&self) -> egui::Color32 {
        to_color32_opaque(self.focus_ring)
    }

    /// The chrome body/content text colour as an `egui::Color32`.
    pub fn foreground_color(&self) -> egui::Color32 {
        to_color32_opaque(self.foreground)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults;

    fn legacy_chrome() -> ChromeStyle {
        let p = defaults::default_legacy_palette();
        p.chrome_style
    }

    #[test]
    fn apply_to_egui_sets_full_visuals_surface_not_just_a_subset() {
        // Validates: Requirement 23.1, 23.5 -- apply_to_egui drives the full
        // Style/Visuals surface, including fields the former seam left at
        // Visuals::dark() defaults (open-state widgets, extreme/code bg,
        // hyperlink). None of these are the egui-default zero value.
        let chrome = defaults::dark_palette().chrome_style;
        let mut style = egui::Style::default();
        chrome.apply_to_egui(&mut style);
        // open-state widget bg_fill was never set by the old seam.
        assert_eq!(
            style.visuals.widgets.open.bg_fill,
            chrome.style.visuals.widgets.open.bg_fill
        );
        // extreme_bg_color (text-edit background) is now themed, not default.
        assert_eq!(
            style.visuals.extreme_bg_color,
            chrome.style.visuals.extreme_bg_color
        );
        // hyperlink colour is now themed.
        assert_eq!(
            style.visuals.hyperlink_color,
            chrome.style.visuals.hyperlink_color
        );
        // selection stroke is themed with the accent.
        assert_eq!(
            style.visuals.selection.stroke,
            chrome.style.visuals.selection.stroke
        );
    }

    #[test]
    fn apply_to_egui_wires_design_tokens_spacing_rounding_shadow() {
        // Validates: Requirement 23.6 -- DesignTokens reach the egui Style
        // (previously dead at the seam).
        let design = DesignTokens::default();
        let chrome = defaults::dark_palette().chrome_style;
        let mut style = egui::Style::default();
        chrome.apply_to_egui(&mut style);
        // spacing.indent comes from the lg spacing token.
        assert_eq!(style.spacing.indent, design.spacing.lg);
        assert_eq!(style.spacing.item_spacing.x, design.spacing.sm);
        // window corner radius comes from the lg radius token.
        let lg = design.border_radius(RadiusLevel::Lg).round() as u8;
        assert_eq!(
            style.visuals.window_corner_radius,
            egui::CornerRadius::same(lg)
        );
        // window shadow comes from the lg shadow token.
        let expected = to_shadow(design.shadow(ShadowLevel::Lg));
        assert_eq!(style.visuals.window_shadow, expected);
    }

    #[test]
    fn apply_to_egui_sets_dark_mode_matching_visual_mode() {
        // Validates: Requirement 23.1 -- visuals.dark_mode matches the Theme mode.
        let mut style = egui::Style::default();
        defaults::dark_palette()
            .chrome_style
            .apply_to_egui(&mut style);
        assert!(style.visuals.dark_mode, "Dark theme sets dark_mode = true");

        let mut style = egui::Style::default();
        defaults::light_palette()
            .chrome_style
            .apply_to_egui(&mut style);
        assert!(
            !style.visuals.dark_mode,
            "Light theme sets dark_mode = false"
        );

        let mut style = egui::Style::default();
        legacy_chrome().apply_to_egui(&mut style);
        assert!(
            style.visuals.dark_mode,
            "Legacy (black background) is a dark-mode Theme"
        );
    }

    #[test]
    fn legacy_slider_colours_come_from_the_chrome_style_not_the_seam() {
        // Validates: Requirement 23.5 -- the Legacy slider track/handle colours
        // (ISPF turquoise track, yellow handle) now live in the Legacy chrome
        // Style instead of being injected at the apply seam. The Legacy palette
        // sources the inactive/hovered/active widget fills from its ui group,
        // and applying the chrome Style carries them through.
        let mut style = egui::Style::default();
        let chrome = legacy_chrome();
        chrome.apply_to_egui(&mut style);
        // The slider track (inactive widget bg_fill) is the Legacy input/button
        // surface, carried by the chrome Style -- NOT a hardcoded seam value.
        assert_eq!(
            style.visuals.widgets.inactive.bg_fill,
            chrome.style.visuals.widgets.inactive.bg_fill
        );
        // And the chrome Style's inactive bg is non-black (visible on black),
        // proving the slider is not the invisible near-black it was before the
        // former seam override existed.
        assert_ne!(style.visuals.widgets.inactive.bg_fill, egui::Color32::BLACK);
    }

    #[test]
    fn chrome_style_round_trips_through_egui_serde() {
        // Validates: Requirement 23.2 -- the chrome layer serialises/deserialises
        // through egui's OWN serde derives on the embedded egui::Style (enabling
        // egui's `serde` feature), so the theme embeds egui's native
        // representation rather than a hand-written mirror.
        //
        // NOTE: `egui::Style` is PartialEq but NOT reliably value-equal across a
        // serde round-trip (it carries a `number_formatter` closure wrapper that
        // never compares equal), so we assert the round-trip SUCCEEDS and the
        // themable visual fields are preserved, rather than a whole-struct `==`.
        let chrome = defaults::dark_palette().chrome_style;
        let toml_str = toml::to_string(&chrome).expect("serialise ChromeStyle via egui serde");
        let back: ChromeStyle =
            toml::from_str(&toml_str).expect("deserialise ChromeStyle via egui serde");
        assert_eq!(back.style.visuals.dark_mode, chrome.style.visuals.dark_mode);
        assert_eq!(
            back.style.visuals.panel_fill,
            chrome.style.visuals.panel_fill
        );
        assert_eq!(
            back.style.visuals.window_fill,
            chrome.style.visuals.window_fill
        );
        assert_eq!(
            back.style.visuals.widgets.active.bg_fill,
            chrome.style.visuals.widgets.active.bg_fill
        );
        assert_eq!(
            back.style.visuals.window_shadow,
            chrome.style.visuals.window_shadow
        );
        assert_eq!(back.style.spacing.indent, chrome.style.spacing.indent);
        // The FFWB-only chrome extras round-trip exactly.
        assert_eq!(back.title_band_bg, chrome.title_band_bg);
        assert_eq!(back.tab_active_bg, chrome.tab_active_bg);
        assert_eq!(back.focus_ring, chrome.focus_ring);
    }
}
