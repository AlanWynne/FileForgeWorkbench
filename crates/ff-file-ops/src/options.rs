//! Options structs for file operations.
//!
//! Provides `FileOpenOptions`, `FileSaveOptions`, `SaveResult`,
//! `FilePickerMode`, and `FilePickerOptions` for configuring operations.

use std::time::SystemTime;

use ff_vfs::ResourceUri;

use crate::save_strategy::SaveStrategy;

/// Options for opening a file resource.
///
/// Addresses: Requirement 4 (Open), Requirement 1 AC 1.6, 1.7
#[derive(Debug, Clone)]
pub struct FileOpenOptions {
    /// The resource URI to open. If None, a File_Picker is displayed.
    pub uri: Option<ResourceUri>,
    /// Encoding override (if None, auto-detected from BOM/content).
    pub encoding: Option<String>,
    /// Whether to force read-only mode regardless of VFS capabilities.
    pub read_only_override: Option<bool>,
    /// Whether to activate the tab after opening (default: true).
    pub activate_tab: bool,
}

impl Default for FileOpenOptions {
    fn default() -> Self {
        Self {
            uri: None,
            encoding: None,
            read_only_override: None,
            activate_tab: true,
        }
    }
}

/// Options controlling how a document is persisted.
///
/// Addresses: Requirement 1 (Save), Requirement 7 (Atomic Write)
#[derive(Debug, Clone)]
pub struct FileSaveOptions {
    /// Target URI. If None, uses the document's current URI.
    pub uri: Option<ResourceUri>,
    /// Override the configured save strategy for this operation.
    pub strategy: SaveStrategy,
    /// Whether to create a backup copy before overwriting.
    pub create_backup: bool,
    /// Size threshold (bytes) above which save is async.
    pub async_threshold_bytes: u64,
    /// Whether to check modification time before writing.
    pub check_modified_time: bool,
}

impl Default for FileSaveOptions {
    fn default() -> Self {
        Self {
            uri: None,
            strategy: SaveStrategy::default(),
            create_backup: false,
            async_threshold_bytes: 1_048_576, // 1 MB
            check_modified_time: true,
        }
    }
}

/// Result of a successful save operation.
///
/// Addresses: Requirement 1 AC 1.2, 1.3
#[derive(Debug, Clone)]
pub struct SaveResult {
    /// The resource URI that was written to.
    pub uri: ResourceUri,
    /// Number of bytes written.
    pub bytes_written: u64,
    /// Modification time of the resource after write (from VFS stat).
    pub modification_time: SystemTime,
    /// Whether the save was performed asynchronously.
    pub was_async: bool,
}

/// Mode for file picker dialogs.
///
/// Addresses: Requirement 2 AC 2.1, Requirement 4 AC 4.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePickerMode {
    /// Opening one or more files for reading.
    Open,
    /// Selecting a target location for saving.
    Save,
}

/// Options for configuring a file picker dialog.
///
/// Addresses: Requirement 2 AC 2.1, Requirement 4 AC 4.1
#[derive(Debug, Clone)]
pub struct FilePickerOptions {
    /// The mode of the picker (Open or Save).
    pub mode: FilePickerMode,
    /// Initial directory to display.
    pub initial_directory: Option<String>,
    /// Title for the dialog window.
    pub title: Option<String>,
    /// Whether to allow multi-select (Open mode only).
    pub allow_multi_select: bool,
    /// File type filters (e.g., `("Text Files", "*.txt")`).
    pub filters: Vec<(String, String)>,
}

impl Default for FilePickerOptions {
    fn default() -> Self {
        Self {
            mode: FilePickerMode::Open,
            initial_directory: None,
            title: None,
            allow_multi_select: false,
            filters: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.1 — FileOpenOptions defaults
    #[test]
    fn file_open_options_default_has_no_uri() {
        let opts = FileOpenOptions::default();
        assert!(opts.uri.is_none());
        assert!(opts.encoding.is_none());
        assert!(opts.read_only_override.is_none());
        assert!(opts.activate_tab);
    }

    // Validates: Requirement 2.2 — FileSaveOptions defaults
    #[test]
    fn file_save_options_default_uses_atomic_strategy() {
        let opts = FileSaveOptions::default();
        assert!(opts.uri.is_none());
        assert_eq!(opts.strategy, SaveStrategy::Atomic);
        assert!(!opts.create_backup);
        assert_eq!(opts.async_threshold_bytes, 1_048_576);
        assert!(opts.check_modified_time);
    }

    // Validates: Requirement 2.5 — FilePickerMode variants
    #[test]
    fn file_picker_mode_has_open_and_save() {
        let open = FilePickerMode::Open;
        let save = FilePickerMode::Save;
        assert_ne!(open, save);
    }

    // Validates: Requirement 2.5 — FilePickerOptions defaults
    #[test]
    fn file_picker_options_default() {
        let opts = FilePickerOptions::default();
        assert_eq!(opts.mode, FilePickerMode::Open);
        assert!(opts.initial_directory.is_none());
        assert!(opts.title.is_none());
        assert!(!opts.allow_multi_select);
        assert!(opts.filters.is_empty());
    }

    // Validates: Requirement 2.4 — SaveResult construction
    #[test]
    fn save_result_can_be_constructed() {
        let result = SaveResult {
            uri: ResourceUri::new("local", "/test.txt"),
            bytes_written: 1024,
            modification_time: SystemTime::now(),
            was_async: false,
        };
        assert_eq!(result.bytes_written, 1024);
        assert!(!result.was_async);
    }
}
