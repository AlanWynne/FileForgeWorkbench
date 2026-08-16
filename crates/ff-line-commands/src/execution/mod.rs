//! Execution engine — performs document mutations and records transactions.
//!
//! Each submodule handles one line command family.

pub mod bounds_shift;
pub mod copy;
pub mod delete;
pub mod exclude;
pub mod insert;
pub mod move_cmd;
pub mod repeat;
pub mod shift_left;
pub mod shift_right;
pub mod tag;

use ff_display_line_mapping::DisplayLineMapping;
use ff_document_model::Document;
use ff_edit_operations::{EditBounds, EditorTransaction};

use crate::command::ExecutableCommand;
use crate::config::LineCommandConfig;
use crate::error::LineCommandError;

/// Executes resolved line commands against the document model.
///
/// Wraps undoable operations in transactions; session-state operations
/// (exclude, tag) bypass the undo stack.
pub struct ExecutionEngine;

impl ExecutionEngine {
    /// Execute a resolved command against the document.
    ///
    /// Returns `Ok(Some(transaction))` for undoable operations.
    /// Returns `Ok(None)` for session-state operations (exclude, tag/untag).
    /// Returns `Err` on failure (document unchanged).
    pub fn execute(
        command: &ExecutableCommand,
        document: &mut Document,
        display_mapping: &mut dyn DisplayLineMapping,
        _config: &LineCommandConfig,
        bounds: Option<&EditBounds>,
    ) -> Result<Option<EditorTransaction>, LineCommandError> {
        match command {
            ExecutableCommand::Delete { start_line, count } => {
                let txn = delete::execute_delete(document, *start_line, *count)?;
                Ok(Some(txn))
            }
            ExecutableCommand::Insert { after_line, count } => {
                let txn = insert::execute_insert(document, *after_line, *count)?;
                Ok(Some(txn))
            }
            ExecutableCommand::Repeat { start_line, count } => {
                let txn = repeat::execute_repeat(document, *start_line, *count)?;
                Ok(Some(txn))
            }
            ExecutableCommand::RepeatBlock {
                start_line,
                end_line,
            } => {
                let txn = repeat::execute_repeat_block(document, *start_line, *end_line)?;
                Ok(Some(txn))
            }
            ExecutableCommand::CopyToTarget(source_target) => {
                let txn = copy::execute_copy(document, source_target)?;
                Ok(Some(txn))
            }
            ExecutableCommand::MoveToTarget(source_target) => {
                let txn = move_cmd::execute_move(document, source_target)?;
                Ok(Some(txn))
            }
            ExecutableCommand::Exclude { start_line, count } => {
                exclude::execute_exclude(display_mapping, *start_line, *count)?;
                Ok(None)
            }
            ExecutableCommand::Tag {
                start_line,
                end_line,
            } => {
                tag::execute_tag(document, *start_line, *end_line)?;
                Ok(None)
            }
            ExecutableCommand::Untag {
                start_line,
                end_line,
            } => {
                tag::execute_untag(document, *start_line, *end_line)?;
                Ok(None)
            }
            ExecutableCommand::ShiftRight {
                start_line,
                end_line,
                columns,
            } => {
                let txn =
                    shift_right::execute_shift_right(document, *start_line, *end_line, *columns)?;
                Ok(Some(txn))
            }
            ExecutableCommand::ShiftLeft {
                start_line,
                end_line,
                columns,
            } => {
                let txn =
                    shift_left::execute_shift_left(document, *start_line, *end_line, *columns)?;
                Ok(Some(txn))
            }
            ExecutableCommand::BoundsShiftRight {
                start_line,
                end_line,
            } => {
                let b = bounds.ok_or(LineCommandError::NoBoundsActive)?;
                let txn =
                    bounds_shift::execute_bounds_shift_right(document, *start_line, *end_line, b)?;
                Ok(Some(txn))
            }
            ExecutableCommand::BoundsShiftLeft {
                start_line,
                end_line,
            } => {
                let b = bounds.ok_or(LineCommandError::NoBoundsActive)?;
                let txn =
                    bounds_shift::execute_bounds_shift_left(document, *start_line, *end_line, b)?;
                Ok(Some(txn))
            }
        }
    }
}
