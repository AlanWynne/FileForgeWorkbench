//! Preview rendering — GUI-independent display model.
//!
//! Transforms raw document lines and ASA controls into a sequence of
//! `PreviewElement` items that the UI layer interprets for rendering.

use crate::control::AsaControl;
use crate::merge::{merge_overstrikes, MergeResult, MergedLine};
use crate::page_index::PageIndex;

/// A single element in the rendered preview output.
///
/// The paginator produces a sequence of these for the UI to render.
// Validates: Requirements 1, 4, 5, 9
#[derive(Debug, Clone, PartialEq)]
pub enum PreviewElement {
    /// A rendered data line (possibly merged from overstrikes).
    DataLine {
        /// The styled content for rendering.
        content: MergedLine,
        /// Which line band shading group this line belongs to (alternates 0/1).
        band_group: u8,
        /// Page-relative line number (1-based within current page).
        page_line: usize,
    },
    /// A blank spacing line (display artifact, not a real document line).
    SpacingLine {
        /// Which line band shading group this line belongs to.
        band_group: u8,
    },
    /// A page break band.
    PageBand {
        /// 1-based page number.
        page_number: usize,
        /// Whether this is an explicit (from `1`) or implicit (page depth) break.
        is_explicit: bool,
    },
    /// A printer halt warning band.
    HaltBand {
        /// 0-based document line number of the halt control.
        source_line: usize,
    },
}

/// Complete preview state for a document.
///
/// Built by the paginator from the parsed document; consumed by the UI layer.
// Validates: Requirement 3.6
#[derive(Debug, Clone)]
pub struct PreviewState {
    /// Ordered sequence of preview elements for rendering.
    pub elements: Vec<PreviewElement>,
    /// Page index for navigation.
    pub page_index: PageIndex,
    /// Total page count.
    pub total_pages: usize,
}

/// Build a complete preview from document lines and their ASA controls.
///
/// This is the primary entry point for preview generation. It:
/// 1. Merges overstrike lines with their base lines
/// 2. Inserts spacing lines for `0` and `-` controls
/// 3. Inserts page bands at `1` controls
/// 4. Inserts halt bands for `H` controls
/// 5. Assigns line band shading groups
// Validates: Requirements 1.2–1.9, 4.1–4.7, 5.4
pub fn build_preview(
    controls: &[AsaControl],
    contents: &[&str],
    page_depth: u16,
    band_size: u8,
    implicit_breaks: bool,
) -> PreviewState {
    let page_index = PageIndex::build(controls, page_depth, implicit_breaks);
    let merge_results = merge_overstrikes(controls, contents);

    let mut elements: Vec<PreviewElement> = Vec::new();
    let mut page_number: usize = 0;
    let mut page_line: usize = 0;
    let mut band_counter: usize = 0;

    // Track which source lines have been processed
    let mut merge_idx = 0;

    for (line_idx, &control) in controls.iter().enumerate() {
        if control.is_overstrike() {
            // Overstrikes are consumed by the merge engine — not emitted
            continue;
        }

        match control {
            AsaControl::PageEject => {
                page_number += 1;
                page_line = 0;
                band_counter = 0;

                elements.push(PreviewElement::PageBand {
                    page_number,
                    is_explicit: true,
                });

                // Emit the data line after the page band
                if let Some(merged) =
                    get_next_merge_result(&merge_results, &mut merge_idx, line_idx)
                {
                    page_line += 1;
                    let band_group = (band_counter / band_size as usize) as u8 % 2;
                    band_counter += 1;
                    elements.push(PreviewElement::DataLine {
                        content: merged,
                        band_group,
                        page_line,
                    });
                }
            }
            AsaControl::Halt => {
                elements.push(PreviewElement::HaltBand {
                    source_line: line_idx,
                });
                // Emit the data content after halt band
                if let Some(merged) =
                    get_next_merge_result(&merge_results, &mut merge_idx, line_idx)
                {
                    page_line += 1;
                    let band_group = (band_counter / band_size as usize) as u8 % 2;
                    band_counter += 1;
                    elements.push(PreviewElement::DataLine {
                        content: merged,
                        band_group,
                        page_line,
                    });
                }
            }
            AsaControl::DoubleSpace => {
                // Insert 1 spacing line
                let band_group = (band_counter / band_size as usize) as u8 % 2;
                band_counter += 1;
                elements.push(PreviewElement::SpacingLine { band_group });

                // Then the data line
                if let Some(merged) =
                    get_next_merge_result(&merge_results, &mut merge_idx, line_idx)
                {
                    page_line += 1;
                    let band_group = (band_counter / band_size as usize) as u8 % 2;
                    band_counter += 1;
                    elements.push(PreviewElement::DataLine {
                        content: merged,
                        band_group,
                        page_line,
                    });
                }
            }
            AsaControl::TripleSpace => {
                // Insert 2 spacing lines
                for _ in 0..2 {
                    let band_group = (band_counter / band_size as usize) as u8 % 2;
                    band_counter += 1;
                    elements.push(PreviewElement::SpacingLine { band_group });
                }

                // Then the data line
                if let Some(merged) =
                    get_next_merge_result(&merge_results, &mut merge_idx, line_idx)
                {
                    page_line += 1;
                    let band_group = (band_counter / band_size as usize) as u8 % 2;
                    band_counter += 1;
                    elements.push(PreviewElement::DataLine {
                        content: merged,
                        band_group,
                        page_line,
                    });
                }
            }
            _ => {
                // Space, Unknown — just emit the data line
                if let Some(merged) =
                    get_next_merge_result(&merge_results, &mut merge_idx, line_idx)
                {
                    page_line += 1;
                    let band_group = (band_counter / band_size as usize) as u8 % 2;
                    band_counter += 1;
                    elements.push(PreviewElement::DataLine {
                        content: merged,
                        band_group,
                        page_line,
                    });
                }
            }
        }
    }

    let total_pages = page_index.page_count();

    PreviewState {
        elements,
        page_index,
        total_pages,
    }
}

