//! Error types for the file tree panel crate.

/// All errors produced by the file tree panel logic.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FileTreeError {
    /// A node was not found in the tree state.
    #[error("[file_tree] {operation}: node not found: {node_id:?}")]
    NodeNotFound {
        operation: String,
        node_id: crate::NodeId,
    },

    /// The path entered in the path bar does not exist.
    #[error("[file_tree] navigate: path not found: {path}")]
    PathNotFound { path: String },

    /// A configuration value is invalid.
    #[error("[file_tree] config: invalid value for '{key}': {reason}")]
    InvalidConfig { key: String, reason: String },

    /// Command registration or dispatch failure.
    #[error("[file_tree] command: {0}")]
    Command(String),

    /// The panel is disabled by configuration.
    #[error("[file_tree] init: panel disabled by configuration")]
    PanelDisabled,

    /// Maximum concurrent loads reached (non-fatal).
    #[error("[file_tree] loader: max concurrent loads reached ({max})")]
    LoaderAtCapacity { max: usize },
}
