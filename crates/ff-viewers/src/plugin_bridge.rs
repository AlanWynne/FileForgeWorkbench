//! Plugin viewer bridge — integration between the plugin architecture and the viewer registry.
//!
//! Provides functions for plugin registration/deregistration of viewers, and
//! handles the plugin shutdown lifecycle (auto-deregistering contributed viewers).

use crate::error::ViewerError;
use crate::key::ViewerKey;
use crate::panel::ViewerPanel;
use crate::registry::{ViewerRegistry, ViewerSource};
use crate::trait_def::FileViewer;

/// Register a plugin-provided viewer into the ViewerRegistry.
///
/// This function is intended to be called from `PluginContext::register_viewer()`
/// during a plugin's `initialize` lifecycle phase.
///
/// # Errors
///
/// Returns `ViewerError::InvalidKeyFormat` if the viewer's key is malformed.
/// Returns `ViewerError::DuplicateKey` if the key already exists in the registry.
pub fn register_plugin_viewer(
    registry: &ViewerRegistry,
    viewer: Box<dyn FileViewer>,
) -> Result<(), ViewerError> {
    registry.register_plugin(viewer)
}

/// Deregister a plugin-provided viewer from the ViewerRegistry.
///
/// If the deregistered viewer is currently active in the panel, the panel
/// is deactivated.
///
/// # Errors
///
/// Returns `ViewerError::UnknownKey` if the key is not found in the registry.
pub fn deregister_plugin_viewer(
    registry: &ViewerRegistry,
    panel: &mut ViewerPanel,
    viewer_key: &str,
) -> Result<(), ViewerError> {
    let key = ViewerKey::new(viewer_key)?;

    // Verify the viewer exists and is a plugin viewer
    match registry.viewer_source(&key) {
        None => {
            return Err(ViewerError::UnknownKey {
                key: viewer_key.to_string(),
            });
        }
        Some(ViewerSource::BuiltIn) => {
            return Err(ViewerError::UnknownKey {
                key: format!("{viewer_key} (built-in viewers cannot be deregistered by plugins)"),
            });
        }
        Some(ViewerSource::Plugin) => {}
    }

    // If the panel is showing this viewer, deactivate it
    if let Some(active_key) = panel.active_viewer_key() {
        if active_key == &key {
            panel.deactivate();
        }
    }

    registry.deregister(&key);
    Ok(())
}

/// Handle plugin shutdown — deregisters all viewers contributed by the plugin
/// and closes any active ViewerPanel using those viewers.
///
/// Returns the list of viewer keys that were deregistered.
pub fn handle_plugin_shutdown(
    registry: &ViewerRegistry,
    panel: &mut ViewerPanel,
) -> Vec<ViewerKey> {
    let removed = registry.deregister_all_plugin_viewers();

    // If the panel was showing a now-deregistered viewer, deactivate it
    if let Some(active_key) = panel.active_viewer_key() {
        if removed.iter().any(|k| k == active_key) {
            panel.deactivate();
        }
    }

    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in::register_built_in_viewers;

    /// A simple plugin viewer for testing.
    struct PluginViewer {
        key: String,
    }

    impl PluginViewer {
        fn new(key: &str) -> Self {
            Self {
                key: key.to_string(),
            }
        }
    }

    impl FileViewer for PluginViewer {
        fn viewer_key(&self) -> &str {
            &self.key
        }
        fn display_name(&self) -> &str {
            "Plugin Viewer"
        }
        fn description(&self) -> &str {
            "A test plugin viewer"
        }
        fn supported_extensions(&self) -> &[&str] {
            &[]
        }
        fn supported_mime_types(&self) -> &[&str] {
            &[]
        }
        fn can_render(&self, _: &str, _: &[u8]) -> bool {
            false
        }
        fn render(&self, _: &[u8]) -> String {
            String::new()
        }
        fn on_content_changed(&mut self, _: &[u8]) {}
    }

    #[test]
    fn register_plugin_viewer_succeeds() {
        // Validates: Requirement 5 AC 1
        let registry = ViewerRegistry::new();
        let viewer = Box::new(PluginViewer::new("my-plugin-viewer"));
        let result = register_plugin_viewer(&registry, viewer);
        assert!(result.is_ok());

        let key = ViewerKey::new("my-plugin-viewer").unwrap();
        assert!(registry.contains(&key));
        assert_eq!(registry.viewer_source(&key), Some(ViewerSource::Plugin));
    }

    #[test]
    fn register_plugin_viewer_duplicate_rejected() {
        // Validates: Requirement 5 AC 2
        let registry = ViewerRegistry::new();
        register_plugin_viewer(&registry, Box::new(PluginViewer::new("my-viewer"))).unwrap();

        let result = register_plugin_viewer(&registry, Box::new(PluginViewer::new("my-viewer")));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ViewerError::DuplicateKey { .. }
        ));
    }

    #[test]
    fn deregister_plugin_viewer_removes_and_deactivates_panel() {
        // Validates: Requirement 5 AC 4, AC 5
        let registry = ViewerRegistry::new();
        register_plugin_viewer(&registry, Box::new(PluginViewer::new("my-viewer"))).unwrap();

        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("my-viewer").unwrap();
        panel.activate(key.clone(), "file:///test".to_string(), vec![]);
        assert!(panel.is_active());

        deregister_plugin_viewer(&registry, &mut panel, "my-viewer").unwrap();

        assert!(!panel.is_active());
        assert!(!registry.contains(&key));
    }

    #[test]
    fn deregister_plugin_viewer_unknown_key_returns_error() {
        let registry = ViewerRegistry::new();
        let mut panel = ViewerPanel::new();

        let result = deregister_plugin_viewer(&registry, &mut panel, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn deregister_builtin_viewer_rejected() {
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let mut panel = ViewerPanel::new();

        let result = deregister_plugin_viewer(&registry, &mut panel, "hex");
        assert!(result.is_err());
    }

    #[test]
    fn handle_plugin_shutdown_deregisters_all_plugin_viewers() {
        // Validates: Requirement 5 AC 3
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        register_plugin_viewer(&registry, Box::new(PluginViewer::new("plugin-a"))).unwrap();
        register_plugin_viewer(&registry, Box::new(PluginViewer::new("plugin-b"))).unwrap();

        assert_eq!(registry.viewer_count(), 6); // 4 built-in + 2 plugin

        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("plugin-a").unwrap();
        panel.activate(key, "file:///test".to_string(), vec![]);

        let removed = handle_plugin_shutdown(&registry, &mut panel);

        assert_eq!(removed.len(), 2);
        assert_eq!(registry.viewer_count(), 4); // only built-ins remain
        assert!(!panel.is_active()); // panel deactivated because plugin-a was removed
    }

    #[test]
    fn handle_plugin_shutdown_preserves_builtin_viewers() {
        // Validates: Requirement 5 AC 3
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        register_plugin_viewer(&registry, Box::new(PluginViewer::new("plugin-x"))).unwrap();

        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("hex").unwrap();
        panel.activate(key, "file:///test".to_string(), vec![]);

        handle_plugin_shutdown(&registry, &mut panel);

        // Panel should remain active because hex is built-in
        assert!(panel.is_active());
        assert_eq!(registry.viewer_count(), 4);
    }
}
