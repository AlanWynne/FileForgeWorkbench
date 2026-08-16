//! Hook registry — maps event names to ordered handler lists.
//!
//! Manages registration, ordering, and dispatch of event handlers.
//! Addresses: Requirement 3 AC 2, AC 3

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A registered hook handler entry in the HookRegistry.
///
/// Addresses: Requirement 3 AC 2, AC 3
#[derive(Debug, Clone)]
pub struct HookHandler {
    /// The script that defined this handler.
    pub script_path: PathBuf,
    /// Registration order (script load order determines priority).
    pub registration_order: u64,
    /// The Lua function name (e.g., "OnOpen").
    pub function_name: String,
}

/// Result of dispatching a hook event.
#[derive(Debug, Clone, Default)]
pub struct HookDispatchResult {
    /// Whether any handler cancelled the event.
    pub cancelled: bool,
    /// The script path that cancelled (if any).
    pub cancelled_by: Option<PathBuf>,
    /// Errors encountered during dispatch (non-fatal for subsequent handlers).
    pub errors: Vec<String>,
}

/// Manages event-to-handler mappings with ordered dispatch.
///
/// Addresses: Requirement 3 AC 2, AC 3
#[derive(Debug, Default)]
pub struct HookRegistry {
    /// Map from event type name to ordered list of handlers.
    handlers: HashMap<String, Vec<HookHandler>>,
    /// Monotonically increasing counter for registration ordering.
    next_order: u64,
}

impl HookRegistry {
    /// Creates a new empty hook registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler for the given event name.
    ///
    /// Handlers are stored in registration order (first registered = first invoked).
    ///
    /// Addresses: Requirement 3 AC 2
    pub fn register(
        &mut self,
        event_name: &str,
        script_path: PathBuf,
        function_name: String,
    ) -> u64 {
        let order = self.next_order;
        self.next_order += 1;

        let handler = HookHandler {
            script_path,
            registration_order: order,
            function_name,
        };

        self.handlers
            .entry(event_name.to_string())
            .or_default()
            .push(handler);

        order
    }

    /// Unregister all handlers from a specific script.
    ///
    /// Used during script reload to prevent duplicate handlers.
    ///
    /// Addresses: Requirement 8 AC 3
    pub fn unregister_by_script(&mut self, script_path: &Path) {
        for handlers in self.handlers.values_mut() {
            handlers.retain(|h| h.script_path != script_path);
        }
    }

    /// Returns the ordered handler list for an event.
    ///
    /// Handlers are in registration order (first loaded = first invoked).
    pub fn handlers_for(&self, event_name: &str) -> &[HookHandler] {
        self.handlers
            .get(event_name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the total number of registered handlers across all events.
    pub fn total_handler_count(&self) -> usize {
        self.handlers.values().map(|v| v.len()).sum()
    }

    /// Returns the number of handlers for a specific event.
    pub fn handler_count_for(&self, event_name: &str) -> usize {
        self.handlers_for(event_name).len()
    }

    /// Returns all event names that have at least one handler registered.
    pub fn active_events(&self) -> Vec<&str> {
        self.handlers
            .iter()
            .filter(|(_, handlers)| !handlers.is_empty())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Clears all handlers (used during shutdown).
    pub fn clear(&mut self) {
        self.handlers.clear();
        self.next_order = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.2
    #[test]
    fn register_adds_handler_in_order() {
        let mut registry = HookRegistry::new();
        registry.register("OnOpen", PathBuf::from("script1.lua"), "OnOpen".to_string());
        registry.register("OnOpen", PathBuf::from("script2.lua"), "OnOpen".to_string());

        let handlers = registry.handlers_for("OnOpen");
        assert_eq!(handlers.len(), 2);
        assert_eq!(handlers[0].script_path, PathBuf::from("script1.lua"));
        assert_eq!(handlers[1].script_path, PathBuf::from("script2.lua"));
    }

    // Validates: Requirement 3.3
    #[test]
    fn handlers_maintain_load_order() {
        let mut registry = HookRegistry::new();
        for i in 0..5 {
            registry.register(
                "OnChar",
                PathBuf::from(format!("script{i}.lua")),
                "OnChar".to_string(),
            );
        }

        let handlers = registry.handlers_for("OnChar");
        for (idx, handler) in handlers.iter().enumerate() {
            assert_eq!(handler.registration_order, idx as u64);
        }
    }

    // Validates: Requirement 8.3
    #[test]
    fn unregister_by_script_removes_only_target_scripts_handlers() {
        let mut registry = HookRegistry::new();
        registry.register("OnOpen", PathBuf::from("keep.lua"), "OnOpen".to_string());
        registry.register("OnOpen", PathBuf::from("remove.lua"), "OnOpen".to_string());
        registry.register("OnChar", PathBuf::from("remove.lua"), "OnChar".to_string());
        registry.register("OnChar", PathBuf::from("keep.lua"), "OnChar".to_string());

        registry.unregister_by_script(Path::new("remove.lua"));

        assert_eq!(registry.handlers_for("OnOpen").len(), 1);
        assert_eq!(
            registry.handlers_for("OnOpen")[0].script_path,
            PathBuf::from("keep.lua")
        );
        assert_eq!(registry.handlers_for("OnChar").len(), 1);
        assert_eq!(
            registry.handlers_for("OnChar")[0].script_path,
            PathBuf::from("keep.lua")
        );
    }

    #[test]
    fn handlers_for_unknown_event_returns_empty() {
        let registry = HookRegistry::new();
        assert!(registry.handlers_for("OnNonexistent").is_empty());
    }

    #[test]
    fn total_handler_count_sums_all_events() {
        let mut registry = HookRegistry::new();
        registry.register("OnOpen", PathBuf::from("s1.lua"), "OnOpen".to_string());
        registry.register("OnChar", PathBuf::from("s1.lua"), "OnChar".to_string());
        registry.register("OnChar", PathBuf::from("s2.lua"), "OnChar".to_string());

        assert_eq!(registry.total_handler_count(), 3);
    }

    #[test]
    fn clear_removes_all_handlers() {
        let mut registry = HookRegistry::new();
        registry.register("OnOpen", PathBuf::from("s.lua"), "OnOpen".to_string());
        registry.clear();
        assert_eq!(registry.total_handler_count(), 0);
    }
}
