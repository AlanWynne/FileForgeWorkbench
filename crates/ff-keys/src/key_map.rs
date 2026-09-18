//! Key map data structures — `KeyBinding` and `KeyMap`.
//!
//! A `KeyMap` holds function-key-to-command assignments, keyed by `ModifiedKey`
//! to support plain (F1–F24), Shift+Fn, Ctrl+Fn, and Alt+Fn bindings.

use std::collections::HashMap;

use crate::function_key::{FunctionKey, ModifiedKey};

/// A single function key assignment within a key map.
///
/// Contains the command string to dispatch, an optional explicit label
/// for the Key Label Bar display, and an optional human-readable description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    /// The command string to dispatch (full primary command syntax).
    ///
    /// Examples: `"FIND 'ERROR' ALL"`, `"MACRO myfix"`, `"SAVE"`.
    command: String,

    /// Optional explicit short label for the Key_Label_Bar.
    ///
    /// If `None`, the label is derived from the first token of `command`.
    label: Option<String>,

    /// Optional human-readable description of what the command does.
    ///
    /// Displayed in the Key Configuration Dialog.
    ///
    /// Validates: Requirement 20.3
    description: Option<String>,
}

impl KeyBinding {
    /// Create a new binding with just a command string (label and description auto-derived).
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            label: None,
            description: None,
        }
    }

    /// Create a new binding with an explicit label.
    pub fn with_label(command: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            label: Some(label.into()),
            description: None,
        }
    }

    /// Create a new binding with an explicit label and description.
    pub fn with_label_and_description(
        command: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            label: Some(label.into()),
            description: Some(description.into()),
        }
    }

    /// Create a new binding with a description but no explicit label.
    pub fn with_description(command: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            label: None,
            description: Some(description.into()),
        }
    }

    /// The command string assigned to this key.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// The explicit label, if configured.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// The human-readable description, if configured.
    ///
    /// Validates: Requirement 20.3
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Derive the display label: explicit label if set, otherwise first token of command.
    ///
    /// The first token is the first whitespace-delimited substring of the command string.
    pub fn display_label(&self) -> &str {
        if let Some(ref label) = self.label {
            label
        } else {
            self.command.split_whitespace().next().unwrap_or("")
        }
    }
}

/// A non-fatal warning produced during key map parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMapWarning {
    /// The configuration key or field that caused the warning.
    pub field: String,
    /// Human-readable description of the issue.
    pub message: String,
}

/// A collection of function key assignments, keyed by `ModifiedKey`.
///
/// Supports plain (F1–F24), Shift+Fn, Ctrl+Fn, and Alt+Fn bindings.
/// Used for both Global_Key_Map and Profile_Key_Map.
///
/// Validates: Requirement 20.9, 20.12
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMap {
    /// The key assignments, keyed by ModifiedKey.
    entries: HashMap<ModifiedKey, KeyBinding>,
    /// Source identifier for diagnostics (e.g., "global", "cobol").
    source: String,
}

impl KeyMap {
    /// Create an empty key map with the given source name.
    pub fn empty(source: impl Into<String>) -> Self {
        Self {
            entries: HashMap::new(),
            source: source.into(),
        }
    }

