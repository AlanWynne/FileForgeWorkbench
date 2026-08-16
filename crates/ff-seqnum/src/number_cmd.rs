//! NUMBER command implementation.
//!
//! Provides argument parsing and execution logic for the NUMBER primary command.
//! Handles: no args (usage), COLS, STD, ON, OFF, SHOW, FORMAT.

use crate::error::SeqNumError;
use crate::number::{apply_numbering, validate_number_params, NumberResult};
use crate::number_show::toggle_show_mode;
use crate::state::SeqNumState;
use crate::traits::{DocumentMutate, LanguageProfile};
use crate::types::{ColumnRange, SequenceFormat};

/// Parsed NUMBER command variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberVariant {
    /// No arguments — display usage.
    Usage,
    /// NUMBER COLS start end [FORMAT format_name].
    Cols {
        range: ColumnRange,
        start_value: u32,
        increment: u32,
        format: SequenceFormat,
    },
    /// NUMBER STD [start increment].
    Std { start_value: u32, increment: u32 },
    /// NUMBER ON — enable auto-numbering.
    On,
    /// NUMBER OFF — disable auto-numbering.
    Off,
    /// NUMBER SHOW — toggle display overlay.
    Show,
}

/// Result of NUMBER command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberCommandResult {
    /// Usage information displayed.
    Usage { message: String },
    /// Needs confirmation before proceeding.
    NeedsConfirmation {
        message: String,
        variant: NumberVariant,
    },
    /// Numbering completed.
    Completed {
        result: NumberResult,
        message: String,
    },
    /// Auto-numbering toggled.
    AutoNumberToggled { active: bool, message: String },
    /// NUMBER SHOW toggled.
    ShowToggled { active: bool, message: String },
    /// Error occurred.
    Error { error: SeqNumError },
}

/// Parse NUMBER command arguments.
pub fn parse_number_args(args: &[&str]) -> Result<NumberVariant, SeqNumError> {
    if args.is_empty() {
        return Ok(NumberVariant::Usage);
    }

    match args[0].to_uppercase().as_str() {
        "COLS" => parse_cols_args(&args[1..]),
        "STD" => parse_std_args(&args[1..]),
        "ON" => Ok(NumberVariant::On),
        "OFF" => Ok(NumberVariant::Off),
        "SHOW" => Ok(NumberVariant::Show),
        _ => Err(SeqNumError::InvalidColumnRange {
            value: args.join(" "),
            reason: "unrecognized NUMBER argument — expected COLS, STD, ON, OFF, or SHOW"
                .to_string(),
        }),
    }
}

/// Parse COLS sub-arguments.
fn parse_cols_args(args: &[&str]) -> Result<NumberVariant, SeqNumError> {
    if args.len() < 2 {
        return Err(SeqNumError::InvalidColumnRange {
            value: args.join(" "),
            reason: "NUMBER COLS requires start and end column numbers".to_string(),
        });
    }
    let start_col: u32 = args[0]
        .parse()
        .map_err(|_| SeqNumError::InvalidColumnRange {
            value: args[0].to_string(),
            reason: "not a valid column number".to_string(),
        })?;
    let end_col: u32 = args[1]
        .parse()
        .map_err(|_| SeqNumError::InvalidColumnRange {
            value: args[1].to_string(),
            reason: "not a valid column number".to_string(),
        })?;
    let range = ColumnRange::new(start_col, end_col)?;

    let mut start_value = 1u32;
    let mut increment = 1u32;
    let mut format = SequenceFormat::Numeric;

    // Check for optional start/increment or FORMAT
    let mut i = 2;
    while i < args.len() {
        match args[i].to_uppercase().as_str() {
            "FORMAT" | "ALPHA" => {
                if args[i].to_uppercase() == "ALPHA" {
                    if i + 1 < args.len() {
                        format = SequenceFormat::AlphaPrefix {
                            prefix: args[i + 1].to_uppercase(),
                        };
                        i += 2;
                    } else {
                        return Err(SeqNumError::PrefixTooLong {
                            prefix: String::new(),
                            width: range.width(),
                        });
                    }
                } else {
                    // FORMAT format_name
                    i += 1;
                    if i < args.len() {
                        match args[i].to_uppercase().as_str() {
                            "NUMERIC" => {
                                format = SequenceFormat::Numeric;
                            }
                            "ALPHA" => {
                                i += 1;
                                if i < args.len() {
                                    format = SequenceFormat::AlphaPrefix {
                                        prefix: args[i].to_uppercase(),
                                    };
                                }
                            }
                            _ => {}
                        }
                    }
                    i += 1;
                }
            }
            _ => {
                // Try to parse as start_value increment
                if let Ok(sv) = args[i].parse::<u32>() {
                    start_value = sv;
                    if i + 1 < args.len() {
                        if let Ok(inc) = args[i + 1].parse::<u32>() {
                            increment = inc;
                            i += 1;
                        }
                    }
                }
                i += 1;
            }
        }
    }

    Ok(NumberVariant::Cols {
        range,
        start_value,
        increment,
        format,
    })
}

