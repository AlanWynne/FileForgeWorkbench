//! The shared Command_Environment CONTRACT (CR-CH-053): the
//! [`CommandEnvironment`] trait and the [`EnvDispatchOutcome`] enum.
//!
//! Split out of `environment.rs` purely to keep each file under the 400-line
//! rule (rust-standards.md) once CR-NR-109 grew the `AliasTable` there. These
//! items are UNCHANGED; `environment.rs` re-exports them (`pub(crate) use
//! environment_contract::{CommandEnvironment, EnvDispatchOutcome}`) so every
//! existing `super::environment::CommandEnvironment` /
//! `crate::shell::environment::EnvDispatchOutcome` path keeps working with no
//! call-site change.
//!
//! See docs/specs/command-environments/{requirements,design}.md.

/// The contract every first-class Command_Environment implements: try to CLAIM a
/// raw command string for this environment's vocabulary (after resolving its
/// surface verb to the canonical verb via the alias table). Returns `true` when
/// claimed+handled; `false` to fall through to the next front-door step.
///
/// The first implementor is [`super::environment_registry::FfEditEnvironment`]
/// (CR-CH-053 Task 17). Because FFEDIT's verbs mutate shell-entangled state
/// (`self.tabs` + the managers), `claim` TAKES `&mut WorkbenchShell` -- the
/// design-sanctioned "thin object that still reaches shell state" (design.md
/// "FFEDIT-as-object"). This is indirection, NOT a behaviour change.
///
/// Validates: command-environments Requirement 1.1, 6.2a, 13.5
//
// core-001 strip: the only implementor (the FFEDIT editor environment) is parked
// with the editor domain, so this contract has no live implementor or caller in
// the core build. It is RETAINED (not deleted) as the framework seam the editor
// plugin re-implements on re-entry; `allow(dead_code)` keeps the dormant contract
// without a warning.
#[allow(dead_code)]
pub(crate) trait CommandEnvironment {
    /// Attempt to claim and execute `raw` against `shell`. `upper` is the
    /// pre-uppercased trimmed line to avoid recomputing it. Returns `true` iff
    /// claimed+handled.
    fn claim(&mut self, shell: &mut super::WorkbenchShell, raw: &str, upper: &str) -> bool;
}

/// Outcome of addressing a command to a NAMED environment via
/// `WorkbenchShell::dispatch_to_environment` (CR-CH-053 Task 18, Req 14.1/14.2).
///
/// Carries a return code so the SAME seam serves both FFEDIT's internal store-verb
/// forwarding and the deferred macro `ADDRESS <env>` (Req 8.3 / Task 10) -- the
/// macro bridge will map `rc` to the macro `RC`. This is NOT a second dispatcher
/// (Req 14.8): `dispatch_to_environment` resolves a name to a registered
/// environment and invokes its existing `claim`, exactly as the active-env gate
/// does; it adds no parallel dispatch path and does not touch `CommandTarget` or
/// the Navigation_Stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvDispatchOutcome {
    /// The named environment claimed and handled the command. `rc` is 0 for a
    /// successful claim (the macro-RC convention; richer codes arrive with the
    /// macro ADDRESS slice, Task 10).
    //
    // core-001 strip: no registered environment CLAIMS at the active-env step now
    // (FFEDIT is parked, the FFCMD base declines), so this variant is not
    // constructed in the core build. Retained for the editor plugin's re-entry.
    #[allow(dead_code)]
    Claimed { rc: i32 },
    /// The named environment is registered but DECLINED the command (it is not
    /// one of that environment's verbs); the caller falls through as if the
    /// environment had not been addressed.
    NotClaimed,
    /// No environment is registered under the given name (degrade-to-base is the
    /// caller's decision, mirroring Req 13.4 for the active-env path).
    NoSuchEnvironment,
}
