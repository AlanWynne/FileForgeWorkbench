//! Overstrike line merging engine.
//!
//! Combines `+` (overprint) lines with their base line to produce a `MergedLine`
//! with per-character bold and underline styling. This simulates the physical
//! effect of a line printer printing multiple characters at the same position.

/// Styling attributes for a single character in a merged line.
// Validates: Requirement 5.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CharStyle {
    /// Whether this character should be rendered in bold weight.
    pub bold: bool,
    /// Whether this character should be rendered with underline decoration.
    pub underline: bool,
}

/// A single character with its associated rendering style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyledChar {
    /// The character to display.
    pub character: char,
    /// Rendering attributes (bold, underline).
    pub style: CharStyle,
}

impl StyledChar {
    /// Create a plain (unstyled) character.
    pub fn plain(ch: char) -> Self {
        Self {
            character: ch,
            style: CharStyle::default(),
        }
    }
}

/// The result of merging a base line with one or more overprint lines.
///
/// Contains character-level styling information for bold and underline rendering.
// Validates: Requirement 5.1–5.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergedLine {
    /// The styled characters after all overprint passes have been applied.
    pub characters: Vec<StyledChar>,
    /// The original document line number of the base line.
    pub source_line: usize,
    /// Number of overprint lines that were merged into this line.
    pub overprint_count: usize,
}

impl MergedLine {
    /// Create a `MergedLine` from a base line (no overprinting applied yet).
    pub fn from_base(content: &str, source_line: usize) -> Self {
        let characters = content.chars().map(StyledChar::plain).collect();
        Self {
            characters,
            source_line,
            overprint_count: 0,
        }
    }

    /// Apply an overprint line to this merged line.
    ///
    /// Implements the character-by-character merging rules from Requirement 5.2:
    /// - Same char as base → bold
    /// - `-` or `_` over printable non-space → underline on base char
    /// - `-` or `_` over space → dash/underscore at that position
    /// - Different printable char over base → overwritten (last wins)
    /// - Space over base → leave unchanged
    // Validates: Requirement 5.2
    pub fn apply_overprint(&mut self, overprint_content: &str) {
        self.overprint_count += 1;

        for (i, op_char) in overprint_content.chars().enumerate() {
            if i >= self.characters.len() {
                // Overprint extends beyond base — append the overprint character
                self.characters.push(StyledChar::plain(op_char));
                continue;
            }

            let base = &mut self.characters[i];

            if op_char == ' ' {
                // Space in overprint → leave base unchanged
                continue;
            }

            if op_char == base.character {
                // Same character → bold
                base.style.bold = true;
            } else if (op_char == '-' || op_char == '_')
                && base.character != ' '
                && base.character.is_ascii_graphic()
            {
                // Dash/underscore over printable non-space → underline
                base.style.underline = true;
            } else if (op_char == '-' || op_char == '_') && base.character == ' ' {
                // Dash/underscore over space → render the dash/underscore
                base.character = op_char;
            } else {
                // Different printable character → overwrite (last wins)
                base.character = op_char;
                base.style = CharStyle::default();
            }
        }
    }

    /// The plain-text content (without styling) for export purposes.
    pub fn plain_text(&self) -> String {
        self.characters.iter().map(|sc| sc.character).collect()
    }

    /// Whether any character in this line has bold styling.
    pub fn has_bold(&self) -> bool {
        self.characters.iter().any(|sc| sc.style.bold)
    }

    /// Whether any character in this line has underline styling.
    pub fn has_underline(&self) -> bool {
        self.characters.iter().any(|sc| sc.style.underline)
    }
}

/// Result of merging a group of lines (base + overprints).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    /// A data line (possibly merged with overprints).
    Merged(MergedLine),
    /// An overprint with no preceding base line (diagnostic case).
    OrphanOverprint {
        /// The content of the orphan overprint line.
        content: String,
        /// 0-based document line number.
        source_line: usize,
    },
}

