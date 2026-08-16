//! Drop indicator types — visual overlays during drag-and-drop.

use crate::dock::zone::DockZone;
use crate::tabs::group::{SplitDirection, TabGroupId};
use crate::Rect;

/// Visual overlay shown during drag-and-drop to highlight valid drop targets.
///
/// The shell layer renders this indicator as a semi-transparent overlay
/// with a distinct border color.
#[derive(Debug, Clone, PartialEq)]
pub struct DropIndicator {
    /// The target area in logical screen coordinates.
    pub bounds: Rect,
    /// Where the panel/tab will be placed upon release.
    pub placement: DropPlacement,
    /// Whether the indicator is currently visible.
    pub visible: bool,
}

impl DropIndicator {
    /// Creates a new visible drop indicator.
    pub fn new(bounds: Rect, placement: DropPlacement) -> Self {
        Self {
            bounds,
            placement,
            visible: true,
        }
    }
}

/// Describes where a dropped item will be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropPlacement {
    /// Dock into specified zone.
    DockZone(DockZone),
    /// Insert as tab at given index in a tab group.
    TabInsertion {
        /// The target tab group.
        group_id: TabGroupId,
        /// The insertion index.
        index: usize,
    },
    /// Split the target group in the specified direction.
    SplitGroup {
        /// The target tab group to split.
        group_id: TabGroupId,
        /// The direction of the split.
        direction: SplitDirection,
        /// Which side of the split the dropped item goes to.
        side: SplitSide,
    },
}

/// Which side of a split the dropped item goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitSide {
    /// First child (left or top).
    First,
    /// Second child (right or bottom).
    Second,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_indicator_new_is_visible() {
        let indicator = DropIndicator::new(
            Rect::new(10.0, 20.0, 100.0, 200.0),
            DropPlacement::DockZone(DockZone::Left),
        );
        assert!(indicator.visible);
        assert_eq!(indicator.bounds.x, 10.0);
    }

    #[test]
    fn drop_placement_equality() {
        let a = DropPlacement::TabInsertion {
            group_id: TabGroupId::new(1),
            index: 3,
        };
        let b = DropPlacement::TabInsertion {
            group_id: TabGroupId::new(1),
            index: 3,
        };
        assert_eq!(a, b);
    }
}
