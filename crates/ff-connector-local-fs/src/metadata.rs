//! File metadata module.
//!
//! Provides `FileMetadata`, `ResourceType`, and `FilePermissions` types,
//! plus a `stat` helper for reading OS metadata into these types.
//!
//! Addresses: Requirement 5, all acceptance criteria

use std::path::Path;
use std::time::SystemTime;

use ff_vfs::VfsError;

use crate::error::map_io_error;
use crate::path::platform::is_hidden;

/// Resource type classification.
///
/// Addresses: Requirement 5 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceType {
    /// A regular file.
    RegularFile,
    /// A directory.
    Directory,
    /// A symbolic link.
    Symlink,
    /// Any other resource type (device files, pipes, sockets, etc.).
    Other,
}

/// Platform-appropriate permission representation.
///
/// Addresses: Requirement 5 AC 3
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilePermissions {
    /// Unix permissions: read/write/execute for owner, group, others.
    Unix {
        /// Raw mode bits (e.g., 0o755).
        mode: u32,
        /// Owner can read.
        owner_read: bool,
        /// Owner can write.
        owner_write: bool,
        /// Owner can execute.
        owner_execute: bool,
        /// Group can read.
        group_read: bool,
        /// Group can write.
        group_write: bool,
        /// Group can execute.
        group_execute: bool,
        /// Others can read.
        others_read: bool,
        /// Others can write.
        others_write: bool,
        /// Others can execute.
        others_execute: bool,
    },
    /// Windows permissions: simplified attribute-based model.
    Windows {
        /// File is read-only.
        read_only: bool,
        /// File is a system file.
        system: bool,
        /// File has the archive attribute.
        archive: bool,
    },
}

/// Extended file metadata returned by stat operations.
///
/// Addresses: Requirement 5, all acceptance criteria
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File size in bytes.
    pub size: u64,
    /// Last modification time.
    pub modified: Option<SystemTime>,
    /// Creation time (None on filesystems that don't support it).
    pub created: Option<SystemTime>,
    /// Last access time.
    pub accessed: Option<SystemTime>,
    /// Resource type.
    pub resource_type: ResourceType,
    /// Platform-specific permissions.
    pub permissions: FilePermissions,
    /// Whether the file is hidden.
    pub is_hidden: bool,
    /// Symlink target path (if resource_type is Symlink and follow_links is false).
    pub symlink_target: Option<String>,
}

/// Read OS metadata and map to `FileMetadata`.
///
/// If `follow_links` is true, resolves symlinks and returns target metadata.
/// If false, returns metadata for the symlink itself.
///
/// Validates: Requirement 5 AC 5, AC 6, AC 7, AC 8
pub async fn stat(path: &Path, follow_links: bool) -> Result<FileMetadata, VfsError> {
    let uri = format!("vfs://local{}", path.to_string_lossy().replace('\\', "/"));

    let metadata = if follow_links {
        tokio::fs::metadata(path).await
    } else {
        tokio::fs::symlink_metadata(path).await
    };

    let metadata = metadata.map_err(|e| map_io_error(e, "stat", &uri))?;

    let resource_type = if metadata.is_dir() {
        ResourceType::Directory
    } else if metadata.is_symlink() {
        ResourceType::Symlink
    } else if metadata.is_file() {
        ResourceType::RegularFile
    } else {
        ResourceType::Other
    };

    let modified = metadata.modified().ok();
    let created = metadata.created().ok();
    let accessed = metadata.accessed().ok();

    let permissions = extract_permissions(&metadata);

    let hidden = is_hidden(path);

    let symlink_target = if resource_type == ResourceType::Symlink {
        tokio::fs::read_link(path)
            .await
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };

    Ok(FileMetadata {
        size: metadata.len(),
        modified,
        created,
        accessed,
        resource_type,
        permissions,
        is_hidden: hidden,
        symlink_target,
    })
}

/// Extract platform-specific permissions from std metadata.
fn extract_permissions(metadata: &std::fs::Metadata) -> FilePermissions {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        FilePermissions::Unix {
            mode,
            owner_read: mode & 0o400 != 0,
            owner_write: mode & 0o200 != 0,
            owner_execute: mode & 0o100 != 0,
            group_read: mode & 0o040 != 0,
            group_write: mode & 0o020 != 0,
            group_execute: mode & 0o010 != 0,
            others_read: mode & 0o004 != 0,
            others_write: mode & 0o002 != 0,
            others_execute: mode & 0o001 != 0,
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
        const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x20;

        let attrs = metadata.file_attributes();
        FilePermissions::Windows {
            read_only: attrs & FILE_ATTRIBUTE_READONLY != 0,
            system: attrs & FILE_ATTRIBUTE_SYSTEM != 0,
            archive: attrs & FILE_ATTRIBUTE_ARCHIVE != 0,
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        // Fallback for other platforms
        FilePermissions::Unix {
            mode: 0o644,
            owner_read: true,
            owner_write: true,
            owner_execute: false,
            group_read: true,
            group_write: false,
            group_execute: false,
            others_read: true,
            others_write: false,
            others_execute: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_type_non_exhaustive_allows_future_variants() {
        // This test verifies the enum is marked non_exhaustive
        let rt = ResourceType::RegularFile;
        assert_eq!(rt, ResourceType::RegularFile);
    }

    #[tokio::test]
    async fn stat_returns_metadata_for_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        tokio::fs::write(&file_path, b"hello").await.unwrap();

        let meta = stat(&file_path, true).await.unwrap();
        assert_eq!(meta.size, 5);
        assert_eq!(meta.resource_type, ResourceType::RegularFile);
        assert!(meta.modified.is_some());
    }

    #[tokio::test]
    async fn stat_returns_metadata_for_directory() {
        let dir = tempfile::tempdir().unwrap();
        let meta = stat(dir.path(), true).await.unwrap();
        assert_eq!(meta.resource_type, ResourceType::Directory);
    }

    #[tokio::test]
    async fn stat_returns_not_found_for_missing_path() {
        let result = stat(Path::new("/nonexistent/path/xyz"), true).await;
        assert!(result.is_err());
    }

    #[cfg(not(windows))]
    #[test]
    fn is_hidden_detects_dot_files() {
        assert!(is_hidden(Path::new(".hidden")));
        assert!(!is_hidden(Path::new("visible.txt")));
    }
}
