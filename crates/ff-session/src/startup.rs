//! Startup sequence orchestration — the 10-phase ordered flow from process
//! launch to first interactive UI frame.
//!
//! Addresses: Requirement 1 (Startup Sequence Ordering)

use std::time::{Duration, Instant};

/// The ordered startup phases executed from process launch to interactive UI.
///
/// Phases 1–7 complete before the first UI frame (Phase 8).
/// Phases 9–10 execute after the first frame is rendered.
///
/// Addresses: Requirement 1 AC 1.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum StartupPhase {
    /// Phase 1: Parse command-line arguments.
    ParseCliArguments = 1,
    /// Phase 2: Load configuration via configuration-system.
    LoadConfiguration = 2,
    /// Phase 3: Initialise the logging subsystem.
    InitialiseLogging = 3,
    /// Phase 4: Initialise User_Data_Dir (create if absent).
    InitialiseUserDataDir = 4,
    /// Phase 5: Load and activate plugins.
    LoadPlugins = 5,
    /// Phase 6: Load Session_State from Session_File.
    LoadSessionState = 6,
    /// Phase 7: Restore Layout_State and Window_Geometry.
    RestoreLayout = 7,
    /// Phase 8: Render first interactive UI frame.
    RenderFirstFrame = 8,
    /// Phase 9: Open files (CLI args, session restore, or empty state).
    OpenFiles = 9,
    /// Phase 10: Check for crash recovery.
    CrashRecovery = 10,
}

impl StartupPhase {
    /// All phases in execution order.
    pub const ALL: [StartupPhase; 10] = [
        Self::ParseCliArguments,
        Self::LoadConfiguration,
        Self::InitialiseLogging,
        Self::InitialiseUserDataDir,
        Self::LoadPlugins,
        Self::LoadSessionState,
        Self::RestoreLayout,
        Self::RenderFirstFrame,
        Self::OpenFiles,
        Self::CrashRecovery,
    ];

    /// Phases that must complete before the first UI frame.
    pub const PRE_RENDER: [StartupPhase; 7] = [
        Self::ParseCliArguments,
        Self::LoadConfiguration,
        Self::InitialiseLogging,
        Self::InitialiseUserDataDir,
        Self::LoadPlugins,
        Self::LoadSessionState,
        Self::RestoreLayout,
    ];

    /// Phases that execute after the first UI frame.
    pub const POST_RENDER: [StartupPhase; 2] = [Self::OpenFiles, Self::CrashRecovery];

    /// The numeric phase number (1-based).
    pub fn number(self) -> u8 {
        self as u8
    }

    /// Whether this phase is pre-render (must complete before Phase 8).
    pub fn is_pre_render(self) -> bool {
        (self as u8) < 8
    }

    /// Whether this phase is post-render (executes after Phase 8).
    pub fn is_post_render(self) -> bool {
        (self as u8) > 8
    }
}

/// Outcome of a single startup phase.
///
/// Addresses: Requirement 1 AC 1.4, 1.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhaseOutcome {
    /// Phase completed successfully.
    Success,
    /// Phase failed non-fatally; workbench continues in degraded mode.
    Degraded {
        /// Description of what went wrong.
        reason: String,
    },
    /// Phase was skipped (e.g., session restore disabled by config).
    Skipped {
        /// Description of why it was skipped.
        reason: String,
    },
    /// Phase failed fatally; startup must abort.
    Fatal {
        /// Description of the fatal error.
        reason: String,
    },
}

impl PhaseOutcome {
    /// Whether this outcome represents a successful or non-blocking result.
    pub fn is_continuable(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::Degraded { .. } | Self::Skipped { .. }
        )
    }

    /// Whether this outcome is fatal (requires abort).
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal { .. })
    }
}

/// Result of executing a single startup phase.
///
/// Addresses: Requirement 1 AC 1.4
#[derive(Debug, Clone)]
pub struct PhaseResult {
    /// Which phase completed.
    pub phase: StartupPhase,
    /// Whether the phase succeeded or failed.
    pub outcome: PhaseOutcome,
    /// Duration of this phase's execution.
    pub duration: Duration,
    /// Sequence number in which this phase was executed.
    pub execution_order: usize,
}

/// Aggregated result of the full startup sequence.
///
/// Addresses: Requirement 1
#[derive(Debug)]
pub struct StartupResult {
    /// Results for each phase that was executed.
    pub phases: Vec<PhaseResult>,
    /// Whether startup was aborted due to a fatal error.
    pub aborted: bool,
    /// The phase that caused the abort (if any).
    pub abort_phase: Option<StartupPhase>,
    /// Total time from start to Phase 8 (first frame).
    pub time_to_interactive: Duration,
}

