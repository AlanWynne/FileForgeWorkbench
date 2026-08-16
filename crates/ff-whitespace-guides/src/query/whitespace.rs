//! Whitespace glyph position computation.

use crate::modes::{TabDrawMode, WhitespaceVisibility};
use crate::types::{GlyphPosition, WhitespaceGlyph};

/// Compute the whitespace glyph positions for a single line.
///
/// Returns positions and glyph types based on the active visibility mode.
/// Returns an empty vec when visibility is `Invisible`.
///
/// # Arguments
///
/// * `line` - The line content as a byte slice.
/// * `tab_size` - The tab stop size (number of columns per tab).
/// * `visibility` - The current whitespace visibility mode.
/// * `tab_draw_mode` - How tab characters should be rendered.
///
/// Addresses: Requirement 1 AC 1.3–1.5, Requirement 2 AC 2.1–2.2
pub fn compute_whitespace_glyphs(
    line: &[u8],
    tab_size: u32,
    visibility: WhitespaceVisibility,
    tab_draw_mode: TabDrawMode,
) -> Vec<GlyphPosition> {
    if visibility == WhitespaceVisibility::Invisible {
        return Vec::new();
    }

    let tab_size = tab_size.max(1);
    let first_non_ws = find_first_non_whitespace(line);

    let mut glyphs = Vec::new();
    let mut column: u32 = 0;

    for (byte_idx, &byte) in line.iter().enumerate() {
        match byte {
            b' ' => {
                if should_include(byte_idx, first_non_ws, visibility) {
                    glyphs.push(GlyphPosition {
                        column,
                        glyph: WhitespaceGlyph::SpaceDot,
                    });
                }
                column += 1;
            }
            b'\t' => {
                let tab_width = tab_size - (column % tab_size);
                if should_include(byte_idx, first_non_ws, visibility) {
                    let glyph = match tab_draw_mode {
                        TabDrawMode::LongArrow => WhitespaceGlyph::TabArrow {
                            width_chars: tab_width,
                        },
                        TabDrawMode::Strikeout => WhitespaceGlyph::TabStrikeout {
                            width_chars: tab_width,
                        },
                    };
                    glyphs.push(GlyphPosition { column, glyph });
                }
                column += tab_width;
            }
            _ => {
                column += 1;
            }
        }
    }

    glyphs
}

/// Find the byte index of the first non-whitespace character, or `None` if all whitespace.
fn find_first_non_whitespace(line: &[u8]) -> Option<usize> {
    line.iter().position(|&b| b != b' ' && b != b'\t')
}

/// Determine whether a character at `byte_idx` should be included based on visibility mode.
fn should_include(
    byte_idx: usize,
    first_non_ws: Option<usize>,
    visibility: WhitespaceVisibility,
) -> bool {
    match visibility {
        WhitespaceVisibility::Invisible => false,
        WhitespaceVisibility::VisibleAlways => true,
        WhitespaceVisibility::VisibleAfterIndent => match first_non_ws {
            Some(first) => byte_idx > first,
            None => false, // all whitespace line — no "after indent" chars
        },
        WhitespaceVisibility::VisibleOnlyInIndent => match first_non_ws {
            Some(first) => byte_idx < first,
            None => true, // all whitespace line — all are "in indent"
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invisible_mode_returns_empty_for_any_line() {
        // Validates: Requirement 1.1
        let line = b"  hello  world  ";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::Invisible,
            TabDrawMode::LongArrow,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn visible_always_returns_all_whitespace() {
        // Validates: Requirement 1.3
        let line = b" a b ";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleAlways,
            TabDrawMode::LongArrow,
        );
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].column, 0); // leading space
        assert_eq!(result[1].column, 2); // middle space
        assert_eq!(result[2].column, 4); // trailing space
    }

    #[test]
    fn visible_after_indent_skips_leading() {
        // Validates: Requirement 1.4
        let line = b"  a b ";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleAfterIndent,
            TabDrawMode::LongArrow,
        );
        // First non-ws at byte_idx=2, so only bytes at idx 3 (space between a and b) and idx 5 (trailing)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].column, 3); // space between a and b
        assert_eq!(result[1].column, 5); // trailing space
    }

    #[test]
    fn visible_only_in_indent_returns_only_leading() {
        // Validates: Requirement 1.5
        let line = b"  a b ";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleOnlyInIndent,
            TabDrawMode::LongArrow,
        );
        // First non-ws at byte_idx=2, so only bytes at idx 0 and 1
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].column, 0);
        assert_eq!(result[1].column, 1);
    }

    #[test]
    fn tab_arrow_glyph_has_correct_width() {
        // Validates: Requirement 2.2
        let line = b"\thello";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleAlways,
            TabDrawMode::LongArrow,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].glyph,
            WhitespaceGlyph::TabArrow { width_chars: 4 }
        );
    }

    #[test]
    fn tab_strikeout_glyph_has_correct_width() {
        // Validates: Requirement 2.2
        let line = b"\thello";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleAlways,
            TabDrawMode::Strikeout,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].glyph,
            WhitespaceGlyph::TabStrikeout { width_chars: 4 }
        );
    }

    #[test]
    fn empty_line_returns_empty() {
        // Validates: Requirement 1.3
        let line = b"";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleAlways,
            TabDrawMode::LongArrow,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn all_whitespace_line_visible_only_in_indent_returns_all() {
        // Validates: Requirement 1.5
        let line = b"   ";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleOnlyInIndent,
            TabDrawMode::LongArrow,
        );
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn all_whitespace_line_visible_after_indent_returns_empty() {
        // Validates: Requirement 1.4
        let line = b"   ";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleAfterIndent,
            TabDrawMode::LongArrow,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn mixed_tabs_and_spaces_visible_always() {
        // Validates: Requirement 1.3
        let line = b"\t a";
        let result = compute_whitespace_glyphs(
            line,
            4,
            WhitespaceVisibility::VisibleAlways,
            TabDrawMode::LongArrow,
        );
        assert_eq!(result.len(), 2); // tab + space
        assert_eq!(result[0].column, 0);
        assert_eq!(
            result[0].glyph,
            WhitespaceGlyph::TabArrow { width_chars: 4 }
        );
        assert_eq!(result[1].column, 4);
        assert_eq!(result[1].glyph, WhitespaceGlyph::SpaceDot);
    }
}
