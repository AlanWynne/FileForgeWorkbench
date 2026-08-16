//! Command framework registration for EXCLUDE, SHOW, and RESET commands.
//!
//! Registers all exclude-show-filter commands with the `ff-command` framework,
//! including aliases (X for EXCLUDE, INCLUDE for SHOW) and line command entries.
//!
//! Addresses: Requirement 9 (AC 9.1–9.8)

use ff_command::{CommandMetadata, CommandRegistry};

/// Command IDs for the exclude-show-filter commands.
pub mod ids {
    use ff_command::CommandId;

    /// Command ID for the EXCLUDE primary command.
    pub fn exclude() -> CommandId {
        CommandId::new("filter.exclude").expect("valid command id")
    }

    /// Command ID for the SHOW primary command.
    pub fn show() -> CommandId {
        CommandId::new("filter.show").expect("valid command id")
    }

    /// Command ID for the RESET primary command (exclusion aspects).
    pub fn reset() -> CommandId {
        CommandId::new("filter.reset").expect("valid command id")
    }

    /// Command ID for the X line command.
    pub fn line_x() -> CommandId {
        CommandId::new("filter.line_x").expect("valid command id")
    }

    /// Command ID for the Xn line command.
    pub fn line_xn() -> CommandId {
        CommandId::new("filter.line_xn").expect("valid command id")
    }

    /// Command ID for the XX block line command.
    pub fn line_xx() -> CommandId {
        CommandId::new("filter.line_xx").expect("valid command id")
    }
}

/// Metadata for the EXCLUDE command.
fn exclude_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "Exclude",
        "Hide lines from the viewport by text match, regex, range, ALL, or TAGGED",
    )
    .category("filter")
    .build()
}

/// Metadata for the SHOW command.
fn show_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "Show",
        "Reveal excluded lines by text match, regex, ALL, or EXCLUDED",
    )
    .category("filter")
    .build()
}

/// Metadata for the RESET command (exclusion aspects).
fn reset_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "Reset",
        "Clear exclusion state: RESET (no args), RESET EXCLUDED, RESET ALL",
    )
    .category("filter")
    .build()
}

/// Metadata for the X line command.
fn line_x_metadata() -> CommandMetadata {
    CommandMetadata::builder(
        "X Line Command",
        "Exclude a single line or block via X/Xn/XX prefix commands",
    )
    .category("filter")
    .build()
}

/// Command handler for EXCLUDE that is non-undoable.
///
/// Addresses: Requirement 9 AC 7 — explicitly marked as non-undoable.
pub struct ExcludeCommandHandler;

impl ff_command::CommandHandler for ExcludeCommandHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(
        &self,
        _ctx: &ff_command::ExecutionContext,
        _params: &ff_command::CommandParams,
    ) -> ff_command::CommandResult {
        // Command execution is dispatched through ExclusionEngine directly.
        // This handler serves as the registration entry point and metadata carrier.
        // Actual logic is invoked by the command-semantics dispatcher which
        // calls ExclusionEngine methods with parsed arguments.
        ff_command::CommandResult::Ok
    }
}

/// Command handler for SHOW/INCLUDE that is non-undoable.
///
/// Addresses: Requirement 9 AC 7
pub struct ShowCommandHandler;

impl ff_command::CommandHandler for ShowCommandHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(
        &self,
        _ctx: &ff_command::ExecutionContext,
        _params: &ff_command::CommandParams,
    ) -> ff_command::CommandResult {
        ff_command::CommandResult::Ok
    }
}

/// Command handler for RESET that is non-undoable.
///
/// Addresses: Requirement 9 AC 7
pub struct ResetCommandHandler;

impl ff_command::CommandHandler for ResetCommandHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(
        &self,
        _ctx: &ff_command::ExecutionContext,
        _params: &ff_command::CommandParams,
    ) -> ff_command::CommandResult {
        ff_command::CommandResult::Ok
    }
}

/// Command handler for X/Xn/XX line commands that is non-undoable.
///
/// Addresses: Requirement 9 AC 7
pub struct LineExcludeCommandHandler;

impl ff_command::CommandHandler for LineExcludeCommandHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(
        &self,
        _ctx: &ff_command::ExecutionContext,
        _params: &ff_command::CommandParams,
    ) -> ff_command::CommandResult {
        ff_command::CommandResult::Ok
    }
}

