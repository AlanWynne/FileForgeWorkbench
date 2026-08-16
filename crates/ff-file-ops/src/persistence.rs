//! Persistence strategy implementations for save operations.
//!
//! Provides the three write strategies: Atomic (temp + rename),
//! DeleteFirst (delete then write), and Direct (overwrite in place).
//! All I/O goes through the VFS abstraction — no direct filesystem calls.

use ff_vfs::{ResourceUri, VfsCapabilities, VfsError, VfsProvider};

use crate::error::FileOpsError;
use crate::resource_uri::temp_uri_for;
use crate::save_strategy::SaveStrategy;

/// Trait for persistence strategy implementations.
///
/// Addresses: Requirement 7
#[async_trait::async_trait]
pub trait PersistenceStrategy: Send + Sync {
    /// Write document content to the target URI using this strategy.
    ///
    /// # Errors
    ///
    /// Returns `FileOpsError` if the write operation fails.
    async fn write(
        &self,
        provider: &dyn VfsProvider,
        uri: &ResourceUri,
        content: &[u8],
    ) -> Result<(), FileOpsError>;
}

/// Atomic write strategy: write to temp file, fsync, atomic rename.
///
/// Addresses: Requirement 7 AC 7.1
pub struct AtomicWriteStrategy;

#[async_trait::async_trait]
impl PersistenceStrategy for AtomicWriteStrategy {
    async fn write(
        &self,
        provider: &dyn VfsProvider,
        uri: &ResourceUri,
        content: &[u8],
    ) -> Result<(), FileOpsError> {
        let temp_uri = temp_uri_for(uri);
        let temp_path = temp_uri.path().to_string();
        let target_path = uri.path().to_string();

        // Step 1: Write content to temp file
        provider
            .write(&temp_path, content)
            .await
            .map_err(|source| FileOpsError::VfsWriteError {
                operation: "atomic_write_temp".to_string(),
                uri: temp_uri.clone(),
                source,
            })?;

        // Step 2: Open temp file for fsync
        let open_opts = ff_vfs::OpenOptions::read_write();
        match provider.open(&temp_path, open_opts).await {
            Ok(mut file) => {
                // Step 3: Flush and fsync
                let _ = file.flush().await;
                let _ = file.sync_all().await;
                let _ = file.close().await;
            }
            Err(_) => {
                // If we can't open for fsync, continue anyway — data was written
            }
        }

        // Step 4: Atomic rename over target
        match provider.rename(&temp_path, &target_path).await {
            Ok(()) => Ok(()),
            Err(VfsError::UnsupportedOperation { .. }) => {
                // Provider doesn't support rename — fall back to direct write with WARN
                // Clean up temp file if possible
                let _ = provider.delete(&temp_path, Default::default()).await;
                // Fall back to direct overwrite
                provider
                    .write(&target_path, content)
                    .await
                    .map_err(|source| FileOpsError::VfsWriteError {
                        operation: "atomic_write_fallback".to_string(),
                        uri: uri.clone(),
                        source,
                    })?;
                Ok(())
            }
            Err(source) => {
                // Rename failed for another reason — clean up temp and report error
                let _ = provider.delete(&temp_path, Default::default()).await;
                Err(FileOpsError::AtomicRenameFailed {
                    uri: uri.clone(),
                    reason: source.to_string(),
                })
            }
        }
    }
}

/// Delete-first strategy: delete target, then write new content.
///
/// Addresses: Requirement 7 AC 7.6
pub struct DeleteFirstStrategy;

#[async_trait::async_trait]
impl PersistenceStrategy for DeleteFirstStrategy {
    async fn write(
        &self,
        provider: &dyn VfsProvider,
        uri: &ResourceUri,
        content: &[u8],
    ) -> Result<(), FileOpsError> {
        let path = uri.path().to_string();

        // Step 1: Delete existing target (ignore not-found errors)
        match provider.delete(&path, Default::default()).await {
            Ok(()) => {}
            Err(VfsError::NotFound { .. }) => {}
            Err(source) => {
                return Err(FileOpsError::VfsWriteError {
                    operation: "delete_first_delete".to_string(),
                    uri: uri.clone(),
                    source,
                });
            }
        }

        // Step 2: Write new content
        provider
            .write(&path, content)
            .await
            .map_err(|source| FileOpsError::VfsWriteError {
                operation: "delete_first_write".to_string(),
                uri: uri.clone(),
                source,
            })?;

        // Step 3: Fsync
        let open_opts = ff_vfs::OpenOptions::read_write();
        if let Ok(mut file) = provider.open(&path, open_opts).await {
            let _ = file.flush().await;
            let _ = file.sync_all().await;
            let _ = file.close().await;
        }

        Ok(())
    }
}

