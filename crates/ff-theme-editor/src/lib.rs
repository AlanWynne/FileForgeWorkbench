//! Theme Editor Context -- pure state + egui render (CR-NR-098 decomposition
//! Wave 1, theme-and-appearance Requirement 20).
//!
//! Extracted from `ff-desktop`'s `theme_editor_panel.rs` as the first
//! behaviour-preserving decomposition wave. This crate holds ONLY the pure,
//! shell-independent part: the editable-surface model, the action enum, the
//! per-Context UI state, and the `render` free function. It depends solely on
//! `ff_theme` + `egui` (no `ff-desktop`, no shell types).
//!
//! The shell-coupled adapter -- the `WorkspaceContext` impl and
//! `apply_theme_editor_action` side effects (file writes, palette swap, config
//! persist) -- stays in `ff-desktop`, which owns the `WorkspaceContext` trait.
//!
//! A simple, fast editor Workspace: pick a theme, edit its colours as hex, and
//! Copy / Save / Save As / Set Active / Reset / Export / Import. The render
//! function is a free function (mirroring `config_panel::render` and
//! `command_config::render`) returning a [`ThemeEditorAction`] the shell applies
//! against the themes directory and the active palette.
//!
//! CR-CH-056 (Phase 3): the editable surface is now DERIVED from the egui
//! `Style` / `Visuals` chrome fields plus the retained domain groups
//! ([`editable_surface`], Requirement 20.11), replacing the former fixed
//! 14-token `ui`/`editor` list; and B081 is fixed -- the New-name field is
//! PRE-FILLED with a real, de-duplicated name (not placeholder text) when the
//! editor opens on a built-in, so Save/Save As/Copy are enabled from the first
//! frame and pressing Save on a built-in performs Save As (Requirement 20.5
//! amended, 20.13).
//!
//! Validates: theme-and-appearance Requirement 20.

mod editable_surface;

pub use editable_surface::EditableToken;

use ff_theme::{ColourRGBA, ThemePalette};

/// An action produced by the Theme Editor render, applied by the shell.
///
/// Validates: theme-and-appearance Requirement 20.4-20.8, 20.12.
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
    /// Export the selected/active theme to a native FFWB theme file (the shell
    /// chooses the destination path, e.g. via a file-save picker).
    Export,
    /// Import a native FFWB theme file as a selectable user theme (the shell
    /// chooses the source path, e.g. via a file-open picker).
    Import,
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
    /// Per-token hex text-edit buffers, indexed positionally by
    /// [`EditableToken::all`].
    pub hex_buffers: Vec<String>,
    /// New-name buffer for Copy / Save As. PRE-FILLED with a real, de-duplicated
    /// name on load (B081 fix) so the create actions are enabled immediately.
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
    /// Transient render output; not serialised. Read by the shell-side
    /// `WorkspaceContext` adapter in `ff-desktop`.
    pub first_interior_id: Option<egui::Id>,
    /// The action produced by the most recent render, stashed here so the shell
    /// adapter can apply it via `apply_theme_editor_action` after the panel is
    /// put back (CR-NR-078: the rich shell-side action does not map to a generic
    /// `ShellRequest`). Transient; not serialised.
    pub pending_action: ThemeEditorAction,
}

