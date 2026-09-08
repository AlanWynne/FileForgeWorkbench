//! Command Configurator -- user-authored command definitions.
//!
//! A `CommandDefinition` pairs a Command_Id with a `CommandTarget`
//! (command-framework Requirement 8) and display metadata. Definitions are
//! stored in a data-driven, hot-reloadable TOML file at
//! `<User_Data_Dir>/commands/commands.toml` (the `CommandStore`).
//!
//! This module owns the data layer (types, load/save/validation, hot-reload)
//! and a `TargetResolver` view so bare command strings that name a definition
//! resolve to that definition's stored `CommandTarget` (command-framework
//! Requirement 8.3; command-configurator Requirement 4.3, 4.5).
//!
//! The Command Configurator Context (the editing UI, Requirement 2) and
//! external execution (Requirement 3) are wired in later phases; this module
//! is the foundation both build on.
//!
//! Validates: command-configurator Requirement 1, 4.
//
// This module is the data + resolution foundation for the Command Configurator.
// It is fully implemented and unit-tested, but its consumers -- the Context UI
// (Requirement 2) and external execution (Requirement 3) -- land in later
// Phase DB steps (DB.9 UI, DB.10). Until a runtime path constructs and reads a
// CommandStore, these items are unused in the compiled binary; the crate-level
// allow keeps the tested foundation in the tree without warnings, mirroring the
// menu_workspace module during its spec-complete-but-unwired phase.
#![allow(dead_code)]

pub mod store;

use serde::{Deserialize, Serialize};

use ff_command::{CommandTarget, TargetResolver};

/// One user-authored command definition.
///
/// Validates: command-configurator Requirement 1.2, 1.3.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandDefinition {
    /// Unique command id (Command_ID naming rule).
    pub id: String,
    /// Human-readable label shown in the configurator and menus.
    pub label: String,
    /// Optional one-line description.
    #[serde(default)]
    pub description: Option<String>,
    /// Grouping category; defaults to `"user"`.
    #[serde(default = "default_category")]
    pub category: String,
    /// The action this definition performs, serialised as `[command.target]`.
    pub target: CommandTarget,
}

fn default_category() -> String {
    "user".to_string()
}

/// A `TargetResolver` view over a slice of definitions.
///
/// Used to feed `ff_command::resolve_target` the user-command lookup
/// (Requirement 4.3): a bare string equal to a definition `id` resolves to that
/// definition's stored `CommandTarget`. The built-in-verb and registered-id
/// lookups are supplied by the caller and composed at the shell layer.
///
/// This view intentionally answers only `user_command_target`; the other two
/// `TargetResolver` methods return "no match" so a composing resolver can chain
/// this ahead of the shell's built-in and registry lookups.
pub struct UserCommandStore<'a> {
    definitions: &'a [CommandDefinition],
}

impl<'a> UserCommandStore<'a> {
    /// Create a resolver view over the given definitions.
    pub fn new(definitions: &'a [CommandDefinition]) -> Self {
        Self { definitions }
    }

    /// Look up a definition by exact id.
    pub fn find(&self, id: &str) -> Option<&CommandDefinition> {
        self.definitions.iter().find(|d| d.id == id)
    }
}

impl TargetResolver for UserCommandStore<'_> {
    fn user_command_target(&self, input: &str) -> Option<CommandTarget> {
        self.find(input).map(|d| d.target.clone())
    }

    fn builtin_workspace_target(&self, _input: &str) -> Option<CommandTarget> {
        None
    }

    fn is_registered_command(&self, _input: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_command::{resolve_target, ExternalMode};

    fn ext_def(id: &str) -> CommandDefinition {
        CommandDefinition {
            id: id.to_string(),
            label: "Build".to_string(),
            description: None,
            category: "user".to_string(),
            target: CommandTarget::External {
                program: "pwsh".to_string(),
                args: vec!["-File".to_string(), "build.ps1".to_string()],
                working_dir: None,
                mode: ExternalMode::Captured,
            },
        }
    }

    // Validates: Requirement 4.3 -- a definition id resolves to its target.
    #[test]
    fn user_command_id_resolves_to_stored_target() {
        let defs = vec![ext_def("build.release")];
        let view = UserCommandStore::new(&defs);
        let resolved = resolve_target("build.release", &view).expect("resolves");
        assert_eq!(resolved, defs[0].target);
    }

    // Validates: Requirement 4.3 -- resolution tolerates surrounding whitespace.
    #[test]
    fn user_command_resolution_trims_input() {
        let defs = vec![ext_def("build.release")];
        let view = UserCommandStore::new(&defs);
        assert!(resolve_target("  build.release  ", &view).is_ok());
    }

    // Validates: Requirement 4.5 -- an unknown id does not resolve here.
    #[test]
    fn unknown_id_does_not_resolve_via_user_store() {
        let defs = vec![ext_def("build.release")];
        let view = UserCommandStore::new(&defs);
        // The user-store view alone has no built-ins/registry, so an unknown
        // id is unresolved (the shell chains further resolvers after this).
        assert!(resolve_target("no.such.command", &view).is_err());
    }

    // Validates: Requirement 1.3 -- category defaults to "user" when omitted.
    #[test]
    fn category_defaults_to_user() {
        let toml = r#"
id = "a.b"
label = "AB"
[target]
kind = "function"
command_id = "file.save"
"#;
        let def: CommandDefinition = toml::from_str(toml).expect("parse");
        assert_eq!(def.category, "user");
        assert!(def.description.is_none());
    }
}
