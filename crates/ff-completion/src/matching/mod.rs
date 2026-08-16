//! Matching algorithms for completion filtering.
//!
//! This module provides prefix and fuzzy matching implementations
//! used by the completion engine to filter candidates against typed text.

pub mod fuzzy;
pub mod prefix;

pub use fuzzy::{fuzzy_match, FuzzyMatchResult};
pub use prefix::prefix_match;
