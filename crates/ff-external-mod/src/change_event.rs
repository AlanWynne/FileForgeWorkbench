//! External change event types.
//!
//! Defines the `ExternalChange` enum representing detected modifications,
//! deletions, and renames of backing files for open documents.

use std::time::SystemTime;

use ff_vfs::ResourceUri;

use crate::types::DocumentId;

/// The type of external change detected on a document's backing file.
///
/// Addresses: Requirement 3, Requirement 6, Requirement 7
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// File content was modified externally.
    ContentChanged,
    /// File was deleted externally.
    FileDeleted,
    /// File was renamed/moved externally.
    FileRenamed {
        /// The original URI before the rename.
        old_uri: ResourceUri,
        /// The new URI after the rename.
        new_uri: ResourceUri,
    },
}

/// An event indicating an open document's backing file was externally changed.
///
/// Emitted by the `ExternalModificationDetector` when a discrepancy is detected
/// between the in-memory state and the on-disk state of a document.
///
/// Addresses: Requirement 3, all acceptance criteria
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalChange {
    /// Identifier for the affected document.
    pub document_id: DocumentId,

    /// The type of external change detected.
    pub change_type: ChangeType,

    /// The previously recorded mtime (None for deletion events).
    pub old_mtime: Option<SystemTime>,

    /// The new mtime detected on disk (None for deletion events).
    pub new_mtime: Option<SystemTime>,

    /// Whether the in-memory buffer has unsaved local changes.
    pub is_dirty: bool,
}

impl ExternalChange {
    /// Create a content-changed event.
    pub fn content_changed(
        document_id: DocumentId,
        old_mtime: SystemTime,
        new_mtime: SystemTime,
        is_dirty: bool,
    ) -> Self {
        Self {
            document_id,
            change_type: ChangeType::ContentChanged,
            old_mtime: Some(old_mtime),
            new_mtime: Some(new_mtime),
            is_dirty,
        }
    }

    /// Create a file-deleted event.
    pub fn file_deleted(document_id: DocumentId, is_dirty: bool) -> Self {
        Self {
            document_id,
            change_type: ChangeType::FileDeleted,
            old_mtime: None,
            new_mtime: None,
            is_dirty,
        }
    }

    /// Create a file-renamed event.
    pub fn file_renamed(
        document_id: DocumentId,
        old_uri: ResourceUri,
        new_uri: ResourceUri,
        is_dirty: bool,
    ) -> Self {
        Self {
            document_id,
            change_type: ChangeType::FileRenamed { old_uri, new_uri },
            old_mtime: None,
            new_mtime: None,
            is_dirty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_changed_event_has_correct_fields() {
        let old = SystemTime::UNIX_EPOCH;
        let new = SystemTime::now();
        let event = ExternalChange::content_changed(DocumentId(1), old, new, true);

        assert_eq!(event.document_id, DocumentId(1));
        assert_eq!(event.change_type, ChangeType::ContentChanged);
        assert_eq!(event.old_mtime, Some(old));
        assert_eq!(event.new_mtime, Some(new));
        assert!(event.is_dirty);
    }

    #[test]
    fn file_deleted_event_has_no_mtime() {
        let event = ExternalChange::file_deleted(DocumentId(2), false);

        assert_eq!(event.document_id, DocumentId(2));
        assert_eq!(event.change_type, ChangeType::FileDeleted);
        assert_eq!(event.old_mtime, None);
        assert_eq!(event.new_mtime, None);
        assert!(!event.is_dirty);
    }

    #[test]
    fn file_renamed_event_contains_both_uris() {
        let old_uri = ResourceUri::new("local", "/old/path.rs");
        let new_uri = ResourceUri::new("local", "/new/path.rs");
        let event =
            ExternalChange::file_renamed(DocumentId(3), old_uri.clone(), new_uri.clone(), false);

        assert_eq!(event.document_id, DocumentId(3));
        assert_eq!(
            event.change_type,
            ChangeType::FileRenamed {
                old_uri: old_uri.clone(),
                new_uri: new_uri.clone(),
            }
        );
        assert!(!event.is_dirty);
    }
}
