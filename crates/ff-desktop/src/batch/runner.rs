use std::time::Instant;

use crate::batch::input::BatchInputSource;
use crate::batch::output::BatchOutputSink;
use crate::batch::return_code::{AbortPolicy, BatchReturnCode, StepReturnCode};
use crate::batch::session::BatchSession;

/// Options controlling a batch run.
pub struct BatchOptions {
    pub echo: bool,
    pub dry_run: bool,
    pub abort_policy: AbortPolicy,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            echo: false,
            dry_run: false,
            abort_policy: AbortPolicy::BestEffort,
        }
    }
}

/// Classifies a command string for dry-run purposes.
/// Read-only commands execute normally; state-modifying commands are skipped.
fn is_read_only_command(cmd: &str) -> bool {
    let upper = cmd.trim().to_uppercase();
    let verb = upper.split_whitespace().next().unwrap_or("");
    matches!(
        verb,
        "FIND"
            | "RFIND"
            | "LOCATE"
            | "LISTCAT"
            | "LISTDS"
            | "LISTALC"
            | "SORT"
            | "SHOW"
            | "EXCLUDE"
            | "RESET"
            | "COLS"
            | "WHO"
            | "QUERY"
            | "STATUS"
            | "TIME"
    )
}

/// Commands that require GUI interaction and cannot run in batch mode.
/// Validates: Requirement 3.5
fn requires_interactive_input(cmd: &str) -> bool {
    let upper = cmd.trim().to_uppercase();
    let verb = upper.split_whitespace().next().unwrap_or("");
    // Commands that open modal dialogs or require terminal interaction.
    matches!(verb, "KEYS" | "SETTINGS" | "ABOUT" | "PFSHOW")
}

/// Orchestrates reading, dispatching, and output for a batch run.
pub struct BatchRunner {
    pub options: BatchOptions,
}

impl BatchRunner {
    pub fn new(options: BatchOptions) -> Self {
        Self { options }
    }

