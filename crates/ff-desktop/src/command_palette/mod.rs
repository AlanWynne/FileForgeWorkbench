//! Command Palette -- fuzzy-search overlay for all registered commands.
//!
//! Activated by Ctrl+Shift+P. Reads from the shell's CommandRegistry and
//! dispatches via handle_command().
//!
//! Validates: Requirement 1-5 (command-palette)

pub mod fuzzy;
pub mod render;
pub mod state;

pub use state::CommandPaletteState;
