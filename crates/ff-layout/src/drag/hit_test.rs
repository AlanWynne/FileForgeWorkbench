//! Hit testing for drag-and-drop — determines valid drop targets.

use crate::dock::zone::DockZone;
use crate::drag::indicator::{DropPlacement, SplitSide};
use crate::tabs::group::SplitDirection;
use crate::{Position, Rect};

/// A potential drop target with its bounds and placement.
#[derive(Debug, Clone, PartialEq)]
pub struct DropTarget {
    /// The screen-space bounds of this target.
    pub bounds: Rect,
    /// What kind of placement this target represents.
    pub placement: DropPlacement,
}

/// Performs hit testing against a set of registered drop targets.
///
/// Returns the first target whose bounds contain the given position.
pub fn hit_test(targets: &[DropTarget], cursor: Position) -> Option<&DropTarget> {
    targets.iter().find(|target| target.bounds.contains(cursor))
}

/// Calculates the tab insertion index based on horizontal cursor position
/// within a tab bar.
///
/// Divides the tab bar into equal segments and returns the index where
/// a tab should be inserted.
pub fn calculate_tab_insertion_index(
    tab_bar_x: f32,
    tab_bar_width: f32,
    tab_count: usize,
    cursor_x: f32,
) -> usize {
    if tab_count == 0 {
        return 0;
    }
    let tab_width = tab_bar_width / tab_count as f32;
    let relative_x = (cursor_x - tab_bar_x).max(0.0);
    let index = (relative_x / tab_width) as usize;
    index.min(tab_count)
}

/// Determines which split side a cursor is on relative to a target area.
///
/// Used for determining whether a drop should create a left/right or top/bottom split.
pub fn determine_split_side(
    bounds: &Rect,
    cursor: Position,
    direction: SplitDirection,
) -> SplitSide {
    match direction {
        SplitDirection::Horizontal => {
            let midpoint = bounds.x + bounds.width / 2.0;
            if cursor.x < midpoint {
                SplitSide::First
            } else {
                SplitSide::Second
            }
        }
        SplitDirection::Vertical => {
            let midpoint = bounds.y + bounds.height / 2.0;
            if cursor.y < midpoint {
                SplitSide::First
            } else {
                SplitSide::Second
            }
        }
    }
}

/// Builds a list of dock zone drop targets for the primary window.
pub fn build_dock_zone_targets(
    left_bounds: Rect,
    right_bounds: Rect,
    bottom_bounds: Rect,
    center_bounds: Rect,
) -> Vec<DropTarget> {
    vec![
        DropTarget {
            bounds: left_bounds,
            placement: DropPlacement::DockZone(DockZone::Left),
        },
        DropTarget {
            bounds: right_bounds,
            placement: DropPlacement::DockZone(DockZone::Right),
        },
        DropTarget {
            bounds: bottom_bounds,
            placement: DropPlacement::DockZone(DockZone::Bottom),
        },
        DropTarget {
            bounds: center_bounds,
            placement: DropPlacement::DockZone(DockZone::Center),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_test_returns_matching_target() {
        let targets = vec![
            DropTarget {
                bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
                placement: DropPlacement::DockZone(DockZone::Left),
            },
            DropTarget {
                bounds: Rect::new(100.0, 0.0, 100.0, 100.0),
                placement: DropPlacement::DockZone(DockZone::Right),
            },
        ];
        let result = hit_test(&targets, Position::new(50.0, 50.0));
        assert_eq!(
            result.unwrap().placement,
            DropPlacement::DockZone(DockZone::Left)
        );
    }

    #[test]
    fn hit_test_returns_none_for_miss() {
        let targets = vec![DropTarget {
            bounds: Rect::new(0.0, 0.0, 100.0, 100.0),
            placement: DropPlacement::DockZone(DockZone::Left),
        }];
        let result = hit_test(&targets, Position::new(200.0, 200.0));
        assert!(result.is_none());
    }

    #[test]
    fn calculate_tab_insertion_at_beginning() {
        let index = calculate_tab_insertion_index(0.0, 300.0, 3, 10.0);
        assert_eq!(index, 0);
    }

    #[test]
    fn calculate_tab_insertion_at_end() {
        let index = calculate_tab_insertion_index(0.0, 300.0, 3, 290.0);
        assert_eq!(index, 2);
    }

    #[test]
    fn calculate_tab_insertion_empty_group() {
        let index = calculate_tab_insertion_index(0.0, 300.0, 0, 150.0);
        assert_eq!(index, 0);
    }

    #[test]
    fn determine_split_side_horizontal() {
        let bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
        assert_eq!(
            determine_split_side(
                &bounds,
                Position::new(50.0, 50.0),
                SplitDirection::Horizontal
            ),
            SplitSide::First
        );
        assert_eq!(
            determine_split_side(
                &bounds,
                Position::new(150.0, 50.0),
                SplitDirection::Horizontal
            ),
            SplitSide::Second
        );
    }
}