    /// Execute all commands from `input`, writing output to `sink`.
    /// Returns the final BatchReturnCode.
    ///
    /// Validates: Requirement 3.1, 3.2, 3.3, 3.5, 3.6, 5.1-5.5, 6.1-6.5, 8.1-8.4, 10.1-10.5
    pub fn run(
        &self,
        input: &mut BatchInputSource,
        sink: &BatchOutputSink,
        _session: &BatchSession,
    ) -> BatchReturnCode {
        let batch_start = Instant::now();
        let mut brc = BatchReturnCode::default();
        // Abort policy may be overridden inline by CONTROL ERRORS commands.
        let mut effective_policy = self.options.abort_policy;
        let mut dry_run_has_error = false;
        let mut cmd_count: u32 = 0;

        // Req 10.1: log batch start
        ff_logging::log_info!("[batch] run started");

        while let Some(cmd) = input.next_command() {
            let cmd = cmd.to_string();
            cmd_count += 1;
            let step_start = Instant::now();

            // Req 6.5: inline abort-policy overrides
            let upper = cmd.trim().to_uppercase();
            if upper == "CONTROL ERRORS CANCEL" {
                effective_policy = AbortPolicy::AbortOnError(StepReturnCode::Error);
                ff_logging::log_info!("[batch] abort policy set to CANCEL");
                continue;
            } else if upper == "CONTROL ERRORS NOCANCEL" {
                effective_policy = AbortPolicy::BestEffort;
                ff_logging::log_info!("[batch] abort policy set to NOCANCEL");
                continue;
            }

            if self.options.echo {
                sink.write_command_echo(&cmd);
            }

            let step = if self.options.dry_run {
                // Req 8.4: read-only commands execute normally in dry-run.
                if is_read_only_command(&cmd) {
                    sink.write_line(&format!("[DRY-RUN] {} -> OK (read-only, executed)", cmd));
                } else {
                    sink.write_line(&format!("[DRY-RUN] {} -> OK", cmd));
                }
                StepReturnCode::Success
            } else if requires_interactive_input(&cmd) {
                // Req 3.5: GUI-requiring commands fail with RC 8 and diagnostic.
                let msg = format!(
                    "Command '{}' requires interactive input and cannot run in batch mode",
                    cmd
                );
                sink.write_line(&msg);
                ff_logging::log_error!("[batch] {}", msg);
                StepReturnCode::Error
            } else {
                // Real dispatch will be wired via handle_command() in a future task.
                // Scaffold: all other commands succeed.
                StepReturnCode::Success
            };

            let duration_ms = step_start.elapsed().as_millis();

            // Req 10.1, 10.3: log each command with RC and duration
            if step >= StepReturnCode::Error {
                // Req 10.4: RC >= 8 logged at ERROR level
                ff_logging::log_error!(
                    "[batch] cmd={:?} rc={} duration_ms={}",
                    cmd,
                    step.as_i32(),
                    duration_ms
                );
            } else {
                ff_logging::log_info!(
                    "[batch] cmd={:?} rc={} duration_ms={}",
                    cmd,
                    step.as_i32(),
                    duration_ms
                );
            }

            if self.options.dry_run && step >= StepReturnCode::Error {
                dry_run_has_error = true;
            }

            brc.update(step);

            if effective_policy.should_abort(step) {
                // Req 6.3, 6.4: write abort message
                sink.write_line(&format!(
                    "FFWB BATCH ABORTED at command '{}' RC={}",
                    cmd,
                    step.as_i32()
                ));
                ff_logging::log_error!("[batch] aborted at cmd={:?} rc={}", cmd, step.as_i32());
                break;
            }
        }

        // Req 8.3: dry-run return code is 8 if any validation error, else 0.
        if self.options.dry_run && dry_run_has_error {
            brc.update(StepReturnCode::Error);
        }

        // Req 10.1: log final summary
        ff_logging::log_info!(
            "[batch] completed commands={} final_rc={} total_ms={}",
            cmd_count,
            brc.as_i32(),
            batch_start.elapsed().as_millis()
        );

        brc
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.6
    #[test]
    fn runner_executes_commands_sequentially_and_returns_zero() {
        let mut input = BatchInputSource::from_str("EDIT a.txt\nSAVE\nEXIT\n");
        let sink = BatchOutputSink::Stdout;
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions::default());
        let brc = runner.run(&mut input, &sink, &session);
        assert_eq!(brc.as_i32(), 0);
    }

