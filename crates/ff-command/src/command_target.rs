//! `CommandTarget` -- the unified description of what a command does.
//!
//! A `CommandTarget` is one of five variants: open a menu, open a built-in
//! custom workspace, invoke an internal function, run a macro, or run an
//! external program. Menu options and keyboard bindings both resolve to a
//! `CommandTarget`, giving one dispatchable representation for every action.
//!
//! This module provides the type, a pure `resolve_target` resolver, an
//! `execute_target` router, TOML serialisation, and the visible-workspace
//! classification. It has no dependency on the desktop or session layers:
//! resolution of built-in workspace verbs and user-defined command ids is
//! delegated to the caller via the [`TargetResolver`] trait, and execution of
//! the visible-workspace variants is delegated to the shell layer via the
//! [`TargetExecutor`] trait.
//!
//! Validates: command-framework Requirement 8.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::CommandError;
use crate::id::CommandId;

// === TargetValue / TargetParams ============================================

/// A single value inside a target parameter map.
///
/// A serde-friendly, deterministically-ordered value model used by
/// `CustomWorkspace` and `Function` targets so that a `CommandTarget`
/// round-trips through TOML (Requirement 8.7). Distinct from
/// `crate::params::ParamValue`, which is the runtime dispatch value model and
/// is intentionally not serialisable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TargetValue {
    /// A boolean value.
    Boolean(bool),
    /// A 64-bit signed integer value.
    Integer(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A string value.
    String(String),
}

impl From<bool> for TargetValue {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<i64> for TargetValue {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<f64> for TargetValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<&str> for TargetValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<String> for TargetValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

/// A small, ordered, serialisable parameter bag carried by `CustomWorkspace`
/// and `Function` targets. `BTreeMap` gives deterministic ordering for stable
/// TOML output and reproducible tests.
pub type TargetParams = BTreeMap<String, TargetValue>;

// === MacroSource / ExternalMode ============================================

/// How a macro target names the macro to run.
///
/// Validates: Requirement 8.1 (Macro_Target).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroSource {
    /// A macro discovered by name in the configured macro directories.
    Name(String),
    /// A macro at an explicit absolute or workspace-relative path.
    Path(String),
}

/// Execution mode for an external process.
///
/// Validates: Requirement 8.1 (External_Target); command-configurator Req 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalMode {
    /// Fire-and-forget Started Task: no capture, no workspace, never persisted.
    Detached,
    /// Async run with stdout/stderr/exit shown in the Output Panel.
    Captured,
}

// === CommandTarget =========================================================

/// A typed description of the action a command performs.
///
/// Exactly one of five variants. Serialises to/from TOML with an internal
/// `kind` tag so a target reads naturally in a data file (Requirement 8.7).
///
/// Validates: Requirement 8.1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CommandTarget {
    /// Open the Menu Workspace backed by `menus/<name>.toml`.
    Menu {
        /// The menu name (without extension); `pom` is the Home Context.
        name: String,
    },
    /// Open a built-in Context with optional typed parameters.
    CustomWorkspace {
        /// The built-in Context kind (e.g. `editor`, `files`, `settings`).
        workspace_kind: String,
        /// Optional typed parameters (e.g. a Settings namespace filter).
        #[serde(default)]
        params: TargetParams,
    },
    /// Invoke a registered internal command through the dispatcher.
    Function {
        /// The registered Command_ID to invoke.
        command_id: String,
        /// Optional parameters passed to the command.
        #[serde(default)]
        params: TargetParams,
    },
    /// Run a Lua/REXX macro by name or path.
    Macro {
        /// How the macro is named.
        source: MacroSource,
    },
    /// Run an external process.
    External {
        /// The program to launch.
        program: String,
        /// Argument list.
        #[serde(default)]
        args: Vec<String>,
        /// Optional working directory.
        #[serde(default)]
        working_dir: Option<String>,
        /// Detached (fire-and-forget) or Captured (async with output).
        mode: ExternalMode,
    },
}

impl CommandTarget {
    /// Returns true if executing this target produces a user-visible Workspace.
    ///
    /// Menu, CustomWorkspace, and External in Captured mode are visible
    /// Workspaces; Function, Macro, and External in Detached mode are not.
    /// Session persistence uses this to decide which Workspaces to persist.
    ///
    /// Validates: Requirement 8.9.
    pub fn produces_visible_workspace(&self) -> bool {
        matches!(
            self,
            CommandTarget::Menu { .. }
                | CommandTarget::CustomWorkspace { .. }
                | CommandTarget::External {
                    mode: ExternalMode::Captured,
                    ..
                }
        )
    }

    /// A short, stable label for the variant, useful in diagnostics.
    pub fn variant_name(&self) -> &'static str {
        match self {
            CommandTarget::Menu { .. } => "menu",
            CommandTarget::CustomWorkspace { .. } => "custom_workspace",
            CommandTarget::Function { .. } => "function",
            CommandTarget::Macro { .. } => "macro",
            CommandTarget::External { .. } => "external",
        }
    }
}

// === Target resolution =====================================================

