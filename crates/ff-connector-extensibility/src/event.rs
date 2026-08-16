//! Platform events emitted by the connector extensibility framework.
//!
//! These events are dispatched via the `EventBus` (from ff-core) to notify
//! the platform and GUI shell of connector registration, state changes, and
//! capability changes.

use crate::capability::ConnectorCapability;
use crate::state::ConnectorState;

/// Event emitted when a connector is registered or deregistered.
///
/// Dispatched via the platform EventBus so that consuming subsystems
/// (UI panels, status bars) can react to connector availability changes.
///
/// Addresses: Requirement 2 AC 6
#[derive(Debug, Clone)]
pub struct ConnectorRegisteredEvent {
    /// The URI scheme of the connector.
    pub scheme: String,
    /// The human-readable display name of the connector.
    pub display_name: String,
    /// `true` = registered, `false` = deregistered.
    pub registered: bool,
}

/// Event emitted when a connector transitions between lifecycle states.
///
/// Dispatched via the platform EventBus so that consuming subsystems
/// can display connection status and react to connectivity changes.
///
/// Addresses: Requirement 4 AC 3
#[derive(Debug, Clone)]
pub struct ConnectorStateChangedEvent {
    /// The URI scheme of the connector.
    pub scheme: String,
    /// The state before the transition.
    pub previous_state: ConnectorState,
    /// The state after the transition.
    pub new_state: ConnectorState,
}

/// Event emitted when a connector's capabilities change at runtime.
///
/// Dispatched when a connector calls `refresh_capabilities` (e.g., after
/// reconnection with different permissions).
///
/// Addresses: Requirement 3 AC 6
#[derive(Debug, Clone)]
pub struct ConnectorCapabilityChangedEvent {
    /// The URI scheme of the connector.
    pub scheme: String,
    /// The new set of capabilities after the change.
    pub capabilities: Vec<ConnectorCapability>,
}
