//! Per-invocation execution state.
//!
//! Maintains LASTCC, MAXCC, message buffer, and line counter for a single
//! IDCAMS invocation. Not shared across invocations.

use super::IdcamsResult;
use crate::messages::{ConditionCode, IdcamsMessage, MessageCode};

/// Per-invocation execution state.
#[derive(Debug, Clone)]
pub struct ExecutionState {
    /// The condition code of the most recently executed command.
    pub lastcc: ConditionCode,
    /// The highest condition code encountered in this invocation.
    pub maxcc: ConditionCode,
    /// Messages generated during execution.
    pub messages: Vec<IdcamsMessage>,
    /// Sequential line counter for output.
    pub line_counter: u32,
}

impl ExecutionState {
    /// Creates a new execution state with all codes at 0.
    pub fn new() -> Self {
        Self {
            lastcc: ConditionCode::Success,
            maxcc: ConditionCode::Success,
            messages: Vec::new(),
            line_counter: 0,
        }
    }

    /// Sets LASTCC and updates MAXCC (MAXCC never decreases).
    pub fn set_lastcc(&mut self, cc: ConditionCode) {
        self.lastcc = cc;
        if cc > self.maxcc {
            self.maxcc = cc;
        }
    }

    /// Directly sets MAXCC (for SET MAXCC command).
    pub fn set_maxcc(&mut self, cc: ConditionCode) {
        self.maxcc = cc;
    }

    /// Emits a message to the output stream.
    pub fn emit_message(&mut self, code: MessageCode, text: &str) {
        self.line_counter += 1;
        self.messages
            .push(IdcamsMessage::new(code, text, self.line_counter));
    }

    /// Converts the execution state into the final result.
    pub fn to_result(&self) -> IdcamsResult {
        IdcamsResult {
            lastcc: self.lastcc,
            maxcc: self.maxcc,
            messages: self.messages.clone(),
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}
