//! Wrap visual flags for continuation line markers.
//!
//! Controls visual indicators rendered at continuation line boundaries
//! to show where a logical line has been wrapped.

/// Wrap visual flags indicating where wrapping has occurred.
///
/// Addresses: Requirement 10 (Wrap Visual Flags)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum WrapVisualFlags {
    /// No visual markers at wrap break points.
    #[default]
    None,

    /// Indicator glyph at the right edge of sub-lines that continue.
    End,

    /// Indicator glyph at the left side of continuation lines.
    Start,

    /// Both Start and End indicators.
    StartEnd,

    /// Indicator in the line-number margin adjacent to continuation lines.
    Margin,
}

/// Location of a wrap marker within a sub-line layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMarkerLocation {
    /// Marker at the right edge (end) of a sub-line that continues.
    End,
    /// Marker at the left side (start) of a continuation line.
    Start,
    /// Marker in the line-number margin.
    Margin,
}

/// A computed marker position for a specific sub-line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapMarkerPosition {
    /// Zero-based index of the sub-line within the wrapped document line.
    pub sub_line_index: u32,
    /// Where the marker should be rendered.
    pub location: WrapMarkerLocation,
}

/// Compute wrap marker positions for a wrapped line given its height and flags.
///
/// Returns an empty vec when flags are `None` or height is 1 (no continuation lines).
///
/// Addresses: Requirement 10 AC 2–5
pub fn compute_markers(line_height: u32, flags: WrapVisualFlags) -> Vec<WrapMarkerPosition> {
    if line_height <= 1 {
        return Vec::new();
    }

    match flags {
        WrapVisualFlags::None => Vec::new(),
        WrapVisualFlags::End => {
            // Marker at the end of each sub-line that continues (all except the last)
            (0..line_height - 1)
                .map(|i| WrapMarkerPosition {
                    sub_line_index: i,
                    location: WrapMarkerLocation::End,
                })
                .collect()
        }
        WrapVisualFlags::Start => {
            // Marker at the start of each continuation line (all except the first)
            (1..line_height)
                .map(|i| WrapMarkerPosition {
                    sub_line_index: i,
                    location: WrapMarkerLocation::Start,
                })
                .collect()
        }
        WrapVisualFlags::StartEnd => {
            let mut markers = Vec::new();
            // End markers on lines that continue
            for i in 0..line_height - 1 {
                markers.push(WrapMarkerPosition {
                    sub_line_index: i,
                    location: WrapMarkerLocation::End,
                });
            }
            // Start markers on continuation lines
            for i in 1..line_height {
                markers.push(WrapMarkerPosition {
                    sub_line_index: i,
                    location: WrapMarkerLocation::Start,
                });
            }
            markers
        }
        WrapVisualFlags::Margin => {
            // Margin marker on each continuation line (all except the first)
            (1..line_height)
                .map(|i| WrapMarkerPosition {
                    sub_line_index: i,
                    location: WrapMarkerLocation::Margin,
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_markers_when_flags_none() {
        // Validates: Requirement 10.5
        let markers = compute_markers(3, WrapVisualFlags::None);
        assert!(markers.is_empty());
    }

    #[test]
    fn no_markers_when_height_is_one() {
        // No continuation lines exist when height is 1
        let markers = compute_markers(1, WrapVisualFlags::End);
        assert!(markers.is_empty());
    }

    #[test]
    fn end_markers_on_all_except_last() {
        // Validates: Requirement 10.2
        let markers = compute_markers(3, WrapVisualFlags::End);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].sub_line_index, 0);
        assert_eq!(markers[0].location, WrapMarkerLocation::End);
        assert_eq!(markers[1].sub_line_index, 1);
        assert_eq!(markers[1].location, WrapMarkerLocation::End);
    }

    #[test]
    fn start_markers_on_all_except_first() {
        // Validates: Requirement 10.3
        let markers = compute_markers(3, WrapVisualFlags::Start);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].sub_line_index, 1);
        assert_eq!(markers[0].location, WrapMarkerLocation::Start);
        assert_eq!(markers[1].sub_line_index, 2);
        assert_eq!(markers[1].location, WrapMarkerLocation::Start);
    }

    #[test]
    fn start_end_markers_combined() {
        // Validates: Requirement 10 AC combined
        let markers = compute_markers(3, WrapVisualFlags::StartEnd);
        // 2 end markers + 2 start markers = 4 total
        assert_eq!(markers.len(), 4);
    }

    #[test]
    fn margin_markers_on_continuation_lines() {
        // Validates: Requirement 10.4
        let markers = compute_markers(3, WrapVisualFlags::Margin);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].sub_line_index, 1);
        assert_eq!(markers[0].location, WrapMarkerLocation::Margin);
        assert_eq!(markers[1].sub_line_index, 2);
        assert_eq!(markers[1].location, WrapMarkerLocation::Margin);
    }
}
