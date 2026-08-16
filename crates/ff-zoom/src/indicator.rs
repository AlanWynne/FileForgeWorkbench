//! Status bar zoom indicator model.
//!
//! Provides formatted zoom data for the status bar. The indicator is hidden
//! when the offset is zero and shows `Zoom: +N` or `Zoom: -N` otherwise.

use crate::types::ZoomOffset;

/// The state of the zoom indicator in the status bar.
///
/// The indicator is hidden when zoom is at the default (zero) offset to
/// reduce visual clutter. It becomes visible with formatted text when
/// any non-zero offset is active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoomIndicatorState {
    /// The indicator is not shown (offset is zero).
    Hidden,
    /// The indicator is visible with formatted text.
    Visible {
        /// The display text (e.g., "Zoom: +3" or "Zoom: -2").
        text: String,
        /// The raw offset value.
        offset: i32,
    },
}

impl ZoomIndicatorState {
    /// Compute the indicator state from a zoom offset.
    ///
    /// Returns [`Hidden`](Self::Hidden) when the offset is zero,
    /// [`Visible`](Self::Visible) with formatted text otherwise.
    pub fn from_offset(offset: ZoomOffset) -> Self {
        if offset.is_zero() {
            Self::Hidden
        } else {
            let value = offset.value();
            let text = if value > 0 {
                format!("Zoom: +{value}")
            } else {
                format!("Zoom: {value}")
            };
            Self::Visible {
                text,
                offset: value,
            }
        }
    }
}

/// A quick-pick option for the zoom popup in the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomQuickPickOption {
    /// The display label for the option.
    pub label: String,
    /// The zoom offset value this option represents.
    pub offset: i32,
}

impl ZoomQuickPickOption {
    /// Returns the default list of quick-pick offsets for the zoom popup.
    ///
    /// Includes common offsets plus a "Reset to 0" action.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                label: "-5".to_string(),
                offset: -5,
            },
            Self {
                label: "-2".to_string(),
                offset: -2,
            },
            Self {
                label: "0 (Reset)".to_string(),
                offset: 0,
            },
            Self {
                label: "+2".to_string(),
                offset: 2,
            },
            Self {
                label: "+5".to_string(),
                offset: 5,
            },
            Self {
                label: "+10".to_string(),
                offset: 10,
            },
        ]
    }
}

/// Format the status message for the ZOOM query command (no arguments).
///
/// Returns a message like: `"Zoom offset: +3 (effective size: 15pt)"`
pub fn format_zoom_query(offset: ZoomOffset, effective_size: u32) -> String {
    let sign = if offset.value() > 0 { "+" } else { "" };
    format!(
        "Zoom offset: {sign}{} (effective size: {effective_size}pt)",
        offset.value()
    )
}

/// Format the boundary-reached status message.
///
/// # Arguments
///
/// * `is_maximum` — true for max boundary, false for min boundary.
/// * `limit_value` — the boundary offset value.
pub fn format_boundary_message(is_maximum: bool, limit_value: i32) -> String {
    if is_maximum {
        format!("Maximum zoom reached (+{limit_value})")
    } else {
        format!("Minimum zoom reached ({limit_value})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.2 — indicator hidden at zero
    #[test]
    fn indicator_hidden_when_offset_is_zero() {
        let offset = ZoomOffset::zero();
        let state = ZoomIndicatorState::from_offset(offset);
        assert_eq!(state, ZoomIndicatorState::Hidden);
    }

    // Validates: Requirement 7.1 — indicator visible when non-zero
    #[test]
    fn indicator_visible_when_offset_positive() {
        let offset = ZoomOffset::new(3, -10, 60);
        let state = ZoomIndicatorState::from_offset(offset);
        assert_eq!(
            state,
            ZoomIndicatorState::Visible {
                text: "Zoom: +3".to_string(),
                offset: 3,
            }
        );
    }

    // Validates: Requirement 7.5 — negative offset format
    #[test]
    fn indicator_visible_when_offset_negative() {
        let offset = ZoomOffset::new(-2, -10, 60);
        let state = ZoomIndicatorState::from_offset(offset);
        assert_eq!(
            state,
            ZoomIndicatorState::Visible {
                text: "Zoom: -2".to_string(),
                offset: -2,
            }
        );
    }

    // Validates: Requirement 7.4 — quick-pick defaults
    #[test]
    fn quick_pick_defaults_contains_expected_values() {
        let options = ZoomQuickPickOption::defaults();
        let offsets: Vec<i32> = options.iter().map(|o| o.offset).collect();
        assert_eq!(offsets, vec![-5, -2, 0, 2, 5, 10]);
    }

    // Validates: Requirement 8.6 — zoom query format
    #[test]
    fn format_zoom_query_positive_offset() {
        let offset = ZoomOffset::new(3, -10, 60);
        let msg = format_zoom_query(offset, 15);
        assert_eq!(msg, "Zoom offset: +3 (effective size: 15pt)");
    }

    // Validates: Requirement 8.6 — zoom query format for negative
    #[test]
    fn format_zoom_query_negative_offset() {
        let offset = ZoomOffset::new(-2, -10, 60);
        let msg = format_zoom_query(offset, 10);
        assert_eq!(msg, "Zoom offset: -2 (effective size: 10pt)");
    }

    // Validates: Requirement 8.6 — zoom query format for zero
    #[test]
    fn format_zoom_query_zero_offset() {
        let offset = ZoomOffset::zero();
        let msg = format_zoom_query(offset, 12);
        assert_eq!(msg, "Zoom offset: 0 (effective size: 12pt)");
    }

    // Validates: Requirement 2.6 — max boundary message
    #[test]
    fn format_boundary_message_maximum() {
        let msg = format_boundary_message(true, 60);
        assert_eq!(msg, "Maximum zoom reached (+60)");
    }

    // Validates: Requirement 2.7 — min boundary message
    #[test]
    fn format_boundary_message_minimum() {
        let msg = format_boundary_message(false, -10);
        assert_eq!(msg, "Minimum zoom reached (-10)");
    }
}
