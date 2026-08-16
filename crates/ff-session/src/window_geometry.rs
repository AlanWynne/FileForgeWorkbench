//! Window geometry persistence and display validation — position tracking,
//! off-screen clamping, and display-disconnect fallback.
//!
//! Addresses: Requirement 8 (Window Geometry Persistence)

use crate::session_state::WindowGeometryState;

/// Describes the usable bounds of a display/monitor.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayBounds {
    /// Horizontal origin of the display in logical pixels.
    pub x: i32,
    /// Vertical origin of the display in logical pixels.
    pub y: i32,
    /// Width of the display in logical pixels.
    pub width: u32,
    /// Height of the display in logical pixels.
    pub height: u32,
    /// Identifier for this display.
    pub display_id: Option<String>,
}

impl DisplayBounds {
    /// Create display bounds for the primary monitor.
    pub fn primary(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
            display_id: None,
        }
    }

    /// Check whether a point is within this display's bounds.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width as i32
            && y < self.y + self.height as i32
    }

    /// The right edge x-coordinate.
    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    /// The bottom edge y-coordinate.
    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }
}

/// Minimum window dimension (width or height) after clamping.
const MIN_WINDOW_SIZE: u32 = 100;

/// Clamp a window geometry to fit within the given display bounds.
///
/// Ensures the window is fully visible on the display after clamping:
/// - Window width/height are clamped to not exceed display dimensions
/// - Window position is adjusted so no part extends beyond display edges
///
/// Addresses: Requirement 8 AC 8.4, 8.5
pub fn clamp_to_display(
    geometry: &WindowGeometryState,
    display: &DisplayBounds,
) -> WindowGeometryState {
    let mut result = geometry.clone();

    // Clamp width and height to display dimensions (minimum MIN_WINDOW_SIZE)
    result.width = result.width.min(display.width).max(MIN_WINDOW_SIZE);
    result.height = result.height.min(display.height).max(MIN_WINDOW_SIZE);

    // Clamp x so the window fits horizontally within the display
    let max_x = display.x + display.width as i32 - result.width as i32;
    result.x = result.x.max(display.x).min(max_x);

    // Clamp y so the window fits vertically within the display
    let max_y = display.y + display.height as i32 - result.height as i32;
    result.y = result.y.max(display.y).min(max_y);

    result
}

/// Check whether a window geometry is fully visible on the given display.
///
/// Returns true if the entire window rectangle fits within the display bounds.
pub fn is_visible_on(geometry: &WindowGeometryState, display: &DisplayBounds) -> bool {
    geometry.x >= display.x
        && geometry.y >= display.y
        && geometry.x + geometry.width as i32 <= display.right()
        && geometry.y + geometry.height as i32 <= display.bottom()
}

/// Find the display that matches the window's stored display_id.
///
/// Returns `None` if no matching display is found (display disconnected).
pub fn find_target_display<'a>(
    geometry: &WindowGeometryState,
    available_displays: &'a [DisplayBounds],
) -> Option<&'a DisplayBounds> {
    if let Some(ref stored_id) = geometry.display_id {
        available_displays
            .iter()
            .find(|d| d.display_id.as_deref() == Some(stored_id.as_str()))
    } else {
        // No stored display_id — use the first available (primary)
        available_displays.first()
    }
}

