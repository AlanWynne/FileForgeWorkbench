//! Event hook system — registration, discovery, and dispatch.
//!
//! Addresses: Requirement 3 (all criteria)

pub mod event;
pub mod registry;

pub use event::HookEvent;
pub use registry::{HookDispatchResult, HookHandler, HookRegistry};
