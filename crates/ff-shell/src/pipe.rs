//! Stdin piping from document content.
//!
//! Delivers document content (full or selection) to a child process's stdin,
//! then closes the handle to signal EOF.

/// Handles stdin piping from document content to child processes.
///
/// Supports piping full document content or selected text, with proper
/// EOF signalling after all content is written.
#[derive(Debug)]
pub struct StdinPiper;

impl StdinPiper {
    /// Prepares the stdin content for piping.
    ///
    /// If a selection is active, pipes only the selected text.
    /// Otherwise, pipes the full document content.
    /// Empty documents result in immediate EOF (empty string).
    pub fn prepare_content(document_content: &str, selection: Option<&str>) -> String {
        match selection {
            Some(selected) => selected.to_string(),
            None => document_content.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 14.1
    #[test]
    fn prepare_content_uses_full_document_when_no_selection() {
        let content = StdinPiper::prepare_content("full document\ncontent", None);
        assert_eq!(content, "full document\ncontent");
    }

    // Validates: Requirement 14.2
    #[test]
    fn prepare_content_uses_selection_when_active() {
        let content = StdinPiper::prepare_content("full document\ncontent", Some("selected text"));
        assert_eq!(content, "selected text");
    }

    // Validates: Requirement 14.6
    #[test]
    fn prepare_content_handles_empty_document() {
        let content = StdinPiper::prepare_content("", None);
        assert_eq!(content, "");
    }

    // Validates: Requirement 14.2
    #[test]
    fn prepare_content_handles_empty_selection() {
        let content = StdinPiper::prepare_content("document", Some(""));
        assert_eq!(content, "");
    }
}