/// Merge a sequence of parsed lines, combining overprint lines with their base lines.
///
/// Lines with `+` control are merged into the preceding non-overprint line.
/// If a `+` line appears without a preceding base line, it produces an
/// `OrphanOverprint` result.
// Validates: Requirement 5.1–5.5
pub fn merge_overstrikes(
    controls: &[crate::control::AsaControl],
    contents: &[&str],
) -> Vec<MergeResult> {
    let mut results: Vec<MergeResult> = Vec::new();

    for (i, (&control, &content)) in controls.iter().zip(contents.iter()).enumerate() {
        if control.is_overstrike() {
            // Find the last Merged result and apply overprint to it
            let last_merged = results
                .iter_mut()
                .rev()
                .find(|r| matches!(r, MergeResult::Merged(_)));
            match last_merged {
                Some(MergeResult::Merged(ref mut merged)) => {
                    merged.apply_overprint(content);
                }
                _ => {
                    results.push(MergeResult::OrphanOverprint {
                        content: content.to_string(),
                        source_line: i,
                    });
                }
            }
        } else {
            results.push(MergeResult::Merged(MergedLine::from_base(content, i)));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 5.2
    fn same_char_overprint_produces_bold() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("HELLO");
        assert!(merged.characters[0].style.bold);
        assert!(merged.characters[4].style.bold);
        assert_eq!(merged.plain_text(), "HELLO");
    }

    #[test]
    // Validates: Requirement 5.2
    fn dash_overprint_on_printable_produces_underline() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("-----");
        assert!(merged.characters[0].style.underline);
        assert_eq!(merged.characters[0].character, 'H');
        assert_eq!(merged.plain_text(), "HELLO");
    }

    #[test]
    // Validates: Requirement 5.2
    fn underscore_overprint_on_printable_produces_underline() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("_____");
        assert!(merged.characters[0].style.underline);
        assert_eq!(merged.characters[0].character, 'H');
    }

    #[test]
    // Validates: Requirement 5.2
    fn dash_overprint_on_space_produces_dash() {
        let mut merged = MergedLine::from_base("   ", 0);
        merged.apply_overprint("---");
        assert_eq!(merged.characters[0].character, '-');
        assert_eq!(merged.characters[1].character, '-');
        assert_eq!(merged.characters[2].character, '-');
    }

    #[test]
    // Validates: Requirement 5.2
    fn different_char_overprint_overwrites() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("WORLD");
        assert_eq!(merged.plain_text(), "WORLD");
        assert!(!merged.characters[0].style.bold);
    }

    #[test]
    // Validates: Requirement 5.2
    fn space_in_overprint_leaves_base_unchanged() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("  L  ");
        assert_eq!(merged.characters[0].character, 'H');
        assert_eq!(merged.characters[1].character, 'E');
        assert_eq!(merged.characters[2].character, 'L');
        assert!(merged.characters[2].style.bold);
        assert_eq!(merged.characters[3].character, 'L');
        assert_eq!(merged.characters[4].character, 'O');
    }

    #[test]
    // Validates: Requirement 5.3
    fn multiple_overprints_merge_sequentially() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("HELLO"); // bold
        merged.apply_overprint("-----"); // underline
        assert!(merged.characters[0].style.bold);
        assert!(merged.characters[0].style.underline);
        assert_eq!(merged.overprint_count, 2);
    }

    #[test]
    // Validates: Requirement 5.2 — overprint longer than base
    fn overprint_longer_than_base_extends_merged_line() {
        let mut merged = MergedLine::from_base("HI", 0);
        merged.apply_overprint("HELLO");
        assert_eq!(merged.characters.len(), 5);
        assert_eq!(merged.plain_text(), "HELLO");
    }

    #[test]
    // Validates: Requirement 5.2 — overprint shorter than base
    fn overprint_shorter_than_base_leaves_remainder_unchanged() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("HI");
        assert_eq!(merged.characters[0].character, 'H');
        assert!(merged.characters[0].style.bold);
        assert_eq!(merged.characters[2].character, 'L');
        assert!(!merged.characters[2].style.bold);
    }

    #[test]
    // Validates: Requirement 5.5
    fn orphan_overprint_at_file_start() {
        use crate::control::AsaControl;
        let controls = vec![AsaControl::Overstrike, AsaControl::Space];
        let contents = vec!["ORPHAN", "NORMAL"];
        let results = merge_overstrikes(&controls, &contents);
        assert_eq!(results.len(), 2);
        assert!(matches!(
            &results[0],
            MergeResult::OrphanOverprint { source_line: 0, .. }
        ));
        assert!(matches!(&results[1], MergeResult::Merged(_)));
    }

    #[test]
    // Validates: Requirement 5.1
    fn overstrikes_merged_into_preceding_base() {
        use crate::control::AsaControl;
        let controls = vec![
            AsaControl::Space,
            AsaControl::Overstrike,
            AsaControl::Overstrike,
        ];
        let contents = vec!["BASE", "BASE", "----"];
        let results = merge_overstrikes(&controls, &contents);
        assert_eq!(results.len(), 1);
        if let MergeResult::Merged(ref merged) = results[0] {
            assert_eq!(merged.overprint_count, 2);
            assert!(merged.has_bold());
            assert!(merged.has_underline());
        } else {
            panic!("Expected Merged result");
        }
    }
}