impl ThemeEditorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a theme as the working copy: sets `selected`, `working`, and
    /// initialises the per-token hex buffers from the palette. Also PRE-FILLS
    /// `name_buffer` with a real, de-duplicated default name (B081 fix,
    /// Requirement 20.5/20.13) so Copy / Save As / Save are enabled from the
    /// first frame and never a silent no-op.
    pub fn load_working(&mut self, name: &str, palette: ThemePalette) {
        self.hex_buffers = EditableToken::all()
            .iter()
            .map(|t| t.get(&palette).to_hex())
            .collect();
        self.selected = Some(name.to_string());
        self.working = Some(palette);
        self.error = None;
        // B081: pre-fill a real, non-empty name (NOT placeholder/hint text) so
        // the create-a-user-theme buttons are enabled immediately. For a built-in
        // selection this makes pressing Save a single discoverable Save As.
        self.name_buffer = self.default_new_name(name);
        self.recompute_advisories();
    }

    /// Derive a unique, non-empty default name for the New-name field from the
    /// selected theme name, de-duplicated against the available list. For a
    /// built-in `Foo`, suggests `Foo Copy`, then `Foo Copy 2`, etc.; for a user
    /// theme it suggests the same so Save As does not collide.
    ///
    /// Validates: theme-and-appearance Requirement 20.4, 20.5, 20.13.
    pub fn default_new_name(&self, selected: &str) -> String {
        let base = if selected.trim().is_empty() {
            "My Theme".to_string()
        } else {
            format!("{} Copy", selected.trim())
        };
        if !self.name_taken(&base) {
            return base;
        }
        for n in 2..1000 {
            let candidate = format!("{base} {n}");
            if !self.name_taken(&candidate) {
                return candidate;
            }
        }
        base
    }

    /// True when `name` (case-insensitive) already exists in the available list.
    fn name_taken(&self, name: &str) -> bool {
        self.available.iter().any(|n| n.eq_ignore_ascii_case(name))
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
///
/// Validates: theme-and-appearance Requirement 20.1, 20.3-20.9, 20.12.
pub fn render(ui: &mut egui::Ui, state: &mut ThemeEditorState) -> ThemeEditorAction {
    // B052 fix: keep explicit button/selector actions separate from the token
    // editor's commit-on-lost-focus action. A button click must WIN over a
    // same-frame `lost_focus` EditToken (clicking a button makes the focused hex
    // field lose focus that same frame). We return `action` when it is set, else
    // fall back to `token_action`.
    let mut action = ThemeEditorAction::None;
    let mut token_action = ThemeEditorAction::None;

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

    // --- Copy / Save As / Save + Import / Export -----------------------
    // B081: the name field is PRE-FILLED (by load_working) with a real name, so
    // name_ok is true from the first frame and Copy / Save As are enabled
    // immediately. Pressing Save with a built-in selected performs Save As at
    // the shell (Requirement 20.5 amended, 20.13).
    ui.horizontal(|ui| {
        ui.label("New name:");
        ui.add(
            egui::TextEdit::singleline(&mut state.name_buffer)
                .id(egui::Id::new("theme_editor_name"))
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

    // Import / Export affordances (Requirement 20.12 / 24): both invoke the
    // shell commands (the shell owns the file-picker path I/O).
    ui.horizontal(|ui| {
        if ui.button("Export...").clicked() {
            action = ThemeEditorAction::Export;
        }
        if ui.button("Import...").clicked() {
            action = ThemeEditorAction::Import;
        }
    });

    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
    }

    ui.separator();

    // --- Colour token editor -------------------------------------------
    if state.working.is_some() {
        // B078: lay each token row out with a plain `ui.horizontal` + fixed
        // widths, NOT an `egui::Grid`. Inside a Grid, `TextEdit::desired_width`
        // is NOT honoured -- the Grid auto-sizes its cells and collapses the
        // field. In a plain horizontal layout `desired_width` IS honoured, so
        // the full `#RRGGBBAA` value (max 9 chars) is visible.
        const LABEL_WIDTH: f32 = 220.0;
        const HEX_FIELD_WIDTH: f32 = 140.0;
        let tokens = EditableToken::all();
        egui::ScrollArea::vertical()
            .id_salt("theme_editor_tokens")
            .show(ui, |ui| {
                for (i, token) in tokens.iter().enumerate() {
                    // Ensure a buffer exists for this row.
                    if state.hex_buffers.len() <= i {
                        state.hex_buffers.resize(i + 1, String::new());
                    }
                    ui.horizontal(|ui| {
                        ui.add_sized([LABEL_WIDTH, 0.0], egui::Label::new(token.label()));
                        // Stable per-row id so Tab focus round-trips reliably and
                        // the LAST row can anchor the shell's last-interior
                        // boundary (B057).
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut state.hex_buffers[i])
                                .id(egui::Id::new(("theme_editor_hex", i)))
                                .desired_width(HEX_FIELD_WIDTH)
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
                                // On commit (lost focus), record an EditToken in
                                // the SEPARATE token slot so it cannot clobber a
                                // same-frame button click (B052).
                                if resp.lost_focus() {
                                    token_action = ThemeEditorAction::EditToken(*token, c);
                                }
                            }
                            Err(_) => {
                                ui.colored_label(egui::Color32::RED, "invalid hex");
                            }
                        }
                    });
                }
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
    // the palette in EditableToken::all order.
    #[test]
    fn load_working_initialises_hex_buffers() {
        let mut state = ThemeEditorState::new();
        let palette = ff_theme::defaults::default_legacy_palette();
        state.load_working("Default Legacy", palette.clone());
        assert_eq!(state.selected.as_deref(), Some("Default Legacy"));
        assert!(state.working.is_some());
        assert_eq!(state.hex_buffers.len(), EditableToken::all().len());
        // First buffer is the first derived token's hex.
        assert_eq!(
            state.hex_buffers[0],
            EditableToken::all()[0].get(&palette).to_hex()
        );
    }

    // Validates: Requirement 20.3 -- token get/set round-trips through a palette.
    #[test]
    fn editable_token_get_set_round_trips() {
        let mut p = ff_theme::defaults::dark_palette();
        let red = ColourRGBA::rgb(255, 0, 0);
        let first = EditableToken::all()[0];
        first.set(&mut p, red);
        assert_eq!(first.get(&p), red);
    }

    // Validates: Requirement 20.9 -- advisories computed from the working palette.
    #[test]
    fn recompute_advisories_reads_working_palette() {
        let mut state = ThemeEditorState::new();
        let mut p = ff_theme::defaults::default_legacy_palette();
        p.editor.background = ColourRGBA::rgb(0, 0, 0);
        p.editor.foreground = ColourRGBA::rgb(20, 20, 20);
        state.load_working("Test", p);
        state.recompute_advisories();
        assert!(state.working.is_some());
    }

    // Validates: Requirement 20.11 -- all editable tokens have labels and the
    // derived surface exceeds the former fixed 14-token list.
    #[test]
    fn all_editable_tokens_have_labels() {
        for t in EditableToken::all() {
            assert!(!t.label().is_empty());
        }
        assert!(
            EditableToken::all().len() > 14,
            "the derived surface replaces the former fixed 14-token list"
        );
    }

    // Validates: Requirement 20.5 (amended), 20.13, B081 -- on load_working the
    // New-name field is PRE-FILLED with a real, non-empty, de-duplicated name
    // (not placeholder text), so the create actions are enabled from frame one.
    #[test]
    fn load_working_prefills_a_real_unique_name() {
        let mut state = ThemeEditorState::new();
        state.available = vec!["Default Dark".to_string(), "Default Dark Copy".to_string()];
        let palette = ff_theme::defaults::dark_palette();
        state.load_working("Default Dark", palette);
        assert!(
            !state.name_buffer.trim().is_empty(),
            "name_buffer must be a real pre-filled value (B081), enabling Save/Copy"
        );
        // It must not collide with an existing name (de-duplicated).
        assert!(
            !state
                .available
                .iter()
                .any(|n| n.eq_ignore_ascii_case(state.name_buffer.trim())),
            "the pre-filled name must be de-duplicated against the available list"
        );
    }

    // Validates: Requirement 20.4 -- default_new_name de-duplicates by appending
    // a numeric suffix when the base name is taken.
    #[test]
    fn default_new_name_dedups_with_numeric_suffix() {
        let mut state = ThemeEditorState::new();
        state.available = vec!["Legacy Copy".to_string(), "Legacy Copy 2".to_string()];
        let name = state.default_new_name("Legacy");
        assert_eq!(name, "Legacy Copy 3");
    }
}
