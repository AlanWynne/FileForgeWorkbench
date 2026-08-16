//! CRITERIA command registration and argument parsing.
//!
//! Registers the CRITERIA command (alias: SELECT) with subcommands
//! SET/CLEAR/SHOW/SAVE in the command framework.

use crate::error::CriteriaError;

/// Parsed CRITERIA command operation.
///
/// Addresses: Requirement 6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CriteriaCommand {
    /// Open the Criteria_Panel (no subcommand).
    OpenPanel,
    /// Load a named criteria set: `CRITERIA SET <name>` or `CRITERIA LOAD <name>`.
    Set {
        /// The name of the criteria set to load.
        name: String,
    },
    /// Clear the active criteria: `CRITERIA CLEAR`.
    Clear,
    /// Show current criteria state: `CRITERIA SHOW` or `CRITERIA STATUS`.
    Show,
    /// Save current criteria: `CRITERIA SAVE <name>`.
    Save {
        /// The name to save the criteria set under.
        name: String,
    },
}

impl CriteriaCommand {
    /// Parse command arguments into a CriteriaCommand.
    ///
    /// Expected formats:
    /// - (empty) → `OpenPanel`
    /// - `SET <name>` or `LOAD <name>` → `Set { name }`
    /// - `CLEAR` → `Clear`
    /// - `SHOW` or `STATUS` → `Show`
    /// - `SAVE <name>` → `Save { name }`
    ///
    /// Addresses: Requirement 6 AC 2–7
    pub fn parse(args: &str) -> Result<Self, CriteriaError> {
        let args = args.trim();
        if args.is_empty() {
            return Ok(Self::OpenPanel);
        }

        let mut parts = args.splitn(2, char::is_whitespace);
        let subcommand = parts.next().unwrap_or("").to_uppercase();
        let remainder = parts.next().unwrap_or("").trim();

        match subcommand.as_str() {
            "SET" | "LOAD" => {
                if remainder.is_empty() {
                    return Err(CriteriaError::InvalidCommandArg {
                        arg: String::from("SET requires a criteria set name"),
                    });
                }
                Ok(Self::Set {
                    name: remainder.to_string(),
                })
            }
            "CLEAR" => Ok(Self::Clear),
            "SHOW" | "STATUS" => Ok(Self::Show),
            "SAVE" => {
                if remainder.is_empty() {
                    return Err(CriteriaError::InvalidCommandArg {
                        arg: String::from("SAVE requires a criteria set name"),
                    });
                }
                Ok(Self::Save {
                    name: remainder.to_string(),
                })
            }
            _ => Err(CriteriaError::InvalidCommandArg { arg: subcommand }),
        }
    }
}

/// Command metadata for the CRITERIA command.
#[derive(Debug, Clone)]
pub struct CriteriaCommandMetadata {
    /// Command ID.
    pub id: &'static str,
    /// Display name.
    pub display_name: &'static str,
    /// Description.
    pub description: &'static str,
    /// Category.
    pub category: &'static str,
    /// Aliases.
    pub aliases: &'static [&'static str],
}

/// Returns the command metadata for the CRITERIA command.
///
/// Addresses: Requirement 6 AC 1, 8
pub fn criteria_command_metadata() -> CriteriaCommandMetadata {
    CriteriaCommandMetadata {
        id: "criteria",
        display_name: "Criteria",
        description: "Manage record selection criteria (SET/CLEAR/SHOW/SAVE)",
        category: "criteria",
        aliases: &["select"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_returns_open_panel() {
        assert_eq!(
            CriteriaCommand::parse("").unwrap(),
            CriteriaCommand::OpenPanel
        );
        assert_eq!(
            CriteriaCommand::parse("  ").unwrap(),
            CriteriaCommand::OpenPanel
        );
    }

    #[test]
    fn parse_set_with_name() {
        assert_eq!(
            CriteriaCommand::parse("SET my_filter").unwrap(),
            CriteriaCommand::Set {
                name: "my_filter".to_string()
            }
        );
    }

    #[test]
    fn parse_load_alias_for_set() {
        assert_eq!(
            CriteriaCommand::parse("LOAD my_filter").unwrap(),
            CriteriaCommand::Set {
                name: "my_filter".to_string()
            }
        );
    }

    #[test]
    fn parse_set_case_insensitive() {
        assert_eq!(
            CriteriaCommand::parse("set My Filter").unwrap(),
            CriteriaCommand::Set {
                name: "My Filter".to_string()
            }
        );
    }

    #[test]
    fn parse_set_without_name_returns_error() {
        let result = CriteriaCommand::parse("SET");
        assert!(matches!(
            result,
            Err(CriteriaError::InvalidCommandArg { .. })
        ));
    }

    #[test]
    fn parse_clear() {
        assert_eq!(
            CriteriaCommand::parse("CLEAR").unwrap(),
            CriteriaCommand::Clear
        );
        assert_eq!(
            CriteriaCommand::parse("clear").unwrap(),
            CriteriaCommand::Clear
        );
    }

    #[test]
    fn parse_show() {
        assert_eq!(
            CriteriaCommand::parse("SHOW").unwrap(),
            CriteriaCommand::Show
        );
        assert_eq!(
            CriteriaCommand::parse("show").unwrap(),
            CriteriaCommand::Show
        );
    }

    #[test]
    fn parse_status_alias_for_show() {
        assert_eq!(
            CriteriaCommand::parse("STATUS").unwrap(),
            CriteriaCommand::Show
        );
    }

    #[test]
    fn parse_save_with_name() {
        assert_eq!(
            CriteriaCommand::parse("SAVE new_filter").unwrap(),
            CriteriaCommand::Save {
                name: "new_filter".to_string()
            }
        );
    }

    #[test]
    fn parse_save_without_name_returns_error() {
        let result = CriteriaCommand::parse("SAVE");
        assert!(matches!(
            result,
            Err(CriteriaError::InvalidCommandArg { .. })
        ));
    }

    #[test]
    fn parse_invalid_subcommand_returns_error() {
        let result = CriteriaCommand::parse("INVALID");
        assert!(matches!(
            result,
            Err(CriteriaError::InvalidCommandArg { .. })
        ));
    }

    #[test]
    fn command_metadata_has_correct_category() {
        let meta = criteria_command_metadata();
        assert_eq!(meta.category, "criteria");
        assert_eq!(meta.id, "criteria");
        assert!(meta.aliases.contains(&"select"));
    }
}
