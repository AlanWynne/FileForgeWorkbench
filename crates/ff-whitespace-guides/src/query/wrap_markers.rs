//! Wrap marker and continuation indent computation.

use crate::modes::{WrapIndentMode, WrapVisualFlag, WrapVisualLocation};
use crate::types::{WrapIndentInfo, WrapMarkerInfo};

/// Compute wrap markers for a wrapped document line.
///
/// Returns marker information for a document line given the number of sub-lines
/// and the active wrap visual configuration.
///
/// Returns `None` when flags are `NONE` or sub_line_count is 1 (no wrapping).
///
/// Addresses: Requirement 6 AC 6.3–6.5
pub fn compute_wrap_markers(
    sub_line_count: u32,
    flags: WrapVisualFlag,
    location: WrapVisualLocation,
) -> Option<WrapMarkerInfo> {
    if flags.bits() == 0 || sub_line_count <= 1 {
        return None;
    }

    let mut end_markers = Vec::new();
    let mut start_markers = Vec::new();

    if flags.has_end() {
        // End markers on all sub-lines except the last
        for i in 0..(sub_line_count - 1) {
            end_markers.push(i);
        }
    }

    if flags.has_start() {
        // Start markers on all continuation sub-lines (index > 0)
        for i in 1..sub_line_count {
            start_markers.push(i);
        }
    }

    let margin_marker = flags.has_margin();

    Some(WrapMarkerInfo {
        end_markers,
        start_markers,
        margin_marker,
        location,
    })
}

/// Compute the effective indentation for continuation sub-lines.
///
/// Clamps the result to at most 3/4 of the viewport width.
///
/// # Arguments
///
/// * `first_subline_indent` - The leading whitespace width of the first sub-line (in chars).
/// * `tab_size` - The tab size for computing additional indent levels.
/// * `mode` - The wrap indentation mode.
/// * `start_indent` - The fixed offset for `Fixed` mode.
/// * `viewport_width` - The viewport width in character units.
///
/// Addresses: Requirement 7 AC 7.1–7.6
pub fn compute_continuation_indent(
    first_subline_indent: u32,
    tab_size: u32,
    mode: WrapIndentMode,
    start_indent: u32,
    viewport_width: u32,
) -> WrapIndentInfo {
    let tab_size = tab_size.max(1);
    let max_indent = viewport_width * 3 / 4;

    let raw_indent = match mode {
        WrapIndentMode::Fixed => start_indent,
        WrapIndentMode::Same => first_subline_indent,
        WrapIndentMode::Indent => first_subline_indent + tab_size,
        WrapIndentMode::DeepIndent => first_subline_indent + tab_size * 2,
    };

    let clamped = raw_indent > max_indent;
    let indent_chars = raw_indent.min(max_indent);

    WrapIndentInfo {
        mode,
        indent_chars,
        clamped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_flags_returns_none() {
        // Validates: Requirement 6.2
        let result = compute_wrap_markers(3, WrapVisualFlag::NONE, WrapVisualLocation::Default);
        assert_eq!(result, None);
    }

    #[test]
    fn single_subline_returns_none() {
        // Validates: Requirement 6.3
        let result = compute_wrap_markers(1, WrapVisualFlag::END, WrapVisualLocation::Default);
        assert_eq!(result, None);
    }

    #[test]
    fn end_flag_marks_non_last_sublines() {
        // Validates: Requirement 6.3
        let result =
            compute_wrap_markers(3, WrapVisualFlag::END, WrapVisualLocation::Default).unwrap();
        assert_eq!(result.end_markers, vec![0, 1]);
        assert!(result.start_markers.is_empty());
        assert!(!result.margin_marker);
    }

    #[test]
    fn start_flag_marks_continuation_sublines() {
        // Validates: Requirement 6.4
        let result =
            compute_wrap_markers(3, WrapVisualFlag::START, WrapVisualLocation::Default).unwrap();
        assert!(result.end_markers.is_empty());
        assert_eq!(result.start_markers, vec![1, 2]);
    }

    #[test]
    fn margin_flag_sets_margin_marker() {
        // Validates: Requirement 6.5
        let result =
            compute_wrap_markers(2, WrapVisualFlag::MARGIN, WrapVisualLocation::Default).unwrap();
        assert!(result.margin_marker);
    }

    #[test]
    fn combined_flags_produce_both_markers() {
        // Validates: Requirement 6.3, 6.4
        let flags = WrapVisualFlag::END.union(WrapVisualFlag::START);
        let result = compute_wrap_markers(3, flags, WrapVisualLocation::EndByText).unwrap();
        assert_eq!(result.end_markers, vec![0, 1]);
        assert_eq!(result.start_markers, vec![1, 2]);
        assert_eq!(result.location, WrapVisualLocation::EndByText);
    }

    #[test]
    fn fixed_mode_uses_start_indent() {
        // Validates: Requirement 7.2
        let result = compute_continuation_indent(8, 4, WrapIndentMode::Fixed, 4, 80);
        assert_eq!(result.indent_chars, 4);
        assert!(!result.clamped);
    }

    #[test]
    fn same_mode_uses_first_subline_indent() {
        // Validates: Requirement 7.5
        let result = compute_continuation_indent(8, 4, WrapIndentMode::Same, 0, 80);
        assert_eq!(result.indent_chars, 8);
        assert!(!result.clamped);
    }

    #[test]
    fn indent_mode_adds_one_tab_stop() {
        // Validates: Requirement 7.5
        let result = compute_continuation_indent(8, 4, WrapIndentMode::Indent, 0, 80);
        assert_eq!(result.indent_chars, 12);
        assert!(!result.clamped);
    }

    #[test]
    fn deep_indent_mode_adds_two_tab_stops() {
        // Validates: Requirement 7.5
        let result = compute_continuation_indent(8, 4, WrapIndentMode::DeepIndent, 0, 80);
        assert_eq!(result.indent_chars, 16);
        assert!(!result.clamped);
    }

    #[test]
    fn continuation_indent_clamped_at_three_quarters_viewport() {
        // Validates: Requirement 7.6
        let result = compute_continuation_indent(100, 4, WrapIndentMode::Same, 0, 80);
        assert_eq!(result.indent_chars, 60); // 80 * 3/4 = 60
        assert!(result.clamped);
    }

    #[test]
    fn wrap_inactive_guard_no_markers_when_sublines_one() {
        // Validates: Requirement 6.9
        let flags = WrapVisualFlag::END
            .union(WrapVisualFlag::START)
            .union(WrapVisualFlag::MARGIN);
        let result = compute_wrap_markers(1, flags, WrapVisualLocation::Default);
        assert_eq!(result, None);
    }
}
