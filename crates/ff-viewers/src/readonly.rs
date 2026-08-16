//! Read-only enforcement — ensures viewers never modify document content.
//!
//! Provides the Command_Dispatch guard that rejects document-mutating commands
//! when Viewer_Mode is active, and performance monitoring for slow viewers.

use std::time::{Duration, Instant};

use crate::error::ViewerError;

/// Maximum allowed time for `on_content_changed` before a warning is logged.
pub const CONTENT_CHANGED_WARNING_THRESHOLD: Duration = Duration::from_millis(100);

/// Commands that are considered document-mutating and should be rejected
/// when Viewer_Mode is active.
const MUTATING_COMMAND_PREFIXES: &[&str] = &[
    "edit.", "delete.", "insert.", "cut.", "paste.", "undo.", "redo.", "format.",
];

/// Check if a command ID represents a document-mutating operation.
///
/// Returns `true` if the command would modify document content.
pub fn is_mutating_command(command_id: &str) -> bool {
    MUTATING_COMMAND_PREFIXES
        .iter()
        .any(|prefix| command_id.starts_with(prefix))
}

/// Guard that enforces the read-only constraint during Viewer_Mode.
///
/// When Viewer_Mode is active, this guard intercepts document-mutating commands
/// and rejects them with a `ViewerReadOnlyViolation` error.
pub struct ReadOnlyGuard {
    /// Whether viewer mode is currently active.
    viewer_mode_active: bool,
    /// The key of the currently active viewer (for error messages).
    active_viewer_key: Option<String>,
}

impl ReadOnlyGuard {
    /// Create a new read-only guard (initially inactive).
    pub fn new() -> Self {
        Self {
            viewer_mode_active: false,
            active_viewer_key: None,
        }
    }

    /// Activate the guard (called when a viewer becomes active).
    pub fn activate(&mut self, viewer_key: &str) {
        self.viewer_mode_active = true;
        self.active_viewer_key = Some(viewer_key.to_string());
    }

    /// Deactivate the guard (called when the viewer is dismissed).
    pub fn deactivate(&mut self) {
        self.viewer_mode_active = false;
        self.active_viewer_key = None;
    }

    /// Returns whether the guard is currently active.
    pub fn is_active(&self) -> bool {
        self.viewer_mode_active
    }

    /// Check if a command should be allowed. Returns an error if the command
    /// is mutating and the guard is active.
    ///
    /// # Errors
    ///
    /// Returns `ViewerError::ViewerReadOnlyViolation` if the command is a
    /// document-mutating operation and Viewer_Mode is active.
    pub fn check_command(&self, command_id: &str) -> Result<(), ViewerError> {
        if !self.viewer_mode_active {
            return Ok(());
        }

        if is_mutating_command(command_id) {
            return Err(ViewerError::ViewerReadOnlyViolation {
                key: self
                    .active_viewer_key
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                command: command_id.to_string(),
            });
        }

        Ok(())
    }
}

impl Default for ReadOnlyGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Measure the execution time of `on_content_changed` and log a warning
/// if it exceeds the threshold.
///
/// Returns `true` if the operation exceeded the warning threshold.
pub fn measure_content_changed_duration(start: Instant) -> bool {
    start.elapsed() > CONTENT_CHANGED_WARNING_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_mutating_command_identifies_edit_commands() {
        assert!(is_mutating_command("edit.delete-line"));
        assert!(is_mutating_command("edit.insert-text"));
        assert!(is_mutating_command("delete.line"));
        assert!(is_mutating_command("insert.char"));
        assert!(is_mutating_command("paste.clipboard"));
        assert!(is_mutating_command("undo.last"));
        assert!(is_mutating_command("redo.last"));
        assert!(is_mutating_command("format.indent"));
    }

    #[test]
    fn is_mutating_command_allows_non_mutating() {
        assert!(!is_mutating_command("viewer.preview"));
        assert!(!is_mutating_command("navigate.goto"));
        assert!(!is_mutating_command("find.next"));
        assert!(!is_mutating_command("view.scroll-down"));
    }

    #[test]
    fn guard_inactive_allows_all_commands() {
        let guard = ReadOnlyGuard::new();
        assert!(guard.check_command("edit.delete-line").is_ok());
        assert!(guard.check_command("viewer.preview").is_ok());
    }

    #[test]
    fn guard_active_rejects_mutating_commands() {
        // Validates: Requirement 8 AC 4
        let mut guard = ReadOnlyGuard::new();
        guard.activate("hex");

        let result = guard.check_command("edit.delete-line");
        assert!(result.is_err());
        match result.unwrap_err() {
            ViewerError::ViewerReadOnlyViolation { key, command } => {
                assert_eq!(key, "hex");
                assert_eq!(command, "edit.delete-line");
            }
            other => panic!("Expected ViewerReadOnlyViolation, got: {other:?}"),
        }
    }

    #[test]
    fn guard_active_allows_non_mutating_commands() {
        // Validates: Requirement 8 AC 4
        let mut guard = ReadOnlyGuard::new();
        guard.activate("hex");

        assert!(guard.check_command("viewer.preview").is_ok());
        assert!(guard.check_command("navigate.goto").is_ok());
        assert!(guard.check_command("find.next").is_ok());
    }

    #[test]
    fn guard_deactivate_allows_mutating_commands_again() {
        let mut guard = ReadOnlyGuard::new();
        guard.activate("hex");
        guard.deactivate();

        assert!(guard.check_command("edit.delete-line").is_ok());
        assert!(!guard.is_active());
    }

    #[test]
    fn measure_content_changed_fast_operation() {
        // Validates: Requirement 8 AC 5
        let start = Instant::now();
        // Immediate check — should be fast
        assert!(!measure_content_changed_duration(start));
    }

    #[test]
    fn measure_content_changed_slow_operation() {
        // Validates: Requirement 8 AC 5
        let start = Instant::now() - Duration::from_millis(150);
        assert!(measure_content_changed_duration(start));
    }
}
