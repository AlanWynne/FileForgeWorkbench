//! Error types for the layout engine.
//!
//! All errors follow the `[layout] operation: description` format per
//! the FileForgeWorkbench error message standards.

use crate::floating::window::FloatingWindowId;
use crate::resize::splitter::SplitterId;
use crate::tabs::group::TabGroupId;

/// Errors produced by the layout engine.
///
/// Formatted per Error Message Standards: `[layout] operation: description`
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LayoutError {
    /// Panel ID is not registered in the PanelRegistry.
    #[error("[layout] panel: '{panel_id}' is not registered")]
    PanelNotFound {
        /// The panel_id that was not found.
        panel_id: String,
    },

    /// Attempted to register a duplicate panel_id.
    #[error("[layout] register: panel '{panel_id}' is already registered")]
    DuplicatePanelId {
        /// The duplicate panel_id.
        panel_id: String,
    },

    /// Invalid dock zone specified for registration.
    #[error("[layout] register: invalid dock zone '{zone}' for panel '{panel_id}'")]
    InvalidDockZone {
        /// The panel_id being registered.
        panel_id: String,
        /// The invalid zone name.
        zone: String,
    },

    /// Invalid panel_id format (must be 1–64 ASCII alphanumeric/underscore).
    #[error("[layout] register: invalid panel_id format '{panel_id}' — {reason}")]
    InvalidPanelId {
        /// The invalid panel_id.
        panel_id: String,
        /// Description of why the format is invalid.
        reason: String,
    },

    /// Maximum floating windows reached.
    #[error("[layout] undock: maximum floating windows ({max}) reached")]
    MaxFloatingWindows {
        /// The maximum number of floating windows allowed.
        max: usize,
    },

    /// OS failed to create a floating window.
    #[error("[layout] undock: OS window creation failed for panel '{panel_id}'")]
    WindowCreationFailed {
        /// The panel that could not be floated.
        panel_id: String,
    },

    /// Floating window not found.
    #[error("[layout] floating: window {window_id:?} not found")]
    FloatingWindowNotFound {
        /// The window ID that was not found.
        window_id: FloatingWindowId,
    },

    /// Tab group not found.
    #[error("[layout] tabs: group {group_id:?} not found")]
    TabGroupNotFound {
        /// The group ID that was not found.
        group_id: TabGroupId,
    },

    /// Cannot split — would create empty editor area.
    #[error("[layout] split: cannot undock the only tab in the only group")]
    CannotEmptyEditor,

    /// Persona not found.
    #[error("[layout] persona: '{name}' not found")]
    PersonaNotFound {
        /// The persona name that was not found.
        name: String,
    },

    /// Cannot delete a built-in persona.
    #[error("[layout] persona: cannot delete built-in persona '{name}'")]
    CannotDeleteBuiltIn {
        /// The built-in persona name.
        name: String,
    },

    /// Serialization/deserialization failure.
    #[error("[layout] serialization: {operation} failed — {reason}")]
    SerializationFailed {
        /// The operation that failed (e.g., "save", "load", "export").
        operation: String,
        /// Description of the failure.
        reason: String,
    },

    /// I/O error during file operations.
    #[error("[layout] io: {0}")]
    Io(#[from] std::io::Error),

    /// Splitter not found.
    #[error("[layout] splitter: {splitter_id:?} not found")]
    SplitterNotFound {
        /// The splitter ID that was not found.
        splitter_id: SplitterId,
    },

    /// Tab index out of bounds.
    #[error("[layout] tab: index {index} out of bounds for group {group_id:?} (has {count} tabs)")]
    TabIndexOutOfBounds {
        /// The group containing the tab.
        group_id: TabGroupId,
        /// The requested index.
        index: usize,
        /// The actual number of tabs in the group.
        count: usize,
    },
}
