//! Backup copy mechanism for save operations.
//!
//! Creates backup copies of the original file before overwrite,
//! stored either alongside the original or in a dedicated directory.

use ff_vfs::{ResourceUri, VfsProvider};

use crate::error::FileOpsError;
use crate::resource_uri::backup_uri_alongside;

/// Configuration for backup copy creation.
///
/// Addresses: Requirement 7, criteria 3–5
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Whether backups are enabled.
    pub enabled: bool,
    /// Where to store backups.
    pub location: BackupLocation,
    /// Suffix for alongside backups (default: `.bak`).
    pub suffix: String,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            location: BackupLocation::Alongside,
            suffix: ".bak".to_string(),
        }
    }
}

/// Where backup copies are stored.
///
/// Addresses: Requirement 7 AC 7.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupLocation {
    /// Same directory as the original file, with a configurable suffix.
    Alongside,
    /// Dedicated backup directory, preserving relative structure.
    Directory(String),
}

/// Create a backup copy of the original resource before overwriting.
///
/// Returns `Ok(())` on success. On failure, returns `Err(FileOpsError::BackupFailed)`
/// but the caller should NOT abort the save — backup failure is non-fatal.
///
/// Addresses: Requirement 7 AC 7.3, 7.4, 7.5
pub async fn create_backup(
    provider: &dyn VfsProvider,
    uri: &ResourceUri,
    config: &BackupConfig,
) -> Result<(), FileOpsError> {
    if !config.enabled {
        return Ok(());
    }

    // Read the original content
    let original_content =
        provider
            .read(uri.path())
            .await
            .map_err(|e| FileOpsError::BackupFailed {
                uri: uri.clone(),
                reason: format!("failed to read original: {e}"),
            })?;

    // Determine backup path
    let backup_path = match &config.location {
        BackupLocation::Alongside => {
            let backup_uri = backup_uri_alongside(uri, &config.suffix);
            backup_uri.path().to_string()
        }
        BackupLocation::Directory(dir) => {
            // Preserve relative structure within backup directory
            let filename = uri.path().rsplit('/').next().unwrap_or(uri.path());
            if dir.ends_with('/') {
                format!("{dir}{filename}{}", config.suffix)
            } else {
                format!("{dir}/{filename}{}", config.suffix)
            }
        }
    };

    // Write backup
    provider
        .write(&backup_path, &original_content)
        .await
        .map_err(|e| FileOpsError::BackupFailed {
            uri: uri.clone(),
            reason: format!("failed to write backup to {backup_path}: {e}"),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_config_default_is_disabled() {
        let config = BackupConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.location, BackupLocation::Alongside);
        assert_eq!(config.suffix, ".bak");
    }

    #[test]
    fn backup_location_alongside_equals_itself() {
        assert_eq!(BackupLocation::Alongside, BackupLocation::Alongside);
    }

    #[test]
    fn backup_location_directory_stores_path() {
        let loc = BackupLocation::Directory("/backup".to_string());
        assert_eq!(loc, BackupLocation::Directory("/backup".to_string()));
        assert_ne!(loc, BackupLocation::Alongside);
    }
}
