//! Shell-provided trait abstractions for GUI-independent file operations.
//!
//! These traits define the contracts that the GUI shell (ff-desktop) must
//! implement. This crate provides the logic; the shell provides dialogs.

use ff_vfs::ResourceUri;

use crate::options::FilePickerOptions;
use crate::unsaved_guard::UnsavedChangesAction;

/// Trait abstraction for file picker dialogs.
///
/// The GUI shell (ff-desktop) provides the concrete implementation.
/// Addresses: GUI Independence cross-cutting requirement
#[async_trait::async_trait]
pub trait DialogProvider: Send + Sync {
    /// Show a file picker dialog. Returns selected URIs or empty if cancelled.
    async fn show_file_picker(&self, options: &FilePickerOptions) -> Vec<ResourceUri>;

    /// Show the unsaved changes dialog. Returns the user's chosen action.
    async fn show_unsaved_changes(&self, document_name: &str) -> UnsavedChangesAction;

    /// Show an overwrite confirmation dialog. Returns true if user confirms.
    async fn show_overwrite_confirmation(&self, uri: &ResourceUri) -> bool;

    /// Show an external modification confirmation dialog.
    async fn show_external_modification_warning(&self, uri: &ResourceUri) -> bool;

    /// Show an error notification to the user.
    async fn show_error_notification(&self, message: &str);

    /// Show a status message in the status bar.
    async fn show_status_message(&self, message: &str);
}

/// Trait for accessing document state needed by file operations.
///
/// This abstracts over the concrete document model to keep file-ops
/// decoupled from the full document model internals.
#[async_trait::async_trait]
pub trait DocumentAccess: Send + Sync {
    /// Get the resource URI associated with this document, if any.
    fn uri(&self) -> Option<&ResourceUri>;

    /// Set the resource URI for this document.
    fn set_uri(&mut self, uri: Option<ResourceUri>);

    /// Get the document's display name (filename or "Untitled-N").
    fn display_name(&self) -> &str;

    /// Whether the document has unsaved modifications.
    fn is_dirty(&self) -> bool;

    /// Get the full document content as bytes.
    fn content_bytes(&self) -> Vec<u8>;

    /// Replace the document content entirely (for revert).
    fn replace_content(&mut self, content: &[u8]);

    /// Get the document size in bytes.
    fn size_bytes(&self) -> u64;

    /// Whether the document is marked read-only.
    fn is_read_only(&self) -> bool;

    /// Set the read-only status.
    fn set_read_only(&mut self, read_only: bool);

    /// Get the recorded modification time (from last open/save).
    fn recorded_mtime(&self) -> Option<std::time::SystemTime>;

    /// Set the recorded modification time.
    fn set_recorded_mtime(&mut self, mtime: Option<std::time::SystemTime>);

    /// Mark the save point (clears dirty flag).
    fn mark_save_point(&mut self);

    /// Clear undo/redo stacks (for revert).
    fn clear_undo_history(&mut self);
}

/// Trait for managing tabs in the editor.
///
/// Abstracts tab creation and activation to keep file-ops GUI-independent.
pub trait TabManager: Send + Sync {
    /// Check if a URI is already open in an existing tab.
    fn find_tab_by_uri(&self, uri: &ResourceUri) -> Option<usize>;

    /// Activate an existing tab by index.
    fn activate_tab(&mut self, index: usize);

    /// Get the currently active document (mutable).
    fn active_document_mut(&mut self) -> Option<&mut dyn DocumentAccess>;

    /// Get the currently active document (immutable).
    fn active_document(&self) -> Option<&dyn DocumentAccess>;
}

/// Trait for emitting events to the command framework event bus.
pub trait EventEmitter: Send + Sync {
    /// Emit a file operation event with an optional URI payload.
    fn emit(&self, event_name: &str, uri: Option<&ResourceUri>);
}

/// Counter for generating sequential untitled document names.
pub struct UntitledCounter {
    next: u32,
}

impl UntitledCounter {
    /// Create a new counter starting at 1.
    pub fn new() -> Self {
        Self { next: 1 }
    }

    /// Generate the next untitled name (e.g., "Untitled-1").
    pub fn next_name(&mut self) -> String {
        let name = format!("Untitled-{}", self.next);
        self.next += 1;
        name
    }

    /// Get the current counter value without incrementing.
    pub fn current(&self) -> u32 {
        self.next
    }
}

impl Default for UntitledCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untitled_counter_starts_at_one() {
        let mut counter = UntitledCounter::new();
        assert_eq!(counter.next_name(), "Untitled-1");
    }

    #[test]
    fn untitled_counter_increments_sequentially() {
        let mut counter = UntitledCounter::new();
        assert_eq!(counter.next_name(), "Untitled-1");
        assert_eq!(counter.next_name(), "Untitled-2");
        assert_eq!(counter.next_name(), "Untitled-3");
    }

    #[test]
    fn untitled_counter_current_does_not_increment() {
        let counter = UntitledCounter::new();
        assert_eq!(counter.current(), 1);
        assert_eq!(counter.current(), 1);
    }

    #[test]
    fn untitled_counter_default_same_as_new() {
        let counter = UntitledCounter::default();
        assert_eq!(counter.current(), 1);
    }
}
