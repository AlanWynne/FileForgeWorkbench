//! Line band shading computation.
//!
//! Computes alternating background shading groups for green-bar/blue-bar
//! paper simulation. Groups of N consecutive lines are assigned alternating
//! tint bands that reset at each page boundary.

use crate::preview::PreviewElement;

/// Compute band group assignments for a sequence of preview elements.
///
/// Rules:
/// - Data lines and spacing lines are grouped into consecutive blocks of `band_size`
/// - Groups alternate (0, 1, 0, 1, ...) for tinting
/// - Page bands reset the band counter to 0
/// - Page bands and halt bands do not carry a band group
///
/// Returns a vector of `Option<u8>` parallel to `elements`:
/// - `Some(0)` or `Some(1)` for data/spacing lines
/// - `None` for page bands and halt bands
// Validates: Requirement 9.1–9.5
pub fn compute_band_groups(elements: &[PreviewElement], band_size: u8) -> Vec<Option<u8>> {
    let band_size = band_size.max(1) as usize;
    let mut result = Vec::with_capacity(elements.len());
    let mut counter: usize = 0;

    for element in elements {
        match element {
            PreviewElement::PageBand { .. } => {
                // Page bands reset counter and don't carry a group
                counter = 0;
                result.push(None);
            }
            PreviewElement::HaltBand { .. } => {
                // Halt bands don't carry a group and don't reset counter
                result.push(None);
            }
            PreviewElement::DataLine { .. } | PreviewElement::SpacingLine { .. } => {
                let group = (counter / band_size) as u8 % 2;
                result.push(Some(group));
                counter += 1;
            }
        }
    }

    result
}

/// Whether a given band group number indicates the tinted band.
///
/// Group 0 = untinted (default background), Group 1 = tinted.
pub fn is_tinted_group(group: u8) -> bool {
    group % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merge::MergedLine;

    fn data_line(source_line: usize) -> PreviewElement {
        PreviewElement::DataLine {
            content: MergedLine::from_base("DATA", source_line),
            band_group: 0,
            page_line: 1,
        }
    }

    fn spacing_line() -> PreviewElement {
        PreviewElement::SpacingLine { band_group: 0 }
    }

    fn page_band(n: usize) -> PreviewElement {
        PreviewElement::PageBand {
            page_number: n,
            is_explicit: true,
        }
    }

    #[test]
    // Validates: Requirement 9.1
    fn alternating_bands_with_band_size_2() {
        let elements = vec![
            data_line(0),
            data_line(1),
            data_line(2),
            data_line(3),
            data_line(4),
            data_line(5),
        ];
        let groups = compute_band_groups(&elements, 2);
        // Groups: 0,0, 1,1, 0,0
        assert_eq!(
            groups,
            vec![Some(0), Some(0), Some(1), Some(1), Some(0), Some(0)]
        );
    }

    #[test]
    // Validates: Requirement 9.3
    fn page_band_resets_counter() {
        let elements = vec![
            data_line(0),
            data_line(1), // group 0,0
            page_band(1), // reset
            data_line(2),
            data_line(3), // group 0,0 (reset)
        ];
        let groups = compute_band_groups(&elements, 2);
        assert_eq!(groups[0], Some(0));
        assert_eq!(groups[1], Some(0));
        assert_eq!(groups[2], None); // page band
        assert_eq!(groups[3], Some(0)); // reset to 0
        assert_eq!(groups[4], Some(0));
    }

    #[test]
    // Validates: Requirement 9.4
    fn spacing_lines_participate_in_band_count() {
        let elements = vec![data_line(0), spacing_line(), data_line(1)];
        let groups = compute_band_groups(&elements, 2);
        // All three count toward the band: 0, 0, 1
        assert_eq!(groups, vec![Some(0), Some(0), Some(1)]);
    }

    #[test]
    // Validates: Requirement 9.5
    fn page_bands_do_not_carry_group() {
        let elements = vec![page_band(1), data_line(0)];
        let groups = compute_band_groups(&elements, 5);
        assert_eq!(groups[0], None);
        assert_eq!(groups[1], Some(0));
    }

    #[test]
    fn halt_band_does_not_carry_group() {
        let elements = vec![
            data_line(0),
            PreviewElement::HaltBand { source_line: 1 },
            data_line(2),
        ];
        let groups = compute_band_groups(&elements, 5);
        assert_eq!(groups[0], Some(0));
        assert_eq!(groups[1], None);
        assert_eq!(groups[2], Some(0)); // counter continues (halt doesn't reset)
    }

    #[test]
    fn is_tinted_group_alternates() {
        assert!(!is_tinted_group(0));
        assert!(is_tinted_group(1));
        assert!(!is_tinted_group(2));
        assert!(is_tinted_group(3));
    }

    #[test]
    fn empty_elements_produces_empty_groups() {
        let groups = compute_band_groups(&[], 5);
        assert!(groups.is_empty());
    }

    #[test]
    fn band_size_one_alternates_every_line() {
        let elements = vec![data_line(0), data_line(1), data_line(2)];
        let groups = compute_band_groups(&elements, 1);
        assert_eq!(groups, vec![Some(0), Some(1), Some(0)]);
    }
}
