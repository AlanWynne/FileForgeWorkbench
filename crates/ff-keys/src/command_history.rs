//! Command History -- bounded, deduplicated, ordered ring.
//!
//! CR-NR-084 (Option B): the recall ring now LIVES in the command-processor
//! crate (`ff_command::command_line_history`). This module re-exports it under
//! the historic `ff_keys` names so existing consumers (`history_store`, the
//! shell tests) keep compiling unchanged. Behaviour is identical; only ownership
//! moved into the command processor.

pub use ff_command::{CommandLineEntry as HistoryEntry, CommandLineRing as CommandHistory};