/// Register all exclude-show-filter commands with the command registry.
///
/// This function registers:
/// - `filter.exclude` (EXCLUDE / X alias) — non-undoable
/// - `filter.show` (SHOW / INCLUDE alias) — non-undoable
/// - `filter.reset` (RESET) — non-undoable
/// - `filter.line_x` (X/Xn/XX line commands) — non-undoable
///
/// All commands are valid in both Edit mode and Browse/View mode.
///
/// # Errors
///
/// Returns an error if any command ID is already registered.
///
/// Addresses: Requirement 9 AC 1–4, 9 AC 6, 9 AC 7
pub fn register_commands(registry: &CommandRegistry) -> Result<(), ff_command::CommandError> {
    registry.register(
        ids::exclude(),
        exclude_metadata(),
        Box::new(ExcludeCommandHandler),
    )?;

    registry.register(ids::show(), show_metadata(), Box::new(ShowCommandHandler))?;

    registry.register(
        ids::reset(),
        reset_metadata(),
        Box::new(ResetCommandHandler),
    )?;

    registry.register(
        ids::line_x(),
        line_x_metadata(),
        Box::new(LineExcludeCommandHandler),
    )?;

    Ok(())
}

/// Recognized command names and aliases for argument parsing.
///
/// These constants define the primary command names and their aliases
/// for use by the command-semantics parser.
pub mod aliases {
    /// Primary name for the EXCLUDE command.
    pub const EXCLUDE: &str = "EXCLUDE";
    /// Alias for EXCLUDE.
    pub const X: &str = "X";
    /// Primary name for the SHOW command.
    pub const SHOW: &str = "SHOW";
    /// Alias for SHOW.
    pub const INCLUDE: &str = "INCLUDE";
    /// Primary name for the RESET command.
    pub const RESET: &str = "RESET";

    /// All recognized names for EXCLUDE (primary + aliases).
    pub const EXCLUDE_NAMES: &[&str] = &[EXCLUDE, X];
    /// All recognized names for SHOW (primary + aliases).
    pub const SHOW_NAMES: &[&str] = &[SHOW, INCLUDE];
    /// All recognized names for RESET.
    pub const RESET_NAMES: &[&str] = &[RESET];

    /// Check if a command name is an EXCLUDE variant.
    pub fn is_exclude(name: &str) -> bool {
        let upper = name.to_uppercase();
        EXCLUDE_NAMES.iter().any(|&n| n == upper)
    }

    /// Check if a command name is a SHOW variant.
    pub fn is_show(name: &str) -> bool {
        let upper = name.to_uppercase();
        SHOW_NAMES.iter().any(|&n| n == upper)
    }

    /// Check if a command name is a RESET variant.
    pub fn is_reset(name: &str) -> bool {
        let upper = name.to_uppercase();
        RESET_NAMES.iter().any(|&n| n == upper)
    }
}

/// Recognized line command prefixes for the line-command parser.
///
/// The line-command parser should recognize these patterns:
/// - `X` — single line exclude
/// - `Xn` (where n is digits) — exclude n lines
/// - `XX` — block marker (paired)
pub mod line_command_patterns {
    /// Check if a prefix string is an X line command.
    /// Returns the variant if recognized.
    pub fn parse_x_prefix(prefix: &str) -> Option<XLineCommandKind> {
        let trimmed = prefix.trim();
        if trimmed.eq_ignore_ascii_case("XX") {
            return Some(XLineCommandKind::Block);
        }
        if trimmed.eq_ignore_ascii_case("X") {
            return Some(XLineCommandKind::Single);
        }
        // Check for Xn pattern (X followed by digits)
        if trimmed.len() > 1
            && trimmed.as_bytes()[0].eq_ignore_ascii_case(&b'X')
            && trimmed[1..].chars().all(|c| c.is_ascii_digit())
        {
            let count: usize = trimmed[1..].parse().ok()?;
            if count > 0 {
                return Some(XLineCommandKind::Count(count));
            }
        }
        None
    }

    /// The kind of X line command recognized from a prefix.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum XLineCommandKind {
        /// `X` — exclude a single line.
        Single,
        /// `Xn` — exclude n consecutive lines.
        Count(usize),
        /// `XX` — block marker for paired exclusion.
        Block,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 9 AC 1
    #[test]
    fn register_commands_succeeds_on_empty_registry() {
        let registry = CommandRegistry::new();
        let result = register_commands(&registry);
        assert!(result.is_ok());
    }

    // Validates: Requirement 9 AC 1
    #[test]
    fn exclude_command_registered_with_correct_metadata() {
        let registry = CommandRegistry::new();
        register_commands(&registry).unwrap();

        let meta = registry.metadata(&ids::exclude()).unwrap();
        assert_eq!(meta.display_name, "Exclude");
        assert_eq!(meta.category, "filter");
    }

