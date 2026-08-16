//! UNNUM command implementation.
//!
//! Provides argument parsing and execution logic for the UNNUM primary command.
//! Handles all variants: no args, COLS, FRONT, BACK, ALL.

use crate::error::SeqNumError;
use crate::state::SeqNumState;
use crate::strip::{strip_document, strip_line_range};
use crate::traits::{DocumentMutate, LanguageProfile};
use crate::types::ColumnRange;

/// Parsed UNNUM command variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnnumVariant {
    /// No arguments — use language profile columns.
    Default,
    /// Explicit column range: `UNNUM COLS start end`.
    Cols { range: ColumnRange },
    /// Strip only front columns: `UNNUM FRONT`.
    Front,
    /// Strip only back columns: `UNNUM BACK`.
    Back,
    /// Strip both front and back: `UNNUM ALL`.
    All,
}

/// Result of UNNUM execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnnumResult {
    /// Number of lines modified.
    pub lines_modified: usize,
    /// Status message for the user.
    pub message: String,
}

/// Parse UNNUM command arguments into a variant.
pub fn parse_unnum_args(args: &[&str]) -> Result<UnnumVariant, SeqNumError> {
    if args.is_empty() {
        return Ok(UnnumVariant::Default);
    }

    match args[0].to_uppercase().as_str() {
        "COLS" => {
            if args.len() < 3 {
                return Err(SeqNumError::InvalidColumnRange {
                    value: args[1..].join(" "),
                    reason: "UNNUM COLS requires start and end column numbers".to_string(),
                });
            }
            let start: u32 = args[1]
                .parse()
                .map_err(|_| SeqNumError::InvalidColumnRange {
                    value: args[1].to_string(),
                    reason: "not a valid column number".to_string(),
                })?;
            let end: u32 = args[2]
                .parse()
                .map_err(|_| SeqNumError::InvalidColumnRange {
                    value: args[2].to_string(),
                    reason: "not a valid column number".to_string(),
                })?;
            let range = ColumnRange::new(start, end)?;
            Ok(UnnumVariant::Cols { range })
        }
        "FRONT" => Ok(UnnumVariant::Front),
        "BACK" => Ok(UnnumVariant::Back),
        "ALL" => Ok(UnnumVariant::All),
        _ => Err(SeqNumError::InvalidColumnRange {
            value: args.join(" "),
            reason: "unrecognized UNNUM argument — expected COLS, FRONT, BACK, or ALL".to_string(),
        }),
    }
}

/// Execute the UNNUM command.
///
/// Resolves the column ranges to strip based on the variant and language profile,
/// then performs the strip operation.
pub fn execute_unnum(
    document: &mut dyn DocumentMutate,
    profile: &dyn LanguageProfile,
    variant: &UnnumVariant,
    scope: Option<(usize, usize)>,
    state: &mut SeqNumState,
) -> Result<UnnumResult, SeqNumError> {
    let ranges = resolve_ranges(variant, profile)?;

    let lines_modified = match scope {
        Some((start, end)) => {
            let result = strip_line_range(document, &ranges, start, end, &mut state.side_table);
            result.lines_modified
        }
        None => {
            let result = strip_document(document, &ranges, state);
            result.lines_modified
        }
    };

    let message = format!("UNNUM: {lines_modified} lines modified");

    Ok(UnnumResult {
        lines_modified,
        message,
    })
}

