//! Drag-and-drop state machine coordinator.

use crate::drag::indicator::DropIndicator;
use crate::floating::window::FloatingWindowId;
use crate::tabs::group::TabGroupId;
use crate::Position;

/// Items that can be dragged in the layout system.
#[derive(Debug, Clone, PartialEq)]
pub enum DragItem {
    /// A docked panel being dragged from its header.
    Panel {
        /// The panel_id of the dragged panel.
        panel_id: String,
    },
    /// A tab being dragged from a tab group.
    Tab {
        /// The group the tab originated from.
        group_id: TabGroupId,
        /// The index of the tab in its origin group.
        tab_index: usize,
    },
    /// A floating window being dragged by its title bar.
    FloatingWindow {
        /// The ID of the floating window being dragged.
        window_id: FloatingWindowId,
    },
}

/// The current phase of a drag-and-drop operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DragPhase {
    /// No drag in progress.
    Idle,
    /// Actively dragging — tracking cursor position.
    Dragging {
        /// The item being dragged.
        item: DragItem,
        /// The position where the drag started.
        start: Position,
        /// The current cursor position.
        current: Position,
    },
    /// Preview phase — showing tear-off thumbnail.
    TearOffPreview {
        /// The item being torn off.
        item: DragItem,
        /// Current cursor position.
        current: Position,
    },
}

/// Result of a completed drag operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DragResult {
    /// Item was docked into a zone.
    Docked {
        /// The panel_id that was docked.
        panel_id: String,
        /// The zone it was docked into.
        zone: crate::dock::zone::DockZone,
    },
    /// Tab was moved to a different group.
    TabMoved {
        /// The tab identifier.
        tab_id: String,
        /// The target group.
        target_group: TabGroupId,
        /// The insertion index.
        index: usize,
    },
    /// Item was floated at the release position.
    Floated {
        /// The new floating window ID.
        window_id: FloatingWindowId,
    },
    /// Drag was cancelled (released in invalid location).
    Cancelled,
}

/// Coordinates drag-and-drop operations across the layout system.
///
/// Manages the state machine for drag gestures, including tab tear-off
/// detection and drop indicator placement.
#[derive(Debug)]
pub struct DragDropCoordinator {
    /// Current drag phase.
    phase: DragPhase,
    /// The current drop indicator (if any).
    indicator: Option<DropIndicator>,
    /// Tab tear-off threshold in logical pixels.
    tear_off_threshold: f32,
    /// Distance outside window boundary for drag-to-float.
    float_threshold: f32,
}

impl DragDropCoordinator {
    /// Tab tear-off distance threshold in logical pixels.
    pub const TEAR_OFF_THRESHOLD: f32 = 30.0;
    /// Distance outside primary window for drag-to-float.
    pub const FLOAT_THRESHOLD: f32 = 20.0;

    /// Creates a new drag-drop coordinator.
    pub fn new() -> Self {
        Self {
            phase: DragPhase::Idle,
            indicator: None,
            tear_off_threshold: Self::TEAR_OFF_THRESHOLD,
            float_threshold: Self::FLOAT_THRESHOLD,
        }
    }

    /// Returns whether a drag is currently in progress.
    pub fn is_dragging(&self) -> bool {
        !matches!(self.phase, DragPhase::Idle)
    }

    /// Returns the current drop indicator for rendering.
    pub fn current_indicator(&self) -> Option<&DropIndicator> {
        self.indicator.as_ref()
    }

    /// Returns the current drag phase.
    pub fn phase(&self) -> &DragPhase {
        &self.phase
    }

    /// Begin a drag operation.
    pub fn begin_drag(&mut self, item: DragItem, origin: Position) {
        self.phase = DragPhase::Dragging {
            item,
            start: origin,
            current: origin,
        };
        self.indicator = None;
    }

    /// Update the cursor position during a drag.
    pub fn update_position(&mut self, cursor: Position) {
        match &mut self.phase {
            DragPhase::Dragging { current, .. } => {
                *current = cursor;
            }
            DragPhase::TearOffPreview { current, .. } => {
                *current = cursor;
            }
            DragPhase::Idle => {}
        }
    }

    /// Set the current drop indicator.
    pub fn set_indicator(&mut self, indicator: Option<DropIndicator>) {
        self.indicator = indicator;
    }

    /// Cancel the current drag operation.
    pub fn cancel(&mut self) {
        self.phase = DragPhase::Idle;
        self.indicator = None;
    }

    /// Complete the drag operation and return to idle.
    pub fn complete(&mut self) {
        self.phase = DragPhase::Idle;
        self.indicator = None;
    }

    /// Transition to tear-off preview state.
    pub fn begin_tear_off(&mut self, item: DragItem, cursor: Position) {
        self.phase = DragPhase::TearOffPreview {
            item,
            current: cursor,
        };
        self.indicator = None;
    }

    /// Returns the tear-off threshold distance.
    pub fn tear_off_threshold(&self) -> f32 {
        self.tear_off_threshold
    }

    /// Returns the float threshold distance.
    pub fn float_threshold(&self) -> f32 {
        self.float_threshold
    }
}

impl Default for DragDropCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinator_starts_idle() {
        let coord = DragDropCoordinator::new();
        assert!(!coord.is_dragging());
        assert!(coord.current_indicator().is_none());
    }

    #[test]
    fn begin_drag_transitions_to_dragging() {
        let mut coord = DragDropCoordinator::new();
        coord.begin_drag(
            DragItem::Panel {
                panel_id: "test".to_string(),
            },
            Position::new(10.0, 20.0),
        );
        assert!(coord.is_dragging());
    }

    #[test]
    fn cancel_returns_to_idle() {
        let mut coord = DragDropCoordinator::new();
        coord.begin_drag(
            DragItem::Panel {
                panel_id: "test".to_string(),
            },
            Position::new(10.0, 20.0),
        );
        coord.cancel();
        assert!(!coord.is_dragging());
    }

    #[test]
    fn update_position_tracks_cursor() {
        let mut coord = DragDropCoordinator::new();
        coord.begin_drag(
            DragItem::Panel {
                panel_id: "test".to_string(),
            },
            Position::new(10.0, 20.0),
        );
        coord.update_position(Position::new(50.0, 60.0));
        if let DragPhase::Dragging { current, .. } = coord.phase() {
            assert_eq!(*current, Position::new(50.0, 60.0));
        } else {
            panic!("Expected Dragging phase");
        }
    }
}
