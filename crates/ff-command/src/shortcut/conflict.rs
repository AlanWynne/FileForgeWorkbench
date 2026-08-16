//! Conflict detection logic for shortcut bindings.

use super::chord::KeyChord;
use super::sequence::ShortcutBinding;
use crate::id::CommandId;

/// The result of checking for conflicts when registering a binding.
#[derive(Debug)]
pub enum ConflictResult {
    /// No conflict — safe to register.
    None,
    /// Conflicts with an existing non-reserved binding.
    Conflict {
        /// The binding that conflicts.
        binding: ShortcutBinding,
        /// The existing command that owns the binding.
        existing_id: CommandId,
    },
    /// Conflicts with a reserved shortcut.
    Reserved {
        /// The reserved binding.
        binding: ShortcutBinding,
    },
}

/// Checks if two bindings conflict.
///
/// Two bindings conflict if they share the same first chord when one is a
/// single binding and the other is the first chord of a sequence, or if
/// they are identical.
pub fn bindings_conflict(a: &ShortcutBinding, b: &ShortcutBinding) -> bool {
    a == b
}

/// Extracts the first chord from a binding for prefix checking.
pub fn first_chord(binding: &ShortcutBinding) -> &KeyChord {
    match binding {
        ShortcutBinding::Single(chord) => chord,
        ShortcutBinding::Sequence(first, _) => first,
    }
}
