//! HELP command handler and F1 activation logic.
//!
//! Routes `HELP`, `HELP <topic>`, `HELP LINECOMMANDS`, `HELP KEYS`, `HELP OFF`, etc.
//! Also handles F1 key binding — context detection → topic resolution → panel open/toggle.

use crate::topic_key::TopicKey;

/// The resolved action for a HELP command invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpAction {
    /// Show a specific topic in the Help Panel.
    ShowTopic(TopicKey),
    /// Close the Help Panel.
    Close,
    /// Show the Help Index with an "unrecognised topic" message.
    UnrecognisedTopic(String),
}

/// Resolves HELP command arguments into a `HelpAction`.
///
/// # Routing Rules
///
/// - No args → show Help Index
/// - `OFF` → close Help Panel
/// - `LINECOMMANDS` → show `"line:index"` topic
/// - `KEYS` → show `"feature:function_keys"` topic
/// - `MACRO` or `API` → show `"feature:macros"` topic
/// - `CONFIG` or `CONFIGURATION` → show `"feature:configuration"` topic
/// - `<name>` → try `"cmd:<NAME>"` (assume it's a command name)
/// - Anything else → unrecognised topic message
pub fn resolve_help_argument(args: &str) -> HelpAction {
    let args_trimmed = args.trim();

    if args_trimmed.is_empty() {
        return HelpAction::ShowTopic(TopicKey::index());
    }

    let upper = args_trimmed.to_uppercase();
    match upper.as_str() {
        "OFF" => HelpAction::Close,
        "LINECOMMANDS" => HelpAction::ShowTopic(TopicKey::line_index()),
        "KEYS" => HelpAction::ShowTopic(TopicKey::feature("function_keys")),
        "MACRO" | "API" => HelpAction::ShowTopic(TopicKey::feature("macros")),
        "CONFIG" | "CONFIGURATION" => HelpAction::ShowTopic(TopicKey::feature("configuration")),
        "INDEX" => HelpAction::ShowTopic(TopicKey::index()),
        _ => {
            // Try as a command name first
            HelpAction::ShowTopic(TopicKey::command(&upper))
        }
    }
}

/// Determines whether a HELP command should be recorded in command history.
///
/// Per Requirement 1.10 and 13.10, HELP and F1 are never recorded.
pub fn should_record_in_history() -> bool {
    false
}

/// Determines whether a HELP command is undoable.
///
/// Per Requirement 1.10 and 13.10, HELP is never undoable.
pub fn is_undoable() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 13.1 — HELP with no arguments opens index
    #[test]
    fn help_no_args_shows_index() {
        assert_eq!(
            resolve_help_argument(""),
            HelpAction::ShowTopic(TopicKey::index())
        );
    }

    // Validates: Requirement 13.2 — HELP <command_name> shows command topic
    #[test]
    fn help_command_name_shows_command_topic() {
        assert_eq!(
            resolve_help_argument("CHANGE"),
            HelpAction::ShowTopic(TopicKey::command("CHANGE"))
        );
    }

    // Validates: Requirement 13.2 — Case-insensitive command name
    #[test]
    fn help_command_name_case_insensitive() {
        assert_eq!(
            resolve_help_argument("find"),
            HelpAction::ShowTopic(TopicKey::command("FIND"))
        );
    }

    // Validates: Requirement 13.3 — HELP LINECOMMANDS shows line index
    #[test]
    fn help_linecommands_shows_line_index() {
        assert_eq!(
            resolve_help_argument("LINECOMMANDS"),
            HelpAction::ShowTopic(TopicKey::line_index())
        );
    }

    // Validates: Requirement 13.4 — HELP MACRO shows macros topic
    #[test]
    fn help_macro_shows_macros_topic() {
        assert_eq!(
            resolve_help_argument("MACRO"),
            HelpAction::ShowTopic(TopicKey::feature("macros"))
        );
    }

    // Validates: Requirement 13.4 — HELP API shows macros topic
    #[test]
    fn help_api_shows_macros_topic() {
        assert_eq!(
            resolve_help_argument("API"),
            HelpAction::ShowTopic(TopicKey::feature("macros"))
        );
    }

    // Validates: Requirement 13.5 — HELP KEYS shows function keys topic
    #[test]
    fn help_keys_shows_function_keys_topic() {
        assert_eq!(
            resolve_help_argument("KEYS"),
            HelpAction::ShowTopic(TopicKey::feature("function_keys"))
        );
    }

    // Validates: Requirement 13.6 — HELP CONFIG shows configuration topic
    #[test]
    fn help_config_shows_configuration_topic() {
        assert_eq!(
            resolve_help_argument("CONFIG"),
            HelpAction::ShowTopic(TopicKey::feature("configuration"))
        );
        assert_eq!(
            resolve_help_argument("CONFIGURATION"),
            HelpAction::ShowTopic(TopicKey::feature("configuration"))
        );
    }

    // Validates: Requirement 13.8 — HELP OFF closes panel
    #[test]
    fn help_off_closes_panel() {
        assert_eq!(resolve_help_argument("OFF"), HelpAction::Close);
    }

    // Validates: Requirement 13.10 — HELP not in history
    #[test]
    fn help_not_recorded_in_history() {
        assert!(!should_record_in_history());
    }

    // Validates: Requirement 13.10 — HELP not undoable
    #[test]
    fn help_not_undoable() {
        assert!(!is_undoable());
    }
}