    /// Parse a key map from a TOML table.
    ///
    /// Each key in the table should be a modified key name:
    /// - `F1`–`F24`   → plain binding
    /// - `SF1`–`SF24` → Shift binding
    /// - `CF1`–`CF24` → Ctrl binding
    /// - `AF1`–`AF24` → Alt binding
    ///
    /// Values can be:
    /// - A plain string: interpreted as the command (label auto-derived)
    /// - A table with `command` (required), `label` (optional), `description` (optional)
    ///
    /// Invalid entries are skipped with warnings collected.
    ///
    /// Validates: Requirement 20.11
    pub fn from_toml_table(
        table: &toml::map::Map<String, toml::Value>,
        source: &str,
    ) -> (Self, Vec<KeyMapWarning>) {
        let mut entries = HashMap::new();
        let mut warnings = Vec::new();

        for (key_str, value) in table {
            // Parse the modified key name
            let modified_key = match ModifiedKey::parse(key_str) {
                Some(k) => k,
                None => {
                    warnings.push(KeyMapWarning {
                        field: key_str.clone(),
                        message: format!(
                            "invalid key identifier '{}' — expected F1–F24, SF1–SF24, CF1–CF24, or AF1–AF24, skipping",
                            key_str
                        ),
                    });
                    continue;
                }
            };

            // Parse the value (string or table)
            let binding = match value {
                toml::Value::String(cmd) => {
                    if cmd.trim().is_empty() {
                        warnings.push(KeyMapWarning {
                            field: key_str.clone(),
                            message: format!(
                                "empty command string for key {}, skipping",
                                modified_key.toml_name()
                            ),
                        });
                        continue;
                    }
                    KeyBinding::new(cmd.clone())
                }
                toml::Value::Table(tbl) => {
                    let command = match tbl.get("command") {
                        Some(toml::Value::String(cmd)) => {
                            if cmd.trim().is_empty() {
                                warnings.push(KeyMapWarning {
                                    field: key_str.clone(),
                                    message: format!(
                                        "empty command string for key {}, skipping",
                                        modified_key.toml_name()
                                    ),
                                });
                                continue;
                            }
                            cmd.clone()
                        }
                        Some(_) => {
                            warnings.push(KeyMapWarning {
                                field: key_str.clone(),
                                message: format!(
                                    "key {} has non-string 'command' field, skipping",
                                    modified_key.toml_name()
                                ),
                            });
                            continue;
                        }
                        None => {
                            warnings.push(KeyMapWarning {
                                field: key_str.clone(),
                                message: format!(
                                    "key {} table missing required 'command' field, skipping",
                                    modified_key.toml_name()
                                ),
                            });
                            continue;
                        }
                    };

                    let label = match tbl.get("label") {
                        Some(toml::Value::String(l)) => Some(l.clone()),
                        Some(_) => {
                            warnings.push(KeyMapWarning {
                                field: key_str.clone(),
                                message: format!(
                                    "key {} has non-string 'label' field, ignoring label",
                                    modified_key.toml_name()
                                ),
                            });
                            None
                        }
                        None => None,
                    };

                    let description = match tbl.get("description") {
                        Some(toml::Value::String(d)) => Some(d.clone()),
                        Some(_) => {
                            warnings.push(KeyMapWarning {
                                field: key_str.clone(),
                                message: format!(
                                    "key {} has non-string 'description' field, ignoring",
                                    modified_key.toml_name()
                                ),
                            });
                            None
                        }
                        None => None,
                    };

                    match (label, description) {
                        (Some(l), Some(d)) => KeyBinding::with_label_and_description(command, l, d),
                        (Some(l), None) => KeyBinding::with_label(command, l),
                        (None, Some(d)) => KeyBinding::with_description(command, d),
                        (None, None) => KeyBinding::new(command),
                    }
                }
                _ => {
                    warnings.push(KeyMapWarning {
                        field: key_str.clone(),
                        message: format!(
                            "key {} has unsupported value type (expected string or table), skipping",
                            modified_key.toml_name()
                        ),
                    });
                    continue;
                }
            };

            // Duplicate key detection (last-wins with warning)
            if entries.contains_key(&modified_key) {
                warnings.push(KeyMapWarning {
                    field: key_str.clone(),
                    message: format!(
                        "duplicate assignment for key {} — last value wins",
                        modified_key.toml_name()
                    ),
                });
            }
            entries.insert(modified_key, binding);
        }

        (
            Self {
                entries,
                source: source.to_string(),
            },
            warnings,
        )
    }

    /// Look up the binding for a `ModifiedKey`. Returns `None` if unassigned.
    pub fn get(&self, key: ModifiedKey) -> Option<&KeyBinding> {
        self.entries.get(&key)
    }

    /// Look up the plain (unmodified) binding for a `FunctionKey`.
    ///
    /// Convenience method equivalent to `get(ModifiedKey::plain(key))`.
    /// Used by the Key Label Bar which only displays plain bindings.
    ///
    /// Validates: Requirement 20.13
    pub fn get_plain(&self, key: FunctionKey) -> Option<&KeyBinding> {
        self.entries.get(&ModifiedKey::plain(key))
    }

