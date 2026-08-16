//! Unsaved-changes guard implementation.
//!
//! Checks if a document has unsaved changes and presents the
//! Save/Discard/Cancel dialog when needed.

use crate::error::FileOpsError;
use crate::traits::{DialogProvider, DocumentAccess};
use crate::unsaved_guard::UnsavedChangesAction;

/// Result of the unsaved-changes guard check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardResult {
    /// Proceed with the operation (document is clean, or user chose Save/Discard).
    Proceed,
    /// The operation should be aborted (user chose Cancel, or save failed).
    Abort,
}

/// Check if a document has unsaved changes and prompt if needed.
///
/// Returns `GuardResult::Proceed` if the operation should continue,
/// or `GuardResult::Abort` if it should be cancelled.
///
/// Addresses: Requirement 9 AC 9.1–9.5
pub async fn check_unsaved_changes(
    document: &dyn DocumentAccess,
    dialog: &dyn DialogProvider,
    unsaved_prompt_enabled: bool,
) -> Result<GuardResult, FileOpsError> {
    // If document is not dirty, no guard needed
    if !document.is_dirty() {
        return Ok(GuardResult::Proceed);
    }

    // If prompt is disabled, proceed without asking
    if !unsaved_prompt_enabled {
        return Ok(GuardResult::Proceed);
    }

    // Show dialog
    let action = dialog.show_unsaved_changes(document.display_name()).await;

    match action {
        UnsavedChangesAction::Save => {
            // Caller is responsible for performing the save
            // We return Proceed to indicate the guard passed
            Ok(GuardResult::Proceed)
        }
        UnsavedChangesAction::Discard => Ok(GuardResult::Proceed),
        UnsavedChangesAction::Cancel => Ok(GuardResult::Abort),
    }
}

/// Extended guard result that includes the user's choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardAction {
    /// Document is clean — no action needed, proceed.
    AlreadyClean,
    /// User chose to save first.
    SaveFirst,
    /// User chose to discard changes.
    Discard,
    /// User cancelled — abort the operation.
    Cancel,
    /// Prompt is disabled — proceed without saving.
    PromptDisabled,
}

/// Check unsaved changes and return the specific action to take.
///
/// This gives the caller more control over the save step.
///
/// Addresses: Requirement 9 AC 9.1–9.7
pub async fn determine_guard_action(
    document: &dyn DocumentAccess,
    dialog: &dyn DialogProvider,
    unsaved_prompt_enabled: bool,
) -> GuardAction {
    if !document.is_dirty() {
        return GuardAction::AlreadyClean;
    }

    if !unsaved_prompt_enabled {
        return GuardAction::PromptDisabled;
    }

    let action = dialog.show_unsaved_changes(document.display_name()).await;

    match action {
        UnsavedChangesAction::Save => GuardAction::SaveFirst,
        UnsavedChangesAction::Discard => GuardAction::Discard,
        UnsavedChangesAction::Cancel => GuardAction::Cancel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_result_variants_are_distinct() {
        assert_ne!(GuardResult::Proceed, GuardResult::Abort);
    }

    #[test]
    fn guard_action_variants_are_distinct() {
        let all = [
            GuardAction::AlreadyClean,
            GuardAction::SaveFirst,
            GuardAction::Discard,
            GuardAction::Cancel,
            GuardAction::PromptDisabled,
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }
}
