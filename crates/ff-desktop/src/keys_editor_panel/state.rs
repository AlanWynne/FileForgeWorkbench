//! State and Action types for the Keys Editor (function-keys-and-history
//! Requirement 22, CR-CH-029).
//!
//! The Keys Workspace edits a per-workspace-KIND key list and saves it to
//! `keymaps/<kind>.toml`. It is modelled on the Menus editor (`menus_editor_panel`):
//! a pure render that stashes a [`KeysEditorAction`] the shell applies.

use ff_keys::{FunctionKey, KeyBinding, KeyMap, ModifiedKey};

/// The stable workspace-kind context names selectable in the Keys Workspace
/// dropdown (function-keys Requirement 22.2). Mirrors `context_name_for_kind`.
pub const KIND_NAMES: &[&str] = &[
    "pom", "editor", "config", "files", "search", "plugins", "log", "macros", "menu", "commands",
    "theme", "menus",
];

/// One editable key row: a physical `FunctionKey` with a command per modifier
/// layer `[plain, shift, ctrl, alt]`. (The ALTGR/CTRL+SHIFT layers and the
/// Command_Picker are deferred, function-keys Req 20/21.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRow {
    /// The physical function key for this row.
    pub key: FunctionKey,
    /// Command per modifier layer: `[plain, shift, ctrl, alt]`.
    pub commands: [String; 4],
}

impl KeyRow {
    /// Build a row from a key map (reads the four modifier-layer bindings).
    pub fn from_map(key: FunctionKey, map: &KeyMap) -> Self {
        let modifiers = [
            ModifiedKey::plain(key),
            ModifiedKey::shift(key),
            ModifiedKey::ctrl(key),
            ModifiedKey::alt(key),
        ];
        let mut commands = [String::new(), String::new(), String::new(), String::new()];
        for (i, mk) in modifiers.iter().enumerate() {
            if let Some(b) = map.get(*mk) {
                commands[i] = b.command().to_string();
            }
        }
        Self { key, commands }
    }

    /// Apply this row's staged commands into a `KeyMap` (empty = unassigned).
    pub fn apply_to_map(&self, map: &mut KeyMap) {
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
                map.set(*mk, KeyBinding::new(cmd));
            }
        }
    }
}

/// Build the staged rows for a key map: one row per physical key F1-F12.
pub fn rows_from_map(map: &KeyMap) -> Vec<KeyRow> {
    FunctionKey::ALL
        .iter()
        .filter(|k| k.number() <= 12)
        .map(|&k| KeyRow::from_map(k, map))
        .collect()
}

/// Collapse the staged rows back into a `KeyMap` under `source`.
pub fn rows_to_map(rows: &[KeyRow], source: &str) -> KeyMap {
    let mut map = KeyMap::empty(source);
    for row in rows {
        row.apply_to_map(&mut map);
    }
    map
}

/// A structural action produced by the Keys Editor render, applied by the shell.
/// Command text fields mutate the working rows directly in the render (mirroring
/// the Menus editor B054 fix) and do NOT produce actions.
///
/// Validates: function-keys-and-history Requirement 22.2, 22.4.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum KeysEditorAction {
    /// No action this frame.
    #[default]
    None,
    /// Select a different workspace kind to edit (loads its key list).
    SelectKind(String),
    /// Save the working key list to `keymaps/<kind>.toml`.
    Save,
    /// Reset the working rows to the compiled default for the selected kind.
    Reset,
}

/// Per-Context UI state for the Keys Editor. Lives on the shell (like
/// `MenusEditorState`), not on the `TabState`.
///
/// Validates: function-keys-and-history Requirement 22.1, 22.2.
#[derive(Debug, Clone, Default)]
pub struct KeysEditorState {
    /// The workspace kind currently selected for editing (a stable context
    /// name). `None` before the first selection.
    pub selected_kind: Option<String>,
    /// The working (unsaved) key rows for the selected kind, F1-F12.
    pub rows: Vec<KeyRow>,
    /// The most recent validation/save error or status, shown inline.
    pub error: Option<String>,
    /// Egui id of the FIRST focusable interior control (the kind dropdown),
    /// captured fresh each frame by `render` so the shell Boundary_Policy can
    /// focus it on Tab from the command field (CR-CH-023). Transient.
    pub first_interior_id: Option<egui::Id>,
    /// Egui id of the LAST focusable interior control (the Save button),
    /// captured fresh each frame by `render` to anchor the Shift+Tab reverse
    /// boundary. Transient.
    pub last_interior_id: Option<egui::Id>,
    /// Action produced by the most recent `WorkspaceContext::render`, stashed for
    /// the shell to apply via `apply_keys_editor_action` (CR-NR-078). Transient.
    pub pending_action: KeysEditorAction,
}

impl KeysEditorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load `map` as the working rows for `kind`, clearing any error.
    pub fn load_kind(&mut self, kind: &str, map: &KeyMap) {
        self.selected_kind = Some(kind.to_string());
        self.rows = rows_from_map(map);
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: function-keys Req 22.2 -- the kind dropdown lists the stable
    // context names.
    #[test]
    fn kind_names_are_the_stable_context_names() {
        assert!(KIND_NAMES.contains(&"pom"));
        assert!(KIND_NAMES.contains(&"editor"));
        assert!(KIND_NAMES.contains(&"config"));
        assert!(
            KIND_NAMES.contains(&"keys") == false,
            "the editor does not list itself"
        );
        assert_eq!(KIND_NAMES.len(), 12);
    }

    // Validates: function-keys Req 22.1 -- default state is empty.
    #[test]
    fn new_state_is_empty() {
        let s = KeysEditorState::new();
        assert!(s.selected_kind.is_none());
        assert!(s.rows.is_empty());
        assert_eq!(s.pending_action, KeysEditorAction::None);
    }

    // Validates: function-keys Req 22.2 -- loading a kind populates F1-F12 rows.
    #[test]
    fn load_kind_populates_twelve_rows_from_default() {
        let mut s = KeysEditorState::new();
        s.load_kind("editor", &KeyMap::default_global());
        assert_eq!(s.selected_kind.as_deref(), Some("editor"));
        assert_eq!(s.rows.len(), 12, "F1-F12");
        // F1 plain should be HELP in the default map.
        let f1 = s.rows.iter().find(|r| r.key == FunctionKey::F1).unwrap();
        assert_eq!(f1.commands[0], "HELP");
    }

    // Validates: function-keys Req 22.4 -- rows round-trip through a KeyMap.
    #[test]
    fn rows_round_trip_through_key_map() {
        let original = KeyMap::default_global();
        let rows = rows_from_map(&original);
        let rebuilt = rows_to_map(&rows, "editor");
        // Base F1 = HELP, Shift+F7 = "UP MAX" survive the round-trip.
        assert_eq!(
            rebuilt
                .get(ModifiedKey::plain(FunctionKey::F1))
                .unwrap()
                .command(),
            "HELP"
        );
        assert_eq!(
            rebuilt
                .get(ModifiedKey::shift(FunctionKey::F7))
                .unwrap()
                .command(),
            "UP MAX"
        );
    }
}