/// Parse STD sub-arguments.
fn parse_std_args(args: &[&str]) -> Result<NumberVariant, SeqNumError> {
    let mut start_value = 1u32;
    let mut increment = 1u32;

    if !args.is_empty() {
        if let Ok(sv) = args[0].parse::<i64>() {
            let (s, _) = validate_number_params(sv, 1)?;
            start_value = s;
        }
        if args.len() > 1 {
            if let Ok(inc) = args[1].parse::<i64>() {
                let (_, i) = validate_number_params(start_value as i64, inc)?;
                increment = i;
            }
        }
    }

    Ok(NumberVariant::Std {
        start_value,
        increment,
    })
}

/// Execute the NUMBER command (after confirmation if needed).
pub fn execute_number(
    document: &mut dyn DocumentMutate,
    profile: &dyn LanguageProfile,
    variant: &NumberVariant,
    scope: Option<(usize, usize)>,
    state: &mut SeqNumState,
    default_format: &SequenceFormat,
) -> NumberCommandResult {
    match variant {
        NumberVariant::Usage => NumberCommandResult::Usage {
            message: get_usage_text(),
        },
        NumberVariant::Cols {
            range,
            start_value,
            increment,
            format,
        } => {
            if !format.validate_for_width(range.width()) {
                return NumberCommandResult::Error {
                    error: SeqNumError::PrefixTooLong {
                        prefix: match format {
                            SequenceFormat::AlphaPrefix { prefix } => prefix.clone(),
                            _ => String::new(),
                        },
                        width: range.width(),
                    },
                };
            }
            let result = apply_numbering(document, range, *start_value, *increment, format, scope);
            let message = format_result_message(&result, range);
            NumberCommandResult::Completed { result, message }
        }
        NumberVariant::Std {
            start_value,
            increment,
        } => {
            let range = resolve_std_columns(profile);
            match range {
                Ok(cols) => {
                    let result = apply_numbering(
                        document,
                        &cols,
                        *start_value,
                        *increment,
                        default_format,
                        scope,
                    );
                    let message = format_result_message(&result, &cols);
                    NumberCommandResult::Completed { result, message }
                }
                Err(e) => NumberCommandResult::Error { error: e },
            }
        }
        NumberVariant::On => {
            state.auto_numbering_active = true;
            NumberCommandResult::AutoNumberToggled {
                active: true,
                message: "NUMBER ON: auto-numbering enabled".to_string(),
            }
        }
        NumberVariant::Off => {
            state.auto_numbering_active = false;
            state.auto_number_state = None;
            NumberCommandResult::AutoNumberToggled {
                active: false,
                message: "NUMBER OFF: auto-numbering disabled".to_string(),
            }
        }
        NumberVariant::Show => {
            let active = toggle_show_mode(state);
            let message = if active {
                "NUMBER SHOW: overlay enabled".to_string()
            } else {
                "NUMBER SHOW: overlay disabled".to_string()
            };
            NumberCommandResult::ShowToggled { active, message }
        }
    }
}

