//! Mode enums for the whitespace-and-guides subsystem.
//!
//! Each enum represents a configurable mode stored in the `editor.*`
//! configuration namespace.

mod edge_mode;
mod indent_guide_mode;
mod tab_draw_mode;
mod whitespace_visibility;
mod wrap_indent_mode;
mod wrap_visual_flag;
mod wrap_visual_location;

pub use edge_mode::EdgeMode;
pub use indent_guide_mode::IndentGuideMode;
pub use tab_draw_mode::TabDrawMode;
pub use whitespace_visibility::WhitespaceVisibility;
pub use wrap_indent_mode::WrapIndentMode;
pub use wrap_visual_flag::WrapVisualFlag;
pub use wrap_visual_location::WrapVisualLocation;
