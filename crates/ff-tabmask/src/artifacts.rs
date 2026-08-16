//! Display artifact lifecycle and rendering.
//!
//! Manages the lifecycle of TABS_Line and MASK_Line display artifacts,
//! including rendering, toggle logic, and metadata generation.

use crate::mask::MaskLine;
use crate::state::ArtifactPosition;
use crate::tab_stops::TabStopList;

/// The kind of display artifact.
///
/// Addresses: Requirement 18, criteria 18.1, 18.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    /// A tab stop ruler line.
    TabsLine,
    /// A mask template line.
    MaskLine,
}

/// Metadata for command framework registration of display artifact commands.
///
/// Addresses: Requirement 18, criterion 18.7
#[derive(Debug, Clone)]
pub struct ArtifactMetadata {
    /// The command identifier (e.g., "edit.tabs").
    pub command_id: &'static str,
    /// The display name (e.g., "TABS").
    pub display_name: &'static str,
    /// Description for help/discoverability.
    pub description: &'static str,
    /// The command category (always "display" for artifacts).
    pub category: &'static str,
    /// Undo classification (always NonUndoable for display artifacts).
    pub undo_classification: UndoClassification,
    /// Modes in which the command is valid.
    pub applicable_modes: Vec<EditorMode>,
    /// Whether this artifact is a real document line (always false).
    pub is_real_document_line: bool,
}

/// Undo classification for commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoClassification {
    /// The command does not create undo transactions.
    NonUndoable,
}

/// Editor modes in which a command is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Insert mode: editable, Tab inserts spaces.
    Insert,
    /// Overstrike mode: editable, Tab moves cursor.
    Overstrike,
    /// Edit mode (general — encompasses Insert and Overstrike).
    Edit,
    /// Browse mode: display-only, not editable.
    Browse,
    /// View mode: read-only.
    View,
}

/// Manages the lifecycle of TABS_Line and MASK_Line display artifacts.
///
/// Addresses: Requirements 1, 3, 6, 8, 11, 17, 18
pub struct DisplayArtifactManager;

impl DisplayArtifactManager {
    /// Renders a TABS_Line string for the given tab stops and line width.
    ///
    /// Places `indicator_char` at each stop position (1-based column → 0-based index),
    /// `filler_char` at all other positions, up to `line_width`.
    ///
    /// Addresses: Requirement 1, criteria 1.2, 1.3; Requirement 17, criteria 17.1–17.5
    pub fn render_tabs_line(
        tab_stops: &TabStopList,
        line_width: usize,
        indicator_char: char,
        filler_char: char,
    ) -> String {
        let mut line = vec![filler_char; line_width];
        for &stop in tab_stops.stops() {
            let idx = (stop as usize).saturating_sub(1);
            if idx < line_width {
                line[idx] = indicator_char;
            }
        }
        // Also render extended stops up to line_width
        if !tab_stops.is_empty() {
            let mut col = *tab_stops.stops().last().unwrap();
            loop {
                match tab_stops.next_stop_after(col) {
                    Some(next) if (next as usize) <= line_width => {
                        let idx = (next as usize).saturating_sub(1);
                        if idx < line_width {
                            line[idx] = indicator_char;
                        }
                        col = next;
                    }
                    _ => break,
                }
            }
        }
        line.into_iter().collect()
    }

    /// Renders a MASK_Line string for display.
    ///
    /// Returns the mask content padded to line_width.
    ///
    /// Addresses: Requirement 6, criterion 6.3; Requirement 16, criteria 16.1, 16.4
    pub fn render_mask_line(mask: &MaskLine, line_width: usize) -> String {
        mask.apply_to_width(line_width)
    }

    /// Determines if a toggle should add or remove lines.
    ///
    /// Returns true if lines should be removed (already displayed).
    ///
    /// Addresses: Requirement 1, criterion 1.4; Requirement 6, criterion 6.5
    pub fn should_toggle_off(existing_lines: &[ArtifactPosition]) -> bool {
        !existing_lines.is_empty()
    }

