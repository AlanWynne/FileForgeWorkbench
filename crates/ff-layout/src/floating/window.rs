//! Floating window data types.
//!
//! Represents an OS-level window containing one or more detached panels or tabs.

use crate::dock::zone::DockZone;
use crate::{Position, Size};

/// Opaque identifier for a floating window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FloatingWindowId(pub(crate) u32);

impl FloatingWindowId {
    /// Creates a new floating window ID from a raw value.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw numeric value.
    pub fn value(self) -> u32 {
        self.0
    }
}

/// Represents an OS-level window containing one or more detached panels/tabs.
///
/// Floating windows are full platform viewports that appear in the OS taskbar
/// and are independently movable, resizable, minimizable, and maximizable.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FloatingWindow {
    /// Unique identifier for this floating window.
    pub id: FloatingWindowId,
    /// Panel IDs contained in this floating window.
    pub panels: Vec<String>,
    /// Position in logical pixels (screen coordinates).
    pub position: Position,
    /// Size in logical pixels (minimum 200×150).
    pub size: Size,
    /// Monitor identifier for multi-monitor persistence.
    pub monitor_id: Option<String>,
    /// The dock zone the panel(s) originated from (for redock).
    pub origin_zone: DockZone,
    /// Original tab index within the origin group (for tab redock).
    pub origin_tab_index: Option<usize>,
    /// DPI scale factor of the monitor this window is on.
    pub dpi_scale: f32,
}

impl FloatingWindow {
    /// Minimum width for a floating window in logical pixels.
    pub const MIN_WIDTH: f32 = 200.0;
    /// Minimum height for a floating window in logical pixels.
    pub const MIN_HEIGHT: f32 = 150.0;

    /// Creates a new floating window with the given parameters.
    pub fn new(
        id: FloatingWindowId,
        panels: Vec<String>,
        position: Position,
        size: Size,
        origin_zone: DockZone,
    ) -> Self {
        let clamped_size = Size {
            width: size.width.max(Self::MIN_WIDTH),
            height: size.height.max(Self::MIN_HEIGHT),
        };
        Self {
            id,
            panels,
            position,
            size: clamped_size,
            monitor_id: None,
            origin_zone,
            origin_tab_index: None,
            dpi_scale: 1.0,
        }
    }

    /// Updates the position and size of this window.
    pub fn update_position_size(&mut self, position: Position, size: Size) {
        self.position = position;
        self.size = Size {
            width: size.width.max(Self::MIN_WIDTH),
            height: size.height.max(Self::MIN_HEIGHT),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_window_clamps_minimum_size() {
        let window = FloatingWindow::new(
            FloatingWindowId(1),
            vec!["panel_a".to_string()],
            Position::new(100.0, 100.0),
            Size::new(50.0, 50.0), // Below minimum
            DockZone::Left,
        );
        assert_eq!(window.size.width, FloatingWindow::MIN_WIDTH);
        assert_eq!(window.size.height, FloatingWindow::MIN_HEIGHT);
    }

    #[test]
    fn floating_window_preserves_valid_size() {
        let window = FloatingWindow::new(
            FloatingWindowId(1),
            vec!["panel_a".to_string()],
            Position::new(100.0, 100.0),
            Size::new(400.0, 300.0),
            DockZone::Left,
        );
        assert_eq!(window.size.width, 400.0);
        assert_eq!(window.size.height, 300.0);
    }

    #[test]
    fn update_position_size_clamps_minimum() {
        let mut window = FloatingWindow::new(
            FloatingWindowId(1),
            vec!["panel_a".to_string()],
            Position::new(0.0, 0.0),
            Size::new(400.0, 300.0),
            DockZone::Left,
        );
        window.update_position_size(Position::new(50.0, 50.0), Size::new(100.0, 100.0));
        assert_eq!(window.position, Position::new(50.0, 50.0));
        assert_eq!(window.size.width, FloatingWindow::MIN_WIDTH);
        assert_eq!(window.size.height, FloatingWindow::MIN_HEIGHT);
    }

    #[test]
    fn floating_window_id_round_trip() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Wrapper {
            id: FloatingWindowId,
        }

        let id = FloatingWindowId::new(42);
        assert_eq!(id.value(), 42);
        let wrapper = Wrapper { id };
        let serialized = toml::to_string(&wrapper).unwrap();
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized.id);
    }
}
