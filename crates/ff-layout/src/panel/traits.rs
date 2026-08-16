//! The `DockablePanel` trait and associated types.
//!
//! Any component that implements `DockablePanel` can participate in dock/undock
//! operations, appear in tab groups, float as an independent OS window, and
//! be included in persona configurations.

use crate::dock::zone::DockZone;

/// The current state of a panel within the layout system.
///
/// Passed to `DockablePanel::on_dock_state_changed` when a panel transitions
/// between states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockState {
    /// Panel is attached to a dock zone and visible at normal size.
    Docked,
    /// Panel is in a floating OS window.
    Floating,
    /// Panel is collapsed to a tab/icon in the zone header.
    Minimized,
    /// Panel is hidden from view (position preserved in state).
    Hidden,
    /// Panel is expanded to fill the entire primary window content area.
    Maximized,
}

/// Trait that all dockable panels must implement.
///
/// The Layout_Engine interacts with panels exclusively through this interface.
/// Panels are contributed by the plugin system and registered with the
/// `PanelRegistry` during initialization.
pub trait DockablePanel: Send + Sync {
    /// Returns the unique panel identifier (1–64 ASCII alphanumeric/underscore chars).
    fn panel_id(&self) -> &str;

    /// Returns the preferred default dock zone.
    fn default_dock_zone(&self) -> DockZone;

    /// Returns the display title (1–128 characters).
    fn title(&self) -> &str;

    /// Called when the panel transitions between dock states.
    fn on_dock_state_changed(&mut self, state: DockState);

    /// Returns the minimum size constraint in logical pixels (width, height).
    ///
    /// Returns `None` to use the default minimum of 48×48 logical pixels.
    fn minimum_size(&self) -> Option<(f32, f32)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test panel for verifying the trait interface.
    struct TestPanel {
        id: String,
        title: String,
        zone: DockZone,
        last_state: Option<DockState>,
    }

    impl TestPanel {
        fn new(id: &str, title: &str, zone: DockZone) -> Self {
            Self {
                id: id.to_string(),
                title: title.to_string(),
                zone,
                last_state: None,
            }
        }
    }

    impl DockablePanel for TestPanel {
        fn panel_id(&self) -> &str {
            &self.id
        }

        fn default_dock_zone(&self) -> DockZone {
            self.zone
        }

        fn title(&self) -> &str {
            &self.title
        }

        fn on_dock_state_changed(&mut self, state: DockState) {
            self.last_state = Some(state);
        }
    }

    #[test]
    fn dockable_panel_trait_methods_work() {
        let mut panel = TestPanel::new("file_tree", "File Tree", DockZone::Left);
        assert_eq!(panel.panel_id(), "file_tree");
        assert_eq!(panel.default_dock_zone(), DockZone::Left);
        assert_eq!(panel.title(), "File Tree");
        assert_eq!(panel.minimum_size(), None);

        panel.on_dock_state_changed(DockState::Floating);
        assert_eq!(panel.last_state, Some(DockState::Floating));
    }

    #[test]
    fn dock_state_equality() {
        assert_eq!(DockState::Docked, DockState::Docked);
        assert_ne!(DockState::Docked, DockState::Floating);
        assert_ne!(DockState::Hidden, DockState::Minimized);
    }
}
