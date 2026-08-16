//! Reload prompt UI abstraction.
//!
//! Defines the `ExternalModDialogProvider` trait and associated action enums
//! for GUI-independent user interaction. The shell layer (ff-desktop) provides
//! concrete implementations.

use crate::change_event::ExternalChange;
use crate::types::DocumentId;

/// The user's response to an external content modification prompt.
///
/// Addresses: Requirement 4, criteria 1–8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptAction {
    /// Reload from disk (discard local changes if any).
    Reload,
    /// Keep in-memory content, ignore the external change.
    Keep,
    /// Show a diff between in-memory and on-disk content.
    Diff,
    /// Save buffer content to a new location (for deleted files).
    SaveAs,
    /// Continue editing with no backing file (for deleted files).
    KeepEditing,
    /// Close the document tab (for deleted files).
    Close,
    /// Update the document's URI to track the new location (for renamed files).
    FollowRename,
    /// Keep the original URI (document becomes orphaned, for renamed files).
    KeepOldPath,
}

/// Options presented to the user in a reload prompt.
///
/// Addresses: Requirement 4, criteria 1–3
#[derive(Debug, Clone)]
pub struct PromptOptions {
    /// Short file name (not full path).
    pub file_name: String,
    /// Whether the local buffer has unsaved changes.
    pub is_dirty: bool,
    /// The type of external change.
    pub change_type: crate::change_event::ChangeType,
    /// Available actions the user can choose from.
    pub available_actions: Vec<PromptAction>,
}

/// The user's response to a prompt.
///
/// Wraps a `PromptAction` representing the user's selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptResponse {
    /// The action selected by the user.
    pub action: PromptAction,
}

/// The user's bulk response to a batch notification.
///
/// Addresses: Requirement 8, criteria 3–5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchAction {
    /// Reload all non-dirty documents in the batch.
    ReloadAll,
    /// Dismiss all notifications (keep everything as-is).
    KeepAll,
    /// Present each change individually for review.
    ReviewIndividually,
}

/// A coalesced group of external changes within a debounce window.
///
/// Addresses: Requirement 8, criteria 1–7
#[derive(Debug, Clone, Default)]
pub struct BatchNotification {
    /// Documents with content changes.
    pub modified: Vec<ExternalChange>,
    /// Documents whose backing files were deleted.
    pub deleted: Vec<ExternalChange>,
    /// Documents whose backing files were renamed.
    pub renamed: Vec<ExternalChange>,
}

impl BatchNotification {
    /// Total count of affected documents.
    pub fn total_count(&self) -> usize {
        self.modified.len() + self.deleted.len() + self.renamed.len()
    }

    /// Document IDs of documents in the batch that have dirty buffers.
    pub fn dirty_documents(&self) -> Vec<DocumentId> {
        self.all_events()
            .filter(|e| e.is_dirty)
            .map(|e| e.document_id)
            .collect()
    }

    /// Document IDs of documents in the batch that are clean (safe for auto-reload).
    pub fn clean_documents(&self) -> Vec<DocumentId> {
        self.all_events()
            .filter(|e| !e.is_dirty)
            .map(|e| e.document_id)
            .collect()
    }

    /// Returns an iterator over all events in the batch.
    fn all_events(&self) -> impl Iterator<Item = &ExternalChange> {
        self.modified
            .iter()
            .chain(self.deleted.iter())
            .chain(self.renamed.iter())
    }
}

/// Trait abstraction for external modification dialogs.
///
/// The GUI shell (ff-desktop) provides the concrete implementation.
/// This trait enables testing without a real UI.
///
/// Addresses: GUI Independence cross-cutting requirement
#[async_trait::async_trait]
pub trait ExternalModDialogProvider: Send + Sync {
    /// Show a reload/keep/diff dialog for a content-changed document.
    ///
    /// Addresses: Requirement 4, criteria 1–8
    async fn show_reload_prompt(&self, file_name: &str, is_dirty: bool) -> PromptAction;

    /// Show a notification for a deleted file.
    ///
    /// Addresses: Requirement 6, criteria 1–5
    async fn show_deleted_prompt(&self, file_name: &str, is_dirty: bool) -> PromptAction;

    /// Show a notification for a renamed file.
    ///
    /// Addresses: Requirement 7, criteria 1–6
    async fn show_rename_prompt(
        &self,
        old_name: &str,
        new_name: &str,
        is_dirty: bool,
    ) -> PromptAction;

    /// Show a batch notification for multiple concurrent changes.
    ///
    /// Addresses: Requirement 8, criteria 1–7
    async fn show_batch_prompt(&self, notification: &BatchNotification) -> BatchAction;

    /// Show a brief status bar message (non-blocking).
    ///
    /// Addresses: Requirement 5 AC 3
    fn show_status_message(&self, message: &str, duration_secs: u32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_event::ChangeType;

    #[test]
    fn batch_notification_total_count_sums_all_categories() {
        let batch = BatchNotification {
            modified: vec![
                ExternalChange::content_changed(
                    DocumentId(1),
                    std::time::SystemTime::UNIX_EPOCH,
                    std::time::SystemTime::now(),
                    false,
                ),
                ExternalChange::content_changed(
                    DocumentId(2),
                    std::time::SystemTime::UNIX_EPOCH,
                    std::time::SystemTime::now(),
                    true,
                ),
            ],
            deleted: vec![ExternalChange::file_deleted(DocumentId(3), false)],
            renamed: vec![],
        };
        assert_eq!(batch.total_count(), 3);
    }

    #[test]
    fn batch_notification_dirty_documents_filters_correctly() {
        let batch = BatchNotification {
            modified: vec![
                ExternalChange::content_changed(
                    DocumentId(1),
                    std::time::SystemTime::UNIX_EPOCH,
                    std::time::SystemTime::now(),
                    false,
                ),
                ExternalChange::content_changed(
                    DocumentId(2),
                    std::time::SystemTime::UNIX_EPOCH,
                    std::time::SystemTime::now(),
                    true,
                ),
            ],
            deleted: vec![ExternalChange::file_deleted(DocumentId(3), true)],
            renamed: vec![],
        };
        let dirty = batch.dirty_documents();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&DocumentId(2)));
        assert!(dirty.contains(&DocumentId(3)));
    }

    #[test]
    fn batch_notification_clean_documents_filters_correctly() {
        let batch = BatchNotification {
            modified: vec![
                ExternalChange::content_changed(
                    DocumentId(1),
                    std::time::SystemTime::UNIX_EPOCH,
                    std::time::SystemTime::now(),
                    false,
                ),
                ExternalChange::content_changed(
                    DocumentId(2),
                    std::time::SystemTime::UNIX_EPOCH,
                    std::time::SystemTime::now(),
                    true,
                ),
            ],
            deleted: vec![],
            renamed: vec![],
        };
        let clean = batch.clean_documents();
        assert_eq!(clean.len(), 1);
        assert!(clean.contains(&DocumentId(1)));
    }

    #[test]
    fn prompt_options_construction() {
        let opts = PromptOptions {
            file_name: "test.rs".to_string(),
            is_dirty: true,
            change_type: ChangeType::ContentChanged,
            available_actions: vec![PromptAction::Reload, PromptAction::Keep, PromptAction::Diff],
        };
        assert_eq!(opts.file_name, "test.rs");
        assert!(opts.is_dirty);
        assert_eq!(opts.available_actions.len(), 3);
    }
}
