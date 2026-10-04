//! # Shell Theme Application and Colour Derivation
//!
//! Theme application (`apply_theme`), the theme-switching commands
//! (`set_theme` / `set_active_theme`), and the Legacy POM / menu colour
//! derivation (`legacy_pom_colours` / `menu_colours`) for WorkbenchShell. Moved
//! out of `render_chrome.rs` verbatim as part of the Phase 2 task 2.2 file-size
//! split; behaviour, method names, signatures, and visibility are unchanged.

use eframe::egui;

use crate::primary_option_menu;

use super::helpers::to_egui_color;
use super::WorkbenchShell;

impl WorkbenchShell {
    pub(super) fn apply_theme(&self, ctx: &egui::Context) {
        let p = &self.palette;
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = to_egui_color(p.editor.background);
        visuals.window_fill = to_egui_color(p.ui.panel_bg);
        visuals.window_stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.ui.panel_border));
        // Use menu_bar_fg as the global text colour — in Legacy this is white (#FFFFFF),
        // which correctly colours menu bar items, tab bar, and chrome text.
        // Editor content text is applied per-element in editor_panel using palette tokens.
        visuals.override_text_color = Some(to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.noninteractive.bg_fill = to_egui_color(p.ui.panel_bg);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.editor.foreground));
        visuals.widgets.inactive.bg_fill = to_egui_color(p.ui.button_bg);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.hovered.bg_fill = to_egui_color(p.ui.button_hover);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.active.bg_fill = to_egui_color(p.ui.input_bg);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.selection.bg_fill = to_egui_color(p.editor.accent).linear_multiply(0.35);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.editor.accent));
        // In Legacy mode the slider track and handle are near-black on black — invisible.
        // Override with high-contrast ISPF colours: turquoise track, yellow handle.
        if p.mode == ff_theme::mode::VisualMode::Legacy {
            let track = egui::Color32::from_rgb(0, 170, 170); // ISPF turquoise
            let handle = egui::Color32::from_rgb(255, 255, 0); // ISPF yellow-hi
            visuals.widgets.inactive.bg_fill = track;
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 210, 210);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 255, 255);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, handle);
        }
        ctx.set_visuals(visuals);
    }

    // === Theme switching ===

    /// Switch to the given visual mode.
    ///
    /// Writes the mode name to the `theme.active` config key so the per-frame
    /// hot-reload block picks it up and the palette is not clobbered next frame.
    ///
    /// CR-CH-024: the `THEME` command is now name-based and routes through
    /// `set_active_theme`; this mode-based helper is retained for the mode
    /// config key path and the `follow_os` opt-out regression test.
    #[allow(dead_code)]
    pub(super) fn set_theme(&mut self, mode: ff_theme::mode::VisualMode) {
        self.palette = ff_theme::defaults::default_palette_for_mode(mode);
        let mode_str = mode.section_name().to_string();

        // An EXPLICIT theme selection opts out of "follow OS" -- otherwise the
        // per-frame follow_os block (update.rs) rebuilds the palette from the OS
        // dark/light preference every frame and clobbers the chosen mode, so the
        // selection never visibly sticks (B039 root cause, confirmed by runtime
        // logging: theme block set Legacy, follow_os reset it to Dark, every
        // frame). theme-and-appearance Req 16.4/16.7: follow_os must not override
        // an explicit user choice. Turn it off before persisting the mode.
        if self
            .config_handle
            .get_bool(ff_config::keys::theme::FOLLOW_OS)
            .unwrap_or(false)
        {
            let _ = self.config_handle.set_user_value(
                ff_config::keys::theme::FOLLOW_OS,
                ff_config::ConfigValue::Boolean(false),
            );
        }

        // Persist the active mode so the per-frame theme-active block reads the
        // same mode back and does not revert the selection. Surface a persist
        // failure instead of swallowing it (was `let _ =`, a hidden revert path).
        if let Err(e) = self.config_handle.set_user_value(
            ff_config::keys::theme::ACTIVE,
            ff_config::ConfigValue::String(mode_str),
        ) {
            self.open_error = Some(format!(
                "Theme changed to {} for this session, but could not be saved: {e}",
                mode.section_name()
            ));
        }

        // CR-NR-074 Req 19.9: keep theme.active_name consistent with the mode so
        // the file-backed resolver and the mode command agree. `THEME <mode>`
        // selects the built-in theme for that mode.
        let builtin_name = ff_theme::defaults::default_palette_for_mode(mode).name;
        let _ = self.config_handle.set_user_value(
            ff_config::keys::theme::ACTIVE_NAME,
            ff_config::ConfigValue::String(builtin_name),
        );
    }

    /// Set the active theme by NAME (a theme file / built-in), loading it,
    /// applying it immediately, and persisting `theme.active_name` so the same
    /// theme is active on the next launch. Used by the Theme editor Set_Active
    /// action (Requirement 20.6) and any name-based theme selection.
    ///
    /// Validates: theme-and-appearance Requirement 19.7
    /// Used by the Theme editor Set_Active action (task 24.5) and any name-based
    /// theme selection, sharing one activation path with startup/hot-reload.
    pub(super) fn set_active_theme(&mut self, name: &str) {
        let themes_dir = self.themes_dir();
        match crate::theme_defaults::load_theme_by_name(name, &themes_dir) {
            Some(palette) => {
                self.palette = palette;
                // An EXPLICIT theme selection opts out of "follow OS" -- otherwise
                // the per-frame follow_os block rebuilds the palette from the OS
                // dark/light preference and clobbers the chosen theme every frame
                // (B039 root cause). theme-and-appearance Req 16.4/16.7.
                if self
                    .config_handle
                    .get_bool(ff_config::keys::theme::FOLLOW_OS)
                    .unwrap_or(false)
                {
                    let _ = self.config_handle.set_user_value(
                        ff_config::keys::theme::FOLLOW_OS,
                        ff_config::ConfigValue::Boolean(false),
                    );
                }
                self.active_theme_file = {
                    let path = themes_dir
                        .join(format!("{}.toml", crate::theme_defaults::theme_slug(name)));
                    std::fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|mt| (path, mt))
                };
                if let Err(e) = self.config_handle.set_user_value(
                    ff_config::keys::theme::ACTIVE_NAME,
                    ff_config::ConfigValue::String(name.to_string()),
                ) {
                    self.open_error = Some(format!(
                        "Theme '{name}' applied for this session, but could not be saved: {e}"
                    ));
                }
            }
            None => {
                self.open_error = Some(format!("Theme '{name}' could not be loaded"));
            }
        }
    }

    // === Legacy POM colours ===

    /// Build `PomColours` for the current palette.
    ///
    /// When the Legacy theme is active, returns ISPF semantic colours.
    /// For all other themes, returns `PomColours::inherited()` so egui
    /// uses its own default colours.
    ///
    /// Validates: Requirement 13 (Legacy Theme Colour Semantics)
    pub(super) fn legacy_pom_colours(&self) -> primary_option_menu::PomColours {
        use ff_theme::mode::VisualMode;
        if self.palette.mode == VisualMode::Legacy {
            primary_option_menu::PomColours::from_palette(&self.palette)
        } else {
            primary_option_menu::PomColours::inherited()
        }
    }

    /// Semantic option + calendar colours for the shared menu renderer, derived
    /// from the same palette semantics as the POM. In Legacy mode the ISPF
    /// white/turquoise/green option scheme and turquoise/reversed calendar are
    /// used; other themes inherit egui colours (PLACEHOLDER).
    ///
    /// Validates: menu-workspace Requirement 2.1a, 2.1b; Requirement 13.4-13.8
    pub(super) fn menu_colours(&self) -> crate::menu_workspace::render::MenuColours {
        let pom = self.legacy_pom_colours();
        crate::menu_workspace::render::MenuColours {
            // Key column = POM option key (white in Legacy).
            option_key: pom.option_key,
            // Command column = POM option label (turquoise in Legacy).
            option_command: pom.option_label,
            // Description column = POM normal body text (green in Legacy).
            description: pom.normal_text,
            calendar_fg: pom.calendar_fg,
            today_bg: pom.today_bg,
            today_fg: pom.today_fg,
            use_today_reverse: pom.today_bg != eframe::egui::Color32::PLACEHOLDER,
        }
    }
}
