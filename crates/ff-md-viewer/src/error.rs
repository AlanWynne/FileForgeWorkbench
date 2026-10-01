use std::path::PathBuf;

use thiserror::Error;

/// Errors produced by the `ff-md-viewer` library.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MdViewerError {
    /// The filesystem watcher could not be created or could not begin watching
    /// the requested root path.
    #[error("failed to watch '{path}': {source}")]
    Watch {
        /// The root path the watcher was asked to observe.
        path: PathBuf,
        /// The underlying `notify` error.
        #[source]
        source: notify::Error,
    },
}
