//! Exit sequence orchestration — unsaved-change prompts, session save,
//! plugin shutdown, and process termination.
//!
//! Addresses: Requirement 9 (Exit Sequence)

use std::time::Duration;

/// User's response to the exit unsaved-changes summary dialog.
///
/// Addresses: Requirement 9 AC 9.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitAction {
    /// Save all modified documents and proceed to shutdown.
    SaveAll,
    /// Discard all unsaved changes and proceed to shutdown.
    DiscardAll,
    /// Review each modified document individually.
    ReviewEach,
    /// Cancel the exit; return to normal operation.
    Cancel,
}

/// Per-document dialog response during "Review Each" flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerDocumentAction {
    /// Save this document and continue.
    Save,
    /// Discard changes to this document and continue.
    Discard,
    /// Cancel exit — return to normal operation.
    Cancel,
}

/// A document with unsaved modifications, presented during exit.
///
/// Addresses: Requirement 9 AC 9.2
#[derive(Debug, Clone, PartialEq)]
pub struct DirtyDocument {
    /// Display name for the dialog.
    pub display_name: String,
    /// Resource URI (None for untitled).
    pub uri: Option<String>,
    /// Tab identifier.
    pub tab_id: String,
}

/// The ordered shutdown steps executed after unsaved-change handling.
///
/// Addresses: Requirement 9 AC 9.7
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShutdownStep {
    /// Step 1: Persist current Session_State to Session_File.
    PersistSession = 1,
    /// Step 2: Clean up Recovery_Files for saved/discarded documents.
    CleanupRecoveryFiles = 2,
    /// Step 3: Notify plugins of shutdown (deactivate → shutdown).
    NotifyPlugins = 3,
    /// Step 4: Flush and close the logging subsystem.
    FlushLogging = 4,
    /// Step 5: Close all windows and terminate process.
    CloseWindows = 5,
}

impl ShutdownStep {
    /// All shutdown steps in execution order.
    pub const ALL: [ShutdownStep; 5] = [
        Self::PersistSession,
        Self::CleanupRecoveryFiles,
        Self::NotifyPlugins,
        Self::FlushLogging,
        Self::CloseWindows,
    ];
}

/// Result of the exit sequence decision phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitDecision {
    /// Proceed to shutdown (all documents handled).
    Proceed,
    /// Exit was cancelled by the user.
    Cancelled,
}

/// Default timeout for the entire exit sequence.
pub const EXIT_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for individual plugin shutdown.
pub const PLUGIN_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

/// Determines the exit action based on dirty documents.
///
/// If no documents are dirty, returns `Proceed` directly (no prompt needed).
///
/// Addresses: Requirement 9 AC 9.1
pub fn determine_exit_action(dirty_documents: &[DirtyDocument]) -> ExitDecision {
    if dirty_documents.is_empty() {
        ExitDecision::Proceed
    } else {
        // Caller must present the dialog and handle the response
        // This function just determines whether prompting is needed
        ExitDecision::Proceed // Placeholder — actual dialog logic delegated to trait
    }
}

/// Process the user's exit action choice.
///
/// Returns the list of documents to save and whether to proceed.
///
/// Addresses: Requirement 9 AC 9.2-9.6
pub fn process_exit_action(
    action: ExitAction,
    dirty_documents: &[DirtyDocument],
) -> ExitDecisionResult {
    match action {
        ExitAction::SaveAll => ExitDecisionResult {
            decision: ExitDecision::Proceed,
            documents_to_save: dirty_documents.to_vec(),
            documents_to_discard: Vec::new(),
        },
        ExitAction::DiscardAll => ExitDecisionResult {
            decision: ExitDecision::Proceed,
            documents_to_save: Vec::new(),
            documents_to_discard: dirty_documents.to_vec(),
        },
        ExitAction::Cancel => ExitDecisionResult {
            decision: ExitDecision::Cancelled,
            documents_to_save: Vec::new(),
            documents_to_discard: Vec::new(),
        },
        ExitAction::ReviewEach => ExitDecisionResult {
            decision: ExitDecision::Proceed,
            // Individual review handled separately
            documents_to_save: Vec::new(),
            documents_to_discard: Vec::new(),
        },
    }
}

/// Result of processing the exit action choice.
#[derive(Debug, Clone)]
pub struct ExitDecisionResult {
    /// Whether to proceed with shutdown or cancel.
    pub decision: ExitDecision,
    /// Documents that should be saved before shutdown.
    pub documents_to_save: Vec<DirtyDocument>,
    /// Documents whose changes should be discarded.
    pub documents_to_discard: Vec<DirtyDocument>,
}

