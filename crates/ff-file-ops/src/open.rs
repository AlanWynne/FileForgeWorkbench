//! Open file command implementation.
//!
//! Handles `file.open` — load a resource from the VFS into a new document/tab.

use std::time::SystemTime;

use ff_vfs::{ResourceUri, VfsCapabilities, VfsProvider};

use crate::error::FileOpsError;
use crate::options::FileOpenOptions;
use crate::read_only::ReadOnlyStatus;

/// Result of an open operation.
#[derive(Debug, Clone)]
pub struct OpenResult {
    /// The URI that was opened.
    pub uri: ResourceUri,
    /// The loaded content.
    pub content: Vec<u8>,
    /// Whether the resource is read-only.
    pub read_only_status: ReadOnlyStatus,
    /// Modification time from VFS stat.
    pub modification_time: Option<SystemTime>,
    /// File size in bytes.
    pub size_bytes: u64,
}

/// Load a resource from the VFS.
///
/// This is the core open logic — it reads the file and determines
/// read-only status. Tab management and dialog interaction are
/// handled by the caller.
///
/// Addresses: Requirement 4 AC 4.2, 4.7, 4.8
pub async fn load_resource(
    provider: &dyn VfsProvider,
    uri: &ResourceUri,
    options: &FileOpenOptions,
) -> Result<OpenResult, FileOpsError> {
    let path = uri.path();

    // Read content
    let content = provider
        .read(path)
        .await
        .map_err(|source| FileOpsError::VfsReadError {
            operation: "open".to_string(),
            uri: uri.clone(),
            source,
        })?;

    // Get metadata for mtime and size
    let metadata = provider.stat(path).await.ok();
    let modification_time = metadata.as_ref().and_then(|m| m.modified);
    let size_bytes = metadata
        .as_ref()
        .and_then(|m| m.size)
        .unwrap_or(content.len() as u64);

    // Determine read-only status
    let read_only_status = determine_read_only_status(provider.capabilities(), options);

    Ok(OpenResult {
        uri: uri.clone(),
        content,
        read_only_status,
        modification_time,
        size_bytes,
    })
}

/// Determine the read-only status of a resource based on provider
/// capabilities and open options.
///
/// Addresses: Requirement 4 AC 4.7, Requirement 8 AC 8.1
pub fn determine_read_only_status(
    capabilities: VfsCapabilities,
    options: &FileOpenOptions,
) -> ReadOnlyStatus {
    // User force override
    if let Some(true) = options.read_only_override {
        return ReadOnlyStatus::UserToggled;
    }

    // Provider capability check
    if !capabilities.write {
        return ReadOnlyStatus::ProviderLacksWrite;
    }

    ReadOnlyStatus::Writable
}

/// Check if a URI is already open (for duplicate detection).
///
/// Addresses: Requirement 4 AC 4.5
pub fn is_duplicate_open(open_uris: &[ResourceUri], uri: &ResourceUri) -> bool {
    open_uris.iter().any(|existing| existing == uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4 AC 4.7 — read-only detection from capabilities
    #[test]
    fn determine_read_only_with_writable_provider() {
        let caps = VfsCapabilities::all();
        let options = FileOpenOptions::default();
        let status = determine_read_only_status(caps, &options);
        assert_eq!(status, ReadOnlyStatus::Writable);
    }

    // Validates: Requirement 8 AC 8.7 — provider lacks write
    #[test]
    fn determine_read_only_with_non_writable_provider() {
        let mut caps = VfsCapabilities::all();
        caps.write = false;
        let options = FileOpenOptions::default();
        let status = determine_read_only_status(caps, &options);
        assert_eq!(status, ReadOnlyStatus::ProviderLacksWrite);
    }

    // Validates: Requirement 8 AC 8.5 — user toggle override
    #[test]
    fn determine_read_only_with_user_override() {
        let caps = VfsCapabilities::all();
        let options = FileOpenOptions {
            read_only_override: Some(true),
            ..Default::default()
        };
        let status = determine_read_only_status(caps, &options);
        assert_eq!(status, ReadOnlyStatus::UserToggled);
    }

    // Validates: Requirement 4 AC 4.5 — duplicate detection
    #[test]
    fn is_duplicate_detects_existing_uri() {
        let uri1 = ResourceUri::new("local", "/file1.txt");
        let uri2 = ResourceUri::new("local", "/file2.txt");
        let open = vec![uri1.clone(), uri2.clone()];

        assert!(is_duplicate_open(&open, &uri1));
        assert!(is_duplicate_open(&open, &uri2));
    }

    // Validates: Requirement 4 AC 4.5 — no false duplicate
    #[test]
    fn is_duplicate_returns_false_for_new_uri() {
        let uri1 = ResourceUri::new("local", "/file1.txt");
        let open = vec![uri1];
        let new_uri = ResourceUri::new("local", "/file3.txt");

        assert!(!is_duplicate_open(&open, &new_uri));
    }

    // Validates: Requirement 4 AC 4.10 — multi-select
    #[test]
    fn is_duplicate_with_empty_list() {
        let uri = ResourceUri::new("local", "/new.txt");
        assert!(!is_duplicate_open(&[], &uri));
    }
}
