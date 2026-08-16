//! Multi-monitor support — monitor detection, DPI tracking, repositioning.

use crate::{Position, Rect, Size};

/// Information about a connected monitor for positioning decisions.
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    /// Unique identifier for this monitor.
    pub id: String,
    /// Whether this is the primary monitor.
    pub is_primary: bool,
    /// Work area bounds (excluding taskbar etc.).
    pub work_area: Rect,
    /// DPI scale factor for this monitor.
    pub dpi_scale: f32,
}

impl MonitorInfo {
    /// Creates a new monitor info.
    pub fn new(id: &str, is_primary: bool, work_area: Rect, dpi_scale: f32) -> Self {
        Self {
            id: id.to_string(),
            is_primary,
            work_area,
            dpi_scale,
        }
    }

    /// Returns the center position of this monitor's work area.
    pub fn center(&self) -> Position {
        self.work_area.center()
    }
}

/// Checks whether at least 50% of a window is visible on any monitor.
///
/// Returns true if the window has at least 50% of its area overlapping
/// with any monitor's work area.
pub fn is_window_sufficiently_visible(
    window_pos: Position,
    window_size: Size,
    monitors: &[MonitorInfo],
) -> bool {
    let window_rect = Rect::new(
        window_pos.x,
        window_pos.y,
        window_size.width,
        window_size.height,
    );
    let window_area = window_rect.area();
    if window_area <= 0.0 {
        return false;
    }

    let total_visible: f32 = monitors
        .iter()
        .map(|m| window_rect.overlap_area(&m.work_area))
        .sum();

    total_visible >= window_area * 0.5
}

/// Finds the primary monitor from a list, or the first available.
pub fn find_primary_monitor(monitors: &[MonitorInfo]) -> Option<&MonitorInfo> {
    monitors
        .iter()
        .find(|m| m.is_primary)
        .or_else(|| monitors.first())
}

/// Calculates the center position on the primary monitor for repositioning.
pub fn center_on_primary(window_size: Size, monitors: &[MonitorInfo]) -> Option<Position> {
    let primary = find_primary_monitor(monitors)?;
    Some(Position::new(
        primary.work_area.x + (primary.work_area.width - window_size.width) / 2.0,
        primary.work_area.y + (primary.work_area.height - window_size.height) / 2.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn primary_monitor() -> MonitorInfo {
        MonitorInfo::new("primary", true, Rect::new(0.0, 0.0, 1920.0, 1080.0), 1.0)
    }

    fn secondary_monitor() -> MonitorInfo {
        MonitorInfo::new(
            "secondary",
            false,
            Rect::new(1920.0, 0.0, 2560.0, 1440.0),
            1.5,
        )
    }

    #[test]
    fn window_fully_on_primary_is_visible() {
        // Validates: Requirement 4 criterion 8
        let monitors = vec![primary_monitor()];
        assert!(is_window_sufficiently_visible(
            Position::new(100.0, 100.0),
            Size::new(400.0, 300.0),
            &monitors,
        ));
    }

    #[test]
    fn window_mostly_off_screen_is_not_visible() {
        // Validates: Requirement 4 criterion 8
        let monitors = vec![primary_monitor()];
        // Window is mostly off the right edge
        assert!(!is_window_sufficiently_visible(
            Position::new(1800.0, 100.0),
            Size::new(400.0, 300.0),
            &monitors,
        ));
    }

    #[test]
    fn window_on_secondary_monitor_is_visible() {
        let monitors = vec![primary_monitor(), secondary_monitor()];
        assert!(is_window_sufficiently_visible(
            Position::new(2000.0, 100.0),
            Size::new(400.0, 300.0),
            &monitors,
        ));
    }

    #[test]
    fn center_on_primary_positions_correctly() {
        // Validates: Requirement 4 criterion 7
        let monitors = vec![primary_monitor()];
        let pos = center_on_primary(Size::new(400.0, 300.0), &monitors).unwrap();
        assert_eq!(pos.x, (1920.0 - 400.0) / 2.0);
        assert_eq!(pos.y, (1080.0 - 300.0) / 2.0);
    }

    #[test]
    fn find_primary_falls_back_to_first() {
        let monitors = vec![MonitorInfo::new(
            "only",
            false,
            Rect::new(0.0, 0.0, 1920.0, 1080.0),
            1.0,
        )];
        let primary = find_primary_monitor(&monitors).unwrap();
        assert_eq!(primary.id, "only");
    }
}
