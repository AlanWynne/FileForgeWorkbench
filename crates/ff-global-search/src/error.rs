//! Error type for ff-global-search.

use thiserror::Error;

/// Errors that can occur during global search or replace operations.
#[derive(Debug, Error)]
pub enum GlobalSearchError {
    /// An I/O error occurred while reading or writing a file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The search query is invalid (e.g. bad regex).
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}
