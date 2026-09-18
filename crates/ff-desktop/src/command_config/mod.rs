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

pub mod edit;
pub mod render;
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

/// A shell-side [`TargetResolver`] composing user-defined command definitions
/// with the workbench command registry.
///
/// Resolution answers, in `resolve_target` order:
/// 1. `user_command_target` -- a definition whose id equals the input
///    (command-configurator Requirement 4.3, menu-workspace Requirement 10.3).
/// 3. `is_registered_command` -- a registered `Command_ID`, so a bare command
///    id resolves to a `Function` target (command-framework Requirement 8.3).
///
/// `builtin_workspace_target` deliberately returns `None`: built-in workspace
/// verbs and fastpaths (FILES, =2, SETTINGS, ...) keep their existing shell
/// handling via fall-through, preserving observable behaviour for every command
/// string that resolves today (command-framework Requirement 8.4,
/// menu-workspace Requirement 10.2).
///
/// CR-CH-025: `menu_name_target` resolves a bare token to a `Menu` target when
/// it names a resolvable menu (a user `menus/<name>.toml` that exists, or a
/// compiled built-in menu name `pom`/`settings`); `macro_name_target` is a
/// DEFERRED stub returning `None` until macro execution is wired.
pub struct ShellTargetResolver<'a> {
    definitions: &'a [CommandDefinition],
    registry: &'a ff_command::CommandRegistry,
    /// Directory scanned for `menus/<name>.toml` user menus (CR-CH-025 stage 3).
    menus_dir: std::path::PathBuf,
}

/// Compiled built-in menu names that always resolve (no file required).
/// Mirrors the code-only Recovery_Baseline menus (menu-workspace Req 12).
const BUILTIN_MENU_NAMES: &[&str] = &["pom", "settings"];

impl<'a> ShellTargetResolver<'a> {
    /// Create a resolver over the given definitions, command registry, and the
    /// menus directory used for menu-name resolution (CR-CH-025).
    pub fn new(
        definitions: &'a [CommandDefinition],
        registry: &'a ff_command::CommandRegistry,
        menus_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            definitions,
            registry,
            menus_dir,
        }
    }

    /// Look up a definition by exact id.
    pub fn find(&self, id: &str) -> Option<&CommandDefinition> {
        self.definitions.iter().find(|d| d.id == id)
    }

    /// True when `name` (case-insensitive) is a resolvable menu: a compiled
    /// built-in (`pom`/`settings`) or a user `menus/<name>.toml` on disk.
    fn is_resolvable_menu(&self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        if BUILTIN_MENU_NAMES.contains(&lower.as_str()) {
            return true;
        }
        // A user menu file. Reuse the slugging rule used elsewhere is overkill
        // here -- menu names are already file-stem tokens; probe `<name>.toml`.
        self.menus_dir.join(format!("{lower}.toml")).exists()
    }
}

impl TargetResolver for ShellTargetResolver<'_> {
    fn user_command_target(&self, input: &str) -> Option<CommandTarget> {
        self.find(input).map(|d| d.target.clone())
    }

    fn builtin_workspace_target(&self, _input: &str) -> Option<CommandTarget> {
        None
    }

    fn is_registered_command(&self, input: &str) -> bool {
        match ff_command::CommandId::new(input.to_string()) {
            Some(id) => self.registry.contains(&id),
            None => false,
        }
    }

    /// Resolve the FIRST token to a `Menu` target when it names a resolvable
    /// menu. The trailing token (if any) is applied as an Option_Key by the
    /// shell's chaining helper, so only the first token is inspected here.
    ///
    /// Validates: command-framework Requirement 8.11 (CR-CH-025)
    fn menu_name_target(&self, input: &str) -> Option<CommandTarget> {
        let first = input.split_whitespace().next().unwrap_or("");
        if first.is_empty() || !self.is_resolvable_menu(first) {
            return None;
        }
        Some(CommandTarget::Menu {
            name: first.to_ascii_lowercase(),
        })
    }

    /// DEFERRED (CR-CH-025, command-framework Requirement 8.12): the macro stage
    /// is part of the specified chain order but matches nothing until the shell
    /// gains Lua/macro execution (ff-desktop does not yet depend on the macro
    /// engine). When macro execution is wired, this will look the FIRST token up
    /// in the Macro_Library and return `CommandTarget::Macro`.
    fn macro_name_target(&self, _input: &str) -> Option<CommandTarget> {
        None
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

    // === CR-CH-025: ShellTargetResolver menu-name + macro stages ============

    // Validates: command-framework Requirement 8.11 -- a compiled built-in menu
    // name resolves to a Menu target without any file on disk.
    #[test]
    fn shell_resolver_builtin_menu_name_resolves_without_file() {
        let defs: Vec<CommandDefinition> = vec![];
        let registry = ff_command::CommandRegistry::new();
        let dir = tempfile::TempDir::new().expect("tempdir");
        let resolver = ShellTargetResolver::new(&defs, &registry, dir.path().to_path_buf());
        assert_eq!(
            resolve_target("settings", &resolver).unwrap(),
            CommandTarget::Menu {
                name: "settings".to_string()
            }
        );
        // Case-insensitive; POM is also a built-in.
        assert_eq!(
            resolve_target("POM", &resolver).unwrap(),
            CommandTarget::Menu {
                name: "pom".to_string()
            }
        );
    }

    // Validates: command-framework Requirement 8.11 -- a user menu file resolves.
    #[test]
    fn shell_resolver_user_menu_file_resolves() {
        let defs: Vec<CommandDefinition> = vec![];
        let registry = ff_command::CommandRegistry::new();
        let dir = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(dir.path().join("reports.toml"), "title = \"Reports\"\n").expect("write");
        let resolver = ShellTargetResolver::new(&defs, &registry, dir.path().to_path_buf());
        assert_eq!(
            resolve_target("reports", &resolver).unwrap(),
            CommandTarget::Menu {
                name: "reports".to_string()
            }
        );
        // Resolution inspects the FIRST token only (trailing key chains later).
        assert_eq!(
            resolve_target("reports x", &resolver).unwrap(),
            CommandTarget::Menu {
                name: "reports".to_string()
            }
        );
    }

    // Validates: command-framework Requirement 8.11 -- an unknown token with no
    // menu file and no built-in match does not resolve as a menu.
    #[test]
    fn shell_resolver_unknown_menu_name_does_not_resolve() {
        let defs: Vec<CommandDefinition> = vec![];
        let registry = ff_command::CommandRegistry::new();
        let dir = tempfile::TempDir::new().expect("tempdir");
        let resolver = ShellTargetResolver::new(&defs, &registry, dir.path().to_path_buf());
        assert!(resolve_target("nosuchmenu", &resolver).is_err());
    }

    // Validates: command-framework Requirement 8.12 -- the macro stage is
    // deferred: it never resolves, so a macro-named token is unresolved for now.
    #[test]
    fn shell_resolver_macro_stage_is_deferred_none() {
        let defs: Vec<CommandDefinition> = vec![];
        let registry = ff_command::CommandRegistry::new();
        let dir = tempfile::TempDir::new().expect("tempdir");
        let resolver = ShellTargetResolver::new(&defs, &registry, dir.path().to_path_buf());
        assert_eq!(resolver.macro_name_target("format"), None);
        assert!(resolve_target("format", &resolver).is_err());
    }
}
