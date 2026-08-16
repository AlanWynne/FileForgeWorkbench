//! Command handlers for TABS and MASK primary and line commands.
//!
//! This module provides the command execution logic for all TABS/MASK operations.

pub mod line_commands;
pub mod mask;
pub mod reset_tabs;
pub mod tabs;

pub use line_commands::execute_line_command;
pub use mask::{execute_mask_command, MaskCommandResult};
pub use reset_tabs::execute_reset_tabs;
pub use tabs::{execute_tabs_command, TabsCommandResult};