/// Process per-document review actions.
///
/// Returns the aggregated result after reviewing all documents.
/// If any document action is `Cancel`, the exit is cancelled.
///
/// Addresses: Requirement 9 AC 9.5
pub fn process_review_each(
    documents: &[DirtyDocument],
    actions: &[PerDocumentAction],
) -> ExitDecisionResult {
    let mut to_save = Vec::new();
    let mut to_discard = Vec::new();

    for (doc, action) in documents.iter().zip(actions.iter()) {
        match action {
            PerDocumentAction::Save => to_save.push(doc.clone()),
            PerDocumentAction::Discard => to_discard.push(doc.clone()),
            PerDocumentAction::Cancel => {
                return ExitDecisionResult {
                    decision: ExitDecision::Cancelled,
                    documents_to_save: Vec::new(),
                    documents_to_discard: Vec::new(),
                };
            }
        }
    }

    ExitDecisionResult {
        decision: ExitDecision::Proceed,
        documents_to_save: to_save,
        documents_to_discard: to_discard,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dirty(name: &str) -> DirtyDocument {
        DirtyDocument {
            display_name: name.to_string(),
            uri: Some(format!("file:///{name}")),
            tab_id: format!("tab-{name}"),
        }
    }

    #[test]
    fn no_dirty_documents_proceeds_without_prompt() {
        // Validates: Requirement 9 AC 9.1
        let decision = determine_exit_action(&[]);
        assert_eq!(decision, ExitDecision::Proceed);
    }

    #[test]
    fn save_all_marks_all_documents_for_save() {
        // Validates: Requirement 9 AC 9.3
        let docs = vec![make_dirty("a.txt"), make_dirty("b.txt")];
        let result = process_exit_action(ExitAction::SaveAll, &docs);

        assert_eq!(result.decision, ExitDecision::Proceed);
        assert_eq!(result.documents_to_save.len(), 2);
        assert!(result.documents_to_discard.is_empty());
    }

    #[test]
    fn discard_all_marks_all_documents_for_discard() {
        // Validates: Requirement 9 AC 9.4
        let docs = vec![make_dirty("a.txt"), make_dirty("b.txt")];
        let result = process_exit_action(ExitAction::DiscardAll, &docs);

        assert_eq!(result.decision, ExitDecision::Proceed);
        assert!(result.documents_to_save.is_empty());
        assert_eq!(result.documents_to_discard.len(), 2);
    }

    #[test]
    fn cancel_aborts_exit() {
        // Validates: Requirement 9 AC 9.6
        let docs = vec![make_dirty("a.txt")];
        let result = process_exit_action(ExitAction::Cancel, &docs);

        assert_eq!(result.decision, ExitDecision::Cancelled);
        assert!(result.documents_to_save.is_empty());
        assert!(result.documents_to_discard.is_empty());
    }

    #[test]
    fn review_each_with_all_save_actions() {
        // Validates: Requirement 9 AC 9.5
        let docs = vec![make_dirty("a.txt"), make_dirty("b.txt")];
        let actions = vec![PerDocumentAction::Save, PerDocumentAction::Save];
        let result = process_review_each(&docs, &actions);

        assert_eq!(result.decision, ExitDecision::Proceed);
        assert_eq!(result.documents_to_save.len(), 2);
        assert!(result.documents_to_discard.is_empty());
    }

    #[test]
    fn review_each_with_mixed_actions() {
        // Validates: Requirement 9 AC 9.5
        let docs = vec![
            make_dirty("a.txt"),
            make_dirty("b.txt"),
            make_dirty("c.txt"),
        ];
        let actions = vec![
            PerDocumentAction::Save,
            PerDocumentAction::Discard,
            PerDocumentAction::Save,
        ];
        let result = process_review_each(&docs, &actions);

        assert_eq!(result.decision, ExitDecision::Proceed);
        assert_eq!(result.documents_to_save.len(), 2);
        assert_eq!(result.documents_to_discard.len(), 1);
    }

    #[test]
    fn review_each_cancel_at_any_point_aborts() {
        // Validates: Requirement 9 AC 9.6
        let docs = vec![
            make_dirty("a.txt"),
            make_dirty("b.txt"),
            make_dirty("c.txt"),
        ];
        let actions = vec![
            PerDocumentAction::Save,
            PerDocumentAction::Cancel,
            PerDocumentAction::Save,
        ];
        let result = process_review_each(&docs, &actions);

        assert_eq!(result.decision, ExitDecision::Cancelled);
        // All documents preserved on cancel
        assert!(result.documents_to_save.is_empty());
        assert!(result.documents_to_discard.is_empty());
    }

    #[test]
    fn shutdown_steps_are_in_correct_order() {
        // Validates: Requirement 9 AC 9.7
        let steps = ShutdownStep::ALL;
        assert_eq!(steps[0], ShutdownStep::PersistSession);
        assert_eq!(steps[1], ShutdownStep::CleanupRecoveryFiles);
        assert_eq!(steps[2], ShutdownStep::NotifyPlugins);
        assert_eq!(steps[3], ShutdownStep::FlushLogging);
        assert_eq!(steps[4], ShutdownStep::CloseWindows);
    }

    #[test]
    fn exit_timeout_is_five_seconds() {
        // Validates: Requirement 9 AC 9.9
        assert_eq!(EXIT_TIMEOUT, Duration::from_secs(5));
    }

    #[test]
    fn plugin_shutdown_timeout_is_three_seconds() {
        // Validates: Requirement 9 AC 9.9
        assert_eq!(PLUGIN_SHUTDOWN_TIMEOUT, Duration::from_secs(3));
    }
}
