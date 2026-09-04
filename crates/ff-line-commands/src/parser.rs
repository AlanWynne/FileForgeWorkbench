//! Line command parser — converts prefix-area input strings into typed commands.
//!
//! Case-insensitive. Supports optional numeric counts.

use crate::command::{LineCommandKind, ParsedLineCommand};
use crate::error::LineCommandError;

/// Parses raw prefix-area input strings into typed line commands.
pub struct LineCommandParser;

impl LineCommandParser {
    /// Parse a single prefix-area input string for a given line.
    ///
    /// Returns `Ok(ParsedLineCommand)` on success.
    /// Returns `Err(LineCommandError::InvalidCommand)` if unrecognised.
    pub fn parse(input: &str, line: u64) -> Result<ParsedLineCommand, LineCommandError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(LineCommandError::InvalidCommand {
                input: input.to_string(),
            });
        }

        let upper = trimmed.to_uppercase();
        let kind = Self::parse_kind(&upper, trimmed)?;

        Ok(ParsedLineCommand { line, kind })
    }

    fn parse_kind(upper: &str, original: &str) -> Result<LineCommandKind, LineCommandError> {
        // Try doubled (block) commands first — order matters for >>/<< vs >n/<n
        match upper {
            "DD" => return Ok(LineCommandKind::DeleteBlock),
            "RR" => return Ok(LineCommandKind::RepeatBlock),
            "CC" => return Ok(LineCommandKind::CopyBlock),
            "MM" => return Ok(LineCommandKind::MoveBlock),
            "XX" => return Ok(LineCommandKind::ExcludeBlock),
            "TT" => return Ok(LineCommandKind::TagBlock),
            "UU" => return Ok(LineCommandKind::UntagBlock),
            ">>" => return Ok(LineCommandKind::ShiftRightBlock),
            "<<" => return Ok(LineCommandKind::ShiftLeftBlock),
            "))" => return Ok(LineCommandKind::BoundsShiftRightBlock),
            "((" => return Ok(LineCommandKind::BoundsShiftLeftBlock),
            "WW" => return Ok(LineCommandKind::ClipboardCopyBlock),
            "]]" => return Ok(LineCommandKind::ShiftRightOneBlock),
            _ => {}
        }

        // Single-character commands without counts
        match upper {
            "D" => return Ok(LineCommandKind::Delete),
            "I" => return Ok(LineCommandKind::Insert),
            "R" => return Ok(LineCommandKind::Repeat),
            "C" => return Ok(LineCommandKind::Copy),
            "M" => return Ok(LineCommandKind::Move),
            "A" => return Ok(LineCommandKind::After),
            "B" => return Ok(LineCommandKind::Before),
            "X" => return Ok(LineCommandKind::Exclude),
            "T" => return Ok(LineCommandKind::Tag),
            "U" => return Ok(LineCommandKind::Untag),
            ">" => return Ok(LineCommandKind::ShiftRight),
            "<" => return Ok(LineCommandKind::ShiftLeft),
            ")" => return Ok(LineCommandKind::BoundsShiftRight),
            "(" => return Ok(LineCommandKind::BoundsShiftLeft),
            "W" => return Ok(LineCommandKind::ClipboardCopy),
            "F" => return Ok(LineCommandKind::ShowFirst),
            "L" => return Ok(LineCommandKind::ShowLast),
            "S" => return Ok(LineCommandKind::ShowLine),
            "]" => return Ok(LineCommandKind::ShiftRightOne),
            "O" => return Ok(LineCommandKind::Overlay),
            _ => {}
        }

        // Commands with numeric counts: D<n>, I<n>, R<n>, X<n>, ><n>, <<n>
        if upper.len() >= 2 {
            let first_char = upper.as_bytes()[0];
            let rest = &upper[1..];

            match first_char {
                b'D' => {
                    if let Ok(n) = rest.parse::<u32>() {
                        if n > 0 {
                            return Ok(LineCommandKind::DeleteCount(n));
                        }
                    }
                }
                b'I' => {
                    if let Ok(n) = rest.parse::<u32>() {
                        if n > 0 {
                            return Ok(LineCommandKind::InsertCount(n));
                        }
                    }
                }
                b'R' => {
                    if let Ok(n) = rest.parse::<u32>() {
                        if n > 0 {
                            return Ok(LineCommandKind::RepeatCount(n));
                        }
                    }
                }
                b'X' => {
                    if let Ok(n) = rest.parse::<u32>() {
                        if n > 0 {
                            return Ok(LineCommandKind::ExcludeCount(n));
                        }
                    }
                }
                b'>' => {
                    if let Ok(n) = rest.parse::<u32>() {
                        if n > 0 {
                            return Ok(LineCommandKind::ShiftRightCount(n));
                        }
                    }
                }
                b'<' => {
                    if let Ok(n) = rest.parse::<u32>() {
                        if n > 0 {
                            return Ok(LineCommandKind::ShiftLeftCount(n));
                        }
                    }
                }
                b'O' => {
                    if let Ok(n) = rest.parse::<u32>() {
                        if n > 0 {
                            return Ok(LineCommandKind::OverlayCount(n));
                        }
                    }
                }
                _ => {}
            }
        }

        Err(LineCommandError::InvalidCommand {
            input: original.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Delete variants ---

    #[test]
    fn parse_d_single_delete() {
        let cmd = LineCommandParser::parse("D", 5).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Delete);
        assert_eq!(cmd.line, 5);
    }

    #[test]
    fn parse_d_lowercase() {
        let cmd = LineCommandParser::parse("d", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Delete);
    }

    #[test]
    fn parse_d5_delete_count() {
        let cmd = LineCommandParser::parse("D5", 3).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::DeleteCount(5));
    }

    #[test]
    fn parse_d99_delete_count() {
        let cmd = LineCommandParser::parse("d99", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::DeleteCount(99));
    }

    #[test]
    fn parse_dd_delete_block() {
        let cmd = LineCommandParser::parse("DD", 10).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::DeleteBlock);
    }

    #[test]
    fn parse_dd_lowercase() {
        let cmd = LineCommandParser::parse("dd", 2).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::DeleteBlock);
    }

    // --- Insert variants ---

    #[test]
    fn parse_i_single_insert() {
        let cmd = LineCommandParser::parse("I", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Insert);
    }

    #[test]
    fn parse_i3_insert_count() {
        let cmd = LineCommandParser::parse("I3", 7).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::InsertCount(3));
    }

    // --- Repeat variants ---

    #[test]
    fn parse_r_single_repeat() {
        let cmd = LineCommandParser::parse("R", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Repeat);
    }

    #[test]
    fn parse_r4_repeat_count() {
        let cmd = LineCommandParser::parse("R4", 1).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::RepeatCount(4));
    }

    #[test]
    fn parse_rr_repeat_block() {
        let cmd = LineCommandParser::parse("RR", 5).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::RepeatBlock);
    }

    // --- Copy/Move/Target ---

    #[test]
    fn parse_c_copy() {
        let cmd = LineCommandParser::parse("C", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Copy);
    }

    #[test]
    fn parse_cc_copy_block() {
        let cmd = LineCommandParser::parse("CC", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::CopyBlock);
    }

    #[test]
    fn parse_m_move() {
        let cmd = LineCommandParser::parse("M", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Move);
    }

    #[test]
    fn parse_mm_move_block() {
        let cmd = LineCommandParser::parse("MM", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::MoveBlock);
    }

    #[test]
    fn parse_a_after() {
        let cmd = LineCommandParser::parse("A", 8).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::After);
    }

    #[test]
    fn parse_b_before() {
        let cmd = LineCommandParser::parse("B", 1).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Before);
    }

    // --- Exclude ---

    #[test]
    fn parse_x_exclude() {
        let cmd = LineCommandParser::parse("X", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Exclude);
    }

    #[test]
    fn parse_x5_exclude_count() {
        let cmd = LineCommandParser::parse("X5", 2).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ExcludeCount(5));
    }

    #[test]
    fn parse_xx_exclude_block() {
        let cmd = LineCommandParser::parse("XX", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ExcludeBlock);
    }

    // --- Tag/Untag ---

    #[test]
    fn parse_t_tag() {
        let cmd = LineCommandParser::parse("T", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Tag);
    }

    #[test]
    fn parse_tt_tag_block() {
        let cmd = LineCommandParser::parse("TT", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::TagBlock);
    }

    #[test]
    fn parse_u_untag() {
        let cmd = LineCommandParser::parse("U", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Untag);
    }

    #[test]
    fn parse_uu_untag_block() {
        let cmd = LineCommandParser::parse("UU", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::UntagBlock);
    }

    // --- Shift Right ---

    #[test]
    fn parse_shift_right() {
        let cmd = LineCommandParser::parse(">", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftRight);
    }

    #[test]
    fn parse_shift_right_count() {
        let cmd = LineCommandParser::parse(">5", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftRightCount(5));
    }

    #[test]
    fn parse_shift_right_block() {
        let cmd = LineCommandParser::parse(">>", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftRightBlock);
    }

    // --- Shift Left ---

    #[test]
    fn parse_shift_left() {
        let cmd = LineCommandParser::parse("<", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftLeft);
    }

    #[test]
    fn parse_shift_left_count() {
        let cmd = LineCommandParser::parse("<3", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftLeftCount(3));
    }

    #[test]
    fn parse_shift_left_block() {
        let cmd = LineCommandParser::parse("<<", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftLeftBlock);
    }

    // --- Bounds-Aware Shift ---

    #[test]
    fn parse_bounds_shift_right() {
        let cmd = LineCommandParser::parse(")", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::BoundsShiftRight);
    }

    #[test]
    fn parse_bounds_shift_right_block() {
        let cmd = LineCommandParser::parse("))", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::BoundsShiftRightBlock);
    }

    #[test]
    fn parse_bounds_shift_left() {
        let cmd = LineCommandParser::parse("(", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::BoundsShiftLeft);
    }

    #[test]
    fn parse_bounds_shift_left_block() {
        let cmd = LineCommandParser::parse("((", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::BoundsShiftLeftBlock);
    }

    // --- Rejection / Error ---

    #[test]
    fn parse_empty_string_returns_error() {
        let result = LineCommandParser::parse("", 0);
        assert!(result.is_err());
    }

    #[test]
    fn parse_whitespace_only_returns_error() {
        let result = LineCommandParser::parse("   ", 0);
        assert!(result.is_err());
    }

    #[test]
    fn parse_gibberish_returns_error() {
        let result = LineCommandParser::parse("ZZZ", 0);
        assert!(result.is_err());
        match result {
            Err(LineCommandError::InvalidCommand { input }) => {
                assert_eq!(input, "ZZZ");
            }
            _ => panic!("Expected InvalidCommand error"),
        }
    }

    #[test]
    fn parse_partial_match_returns_error() {
        let result = LineCommandParser::parse("DX", 0);
        assert!(result.is_err());
    }

    #[test]
    fn parse_d0_returns_error() {
        // Zero count is invalid
        let result = LineCommandParser::parse("D0", 0);
        assert!(result.is_err());
    }

    #[test]
    fn parse_mixed_case_works() {
        let cmd = LineCommandParser::parse("Dd", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::DeleteBlock);
    }

    #[test]
    fn parse_with_leading_whitespace() {
        let cmd = LineCommandParser::parse("  D", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Delete);
    }

    #[test]
    fn parse_with_trailing_whitespace() {
        let cmd = LineCommandParser::parse("D  ", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Delete);
    }

    // --- BX: Overlay ---

    #[test]
    fn parse_o_overlay() {
        // Validates: Requirement 15.1
        let cmd = LineCommandParser::parse("O", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Overlay);
    }

    #[test]
    fn parse_o3_overlay_count() {
        // Validates: Requirement 15.2
        let cmd = LineCommandParser::parse("O3", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::OverlayCount(3));
    }

    #[test]
    fn parse_o_lowercase() {
        let cmd = LineCommandParser::parse("o", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::Overlay);
    }

    // --- BX: Clipboard Copy ---

    #[test]
    fn parse_w_clipboard_copy() {
        // Validates: Requirement 15.3
        let cmd = LineCommandParser::parse("W", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ClipboardCopy);
    }

    #[test]
    fn parse_ww_clipboard_copy_block() {
        // Validates: Requirement 15.4
        let cmd = LineCommandParser::parse("WW", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ClipboardCopyBlock);
    }

    #[test]
    fn parse_w_lowercase() {
        let cmd = LineCommandParser::parse("w", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ClipboardCopy);
    }

    // --- BX: Show Excluded ---

    #[test]
    fn parse_f_show_first() {
        // Validates: Requirement 15.5
        let cmd = LineCommandParser::parse("F", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShowFirst);
    }

    #[test]
    fn parse_l_show_last() {
        // Validates: Requirement 15.6
        let cmd = LineCommandParser::parse("L", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShowLast);
    }

    #[test]
    fn parse_s_show_line() {
        // Validates: Requirement 15.9
        let cmd = LineCommandParser::parse("S", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShowLine);
    }

    // --- BX: Single-Column Shift Right ---

    #[test]
    fn parse_bracket_shift_right_one() {
        // Validates: Requirement 15.7
        let cmd = LineCommandParser::parse("]", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftRightOne);
    }

    #[test]
    fn parse_double_bracket_shift_right_one_block() {
        // Validates: Requirement 15.8
        let cmd = LineCommandParser::parse("]]", 0).unwrap();
        assert_eq!(cmd.kind, LineCommandKind::ShiftRightOneBlock);
    }
}