    // Validates: Requirement 9 AC 2
    #[test]
    fn show_command_registered_with_correct_metadata() {
        let registry = CommandRegistry::new();
        register_commands(&registry).unwrap();

        let meta = registry.metadata(&ids::show()).unwrap();
        assert_eq!(meta.display_name, "Show");
        assert_eq!(meta.category, "filter");
    }

    // Validates: Requirement 9 AC 3
    #[test]
    fn reset_command_registered_with_correct_metadata() {
        let registry = CommandRegistry::new();
        register_commands(&registry).unwrap();

        let meta = registry.metadata(&ids::reset()).unwrap();
        assert_eq!(meta.display_name, "Reset");
        assert_eq!(meta.category, "filter");
    }

    // Validates: Requirement 9 AC 4
    #[test]
    fn line_command_registered() {
        let registry = CommandRegistry::new();
        register_commands(&registry).unwrap();

        assert!(registry.contains(&ids::line_x()));
    }

    // Validates: Requirement 9 AC 7
    #[test]
    fn all_commands_are_non_undoable() {
        let registry = CommandRegistry::new();
        register_commands(&registry).unwrap();

        assert_eq!(registry.is_undoable(&ids::exclude()), Some(false));
        assert_eq!(registry.is_undoable(&ids::show()), Some(false));
        assert_eq!(registry.is_undoable(&ids::reset()), Some(false));
        assert_eq!(registry.is_undoable(&ids::line_x()), Some(false));
    }

    // Validates: Requirement 9 AC 1 (alias recognition)
    #[test]
    fn alias_x_recognized_as_exclude() {
        assert!(aliases::is_exclude("X"));
        assert!(aliases::is_exclude("x"));
        assert!(aliases::is_exclude("EXCLUDE"));
        assert!(aliases::is_exclude("exclude"));
        assert!(!aliases::is_exclude("SHOW"));
    }

    // Validates: Requirement 9 AC 2 (alias recognition)
    #[test]
    fn alias_include_recognized_as_show() {
        assert!(aliases::is_show("INCLUDE"));
        assert!(aliases::is_show("include"));
        assert!(aliases::is_show("SHOW"));
        assert!(aliases::is_show("show"));
        assert!(!aliases::is_show("EXCLUDE"));
    }

    // Validates: Requirement 9 AC 3
    #[test]
    fn reset_name_recognized() {
        assert!(aliases::is_reset("RESET"));
        assert!(aliases::is_reset("reset"));
        assert!(!aliases::is_reset("SHOW"));
    }

    // Validates: Requirement 9 AC 4
    #[test]
    fn line_command_prefix_parsing() {
        use line_command_patterns::*;

        assert_eq!(parse_x_prefix("X"), Some(XLineCommandKind::Single));
        assert_eq!(parse_x_prefix("x"), Some(XLineCommandKind::Single));
        assert_eq!(parse_x_prefix("X5"), Some(XLineCommandKind::Count(5)));
        assert_eq!(parse_x_prefix("x10"), Some(XLineCommandKind::Count(10)));
        assert_eq!(parse_x_prefix("XX"), Some(XLineCommandKind::Block));
        assert_eq!(parse_x_prefix("xx"), Some(XLineCommandKind::Block));
        assert_eq!(parse_x_prefix("Y"), None);
        assert_eq!(parse_x_prefix("X0"), None); // zero count invalid
        assert_eq!(parse_x_prefix(""), None);
    }

    // Validates: Requirement 9 AC 6
    #[test]
    fn commands_enabled_in_all_modes() {
        let registry = CommandRegistry::new();
        register_commands(&registry).unwrap();

        // Commands should be enabled regardless of edit/browse mode
        let ctx = ff_command::ExecutionContext::empty();
        assert_eq!(registry.is_enabled(&ids::exclude(), &ctx), Some(true));
        assert_eq!(registry.is_enabled(&ids::show(), &ctx), Some(true));
        assert_eq!(registry.is_enabled(&ids::reset(), &ctx), Some(true));
        assert_eq!(registry.is_enabled(&ids::line_x(), &ctx), Some(true));
    }

    #[test]
    fn duplicate_registration_returns_error() {
        let registry = CommandRegistry::new();
        register_commands(&registry).unwrap();
        // Second registration should fail
        let result = register_commands(&registry);
        assert!(result.is_err());
    }
}
