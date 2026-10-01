//! Re-export shim for the SCROLL ===> field value model.
//!
//! The `ScrollAmount` model moved into the pure leaf crate `ff-scroll-amount`
//! (CR-NR-098 decomposition Wave 6, Task 20). This module re-exports it so every
//! existing `crate::scroll_amount::ScrollAmount` path keeps resolving unchanged.

pub use ff_scroll_amount::*;