/// Resolve columns for NUMBER STD: prefer back, fallback to front.
fn resolve_std_columns(profile: &dyn LanguageProfile) -> Result<ColumnRange, SeqNumError> {
    if let Some(back) = profile.sequence_cols_back() {
        return Ok(back);
    }
    if let Some(front) = profile.sequence_cols_front() {
        return Ok(front);
    }
    Err(SeqNumError::NoSequenceColumns {
        command: "NUMBER".to_string(),
    })
}

/// Format the result message for a completed numbering operation.
fn format_result_message(result: &NumberResult, range: &ColumnRange) -> String {
    let mut msg = format!("NUMBER: {} lines numbered", result.lines_numbered);
    if result.overflow_occurred {
        msg.push_str(&format!(
            " — WARNING: sequence overflow at COLS {}-{}",
            range.start(),
            range.end()
        ));
    }
    msg
}

/// Generate confirmation prompt text.
pub fn get_confirmation_prompt(range: &ColumnRange) -> String {
    format!(
        "NUMBER will overwrite column range {}-{} on all lines. Confirm? (YES/NO)",
        range.start(),
        range.end()
    )
}

/// Get usage text for NUMBER with no arguments.
fn get_usage_text() -> String {
    "NUMBER command usage:\n\
     NUMBER COLS start end [start_val increment] [FORMAT NUMERIC|ALPHA prefix]\n\
     NUMBER STD [start_val increment]\n\
     NUMBER ON    — enable auto-numbering\n\
     NUMBER OFF   — disable auto-numbering\n\
     NUMBER SHOW  — toggle sequence number overlay"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::DocumentAccess;

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
                    let padding = " ".repeat(start - line.len());
                    line.push_str(&padding);
                    line.push_str(content);
                    return;
                }
                let actual_end = end.min(line.len());
                let mut new_line = String::with_capacity(line.len().max(end));
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

    #[test]
    fn parse_no_args_returns_usage() {
        // Validates: Requirement 6.2
        assert_eq!(parse_number_args(&[]).unwrap(), NumberVariant::Usage);
    }

    #[test]
    fn parse_cols_basic() {
        // Validates: Requirement 6.3
        let variant = parse_number_args(&["COLS", "73", "80"]).unwrap();
        match variant {
            NumberVariant::Cols {
                range,
                start_value,
                increment,
                ..
            } => {
                assert_eq!(range.start(), 73);
                assert_eq!(range.end(), 80);
                assert_eq!(start_value, 1);
                assert_eq!(increment, 1);
            }
            _ => panic!("Expected Cols"),
        }
    }

    #[test]
    fn parse_std_with_params() {
        // Validates: Requirement 6.5
        let variant = parse_number_args(&["STD", "10", "10"]).unwrap();
        match variant {
            NumberVariant::Std {
                start_value,
                increment,
            } => {
                assert_eq!(start_value, 10);
                assert_eq!(increment, 10);
            }
            _ => panic!("Expected Std"),
        }
    }

    #[test]
    fn parse_on() {
        // Validates: Requirement 6.7
        assert_eq!(parse_number_args(&["ON"]).unwrap(), NumberVariant::On);
    }

    #[test]
    fn parse_off() {
        // Validates: Requirement 6.8
        assert_eq!(parse_number_args(&["OFF"]).unwrap(), NumberVariant::Off);
    }

    #[test]
    fn parse_show() {
        // Validates: Requirement 8.1
        assert_eq!(parse_number_args(&["SHOW"]).unwrap(), NumberVariant::Show);
    }

    #[test]
    fn execute_number_usage() {
        // Validates: Requirement 6.2
        let mut doc = MockDoc::new(&["test"]);
        let profile = MockProfile {
            front: None,
            back: None,
        };
        let mut state = SeqNumState::new();
        let result = execute_number(
            &mut doc,
            &profile,
            &NumberVariant::Usage,
            None,
            &mut state,
            &SequenceFormat::Numeric,
        );
        match result {
            NumberCommandResult::Usage { message } => {
                assert!(message.contains("NUMBER COLS"));
            }
            _ => panic!("Expected Usage"),
        }
    }

    #[test]
    fn execute_number_on_off_toggle() {
        // Validates: Requirements 6.7, 6.8
        let mut doc = MockDoc::new(&["test"]);
        let profile = MockProfile {
            front: None,
            back: None,
        };
        let mut state = SeqNumState::new();

        let result = execute_number(
            &mut doc,
            &profile,
            &NumberVariant::On,
            None,
            &mut state,
            &SequenceFormat::Numeric,
        );
        match result {
            NumberCommandResult::AutoNumberToggled { active, .. } => assert!(active),
            _ => panic!("Expected AutoNumberToggled"),
        }
        assert!(state.auto_numbering_active);

        let result = execute_number(
            &mut doc,
            &profile,
            &NumberVariant::Off,
            None,
            &mut state,
            &SequenceFormat::Numeric,
        );
        match result {
            NumberCommandResult::AutoNumberToggled { active, .. } => assert!(!active),
            _ => panic!("Expected AutoNumberToggled"),
        }
        assert!(!state.auto_numbering_active);
    }

    #[test]
    fn execute_number_std_prefers_back() {
        // Validates: Requirement 6.4
        let lines = vec![
            "      LINE 1                                                                ",
            "      LINE 2                                                                ",
        ];
        let mut doc = MockDoc::new(&lines);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
        };
        let mut state = SeqNumState::new();

        let result = execute_number(
            &mut doc,
            &profile,
            &NumberVariant::Std {
                start_value: 100,
                increment: 100,
            },
            None,
            &mut state,
            &SequenceFormat::Numeric,
        );
        match result {
            NumberCommandResult::Completed { result, .. } => {
                assert_eq!(result.lines_numbered, 2);
            }
            _ => panic!("Expected Completed"),
        }
        // Back columns should be numbered
        assert!(doc.line_content(0).unwrap().contains("00000100"));
    }

    #[test]
    fn execute_number_std_fallback_to_front() {
        // Validates: Requirement 6.4
        let lines = vec!["      LINE 1"];
        let mut doc = MockDoc::new(&lines);
        let profile = MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: None,
        };
        let mut state = SeqNumState::new();

        let result = execute_number(
            &mut doc,
            &profile,
            &NumberVariant::Std {
                start_value: 1,
                increment: 1,
            },
            None,
            &mut state,
            &SequenceFormat::Numeric,
        );
        match result {
            NumberCommandResult::Completed { result, .. } => {
                assert_eq!(result.lines_numbered, 1);
            }
            _ => panic!("Expected Completed"),
        }
    }

    #[test]
    fn execute_number_std_no_columns_errors() {
        // Validates: Requirement 6.4
        let mut doc = MockDoc::new(&["test"]);
        let profile = MockProfile {
            front: None,
            back: None,
        };
        let mut state = SeqNumState::new();

        let result = execute_number(
            &mut doc,
            &profile,
            &NumberVariant::Std {
                start_value: 1,
                increment: 1,
            },
            None,
            &mut state,
            &SequenceFormat::Numeric,
        );
        match result {
            NumberCommandResult::Error { .. } => {}
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn confirmation_prompt_format() {
        // Validates: Requirement 6.9
        let range = ColumnRange::new(73, 80).unwrap();
        let prompt = get_confirmation_prompt(&range);
        assert!(prompt.contains("73-80"));
        assert!(prompt.contains("YES/NO"));
    }
}