impl StartupResult {
    /// Whether all phases completed successfully.
    pub fn all_successful(&self) -> bool {
        !self.aborted
            && self.phases.iter().all(|r| {
                matches!(
                    r.outcome,
                    PhaseOutcome::Success | PhaseOutcome::Skipped { .. }
                )
            })
    }

    /// Collect all degraded phase results.
    pub fn degraded_phases(&self) -> Vec<&PhaseResult> {
        self.phases
            .iter()
            .filter(|r| matches!(r.outcome, PhaseOutcome::Degraded { .. }))
            .collect()
    }

    /// Collect deferred warnings from degraded phases.
    pub fn deferred_warnings(&self) -> Vec<String> {
        self.phases
            .iter()
            .filter_map(|r| {
                if let PhaseOutcome::Degraded { reason } = &r.outcome {
                    Some(format!("Phase {}: {}", r.phase.number(), reason))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Executes the startup sequence and records results.
///
/// Each phase is executed by a caller-provided closure. This function
/// handles ordering, timing, fatal detection, and result collection.
///
/// Addresses: Requirement 1 AC 1.1, 1.2, 1.3, 1.4, 1.5
pub fn execute_startup_sequence<F>(mut execute_phase: F) -> StartupResult
where
    F: FnMut(StartupPhase) -> PhaseOutcome,
{
    let start = Instant::now();
    let mut phases = Vec::with_capacity(10);
    let mut execution_order = 0;
    let mut aborted = false;
    let mut abort_phase = None;
    let time_to_interactive: Duration;

    // Execute pre-render phases (1–7) in order
    for &phase in &StartupPhase::PRE_RENDER {
        execution_order += 1;
        let phase_start = Instant::now();
        let outcome = execute_phase(phase);
        let duration = phase_start.elapsed();

        let is_fatal = outcome.is_fatal();
        phases.push(PhaseResult {
            phase,
            outcome,
            duration,
            execution_order,
        });

        if is_fatal {
            aborted = true;
            abort_phase = Some(phase);
            time_to_interactive = start.elapsed();
            return StartupResult {
                phases,
                aborted,
                abort_phase,
                time_to_interactive,
            };
        }
    }

    // Execute Phase 8 (render first frame)
    execution_order += 1;
    let phase_start = Instant::now();
    let outcome = execute_phase(StartupPhase::RenderFirstFrame);
    let duration = phase_start.elapsed();
    time_to_interactive = start.elapsed();

    let is_fatal = outcome.is_fatal();
    phases.push(PhaseResult {
        phase: StartupPhase::RenderFirstFrame,
        outcome,
        duration,
        execution_order,
    });

    if is_fatal {
        return StartupResult {
            phases,
            aborted: true,
            abort_phase: Some(StartupPhase::RenderFirstFrame),
            time_to_interactive,
        };
    }

    // Execute post-render phases (9–10)
    for &phase in &StartupPhase::POST_RENDER {
        execution_order += 1;
        let phase_start = Instant::now();
        let outcome = execute_phase(phase);
        let duration = phase_start.elapsed();

        phases.push(PhaseResult {
            phase,
            outcome,
            duration,
            execution_order,
        });
    }

    StartupResult {
        phases,
        aborted,
        abort_phase,
        time_to_interactive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phases_execute_in_correct_order() {
        // Validates: Requirement 1 AC 1.1
        let result = execute_startup_sequence(|_| PhaseOutcome::Success);

        assert_eq!(result.phases.len(), 10);
        for (i, phase_result) in result.phases.iter().enumerate() {
            assert_eq!(phase_result.execution_order, i + 1);
            assert_eq!(
                phase_result.phase,
                StartupPhase::ALL[i],
                "Phase at index {} should be {:?}",
                i,
                StartupPhase::ALL[i]
            );
        }
    }

    #[test]
    fn pre_render_phases_complete_before_render() {
        // Validates: Requirement 1 AC 1.2
        let result = execute_startup_sequence(|_| PhaseOutcome::Success);

        let render_order = result
            .phases
            .iter()
            .find(|p| p.phase == StartupPhase::RenderFirstFrame)
            .unwrap()
            .execution_order;

        for phase_result in &result.phases {
            if phase_result.phase.is_pre_render() {
                assert!(
                    phase_result.execution_order < render_order,
                    "Pre-render phase {:?} (order {}) should execute before render (order {})",
                    phase_result.phase,
                    phase_result.execution_order,
                    render_order
                );
            }
        }
    }

    #[test]
    fn post_render_phases_execute_after_render() {
        // Validates: Requirement 1 AC 1.3
        let result = execute_startup_sequence(|_| PhaseOutcome::Success);

        let render_order = result
            .phases
            .iter()
            .find(|p| p.phase == StartupPhase::RenderFirstFrame)
            .unwrap()
            .execution_order;

        for phase_result in &result.phases {
            if phase_result.phase.is_post_render() {
                assert!(
                    phase_result.execution_order > render_order,
                    "Post-render phase {:?} (order {}) should execute after render (order {})",
                    phase_result.phase,
                    phase_result.execution_order,
                    render_order
                );
            }
        }
    }

    #[test]
    fn non_fatal_failure_continues_to_next_phase() {
        // Validates: Requirement 1 AC 1.4
        let result = execute_startup_sequence(|phase| {
            if phase == StartupPhase::LoadPlugins {
                PhaseOutcome::Degraded {
                    reason: "plugin X failed".to_string(),
                }
            } else {
                PhaseOutcome::Success
            }
        });

        assert!(!result.aborted);
        assert_eq!(result.phases.len(), 10);
        // Phase 5 degraded but subsequent phases executed
        let phase5 = result
            .phases
            .iter()
            .find(|p| p.phase == StartupPhase::LoadPlugins)
            .unwrap();
        assert!(matches!(phase5.outcome, PhaseOutcome::Degraded { .. }));
    }

    #[test]
    fn fatal_phase_1_aborts_startup() {
        // Validates: Requirement 1 AC 1.5
        let result = execute_startup_sequence(|phase| {
            if phase == StartupPhase::ParseCliArguments {
                PhaseOutcome::Fatal {
                    reason: "invalid arguments".to_string(),
                }
            } else {
                PhaseOutcome::Success
            }
        });

        assert!(result.aborted);
        assert_eq!(result.abort_phase, Some(StartupPhase::ParseCliArguments));
        assert_eq!(result.phases.len(), 1); // Only Phase 1 executed
    }

    #[test]
    fn multiple_degraded_phases_still_complete() {
        // Validates: Requirement 1 AC 1.4, Requirement 11 AC 11.1
        let result = execute_startup_sequence(|phase| match phase {
            StartupPhase::LoadPlugins => PhaseOutcome::Degraded {
                reason: "plugin failed".to_string(),
            },
            StartupPhase::LoadSessionState => PhaseOutcome::Degraded {
                reason: "session corrupt".to_string(),
            },
            StartupPhase::RestoreLayout => PhaseOutcome::Degraded {
                reason: "layout corrupt".to_string(),
            },
            _ => PhaseOutcome::Success,
        });

        assert!(!result.aborted);
        assert_eq!(result.phases.len(), 10);
        assert_eq!(result.degraded_phases().len(), 3);
    }

    #[test]
    fn deferred_warnings_collected_from_degraded_phases() {
        let result = execute_startup_sequence(|phase| {
            if phase == StartupPhase::LoadSessionState {
                PhaseOutcome::Degraded {
                    reason: "session file corrupt".to_string(),
                }
            } else {
                PhaseOutcome::Success
            }
        });

        let warnings = result.deferred_warnings();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("session file corrupt"));
    }

    #[test]
    fn all_successful_returns_true_when_no_failures() {
        let result = execute_startup_sequence(|_| PhaseOutcome::Success);
        assert!(result.all_successful());
    }

    #[test]
    fn all_successful_returns_false_when_degraded() {
        let result = execute_startup_sequence(|phase| {
            if phase == StartupPhase::LoadPlugins {
                PhaseOutcome::Degraded {
                    reason: "test".to_string(),
                }
            } else {
                PhaseOutcome::Success
            }
        });
        assert!(!result.all_successful());
    }

    #[test]
    fn skipped_phases_count_as_successful() {
        let result = execute_startup_sequence(|phase| {
            if phase == StartupPhase::LoadSessionState {
                PhaseOutcome::Skipped {
                    reason: "restore disabled".to_string(),
                }
            } else {
                PhaseOutcome::Success
            }
        });
        assert!(result.all_successful());
    }

    #[test]
    fn phase_number_returns_correct_value() {
        assert_eq!(StartupPhase::ParseCliArguments.number(), 1);
        assert_eq!(StartupPhase::RenderFirstFrame.number(), 8);
        assert_eq!(StartupPhase::CrashRecovery.number(), 10);
    }

    #[test]
    fn phase_is_pre_render_correct() {
        assert!(StartupPhase::ParseCliArguments.is_pre_render());
        assert!(StartupPhase::RestoreLayout.is_pre_render());
        assert!(!StartupPhase::RenderFirstFrame.is_pre_render());
        assert!(!StartupPhase::OpenFiles.is_pre_render());
    }

    #[test]
    fn phase_is_post_render_correct() {
        assert!(!StartupPhase::ParseCliArguments.is_post_render());
        assert!(!StartupPhase::RenderFirstFrame.is_post_render());
        assert!(StartupPhase::OpenFiles.is_post_render());
        assert!(StartupPhase::CrashRecovery.is_post_render());
    }
}
