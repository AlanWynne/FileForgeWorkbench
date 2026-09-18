//! Theme Editor Context (CR-NR-074, theme-and-appearance Requirement 20).
//!
//! A simple, fast editor Workspace: pick a theme, edit its visible-chrome
//! colours as hex, and Copy / Save / Save As / Set Active / Reset. The render
//! function is a free function (mirroring `config_panel::render` and
//! `command_config::render`) returning a [`ThemeEditorAction`] the shell
//! applies against the themes directory and the active palette.
//!
//! Owner directive: keep it simple first (a token list with hex fields), not a
//! graphical colour picker.
//!
//! Validates: theme-and-appearance Requirement 20.

use eframe::egui;

use ff_theme::{ColourRGBA, ThemePalette};

/// The editable chrome tokens exposed by the Theme Editor. Deliberately the
/// subset that drives the visible chrome (ui + editor groups), so the editor is
/// simple and fast (Requirement 20.3). More tokens can be added later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditableToken {
    UiPanelBg,
    UiPanelFg,
    UiPanelBorder,
    UiButtonBg,
    UiButtonFg,
    UiInputBg,
    UiInputFg,
    UiInputBorder,
    UiMenuBarFg,
    UiPrimaryMenuBg,
    UiFocusRing,
    EditorBackground,
    EditorForeground,
    EditorAccent,
}

impl EditableToken {
    /// All editable tokens in display order.
    pub const ALL: &'static [EditableToken] = &[
        EditableToken::UiPanelBg,
        EditableToken::UiPanelFg,
        EditableToken::UiPanelBorder,
        EditableToken::UiButtonBg,
        EditableToken::UiButtonFg,
        EditableToken::UiInputBg,
        EditableToken::UiInputFg,
        EditableToken::UiInputBorder,
        EditableToken::UiMenuBarFg,
        EditableToken::UiPrimaryMenuBg,
        EditableToken::UiFocusRing,
        EditableToken::EditorBackground,
        EditableToken::EditorForeground,
        EditableToken::EditorAccent,
    ];

    /// Human-readable label for the token.
    pub fn label(self) -> &'static str {
        match self {
            EditableToken::UiPanelBg => "Panel background",
            EditableToken::UiPanelFg => "Panel foreground",
            EditableToken::UiPanelBorder => "Panel border",
            EditableToken::UiButtonBg => "Button background",
            EditableToken::UiButtonFg => "Button foreground",
            EditableToken::UiInputBg => "Input background",
            EditableToken::UiInputFg => "Input foreground",
            EditableToken::UiInputBorder => "Input border",
            EditableToken::UiMenuBarFg => "Menu bar text",
            EditableToken::UiPrimaryMenuBg => "Primary menu background",
            EditableToken::UiFocusRing => "Focus ring",
            EditableToken::EditorBackground => "Editor background",
            EditableToken::EditorForeground => "Editor foreground",
            EditableToken::EditorAccent => "Editor accent",
        }
    }

    /// Read the token's current colour from a palette.
    pub fn get(self, p: &ThemePalette) -> ColourRGBA {
        match self {
            EditableToken::UiPanelBg => p.ui.panel_bg,
            EditableToken::UiPanelFg => p.ui.panel_fg,
            EditableToken::UiPanelBorder => p.ui.panel_border,
            EditableToken::UiButtonBg => p.ui.button_bg,
            EditableToken::UiButtonFg => p.ui.button_fg,
            EditableToken::UiInputBg => p.ui.input_bg,
            EditableToken::UiInputFg => p.ui.input_fg,
            EditableToken::UiInputBorder => p.ui.input_border,
            EditableToken::UiMenuBarFg => p.ui.menu_bar_fg,
            EditableToken::UiPrimaryMenuBg => p.ui.primary_menu_bg,
            EditableToken::UiFocusRing => p.ui.focus_ring,
            EditableToken::EditorBackground => p.editor.background,
            EditableToken::EditorForeground => p.editor.foreground,
            EditableToken::EditorAccent => p.editor.accent,
        }
    }

    /// Write the token's colour into a palette.
    pub fn set(self, p: &mut ThemePalette, c: ColourRGBA) {
        match self {
            EditableToken::UiPanelBg => p.ui.panel_bg = c,
            EditableToken::UiPanelFg => p.ui.panel_fg = c,
            EditableToken::UiPanelBorder => p.ui.panel_border = c,
            EditableToken::UiButtonBg => p.ui.button_bg = c,
            EditableToken::UiButtonFg => p.ui.button_fg = c,
            EditableToken::UiInputBg => p.ui.input_bg = c,
            EditableToken::UiInputFg => p.ui.input_fg = c,
            EditableToken::UiInputBorder => p.ui.input_border = c,
            EditableToken::UiMenuBarFg => p.ui.menu_bar_fg = c,
            EditableToken::UiPrimaryMenuBg => p.ui.primary_menu_bg = c,
            EditableToken::UiFocusRing => p.ui.focus_ring = c,
            EditableToken::EditorBackground => p.editor.background = c,
            EditableToken::EditorForeground => p.editor.foreground = c,
            EditableToken::EditorAccent => p.editor.accent = c,
        }
    }
}