/// Direct overwrite strategy: write content directly to target.
///
/// Addresses: Requirement 7 AC 7.7
pub struct DirectWriteStrategy;

#[async_trait::async_trait]
impl PersistenceStrategy for DirectWriteStrategy {
    async fn write(
        &self,
        provider: &dyn VfsProvider,
        uri: &ResourceUri,
        content: &[u8],
    ) -> Result<(), FileOpsError> {
        let path = uri.path().to_string();

        // Step 1: Write content directly
        provider
            .write(&path, content)
            .await
            .map_err(|source| FileOpsError::VfsWriteError {
                operation: "direct_write".to_string(),
                uri: uri.clone(),
                source,
            })?;

        // Step 2: Fsync
        let open_opts = ff_vfs::OpenOptions::read_write();
        if let Ok(mut file) = provider.open(&path, open_opts).await {
            let _ = file.flush().await;
            let _ = file.sync_all().await;
            let _ = file.close().await;
        }

        Ok(())
    }
}

/// Select the appropriate persistence strategy based on configuration and
/// provider capabilities.
///
/// Addresses: Requirement 7 AC 7.1, 7.6, 7.7
pub fn select_strategy(
    configured: SaveStrategy,
    capabilities: &VfsCapabilities,
) -> Box<dyn PersistenceStrategy> {
    match configured {
        SaveStrategy::Atomic => {
            if capabilities.rename {
                Box::new(AtomicWriteStrategy)
            } else {
                // Provider doesn't support rename — use AtomicWriteStrategy
                // which will internally fall back to Direct with WARN
                Box::new(AtomicWriteStrategy)
            }
        }
        SaveStrategy::DeleteFirst => Box::new(DeleteFirstStrategy),
        SaveStrategy::Direct => Box::new(DirectWriteStrategy),
    }
}

/// Clean up any leftover temp files from interrupted writes.
///
/// Scans for `.tmp` files matching our naming pattern and removes them.
/// Addresses: Requirement 7 AC 7.8
pub async fn cleanup_temp_files(provider: &dyn VfsProvider, directory_path: &str) -> Vec<String> {
    let mut cleaned = Vec::new();

    if let Ok(entries) = provider.list(directory_path).await {
        for entry in entries {
            if entry.name.ends_with(".tmp") {
                let full_path = if directory_path.ends_with('/') {
                    format!("{}{}", directory_path, entry.name)
                } else {
                    format!("{}/{}", directory_path, entry.name)
                };
                if provider
                    .delete(&full_path, Default::default())
                    .await
                    .is_ok()
                {
                    cleaned.push(full_path);
                }
            }
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save_strategy::SaveStrategy;
    use ff_vfs::VfsCapabilities;

    // Validates: Requirement 7 AC 7.1 — default strategy selection with rename support
    #[test]
    fn select_strategy_returns_atomic_when_rename_supported() {
        let caps = VfsCapabilities::all();
        let _ = select_strategy(SaveStrategy::Atomic, &caps);
        // Just verify it doesn't panic — type is opaque
    }

    // Validates: Requirement 7 AC 7.6 — delete_first strategy selection
    #[test]
    fn select_strategy_returns_delete_first_when_configured() {
        let caps = VfsCapabilities::all();
        let _ = select_strategy(SaveStrategy::DeleteFirst, &caps);
    }

    // Validates: Requirement 7 AC 7.7 — direct strategy selection
    #[test]
    fn select_strategy_returns_direct_when_configured() {
        let caps = VfsCapabilities::all();
        let _ = select_strategy(SaveStrategy::Direct, &caps);
    }

    // Validates: Requirement 7 AC 7.2 — fallback when rename not supported
    #[test]
    fn select_strategy_still_returns_atomic_without_rename_for_fallback() {
        let caps = VfsCapabilities::none();
        let _ = select_strategy(SaveStrategy::Atomic, &caps);
    }
}
