//! RETRIEVE pointer -- single-step backward recall through Command History.
//!
//! CR-NR-084 (Option B): the pointer logic now LIVES in the command-processor
//! crate (`ff_command::command_line_history`). This module re-exports it under
//! the historic `ff_keys` names so existing consumers keep compiling unchanged.
//! Behaviour is identical; only ownership moved into the command processor.

pub use ff_command::{RetrieveResult, RetrieveState};
