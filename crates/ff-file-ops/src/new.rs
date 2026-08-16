//! New file command implementation.
//!
//! Handles `file.new` — create a new empty document.

use crate::traits::UntitledCounter;

/// Result of a new file operation.
#[derive(Debug, Clone)]
pub struct NewFileResult {
    /// The assigned untitled name (e.g., "Untitled-1").
    pub display_name: String,
    /// Status message for the status bar.
    pub status_message: String,
}

/// Create a new empty document with a sequential untitled name.
///
/// The caller is responsible for:
/// - Running the unsaved-changes guard on the active document
/// - Creating the actual tab/document
/// - Emitting the `file.new_created` event
///
/// Addresses: Requirement 3 AC 3.1, 3.7, 3.8
pub fn create_new_file(counter: &mut UntitledCounter) -> NewFileResult {
    let display_name = counter.next_name();
    NewFileResult {
        display_name,
        status_message: "New file".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3 AC 3.8 — sequential untitled naming
    #[test]
    fn create_new_file_assigns_sequential_names() {
        let mut counter = UntitledCounter::new();

        let result1 = create_new_file(&mut counter);
        assert_eq!(result1.display_name, "Untitled-1");

        let result2 = create_new_file(&mut counter);
        assert_eq!(result2.display_name, "Untitled-2");

        let result3 = create_new_file(&mut counter);
        assert_eq!(result3.display_name, "Untitled-3");
    }

    // Validates: Requirement 3 AC 3.7 — status message
    #[test]
    fn create_new_file_returns_status_message() {
        let mut counter = UntitledCounter::new();
        let result = create_new_file(&mut counter);
        assert_eq!(result.status_message, "New file");
    }
}
