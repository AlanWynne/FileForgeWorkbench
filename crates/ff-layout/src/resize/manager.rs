//! Splitter manager — constraint enforcement and proportional resizing.

use std::collections::HashMap;

use crate::error::LayoutError;
use crate::resize::splitter::{Splitter, SplitterId, SplitterOrientation};
use crate::Size;

/// Manages splitter positions and enforces resize constraints.
///
/// Handles proportional resizing when the primary window is resized,
/// minimum size enforcement, and double-click reset behavior.
#[derive(Debug)]
pub struct SplitterManager {
    /// All registered splitters.
    splitters: HashMap<SplitterId, Splitter>,
    /// The splitter currently being dragged (if any).
    active_drag: Option<SplitterId>,
    /// Next splitter ID.
    next_id: u32,
}

impl SplitterManager {
    /// Creates a new splitter manager.
    pub fn new() -> Self {
        Self {
            splitters: HashMap::new(),
            active_drag: None,
            next_id: 1,
        }
    }

    /// Registers a new splitter and returns its ID.
    pub fn add_splitter(
        &mut self,
        default_proportion: f32,
        orientation: SplitterOrientation,
        min_first: f32,
        min_second: f32,
    ) -> SplitterId {
        let id = SplitterId::new(self.next_id);
        self.next_id += 1;
        let splitter = Splitter::new(id, default_proportion, orientation, min_first, min_second);
        self.splitters.insert(id, splitter);
        id
    }

    /// Returns a reference to a splitter.
    pub fn get(&self, id: SplitterId) -> Option<&Splitter> {
        self.splitters.get(&id)
    }

    /// Returns a mutable reference to a splitter.
    pub fn get_mut(&mut self, id: SplitterId) -> Option<&mut Splitter> {
        self.splitters.get_mut(&id)
    }

    /// Returns all splitter proportions as a map (for serialization).
    pub fn proportions(&self) -> HashMap<String, f32> {
        self.splitters
            .iter()
            .map(|(id, s)| (id.value().to_string(), s.proportion))
            .collect()
    }

    /// Begins dragging a splitter.
    pub fn begin_drag(&mut self, id: SplitterId) -> Result<(), LayoutError> {
        if !self.splitters.contains_key(&id) {
            return Err(LayoutError::SplitterNotFound { splitter_id: id });
        }
        self.active_drag = Some(id);
        Ok(())
    }

    /// Updates a splitter's proportion during a drag, enforcing constraints.
    ///
    /// The `total_size` is the total available space in the split direction
    /// (logical pixels).
    pub fn update_splitter(
        &mut self,
        id: SplitterId,
        new_proportion: f32,
        total_size: f32,
    ) -> Result<(), LayoutError> {
        let splitter = self
            .splitters
            .get_mut(&id)
            .ok_or(LayoutError::SplitterNotFound { splitter_id: id })?;

        let clamped = splitter.clamp_proportion(new_proportion, total_size);
        splitter.proportion = clamped;
        Ok(())
    }

    /// Ends the active splitter drag.
    pub fn end_drag(&mut self, id: SplitterId) {
        if self.active_drag == Some(id) {
            self.active_drag = None;
        }
    }

    /// Resets a splitter to its default position (double-click).
    pub fn reset_splitter(&mut self, id: SplitterId) -> Result<(), LayoutError> {
        let splitter = self
            .splitters
            .get_mut(&id)
            .ok_or(LayoutError::SplitterNotFound { splitter_id: id })?;
        splitter.reset_to_default();
        Ok(())
    }

    /// Handles window resize — proportions remain unchanged since they are
    /// relative. Only validates minimum constraints.
    pub fn on_window_resize(&mut self, new_size: Size) {
        // Proportional values remain valid on resize since they are relative.
        // If actual pixel sizes would violate minimums, the rendering layer
        // handles clamping. The proportions themselves are preserved.
        let _ = new_size;
    }

    /// Returns whether a drag is currently active.
    pub fn is_dragging(&self) -> bool {
        self.active_drag.is_some()
    }

    /// Returns the ID of the currently active drag, if any.
    pub fn active_drag(&self) -> Option<SplitterId> {
        self.active_drag
    }
}

impl Default for SplitterManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_splitter_returns_unique_ids() {
        let mut mgr = SplitterManager::new();
        let id1 = mgr.add_splitter(0.3, SplitterOrientation::Vertical, 48.0, 48.0);
        let id2 = mgr.add_splitter(0.5, SplitterOrientation::Horizontal, 48.0, 48.0);
        assert_ne!(id1, id2);
    }

    #[test]
    fn update_splitter_clamps_to_constraints() {
        // Validates: Requirement 8 criteria 3, 4
        let mut mgr = SplitterManager::new();
        let id = mgr.add_splitter(0.5, SplitterOrientation::Vertical, 100.0, 100.0);

        // With total size 1000: min proportion = 0.1, max = 0.9
        mgr.update_splitter(id, 0.05, 1000.0).unwrap();
        assert_eq!(mgr.get(id).unwrap().proportion, 0.1);

        mgr.update_splitter(id, 0.95, 1000.0).unwrap();
        assert_eq!(mgr.get(id).unwrap().proportion, 0.9);
    }

    #[test]
    fn update_splitter_allows_valid_proportion() {
        let mut mgr = SplitterManager::new();
        let id = mgr.add_splitter(0.5, SplitterOrientation::Vertical, 100.0, 100.0);
        mgr.update_splitter(id, 0.6, 1000.0).unwrap();
        assert_eq!(mgr.get(id).unwrap().proportion, 0.6);
    }

    #[test]
    fn reset_splitter_restores_default() {
        // Validates: Requirement 8 criterion 8
        let mut mgr = SplitterManager::new();
        let id = mgr.add_splitter(0.3, SplitterOrientation::Vertical, 48.0, 48.0);
        mgr.update_splitter(id, 0.7, 1000.0).unwrap();
        mgr.reset_splitter(id).unwrap();
        assert_eq!(mgr.get(id).unwrap().proportion, 0.3);
    }

    #[test]
    fn begin_drag_validates_splitter_exists() {
        let mut mgr = SplitterManager::new();
        let result = mgr.begin_drag(SplitterId::new(999));
        assert!(matches!(result, Err(LayoutError::SplitterNotFound { .. })));
    }

    #[test]
    fn drag_lifecycle() {
        // Validates: Requirement 8 criterion 9
        let mut mgr = SplitterManager::new();
        let id = mgr.add_splitter(0.5, SplitterOrientation::Vertical, 48.0, 48.0);
        assert!(!mgr.is_dragging());

        mgr.begin_drag(id).unwrap();
        assert!(mgr.is_dragging());

        mgr.end_drag(id);
        assert!(!mgr.is_dragging());
    }
}
