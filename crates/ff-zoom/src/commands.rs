//! ZOOM primary command and keyboard shortcut handling.
//!
//! Implements the `ZOOM` command parser and the keyboard shortcut/mouse wheel
//! action types used by the zoom subsystem.

use crate::error::ZoomError;

/// All possible zoom operations that can be dispatched.
///
/// Used by the command handlers and shortcut handlers to route into
/// [`ZoomState`](crate::state::ZoomState) methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoomOperation {
    /// Increase offset by one step.
    ZoomIn,
    /// Decrease offset by one step.
    ZoomOut,
    /// Reset offset to zero.
    Reset,
    /// Set offset to an absolute value (clamped to range).
    SetAbsolute(i32),
    /// Query current state (no mutation — returns info).
    Query,
}

/// Keyboard shortcut actions for zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomShortcutAction {
    /// Ctrl+= → zoom in.
    ZoomIn,
    /// Ctrl+- → zoom out.
    ZoomOut,
    /// Ctrl+0 → reset zoom.
    Reset,
}

/// Scroll direction for Ctrl+Mouse Wheel zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    /// Scroll up (away from user) → zoom in.
    Up,
    /// Scroll down (toward user) → zoom out.
    Down,
}

/// Represents a Ctrl+Scroll zoom action detected from a scroll event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomScrollAction {
    /// The scroll direction.
    pub direction: ScrollDirection,
    /// The editor instance under the cursor (if any).
    pub editor_instance_id: Option<u64>,
}

impl ZoomScrollAction {
    /// Detect a zoom scroll action from input parameters.
    ///
    /// Returns `None` if:
    /// - Ctrl is not held (normal scroll passthrough)
    /// - Mouse cursor is not over any editor instance
    ///
    /// # Arguments
    ///
    /// * `scroll_delta_y` — Vertical scroll delta (positive = up/away from user).
    /// * `ctrl_held` — Whether the Ctrl modifier is active.
    /// * `editor_instance_id` — The editor instance under the cursor, if any.
    pub fn from_scroll_event(
        scroll_delta_y: f32,
        ctrl_held: bool,
        editor_instance_id: Option<u64>,
    ) -> Option<Self> {
        if !ctrl_held {
            return None;
        }
        let editor_id = editor_instance_id?;
        if scroll_delta_y == 0.0 {
            return None;
        }
        let direction = if scroll_delta_y > 0.0 {
            ScrollDirection::Up
        } else {
            ScrollDirection::Down
        };
        Some(Self {
            direction,
            editor_instance_id: Some(editor_id),
        })
    }
}

/// Parse ZOOM command arguments into a [`ZoomOperation`].
///
/// Supported forms:
/// - `""` (empty) → [`ZoomOperation::Query`]
/// - `"IN"` → [`ZoomOperation::ZoomIn`]
/// - `"OUT"` → [`ZoomOperation::ZoomOut`]
/// - `"RESET"` → [`ZoomOperation::Reset`]
/// - `"n"` (signed integer) → [`ZoomOperation::SetAbsolute(n)`]
///
/// # Errors
///
/// Returns [`ZoomError::InvalidCommandArg`] if the argument cannot be parsed.
pub fn parse_zoom_args(args: &str) -> Result<ZoomOperation, ZoomError> {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return Ok(ZoomOperation::Query);
    }

    match trimmed.to_uppercase().as_str() {
        "IN" => Ok(ZoomOperation::ZoomIn),
        "OUT" => Ok(ZoomOperation::ZoomOut),
        "RESET" => Ok(ZoomOperation::Reset),
        _ => {
            // Try to parse as signed integer
            trimmed
                .parse::<i32>()
                .map(ZoomOperation::SetAbsolute)
                .map_err(|_| ZoomError::InvalidCommandArg {
                    arg: trimmed.to_string(),
                })
        }
    }
}

/// Command metadata for zoom commands.
pub struct ZoomCommandIds;

