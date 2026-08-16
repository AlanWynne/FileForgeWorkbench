//! Floating window manager — creation, tracking, and limit enforcement.

use crate::dock::zone::DockZone;
use crate::error::LayoutError;
use crate::floating::window::{FloatingWindow, FloatingWindowId};
use crate::{Position, Size, MAX_FLOATING_WINDOWS};

/// Manages the lifecycle of floating OS-level windows.
///
/// Tracks all active floating windows, enforces the maximum window count,
/// and calculates cascade positions for new windows.
#[derive(Debug)]
pub struct FloatingWindowManager {
    /// All active floating windows.
    windows: Vec<FloatingWindow>,
    /// Next ID to assign.
    next_id: u32,
    /// Maximum floating windows allowed.
    max_windows: usize,
}

impl FloatingWindowManager {
    /// Cascade offset in logical pixels per floating window.
    pub const CASCADE_OFFSET: f32 = 50.0;

    /// Creates a new floating window manager.
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            max_windows: MAX_FLOATING_WINDOWS,
        }
    }

    /// Creates a manager with existing windows (for state restoration).
    pub fn from_windows(windows: Vec<FloatingWindow>) -> Self {
        let max_id = windows.iter().map(|w| w.id.value()).max().unwrap_or(0);
        Self {
            windows,
            next_id: max_id + 1,
            max_windows: MAX_FLOATING_WINDOWS,
        }
    }

    /// Returns the number of active floating windows.
    pub fn count(&self) -> usize {
        self.windows.len()
    }

    /// Returns all floating windows.
    pub fn windows(&self) -> &[FloatingWindow] {
        &self.windows
    }

    /// Returns a reference to a specific floating window.
    pub fn get(&self, id: FloatingWindowId) -> Option<&FloatingWindow> {
        self.windows.iter().find(|w| w.id == id)
    }

    /// Returns a mutable reference to a specific floating window.
    pub fn get_mut(&mut self, id: FloatingWindowId) -> Option<&mut FloatingWindow> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    /// Creates a new floating window for a panel at the cascade position.
    ///
    /// # Errors
    ///
    /// Returns `MaxFloatingWindows` if the limit has been reached.
    pub fn create_window(
        &mut self,
        panel_id: &str,
        size: Size,
        origin_zone: DockZone,
    ) -> Result<FloatingWindowId, LayoutError> {
        if self.windows.len() >= self.max_windows {
            return Err(LayoutError::MaxFloatingWindows {
                max: self.max_windows,
            });
        }

        let id = self.next_window_id();
        let position = self.cascade_position();
        let window =
            FloatingWindow::new(id, vec![panel_id.to_string()], position, size, origin_zone);
        self.windows.push(window);
        Ok(id)
    }

    /// Creates a new floating window at a specific position.
    ///
    /// # Errors
    ///
    /// Returns `MaxFloatingWindows` if the limit has been reached.
    pub fn create_window_at(
        &mut self,
        panel_id: &str,
        position: Position,
        size: Size,
        origin_zone: DockZone,
    ) -> Result<FloatingWindowId, LayoutError> {
        if self.windows.len() >= self.max_windows {
            return Err(LayoutError::MaxFloatingWindows {
                max: self.max_windows,
            });
        }

        let id = self.next_window_id();
        let window =
            FloatingWindow::new(id, vec![panel_id.to_string()], position, size, origin_zone);
        self.windows.push(window);
        Ok(id)
    }

    /// Removes a floating window and returns it.
    pub fn remove_window(&mut self, id: FloatingWindowId) -> Option<FloatingWindow> {
        let pos = self.windows.iter().position(|w| w.id == id)?;
        Some(self.windows.remove(pos))
    }

    /// Updates a floating window's position and size.
    pub fn update_window(
        &mut self,
        id: FloatingWindowId,
        position: Position,
        size: Size,
    ) -> Result<(), LayoutError> {
        let window = self
            .get_mut(id)
            .ok_or(LayoutError::FloatingWindowNotFound { window_id: id })?;
        window.update_position_size(position, size);
        Ok(())
    }

    /// Calculates the cascade position for the next floating window.
    fn cascade_position(&self) -> Position {
        let n = self.windows.len() as f32 + 1.0;
        Position::new(Self::CASCADE_OFFSET * n, Self::CASCADE_OFFSET * n)
    }

    /// Allocates the next window ID.
    fn next_window_id(&mut self) -> FloatingWindowId {
        let id = FloatingWindowId::new(self.next_id);
        self.next_id += 1;
        id
    }
}

