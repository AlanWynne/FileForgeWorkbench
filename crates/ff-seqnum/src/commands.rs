//! Command registration and dispatch integration.
//!
//! Registers UNNUM, NUMBER, and NUMBER SHOW with the command framework.

use crate::traits::CommandRegistry;

/// Command ID for the UNNUM command.
pub const UNNUM_COMMAND_ID: &str = "sequence.unnum";

/// Command ID for the NUMBER command.
pub const NUMBER_COMMAND_ID: &str = "sequence.number";

/// Command ID for the NUMBER SHOW command.
pub const NUMBER_SHOW_COMMAND_ID: &str = "sequence.number_show";

/// Register all sequence number commands with the command registry.
///
/// - UNNUM: valid in Edit and Browse modes
/// - NUMBER: valid in Edit mode only
/// - NUMBER SHOW: valid in Edit and Browse modes
pub fn register_commands(registry: &mut dyn CommandRegistry) {
    registry.register_command(
        UNNUM_COMMAND_ID,
        "Remove sequence numbers from the current document",
        true, // valid_in_edit
        true, // valid_in_browse
    );

    registry.register_command(
        NUMBER_COMMAND_ID,
        "Write sequential numbers into defined column positions",
        true,  // valid_in_edit
        false, // NOT valid_in_browse
    );

    registry.register_command(
        NUMBER_SHOW_COMMAND_ID,
        "Toggle sequence number display overlay",
        true, // valid_in_edit
        true, // valid_in_browse
    );
}

/// Check if a command is valid in the current editor mode.
///
/// Returns an error message if the command is not valid, or None if it is valid.
pub fn validate_mode(
    command_id: &str,
    is_edit_mode: bool,
    is_grid_edit_mode: bool,
) -> Option<String> {
    // Grid_Edit_Mode rejects all sequence commands
    if is_grid_edit_mode {
        let cmd_name = match command_id {
            UNNUM_COMMAND_ID => "UNNUM",
            NUMBER_COMMAND_ID => "NUMBER",
            NUMBER_SHOW_COMMAND_ID => "NUMBER SHOW",
            _ => "Unknown",
        };
        return Some(format!("{cmd_name}: not applicable in Grid Edit Mode"));
    }

    // NUMBER is Edit-mode only
    if command_id == NUMBER_COMMAND_ID && !is_edit_mode {
        return Some("NUMBER: not valid in Browse mode".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRegistry {
        commands: Vec<(String, String, bool, bool)>,
    }

    impl MockRegistry {
        fn new() -> Self {
            Self {
                commands: Vec::new(),
            }
        }
    }

    impl CommandRegistry for MockRegistry {
        fn register_command(
            &mut self,
            command_id: &str,
            description: &str,
            valid_in_edit: bool,
            valid_in_browse: bool,
        ) {
            self.commands.push((
                command_id.to_string(),
                description.to_string(),
                valid_in_edit,
                valid_in_browse,
            ));
        }
    }

    #[test]
    fn registers_all_commands() {
        // Validates: Requirements 5.1, 6.1, 8.1
        let mut registry = MockRegistry::new();
        register_commands(&mut registry);

        assert_eq!(registry.commands.len(), 3);
    }

    #[test]
    fn unnum_registered_for_edit_and_browse() {
        // Validates: Requirement 5.1
        let mut registry = MockRegistry::new();
        register_commands(&mut registry);

        let unnum = registry
            .commands
            .iter()
            .find(|(id, _, _, _)| id == UNNUM_COMMAND_ID)
            .unwrap();
        assert!(unnum.2); // valid_in_edit
        assert!(unnum.3); // valid_in_browse
    }

    #[test]
    fn number_registered_for_edit_only() {
        // Validates: Requirement 6.1
        let mut registry = MockRegistry::new();
        register_commands(&mut registry);

        let number = registry
            .commands
            .iter()
            .find(|(id, _, _, _)| id == NUMBER_COMMAND_ID)
            .unwrap();
        assert!(number.2); // valid_in_edit
        assert!(!number.3); // NOT valid_in_browse
    }

    #[test]
    fn number_show_registered_for_edit_and_browse() {
        // Validates: Requirement 8.1
        let mut registry = MockRegistry::new();
        register_commands(&mut registry);

        let show = registry
            .commands
            .iter()
            .find(|(id, _, _, _)| id == NUMBER_SHOW_COMMAND_ID)
            .unwrap();
        assert!(show.2); // valid_in_edit
        assert!(show.3); // valid_in_browse
    }

    #[test]
    fn grid_edit_mode_rejects_all_commands() {
        // Validates: Requirements 13.1, 13.2
        let result = validate_mode(UNNUM_COMMAND_ID, true, true);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Grid Edit Mode"));

        let result = validate_mode(NUMBER_COMMAND_ID, true, true);
        assert!(result.is_some());

        let result = validate_mode(NUMBER_SHOW_COMMAND_ID, true, true);
        assert!(result.is_some());
    }

    #[test]
    fn number_rejected_in_browse_mode() {
        // Validates: Requirement 6.1
        let result = validate_mode(NUMBER_COMMAND_ID, false, false);
        assert!(result.is_some());
        assert!(result.unwrap().contains("Browse mode"));
    }

    #[test]
    fn unnum_valid_in_browse_mode() {
        // Validates: Requirement 5.1
        let result = validate_mode(UNNUM_COMMAND_ID, false, false);
        assert!(result.is_none());
    }

    #[test]
    fn all_commands_valid_in_edit_mode() {
        // Validates: Requirements 5.1, 6.1, 8.1
        assert!(validate_mode(UNNUM_COMMAND_ID, true, false).is_none());
        assert!(validate_mode(NUMBER_COMMAND_ID, true, false).is_none());
        assert!(validate_mode(NUMBER_SHOW_COMMAND_ID, true, false).is_none());
    }
}
