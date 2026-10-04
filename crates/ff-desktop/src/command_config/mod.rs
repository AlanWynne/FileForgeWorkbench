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

use ff_command::{CommandTarget, TargetParams, TargetResolver, TargetValue};

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
/// `builtin_workspace_target` classifies the CustomWorkspace / navigation family
/// (B080, Step 2): the in-scope verbs FILES / =FILES, FILE CATALOGS / CATALOGS,
/// CONFIG [<ns>], COMMANDS, LOG, PLUGINS, MACROS, GSEARCH / SEARCH, KEYS [<kind>],
/// KINDS, MENUS, and bare THEME resolve to `CommandTarget::CustomWorkspace` so the
/// TYPED path reaches the SAME classification the key/menu path uses (one front
/// door, Req 2.1; ordered chain, Req 8.3). It returns `None` for every OTHER verb
/// so user menus / macros / registered ids still resolve at later stages and
/// non-migrated verbs fall through to the existing shell ladder, preserving
/// observable behaviour for every command string that resolves today
/// (command-framework Requirement 8.4, menu-workspace Requirement 10.2).
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

/// Classify a built-in CustomWorkspace / navigation verb (B080, Step 2) to its
/// `CommandTarget::CustomWorkspace`, or `None` when `input` is NOT one of the
/// in-scope verbs. A free function (not a method) so it is unit-testable without
/// a registry and keeps the `TargetResolver` impl thin.
///
/// The verb is matched case-insensitively; a trailing Argument_String keeps its
/// case except where the verb's shell method lowercases it (CONFIG namespace).
/// The `workspace_kind` strings are the EXPLICIT, stable Step-2 vocabulary that
/// the `dispatch_command_target` CustomWorkspace arm matches on and routes to the
/// SAME shell method the ladder arm calls (identical observable result, Req 8.4).
///
/// Returns `None` for every verb not in the table so non-migrated verbs fall
/// through to the ladder and user menus / macros still resolve at later stages
/// (no over-claim). THEME is the one verb where a non-empty argument means DO NOT
/// claim: bare `THEME` opens the Theme editor (claimed here), but `THEME <name>`
/// is a theme-apply action owned by the ladder (falls through).
///
/// Validates: command-framework Requirement 8.3 (stage 2), 8.4, 9.2, 9.7
pub(crate) fn builtin_workspace_target_for(input: &str) -> Option<CommandTarget> {
    let trimmed = input.trim();
    // The single Req 9.7 split: first whitespace-delimited token is the verb
    // (case-insensitive), the trimmed remainder is the case-preserved arg (B062).
    let (verb, arg) = trimmed
        .split_once(char::is_whitespace)
        .map(|(h, r)| (h, r.trim()))
        .unwrap_or((trimmed, ""));

    let custom = |workspace_kind: &str| {
        Some(CommandTarget::CustomWorkspace {
            workspace_kind: workspace_kind.to_string(),
            params: TargetParams::new(),
        })
    };

    // Case-insensitive verb comparison helper.
    let is = |name: &str| verb.eq_ignore_ascii_case(name);

    // =FILES is a single token (no whitespace), so it arrives as the verb.
    if is("=FILES") {
        return custom("file_explorer");
    }
    // FILE CATALOGS is the ONLY two-word in-scope verb.
    if is("FILE") && arg.eq_ignore_ascii_case("CATALOGS") {
        return custom("files");
    }
    // CONFIG [<namespace>]: claim with or without an arg; fold a non-empty
    // namespace (LOWERCASED, mirroring the ladder's `open_config_view`) into
    // params under `namespace`.
    if is("CONFIG") {
        if arg.is_empty() {
            return custom("config");
        }
        let mut params = TargetParams::new();
        params.insert(
            "namespace".to_string(),
            TargetValue::String(arg.to_lowercase()),
        );
        return Some(CommandTarget::CustomWorkspace {
            workspace_kind: "config".to_string(),
            params,
        });
    }
    // KEYS [<kind>]: claim with or without an arg; fold a non-empty kind
    // (case PRESERVED) into params under `kind`.
    if is("KEYS") {
        if arg.is_empty() {
            return custom("keys");
        }
        let mut params = TargetParams::new();
        params.insert("kind".to_string(), TargetValue::String(arg.to_string()));
        return Some(CommandTarget::CustomWorkspace {
            workspace_kind: "keys".to_string(),
            params,
        });
    }
    // THEME: bare only. A non-empty arg is the theme-apply action (ladder).
    if is("THEME") {
        if arg.is_empty() {
            return custom("theme_editor");
        }
        return None;
    }

    // The remaining verbs claim ONLY when the arg is EMPTY, mirroring the
    // ladder's exact `upper == "..."` match (so e.g. `COMMANDS foo` is NOT
    // claimed here and falls through to the ladder / later stages).
    if !arg.is_empty() {
        return None;
    }
    match () {
        _ if is("FILES") => custom("file_explorer"),
        _ if is("CATALOGS") => custom("files"),
        _ if is("COMMANDS") => custom("command_configurator"),
        _ if is("LOG") => custom("event_log"),
        _ if is("PLUGINS") => custom("plugin_manager"),
        _ if is("MACROS") => custom("macro_library"),
        _ if is("GSEARCH") || is("SEARCH") => custom("search"),
        _ if is("KINDS") => custom("kinds"),
        _ if is("MENUS") => custom("menus"),
        _ => None,
    }
}

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

    fn builtin_workspace_target(&self, input: &str) -> Option<CommandTarget> {
        builtin_workspace_target_for(input)
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

    // === B080 Step 2: builtin_workspace_target classifier ===================

    // Validates: command-framework Requirement 8.3 (stage 2) -- built-in
    // workspace verbs are classified by resolve_target (the None stub removed).
    #[test]
    fn builtin_workspace_target_classifies_nav_verb() {
        // FILES -> file_explorer CustomWorkspace, no params; case-insensitive.
        let expect_fe = Some(CommandTarget::CustomWorkspace {
            workspace_kind: "file_explorer".to_string(),
            params: TargetParams::new(),
        });
        assert_eq!(builtin_workspace_target_for("FILES"), expect_fe);
        assert_eq!(builtin_workspace_target_for("files"), expect_fe);

        // CONFIG <ns> folds a LOWERCASED namespace into params.
        let mut cfg_params = TargetParams::new();
        cfg_params.insert(
            "namespace".to_string(),
            TargetValue::String("core".to_string()),
        );
        assert_eq!(
            builtin_workspace_target_for("CONFIG Core"),
            Some(CommandTarget::CustomWorkspace {
                workspace_kind: "config".to_string(),
                params: cfg_params,
            }),
            "CONFIG <ns> lowercases and folds the namespace into params"
        );
        // Bare CONFIG claims with no params.
        assert_eq!(
            builtin_workspace_target_for("CONFIG"),
            Some(CommandTarget::CustomWorkspace {
                workspace_kind: "config".to_string(),
                params: TargetParams::new(),
            })
        );

        // KEYS <kind> preserves the arg case.
        let mut keys_params = TargetParams::new();
        keys_params.insert(
            "kind".to_string(),
            TargetValue::String("Editor".to_string()),
        );
        assert_eq!(
            builtin_workspace_target_for("KEYS Editor"),
            Some(CommandTarget::CustomWorkspace {
                workspace_kind: "keys".to_string(),
                params: keys_params,
            }),
            "KEYS <kind> preserves the argument case"
        );

        // =FILES single token and FILE CATALOGS two-word verb.
        assert_eq!(
            builtin_workspace_target_for("=FILES"),
            Some(CommandTarget::CustomWorkspace {
                workspace_kind: "file_explorer".to_string(),
                params: TargetParams::new(),
            })
        );
        assert_eq!(
            builtin_workspace_target_for("FILE CATALOGS"),
            Some(CommandTarget::CustomWorkspace {
                workspace_kind: "files".to_string(),
                params: TargetParams::new(),
            })
        );

        // THEME: bare claims the editor; THEME <name> is NOT claimed (ladder).
        assert_eq!(
            builtin_workspace_target_for("THEME"),
            Some(CommandTarget::CustomWorkspace {
                workspace_kind: "theme_editor".to_string(),
                params: TargetParams::new(),
            })
        );
        assert_eq!(
            builtin_workspace_target_for("THEME legacy"),
            None,
            "THEME <name> is a theme-apply action owned by the ladder, not claimed"
        );

        // Exact-match verbs do NOT claim when given a trailing arg (so e.g.
        // `COMMANDS foo` falls through, matching the ladder's exact match).
        assert_eq!(builtin_workspace_target_for("COMMANDS foo"), None);

        // No over-claim: an unknown verb is None.
        assert_eq!(builtin_workspace_target_for("ZXQWV"), None);
    }

    // Validates: command-framework Requirement 8.3 (stage 2) -- a full
    // ShellTargetResolver classifies FILES through resolve_target (proving the
    // former None stub no longer wins).
    #[test]
    fn shell_resolver_classifies_builtin_workspace_verb() {
        let defs: Vec<CommandDefinition> = vec![];
        let registry = ff_command::CommandRegistry::new();
        let dir = tempfile::TempDir::new().expect("tempdir");
        let resolver = ShellTargetResolver::new(&defs, &registry, dir.path().to_path_buf());
        assert_eq!(
            resolve_target("FILES", &resolver).unwrap(),
            CommandTarget::CustomWorkspace {
                workspace_kind: "file_explorer".to_string(),
                params: TargetParams::new(),
            }
        );
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