    /// Insert or replace an assignment.
    pub fn set(&mut self, key: ModifiedKey, binding: KeyBinding) {
        self.entries.insert(key, binding);
    }

    /// Remove an assignment. Returns the removed binding if present.
    pub fn remove(&mut self, key: ModifiedKey) -> Option<KeyBinding> {
        self.entries.remove(&key)
    }

    /// Iterate over all assigned keys in order.
    pub fn iter(&self) -> impl Iterator<Item = (ModifiedKey, &KeyBinding)> {
        let mut pairs: Vec<_> = self.entries.iter().map(|(&k, v)| (k, v)).collect();
        pairs.sort_by_key(|(k, _)| *k);
        pairs.into_iter()
    }

    /// Number of assigned keys (across all modifier variants).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the key map has no assignments.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The source name for this key map.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Build the built-in default global key map.
    ///
    /// The full owner-specified ISPF-standard default (CR-CH-027), CODE-ONLY
    /// (compiled, never a shipped TOML file -- mirroring the compiled default
    /// POM/Settings menus). It binds the Base (unmodified) F1-F12 row and the
    /// Shift+F1-F12 row:
    ///
    /// Base : F1 HELP, F2 SPLIT, F3 END, F4 RETURN, F5 RFIND, F6 RCHANGE,
    ///        F7 UP, F8 DOWN, F9 SWAP, F10 LEFT, F11 RIGHT, F12 RETRIEVE.
    /// Shift: mirrors Base except SF7 "UP MAX", SF8 "DOWN MAX", SF10 "LEFT MAX",
    ///        SF11 "RIGHT MAX", SF12 CURSOR.
    ///
    /// Base F7/F8 scroll by one page (bare UP/DOWN); the MAX scroll variants are
    /// on Shift+F7/F8/F10/F11 (B046: Base F7/F8 are NOT "UP MAX"/"DOWN MAX").
    /// Ctrl/Alt layers and keys beyond F12 are unassigned in the baseline.
    ///
    /// Every binding here is a DEFAULT only and is fully overridable through the
    /// key configuration or a `keymaps/<context>.toml` override file
    /// (function-keys-and-history Req 14/15).
    ///
    /// Validates: function-keys-and-history Requirement 15.1, 15.2, 15.3
    pub fn default_global() -> Self {
        let mut map = Self::empty("global");
        // Base (unmodified) row: (key, command, label).
        let base: [(FunctionKey, &str, &str); 12] = [
            (FunctionKey::F1, "HELP", "Help"),
            (FunctionKey::F2, "SPLIT", "Split"),
            (FunctionKey::F3, "END", "End"),
            (FunctionKey::F4, "RETURN", "Return"),
            (FunctionKey::F5, "RFIND", "RFind"),
            (FunctionKey::F6, "RCHANGE", "RChange"),
            (FunctionKey::F7, "UP", "Up"),
            (FunctionKey::F8, "DOWN", "Down"),
            (FunctionKey::F9, "SWAP", "Swap"),
            (FunctionKey::F10, "LEFT", "Left"),
            (FunctionKey::F11, "RIGHT", "Right"),
            (FunctionKey::F12, "RETRIEVE", "Retrieve"),
        ];
        for (key, cmd, label) in base {
            map.set(ModifiedKey::plain(key), KeyBinding::with_label(cmd, label));
        }
        // Shift row: mirrors Base except the four scroll-max variants and Cursor.
        let shift: [(FunctionKey, &str, &str); 12] = [
            (FunctionKey::F1, "HELP", "Help"),
            (FunctionKey::F2, "SPLIT", "Split"),
            (FunctionKey::F3, "END", "End"),
            (FunctionKey::F4, "RETURN", "Return"),
            (FunctionKey::F5, "RFIND", "RFind"),
            (FunctionKey::F6, "RCHANGE", "RChange"),
            (FunctionKey::F7, "UP MAX", "Up Max"),
            (FunctionKey::F8, "DOWN MAX", "Down Max"),
            (FunctionKey::F9, "SWAP", "Swap"),
            (FunctionKey::F10, "LEFT MAX", "Left Max"),
            (FunctionKey::F11, "RIGHT MAX", "Right Max"),
            (FunctionKey::F12, "CURSOR", "Cursor"),
        ];
        for (key, cmd, label) in shift {
            map.set(ModifiedKey::shift(key), KeyBinding::with_label(cmd, label));
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_binding_new_derives_label_from_first_token() {
        // Validates: Requirement 4.4
        let binding = KeyBinding::new("FIND 'ERROR' ALL");
        assert_eq!(binding.display_label(), "FIND");
        assert_eq!(binding.command(), "FIND 'ERROR' ALL");
        assert_eq!(binding.label(), None);
        assert_eq!(binding.description(), None);
    }

    #[test]
    fn key_binding_with_label_uses_explicit_label() {
        // Validates: Requirement 4.5
        let binding = KeyBinding::with_label("UP MAX", "UP");
        assert_eq!(binding.display_label(), "UP");
        assert_eq!(binding.command(), "UP MAX");
        assert_eq!(binding.label(), Some("UP"));
    }

    #[test]
    fn key_binding_with_description() {
        // Validates: Requirement 20.3
        let binding = KeyBinding::with_description("END", "Close current panel");
        assert_eq!(binding.description(), Some("Close current panel"));
        assert_eq!(binding.label(), None);
    }

    #[test]
    fn key_binding_with_label_and_description() {
        // Validates: Requirement 20.3
        let binding = KeyBinding::with_label_and_description("END", "End", "Close current panel");
        assert_eq!(binding.label(), Some("End"));
        assert_eq!(binding.description(), Some("Close current panel"));
    }

    #[test]
    fn key_binding_single_word_command_label() {
        let binding = KeyBinding::new("SAVE");
        assert_eq!(binding.display_label(), "SAVE");
    }

    #[test]
    fn key_map_empty_has_no_entries() {
        let map = KeyMap::empty("test");
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.source(), "test");
    }

    #[test]
    fn key_map_set_and_get_plain() {
        // Validates: Requirement 20.12 — get_plain() returns plain binding
        let mut map = KeyMap::empty("test");
        map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
        assert_eq!(map.get_plain(FunctionKey::F3).unwrap().command(), "END");
        assert_eq!(map.get_plain(FunctionKey::F4), None);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn key_map_modifier_bindings_independent() {
        // Validates: Requirement 20.9 — modifier bindings are independent
        let mut map = KeyMap::empty("test");
        map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
        map.set(ModifiedKey::shift(FunctionKey::F3), KeyBinding::new("SWAP"));
        map.set(ModifiedKey::ctrl(FunctionKey::F3), KeyBinding::new("COPY"));
        map.set(ModifiedKey::alt(FunctionKey::F3), KeyBinding::new("MOVE"));

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
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn key_map_remove() {
        let mut map = KeyMap::empty("test");
        map.set(ModifiedKey::plain(FunctionKey::F5), KeyBinding::new("FIND"));
        let removed = map.remove(ModifiedKey::plain(FunctionKey::F5));
        assert!(removed.is_some());
        assert_eq!(map.get_plain(FunctionKey::F5), None);
        assert!(map.is_empty());
    }

    #[test]
    fn key_map_from_toml_table_string_values() {
        // Validates: Requirement 11.1 — plain string format
        let toml_str = r#"
            F3 = "END"
            F5 = "FIND 'ERROR' ALL"
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert!(warnings.is_empty());
        assert_eq!(map.len(), 2);
        assert_eq!(map.get_plain(FunctionKey::F3).unwrap().command(), "END");
        assert_eq!(
            map.get_plain(FunctionKey::F5).unwrap().command(),
            "FIND 'ERROR' ALL"
        );
    }

    #[test]
    fn key_map_from_toml_table_table_values() {
        // Validates: Requirement 11.1 — table format with command and label
        let toml_str = r#"
            F7 = { command = "UP MAX", label = "UP" }
            F8 = { command = "DOWN MAX", label = "DOWN" }
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert!(warnings.is_empty());
        assert_eq!(map.get_plain(FunctionKey::F7).unwrap().command(), "UP MAX");
        assert_eq!(map.get_plain(FunctionKey::F7).unwrap().label(), Some("UP"));
        assert_eq!(
            map.get_plain(FunctionKey::F8).unwrap().display_label(),
            "DOWN"
        );
    }

    #[test]
    fn key_map_from_toml_table_with_description() {
        // Validates: Requirement 20.3 — description field parsed from TOML
        let toml_str = r#"
            F3 = { command = "END", label = "End", description = "Close current panel" }
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert!(warnings.is_empty());
        let binding = map.get_plain(FunctionKey::F3).unwrap();
        assert_eq!(binding.command(), "END");
        assert_eq!(binding.label(), Some("End"));
        assert_eq!(binding.description(), Some("Close current panel"));
    }

    #[test]
    fn key_map_from_toml_table_modifier_prefixes() {
        // Validates: Requirement 20.11 — SF/CF/AF prefixes parsed correctly
        let toml_str = r#"
            F3 = "END"
            SF3 = "SWAP"
            CF3 = "COPY"
            AF3 = "MOVE"
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(map.len(), 4);
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

    #[test]
    fn key_map_from_toml_table_invalid_key_produces_warning() {
        // Validates: Requirement 1.5 — reject invalid keys with warning
        let toml_str = r#"
            F3 = "END"
            F25 = "INVALID"
            G3 = "ALSO_INVALID"
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert_eq!(map.len(), 1);
        assert_eq!(map.get_plain(FunctionKey::F3).unwrap().command(), "END");
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn key_map_from_toml_table_empty_command_produces_warning() {
        let toml_str = r#"
            F3 = ""
            F5 = "FIND"
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert_eq!(map.len(), 1);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn key_map_from_toml_table_missing_command_field_produces_warning() {
        let toml_str = r#"
            F7 = { label = "UP" }
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert_eq!(map.len(), 0);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0]
            .message
            .contains("missing required 'command' field"));
    }

    #[test]
    fn key_map_from_toml_table_unsupported_value_type_produces_warning() {
        let toml_str = r#"
            F3 = 42
        "#;
        let table: toml::Table = toml_str.parse().unwrap();
        let (map, warnings) = KeyMap::from_toml_table(&table, "global");

        assert_eq!(map.len(), 0);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("unsupported value type"));
    }

    #[test]
    fn key_map_default_global_has_full_base_row() {
        // Validates: function-keys Requirement 15.1 (CR-CH-027) -- the full Base
        // (unmodified F-key) row with commands and labels.
        let map = KeyMap::default_global();
        let expected: [(FunctionKey, &str, &str); 12] = [
            (FunctionKey::F1, "HELP", "Help"),
            (FunctionKey::F2, "SPLIT", "Split"),
            (FunctionKey::F3, "END", "End"),
            (FunctionKey::F4, "RETURN", "Return"),
            (FunctionKey::F5, "RFIND", "RFind"),
            (FunctionKey::F6, "RCHANGE", "RChange"),
            (FunctionKey::F7, "UP", "Up"),
            (FunctionKey::F8, "DOWN", "Down"),
            (FunctionKey::F9, "SWAP", "Swap"),
            (FunctionKey::F10, "LEFT", "Left"),
            (FunctionKey::F11, "RIGHT", "Right"),
            (FunctionKey::F12, "RETRIEVE", "Retrieve"),
        ];
        for (key, cmd, label) in expected {
            let b = map
                .get(ModifiedKey::plain(key))
                .unwrap_or_else(|| panic!("Base {key} must be bound"));
            assert_eq!(b.command(), cmd, "Base {key} command");
            assert_eq!(b.display_label(), label, "Base {key} label");
        }
    }

    #[test]
    fn key_map_default_global_has_full_shift_row() {
        // Validates: function-keys Requirement 15.2 (CR-CH-027) -- the Shift row
        // mirrors Base except the four scroll-max variants and Cursor.
        let map = KeyMap::default_global();
        let expected: [(FunctionKey, &str, &str); 12] = [
            (FunctionKey::F1, "HELP", "Help"),
            (FunctionKey::F2, "SPLIT", "Split"),
            (FunctionKey::F3, "END", "End"),
            (FunctionKey::F4, "RETURN", "Return"),
            (FunctionKey::F5, "RFIND", "RFind"),
            (FunctionKey::F6, "RCHANGE", "RChange"),
            (FunctionKey::F7, "UP MAX", "Up Max"),
            (FunctionKey::F8, "DOWN MAX", "Down Max"),
            (FunctionKey::F9, "SWAP", "Swap"),
            (FunctionKey::F10, "LEFT MAX", "Left Max"),
            (FunctionKey::F11, "RIGHT MAX", "Right Max"),
            (FunctionKey::F12, "CURSOR", "Cursor"),
        ];
        for (key, cmd, label) in expected {
            let b = map
                .get(ModifiedKey::shift(key))
                .unwrap_or_else(|| panic!("Shift+{key} must be bound"));
            assert_eq!(b.command(), cmd, "Shift+{key} command");
            assert_eq!(b.display_label(), label, "Shift+{key} label");
        }
    }

    #[test]
    fn key_map_default_global_binds_exactly_base_and_shift_f1_to_f12() {
        // Validates: function-keys Requirement 15.3 -- Ctrl/Alt layers and keys
        // beyond F12 are unassigned in the baseline (24 bindings: 12 Base + 12
        // Shift). No Ctrl/Alt bindings exist by default.
        let map = KeyMap::default_global();
        assert_eq!(map.len(), 24, "default map = 12 Base + 12 Shift");
        for key in FunctionKey::ALL {
            // No Ctrl or Alt bindings in the default.
            assert!(
                map.get(ModifiedKey::ctrl(key)).is_none(),
                "Ctrl+{key} must be unassigned by default"
            );
            assert!(
                map.get(ModifiedKey::alt(key)).is_none(),
                "Alt+{key} must be unassigned by default"
            );
            // Keys beyond F12 have no Base or Shift binding either.
            let num = key.number();
            if num > 12 {
                assert!(
                    map.get(ModifiedKey::plain(key)).is_none(),
                    "Base {key} (>F12) must be unassigned"
                );
                assert!(
                    map.get(ModifiedKey::shift(key)).is_none(),
                    "Shift+{key} (>F12) must be unassigned"
                );
            }
        }
    }

    #[test]
    fn key_map_iter_returns_keys_in_order() {
        let mut map = KeyMap::empty("test");
        map.set(
            ModifiedKey::plain(FunctionKey::F12),
            KeyBinding::new("RETRIEVE"),
        );
        map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
        map.set(
            ModifiedKey::plain(FunctionKey::F7),
            KeyBinding::new("UP MAX"),
        );

        let keys: Vec<ModifiedKey> = map.iter().map(|(k, _)| k).collect();
        assert_eq!(
            keys,
            vec![
                ModifiedKey::plain(FunctionKey::F3),
                ModifiedKey::plain(FunctionKey::F7),
                ModifiedKey::plain(FunctionKey::F12),
            ]
        );
    }

    #[test]
    fn modifier_binding_does_not_affect_plain_binding() {
        // Validates: Requirement 20.9 — modifier bindings are independent of plain
        let mut map = KeyMap::empty("test");
        map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
        map.set(ModifiedKey::shift(FunctionKey::F3), KeyBinding::new("SWAP"));

        // Plain binding unchanged
        assert_eq!(map.get_plain(FunctionKey::F3).unwrap().command(), "END");
        // Shift binding independent
        assert_eq!(
            map.get(ModifiedKey::shift(FunctionKey::F3))
                .unwrap()
                .command(),
            "SWAP"
        );
        // Ctrl/Alt still unassigned
        assert!(map.get(ModifiedKey::ctrl(FunctionKey::F3)).is_none());
        assert!(map.get(ModifiedKey::alt(FunctionKey::F3)).is_none());
    }
}
