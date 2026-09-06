//! Key Configuration Dialog — Validates: Requirement 20
//!
//! A non-modal egui window for editing all function key assignments:
//! - Default (Global) scope
//! - One tab per named context (pom, editor, settings, files, hex, toolchain)
//!
//! Each scope shows a 24-row grid (F1–F24) with columns for plain, Shift,
//! Ctrl, and Alt modifier variants, each with a Command and Description field.

use std::collections::HashMap;

use eframe::egui;
use ff_config::{ConfigHandle, ConfigValue};
use ff_keys::{FunctionKey, KeyBinding, KeyMap, KeyMapResolver, ModifiedKey};

/// The named context scopes shown as tabs in the dialog.
const CONTEXT_NAMES: &[&str] = &["pom", "editor", "settings", "files", "hex", "toolchain"];

/// Which scope tab is currently active.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ScopeTab {
    Default,
    Context(String),
}

/// State for one editable row in the grid (one function key, all four modifier variants).
#[derive(Debug, Clone)]
struct KeyRow {
    key: FunctionKey,
    /// [plain, shift, ctrl, alt]
    commands: [String; 4],
    descriptions: [String; 4],
}

impl KeyRow {
    fn from_map(key: FunctionKey, map: &KeyMap) -> Self {
        let modifiers = [
            ModifiedKey::plain(key),
            ModifiedKey::shift(key),
            ModifiedKey::ctrl(key),
            ModifiedKey::alt(key),
        ];
        let mut commands = [String::new(), String::new(), String::new(), String::new()];
        let mut descriptions = [String::new(), String::new(), String::new(), String::new()];
        for (i, mk) in modifiers.iter().enumerate() {
            if let Some(b) = map.get(*mk) {
                commands[i] = b.command().to_string();
                descriptions[i] = b.description().unwrap_or("").to_string();
            }
        }
        Self {
            key,
            commands,
            descriptions,
        }
    }

    /// Apply this row's staged values back into a `KeyMap`.
    fn apply_to_map(&self, map: &mut KeyMap) {
        let modifiers = [
            ModifiedKey::plain(self.key),
            ModifiedKey::shift(self.key),
            ModifiedKey::ctrl(self.key),
            ModifiedKey::alt(self.key),
        ];
        for (i, mk) in modifiers.iter().enumerate() {
            let cmd = self.commands[i].trim();
            if cmd.is_empty() {
                map.remove(*mk);
            } else {
                let desc = self.descriptions[i].trim();
                let binding = if desc.is_empty() {
                    KeyBinding::new(cmd)
                } else {
                    KeyBinding::with_description(cmd, desc)
                };
                map.set(*mk, binding);
            }
        }
    }
}

/// Staged rows for one scope tab.
#[derive(Debug, Clone)]
struct ScopeRows {
    rows: Vec<KeyRow>,
}

impl ScopeRows {
    fn from_map(map: &KeyMap) -> Self {
        let rows = FunctionKey::ALL
            .iter()
            .map(|&k| KeyRow::from_map(k, map))
            .collect();
        Self { rows }
    }

    fn empty_for(source: &str) -> Self {
        Self::from_map(&KeyMap::empty(source))
    }

    fn to_map(&self, source: &str) -> KeyMap {
        let mut map = KeyMap::empty(source);
        for row in &self.rows {
            row.apply_to_map(&mut map);
        }
        map
    }

    /// Serialise the staged rows into a TOML table value suitable for `set_user_value`.
    ///
    /// Each assigned binding is written using the canonical TOML key name
    /// (e.g. `F3`, `SF3`, `CF12`) with a table value `{ command, description? }`
    /// or a plain string when no description is set.
    ///
    /// Validates: Requirement 20.8
    fn to_config_table(&self, source: &str) -> ConfigValue {
        let map = self.to_map(source);
        let mut table: ff_config::ConfigTable = std::collections::BTreeMap::new();
        for (mk, binding) in map.iter() {
            let key_name = mk.toml_name();
            let value = if binding.description().is_some() || binding.label().is_some() {
                let mut entry: ff_config::ConfigTable = std::collections::BTreeMap::new();
                entry.insert(
                    "command".to_string(),
                    ConfigValue::String(binding.command().to_string()),
                );
                if let Some(lbl) = binding.label() {
                    entry.insert("label".to_string(), ConfigValue::String(lbl.to_string()));
                }
                if let Some(desc) = binding.description() {
                    entry.insert(
                        "description".to_string(),
                        ConfigValue::String(desc.to_string()),
                    );
                }
                ConfigValue::Table(entry)
            } else {
                ConfigValue::String(binding.command().to_string())
            };
            table.insert(key_name, value);
        }
        ConfigValue::Table(table)
    }
}