/// Restore window geometry with display validation.
///
/// - If the target display is connected, clamp to that display
/// - If the target display is disconnected, fallback to primary display (centred)
/// - Always ensures window is fully on-screen
///
/// Addresses: Requirement 8 AC 8.3, 8.4, 8.5
pub fn restore_geometry(
    geometry: &WindowGeometryState,
    available_displays: &[DisplayBounds],
) -> WindowGeometryState {
    if available_displays.is_empty() {
        // No displays available (shouldn't happen in practice) — return as-is
        return geometry.clone();
    }

    // Try to find the target display
    let target = find_target_display(geometry, available_displays);

    match target {
        Some(display) => {
            // Target display found — clamp to it
            clamp_to_display(geometry, display)
        }
        None => {
            // Target display disconnected — fallback to primary (first) display, centred
            let primary = &available_displays[0];
            let width = geometry.width.min(primary.width).max(MIN_WINDOW_SIZE);
            let height = geometry.height.min(primary.height).max(MIN_WINDOW_SIZE);

            // Centre on primary display
            let x = primary.x + (primary.width as i32 - width as i32) / 2;
            let y = primary.y + (primary.height as i32 - height as i32) / 2;

            WindowGeometryState {
                window_id: geometry.window_id.clone(),
                x,
                y,
                width,
                height,
                is_maximised: geometry.is_maximised,
                is_fullscreen: geometry.is_fullscreen,
                display_id: primary.display_id.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn primary_display() -> DisplayBounds {
        DisplayBounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            display_id: Some("primary".to_string()),
        }
    }

    fn secondary_display() -> DisplayBounds {
        DisplayBounds {
            x: 1920,
            y: 0,
            width: 2560,
            height: 1440,
            display_id: Some("secondary".to_string()),
        }
    }

    fn window_on_secondary() -> WindowGeometryState {
        WindowGeometryState {
            window_id: "primary".to_string(),
            x: 2000,
            y: 100,
            width: 1200,
            height: 800,
            is_maximised: false,
            is_fullscreen: false,
            display_id: Some("secondary".to_string()),
        }
    }

    #[test]
    fn clamp_to_display_keeps_visible_window_unchanged() {
        // Validates: Requirement 8 AC 8.5
        let display = primary_display();
        let geom = WindowGeometryState::primary(100, 100, 800, 600);

        let clamped = clamp_to_display(&geom, &display);
        assert_eq!(clamped.x, 100);
        assert_eq!(clamped.y, 100);
        assert_eq!(clamped.width, 800);
        assert_eq!(clamped.height, 600);
    }

    #[test]
    fn clamp_to_display_fixes_negative_position() {
        // Validates: Requirement 8 AC 8.5
        let display = primary_display();
        let geom = WindowGeometryState {
            x: -500,
            y: -200,
            ..WindowGeometryState::primary(0, 0, 800, 600)
        };

        let clamped = clamp_to_display(&geom, &display);
        assert_eq!(clamped.x, 0);
        assert_eq!(clamped.y, 0);
    }

    #[test]
    fn clamp_to_display_fixes_overflow_right() {
        // Validates: Requirement 8 AC 8.5
        let display = primary_display();
        let geom = WindowGeometryState::primary(1800, 100, 800, 600);

        let clamped = clamp_to_display(&geom, &display);
        // x should be at most 1920-800 = 1120
        assert_eq!(clamped.x, 1120);
        assert!(clamped.x + clamped.width as i32 <= display.right());
    }

    #[test]
    fn clamp_to_display_fixes_overflow_bottom() {
        // Validates: Requirement 8 AC 8.5
        let display = primary_display();
        let geom = WindowGeometryState::primary(100, 900, 800, 600);

        let clamped = clamp_to_display(&geom, &display);
        // y should be at most 1080-600 = 480
        assert_eq!(clamped.y, 480);
        assert!(clamped.y + clamped.height as i32 <= display.bottom());
    }

    #[test]
    fn clamp_to_display_shrinks_oversized_window() {
        // Validates: Requirement 8 AC 8.5
        let display = primary_display();
        let geom = WindowGeometryState::primary(0, 0, 5000, 3000);

        let clamped = clamp_to_display(&geom, &display);
        assert_eq!(clamped.width, 1920);
        assert_eq!(clamped.height, 1080);
    }

    #[test]
    fn clamp_to_display_enforces_minimum_size() {
        let display = primary_display();
        let geom = WindowGeometryState::primary(100, 100, 10, 10);

        let clamped = clamp_to_display(&geom, &display);
        assert!(clamped.width >= MIN_WINDOW_SIZE);
        assert!(clamped.height >= MIN_WINDOW_SIZE);
    }

    #[test]
    fn is_visible_on_detects_fully_visible_window() {
        let display = primary_display();
        let geom = WindowGeometryState::primary(100, 100, 800, 600);
        assert!(is_visible_on(&geom, &display));
    }

    #[test]
    fn is_visible_on_detects_partially_offscreen_window() {
        let display = primary_display();
        let geom = WindowGeometryState::primary(1500, 100, 800, 600);
        assert!(!is_visible_on(&geom, &display));
    }

    #[test]
    fn restore_geometry_on_connected_display_clamps_to_target() {
        // Validates: Requirement 8 AC 8.3
        let displays = vec![primary_display(), secondary_display()];
        let geom = window_on_secondary();

        let restored = restore_geometry(&geom, &displays);
        // Should be on secondary display and fully visible
        assert!(is_visible_on(&restored, &secondary_display()));
    }

    #[test]
    fn restore_geometry_on_disconnected_display_falls_back_to_primary() {
        // Validates: Requirement 8 AC 8.4
        let displays = vec![primary_display()]; // Only primary available
        let geom = window_on_secondary(); // Was on secondary

        let restored = restore_geometry(&geom, &displays);
        // Should be centred on primary display
        assert!(is_visible_on(&restored, &primary_display()));
        // Should be centred
        let center_x = (1920 - restored.width as i32) / 2;
        let center_y = (1080 - restored.height as i32) / 2;
        assert_eq!(restored.x, center_x);
        assert_eq!(restored.y, center_y);
    }

    #[test]
    fn restore_geometry_preserves_maximised_state() {
        let displays = vec![primary_display()];
        let geom = WindowGeometryState {
            is_maximised: true,
            ..WindowGeometryState::primary(100, 100, 800, 600)
        };

        let restored = restore_geometry(&geom, &displays);
        assert!(restored.is_maximised);
    }

    #[test]
    fn display_bounds_contains_point_works() {
        let display = primary_display();
        assert!(display.contains_point(0, 0));
        assert!(display.contains_point(1919, 1079));
        assert!(!display.contains_point(1920, 0));
        assert!(!display.contains_point(0, 1080));
        assert!(!display.contains_point(-1, 0));
    }

    #[test]
    fn find_target_display_finds_matching_id() {
        let displays = vec![primary_display(), secondary_display()];
        let geom = window_on_secondary();

        let target = find_target_display(&geom, &displays);
        assert!(target.is_some());
        assert_eq!(target.unwrap().display_id, Some("secondary".to_string()));
    }

    #[test]
    fn find_target_display_returns_none_for_missing_id() {
        let displays = vec![primary_display()];
        let geom = window_on_secondary();

        let target = find_target_display(&geom, &displays);
        assert!(target.is_none());
    }

    #[test]
    fn clamp_handles_display_with_offset_origin() {
        // Secondary display at x=1920
        let display = secondary_display();
        let geom = WindowGeometryState {
            window_id: "primary".to_string(),
            x: 1800, // Partially before secondary display starts
            y: 50,
            width: 800,
            height: 600,
            is_maximised: false,
            is_fullscreen: false,
            display_id: Some("secondary".to_string()),
        };

        let clamped = clamp_to_display(&geom, &display);
        assert!(clamped.x >= display.x);
        assert!(clamped.x + clamped.width as i32 <= display.right());
        assert!(clamped.y >= display.y);
        assert!(clamped.y + clamped.height as i32 <= display.bottom());
    }
}