/// Error returned when a bare string cannot be resolved to a `CommandTarget`.
///
/// Validates: Requirement 8.8.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("[command] resolve: '{input}' could not be resolved to a command target")]
pub struct TargetResolveError {
    /// The input string that failed to resolve.
    pub input: String,
}

/// Environment consulted by [`resolve_target`] to interpret bare command
/// strings without pulling desktop/config dependencies into this crate.
///
/// Implementors provide the lookups that live in higher layers:
/// user-defined command definitions (command-configurator) and built-in
/// workspace verbs / fastpaths (the desktop shell).
pub trait TargetResolver {
    /// Return the stored `CommandTarget` for a user-defined command whose id
    /// equals `input` (trimmed), or `None` if there is no such definition.
    fn user_command_target(&self, input: &str) -> Option<CommandTarget>;

    /// Return a `CustomWorkspace` (or `Menu`) target for a built-in workspace
    /// verb or fastpath (e.g. `FILES`, `=2`, `SETTINGS editor`), or `None`.
    fn builtin_workspace_target(&self, input: &str) -> Option<CommandTarget>;

    /// Return true if `input` (trimmed) is a registered Command_ID.
    fn is_registered_command(&self, input: &str) -> bool;
}

/// Convert a bare command string into a [`CommandTarget`].
///
/// Resolution order (first match wins), chosen to preserve existing behaviour:
/// 1. A user-defined command definition whose id equals the trimmed input.
/// 2. A built-in workspace verb / fastpath.
/// 3. A registered Command_ID -> a `Function` target.
/// 4. Otherwise -> `Err(TargetResolveError)`.
///
/// Callers whose input string is not resolvable here (e.g. editor pipeline
/// verbs like `LOCATE`) are expected to fall through to the existing command
/// pipeline; this resolver only classifies the target variants it owns.
///
/// Validates: Requirement 8.3, 8.4, 8.8.
pub fn resolve_target<R: TargetResolver + ?Sized>(
    input: &str,
    resolver: &R,
) -> Result<CommandTarget, TargetResolveError> {
    let trimmed = input.trim();
    if let Some(t) = resolver.user_command_target(trimmed) {
        return Ok(t);
    }
    if let Some(t) = resolver.builtin_workspace_target(trimmed) {
        return Ok(t);
    }
    if resolver.is_registered_command(trimmed) {
        // A bare Command_ID is the Function variant of a target.
        return Ok(CommandTarget::Function {
            command_id: trimmed.to_string(),
            params: TargetParams::new(),
        });
    }
    Err(TargetResolveError {
        input: trimmed.to_string(),
    })
}

// === Target execution ======================================================

/// The outcome of routing a `CommandTarget` through [`execute_target`].
///
/// The `Function` variant is executed in-crate via the provided dispatch
/// callback. The visible-workspace variants (Menu, CustomWorkspace, External)
/// and Macro are handled by higher layers (the desktop shell / shell engine);
/// `execute_target` classifies and hands them back as `Deferred` so the caller
/// can perform the layer-specific action. This keeps `ff-command` free of
/// desktop and process-spawning dependencies.
///
/// Validates: Requirement 8.2.
#[derive(Debug)]
pub enum TargetExecution {
    /// A `Function` target was dispatched; carries the dispatcher's result.
    Function(crate::result::CommandResult),
    /// A non-function target that the shell layer must carry out.
    Deferred(CommandTarget),
    /// The target could not be handled (e.g. an invalid Function id).
    Failed(CommandError),
}

/// Route a `CommandTarget` to its handling path.
///
/// `Function` targets are dispatched immediately via `dispatch_fn`, which the
/// caller supplies (typically wrapping `CommandDispatch::execute_command`).
/// All other variants are returned as `TargetExecution::Deferred` for the shell
/// layer to open the corresponding Workspace or run the external process.
///
/// Validates: Requirement 8.2.
pub fn execute_target<F>(target: &CommandTarget, mut dispatch_fn: F) -> TargetExecution
where
    F: FnMut(&CommandId, &TargetParams) -> crate::result::CommandResult,
{
    match target {
        CommandTarget::Function { command_id, params } => {
            match CommandId::new(command_id.clone()) {
                Some(id) => TargetExecution::Function(dispatch_fn(&id, params)),
                None => TargetExecution::Failed(CommandError::InvalidId {
                    id: command_id.clone(),
                    reason: "not a valid command id".to_string(),
                }),
            }
        }
        other => TargetExecution::Deferred(other.clone()),
    }
}

// === TOML serialisation helpers ============================================

/// Serialise a `CommandTarget` to a TOML string (Requirement 8.7).
///
/// # Errors
/// Returns an error if the target cannot be represented as TOML.
pub fn target_to_toml(target: &CommandTarget) -> Result<String, toml::ser::Error> {
    toml::to_string(target)
}

/// Deserialise a `CommandTarget` from a TOML string (Requirement 8.7).
///
/// # Errors
/// Returns an error if the TOML is malformed or does not match the schema.
pub fn target_from_toml(s: &str) -> Result<CommandTarget, toml::de::Error> {
    toml::from_str(s)
}
