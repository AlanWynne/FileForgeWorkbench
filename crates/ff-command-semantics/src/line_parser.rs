//! Line command parser — interprets prefix-area strings into kind + count descriptors.
//!
//! Handles all defined line command kinds (single-line and block), repeat counts,
//! case-insensitive normalization, and unknown kind detection.

/// Maximum allowed repeat count for line commands.
const MAX_LINE_COMMAND_COUNT: u32 = 99999;

/// A parsed line command from the prefix area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineCommandDescriptor {
    /// A recognised line command with kind and repeat count.
    Known { kind: LineCommandKind, count: u32 },
    /// An unrecognised prefix-area string.
    Unknown(String),
}

/// The kind of a line command (single-line or block).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LineCommandKind {
    // Single-line commands
    /// Copy line (C)
    Copy,
    /// Move line (M)
    Move,
    /// Delete line (D)
    Delete,
    /// Repeat/duplicate line (R)
    Repeat,
    /// Exclude line from display (X)
    Exclude,
    /// Insert lines after (I)
    Insert,
    /// After destination (A)
    After,
    /// Before destination (B)
    Before,
    /// Overlay (O)
    Overlay,
    /// Show/reveal excluded line (W)
    Show,
    /// Select line (S)
    Select,
    /// Tag line (T)
    Tag,
    /// Shift right (>)
    ShiftRight,
    /// Shift left (<)
    ShiftLeft,
    /// Indent in (()
    IndentIn,
    /// Indent out ())
    IndentOut,
    /// Set bounds (])
    Bounds,

    // Block commands (paired)
    /// Copy block (CC)
    CopyBlock,
    /// Move block (MM)
    MoveBlock,
    /// Delete block (DD)
    DeleteBlock,
    /// Repeat block (RR)
    RepeatBlock,
    /// Exclude block (XX)
    ExcludeBlock,
    /// Tag block (TT)
    TagBlock,
}

impl LineCommandKind {
    /// Returns true if this is a block command that requires pairing.
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            Self::CopyBlock
                | Self::MoveBlock
                | Self::DeleteBlock
                | Self::RepeatBlock
                | Self::ExcludeBlock
                | Self::TagBlock
        )
    }

    /// Returns the text representation (e.g., "C", "CC", "M", "MM").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Copy => "C",
            Self::Move => "M",
            Self::Delete => "D",
            Self::Repeat => "R",
            Self::Exclude => "X",
            Self::Insert => "I",
            Self::After => "A",
            Self::Before => "B",
            Self::Overlay => "O",
            Self::Show => "W",
            Self::Select => "S",
            Self::Tag => "T",
            Self::ShiftRight => ">",
            Self::ShiftLeft => "<",
            Self::IndentIn => "(",
            Self::IndentOut => ")",
            Self::Bounds => "]",
            Self::CopyBlock => "CC",
            Self::MoveBlock => "MM",
            Self::DeleteBlock => "DD",
            Self::RepeatBlock => "RR",
            Self::ExcludeBlock => "XX",
            Self::TagBlock => "TT",
        }
    }

    /// Attempt to parse a kind string into a LineCommandKind.
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "CC" => Some(Self::CopyBlock),
            "MM" => Some(Self::MoveBlock),
            "DD" => Some(Self::DeleteBlock),
            "RR" => Some(Self::RepeatBlock),
            "XX" => Some(Self::ExcludeBlock),
            "TT" => Some(Self::TagBlock),
            "C" => Some(Self::Copy),
            "M" => Some(Self::Move),
            "D" => Some(Self::Delete),
            "R" => Some(Self::Repeat),
            "X" => Some(Self::Exclude),
            "I" => Some(Self::Insert),
            "A" => Some(Self::After),
            "B" => Some(Self::Before),
            "O" => Some(Self::Overlay),
            "W" => Some(Self::Show),
            "S" => Some(Self::Select),
            "T" => Some(Self::Tag),
            ">" => Some(Self::ShiftRight),
            "<" => Some(Self::ShiftLeft),
            "(" => Some(Self::IndentIn),
            ")" => Some(Self::IndentOut),
            "]" => Some(Self::Bounds),
            _ => None,
        }
    }
}

/// Parses prefix-area strings into line command descriptors.
pub struct LineCommandParser;

/// Error returned when a line command count exceeds the maximum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCommandCountOverflow {
    /// The count that was too large.
    pub count: u64,
}

impl LineCommandParser {
    /// Parse a prefix-area string into a LineCommandDescriptor.
    ///
    /// Returns `None` for empty/whitespace-only input.
    /// Returns `Err` if the count exceeds the maximum (99999).
    pub fn parse(input: &str) -> Result<Option<LineCommandDescriptor>, LineCommandCountOverflow> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let upper = trimmed.to_uppercase();

        // Find the split between the kind prefix and numeric count suffix.
        // The kind is the maximal leading non-digit prefix.
        let kind_end = upper
            .chars()
            .position(|c| c.is_ascii_digit())
            .unwrap_or(upper.len());

        let kind_str = &upper[..kind_end];
        let count_str = &upper[kind_end..];

        // Parse count (default 1 if empty)
        let count: u64 = if count_str.is_empty() {
            1
        } else {
            count_str.parse::<u64>().unwrap_or(0)
        };

        // Validate count range
        if count > MAX_LINE_COMMAND_COUNT as u64 {
            return Err(LineCommandCountOverflow { count });
        }

        // Special case: count of 0 should be treated as default 1
        let count = if count == 0 { 1 } else { count as u32 };

