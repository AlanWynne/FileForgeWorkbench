//! Reserved shortcut definitions — globally reserved bindings that cannot be overridden.

use super::chord::{KeyChord, KeyCode, Modifiers};
use super::sequence::ShortcutBinding;
use crate::id::CommandId;

/// A reserved shortcut entry: binding and the command it maps to.
pub struct ReservedShortcut {
    /// The keyboard binding.
    pub binding: ShortcutBinding,
    /// The command ID this shortcut is reserved for.
    pub command_id: CommandId,
}

/// Returns the complete list of reserved shortcuts.
///
/// These bindings cannot be overridden by user configuration or plugins.
pub fn reserved_shortcuts() -> Vec<ReservedShortcut> {
    vec![
        // F1 → help.show
        reserved_single(Modifiers::none(), KeyCode::F1, "help.show"),
        // Ctrl+Plus → view.zoom_in
        reserved_single(Modifiers::ctrl(), KeyCode::Plus, "view.zoom_in"),
        // Ctrl+Minus → view.zoom_out
        reserved_single(Modifiers::ctrl(), KeyCode::Minus, "view.zoom_out"),
        // Ctrl+0 → view.zoom_reset
        reserved_single(Modifiers::ctrl(), KeyCode::Key0, "view.zoom_reset"),
        // Ctrl+Z → edit.undo
        reserved_single(Modifiers::ctrl(), KeyCode::Z, "edit.undo"),
        // Ctrl+Y → edit.redo
        reserved_single(Modifiers::ctrl(), KeyCode::Y, "edit.redo"),
        // Ctrl+Shift+Z → edit.redo
        reserved_single(Modifiers::ctrl_shift(), KeyCode::Z, "edit.redo"),
        // Ctrl+C → edit.copy
        reserved_single(Modifiers::ctrl(), KeyCode::C, "edit.copy"),
        // Ctrl+X → edit.cut
        reserved_single(Modifiers::ctrl(), KeyCode::X, "edit.cut"),
        // Ctrl+V → edit.paste
        reserved_single(Modifiers::ctrl(), KeyCode::V, "edit.paste"),
        // Ctrl+A → edit.select_all
        reserved_single(Modifiers::ctrl(), KeyCode::A, "edit.select_all"),
        // Ctrl+S → file.save
        reserved_single(Modifiers::ctrl(), KeyCode::S, "file.save"),
        // Ctrl+F → find.focus
        reserved_single(Modifiers::ctrl(), KeyCode::F, "find.focus"),
        // Ctrl+H → find.change
        reserved_single(Modifiers::ctrl(), KeyCode::H, "find.change"),
        // Ctrl+G → navigate.goto_line
        reserved_single(Modifiers::ctrl(), KeyCode::G, "navigate.goto_line"),
        // Ctrl+Tab → tab.next
        reserved_single(Modifiers::ctrl(), KeyCode::Tab, "tab.next"),
        // Ctrl+Shift+Tab → tab.previous
        reserved_single(Modifiers::ctrl_shift(), KeyCode::Tab, "tab.previous"),
        // Ctrl+W → tab.close
        reserved_single(Modifiers::ctrl(), KeyCode::W, "tab.close"),
        // Ctrl+N → tab.new
        reserved_single(Modifiers::ctrl(), KeyCode::N, "tab.new"),
        // Ctrl+Shift+D → layout.dock_toggle
        reserved_single(Modifiers::ctrl_shift(), KeyCode::D, "layout.dock_toggle"),
        // Ctrl+Shift+T → layout.tab_undock
        reserved_single(Modifiers::ctrl_shift(), KeyCode::T, "layout.tab_undock"),
    ]
}

fn reserved_single(modifiers: Modifiers, key: KeyCode, command: &str) -> ReservedShortcut {
    ReservedShortcut {
        binding: ShortcutBinding::Single(KeyChord::new(modifiers, key)),
        // Safe: all reserved command IDs are valid by construction.
        command_id: CommandId::new(command).expect("reserved command ID must be valid"),
    }
}
