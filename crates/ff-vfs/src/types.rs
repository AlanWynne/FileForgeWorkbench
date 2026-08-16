//! Core data types for the VFS abstraction layer.
//!
//! Defines entry types, metadata, options structs, capabilities, and write modes
//! used throughout the VFS API surface.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// The type of a VFS entry.
///
/// Represents the kind of resource in a directory listing or metadata query.
/// Marked `#[non_exhaustive]` to allow future extension without breaking downstream code.
///
/// Addresses: Requirement 6 AC 6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VfsEntryType {
    /// A regular file.
    File,
    /// A directory or container.
    Directory,
    /// A symbolic link.
    Symlink,
    /// Any other resource type (provider-specific).
    Other,
}

/// Metadata for a resource.
///
/// Returned by the `stat` operation. Contains size, modification time,
/// entry type, and provider-specific extra metadata.
///
/// Addresses: Requirement 6 AC 4
#[derive(Debug, Clone)]
pub struct VfsMetadata {
    /// Size in bytes (if applicable — `None` for directories on some providers).
    pub size: Option<u64>,
    /// Last modified time (if available from the provider).
    pub modified: Option<SystemTime>,
    /// Type of the resource.
    pub entry_type: VfsEntryType,
    /// Provider-specific metadata as key-value pairs.
    pub extra: HashMap<String, String>,
}

/// An entry in a directory listing.
///
/// Returned by the `list` operation. Contains the entry name, type, and
/// basic metadata available without a full `stat` call.
///
/// Addresses: Requirement 6 AC 1, AC 6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VfsEntry {
    /// Entry name (file or directory name, not the full path).
    pub name: String,
    /// Type of the entry.
    pub entry_type: VfsEntryType,
    /// Size in bytes (if applicable — `None` for directories on some providers).
    pub size: Option<u64>,
    /// Last modified time (if available).
    pub modified: Option<SystemTime>,
}

/// Mode for write operations.
///
/// Determines how the VFS handles existing content when writing.
///
/// Addresses: Requirement 5 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    /// Create a new resource; fail if it already exists.
    Create,
    /// Overwrite the existing resource content.
    Truncate,
    /// Append to the end of the existing resource content.
    Append,
}

/// Options for opening a resource.
///
/// Controls how a resource is opened: for reading, writing, creation, etc.
///
/// Addresses: Requirement 4 AC 2, Requirement 5 AC 1–3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenOptions {
    /// Allow reading from the resource.
    pub read: bool,
    /// Allow writing to the resource.
    pub write: bool,
    /// Create the resource if it does not exist.
    pub create: bool,
    /// Truncate the resource to zero length on open.
    pub truncate: bool,
    /// Open in append mode (writes go to the end).
    pub append: bool,
}

impl OpenOptions {
    /// Open for reading only.
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            create: false,
            truncate: false,
            append: false,
        }
    }

    /// Open for writing only (no create, no truncate).
    pub fn write_only() -> Self {
        Self {
            read: false,
            write: true,
            create: false,
            truncate: false,
            append: false,
        }
    }

    /// Open for both reading and writing.
    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            create: false,
            truncate: false,
            append: false,
        }
    }

    /// Create a new resource; fail if it already exists.
    pub fn create_new() -> Self {
        Self {
            read: false,
            write: true,
            create: true,
            truncate: false,
            append: false,
        }
    }

    /// Open for appending (writes go to the end).
    pub fn append() -> Self {
        Self {
            read: false,
            write: true,
            create: false,
            truncate: false,
            append: true,
        }
    }
}

/// Options for creating a resource or container.
///
/// Addresses: Requirement 6 AC 2
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreateOptions {
    /// Create intermediate parent containers if they don't exist (like `mkdir -p`).
    pub create_parents: bool,
    /// Whether to create a directory/container rather than a file.
    pub is_directory: bool,
}

/// Options for deleting a resource or container.
///
/// Addresses: Requirement 6 AC 3
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeleteOptions {
    /// If `true`, delete the container and all its contents recursively.
    /// If `false` and the container is non-empty, the operation fails.
    pub recursive: bool,
}

/// Capabilities that a provider can declare.
///
/// Consumers query capabilities before invoking operations. If a capability
/// is `false`, invoking the corresponding operation returns
/// `VfsError::UnsupportedOperation`.
///
/// Addresses: Requirement 4 AC 4, AC 5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VfsCapabilities {
    /// Provider supports read operations.
    pub read: bool,
    /// Provider supports write operations.
    pub write: bool,
    /// Provider supports file watching.
    pub watch: bool,
    /// Provider supports content/filename search.
    pub search: bool,
    /// Provider supports random-access reads/writes (seeking).
    pub random_access: bool,
    /// Provider supports append mode.
    pub append: bool,
    /// Provider supports rename/move operations.
    pub rename: bool,
    /// Provider supports delete operations.
    pub delete: bool,
    /// Provider supports listing directory contents.
    pub list: bool,
    /// Provider supports creating directories/containers.
    pub create_directory: bool,
}

