//! Scrollbar models (vertical and horizontal).
//!
//! Pure-function computation of scrollbar positions, thumb sizes, and
//! fraction-to-position mappings. No GUI dependency.

pub mod horizontal;
pub mod vertical;

pub use horizontal::HorizontalScrollbar;
pub use vertical::VerticalScrollbar;
