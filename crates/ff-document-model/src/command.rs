//! Command framework integration surface.
//!
//! Defines the `DocumentCommand` trait and basic command structs
//! (InsertCommand, DeleteCommand) for mutation operations routable
//! through the command framework.

use crate::document::Document;
use crate::error::DocumentError;
use crate::types::{BytePosition, DeleteResult, InsertResult};

/// Trait for mutation operations routable through the command framework.
///
/// Downstream crates (ff-command, ff-undo-redo) consume this trait to
/// integrate document mutations with undo recording and command dispatch.
pub trait DocumentCommand: Send + Sync {
    /// Execute the command on the document.
    fn execute(&self, document: &mut Document) -> Result<CommandResult, DocumentError>;

    /// Description of the command for undo history display.
    fn description(&self) -> &str;
}

/// Result of a command execution.
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// An insertion was performed.
    Insert(InsertResult),
    /// A deletion was performed.
    Delete(DeleteResult),
}

/// Command that inserts text at a position.
#[derive(Debug, Clone)]
pub struct InsertCommand {
    /// Position to insert at.
    pub position: BytePosition,
    /// Text to insert.
    pub text: Vec<u8>,
}

impl InsertCommand {
    /// Create a new insert command.
    pub fn new(position: BytePosition, text: Vec<u8>) -> Self {
        Self { position, text }
    }
}

impl DocumentCommand for InsertCommand {
    fn execute(&self, document: &mut Document) -> Result<CommandResult, DocumentError> {
        let result = document.insert(self.position, &self.text)?;
        Ok(CommandResult::Insert(result))
    }

    fn description(&self) -> &str {
        "Insert text"
    }
}

/// Command that deletes bytes at a position.
#[derive(Debug, Clone)]
pub struct DeleteCommand {
    /// Position to delete from.
    pub position: BytePosition,
    /// Number of bytes to delete.
    pub length: u64,
}

impl DeleteCommand {
    /// Create a new delete command.
    pub fn new(position: BytePosition, length: u64) -> Self {
        Self { position, length }
    }
}

impl DocumentCommand for DeleteCommand {
    fn execute(&self, document: &mut Document) -> Result<CommandResult, DocumentError> {
        let result = document.delete(self.position, self.length)?;
        Ok(CommandResult::Delete(result))
    }

    fn description(&self) -> &str {
        "Delete text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_command_executes() {
        let mut doc = Document::new();
        let cmd = InsertCommand::new(BytePosition(0), b"hello".to_vec());
        let result = cmd.execute(&mut doc).unwrap();
        assert!(matches!(result, CommandResult::Insert(_)));
        assert_eq!(doc.length(), 5);
    }

    #[test]
    fn delete_command_executes() {
        let mut doc = Document::new();
        doc.insert(BytePosition(0), b"hello world").unwrap();

        let cmd = DeleteCommand::new(BytePosition(5), 6);
        let result = cmd.execute(&mut doc).unwrap();
        assert!(matches!(result, CommandResult::Delete(_)));
        assert_eq!(doc.length(), 5);
    }

    #[test]
    fn command_on_read_only_fails() {
        let mut doc = Document::new();
        doc.insert(BytePosition(0), b"test").unwrap();
        doc.set_read_only(true);

        let cmd = InsertCommand::new(BytePosition(0), b"x".to_vec());
        assert!(cmd.execute(&mut doc).is_err());
    }

    #[test]
    fn command_descriptions() {
        let insert = InsertCommand::new(BytePosition(0), b"x".to_vec());
        assert_eq!(insert.description(), "Insert text");

        let delete = DeleteCommand::new(BytePosition(0), 1);
        assert_eq!(delete.description(), "Delete text");
    }
}
