//! # Command_Environment Addressing
//!
//! The `dispatch_to_environment` method extracted VERBATIM from `commands.rs`
//! as part of the behaviour-preserving file-size split (pure code movement --
//! no logic edits). It addresses a raw command to a NAMED Command_Environment
//! (the REXX ADDRESS pattern applied internally).

use super::environment::EnvDispatchOutcome;
use super::environment_registry::RegisteredEnv;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Address a raw command to a NAMED Command_Environment (CR-CH-053 Task 18,
    /// Req 14.1/14.2): the REXX ADDRESS pattern applied INTERNALLY. Resolves the
    /// name through the built `EnvironmentRegistry` and invokes that
    /// environment's `claim` against the shell -- the SAME `claim` seam the
    /// active-env gate uses, so there is ONE invocation path, not a second
    /// dispatcher (Req 14.8). It touches neither `CommandTarget` nor the per-tab
    /// Navigation_Stack.
    ///
    /// Addressing the environment that is ALREADY active is therefore identical
    /// to not addressing it (Req 14.1): both reach the same `claim`. This single
    /// entry point is also what the deferred macro `ADDRESS <env>` (Req 8 /
    /// Task 10) will call, so FFEDIT's internal store-verb forwarding and macro
    /// addressing share one seam (Req 14.2).
    ///
    /// Returns an [`EnvDispatchOutcome`] carrying a return code (Req 14.2): the
    /// FFCMD base and the phase-1 host-FS placeholder do not CLAIM at this step
    /// (they resolve through the ordinary ladder / are non-claiming), so
    /// addressing them yields `NotClaimed`.
    ///
    /// Validates: command-environments Requirement 14.1, 14.2, 14.8
    pub(super) fn dispatch_to_environment(&mut self, name: &str, _raw: &str) -> EnvDispatchOutcome {
        let Some(env) = self.environments.resolve(name) else {
            return EnvDispatchOutcome::NoSuchEnvironment;
        };
        match env {
            // core-001 strip: the only registered environment is the FFCMD base.
            // FFEDIT (the editor verb-claiming environment) and the host-FS /
            // mainframe SAVE backend seam are parked with the editor domain (see
            // docs/reentry-register.md), so there is no environment that claims
            // at the active-env step. The FFCMD base is reached through the
            // ordinary ladder, not claimed here, so addressing it declines.
            RegisteredEnv::FfCmdBase => EnvDispatchOutcome::NotClaimed,
        }
    }
}
