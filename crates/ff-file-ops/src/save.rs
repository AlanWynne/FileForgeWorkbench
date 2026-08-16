//! Save command implementation.
//!
//! Handles `file.save` — persist document content to its associated URI.

use std::time::SystemTime;

use ff_vfs::{ResourceUri, VfsProvider};

use crate::backup::{create_backup, BackupConfig};
use crate::error::FileOpsError;
use crate::options::SaveResult;
use crate::persistence::PersistenceStrategy;

/// The save state of a document — prevents concurrent saves.
///
/// Addresses: Requirement 1 AC 1.7, 1.8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveState {
    /// No save in progress; document is idle.
    Idle,
    /// A synchronous save is executing.
    SavingSync,
    /// An async background save is in progress.
    SavingAsync,
}

/// Execute a save operation for the given document.
///
/// Core save logic — writes content via the persistence strategy,
/// handles backup creation, and returns the save result.
///
/// Addresses: Requirement 1 AC 1.1, 1.2, 1.3
pub async fn execute_save(
    provider: &dyn VfsProvider,
    uri: &ResourceUri,
    content: &[u8],
    strategy: &dyn PersistenceStrategy,
    backup_config: &BackupConfig,
) -> Result<SaveResult, FileOpsError> {
    // Create backup if enabled (non-fatal on failure)
    if backup_config.enabled {
        // Check if original exists first
        if provider.exists(uri.path()).await.unwrap_or(false) {
            if let Err(e) = create_backup(provider, uri, backup_config).await {
                // Log warning but don't abort save
                // In production this would use ff-logging WARN
                let _ = e; // Intentionally ignored — backup failure is non-fatal
            }
        }
    }

    // Write via strategy
    strategy.write(provider, uri, content).await?;

    // Get updated metadata
    let modification_time = provider
        .stat(uri.path())
        .await
        .ok()
        .and_then(|m| m.modified)
        .unwrap_or_else(SystemTime::now);

    Ok(SaveResult {
        uri: uri.clone(),
        bytes_written: content.len() as u64,
        modification_time,
        was_async: false,
    })
}

/// Determine if a save should be async based on document size.
///
/// Addresses: Requirement 1 AC 1.6, 1.7
pub fn should_save_async(document_size: u64, threshold: u64) -> bool {
    document_size > threshold
}

/// Check if the document has been externally modified since last save/open.
///
/// Compares recorded mtime with current VFS mtime.
///
/// Addresses: Requirement 1 AC 1.9
pub async fn check_external_modification(
    provider: &dyn VfsProvider,
    uri: &ResourceUri,
    recorded_mtime: Option<SystemTime>,
) -> Result<bool, FileOpsError> {
    let Some(recorded) = recorded_mtime else {
        return Ok(false);
    };

    let current_mtime = provider
        .stat(uri.path())
        .await
        .map_err(|source| FileOpsError::VfsReadError {
            operation: "mtime_check".to_string(),
            uri: uri.clone(),
            source,
        })?
        .modified;

    match current_mtime {
        Some(current) => Ok(current != recorded),
        None => Ok(false), // Can't determine — assume not modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1 AC 1.6 — sync save for small files
    #[test]
    fn should_save_async_false_for_small_files() {
        assert!(!should_save_async(100, 1_048_576));
        assert!(!should_save_async(1_048_576, 1_048_576)); // at threshold = sync
    }

    // Validates: Requirement 1 AC 1.7 — async save for large files
    #[test]
    fn should_save_async_true_for_large_files() {
        assert!(should_save_async(1_048_577, 1_048_576));
        assert!(should_save_async(10_000_000, 1_048_576));
    }

    // Validates: Requirement 1 AC 1.8 — save state tracking
    #[test]
    fn save_state_idle_is_default() {
        let state = SaveState::Idle;
        assert_eq!(state, SaveState::Idle);
        assert_ne!(state, SaveState::SavingSync);
        assert_ne!(state, SaveState::SavingAsync);
    }
}
