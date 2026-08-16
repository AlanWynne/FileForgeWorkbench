//! Status message types for command execution feedback.
//!
//! All status messages are guaranteed to be at most 200 characters.

/// Maximum length for status messages.
const MAX_STATUS_LENGTH: usize = 200;

/// A status message produced by the command engine.
///
/// Guaranteed to be ≤200 characters. Messages exceeding this limit
/// are truncated with a trailing ellipsis ("...").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMessage {
    /// The message text (≤200 characters, truncated with "..." if needed).
    pub text: String,
    /// The severity/kind of this message.
    pub kind: StatusKind,
}

/// Categorisation of status messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    /// Informational success message (e.g., "CHANGE - 3 occurrences changed").
    Info,
    /// Syntax error from parsing (prefix: "Syntax error").
    SyntaxError,
    /// Structural error from command pairing (prefix: "Structure error").
    StructureError,
    /// Runtime error during execution (prefix: "Error").
    RuntimeError,
}

impl StatusMessage {
    /// Create a new status message, truncating to 200 chars if necessary.
    pub fn new(text: impl Into<String>, kind: StatusKind) -> Self {
        let text = Self::truncate(text.into());
        Self { text, kind }
    }

    /// Create an info message.
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(text, StatusKind::Info)
    }

    /// Create a syntax error message identifying the command.
    pub fn syntax_error(command: &str, detail: &str) -> Self {
        let text = format!("Syntax error in {}: {}", command, detail);
        Self::new(text, StatusKind::SyntaxError)
    }

    /// Create a structure error message.
    pub fn structure_error(command: &str, detail: &str) -> Self {
        let text = format!("Structure error in {}: {}", command, detail);
        Self::new(text, StatusKind::StructureError)
    }

    /// Create a runtime error message.
    pub fn runtime_error(command: &str, detail: &str) -> Self {
        let text = format!("Error in {}: {}", command, detail);
        Self::new(text, StatusKind::RuntimeError)
    }

    /// Truncate a string to MAX_STATUS_LENGTH, appending "..." if truncated.
    ///
    /// Uses character-aware truncation to avoid splitting multi-byte UTF-8
    /// sequences. The resulting string is guaranteed to be at most
    /// MAX_STATUS_LENGTH bytes (with the "..." suffix included).
    fn truncate(text: String) -> String {
        if text.len() <= MAX_STATUS_LENGTH {
            text
        } else {
            // Find the largest char boundary at or below MAX_STATUS_LENGTH - 3
            let max_content = MAX_STATUS_LENGTH - 3;
            let boundary = text
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|&i| i <= max_content)
                .last()
                .unwrap_or(0);
            let mut truncated = text[..boundary].to_string();
            truncated.push_str("...");
            truncated
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 5.4
    #[test]
    fn status_message_respects_200_char_limit() {
        let long_text = "a".repeat(300);
        let msg = StatusMessage::info(&long_text);
        assert!(msg.text.len() <= MAX_STATUS_LENGTH);
        assert!(msg.text.ends_with("..."));
    }

    // Validates: Requirement 5.4
    #[test]
    fn status_message_short_text_preserved() {
        let msg = StatusMessage::info("OK");
        assert_eq!(msg.text, "OK");
        assert_eq!(msg.kind, StatusKind::Info);
    }

    // Validates: Requirement 5.1
    #[test]
    fn syntax_error_includes_command_name_and_prefix() {
        let msg = StatusMessage::syntax_error("FIND", "unclosed quote");
        assert!(msg.text.starts_with("Syntax error"));
        assert!(msg.text.contains("FIND"));
        assert!(msg.text.contains("unclosed quote"));
        assert_eq!(msg.kind, StatusKind::SyntaxError);
    }

    // Validates: Requirement 5.2
    #[test]
    fn structure_error_includes_command_name_and_prefix() {
        let msg = StatusMessage::structure_error("CC", "no matching pair");
        assert!(msg.text.starts_with("Structure error"));
        assert!(msg.text.contains("CC"));
        assert_eq!(msg.kind, StatusKind::StructureError);
    }

    // Validates: Requirement 5.3
    #[test]
    fn runtime_error_includes_command_name_and_prefix() {
        let msg = StatusMessage::runtime_error("CHANGE", "line out of range");
        assert!(msg.text.starts_with("Error"));
        assert!(msg.text.contains("CHANGE"));
        assert_eq!(msg.kind, StatusKind::RuntimeError);
    }

    // Validates: Requirement 5.5
    #[test]
    fn error_messages_include_command_name() {
        let msg = StatusMessage::syntax_error("LOCATE", "details");
        assert!(msg.text.contains("LOCATE"));

        let msg = StatusMessage::runtime_error("SORT", "details");
        assert!(msg.text.contains("SORT"));
    }

    // Validates: Requirement 5.6
    #[test]
    fn info_message_for_success() {
        let msg = StatusMessage::info("CHANGE - 3 occurrences changed");
        assert_eq!(msg.kind, StatusKind::Info);
        assert_eq!(msg.text, "CHANGE - 3 occurrences changed");
    }

    // Validates: Requirement 5.4
    #[test]
    fn truncation_adds_ellipsis_at_exactly_200_chars() {
        let text = "x".repeat(201);
        let msg = StatusMessage::info(&text);
        assert_eq!(msg.text.len(), 200);
        assert!(msg.text.ends_with("..."));
    }

    // Validates: Requirement 5.4
    #[test]
    fn exactly_200_chars_not_truncated() {
        let text = "y".repeat(200);
        let msg = StatusMessage::info(&text);
        assert_eq!(msg.text.len(), 200);
        assert!(!msg.text.ends_with("..."));
    }
}
