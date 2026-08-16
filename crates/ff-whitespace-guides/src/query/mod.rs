//! Per-line query functions for whitespace glyphs, indent guides, edge, and wrap markers.

pub mod edge;
pub mod indent_guides;
pub mod whitespace;
pub mod wrap_markers;

pub use edge::compute_edge_indicator;
pub use indent_guides::{
    compute_look_both_guides, compute_look_forward_guides, compute_real_guides,
};
pub use whitespace::compute_whitespace_glyphs;
pub use wrap_markers::{compute_continuation_indent, compute_wrap_markers};
