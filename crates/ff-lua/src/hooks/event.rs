//! Hook event types and parameters.
//!
//! Defines the supported editor events that macros can respond to.
//! Addresses: Requirement 3 AC 1

/// Identifies a supported event hook type with its parameters.
///
/// Addresses: Requirement 3 AC 1
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HookEvent {
    /// File opened and buffer ready. Param: file_path.
    OnOpen {
        /// Path of the opened file.
        file_path: String,
    },
    /// Before file save (cancellable). Param: file_path.
    OnBeforeSave {
        /// Path of the file about to be saved.
        file_path: String,
    },
    /// After file saved. Param: file_path.
    OnAfterSave {
        /// Path of the saved file.
        file_path: String,
    },
    /// Buffer closing. Param: file_path.
    OnClose {
        /// Path of the file being closed.
        file_path: String,
    },
    /// Active buffer switched. Param: file_path of new buffer.
    OnSwitchBuffer {
        /// Path of the newly active buffer.
        file_path: String,
    },
    /// Character inserted (not cancellable). Param: character.
    OnChar {
        /// The character that was inserted.
        character: char,
    },
    /// Key pressed (cancellable). Params: key_code, modifiers.
    OnKey {
        /// Name of the key.
        key_code: String,
        /// Whether Shift is held.
        shift: bool,
        /// Whether Ctrl is held.
        ctrl: bool,
        /// Whether Alt is held.
        alt: bool,
    },
    /// Command about to execute (cancellable). Params: command_id, params.
    OnCommand {
        /// ID of the command being executed.
        command_id: String,
        /// Parameters passed to the command.
        params: String,
    },
    /// Error occurred in another hook/macro. Param: error_message.
    OnError {
        /// The error message.
        error_message: String,
    },
}

impl HookEvent {
    /// Returns the Lua global function name for this event.
    pub fn lua_function_name(&self) -> &'static str {
        match self {
            HookEvent::OnOpen { .. } => "OnOpen",
            HookEvent::OnBeforeSave { .. } => "OnBeforeSave",
            HookEvent::OnAfterSave { .. } => "OnAfterSave",
            HookEvent::OnClose { .. } => "OnClose",
            HookEvent::OnSwitchBuffer { .. } => "OnSwitchBuffer",
            HookEvent::OnChar { .. } => "OnChar",
            HookEvent::OnKey { .. } => "OnKey",
            HookEvent::OnCommand { .. } => "OnCommand",
            HookEvent::OnError { .. } => "OnError",
        }
    }

    /// Whether this event type is cancellable (handler can return false to cancel).
    ///
    /// Addresses: Requirement 3 AC 3, 3.4, 3.5, 3.7
    pub fn is_cancellable(&self) -> bool {
        matches!(
            self,
            HookEvent::OnBeforeSave { .. } | HookEvent::OnKey { .. } | HookEvent::OnCommand { .. }
        )
    }

    /// Returns all known hook function names for discovery.
    pub fn all_hook_names() -> &'static [&'static str] {
        &[
            "OnOpen",
            "OnBeforeSave",
            "OnAfterSave",
            "OnClose",
            "OnSwitchBuffer",
            "OnChar",
            "OnKey",
            "OnCommand",
            "OnError",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.1
    #[test]
    fn hook_event_lua_function_names_are_correct() {
        assert_eq!(
            HookEvent::OnOpen {
                file_path: String::new()
            }
            .lua_function_name(),
            "OnOpen"
        );
        assert_eq!(
            HookEvent::OnBeforeSave {
                file_path: String::new()
            }
            .lua_function_name(),
            "OnBeforeSave"
        );
        assert_eq!(
            HookEvent::OnChar { character: 'a' }.lua_function_name(),
            "OnChar"
        );
        assert_eq!(
            HookEvent::OnKey {
                key_code: String::new(),
                shift: false,
                ctrl: false,
                alt: false,
            }
            .lua_function_name(),
            "OnKey"
        );
    }

    // Validates: Requirement 3.4, 3.5, 3.7
    #[test]
    fn cancellable_hooks_are_correctly_identified() {
        assert!(HookEvent::OnBeforeSave {
            file_path: String::new()
        }
        .is_cancellable());
        assert!(HookEvent::OnKey {
            key_code: String::new(),
            shift: false,
            ctrl: false,
            alt: false,
        }
        .is_cancellable());
        assert!(HookEvent::OnCommand {
            command_id: String::new(),
            params: String::new(),
        }
        .is_cancellable());
    }

    // Validates: Requirement 3.6
    #[test]
    fn non_cancellable_hooks_are_correctly_identified() {
        assert!(!HookEvent::OnChar { character: 'x' }.is_cancellable());
        assert!(!HookEvent::OnOpen {
            file_path: String::new()
        }
        .is_cancellable());
        assert!(!HookEvent::OnAfterSave {
            file_path: String::new()
        }
        .is_cancellable());
        assert!(!HookEvent::OnClose {
            file_path: String::new()
        }
        .is_cancellable());
        assert!(!HookEvent::OnSwitchBuffer {
            file_path: String::new()
        }
        .is_cancellable());
        assert!(!HookEvent::OnError {
            error_message: String::new()
        }
        .is_cancellable());
    }

    #[test]
    fn all_hook_names_lists_all_events() {
        let names = HookEvent::all_hook_names();
        assert_eq!(names.len(), 9);
        assert!(names.contains(&"OnOpen"));
        assert!(names.contains(&"OnChar"));
        assert!(names.contains(&"OnKey"));
        assert!(names.contains(&"OnError"));
    }
}
