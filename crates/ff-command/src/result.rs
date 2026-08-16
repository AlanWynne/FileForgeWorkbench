//! `CommandResult` enum and `UndoRecord` trait.
//!
//! Defines the outcome of command execution and the interface for undo records.

use crate::context::ExecutionContext;
use crate::error::CommandError;
use crate::id::CommandId;
use crate::params::ParamValue;

/// An opaque token encapsulating information needed to reverse a command's effect.
///
/// Implemented by each undoable command. The undo/redo system stores these
/// as trait objects.
pub trait UndoRecord: Send + Sync + std::fmt::Debug {
    /// Apply this record to reverse the original command's effect.
    fn undo(&self, ctx: &ExecutionContext) -> Result<(), CommandError>;

    /// Re-apply the original command's effect (for redo).
    fn redo(&self, ctx: &ExecutionContext) -> Result<(), CommandError>;

    /// Human-readable description for undo/redo history display.
    fn description(&self) -> &str;

    /// The command ID that produced this record.
    fn command_id(&self) -> &CommandId;
}

/// The outcome of a command execution.
///
/// Contains success/failure status, optional return value, and optional
/// undo record for undoable commands.
#[derive(Debug)]
pub enum CommandResult {
    /// Command executed successfully with no return value.
    Ok,

    /// Command executed successfully and produced an undo record.
    OkUndoable {
        /// The undo record for reversing this command.
        undo_record: Box<dyn UndoRecord>,
    },

    /// Command executed successfully with a return value (for scripting bridge).
    OkValue(ParamValue),

    /// Command executed successfully with both a return value and an undo record.
    OkValueUndoable {
        /// The return value.
        value: ParamValue,
        /// The undo record for reversing this command.
        undo_record: Box<dyn UndoRecord>,
    },

    /// Command execution failed.
    Err(CommandError),
}

impl CommandResult {
    /// Returns true if the result indicates success.
    pub fn is_ok(&self) -> bool {
        !matches!(self, Self::Err(_))
    }

    /// Returns true if the result indicates failure.
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }

    /// Extracts the undo record, consuming the result.
    pub fn into_undo_record(self) -> Option<Box<dyn UndoRecord>> {
        match self {
            Self::OkUndoable { undo_record } => Some(undo_record),
            Self::OkValueUndoable { undo_record, .. } => Some(undo_record),
            _ => None,
        }
    }

    /// Returns a reference to the return value, if any.
    pub fn value(&self) -> Option<&ParamValue> {
        match self {
            Self::OkValue(v) => Some(v),
            Self::OkValueUndoable { value, .. } => Some(value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockUndoRecord {
        desc: String,
        cmd_id: CommandId,
    }

    impl UndoRecord for MockUndoRecord {
        fn undo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
            Ok(())
        }

        fn redo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
            Ok(())
        }

        fn description(&self) -> &str {
            &self.desc
        }

        fn command_id(&self) -> &CommandId {
            &self.cmd_id
        }
    }

    // Validates: Requirement 2.1
    #[test]
    fn ok_result_is_ok() {
        let result = CommandResult::Ok;
        assert!(result.is_ok());
        assert!(!result.is_err());
    }

    // Validates: Requirement 2.6
    #[test]
    fn err_result_is_err() {
        let result = CommandResult::Err(CommandError::NotFound {
            id: "test".to_string(),
        });
        assert!(result.is_err());
        assert!(!result.is_ok());
    }

    // Validates: Requirement 4.1
    #[test]
    fn undoable_result_contains_undo_record() {
        let record = MockUndoRecord {
            desc: "test undo".to_string(),
            cmd_id: CommandId::new("edit.insert").unwrap(),
        };
        let result = CommandResult::OkUndoable {
            undo_record: Box::new(record),
        };
        assert!(result.is_ok());
        let undo = result.into_undo_record();
        assert!(undo.is_some());
        assert_eq!(undo.unwrap().description(), "test undo");
    }

    // Validates: Requirement 4.4
    #[test]
    fn err_result_has_no_undo_record() {
        let result = CommandResult::Err(CommandError::ExecutionFailed {
            id: "test".to_string(),
            description: "failed".to_string(),
        });
        let undo = result.into_undo_record();
        assert!(undo.is_none());
    }

    #[test]
    fn ok_value_result_has_value() {
        let result = CommandResult::OkValue(ParamValue::String("hello".to_string()));
        assert_eq!(
            result.value(),
            Some(&ParamValue::String("hello".to_string()))
        );
    }
}
