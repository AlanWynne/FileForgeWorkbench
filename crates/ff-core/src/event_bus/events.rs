//! The WorkbenchEvent enum, its categories, and category classification.

use super::{
    CommandOutcome, CommandParams, DocumentId, NotificationSeverity, OperationId, ProgressInfo,
};

// === WorkbenchEvent Enum ====================================================

/// All events that flow through the Event Bus. Categorized per Requirement 3.2.
///
/// Addresses: Requirement 3, criteria 1/2
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WorkbenchEvent {
    // --- Commands (user-initiated operations) ---
    /// A command was dispatched for execution.
    CommandDispatched {
        /// The unique identifier of the command being dispatched.
        command_id: String,
        /// Parameters passed to the command.
        params: CommandParams,
    },
    /// A command completed execution.
    CommandCompleted {
        /// The unique identifier of the command that completed.
        command_id: String,
        /// The outcome of the command execution.
        outcome: CommandOutcome,
    },

    // --- Notifications (informational messages to GUI) ---
    /// Informational message for the status bar or notification area.
    Notification {
        /// The notification message text.
        message: String,
        /// The severity level of the notification.
        severity: NotificationSeverity,
    },

    // --- State-change signals (model updates requiring re-render) ---
    /// A document's content changed.
    DocumentChanged {
        /// The identifier of the document that changed.
        document_id: DocumentId,
    },
    /// The active document/tab changed.
    ActiveDocumentChanged {
        /// The identifier of the newly active document, or `None` if no document is active.
        document_id: Option<DocumentId>,
    },
    /// Configuration was reloaded.
    ConfigReloaded,

    // --- Progress updates (long-running operation status) ---
    /// Progress update for an async operation.
    Progress {
        /// The identifier of the operation reporting progress.
        operation_id: OperationId,
        /// The current progress information.
        progress: ProgressInfo,
    },

    // --- Lifecycle events ---
    /// The workbench has completed startup and is ready for interaction.
    WorkbenchReady,
    /// A shutdown sequence has been initiated.
    ShutdownInitiated,
    /// A plugin was successfully hot-reloaded.
    PluginReloaded {
        /// The name of the plugin that was reloaded.
        plugin_name: String,
    },
}

// === Event Categories =======================================================

/// Event categories for subscription filtering.
///
/// Subscribers can register interest in one or more categories to receive
/// only relevant events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    /// Command dispatch and completion events.
    Command,
    /// Informational notification events.
    Notification,
    /// State-change signals requiring UI updates.
    StateChange,
    /// Progress updates for long-running operations.
    Progress,
    /// Application lifecycle events (ready, shutdown, plugin reload).
    Lifecycle,
}

// === WorkbenchEvent Implementation =========================================

impl WorkbenchEvent {
    /// Returns the category of this event for subscription filtering.
    pub fn category(&self) -> EventCategory {
        match self {
            Self::CommandDispatched { .. } | Self::CommandCompleted { .. } => {
                EventCategory::Command
            }
            Self::Notification { .. } => EventCategory::Notification,
            Self::DocumentChanged { .. }
            | Self::ActiveDocumentChanged { .. }
            | Self::ConfigReloaded => EventCategory::StateChange,
            Self::Progress { .. } => EventCategory::Progress,
            Self::WorkbenchReady | Self::ShutdownInitiated | Self::PluginReloaded { .. } => {
                EventCategory::Lifecycle
            }
        }
    }
}