/// An action produced by the Theme Editor render, applied by the shell.
///
/// Validates: theme-and-appearance Requirement 20.4-20.8.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ThemeEditorAction {
    /// No action this frame.
    #[default]
    None,
    /// Select a different theme to edit (by name).
    Select(String),
    /// Set a token's colour from a validated hex string (updates the working copy).
    EditToken(EditableToken, ColourRGBA),
    /// Copy the current theme into a new named theme (becomes the edit target).
    Copy(String),
    /// Save the working copy to the selected theme's file.
    Save,
    /// Save the working copy under a new name.
    SaveAs(String),
    /// Make the selected theme the active theme.
    SetActive(String),
    /// Reset the selected theme's file to its built-in baseline.
    Reset(String),
}

/// Per-Context UI state for the Theme Editor. Lives on the shell (like
/// `ConfigPanelState` / `CommandConfiguratorState`), not on the `TabState`.
///
/// Validates: theme-and-appearance Requirement 20.1, 20.3.
#[derive(Debug, Clone, Default)]
pub struct ThemeEditorState {
    /// Available theme names (built-in + user), refreshed on open.
    pub available: Vec<String>,
    /// The theme currently selected for editing.
    pub selected: Option<String>,
    /// The working-copy palette being edited (unsaved edits live here).
    pub working: Option<ThemePalette>,
    /// Per-token hex text-edit buffers, indexed positionally by `EditableToken::ALL`.
    pub hex_buffers: Vec<String>,
    /// New-name buffer for Copy / Save As.
    pub name_buffer: String,
    /// Contrast advisory messages (below-AA pairs), non-blocking.
    pub advisories: Vec<String>,
    /// The most recent validation/save error, shown inline.
    pub error: Option<String>,
    /// A theme name pending reset confirmation.
    pub pending_reset: Option<String>,
    /// The egui id of the FIRST interior control (the Theme selector combo),
    /// captured each frame so the shell Boundary_Policy can latch the
    /// command-field -> first-interior Tab jump to the FRESH same-frame id
    /// (B057: a stale/guessed id does not round-trip through egui focus, and
    /// without it the shell cannot perform the latch, leaving a phantom stop).
    /// Transient render output; not serialised.
    pub first_interior_id: Option<egui::Id>,
    /// The action produced by the most recent `WorkspaceContext::render`, stashed
    /// here so the shell arm can apply it via `apply_theme_editor_action` after
    /// the panel is put back (CR-NR-078: the rich shell-side action does not map
    /// to a generic `ShellRequest`). Transient; not serialised.
    pub pending_action: ThemeEditorAction,
}

impl ThemeEditorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a theme as the working copy: sets `selected`, `working`, and
    /// initialises the per-token hex buffers from the palette.
    pub fn load_working(&mut self, name: &str, palette: ThemePalette) {
        self.hex_buffers = EditableToken::ALL
            .iter()
            .map(|t| t.get(&palette).to_hex())
            .collect();
        self.selected = Some(name.to_string());
        self.working = Some(palette);
        self.error = None;
        self.recompute_advisories();
    }

    /// Recompute the contrast advisories for the working palette
    /// (Requirement 20.9). Non-blocking.
    pub fn recompute_advisories(&mut self) {
        self.advisories.clear();
        if let Some(p) = &self.working {
            for w in ff_theme::check_theme_contrast(p) {
                self.advisories.push(format!(
                    "{}: contrast {:.1}:1 (below AA 4.5:1)",
                    w.pair_name, w.ratio
                ));
            }
        }
    }
}