    /// Creates artifact metadata for a display artifact line.
    ///
    /// Addresses: Requirement 18, criteria 18.1, 18.2, 18.7
    pub fn artifact_metadata(kind: ArtifactKind) -> ArtifactMetadata {
        match kind {
            ArtifactKind::TabsLine => ArtifactMetadata {
                command_id: "edit.tabs",
                display_name: "TABS",
                description: "Display/configure tab stop positions",
                category: "display",
                undo_classification: UndoClassification::NonUndoable,
                applicable_modes: vec![EditorMode::Edit, EditorMode::Browse, EditorMode::View],
                is_real_document_line: false,
            },
            ArtifactKind::MaskLine => ArtifactMetadata {
                command_id: "edit.mask",
                display_name: "MASK",
                description: "Display/edit insert mask template",
                category: "display",
                undo_classification: UndoClassification::NonUndoable,
                applicable_modes: vec![EditorMode::Edit, EditorMode::Browse],
                is_real_document_line: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_tabs_line_places_indicators_at_stops() {
        // Validates: Requirement 1.2, 17.1
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let line = DisplayArtifactManager::render_tabs_line(&stops, 20, 'T', '-');
        // Column 5 → index 4, column 10 → index 9, column 15 → index 14
        assert_eq!(line.chars().nth(4), Some('T'));
        assert_eq!(line.chars().nth(9), Some('T'));
        assert_eq!(line.chars().nth(14), Some('T'));
        // Index 19 (column 20) has extended stop at col 20 → index 19
        assert_eq!(line.chars().nth(19), Some('T'));
        // Other positions are filler
        assert_eq!(line.chars().next(), Some('-'));
        assert_eq!(line.chars().nth(5), Some('-'));
    }

    #[test]
    fn render_tabs_line_respects_line_width() {
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let line = DisplayArtifactManager::render_tabs_line(&stops, 12, 'T', '-');
        assert_eq!(line.len(), 12);
        // Column 15 (index 14) is beyond line_width 12 so should not appear
        assert_eq!(line.chars().nth(4), Some('T'));
        assert_eq!(line.chars().nth(9), Some('T'));
    }

    #[test]
    fn render_tabs_line_empty_stops_all_filler() {
        let stops = TabStopList::empty();
        let line = DisplayArtifactManager::render_tabs_line(&stops, 10, 'T', '-');
        assert_eq!(line, "----------");
    }

    #[test]
    fn render_mask_line_pads_to_width() {
        // Validates: Requirement 6.3
        let mask = MaskLine::new("ABC");
        let line = DisplayArtifactManager::render_mask_line(&mask, 8);
        assert_eq!(line, "ABC     ");
        assert_eq!(line.len(), 8);
    }

    #[test]
    fn should_toggle_off_with_existing_lines_returns_true() {
        // Validates: Requirement 1.4
        let lines = vec![ArtifactPosition {
            anchor_line: 5,
            from_line_command: false,
        }];
        assert!(DisplayArtifactManager::should_toggle_off(&lines));
    }

    #[test]
    fn should_toggle_off_with_no_lines_returns_false() {
        let lines: Vec<ArtifactPosition> = vec![];
        assert!(!DisplayArtifactManager::should_toggle_off(&lines));
    }

    #[test]
    fn artifact_metadata_tabs_line_correct() {
        // Validates: Requirement 18.7
        let meta = DisplayArtifactManager::artifact_metadata(ArtifactKind::TabsLine);
        assert_eq!(meta.command_id, "edit.tabs");
        assert_eq!(meta.category, "display");
        assert_eq!(meta.undo_classification, UndoClassification::NonUndoable);
        assert!(!meta.is_real_document_line);
        assert!(meta.applicable_modes.contains(&EditorMode::Edit));
        assert!(meta.applicable_modes.contains(&EditorMode::Browse));
        assert!(meta.applicable_modes.contains(&EditorMode::View));
    }

    #[test]
    fn artifact_metadata_mask_line_correct() {
        // Validates: Requirement 18.7
        let meta = DisplayArtifactManager::artifact_metadata(ArtifactKind::MaskLine);
        assert_eq!(meta.command_id, "edit.mask");
        assert_eq!(meta.category, "display");
        assert!(!meta.is_real_document_line);
        assert!(meta.applicable_modes.contains(&EditorMode::Edit));
        assert!(meta.applicable_modes.contains(&EditorMode::Browse));
        assert!(!meta.applicable_modes.contains(&EditorMode::View));
    }
}
