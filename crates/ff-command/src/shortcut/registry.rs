//! `ShortcutRegistry` — keyboard chord to command ID mapping with conflict detection.

use std::collections::HashMap;
use std::sync::RwLock;

use super::chord::KeyChord;
use super::reserved::reserved_shortcuts;
use super::sequence::ShortcutBinding;
use crate::error::CommandError;
use crate::id::CommandId;

/// The keyboard shortcut registry. Manages chord → CommandId mappings.
///
/// Pre-populated with reserved shortcuts at construction. Supports conflict
/// detection, user customization, and multi-key sequence resolution.
pub struct ShortcutRegistry {
    bindings: RwLock<HashMap<ShortcutBinding, CommandId>>,
    reserved: Vec<ShortcutBinding>,
}

impl ShortcutRegistry {
    /// Creates a new registry pre-populated with reserved shortcuts.
    pub fn new() -> Self {
        let reserved_list = reserved_shortcuts();
        let mut bindings = HashMap::new();
        let mut reserved_bindings = Vec::new();

        for entry in reserved_list {
            reserved_bindings.push(entry.binding.clone());
            bindings.insert(entry.binding, entry.command_id);
        }

        Self {
            bindings: RwLock::new(bindings),
            reserved: reserved_bindings,
        }
    }

    /// Registers a shortcut binding for a command.
    ///
    /// Returns `Err` if the binding conflicts with an existing or reserved shortcut.
    pub fn register(
        &self,
        binding: ShortcutBinding,
        command_id: CommandId,
    ) -> Result<(), CommandError> {
        // Check reserved first
        if self.is_reserved(&binding) {
            return Err(CommandError::ShortcutReserved {
                binding: binding.to_string(),
            });
        }

        let mut map = self
            .bindings
            .write()
            .expect("shortcut registry lock poisoned");

        if let Some(existing) = map.get(&binding) {
            return Err(CommandError::ShortcutConflict {
                binding: binding.to_string(),
                new_id: command_id.to_string(),
                existing_id: existing.to_string(),
            });
        }

        map.insert(binding, command_id);
        Ok(())
    }

    /// Deregisters a shortcut binding. Returns true if removed, false if not found.
    ///
    /// Reserved shortcuts cannot be deregistered.
    pub fn deregister(&self, binding: &ShortcutBinding) -> bool {
        if self.is_reserved(binding) {
            return false;
        }
        let mut map = self
            .bindings
            .write()
            .expect("shortcut registry lock poisoned");
        map.remove(binding).is_some()
    }

    /// Resolves a single chord to a command ID.
    ///
    /// Returns `Some(CommandId)` if the chord matches a single-chord binding.
    /// Returns `None` if no match.
    pub fn resolve_chord(&self, chord: &KeyChord) -> Option<CommandId> {
        let map = self
            .bindings
            .read()
            .expect("shortcut registry lock poisoned");
        let binding = ShortcutBinding::Single(chord.clone());
        map.get(&binding).cloned()
    }

    /// Resolves a two-chord sequence to a command ID.
    pub fn resolve_sequence(&self, first: &KeyChord, second: &KeyChord) -> Option<CommandId> {
        let map = self
            .bindings
            .read()
            .expect("shortcut registry lock poisoned");
        let binding = ShortcutBinding::Sequence(first.clone(), second.clone());
        map.get(&binding).cloned()
    }

    /// Returns true if the chord is the first part of any multi-key sequence.
    pub fn is_prefix(&self, chord: &KeyChord) -> bool {
        let map = self
            .bindings
            .read()
            .expect("shortcut registry lock poisoned");
        map.keys()
            .any(|binding| matches!(binding, ShortcutBinding::Sequence(first, _) if first == chord))
    }

    /// Checks if a binding is reserved (cannot be overridden).
    pub fn is_reserved(&self, binding: &ShortcutBinding) -> bool {
        self.reserved.contains(binding)
    }

    /// Lists all current bindings.
    pub fn list_all(&self) -> Vec<(ShortcutBinding, CommandId)> {
        let map = self
            .bindings
            .read()
            .expect("shortcut registry lock poisoned");
        map.iter().map(|(b, id)| (b.clone(), id.clone())).collect()
    }

    /// Gets the binding for a specific command, if any.
    pub fn binding_for(&self, command_id: &CommandId) -> Option<ShortcutBinding> {
        let map = self
            .bindings
            .read()
            .expect("shortcut registry lock poisoned");
        map.iter()
            .find(|(_, id)| *id == command_id)
            .map(|(binding, _)| binding.clone())
    }

    /// Loads user-configurable shortcut overrides from a TOML key map.
    ///
    /// Returns a list of errors for bindings that could not be applied
    /// (conflicts, reserved shortcuts, parse errors).
    pub fn load_user_overrides(&self, _keymap: &toml::Value) -> Vec<CommandError> {
        // Placeholder: real implementation parses TOML keybinding section.
        // For now, return empty — will be wired when configuration-system exists.
        Vec::new()
    }
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: ShortcutRegistry is Send + Sync because the inner RwLock provides
// thread-safe access and all stored types are Send + Sync.
unsafe impl Send for ShortcutRegistry {}
unsafe impl Sync for ShortcutRegistry {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shortcut::chord::{KeyCode, Modifiers};