        // Try to match the kind
        match LineCommandKind::from_str(kind_str) {
            Some(kind) => Ok(Some(LineCommandDescriptor::Known { kind, count })),
            None => {
                // Unknown kind — preserve original text
                Ok(Some(LineCommandDescriptor::Unknown(trimmed.to_string())))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4.1
    #[test]
    fn parse_single_line_command_no_count() {
        let result = LineCommandParser::parse("C").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Copy,
                count: 1,
            }
        );
    }

    // Validates: Requirement 4.1
    #[test]
    fn parse_single_line_command_with_count() {
        let result = LineCommandParser::parse("M5").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Move,
                count: 5,
            }
        );
    }

    // Validates: Requirement 4.5
    #[test]
    fn parse_block_command() {
        let result = LineCommandParser::parse("CC").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::CopyBlock,
                count: 1,
            }
        );
    }

    // Validates: Requirement 4.5
    #[test]
    fn parse_block_command_dd() {
        let result = LineCommandParser::parse("DD").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::DeleteBlock,
                count: 1,
            }
        );
    }

    // Validates: Requirement 4.2
    #[test]
    fn parse_case_insensitive() {
        let result = LineCommandParser::parse("c").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Copy,
                count: 1,
            }
        );

        let result = LineCommandParser::parse("dd").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::DeleteBlock,
                count: 1,
            }
        );
    }

    // Validates: Requirement 4.6
    #[test]
    fn parse_empty_returns_none() {
        assert_eq!(LineCommandParser::parse("").unwrap(), None);
        assert_eq!(LineCommandParser::parse("   ").unwrap(), None);
        assert_eq!(LineCommandParser::parse("\t").unwrap(), None);
    }

    // Validates: Requirement 4.4
    #[test]
    fn parse_unknown_kind_produces_unknown_variant() {
        let result = LineCommandParser::parse("ZZ").unwrap().unwrap();
        assert_eq!(result, LineCommandDescriptor::Unknown("ZZ".to_string()));
    }

    // Validates: Requirement 4.7
    #[test]
    fn parse_count_at_max_accepted() {
        let result = LineCommandParser::parse("D99999").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Delete,
                count: 99999,
            }
        );
    }

    // Validates: Requirement 4.7
    #[test]
    fn parse_count_exceeds_max_returns_error() {
        let result = LineCommandParser::parse("D100000");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().count, 100000);
    }

    // Validates: Requirement 4.3
    #[test]
    fn parse_all_single_line_kinds() {
        let cases = vec![
            ("C", LineCommandKind::Copy),
            ("M", LineCommandKind::Move),
            ("D", LineCommandKind::Delete),
            ("R", LineCommandKind::Repeat),
            ("X", LineCommandKind::Exclude),
            ("I", LineCommandKind::Insert),
            ("A", LineCommandKind::After),
            ("B", LineCommandKind::Before),
            ("O", LineCommandKind::Overlay),
            ("W", LineCommandKind::Show),
            ("S", LineCommandKind::Select),
            ("T", LineCommandKind::Tag),
            (">", LineCommandKind::ShiftRight),
            ("<", LineCommandKind::ShiftLeft),
            ("(", LineCommandKind::IndentIn),
            (")", LineCommandKind::IndentOut),
            ("]", LineCommandKind::Bounds),
        ];
        for (input, expected_kind) in cases {
            let result = LineCommandParser::parse(input).unwrap().unwrap();
            assert_eq!(
                result,
                LineCommandDescriptor::Known {
                    kind: expected_kind,
                    count: 1,
                },
                "Failed for input: {}",
                input
            );
        }
    }

    // Validates: Requirement 4.3
    #[test]
    fn parse_all_block_kinds() {
        let cases = vec![
            ("CC", LineCommandKind::CopyBlock),
            ("MM", LineCommandKind::MoveBlock),
            ("DD", LineCommandKind::DeleteBlock),
            ("RR", LineCommandKind::RepeatBlock),
            ("XX", LineCommandKind::ExcludeBlock),
            ("TT", LineCommandKind::TagBlock),
        ];
        for (input, expected_kind) in cases {
            let result = LineCommandParser::parse(input).unwrap().unwrap();
            assert_eq!(
                result,
                LineCommandDescriptor::Known {
                    kind: expected_kind,
                    count: 1,
                },
                "Failed for input: {}",
                input
            );
        }
    }

    // Validates: Requirement 4.5
    #[test]
    fn parse_kind_count_decomposition() {
        let result = LineCommandParser::parse("M10").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Move,
                count: 10,
            }
        );
    }

    #[test]
    fn block_command_is_block_returns_true() {
        assert!(LineCommandKind::CopyBlock.is_block());
        assert!(LineCommandKind::MoveBlock.is_block());
        assert!(LineCommandKind::DeleteBlock.is_block());
        assert!(!LineCommandKind::Copy.is_block());
        assert!(!LineCommandKind::Delete.is_block());
    }

    #[test]
    fn kind_as_str_returns_correct_text() {
        assert_eq!(LineCommandKind::Copy.as_str(), "C");
        assert_eq!(LineCommandKind::CopyBlock.as_str(), "CC");
        assert_eq!(LineCommandKind::ShiftRight.as_str(), ">");
    }

    // Validates: Requirement 4.5
    #[test]
    fn parse_d1_gives_delete_count_1() {
        let result = LineCommandParser::parse("D1").unwrap().unwrap();
        assert_eq!(
            result,
            LineCommandDescriptor::Known {
                kind: LineCommandKind::Delete,
                count: 1,
            }
        );
    }
}