impl ZoomCommandIds {
    /// The primary ZOOM command ID.
    pub const ZOOM: &'static str = "view.zoom";
    /// Zoom in command ID.
    pub const ZOOM_IN: &'static str = "view.zoom_in";
    /// Zoom out command ID.
    pub const ZOOM_OUT: &'static str = "view.zoom_out";
    /// Zoom reset command ID.
    pub const ZOOM_RESET: &'static str = "view.zoom_reset";
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 8.6 — empty args = query
    #[test]
    fn parse_empty_args_returns_query() {
        assert_eq!(parse_zoom_args(""), Ok(ZoomOperation::Query));
        assert_eq!(parse_zoom_args("  "), Ok(ZoomOperation::Query));
    }

    // Validates: Requirement 8.3 — ZOOM IN
    #[test]
    fn parse_in_returns_zoom_in() {
        assert_eq!(parse_zoom_args("IN"), Ok(ZoomOperation::ZoomIn));
        assert_eq!(parse_zoom_args("in"), Ok(ZoomOperation::ZoomIn));
        assert_eq!(parse_zoom_args(" In "), Ok(ZoomOperation::ZoomIn));
    }

    // Validates: Requirement 8.4 — ZOOM OUT
    #[test]
    fn parse_out_returns_zoom_out() {
        assert_eq!(parse_zoom_args("OUT"), Ok(ZoomOperation::ZoomOut));
        assert_eq!(parse_zoom_args("out"), Ok(ZoomOperation::ZoomOut));
    }

    // Validates: Requirement 8.5 — ZOOM RESET
    #[test]
    fn parse_reset_returns_reset() {
        assert_eq!(parse_zoom_args("RESET"), Ok(ZoomOperation::Reset));
        assert_eq!(parse_zoom_args("reset"), Ok(ZoomOperation::Reset));
    }

    // Validates: Requirement 8.2 — ZOOM n (positive)
    #[test]
    fn parse_positive_integer_returns_set_absolute() {
        assert_eq!(parse_zoom_args("5"), Ok(ZoomOperation::SetAbsolute(5)));
        assert_eq!(parse_zoom_args("+3"), Ok(ZoomOperation::SetAbsolute(3)));
    }

    // Validates: Requirement 8.2 — ZOOM n (negative)
    #[test]
    fn parse_negative_integer_returns_set_absolute() {
        assert_eq!(parse_zoom_args("-2"), Ok(ZoomOperation::SetAbsolute(-2)));
    }

    // Validates: Requirement 8.2 — ZOOM 0
    #[test]
    fn parse_zero_returns_set_absolute_zero() {
        assert_eq!(parse_zoom_args("0"), Ok(ZoomOperation::SetAbsolute(0)));
    }

    // Validates: Requirement 8 — invalid argument
    #[test]
    fn parse_invalid_arg_returns_error() {
        let result = parse_zoom_args("abc");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ZoomError::InvalidCommandArg { .. }
        ));
    }

    // Validates: Requirement 3.1 — Ctrl+Scroll up = zoom in
    #[test]
    fn scroll_up_with_ctrl_over_editor_returns_zoom_action() {
        let action = ZoomScrollAction::from_scroll_event(1.0, true, Some(42));
        assert_eq!(
            action,
            Some(ZoomScrollAction {
                direction: ScrollDirection::Up,
                editor_instance_id: Some(42),
            })
        );
    }

    // Validates: Requirement 3.2 — Ctrl+Scroll down = zoom out
    #[test]
    fn scroll_down_with_ctrl_over_editor_returns_zoom_action() {
        let action = ZoomScrollAction::from_scroll_event(-1.0, true, Some(42));
        assert_eq!(
            action,
            Some(ZoomScrollAction {
                direction: ScrollDirection::Down,
                editor_instance_id: Some(42),
            })
        );
    }

    // Validates: Requirement 3.3 — no Ctrl = no zoom
    #[test]
    fn scroll_without_ctrl_returns_none() {
        let action = ZoomScrollAction::from_scroll_event(1.0, false, Some(42));
        assert_eq!(action, None);
    }

    // Validates: Requirement 3.4 — cursor not over editor = no zoom
    #[test]
    fn scroll_with_ctrl_but_no_editor_returns_none() {
        let action = ZoomScrollAction::from_scroll_event(1.0, true, None);
        assert_eq!(action, None);
    }

    // Validates: Requirement 3.5 — zero delta is ignored
    #[test]
    fn scroll_zero_delta_returns_none() {
        let action = ZoomScrollAction::from_scroll_event(0.0, true, Some(42));
        assert_eq!(action, None);
    }
}