/// Helper to find the merge result corresponding to a given source line.
fn get_next_merge_result(
    merge_results: &[MergeResult],
    merge_idx: &mut usize,
    source_line: usize,
) -> Option<MergedLine> {
    while *merge_idx < merge_results.len() {
        match &merge_results[*merge_idx] {
            MergeResult::Merged(merged) => {
                if merged.source_line == source_line {
                    *merge_idx += 1;
                    return Some(merged.clone());
                } else if merged.source_line > source_line {
                    return None;
                }
                *merge_idx += 1;
            }
            MergeResult::OrphanOverprint {
                source_line: sl,
                content,
            } => {
                if *sl == source_line {
                    // Treat orphan as a regular line
                    let merged = MergedLine::from_base(content, *sl);
                    *merge_idx += 1;
                    return Some(merged);
                }
                *merge_idx += 1;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 1.2
    fn single_space_lines_emit_data_lines() {
        let controls = vec![AsaControl::Space, AsaControl::Space];
        let contents = vec!["LINE 1", "LINE 2"];
        let state = build_preview(&controls, &contents, 60, 5, false);
        assert_eq!(state.elements.len(), 2);
        assert!(matches!(
            &state.elements[0],
            PreviewElement::DataLine { .. }
        ));
        assert!(matches!(
            &state.elements[1],
            PreviewElement::DataLine { .. }
        ));
    }

    #[test]
    // Validates: Requirement 1.3
    fn double_space_inserts_one_spacing_line() {
        let controls = vec![AsaControl::Space, AsaControl::DoubleSpace];
        let contents = vec!["LINE 1", "LINE 2"];
        let state = build_preview(&controls, &contents, 60, 5, false);
        // Expected: DataLine, SpacingLine, DataLine
        assert_eq!(state.elements.len(), 3);
        assert!(matches!(
            &state.elements[0],
            PreviewElement::DataLine { .. }
        ));
        assert!(matches!(
            &state.elements[1],
            PreviewElement::SpacingLine { .. }
        ));
        assert!(matches!(
            &state.elements[2],
            PreviewElement::DataLine { .. }
        ));
    }

    #[test]
    // Validates: Requirement 1.4
    fn triple_space_inserts_two_spacing_lines() {
        let controls = vec![AsaControl::Space, AsaControl::TripleSpace];
        let contents = vec!["LINE 1", "LINE 2"];
        let state = build_preview(&controls, &contents, 60, 5, false);
        // Expected: DataLine, SpacingLine, SpacingLine, DataLine
        assert_eq!(state.elements.len(), 4);
        assert!(matches!(
            &state.elements[1],
            PreviewElement::SpacingLine { .. }
        ));
        assert!(matches!(
            &state.elements[2],
            PreviewElement::SpacingLine { .. }
        ));
    }

    #[test]
    // Validates: Requirement 1.5, 4.1
    fn page_eject_inserts_page_band() {
        let controls = vec![AsaControl::PageEject, AsaControl::Space];
        let contents = vec!["PAGE 1 LINE 1", "LINE 2"];
        let state = build_preview(&controls, &contents, 60, 5, false);
        // Expected: PageBand, DataLine, DataLine
        assert_eq!(state.elements.len(), 3);
        assert!(matches!(
            &state.elements[0],
            PreviewElement::PageBand { page_number: 1, .. }
        ));
    }

    #[test]
    // Validates: Requirement 1.6
    fn overstrike_lines_not_emitted_as_separate_rows() {
        let controls = vec![AsaControl::Space, AsaControl::Overstrike];
        let contents = vec!["BASE", "BASE"];
        let state = build_preview(&controls, &contents, 60, 5, false);
        // Only the merged base line should appear
        assert_eq!(state.elements.len(), 1);
        if let PreviewElement::DataLine { ref content, .. } = state.elements[0] {
            assert!(content.has_bold());
        } else {
            panic!("Expected DataLine");
        }
    }

    #[test]
    // Validates: Requirement 1.7
    fn halt_control_inserts_halt_band() {
        let controls = vec![AsaControl::Halt];
        let contents = vec!["HALT LINE"];
        let state = build_preview(&controls, &contents, 60, 5, false);
        assert!(matches!(
            &state.elements[0],
            PreviewElement::HaltBand { source_line: 0 }
        ));
        assert!(matches!(
            &state.elements[1],
            PreviewElement::DataLine { .. }
        ));
    }

    #[test]
    // Validates: Requirement 4.2 — page numbering
    fn page_bands_numbered_sequentially() {
        let controls = vec![
            AsaControl::PageEject,
            AsaControl::Space,
            AsaControl::PageEject,
            AsaControl::Space,
        ];
        let contents = vec!["P1L1", "P1L2", "P2L1", "P2L2"];
        let state = build_preview(&controls, &contents, 60, 5, false);
        let page_bands: Vec<_> = state
            .elements
            .iter()
            .filter_map(|e| {
                if let PreviewElement::PageBand { page_number, .. } = e {
                    Some(*page_number)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(page_bands, vec![1, 2]);
    }
}
