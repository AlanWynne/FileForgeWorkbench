//! `CompletionContext` — the state snapshot when completion is triggered.

/// Identifies which input field is being completed.
///
/// The completion engine behaves differently depending on the field:
/// - `PrimaryCommand`: offers command names, argument completions
/// - `PrefixArea`: offers line command kinds only
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionField {
    /// The primary command field at the top of the editor panel.
    PrimaryCommand,
    /// The prefix area (line command input) on a specific line.
    PrefixArea,
}

/// The state snapshot at the moment completion is triggered or re-evaluated.
///
/// Passed to providers so they can generate contextually-relevant candidates.
/// The context captures the field being edited, the typed text, cursor position,
/// and any parsed command information for argument-position completion.
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Which input field triggered the completion.
    pub field: CompletionField,
    /// The full text content of the field.
    pub field_text: String,
    /// The cursor position within the field (0-indexed character offset).
    pub cursor_offset: usize,
    /// The prefix being completed — substring from anchor to cursor.
    pub prefix: String,
    /// The anchor offset — start of the prefix within the field.
    pub anchor_offset: usize,
    /// The parsed command name, if in argument position (None if completing command name).
    pub command_name: Option<String>,
    /// The argument index being completed (0 = first arg after command name).
    pub argument_index: Option<usize>,
    /// The Command_ID of the resolved command (if known).
    pub command_id: Option<String>,
}

impl CompletionContext {
    /// Returns true when the cursor is in the command name (first token) position.
    ///
    /// This is the case when no command has been parsed yet — the user is
    /// typing the command name itself.
    pub fn is_command_position(&self) -> bool {
        self.command_name.is_none() && self.field == CompletionField::PrimaryCommand
    }

    /// Returns true when the cursor is in an argument position (after the command name).
    ///
    /// This means a command has been identified and the user is now completing
    /// one of its arguments.
    pub fn is_argument_position(&self) -> bool {
        self.command_name.is_some() && self.field == CompletionField::PrimaryCommand
    }

    /// Returns true when the context is for line command prefix area completion.
    pub fn is_line_command_position(&self) -> bool {
        self.field == CompletionField::PrefixArea
    }
}

/// Builder for constructing `CompletionContext` instances, primarily for testing.
#[derive(Debug)]
pub struct CompletionContextBuilder {
    field: CompletionField,
    field_text: String,
    cursor_offset: usize,
    prefix: String,
    anchor_offset: usize,
    command_name: Option<String>,
    argument_index: Option<usize>,
    command_id: Option<String>,
}

impl CompletionContextBuilder {
    /// Creates a new builder with default values for the primary command field.
    pub fn new() -> Self {
        Self {
            field: CompletionField::PrimaryCommand,
            field_text: String::new(),
            cursor_offset: 0,
            prefix: String::new(),
            anchor_offset: 0,
            command_name: None,
            argument_index: None,
            command_id: None,
        }
    }

    /// Sets the completion field.
    pub fn field(mut self, field: CompletionField) -> Self {
        self.field = field;
        self
    }

    /// Sets the full field text.
    pub fn field_text(mut self, text: impl Into<String>) -> Self {
        self.field_text = text.into();
        self
    }

    /// Sets the cursor offset.
    pub fn cursor_offset(mut self, offset: usize) -> Self {
        self.cursor_offset = offset;
        self
    }

    /// Sets the prefix being completed.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Sets the anchor offset.
    pub fn anchor_offset(mut self, offset: usize) -> Self {
        self.anchor_offset = offset;
        self
    }

    /// Sets the command name (indicates argument position).
    pub fn command_name(mut self, name: impl Into<String>) -> Self {
        self.command_name = Some(name.into());
        self
    }

    /// Sets the argument index.
    pub fn argument_index(mut self, index: usize) -> Self {
        self.argument_index = Some(index);
        self
    }

    /// Sets the command ID.
    pub fn command_id(mut self, id: impl Into<String>) -> Self {
        self.command_id = Some(id.into());
        self
    }

    /// Builds the `CompletionContext`.
    pub fn build(self) -> CompletionContext {
        CompletionContext {
            field: self.field,
            field_text: self.field_text,
            cursor_offset: self.cursor_offset,
            prefix: self.prefix,
            anchor_offset: self.anchor_offset,
            command_name: self.command_name,
            argument_index: self.argument_index,
            command_id: self.command_id,
        }
    }
}

impl Default for CompletionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.1 (command position detection)
    #[test]
    fn is_command_position_when_no_command_parsed() {
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .field_text("fi")
            .prefix("fi")
            .cursor_offset(2)
            .build();

        assert!(ctx.is_command_position());
        assert!(!ctx.is_argument_position());
        assert!(!ctx.is_line_command_position());
    }

    // Validates: Requirement 2.1 (argument position detection)
    #[test]
    fn is_argument_position_when_command_is_known() {
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .field_text("FIND pref")
            .prefix("pref")
            .cursor_offset(9)
            .anchor_offset(5)
            .command_name("FIND")
            .argument_index(0)
            .build();

        assert!(!ctx.is_command_position());
        assert!(ctx.is_argument_position());
        assert!(!ctx.is_line_command_position());
    }

    // Validates: Requirement 7.1 (line command position)
    #[test]
    fn is_line_command_position_in_prefix_area() {
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrefixArea)
            .field_text("C")
            .prefix("C")
            .cursor_offset(1)
            .build();

        assert!(!ctx.is_command_position());
        assert!(!ctx.is_argument_position());
        assert!(ctx.is_line_command_position());
    }

    #[test]
    fn builder_defaults_to_primary_command_field() {
        let ctx = CompletionContextBuilder::new().build();
        assert_eq!(ctx.field, CompletionField::PrimaryCommand);
        assert!(ctx.field_text.is_empty());
        assert_eq!(ctx.cursor_offset, 0);
        assert!(ctx.prefix.is_empty());
        assert_eq!(ctx.anchor_offset, 0);
        assert!(ctx.command_name.is_none());
        assert!(ctx.argument_index.is_none());
        assert!(ctx.command_id.is_none());
    }

    #[test]
    fn builder_sets_all_fields() {
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .field_text("EDIT /path/to/file")
            .cursor_offset(18)
            .prefix("/path/to/file")
            .anchor_offset(5)
            .command_name("EDIT")
            .argument_index(0)
            .command_id("edit.open")
            .build();

        assert_eq!(ctx.field, CompletionField::PrimaryCommand);
        assert_eq!(ctx.field_text, "EDIT /path/to/file");
        assert_eq!(ctx.cursor_offset, 18);
        assert_eq!(ctx.prefix, "/path/to/file");
        assert_eq!(ctx.anchor_offset, 5);
        assert_eq!(ctx.command_name.as_deref(), Some("EDIT"));
        assert_eq!(ctx.argument_index, Some(0));
        assert_eq!(ctx.command_id.as_deref(), Some("edit.open"));
    }
}
