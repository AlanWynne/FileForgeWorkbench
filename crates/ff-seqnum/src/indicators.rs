//! Visual indicators and status bar integration.
//!
//! Provides the data model for sequence number status bar indicators
//! and column highlighting support.

use crate::config::SeqNumConfig;
use crate::state::{SeqNumState, SeqNumStatusIndicator};
use crate::types::ColumnRange;

/// The formatted indicator for display in the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqNumIndicator {
    /// No indicator to display.
    None,
    /// Sequence numbers were stripped. Display "SEQNUM x-y[,x-y]".
    Stripped { ranges: String },
    /// Sequence numbers detected but not stripped. Display "SEQNUM?".
    Detected,
    /// NUMBER SHOW overlay is active. Display "SEQSHOW".
    NumberShow,
}

/// Get the status bar indicator based on current state.
pub fn get_indicator(state: &SeqNumState) -> SeqNumIndicator {
    match state.status_indicator() {
        SeqNumStatusIndicator::None => SeqNumIndicator::None,
        SeqNumStatusIndicator::Stripped {
            has_front,
            has_back,
        } => {
            let ranges = format_stripped_ranges(
                if has_front {
                    state.stripped_front
                } else {
                    None
                },
                if has_back { state.stripped_back } else { None },
            );
            SeqNumIndicator::Stripped { ranges }
        }
        SeqNumStatusIndicator::DetectedNotStripped => SeqNumIndicator::Detected,
        SeqNumStatusIndicator::ShowMode => SeqNumIndicator::NumberShow,
    }
}

/// Format the indicator text for display.
pub fn format_indicator_text(indicator: &SeqNumIndicator) -> Option<String> {
    match indicator {
        SeqNumIndicator::None => None,
        SeqNumIndicator::Stripped { ranges } => Some(format!("SEQNUM {ranges}")),
        SeqNumIndicator::Detected => Some("SEQNUM?".to_string()),
        SeqNumIndicator::NumberShow => Some("SEQSHOW".to_string()),
    }
}

/// Whether column highlighting should be active.
pub fn should_highlight_columns(config: &SeqNumConfig) -> bool {
    config.highlight_columns
}

/// Format the stripped ranges string for display.
fn format_stripped_ranges(front: Option<ColumnRange>, back: Option<ColumnRange>) -> String {
    let mut parts = Vec::new();
    if let Some(f) = front {
        parts.push(format!("{f}"));
    }
    if let Some(b) = back {
        parts.push(format!("{b}"));
    }
    parts.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::FullDetectionResult;
    use crate::types::DetectionResult;

    #[test]
    fn indicator_none_for_empty_state() {
        // Validates: Requirement 4.1
        let state = SeqNumState::new();
        let indicator = get_indicator(&state);
        assert_eq!(indicator, SeqNumIndicator::None);
        assert_eq!(format_indicator_text(&indicator), None);
    }

    #[test]
    fn indicator_stripped_shows_ranges() {
        // Validates: Requirement 4.1
        let mut state = SeqNumState::new();
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state.stripped_back = Some(ColumnRange::new(73, 80).unwrap());

        let indicator = get_indicator(&state);
        match &indicator {
            SeqNumIndicator::Stripped { ranges } => {
                assert!(ranges.contains("1-6"));
                assert!(ranges.contains("73-80"));
            }
            _ => panic!("Expected Stripped"),
        }
        let text = format_indicator_text(&indicator).unwrap();
        assert_eq!(text, "SEQNUM 1-6,73-80");
    }

    #[test]
    fn indicator_detected_not_stripped() {
        // Validates: Requirement 4.2
        let mut state = SeqNumState::new();
        state.detection = Some(FullDetectionResult {
            front: DetectionResult::Present,
            back: DetectionResult::Absent,
            front_columns: Some(ColumnRange::new(1, 6).unwrap()),
            back_columns: None,
            front_format: None,
            back_format: None,
            lines_sampled: 10,
        });

        let indicator = get_indicator(&state);
        assert_eq!(indicator, SeqNumIndicator::Detected);
        let text = format_indicator_text(&indicator).unwrap();
        assert_eq!(text, "SEQNUM?");
    }

    #[test]
    fn indicator_number_show() {
        // Validates: Requirement 4.4
        let mut state = SeqNumState::new();
        state.number_show_active = true;

        let indicator = get_indicator(&state);
        assert_eq!(indicator, SeqNumIndicator::NumberShow);
        let text = format_indicator_text(&indicator).unwrap();
        assert_eq!(text, "SEQSHOW");
    }

    #[test]
    fn highlight_columns_respects_config() {
        // Validates: Requirement 4.5
        let mut config = SeqNumConfig::default();
        assert!(!should_highlight_columns(&config));

        config.highlight_columns = true;
        assert!(should_highlight_columns(&config));
    }
}
