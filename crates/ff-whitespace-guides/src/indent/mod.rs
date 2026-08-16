//! Indent level computation and blank-line scanning utilities.

pub mod level;
pub mod scan;

pub use level::indent_level_of;
pub use scan::{scan_backward_indent, scan_forward_indent};