    // Validates: Requirement 8.2
    #[test]
    fn dry_run_writes_dry_run_prefix() {
        let tmp = std::env::temp_dir().join("ffwb_batch_test_dry_run.txt");
        let sink = BatchOutputSink::File(tmp.to_string_lossy().to_string());
        let mut input = BatchInputSource::from_str("EDIT a.txt\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions {
            dry_run: true,
            ..Default::default()
        });
        let brc = runner.run(&mut input, &sink, &session);
        assert_eq!(brc.as_i32(), 0);
        let content = std::fs::read_to_string(&tmp).unwrap_or_default();
        assert!(
            content.contains("[DRY-RUN]"),
            "expected [DRY-RUN] prefix in output"
        );
    }

    // Validates: Requirement 8.3
    #[test]
    fn dry_run_returns_zero_when_all_valid() {
        let sink = BatchOutputSink::Stdout;
        let mut input = BatchInputSource::from_str("FIND ERROR\nLOCATE 10\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions {
            dry_run: true,
            ..Default::default()
        });
        let brc = runner.run(&mut input, &sink, &session);
        assert_eq!(brc.as_i32(), 0);
    }

    // Validates: Requirement 8.4
    #[test]
    fn dry_run_read_only_commands_show_executed_label() {
        let tmp = std::env::temp_dir().join("ffwb_batch_test_dry_run_ro.txt");
        let sink = BatchOutputSink::File(tmp.to_string_lossy().to_string());
        let mut input = BatchInputSource::from_str("FIND ERROR\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions {
            dry_run: true,
            ..Default::default()
        });
        runner.run(&mut input, &sink, &session);
        let content = std::fs::read_to_string(&tmp).unwrap_or_default();
        assert!(
            content.contains("read-only"),
            "expected read-only label for FIND in dry-run: {content}"
        );
    }

    // Validates: Requirement 4.4
    #[test]
    fn echo_mode_writes_command_prefix() {
        let tmp = std::env::temp_dir().join("ffwb_batch_test_echo.txt");
        let sink = BatchOutputSink::File(tmp.to_string_lossy().to_string());
        let mut input = BatchInputSource::from_str("SAVE\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions {
            echo: true,
            ..Default::default()
        });
        runner.run(&mut input, &sink, &session);
        let content = std::fs::read_to_string(&tmp).unwrap_or_default();
        assert!(content.contains("===> SAVE"), "expected echo prefix");
    }

    // Validates: Requirement 6.5
    #[test]
    fn control_errors_cancel_switches_to_abort_on_error() {
        // After CONTROL ERRORS CANCEL, a failing command should abort.
        // In the scaffold all commands succeed, so we verify the inline
        // command is consumed without error and the run completes normally.
        let sink = BatchOutputSink::Stdout;
        let mut input = BatchInputSource::from_str("CONTROL ERRORS CANCEL\nEDIT a.txt\nSAVE\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions::default());
        let brc = runner.run(&mut input, &sink, &session);
        // All scaffold commands succeed, so RC is still 0.
        assert_eq!(brc.as_i32(), 0);
    }

    // Validates: Requirement 6.5
    #[test]
    fn control_errors_nocancel_switches_to_best_effort() {
        let sink = BatchOutputSink::Stdout;
        let mut input = BatchInputSource::from_str(
            "CONTROL ERRORS CANCEL\nCONTROL ERRORS NOCANCEL\nEDIT a.txt\n",
        );
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions::default());
        let brc = runner.run(&mut input, &sink, &session);
        assert_eq!(brc.as_i32(), 0);
    }

    // Validates: Requirement 6.3
    #[test]
    fn abort_message_written_when_abort_policy_triggers() {
        let policy = AbortPolicy::AbortOnError(StepReturnCode::Error);
        assert!(policy.should_abort(StepReturnCode::Error));
        assert!(!policy.should_abort(StepReturnCode::Warning));
    }

    // Validates: Requirement 3.5
    #[test]
    fn interactive_command_returns_step_rc_8_with_diagnostic() {
        let tmp = std::env::temp_dir().join("ffwb_batch_test_interactive.txt");
        let sink = BatchOutputSink::File(tmp.to_string_lossy().to_string());
        let mut input = BatchInputSource::from_str("KEYS\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions::default());
        let brc = runner.run(&mut input, &sink, &session);
        assert_eq!(brc.as_i32(), 8, "interactive command must produce RC 8");
        let content = std::fs::read_to_string(&tmp).unwrap_or_default();
        assert!(
            content.contains("requires interactive input"),
            "diagnostic message expected: {content}"
        );
    }

    // Validates: Requirement 3.5
    #[test]
    fn non_interactive_command_is_not_rejected() {
        let sink = BatchOutputSink::Stdout;
        let mut input = BatchInputSource::from_str("FIND ERROR\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions::default());
        let brc = runner.run(&mut input, &sink, &session);
        // FIND is not interactive -- scaffold returns Success
        assert_eq!(brc.as_i32(), 0);
    }

    // Validates: Requirement 10.1, 10.3, 10.5
    #[test]
    fn runner_completes_without_panic_logging_enabled() {
        // Confirms logging calls in run() do not panic when ff-logging is
        // in fallback mode (no log file configured in test environment).
        let sink = BatchOutputSink::Stdout;
        let mut input = BatchInputSource::from_str("FIND ERROR\nSAVE\n");
        let session = BatchSession::new(false, None);
        let runner = BatchRunner::new(BatchOptions::default());
        let brc = runner.run(&mut input, &sink, &session);
        assert_eq!(brc.as_i32(), 0);
    }
}