impl VfsCapabilities {
    /// Returns capabilities with all fields set to `true`.
    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            watch: true,
            search: true,
            random_access: true,
            append: true,
            rename: true,
            delete: true,
            list: true,
            create_directory: true,
        }
    }

    /// Returns capabilities with all fields set to `false`.
    pub fn none() -> Self {
        Self {
            read: false,
            write: false,
            watch: false,
            search: false,
            random_access: false,
            append: false,
            rename: false,
            delete: false,
            list: false,
            create_directory: false,
        }
    }
}

impl Default for VfsCapabilities {
    fn default() -> Self {
        Self::none()
    }
}

/// Options for watching a resource or directory for changes.
///
/// Addresses: Requirement 7 AC 5
#[derive(Debug, Clone)]
pub struct WatchOptions {
    /// Minimum interval between consecutive events for the same resource.
    pub debounce: Duration,
    /// Whether to watch recursively into subdirectories.
    pub recursive: bool,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self {
            debounce: Duration::from_millis(100),
            recursive: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_options_read_only_has_correct_defaults() {
        // Validates: Requirement 4 AC 2, Requirement 5 AC 1
        let opts = OpenOptions::read_only();
        assert!(opts.read);
        assert!(!opts.write);
        assert!(!opts.create);
        assert!(!opts.truncate);
        assert!(!opts.append);
    }

    #[test]
    fn open_options_write_only_has_correct_defaults() {
        // Validates: Requirement 4 AC 2, Requirement 5 AC 2
        let opts = OpenOptions::write_only();
        assert!(!opts.read);
        assert!(opts.write);
        assert!(!opts.create);
        assert!(!opts.truncate);
        assert!(!opts.append);
    }

    #[test]
    fn open_options_create_new_has_correct_defaults() {
        // Validates: Requirement 5 AC 2
        let opts = OpenOptions::create_new();
        assert!(!opts.read);
        assert!(opts.write);
        assert!(opts.create);
        assert!(!opts.truncate);
        assert!(!opts.append);
    }

    #[test]
    fn open_options_append_has_correct_defaults() {
        // Validates: Requirement 5 AC 2
        let opts = OpenOptions::append();
        assert!(!opts.read);
        assert!(opts.write);
        assert!(!opts.create);
        assert!(!opts.truncate);
        assert!(opts.append);
    }

    #[test]
    fn vfs_capabilities_none_has_all_false() {
        // Validates: Requirement 4 AC 4
        let caps = VfsCapabilities::none();
        assert!(!caps.read);
        assert!(!caps.write);
        assert!(!caps.watch);
        assert!(!caps.search);
        assert!(!caps.random_access);
        assert!(!caps.append);
        assert!(!caps.rename);
        assert!(!caps.delete);
        assert!(!caps.list);
        assert!(!caps.create_directory);
    }

    #[test]
    fn vfs_capabilities_all_has_all_true() {
        // Validates: Requirement 4 AC 4
        let caps = VfsCapabilities::all();
        assert!(caps.read);
        assert!(caps.write);
        assert!(caps.watch);
        assert!(caps.search);
        assert!(caps.random_access);
        assert!(caps.append);
        assert!(caps.rename);
        assert!(caps.delete);
        assert!(caps.list);
        assert!(caps.create_directory);
    }

    #[test]
    fn vfs_capabilities_default_equals_none() {
        // Validates: Requirement 4 AC 4
        assert_eq!(VfsCapabilities::default(), VfsCapabilities::none());
    }

    #[test]
    fn watch_options_default_debounce_is_100ms() {
        // Validates: Requirement 7 AC 5
        let opts = WatchOptions::default();
        assert_eq!(opts.debounce, Duration::from_millis(100));
        assert!(!opts.recursive);
    }

    #[test]
    fn vfs_entry_clone_and_partial_eq() {
        // Validates: Requirement 6 AC 6
        let entry = VfsEntry {
            name: "test.txt".to_string(),
            entry_type: VfsEntryType::File,
            size: Some(1024),
            modified: None,
        };
        let cloned = entry.clone();
        assert_eq!(entry, cloned);

        let different = VfsEntry {
            name: "other.txt".to_string(),
            entry_type: VfsEntryType::Directory,
            size: None,
            modified: None,
        };
        assert_ne!(entry, different);
    }

    #[test]
    fn vfs_entry_type_debug_output() {
        // Validates: Requirement 6 AC 6
        assert_eq!(format!("{:?}", VfsEntryType::File), "File");
        assert_eq!(format!("{:?}", VfsEntryType::Directory), "Directory");
        assert_eq!(format!("{:?}", VfsEntryType::Symlink), "Symlink");
        assert_eq!(format!("{:?}", VfsEntryType::Other), "Other");
    }

    #[test]
    fn create_options_default_values() {
        // Validates: Requirement 6 AC 2
        let opts = CreateOptions::default();
        assert!(!opts.create_parents);
        assert!(!opts.is_directory);
    }

    #[test]
    fn delete_options_default_values() {
        // Validates: Requirement 6 AC 3
        let opts = DeleteOptions::default();
        assert!(!opts.recursive);
    }
}
