//! CommandHandler implementations for line command operations.
//!
//! Each line command type is registered with the command framework and
//! dispatched through the standard command path.

/// Command IDs for line command operations.
pub mod command_ids {
    /// Delete line command.
    pub const DELETE: &str = "linecmd.delete";
    /// Insert line command.
    pub const INSERT: &str = "linecmd.insert";
    /// Repeat line command.
    pub const REPEAT: &str = "linecmd.repeat";
    /// Copy line command.
    pub const COPY: &str = "linecmd.copy";
    /// Move line command.
    pub const MOVE: &str = "linecmd.move";
    /// Exclude line command.
    pub const EXCLUDE: &str = "linecmd.exclude";
    /// Tag line command.
    pub const TAG: &str = "linecmd.tag";
    /// Untag line command.
    pub const UNTAG: &str = "linecmd.untag";
    /// Shift right line command.
    pub const SHIFT_RIGHT: &str = "linecmd.shift_right";
    /// Shift left line command.
    pub const SHIFT_LEFT: &str = "linecmd.shift_left";
    /// Bounds-aware shift right.
    pub const BOUNDS_SHIFT_RIGHT: &str = "linecmd.bounds_shift_right";
    /// Bounds-aware shift left.
    pub const BOUNDS_SHIFT_LEFT: &str = "linecmd.bounds_shift_left";
    /// Overlay line command (O, On).
    pub const OVERLAY: &str = "linecmd.overlay";
    /// Clipboard copy line command (W, WW).
    pub const CLIPBOARD_COPY: &str = "linecmd.clipboard_copy";
    /// Show first of excluded block (F).
    pub const SHOW_FIRST: &str = "linecmd.show_first";
    /// Show last of excluded block (L).
    pub const SHOW_LAST: &str = "linecmd.show_last";
    /// Show line of excluded block (S).
    pub const SHOW_LINE: &str = "linecmd.show_line";
    /// Single-column shift right (]).
    pub const SHIFT_RIGHT_ONE: &str = "linecmd.shift_right_one";
    /// Resolution cycle -- main entry point invoked by primary command execution.
    pub const RESOLVE_CYCLE: &str = "linecmd.resolve_cycle";
    /// Reset all pending commands.
    pub const RESET: &str = "linecmd.reset";
}

/// Returns all command IDs registered by this crate.
pub fn all_command_ids() -> &'static [&'static str] {
    &[
        command_ids::DELETE,
        command_ids::INSERT,
        command_ids::REPEAT,
        command_ids::COPY,
        command_ids::MOVE,
        command_ids::EXCLUDE,
        command_ids::TAG,
        command_ids::UNTAG,
        command_ids::SHIFT_RIGHT,
        command_ids::SHIFT_LEFT,
        command_ids::BOUNDS_SHIFT_RIGHT,
        command_ids::BOUNDS_SHIFT_LEFT,
        command_ids::OVERLAY,
        command_ids::CLIPBOARD_COPY,
        command_ids::SHOW_FIRST,
        command_ids::SHOW_LAST,
        command_ids::SHOW_LINE,
        command_ids::SHIFT_RIGHT_ONE,
        command_ids::RESOLVE_CYCLE,
        command_ids::RESET,
    ]
}

/// Returns true if the given command ID is an undoable line command.
pub fn is_undoable(command_id: &str) -> bool {
    matches!(
        command_id,
        command_ids::DELETE
            | command_ids::INSERT
            | command_ids::REPEAT
            | command_ids::COPY
            | command_ids::MOVE
            | command_ids::SHIFT_RIGHT
            | command_ids::SHIFT_LEFT
            | command_ids::BOUNDS_SHIFT_RIGHT
            | command_ids::BOUNDS_SHIFT_LEFT
            | command_ids::OVERLAY
            | command_ids::SHIFT_RIGHT_ONE
    )
}

/// Returns true if the given command ID is a session-state operation (no undo).
pub fn is_session_state(command_id: &str) -> bool {
    matches!(
        command_id,
        command_ids::EXCLUDE
            | command_ids::TAG
            | command_ids::UNTAG
            | command_ids::CLIPBOARD_COPY
            | command_ids::SHOW_FIRST
            | command_ids::SHOW_LAST
            | command_ids::SHOW_LINE
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_command_ids_contains_expected_count() {
        assert_eq!(all_command_ids().len(), 20);
    }

    #[test]
    fn delete_is_undoable() {
        assert!(is_undoable(command_ids::DELETE));
    }

    #[test]
    fn exclude_is_not_undoable() {
        assert!(!is_undoable(command_ids::EXCLUDE));
    }

    #[test]
    fn exclude_is_session_state() {
        assert!(is_session_state(command_ids::EXCLUDE));
    }

    #[test]
    fn tag_is_session_state() {
        assert!(is_session_state(command_ids::TAG));
    }

    #[test]
    fn shift_right_is_undoable() {
        assert!(is_undoable(command_ids::SHIFT_RIGHT));
    }

    #[test]
    fn reset_is_neither_undoable_nor_session_state() {
        assert!(!is_undoable(command_ids::RESET));
        assert!(!is_session_state(command_ids::RESET));
    }
}