    fn chord(mods: Modifiers, key: KeyCode) -> KeyChord {
        KeyChord::new(mods, key)
    }

    // Validates: Requirement 5.1
    #[test]
    fn register_and_resolve_single_chord_binding() {
        let registry = ShortcutRegistry::new();
        let binding = ShortcutBinding::Single(chord(Modifiers::ctrl(), KeyCode::B));
        let cmd_id = CommandId::new("edit.toggle_bold").unwrap();

        registry.register(binding.clone(), cmd_id.clone()).unwrap();

        let resolved = registry.resolve_chord(&chord(Modifiers::ctrl(), KeyCode::B));
        assert_eq!(resolved, Some(cmd_id));
    }

    // Validates: Requirement 5.4
    #[test]
    fn conflict_detection_rejects_duplicate_binding() {
        let registry = ShortcutRegistry::new();
        let binding = ShortcutBinding::Single(chord(Modifiers::ctrl(), KeyCode::B));
        let cmd1 = CommandId::new("edit.bold").unwrap();
        let cmd2 = CommandId::new("edit.bookmark").unwrap();

        registry.register(binding.clone(), cmd1).unwrap();
        let result = registry.register(binding, cmd2);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandError::ShortcutConflict { .. }
        ));
    }

    // Validates: Requirement 5.3, 5.5
    #[test]
    fn reserved_shortcut_cannot_be_registered() {
        let registry = ShortcutRegistry::new();
        let binding = ShortcutBinding::Single(chord(Modifiers::ctrl(), KeyCode::S));
        let cmd = CommandId::new("plugin.custom_save").unwrap();

        let result = registry.register(binding, cmd);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandError::ShortcutReserved { .. }
        ));
    }

    // Validates: Requirement 5.3
    #[test]
    fn is_reserved_returns_true_for_reserved_bindings() {
        let registry = ShortcutRegistry::new();
        let ctrl_s = ShortcutBinding::Single(chord(Modifiers::ctrl(), KeyCode::S));
        let f1 = ShortcutBinding::Single(chord(Modifiers::none(), KeyCode::F1));

        assert!(registry.is_reserved(&ctrl_s));
        assert!(registry.is_reserved(&f1));
    }

    // Validates: Requirement 5.3
    #[test]
    fn is_reserved_returns_false_for_non_reserved() {
        let registry = ShortcutRegistry::new();
        let ctrl_b = ShortcutBinding::Single(chord(Modifiers::ctrl(), KeyCode::B));

        assert!(!registry.is_reserved(&ctrl_b));
    }

    // Validates: Requirement 5.2
    #[test]
    fn register_and_resolve_multi_key_sequence() {
        let registry = ShortcutRegistry::new();
        let first = chord(Modifiers::ctrl(), KeyCode::K);
        let second = chord(Modifiers::ctrl(), KeyCode::C);
        let binding = ShortcutBinding::Sequence(first.clone(), second.clone());
        let cmd = CommandId::new("edit.comment_line").unwrap();

        registry.register(binding, cmd.clone()).unwrap();

        assert!(registry.is_prefix(&first));
        let resolved = registry.resolve_sequence(&first, &second);
        assert_eq!(resolved, Some(cmd));
    }

    // Validates: Requirement 5.2
    #[test]
    fn is_prefix_returns_false_when_no_sequence_starts_with_chord() {
        let registry = ShortcutRegistry::new();
        let chord = chord(Modifiers::ctrl(), KeyCode::Q);
        assert!(!registry.is_prefix(&chord));
    }

    #[test]
    fn deregister_removes_non_reserved_binding() {
        let registry = ShortcutRegistry::new();
        let binding = ShortcutBinding::Single(chord(Modifiers::ctrl(), KeyCode::B));
        let cmd = CommandId::new("edit.bold").unwrap();

        registry.register(binding.clone(), cmd).unwrap();
        assert!(registry.deregister(&binding));
        assert!(registry
            .resolve_chord(&chord(Modifiers::ctrl(), KeyCode::B))
            .is_none());
    }

    #[test]
    fn deregister_cannot_remove_reserved_binding() {
        let registry = ShortcutRegistry::new();
        let binding = ShortcutBinding::Single(chord(Modifiers::ctrl(), KeyCode::S));
        assert!(!registry.deregister(&binding));
        // Still resolves
        assert!(registry
            .resolve_chord(&chord(Modifiers::ctrl(), KeyCode::S))
            .is_some());
    }

    #[test]
    fn reserved_shortcuts_resolve_to_correct_commands() {
        let registry = ShortcutRegistry::new();

        let ctrl_z = registry.resolve_chord(&chord(Modifiers::ctrl(), KeyCode::Z));
        assert_eq!(ctrl_z.unwrap().as_str(), "edit.undo");

        let ctrl_s = registry.resolve_chord(&chord(Modifiers::ctrl(), KeyCode::S));
        assert_eq!(ctrl_s.unwrap().as_str(), "file.save");

        let f1 = registry.resolve_chord(&chord(Modifiers::none(), KeyCode::F1));
        assert_eq!(f1.unwrap().as_str(), "help.show");
    }
}
