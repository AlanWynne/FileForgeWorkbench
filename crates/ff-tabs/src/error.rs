//! Error types for the `ff-tabs` crate.
//!
//! Defines `TabsError` — the unified error enum for all tab operations.

use crate::tab_id::TabId;
use ff_layout::TabGroupId;
use ff_vfs::ResourceUri;

/// Unified error type for all tab-related operations.
#[derive(Debug, thiserror::Error)]
pub enum TabsError {
    /// The specified tab was not found in any collection.
    #[error("[tabs] tab not found: {0}")]
    TabNotFound(TabId),

    /// The specified tab group was not found.
    #[error("[tabs] tab group not found: {0:?}")]
    TabGroupNotFound(TabGroupId),

    /// A tab with the same ResourceUri already exists.
    #[error("[tabs] duplicate resource: {0}")]
    DuplicateResource(ResourceUri),

    /// The maximum tab count has been reached and no evictable tab exists.
    #[error("[tabs] max tabs reached ({max}): all non-pinned tabs have unsaved changes")]
    MaxTabsReached {
        /// The configured maximum tab count.
        max: usize,
    },

    /// All non-pinned tabs are modified; cannot evict to make room.
    #[error("[tabs] all tabs modified: cannot close any tab to make room")]
    AllTabsModified,

    /// Failed to open the specified resource.
    #[error("[tabs] resource open failed: {uri} — {reason}")]
    ResourceOpenFailed {
        /// The URI that could not be opened.
        uri: ResourceUri,
        /// The reason for the failure.
        reason: String,
    },

    /// Failed to deserialise session tab data.
    #[error("[tabs] session deserialize failed: {0}")]
    SessionDeserializeFailed(String),

    /// Session data migration from an older schema version failed.
    #[error("[tabs] session migration failed from version {version}: {reason}")]
    SessionMigrationFailed {
        /// The schema version that failed to migrate.
        version: u32,
        /// The reason for the migration failure.
        reason: String,
    },

    /// An invalid TabId was provided (e.g., from session restore).
    #[error("[tabs] invalid tab id: {0}")]
    InvalidTabId(String),

    /// A split operation could not be completed.
    #[error("[tabs] split failed: {0}")]
    SplitFailed(String),

    /// A drag operation was cancelled.
    #[error("[tabs] drag cancelled")]
    DragCancelled,
}
