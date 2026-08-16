//! Unsaved-changes guard for destructive operations.
//!
//! Presents a Save/Discard/Cancel dialog when an operation would
//! discard unsaved modifications.

/// The user's response to an unsaved-changes dialog.
///
/// Addresses: Requirement 9, criteria 1–8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChangesAction {
    /// Save the document before proceeding with the destructive operation.
    Save,
    /// Discard modifications and proceed immediately.
    Discard,
    /// Cancel the operation entirely; leave document unchanged.
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_variants_are_distinct() {
        assert_ne!(UnsavedChangesAction::Save, UnsavedChangesAction::Discard);
        assert_ne!(UnsavedChangesAction::Save, UnsavedChangesAction::Cancel);
        assert_ne!(UnsavedChangesAction::Discard, UnsavedChangesAction::Cancel);
    }
}
