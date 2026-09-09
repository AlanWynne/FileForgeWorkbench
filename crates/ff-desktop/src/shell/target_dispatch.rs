//! Command Target resolution and dispatch for menu options and shortcuts.
//!
//! Menu options (menu-workspace Requirement 10) and keyboard shortcuts
//! (command-framework Requirement 8.5, command-configurator Requirement 4.4)
//! resolve their command value to a [`ff_command::CommandTarget`] and execute
//! it. A command value equal to a user Command_Definition id resolves to that
//! definition's stored target (menu-workspace Requirement 10.3,
//! command-configurator Requirement 4.3); an inline `[options.target]` on a
//! menu option takes precedence (Requirement 10.6).
//!
//! Backward compatibility (Requirement 10.2, command-framework Requirement 8.4):
//! a bare command string that the resolver does not own (built-in workspace
//! verbs, editor pipeline verbs) is NOT diverted here; the caller falls through
//! to the existing `handle_command` pipeline unchanged.
//!
//! Scope note (DB.4, Option B): `Function` targets execute through the existing
//! command pipeline. `External` target execution via `ff-shell` (the desktop
//! adapter, command-configurator Requirement 3.3-3.8) and the standalone
//! `MENU <name>` command that fully executes a `Menu_Target`
//! (menu-workspace Requirement 11, 10.4) are tracked as separate Phase DB steps;
//! until they land, those variants report a deferred-status message rather than
//! acting, so a binding never fails silently.

use ff_command::CommandTarget;

use super::WorkbenchShell;
use crate::command_config::ShellTargetResolver;

/// Outcome of attempting to resolve a bare command string to a user-owned
/// Command_Target.
pub(super) enum ResolveOutcome {
    /// The string resolved to a target that was dispatched here.
    Dispatched,
    /// The string is not owned by the resolver; the caller should fall through
    /// to the existing command pipeline (Requirement 10.2).
    FallThrough,
}

impl WorkbenchShell {
    /// Resolve a bare command string against user definitions + the command
    /// registry, and dispatch the resulting target.
    ///
    /// Returns [`ResolveOutcome::FallThrough`] when the string is not a
    /// user-defined command id and not a registered Command_ID, so the caller
    /// preserves existing behaviour by handling the raw string itself.
    ///
    /// Validates: menu-workspace Requirement 10.1, 10.3; command-framework
    /// Requirement 8.3, 8.4
    pub(super) fn resolve_and_dispatch_command(&mut self, input: &str) -> ResolveOutcome {
        let resolver =
            ShellTargetResolver::new(&self.command_store.definitions, &self.cmd_registry);
        match ff_command::resolve_target(input, &resolver) {
            Ok(target) => {
                self.dispatch_command_target(&target);
                ResolveOutcome::Dispatched
            }
            // Not owned by the resolver (built-in verb, editor verb, unknown):
            // let the existing pipeline handle it (Requirement 10.2).
            Err(_) => ResolveOutcome::FallThrough,
        }
    }

    /// Dispatch a command string bound to a menu option or keyboard shortcut.
    ///
    /// Resolves the string against user definitions + the registry; a
    /// user-owned target is dispatched, otherwise the string is handled by the
    /// existing command pipeline unchanged (Requirement 10.2). This is the
    /// single entry point shared by the menu-option and shortcut seams
    /// (command-framework Requirement 8.5, command-configurator Requirement 4.4).
    pub(super) fn dispatch_bound_command(&mut self, command: &str) {
        match self.resolve_and_dispatch_command(command) {
            ResolveOutcome::Dispatched => {}
            ResolveOutcome::FallThrough => self.handle_command(command),
        }
    }

    /// Run a user Command_Definition by its id.
    ///
    /// This is the explicit definition-reference entry point used when a
    /// binding (menu option or keyboard shortcut) is known to target a
    /// definition id rather than an arbitrary command string. When no such
    /// definition exists, it reports `Command '<id>' is not defined.`
    ///
    /// Validates: command-configurator Requirement 4.4, 4.5
    // Explicit definition-reference entry point. Exercised by unit tests now;
    // its runtime caller is the binding-config surface (a Menu_Option or
    // Shortcut_Binding that explicitly names a definition id), which lands with
    // the Command Configurator UI / binding config (a later Phase DB step).
    #[allow(dead_code)]
    pub(super) fn run_command_definition(&mut self, id: &str) {
        let target = self
            .command_store
            .definitions
            .iter()
            .find(|d| d.id == id.trim())
            .map(|d| d.target.clone());
        match target {
            Some(t) => self.dispatch_command_target(&t),
            None => {
                self.open_error = Some(format!("Command '{}' is not defined.", id.trim()));
            }
        }
    }

    /// Execute a resolved [`CommandTarget`].
    ///
    /// `Function` targets run through the existing `handle_command` pipeline so
    /// the observable result is identical to typing the command
    /// (Requirement 10.2). The visible-workspace variants (Menu,
    /// CustomWorkspace, External) and Macro are routed by later Phase DB steps;
    /// until then they set a deferred-status message.
    ///
    /// Validates: command-framework Requirement 8.2; menu-workspace
    /// Requirement 10.1
    pub(super) fn dispatch_command_target(&mut self, target: &CommandTarget) {
        // Route by variant. `Function` targets go through the existing command
        // pipeline for an identical observable result (Requirement 10.2); the
        // remaining variants are routed by later Phase DB steps.
        match target {
            CommandTarget::Function { command_id, .. } => {
                // Identical observable result to typing the command.
                self.handle_command(command_id);
            }
            CommandTarget::Menu { name } => {
                // A Menu_Target opens the referenced Menu_Workspace via the same
                // path as the MENU command (menu-workspace Requirement 10.4, 11.5).
                self.open_menu_by_name(name);
            }
            CommandTarget::CustomWorkspace { workspace_kind, .. } => {
                self.open_error = Some(format!(
                    "Custom workspace target '{workspace_kind}' is not yet runnable via a \
                     command binding."
                ));
            }
            CommandTarget::Macro { .. } => {
                self.open_error =
                    Some("Macro targets are not yet runnable via a command binding.".to_string());
            }
            CommandTarget::External {
                program,
                args,
                working_dir,
                mode,
            } => {
                // Run via the ff-shell adapter: placeholder expansion + shell.mode
                // gate; prompt mode stages a confirmation dialog.
                // Validates: command-configurator Requirement 3.2-3.9.
                self.run_external_target(program, args, working_dir.as_deref(), *mode);
            }
        }
    }
}
