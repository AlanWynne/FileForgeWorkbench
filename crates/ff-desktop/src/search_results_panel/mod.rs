//! Global Search Results panel for ff-desktop.
//!
//! Addresses: global-search Requirement 1, 2, 3, 4, 5, 6

pub mod render;
pub mod state;

pub use render::{render, SearchPanelOutcome};
pub use state::SearchResultsPanelState;