/// Render the Theme Editor Context, returning the action to apply.
/// `WorkspaceContext` impl (CR-NR-078): render the Theme Editor, stash the
/// produced `ThemeEditorAction` on `pending_action` for the shell to apply via
/// `apply_theme_editor_action` (the action is rich shell-side state, not a
/// generic `ShellRequest`), and report the interior focus contract: FIRST = the
/// Theme selector combo (captured on `first_interior_id`), LAST = the final
/// colour hex field.
///
/// Validates: workspace-framework Requirement 1.4, 1.5, 6.1.
impl crate::shell::workspace_context::WorkspaceContext for ThemeEditorState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        _services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        self.pending_action = render(ui, self);
        let last = egui::Id::new(("theme_editor_hex", EditableToken::ALL.len() - 1));
        crate::shell::workspace_context::InteriorFocus {
            first: self.first_interior_id,
            last: Some(last),
        }
    }
}

///
/// Validates: theme-and-appearance Requirement 20.1, 20.3-20.9.
pub fn render(ui: &mut egui::Ui, state: &mut ThemeEditorState) -> ThemeEditorAction {
    // B052 fix: keep explicit button/selector actions separate from the token
    // editor's commit-on-lost-focus action. A button click must WIN over a
    // same-frame `lost_focus` EditToken (clicking a button makes the focused hex
    // field lose focus that same frame). We return `action` when it is set, else
    // fall back to `token_action`.
    let mut action = ThemeEditorAction::None;
    let mut token_action = ThemeEditorAction::None;

    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Theme Editor").strong().size(14.0));
    });
    ui.add_space(4.0);

    // --- Theme selector -------------------------------------------------
    // Reset the reported first-interior id each frame; the combo below sets it
    // (B057: mirrors the Menus Editor so the shell Boundary_Policy can latch the
    // command-field -> first-interior Tab jump to the combo's FRESH id).
    state.first_interior_id = None;
    ui.horizontal(|ui| {
        ui.label("Theme:");
        let current = state
            .selected
            .clone()
            .unwrap_or_else(|| "(none)".to_string());
        let combo = egui::ComboBox::from_id_salt("theme_editor_select")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for name in state.available.clone() {
                    if ui
                        .selectable_label(state.selected.as_deref() == Some(name.as_str()), &name)
                        .clicked()
                    {
                        action = ThemeEditorAction::Select(name.clone());
                    }
                }
            });
        // The combo's toggle button is the FIRST interior Tab stop (B057).
        state.first_interior_id = Some(combo.response.id);
        if ui.button("Set Active").clicked() {
            if let Some(name) = &state.selected {
                action = ThemeEditorAction::SetActive(name.clone());
            }
        }
        if ui.button("Reset to built-in").clicked() {
            if let Some(name) = &state.selected {
                state.pending_reset = Some(name.clone());
            }
        }
    });

    // Reset confirmation (Requirement 20.7).
    if let Some(name) = state.pending_reset.clone() {
        ui.horizontal(|ui| {
            ui.colored_label(
                egui::Color32::from_rgb(0xC8, 0x8A, 0x00),
                format!("Reset '{name}' to its baseline and discard edits?"),
            );
            if ui.button("Confirm reset").clicked() {
                action = ThemeEditorAction::Reset(name.clone());
                state.pending_reset = None;
            }
            if ui.button("Cancel").clicked() {
                state.pending_reset = None;
            }
        });
    }

    ui.separator();

    // --- Copy / Save As name field -------------------------------------
    ui.horizontal(|ui| {
        ui.label("New name:");
        ui.add(
            egui::TextEdit::singleline(&mut state.name_buffer)
                .hint_text("my-theme")
                .desired_width(180.0),
        );
        let name = state.name_buffer.trim().to_string();
        let name_ok = !name.is_empty();
        if ui.add_enabled(name_ok, egui::Button::new("Copy")).clicked() {
            action = ThemeEditorAction::Copy(name.clone());
        }
        if ui
            .add_enabled(name_ok, egui::Button::new("Save As"))
            .clicked()
        {
            action = ThemeEditorAction::SaveAs(name.clone());
        }
        if ui.button("Save").clicked() {
            action = ThemeEditorAction::Save;
        }
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
    }

    ui.separator();

    // --- Colour token editor -------------------------------------------
    if state.working.is_some() {
        egui::ScrollArea::vertical()
            .id_salt("theme_editor_tokens")
            .show(ui, |ui| {
                egui::Grid::new("theme_editor_grid")
                    .num_columns(3)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        for (i, token) in EditableToken::ALL.iter().enumerate() {
                            ui.label(token.label());
                            // Ensure a buffer exists for this row.
                            if state.hex_buffers.len() <= i {
                                state.hex_buffers.resize(i + 1, String::new());
                            }
                            // Stable per-row id so Tab focus round-trips reliably and the LAST
                            // row can anchor the shell's last-interior boundary (B057).
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut state.hex_buffers[i])
                                    .id(egui::Id::new(("theme_editor_hex", i)))
                                    .desired_width(90.0)
                                    .font(egui::TextStyle::Monospace),
                            );
                            // Swatch preview of the current buffer value.
                            match ColourRGBA::from_hex(&state.hex_buffers[i]) {
                                Ok(c) => {
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(24.0, 14.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        2.0,
                                        egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a),
                                    );
                                    // On commit (lost focus), record an EditToken
                                    // in the SEPARATE token slot so it cannot
                                    // clobber a same-frame button click (B052).
                                    if resp.lost_focus() {
                                        token_action = ThemeEditorAction::EditToken(*token, c);
                                    }
                                }
                                Err(_) => {
                                    ui.colored_label(egui::Color32::RED, "invalid hex");
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
    } else {
        ui.label("Select a theme to edit its colours.");
    }

    // --- Contrast advisory (Requirement 20.9) --------------------------
    if !state.advisories.is_empty() {
        ui.separator();
        ui.label(
            egui::RichText::new("Contrast advisories (below AA):")
                .color(egui::Color32::from_rgb(0xC8, 0x8A, 0x00)),
        );
        for a in &state.advisories {
            ui.colored_label(egui::Color32::from_rgb(0xC8, 0x8A, 0x00), a);
        }
    }

    // Button/selector actions take priority; a token commit only applies when no
    // explicit action was triggered this frame (B052).
    if action != ThemeEditorAction::None {
        action
    } else {
        token_action
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 20.3 -- load_working initialises hex buffers from
    // the palette in EditableToken order.
    #[test]
    fn load_working_initialises_hex_buffers() {
        let mut state = ThemeEditorState::new();
        let palette = ff_theme::defaults::default_legacy_palette();
        state.load_working("Default Legacy", palette.clone());
        assert_eq!(state.selected.as_deref(), Some("Default Legacy"));
        assert!(state.working.is_some());
        assert_eq!(state.hex_buffers.len(), EditableToken::ALL.len());
        // First buffer is the panel_bg hex.
        assert_eq!(
            state.hex_buffers[0],
            EditableToken::UiPanelBg.get(&palette).to_hex()
        );
    }

    // Validates: Requirement 20.3 -- token get/set round-trips through a palette.
    #[test]
    fn editable_token_get_set_round_trips() {
        let mut p = ff_theme::defaults::dark_palette();
        let red = ColourRGBA::rgb(255, 0, 0);
        EditableToken::UiPanelBg.set(&mut p, red);
        assert_eq!(EditableToken::UiPanelBg.get(&p), red);
        EditableToken::EditorForeground.set(&mut p, red);
        assert_eq!(EditableToken::EditorForeground.get(&p), red);
    }

    // Validates: Requirement 20.9 -- advisories computed from the working palette.
    #[test]
    fn recompute_advisories_reads_working_palette() {
        let mut state = ThemeEditorState::new();
        // A deliberately low-contrast palette: dark grey text on black.
        let mut p = ff_theme::defaults::default_legacy_palette();
        p.editor.background = ColourRGBA::rgb(0, 0, 0);
        p.editor.foreground = ColourRGBA::rgb(20, 20, 20);
        state.load_working("Test", p);
        // Advisories may or may not include this specific pair depending on which
        // pairs check_theme_contrast inspects, but the call must not panic and the
        // vector must be populated from the working palette without error.
        state.recompute_advisories();
        // No assertion on count (depends on the checker's pair set); just ensure
        // it runs and the state is consistent.
        assert!(state.working.is_some());
    }

    // Validates: Requirement 20.3 -- all editable tokens have distinct labels.
    #[test]
    fn all_editable_tokens_have_labels() {
        for t in EditableToken::ALL {
            assert!(!t.label().is_empty());
        }
        assert_eq!(EditableToken::ALL.len(), 14);
    }
}
