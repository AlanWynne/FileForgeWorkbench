//! Connector lifecycle state machine.
//!
//! Defines `ConnectorState` — the lifecycle state of a connector instance —
//! and transition validation logic enforcing the state machine rules.

/// The lifecycle state of a connector instance.
///
/// State machine transitions:
/// ```text
/// Registered → Connecting → Connected | Error
/// Connected → Disconnecting → Disconnected
/// Error → Connecting (retry) | Disconnected
/// Disconnected → Connecting
/// ```
///
/// Addresses: Requirement 4 AC 1
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectorState {
    /// Connector registered but not yet connected.
    Registered,
    /// Connection attempt in progress.
    Connecting,
    /// Successfully connected and ready for operations.
    Connected,
    /// Graceful disconnect in progress.
    Disconnecting,
    /// Disconnected (idle — can reconnect).
    Disconnected,
    /// Error state — connection failed; stores the error message for state queries.
    Error {
        /// Human-readable description of the error that caused this state.
        message: String,
    },
}

impl ConnectorState {
    /// Returns `true` if the connector is in the `Connected` state.
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Returns `true` if the connector is operational (connected or connecting).
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Connected | Self::Connecting)
    }

    /// Returns `true` if a `connect()` call is valid from the current state.
    ///
    /// Connect is valid from: Registered, Disconnected, Error.
    pub fn can_connect(&self) -> bool {
        matches!(
            self,
            Self::Registered | Self::Disconnected | Self::Error { .. }
        )
    }

    /// Returns `true` if a `disconnect()` call is valid from the current state.
    ///
    /// Disconnect is valid from: Connected.
    pub fn can_disconnect(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

/// Validates whether a state transition from `from` to `to` is valid
/// according to the connector lifecycle state machine.
///
/// Valid transitions:
/// - Registered → Connecting
/// - Connecting → Connected
/// - Connecting → Error
/// - Connected → Disconnecting
/// - Disconnecting → Disconnected
/// - Error → Connecting (retry)
/// - Error → Disconnected (give up)
/// - Disconnected → Connecting (reconnect)
///
/// Addresses: Requirement 4 AC 1, AC 2
pub fn is_valid_transition(from: &ConnectorState, to: &ConnectorState) -> bool {
    matches!(
        (from, to),
        (ConnectorState::Registered, ConnectorState::Connecting)
            | (ConnectorState::Connecting, ConnectorState::Connected)
            | (ConnectorState::Connecting, ConnectorState::Error { .. })
            | (ConnectorState::Connected, ConnectorState::Disconnecting)
            | (ConnectorState::Disconnecting, ConnectorState::Disconnected)
            | (ConnectorState::Error { .. }, ConnectorState::Connecting)
            | (ConnectorState::Error { .. }, ConnectorState::Disconnected)
            | (ConnectorState::Disconnected, ConnectorState::Connecting)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4 AC 1
    #[test]
    fn registered_can_transition_to_connecting() {
        assert!(is_valid_transition(
            &ConnectorState::Registered,
            &ConnectorState::Connecting
        ));
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn connecting_can_transition_to_connected() {
        assert!(is_valid_transition(
            &ConnectorState::Connecting,
            &ConnectorState::Connected
        ));
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn connecting_can_transition_to_error() {
        assert!(is_valid_transition(
            &ConnectorState::Connecting,
            &ConnectorState::Error {
                message: "timeout".to_string()
            }
        ));
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn connected_can_transition_to_disconnecting() {
        assert!(is_valid_transition(
            &ConnectorState::Connected,
            &ConnectorState::Disconnecting
        ));
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn disconnecting_can_transition_to_disconnected() {
        assert!(is_valid_transition(
            &ConnectorState::Disconnecting,
            &ConnectorState::Disconnected
        ));
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn error_can_transition_to_connecting_for_retry() {
        assert!(is_valid_transition(
            &ConnectorState::Error {
                message: "network failure".to_string()
            },
            &ConnectorState::Connecting
        ));
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn error_can_transition_to_disconnected() {
        assert!(is_valid_transition(
            &ConnectorState::Error {
                message: "fatal".to_string()
            },
            &ConnectorState::Disconnected
        ));
    }

    // Validates: Requirement 4 AC 1
    #[test]
    fn disconnected_can_transition_to_connecting() {
        assert!(is_valid_transition(
            &ConnectorState::Disconnected,
            &ConnectorState::Connecting
        ));
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn registered_cannot_transition_to_connected_directly() {
        assert!(!is_valid_transition(
            &ConnectorState::Registered,
            &ConnectorState::Connected
        ));
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn connected_cannot_transition_to_registered() {
        assert!(!is_valid_transition(
            &ConnectorState::Connected,
            &ConnectorState::Registered
        ));
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn disconnecting_cannot_transition_to_connected() {
        assert!(!is_valid_transition(
            &ConnectorState::Disconnecting,
            &ConnectorState::Connected
        ));
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn disconnected_cannot_transition_to_connected_directly() {
        assert!(!is_valid_transition(
            &ConnectorState::Disconnected,
            &ConnectorState::Connected
        ));
    }

    #[test]
    fn is_connected_returns_true_only_for_connected() {
        assert!(ConnectorState::Connected.is_connected());
        assert!(!ConnectorState::Connecting.is_connected());
        assert!(!ConnectorState::Registered.is_connected());
    }

    #[test]
    fn can_connect_returns_true_for_valid_states() {
        assert!(ConnectorState::Registered.can_connect());
        assert!(ConnectorState::Disconnected.can_connect());
        assert!(ConnectorState::Error {
            message: "err".to_string()
        }
        .can_connect());
        assert!(!ConnectorState::Connected.can_connect());
        assert!(!ConnectorState::Connecting.can_connect());
    }

    #[test]
    fn can_disconnect_returns_true_only_for_connected() {
        assert!(ConnectorState::Connected.can_disconnect());
        assert!(!ConnectorState::Disconnected.can_disconnect());
        assert!(!ConnectorState::Registered.can_disconnect());
    }
}
