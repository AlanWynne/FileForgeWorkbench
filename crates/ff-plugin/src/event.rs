//! Platform event types for plugin lifecycle notifications.
//!
//! Defines the `PlatformEvent` enum, `SubscriptionId`, and `EventHandler`
//! type alias used for plugin event subscription and emission.

use crate::capability::CapabilityType;

/// Unique identifier for an event subscription, used for unsubscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub(crate) u64);

impl SubscriptionId {
    /// Creates a new subscription ID from a raw value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw numeric ID.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Platform events that plugins can subscribe to via PluginContext.
///
/// These are a subset of workbench events relevant to plugins.
/// The Capability_Registry emits `CapabilityChanged` events when
/// capabilities are added or removed.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PlatformEvent {
    /// A configuration key changed.
    ConfigChanged {
        /// The configuration key that changed.
        key: String,
    },
    /// A document was opened.
    DocumentOpened {
        /// URI of the opened document.
        uri: String,
    },
    /// A document was closed.
    DocumentClosed {
        /// URI of the closed document.
        uri: String,
    },
    /// Application shutdown has been requested.
    ShutdownRequested,
    /// A capability was registered or unregistered.
    CapabilityChanged {
        /// The type of capability that changed.
        capability_type: CapabilityType,
        /// The plugin that owns the changed capability.
        owner_plugin: String,
        /// Whether the capability was added (true) or removed (false).
        added: bool,
    },
    /// A plugin lifecycle state changed.
    PluginStateChanged {
        /// Name of the plugin.
        plugin_name: String,
        /// Description of the new state.
        new_state: String,
    },
}

/// Callback type for plugin event handlers.
///
/// Event handlers are invoked when a subscribed event occurs. They must be
/// thread-safe (`Send + Sync`) since events may be dispatched from any thread.
pub type EventHandler = Box<dyn Fn(&PlatformEvent) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_id_equality() {
        // Validates: Requirement 2.2 (event subscription)
        let id1 = SubscriptionId::new(42);
        let id2 = SubscriptionId::new(42);
        let id3 = SubscriptionId::new(43);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn subscription_id_as_u64() {
        // Validates: Requirement 2.2
        let id = SubscriptionId::new(99);
        assert_eq!(id.as_u64(), 99);
    }

    #[test]
    fn platform_event_capability_changed_construction() {
        // Validates: Requirement 4.6
        let event = PlatformEvent::CapabilityChanged {
            capability_type: CapabilityType::Commands,
            owner_plugin: "test-plugin".to_string(),
            added: true,
        };
        match event {
            PlatformEvent::CapabilityChanged {
                capability_type,
                owner_plugin,
                added,
            } => {
                assert_eq!(capability_type, CapabilityType::Commands);
                assert_eq!(owner_plugin, "test-plugin");
                assert!(added);
            }
            _ => panic!("unexpected event variant"),
        }
    }

    #[test]
    fn event_handler_type_is_send_sync() {
        // Validates: Requirement 2.5
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EventHandler>();
    }
}
