//! Command executor subsystem.
//!
//! Provides async process spawning, output streaming, timeout management,
//! and signal delivery for shell command execution.

pub mod output;
pub mod signal;
pub mod spawn;
pub mod timeout;

pub use output::OutputCapture;
pub use spawn::CommandExecutor;
pub use timeout::TimeoutGuard;
