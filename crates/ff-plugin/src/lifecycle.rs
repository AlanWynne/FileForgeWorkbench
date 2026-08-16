//! Plugin lifecycle state machine.
//!
//! Defines the `PluginState` enum and validates state transitions.
//! The state machine is forward-only except for hot-reload cycles.

/// The lifecycle state of a plugin instance.
///
/// States progress forward: Discovered → Loaded → Initialized → Active → Deactivating → Shutdown.
/// The only exception is hot-reload, which cycles back from Active through Shutdown to Discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PluginState {
    /// Plugin binary/manifest found on disk, not yet loaded.
    Discovered,
    /// Plugin loaded into memory, manifest parsed.
    Loaded,
    /// Plugin's `initialize` method has been called successfully.
    Initialized,
    /// Plugin is fully active, capabilities registered.
    Active,
    /// Plugin is in the process of deactivating.
    Deactivating,
    /// Plugin has been shut down, resources released.
    Shutdown,
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discovered => write!(f, "Discovered"),
            Self::Loaded => write!(f, "Loaded"),
            Self::Initialized => write!(f, "Initialized"),
            Self::Active => write!(f, "Active"),
            Self::Deactivating => write!(f, "Deactivating"),
            Self::Shutdown => write!(f, "Shutdown"),
        }
    }
}

/// Checks whether a transition from `from` to `to` is valid.
///
/// Valid forward transitions:
/// - Discovered → Loaded
/// - Loaded → Initialized
/// - Initialized → Active
/// - Active → Deactivating
/// - Deactivating → Shutdown
///
/// Hot-reload exception:
/// - Shutdown → Discovered (allows re-discovery for reload)
///
/// Any plugin can transition to Shutdown from any state (forced shutdown on error/panic).
pub fn is_valid_transition(from: PluginState, to: PluginState) -> bool {
    matches!(
        (from, to),
        (PluginState::Discovered, PluginState::Loaded)
            | (PluginState::Loaded, PluginState::Initialized)
            | (PluginState::Initialized, PluginState::Active)
            | (PluginState::Active, PluginState::Deactivating)
            | (PluginState::Deactivating, PluginState::Shutdown)
            // Hot-reload cycle restart
            | (PluginState::Shutdown, PluginState::Discovered)
            // Forced shutdown from any state (error/panic recovery)
            | (PluginState::Discovered, PluginState::Shutdown)
            | (PluginState::Loaded, PluginState::Shutdown)
            | (PluginState::Initialized, PluginState::Shutdown)
            | (PluginState::Active, PluginState::Shutdown)
    )
}

/// Attempts a state transition, returning the new state or an error description.
///
/// This function validates the transition and returns the target state if valid,
/// or `None` if the transition is invalid.
pub fn try_transition(from: PluginState, to: PluginState) -> Option<PluginState> {
    if is_valid_transition(from, to) {
        Some(to)
    } else {
        None
    }
}

/// Validates a state transition, returning the new state or a PluginError.
///
/// This is the primary API for the lifecycle module, used by the registry
/// and property tests.
///
/// # Errors
///
/// Returns `PluginError::InvalidStateTransition` if the transition is not valid.
pub fn validate_transition(
    plugin_name: &str,
    from: PluginState,
    to: PluginState,
) -> Result<PluginState, crate::error::PluginError> {
    if is_valid_transition(from, to) {
        Ok(to)
    } else {
        Err(crate::error::PluginError::InvalidStateTransition {
            plugin: plugin_name.to_string(),
            from,
            to,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Valid Forward Transitions ──────────────────────────────────────────

    #[test]
    fn discovered_to_loaded_is_valid() {
        // Validates: Requirement 5.1
        assert!(is_valid_transition(
            PluginState::Discovered,
            PluginState::Loaded
        ));
    }

    #[test]
    fn loaded_to_initialized_is_valid() {
        // Validates: Requirement 5.1
        assert!(is_valid_transition(
            PluginState::Loaded,
            PluginState::Initialized
        ));
    }

    #[test]
    fn initialized_to_active_is_valid() {
        // Validates: Requirement 5.1
        assert!(is_valid_transition(
            PluginState::Initialized,
            PluginState::Active
        ));
    }

    #[test]
    fn active_to_deactivating_is_valid() {
        // Validates: Requirement 5.1
        assert!(is_valid_transition(
            PluginState::Active,
            PluginState::Deactivating
        ));
    }

    #[test]
    fn deactivating_to_shutdown_is_valid() {
        // Validates: Requirement 5.1
        assert!(is_valid_transition(
            PluginState::Deactivating,
            PluginState::Shutdown
        ));
    }

    // ─── Hot-Reload Cycle ───────────────────────────────────────────────────

    #[test]
    fn shutdown_to_discovered_is_valid_for_hot_reload() {
        // Validates: Requirement 5.1 (hot-reload exception)
        assert!(is_valid_transition(
            PluginState::Shutdown,
            PluginState::Discovered
        ));
    }

    // ─── Forced Shutdown From Any State ─────────────────────────────────────

    #[test]
    fn any_state_to_shutdown_is_valid_for_error_recovery() {
        // Validates: Requirement 5.3, 5.4
        assert!(is_valid_transition(
            PluginState::Discovered,
            PluginState::Shutdown
        ));
        assert!(is_valid_transition(
            PluginState::Loaded,
            PluginState::Shutdown
        ));
        assert!(is_valid_transition(
            PluginState::Initialized,
            PluginState::Shutdown
        ));
        assert!(is_valid_transition(
            PluginState::Active,
            PluginState::Shutdown
        ));
    }

    // ─── Invalid Transitions ────────────────────────────────────────────────

    #[test]
    fn active_to_discovered_is_invalid() {
        // Validates: Requirement 5.1
        assert!(!is_valid_transition(
            PluginState::Active,
            PluginState::Discovered
        ));
    }

    #[test]
    fn active_to_loaded_is_invalid() {
        // Validates: Requirement 5.1
        assert!(!is_valid_transition(
            PluginState::Active,
            PluginState::Loaded
        ));
    }

    #[test]
    fn discovered_to_active_is_invalid() {
        // Validates: Requirement 5.1
        assert!(!is_valid_transition(
            PluginState::Discovered,
            PluginState::Active
        ));
    }

    #[test]
    fn shutdown_to_active_is_invalid() {
        // Validates: Requirement 5.1
        assert!(!is_valid_transition(
            PluginState::Shutdown,
            PluginState::Active
        ));
    }

    #[test]
    fn loaded_to_active_is_invalid_must_initialize_first() {
        // Validates: Requirement 5.1
        assert!(!is_valid_transition(
            PluginState::Loaded,
            PluginState::Active
        ));
    }

    #[test]
    fn shutdown_to_shutdown_is_invalid() {
        // Validates: Requirement 5.1
        assert!(!is_valid_transition(
            PluginState::Shutdown,
            PluginState::Shutdown
        ));
    }

    // ─── try_transition ─────────────────────────────────────────────────────

    #[test]
    fn try_transition_returns_target_on_valid() {
        // Validates: Requirement 5.1
        assert_eq!(
            try_transition(PluginState::Discovered, PluginState::Loaded),
            Some(PluginState::Loaded)
        );
    }

    #[test]
    fn try_transition_returns_none_on_invalid() {
        // Validates: Requirement 5.1
        assert_eq!(
            try_transition(PluginState::Active, PluginState::Discovered),
            None
        );
    }
}