/// Resolve the column ranges for the given UNNUM variant and language profile.
fn resolve_ranges(
    variant: &UnnumVariant,
    profile: &dyn LanguageProfile,
) -> Result<Vec<ColumnRange>, SeqNumError> {
    match variant {
        UnnumVariant::Default | UnnumVariant::All => {
            let mut ranges = Vec::new();
            if let Some(front) = profile.sequence_cols_front() {
                ranges.push(front);
            }
            if let Some(back) = profile.sequence_cols_back() {
                ranges.push(back);
            }
            if ranges.is_empty() {
                return Err(SeqNumError::NoSequenceColumns {
                    command: "UNNUM".to_string(),
                });
            }
            Ok(ranges)
        }
        UnnumVariant::Cols { range } => Ok(vec![*range]),
        UnnumVariant::Front => {
            if let Some(front) = profile.sequence_cols_front() {
                Ok(vec![front])
            } else {
                Err(SeqNumError::FrontColumnsNotDefined {
                    command: "UNNUM".to_string(),
                })
            }
        }
        UnnumVariant::Back => {
            if let Some(back) = profile.sequence_cols_back() {
                Ok(vec![back])
            } else {
                Err(SeqNumError::BackColumnsNotDefined {
                    command: "UNNUM".to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::DocumentAccess;

    // ─── Test Helpers ───────────────────────────────────────────────────────

    struct MockDoc {
        lines: Vec<String>,
    }

    impl MockDoc {
        fn new(lines: &[&str]) -> Self {
            Self {
                lines: lines.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl DocumentAccess for MockDoc {
        fn line_count(&self) -> usize {
            self.lines.len()
        }
        fn line_content(&self, index: usize) -> Option<&str> {
            self.lines.get(index).map(|s| s.as_str())
        }
    }

    impl DocumentMutate for MockDoc {
        fn replace_columns(&mut self, line_index: usize, range: &ColumnRange, content: &str) {
            if let Some(line) = self.lines.get_mut(line_index) {
                let start = range.start_offset();
                let end = range.end_offset();
                if line.len() <= start {
                    return;
                }
                let actual_end = end.min(line.len());
                let mut new_line = String::with_capacity(line.len());
                new_line.push_str(&line[..start]);
                new_line.push_str(content);
                if actual_end < line.len() {
                    new_line.push_str(&line[actual_end..]);
                }
                *line = new_line;
            }
        }
    }

    struct MockProfile {
        front: Option<ColumnRange>,
        back: Option<ColumnRange>,
    }

    impl LanguageProfile for MockProfile {
        fn sequence_cols_front(&self) -> Option<ColumnRange> {
            self.front
        }
        fn sequence_cols_back(&self) -> Option<ColumnRange> {
            self.back
        }
        fn auto_unnum(&self) -> bool {
            true
        }
        fn language_id(&self) -> &str {
            "test"
        }
    }

    fn make_80col_line(front: &str, body: &str, back: &str) -> String {
        let f = format!("{:<6}", front);
        let b_pad = format!("{:<66}", body);
        let bk = format!("{:<8}", back);
        format!("{}{}{}", &f[..6], &b_pad[..66], &bk[..8])
    }

    // ─── Tests ──────────────────────────────────────────────────────────────

    #[test]
    fn parse_no_args() {
        // Validates: Requirement 5.2
        assert_eq!(parse_unnum_args(&[]).unwrap(), UnnumVariant::Default);
    }

    #[test]
    fn parse_cols_variant() {
        // Validates: Requirement 5.3
        let result = parse_unnum_args(&["COLS", "1", "6"]).unwrap();
        match result {
            UnnumVariant::Cols { range } => {
                assert_eq!(range.start(), 1);
                assert_eq!(range.end(), 6);
            }
            _ => panic!("Expected Cols variant"),
        }
    }

    #[test]
    fn parse_front_variant() {
        // Validates: Requirement 5.4
        assert_eq!(parse_unnum_args(&["FRONT"]).unwrap(), UnnumVariant::Front);
    }

    #[test]
    fn parse_back_variant() {
        // Validates: Requirement 5.5
        assert_eq!(parse_unnum_args(&["BACK"]).unwrap(), UnnumVariant::Back);
    }

    #[test]
    fn parse_all_variant() {
        // Validates: Requirement 5.6
        assert_eq!(parse_unnum_args(&["ALL"]).unwrap(), UnnumVariant::All);
    }

    #[test]
    fn execute_unnum_default_uses_profile() {
        // Validates: Requirement 5.2
        let lines: Vec<String> = (1..=5)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
        };
        let mut state = SeqNumState::new();

        let result =
            execute_unnum(&mut doc, &profile, &UnnumVariant::Default, None, &mut state).unwrap();

        assert_eq!(result.lines_modified, 5);
        assert!(result.message.contains("5 lines modified"));
    }

    #[test]
    fn execute_unnum_no_columns_defined_errors() {
        // Validates: Requirement 5.2
        let lines = vec!["some text"];
        let mut doc = MockDoc::new(&lines);
        let profile = MockProfile {
            front: None,
            back: None,
        };
        let mut state = SeqNumState::new();

        let result = execute_unnum(&mut doc, &profile, &UnnumVariant::Default, None, &mut state);
        assert!(result.is_err());
    }

    #[test]
    fn execute_unnum_front_not_defined_errors() {
        // Validates: Requirement 5.4
        let lines = vec!["some text"];
        let mut doc = MockDoc::new(&lines);
        let profile = MockProfile {
            front: None,
            back: Some(ColumnRange::new(73, 80).unwrap()),
        };
        let mut state = SeqNumState::new();

        let result = execute_unnum(&mut doc, &profile, &UnnumVariant::Front, None, &mut state);
        assert!(result.is_err());
    }

    #[test]
    fn execute_unnum_back_not_defined_errors() {
        // Validates: Requirement 5.5
        let lines = vec!["some text"];
        let mut doc = MockDoc::new(&lines);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: None,
        };
        let mut state = SeqNumState::new();

        let result = execute_unnum(&mut doc, &profile, &UnnumVariant::Back, None, &mut state);
        assert!(result.is_err());
    }

    #[test]
    fn execute_unnum_scoped() {
        // Validates: Requirement 5.7
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
        };
        let mut state = SeqNumState::new();

        let result = execute_unnum(
            &mut doc,
            &profile,
            &UnnumVariant::Default,
            Some((2, 5)),
            &mut state,
        )
        .unwrap();

        assert_eq!(result.lines_modified, 3);
        // Line 0 unchanged
        assert!(doc.line_content(0).unwrap().starts_with("000100"));
    }

    #[test]
    fn execute_unnum_status_message() {
        // Validates: Requirement 5.10
        let lines: Vec<String> = (1..=3)
            .map(|i| {
                make_80col_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut doc = MockDoc::new(&line_refs);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: None,
        };
        let mut state = SeqNumState::new();

        let result =
            execute_unnum(&mut doc, &profile, &UnnumVariant::Front, None, &mut state).unwrap();

        assert!(result.message.contains("3 lines modified"));
    }
}