/// Key Configuration Dialog state.
///
/// Validates: Requirement 20.1–20.15
pub struct KeyConfigDialog {
    /// Whether the dialog is currently open.
    pub open: bool,
    active_tab: ScopeTab,
    staged_default: ScopeRows,
    staged_contexts: HashMap<String, ScopeRows>,
    original_default: ScopeRows,
    original_contexts: HashMap<String, ScopeRows>,
}

impl KeyConfigDialog {
    /// Create a new dialog in the closed state with empty staged maps.
    pub fn new() -> Self {
        let empty = ScopeRows::empty_for("global");
        let ctx_map: HashMap<String, ScopeRows> = CONTEXT_NAMES
            .iter()
            .map(|&n| (n.to_string(), ScopeRows::empty_for(n)))
            .collect();
        Self {
            open: false,
            active_tab: ScopeTab::Default,
            staged_default: empty.clone(),
            staged_contexts: ctx_map.clone(),
            original_default: empty,
            original_contexts: ctx_map,
        }
    }

    /// Re-populate staged and original maps from the current resolver state.
    ///
    /// Validates: Requirement 20.6
    pub fn load_from_resolver(&mut self, resolver: &KeyMapResolver) {
        let rows = ScopeRows::from_map(resolver.global_key_map());
        self.staged_default = rows.clone();
        self.original_default = rows;

        for &name in CONTEXT_NAMES {
            let empty = ScopeRows::empty_for(name);
            self.staged_contexts.insert(name.to_string(), empty.clone());
            self.original_contexts.insert(name.to_string(), empty);
        }
    }

    /// Reset the active tab to defaults.
    ///
    /// Validates: Requirement 20.15
    fn reset_active_tab(&mut self) {
        match self.active_tab.clone() {
            ScopeTab::Default => {
                self.staged_default = ScopeRows::from_map(&KeyMap::default_global());
            }
            ScopeTab::Context(name) => {
                self.staged_contexts
                    .insert(name.clone(), ScopeRows::empty_for(&name));
            }
        }
    }

    /// Persist all staged key maps to the user-layer configuration file.
    ///
    /// Writes `[global_key_map]` for the Default scope and
    /// `[context_key_maps.<name>]` for each context scope.
    ///
    /// Validates: Requirement 20.8
    pub fn save_to_config(&self, config: &ConfigHandle) {
        // Write global key map
        let global_table = self.staged_default.to_config_table("global");
        let _ = config.set_user_value("global_key_map", global_table);

        // Write each context key map
        for (name, rows) in &self.staged_contexts {
            let ctx_table = rows.to_config_table(name);
            let key = format!("context_key_maps.{name}");
            let _ = config.set_user_value(&key, ctx_table);
        }
    }

    /// Discard staged changes and close.
    ///
    /// Validates: Requirement 20.5, accessibility Requirement 2.3
    pub fn cancel(&mut self) {
        self.staged_default = self.original_default.clone();
        self.staged_contexts = self.original_contexts.clone();
        self.open = false;
    }

    /// Get a mutable reference to the rows for the currently active tab.
    fn active_rows_mut(&mut self) -> &mut Vec<KeyRow> {
        match &self.active_tab {
            ScopeTab::Default => &mut self.staged_default.rows,
            ScopeTab::Context(name) => {
                let name = name.clone();
                self.staged_contexts
                    .entry(name.clone())
                    .or_insert_with(|| ScopeRows::empty_for(&name))
                    .rows
                    .as_mut()
            }
        }
    }
}