impl Default for FloatingWindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_has_no_windows() {
        let mgr = FloatingWindowManager::new();
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn create_window_increments_count() {
        // Validates: Requirement 3 criterion 1
        let mut mgr = FloatingWindowManager::new();
        let id = mgr
            .create_window("panel_a", Size::new(400.0, 300.0), DockZone::Left)
            .unwrap();
        assert_eq!(mgr.count(), 1);
        assert!(mgr.get(id).is_some());
    }

    #[test]
    fn create_window_uses_cascade_position() {
        // Validates: Requirement 3 criterion 2
        let mut mgr = FloatingWindowManager::new();
        let id1 = mgr
            .create_window("panel_a", Size::new(400.0, 300.0), DockZone::Left)
            .unwrap();
        let id2 = mgr
            .create_window("panel_b", Size::new(400.0, 300.0), DockZone::Right)
            .unwrap();

        let w1 = mgr.get(id1).unwrap();
        let w2 = mgr.get(id2).unwrap();

        // First window at offset 1, second at offset 2
        assert_eq!(w1.position.x, 50.0);
        assert_eq!(w1.position.y, 50.0);
        assert_eq!(w2.position.x, 100.0);
        assert_eq!(w2.position.y, 100.0);
    }

    #[test]
    fn create_window_enforces_max_limit() {
        // Validates: Requirement 3 criterion 14
        let mut mgr = FloatingWindowManager::new();
        for i in 0..MAX_FLOATING_WINDOWS {
            mgr.create_window(
                &format!("panel_{i}"),
                Size::new(400.0, 300.0),
                DockZone::Left,
            )
            .unwrap();
        }
        let result = mgr.create_window("panel_overflow", Size::new(400.0, 300.0), DockZone::Left);
        assert!(matches!(
            result,
            Err(LayoutError::MaxFloatingWindows { max: 16 })
        ));
    }

    #[test]
    fn remove_window_decrements_count() {
        let mut mgr = FloatingWindowManager::new();
        let id = mgr
            .create_window("panel_a", Size::new(400.0, 300.0), DockZone::Left)
            .unwrap();
        let removed = mgr.remove_window(id);
        assert!(removed.is_some());
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn update_window_changes_position_and_size() {
        // Validates: Requirement 3 criterion 4
        let mut mgr = FloatingWindowManager::new();
        let id = mgr
            .create_window("panel_a", Size::new(400.0, 300.0), DockZone::Left)
            .unwrap();
        mgr.update_window(id, Position::new(200.0, 150.0), Size::new(500.0, 400.0))
            .unwrap();
        let window = mgr.get(id).unwrap();
        assert_eq!(window.position, Position::new(200.0, 150.0));
        assert_eq!(window.size.width, 500.0);
        assert_eq!(window.size.height, 400.0);
    }

    #[test]
    fn create_window_at_uses_specified_position() {
        let mut mgr = FloatingWindowManager::new();
        let id = mgr
            .create_window_at(
                "panel_a",
                Position::new(300.0, 200.0),
                Size::new(400.0, 300.0),
                DockZone::Left,
            )
            .unwrap();
        let window = mgr.get(id).unwrap();
        assert_eq!(window.position, Position::new(300.0, 200.0));
    }
}
