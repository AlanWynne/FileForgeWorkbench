//! Revert command implementation.
//!
//! Handles `file.revert` — reload resource content from VFS,
//! discarding all in-memory modifications.

use std::time::SystemTime;

use ff_vfs::{ResourceUri, VfsProvider};

use crate::error::FileOpsError;

/// Result of a revert operation.
#[derive(Debug, Clone)]
pub struct RevertResult {
    /// The URI that was reverted.
    pub uri: ResourceUri,
    /// The reloaded content.
    pub content: Vec<u8>,
    /// Updated modification time from VFS.
    pub modification_time: Option<SystemTime>,
    /// Status message for the status bar.
    pub status_message: String,
}

/// Reload resource content from the VFS.
///
/// This is the core revert logic. The caller is responsible for:
/// - Showing the confirmation dialog (when dirty)
/// - Checking that the document has a URI (disabled for untitled)
/// - Replacing the document buffer
/// - Clearing undo/redo stacks
/// - Resetting viewport to line 1
/// - Emitting the `file.reverted` event
///
/// Addresses: Requirement 5 AC 5.2, 5.3, 5.8
pub async fn reload_from_vfs(
    provider: &dyn VfsProvider,
    uri: &ResourceUri,
) -> Result<RevertResult, FileOpsError> {
    let path = uri.path();

    // Read current content from VFS
    let content = provider
        .read(path)
        .await
        .map_err(|source| FileOpsError::VfsReadError {
            operation: "revert".to_string(),
            uri: uri.clone(),
            source,
        })?;

    // Get updated modification time
    let modification_time = provider.stat(path).await.ok().and_then(|m| m.modified);

    Ok(RevertResult {
        uri: uri.clone(),
        content,
        modification_time,
        status_message: "Reverted to saved".to_string(),
    })
}

/// Check if revert is available for a document.
///
/// Revert is disabled when the document has no associated URI (untitled).
///
/// Addresses: Requirement 5 AC 5.6
pub fn is_revert_available(uri: Option<&ResourceUri>) -> bool {
    uri.is_some()
}

/// Whether confirmation is needed before reverting.
///
/// Confirmation is shown only when the document is dirty.
/// When clean, revert reloads immediately (for explicit refresh).
///
/// Addresses: Requirement 5 AC 5.1, 5.5
pub fn needs_revert_confirmation(is_dirty: bool) -> bool {
    is_dirty
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 5 AC 5.6 — disabled for untitled
    #[test]
    fn is_revert_available_false_for_untitled() {
        assert!(!is_revert_available(None));
    }

    // Validates: Requirement 5 AC 5.6 — enabled when URI exists
    #[test]
    fn is_revert_available_true_with_uri() {
        let uri = ResourceUri::new("local", "/file.txt");
        assert!(is_revert_available(Some(&uri)));
    }

    // Validates: Requirement 5 AC 5.1 — confirmation when dirty
    #[test]
    fn needs_confirmation_when_dirty() {
        assert!(needs_revert_confirmation(true));
    }

    // Validates: Requirement 5 AC 5.5 — no confirmation when clean
    #[test]
    fn no_confirmation_when_clean() {
        assert!(!needs_revert_confirmation(false));
    }
}