impl Default for KeyConfigDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the dialog if it is open.
///
/// Validates: Requirement 20.1, 20.14
pub fn render_if_open(
    ctx: &egui::Context,
    dialog: &mut KeyConfigDialog,
    resolver: &KeyMapResolver,
    config: &ConfigHandle,
) {
    if !dialog.open {
        return;
    }
    // Load from resolver on first open (when staged default is still empty)
    if dialog
        .staged_default
        .rows
        .iter()
        .all(|r| r.commands[0].is_empty())
    {
        dialog.load_from_resolver(resolver);
    }
    render(ctx, dialog, resolver, config);
}

/// Render the Key Configuration Dialog window.
///
/// Validates: Requirement 20.2–20.15
pub fn render(
    ctx: &egui::Context,
    dialog: &mut KeyConfigDialog,
    _resolver: &KeyMapResolver,
    config: &ConfigHandle,
) {
    if !dialog.open {
        return;
    }

    let mut save_clicked = false;
    let mut cancel_clicked = false;
    let mut reset_clicked = false;

    egui::Window::new("Key Assignments")
        .collapsible(false)
        .resizable(true)
        .min_width(900.0)
        .min_height(400.0)
        .show(ctx, |ui| {
            // ── Scope selector tabs — Validates: Requirement 20.2 ──────────
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(dialog.active_tab == ScopeTab::Default, "Default (Global)")
                    .clicked()
                {
                    dialog.active_tab = ScopeTab::Default;
                }
                for &name in CONTEXT_NAMES {
                    let selected = dialog.active_tab == ScopeTab::Context(name.to_string());
                    if ui.selectable_label(selected, name).clicked() {
                        dialog.active_tab = ScopeTab::Context(name.to_string());
                    }
                }
            });
            ui.separator();

            // ── Grid — Validates: Requirement 20.3 ─────────────────────────
            let rows = dialog.active_rows_mut();

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    egui::Grid::new("key_config_grid")
                        .num_columns(10)
                        .striped(true)
                        .spacing([4.0, 2.0])
                        .show(ui, |ui| {
                            // Header row
                            ui.label("Key");
                            ui.label("Command");
                            ui.label("Label*");
                            ui.label("Description");
                            ui.label("Shift Cmd");
                            ui.label("Shift Desc");
                            ui.label("Ctrl Cmd");
                            ui.label("Ctrl Desc");
                            ui.label("Alt Cmd");
                            ui.label("Alt Desc");
                            ui.end_row();

                            for row in rows.iter_mut() {
                                ui.label(row.key.display_name());

                                // Plain command + derived label + description
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.commands[0])
                                        .desired_width(120.0)
                                        .hint_text("unassigned"),
                                );
                                // Label (read-only derived) — Validates: Requirement 20.7
                                let derived = row.commands[0]
                                    .split_whitespace()
                                    .next()
                                    .unwrap_or("")
                                    .to_string();
                                ui.label(egui::RichText::new(derived).weak());
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.descriptions[0])
                                        .desired_width(140.0)
                                        .hint_text("description"),
                                );

                                // Shift
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.commands[1])
                                        .desired_width(120.0)
                                        .hint_text("unassigned"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.descriptions[1])
                                        .desired_width(100.0),
                                );

                                // Ctrl
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.commands[2])
                                        .desired_width(120.0)
                                        .hint_text("unassigned"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.descriptions[2])
                                        .desired_width(100.0),
                                );

                                // Alt
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.commands[3])
                                        .desired_width(120.0)
                                        .hint_text("unassigned"),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut row.descriptions[3])
                                        .desired_width(100.0),
                                );

                                ui.end_row();
                            }
                        });
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    save_clicked = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel_clicked = true;
                }
                if ui.button("Reset to Defaults").clicked() {
                    reset_clicked = true;
                }
                ui.label(
                    egui::RichText::new("* Label is auto-derived from command")
                        .weak()
                        .small(),
                );
            });
        });

    if reset_clicked {
        dialog.reset_active_tab();
    }
    if cancel_clicked {
        dialog.cancel();
    }
    // Validates: accessibility Requirement 2.1, 2.3 -- Escape closes the dialog.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        dialog.cancel();
    }
    if save_clicked {
        // Validates: Requirement 20.5, 20.8 — persist staged maps to user-layer TOML
        dialog.save_to_config(config);
        dialog.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_keys::{FunctionKey, KeyMap, KeyMapResolver};

    fn make_resolver() -> KeyMapResolver {
        KeyMapResolver::new(KeyMap::default_global())
    }

    /// Validates: Requirement 20.8 — save_to_config writes global_key_map and context_key_maps
    /// to the user-layer config. We test the serialisation logic (to_config_table) directly
    /// since ConfigHandle requires a real filesystem path.
    #[test]
    fn save_produces_correct_config_values_for_global_scope() {
        // Validates: Requirement 20.8
        let resolver = make_resolver();
        let mut dialog = KeyConfigDialog::new();
        dialog.load_from_resolver(&resolver);

        // The default global map has F3=END, F7=UP MAX/Up, F8=DOWN MAX/Down, F12=RETRIEVE/Retrieve
        let table = dialog.staged_default.to_config_table("global");
        if let ConfigValue::Table(map) = table {
            // F3 is a plain string (no label in KeyBinding::new, but default_global uses with_label)
            // F3 = { command = "END", label = "End" }
            assert!(map.contains_key("F3"), "F3 must be in global_key_map");
            assert!(map.contains_key("F7"), "F7 must be in global_key_map");
            assert!(map.contains_key("F12"), "F12 must be in global_key_map");
            // F4 is unassigned — must not appear
            assert!(
                !map.contains_key("F4"),
                "F4 must not appear in global_key_map"
            );
        } else {
            panic!("expected ConfigValue::Table for global_key_map");
        }
    }

    /// Validates: Requirement 20.8 — context scope produces correct key under context_key_maps.
    #[test]
    fn save_produces_correct_config_key_for_context_scope() {
        // Validates: Requirement 20.8
        let mut dialog = KeyConfigDialog::new();
        // Set a binding in the editor context
        if let Some(rows) = dialog.staged_contexts.get_mut("editor") {
            if let Some(row) = rows.rows.iter_mut().find(|r| r.key == FunctionKey::F5) {
                row.commands[0] = "FIND".to_string();
            }
        }
        let ctx_table = dialog
            .staged_contexts
            .get("editor")
            .unwrap()
            .to_config_table("editor");
        if let ConfigValue::Table(map) = ctx_table {
            assert!(
                map.contains_key("F5"),
                "F5 must appear in editor context map"
            );
        } else {
            panic!("expected ConfigValue::Table for context map");
        }
    }

    /// Validates: Requirement 20.8 — empty context scope produces empty table (no spurious keys).
    #[test]
    fn empty_context_scope_produces_empty_table() {
        // Validates: Requirement 20.8
        let dialog = KeyConfigDialog::new();
        let ctx_table = dialog
            .staged_contexts
            .get("pom")
            .unwrap()
            .to_config_table("pom");
        if let ConfigValue::Table(map) = ctx_table {
            assert!(map.is_empty(), "empty context must produce empty table");
        } else {
            panic!("expected ConfigValue::Table");
        }
    }

    /// Validates: Requirement 20.1 — dialog starts closed.
    #[test]
    fn dialog_new_starts_closed() {
        let dialog = KeyConfigDialog::new();
        assert!(!dialog.open);
    }

    /// Validates: Requirement 20.6 — load_from_resolver populates staged rows.
    #[test]
    fn load_from_resolver_populates_default_rows() {
        let resolver = make_resolver();
        let mut dialog = KeyConfigDialog::new();
        dialog.load_from_resolver(&resolver);

        let f3_row = dialog
            .staged_default
            .rows
            .iter()
            .find(|r| r.key == FunctionKey::F3)
            .unwrap();
        assert_eq!(f3_row.commands[0], "END");
    }

    /// Validates: Requirement 20.5 — Cancel discards staged changes.
    #[test]
    fn cancel_discards_staged_changes() {
        let resolver = make_resolver();
        let mut dialog = KeyConfigDialog::new();
        dialog.open = true;
        dialog.load_from_resolver(&resolver);

        if let Some(row) = dialog
            .staged_default
            .rows
            .iter_mut()
            .find(|r| r.key == FunctionKey::F3)
        {
            row.commands[0] = "QUIT".to_string();
        }

        dialog.cancel();
        assert!(!dialog.open);

        let f3_row = dialog
            .staged_default
            .rows
            .iter()
            .find(|r| r.key == FunctionKey::F3)
            .unwrap();
        assert_eq!(f3_row.commands[0], "END");
    }

    /// Validates: Requirement 20.15 — Reset to Defaults restores Default tab.
    #[test]
    fn reset_default_tab_restores_built_in_defaults() {
        let mut dialog = KeyConfigDialog::new();
        if let Some(row) = dialog
            .staged_default
            .rows
            .iter_mut()
            .find(|r| r.key == FunctionKey::F3)
        {
            row.commands[0] = "QUIT".to_string();
        }
        dialog.active_tab = ScopeTab::Default;
        dialog.reset_active_tab();

        let f3_row = dialog
            .staged_default
            .rows
            .iter()
            .find(|r| r.key == FunctionKey::F3)
            .unwrap();
        assert_eq!(f3_row.commands[0], "END");
    }

    /// Validates: Requirement 20.4 — empty command treated as unassigned.
    #[test]
    fn empty_command_produces_no_binding_in_map() {
        let row = KeyRow {
            key: FunctionKey::F5,
            commands: [String::new(), String::new(), String::new(), String::new()],
            descriptions: [String::new(), String::new(), String::new(), String::new()],
        };
        let mut map = KeyMap::empty("test");
        row.apply_to_map(&mut map);
        assert!(map.get_plain(FunctionKey::F5).is_none());
    }

    /// Validates: Requirement 20.4 — non-empty command produces binding.
    #[test]
    fn non_empty_command_produces_binding_in_map() {
        let row = KeyRow {
            key: FunctionKey::F5,
            commands: [
                "FIND".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
            descriptions: [String::new(), String::new(), String::new(), String::new()],
        };
        let mut map = KeyMap::empty("test");
        row.apply_to_map(&mut map);
        assert_eq!(map.get_plain(FunctionKey::F5).unwrap().command(), "FIND");
    }

    /// Validates: Requirement 20.9 — modifier bindings stored independently.
    #[test]
    fn modifier_bindings_stored_independently_in_staged_map() {
        let row = KeyRow {
            key: FunctionKey::F3,
            commands: [
                "END".to_string(),
                "SWAP".to_string(),
                "COPY".to_string(),
                "MOVE".to_string(),
            ],
            descriptions: [String::new(), String::new(), String::new(), String::new()],
        };
        let mut map = KeyMap::empty("test");
        row.apply_to_map(&mut map);

        assert_eq!(map.get_plain(FunctionKey::F3).unwrap().command(), "END");
        assert_eq!(
            map.get(ModifiedKey::shift(FunctionKey::F3))
                .unwrap()
                .command(),
            "SWAP"
        );
        assert_eq!(
            map.get(ModifiedKey::ctrl(FunctionKey::F3))
                .unwrap()
                .command(),
            "COPY"
        );
        assert_eq!(
            map.get(ModifiedKey::alt(FunctionKey::F3))
                .unwrap()
                .command(),
            "MOVE"
        );
    }

    /// Validates: Requirement 20.2 — dialog has Default tab and all context tabs.
    #[test]
    fn dialog_has_all_scope_tabs() {
        let dialog = KeyConfigDialog::new();
        assert_eq!(dialog.active_tab, ScopeTab::Default);
        for &name in CONTEXT_NAMES {
            assert!(
                dialog.staged_contexts.contains_key(name),
                "missing context tab: {name}"
            );
        }
    }

    /// Validates: Requirement 20.3 — staged rows contain 24 entries (F1–F24).
    #[test]
    fn staged_default_has_24_rows() {
        let dialog = KeyConfigDialog::new();
        assert_eq!(dialog.staged_default.rows.len(), 24);
    }
}
