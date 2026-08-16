//! SORT command implementation.
//!
//! Performs stable sorting of document lines by a column-key comparison.
//! The only undoable command in this crate.

use crate::error::NavigationError;
use crate::types::{ActiveBounds, SortDirection, SortParams, SortScope};

/// The undo record for a SORT operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortUndoRecord {
    /// The original line ordering (indices before sort).
    pub original_order: Vec<usize>,
    /// Description for undo history.
    pub description: String,
}

/// SORT command executor.
pub struct SortCommand;

impl SortCommand {
    /// Execute SORT on the given lines.
    ///
    /// Returns the undo record (original indices) for transaction recording.
    ///
    /// # Arguments
    ///
    /// * `lines` - The lines to sort (mutable, sorted in-place).
    /// * `params` - Parsed SORT parameters.
    /// * `bounds` - Optional active bounds to intersect with explicit column range.
    ///
    /// # Errors
    ///
    /// Returns `NavigationError::NothingToSort` if fewer than 2 lines are in scope.
    pub fn execute(
        lines: &mut [String],
        params: &SortParams,
        bounds: Option<ActiveBounds>,
    ) -> Result<SortUndoRecord, NavigationError> {
        if lines.len() < 2 {
            return Err(NavigationError::NothingToSort);
        }

        // Determine effective column range
        let effective_range = Self::resolve_column_range(params.column_range, bounds);

        // Create indexed entries to preserve original order for undo
        let mut indexed: Vec<(usize, &str)> = lines
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.as_str()))
            .collect();

        // Stable sort by extracted key
        indexed.sort_by(|a, b| {
            let key_a = Self::extract_key(a.1, effective_range);
            let key_b = Self::extract_key(b.1, effective_range);
            match params.direction {
                SortDirection::Ascending => key_a.cmp(&key_b),
                SortDirection::Descending => key_b.cmp(&key_a),
            }
        });

        // Record original order for undo
        let original_order: Vec<usize> = indexed.iter().map(|(i, _)| *i).collect();

        // Apply the sorted order
        let sorted_lines: Vec<String> = indexed.iter().map(|(i, _)| lines[*i].clone()).collect();
        lines.clone_from_slice(&sorted_lines);

        Ok(SortUndoRecord {
            original_order,
            description: "SORT".to_string(),
        })
    }

    /// Resolve the effective column range considering bounds intersection.
    fn resolve_column_range(
        explicit: Option<(u64, u64)>,
        bounds: Option<ActiveBounds>,
    ) -> Option<(u64, u64)> {
        match (bounds, explicit) {
            (Some(b), Some((col1, col2))) => b.intersect(col1, col2),
            (Some(b), None) => Some((b.left, b.right)),
            (None, explicit) => explicit,
        }
    }

    /// Extract the sort key from a line given an optional column range.
    ///
    /// Columns are 1-based, inclusive. If the line is shorter than the range,
    /// the key is padded with spaces (shorter lines sort before longer ones
    /// in ascending order).
    fn extract_key(line: &str, range: Option<(u64, u64)>) -> String {
        match range {
            Some((col1, col2)) => {
                let start = (col1 as usize).saturating_sub(1);
                let end = col2 as usize;
                let chars: Vec<char> = line.chars().collect();
                let slice_end = end.min(chars.len());
                if start >= chars.len() {
                    String::new()
                } else {
                    chars[start..slice_end].iter().collect()
                }
            }
            None => line.to_string(),
        }
    }

    /// Parse SORT command arguments.
    ///
    /// Format: `SORT [col1 col2] [A|D] [TAGGED|VISIBLE]`
    pub fn parse_args(args: &[&str]) -> Result<SortParams, NavigationError> {
        let mut column_range = None;
        let mut direction = SortDirection::Ascending;
        let mut scope = SortScope::AllVisible;
        let mut idx = 0;

        // Try to parse two consecutive integers as column range
        if idx + 1 < args.len() {
            if let (Ok(col1), Ok(col2)) = (args[idx].parse::<u64>(), args[idx + 1].parse::<u64>()) {
                column_range = Some((col1, col2));
                idx += 2;
            }
        }

        // Parse remaining arguments
        while idx < args.len() {
            match args[idx].to_uppercase().as_str() {
                "A" => direction = SortDirection::Ascending,
                "D" => direction = SortDirection::Descending,
                "TAGGED" => scope = SortScope::Tagged,
                "VISIBLE" => scope = SortScope::Visible,
                other => {
                    return Err(NavigationError::InvalidArgument {
                        command: "SORT".to_string(),
                        description: format!("Unknown argument: {other}"),
                    });
                }
            }
            idx += 1;
        }

        Ok(SortParams {
            column_range,
            direction,
            scope,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_ascending_full_line() {
        // Validates: Requirement 2.1, 2.3
        let mut lines = vec![
            "cherry".to_string(),
            "apple".to_string(),
            "banana".to_string(),
        ];
        let params = SortParams {
            column_range: None,
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        let result = SortCommand::execute(&mut lines, &params, None);
        assert!(result.is_ok());
        assert_eq!(lines, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn sort_descending() {
        // Validates: Requirement 2.4
        let mut lines = vec![
            "apple".to_string(),
            "cherry".to_string(),
            "banana".to_string(),
        ];
        let params = SortParams {
            column_range: None,
            direction: SortDirection::Descending,
            scope: SortScope::AllVisible,
        };
        let result = SortCommand::execute(&mut lines, &params, None);
        assert!(result.is_ok());
        assert_eq!(lines, vec!["cherry", "banana", "apple"]);
    }

    #[test]
    fn sort_by_column_range() {
        // Validates: Requirement 2.2
        let mut lines = vec![
            "XX_cherry".to_string(),
            "AA_apple".to_string(),
            "MM_banana".to_string(),
        ];
        let params = SortParams {
            column_range: Some((4, 9)),
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        let result = SortCommand::execute(&mut lines, &params, None);
        assert!(result.is_ok());
        assert_eq!(lines[0], "AA_apple");
        assert_eq!(lines[1], "MM_banana");
        assert_eq!(lines[2], "XX_cherry");
    }

    #[test]
    fn sort_stable_with_equal_keys() {
        // Validates: Requirement 2.8
        let mut lines = vec![
            "AAA first".to_string(),
            "AAA second".to_string(),
            "AAA third".to_string(),
        ];
        let params = SortParams {
            column_range: Some((1, 3)),
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        let result = SortCommand::execute(&mut lines, &params, None);
        assert!(result.is_ok());
        // Stable: equal keys retain original order
        assert_eq!(lines[0], "AAA first");
        assert_eq!(lines[1], "AAA second");
        assert_eq!(lines[2], "AAA third");
    }

    #[test]
    fn sort_nothing_to_sort_zero_lines() {
        // Validates: Requirement 2.13
        let mut lines: Vec<String> = vec![];
        let params = SortParams {
            column_range: None,
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        let result = SortCommand::execute(&mut lines, &params, None);
        assert_eq!(result, Err(NavigationError::NothingToSort));
    }

    #[test]
    fn sort_nothing_to_sort_one_line() {
        // Validates: Requirement 2.13
        let mut lines = vec!["only line".to_string()];
        let params = SortParams {
            column_range: None,
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        let result = SortCommand::execute(&mut lines, &params, None);
        assert_eq!(result, Err(NavigationError::NothingToSort));
    }

    #[test]
    fn sort_with_bounds_no_explicit_columns() {
        // Validates: Requirement 2.9
        let mut lines = vec!["XXBB".to_string(), "XXAA".to_string(), "XXCC".to_string()];
        let params = SortParams {
            column_range: None,
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        let bounds = ActiveBounds::new(3, 4);
        let result = SortCommand::execute(&mut lines, &params, bounds);
        assert!(result.is_ok());
        assert_eq!(lines[0], "XXAA");
        assert_eq!(lines[1], "XXBB");
        assert_eq!(lines[2], "XXCC");
    }

    #[test]
    fn sort_with_bounds_intersection() {
        // Validates: Requirement 2.10
        let mut lines = vec![
            "AABBCC".to_string(),
            "AAAACC".to_string(),
            "AACCCC".to_string(),
        ];
        let params = SortParams {
            column_range: Some((2, 5)),
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        // Bounds are columns 3..4, intersection with explicit 2..5 = 3..4
        let bounds = ActiveBounds::new(3, 4);
        let result = SortCommand::execute(&mut lines, &params, bounds);
        assert!(result.is_ok());
        // Key is columns 3..4: "BB", "AA", "CC" → sorted: "AA", "BB", "CC"
        assert_eq!(lines[0], "AAAACC");
        assert_eq!(lines[1], "AABBCC");
        assert_eq!(lines[2], "AACCCC");
    }

    #[test]
    fn sort_undo_record_has_original_order() {
        // Validates: Requirement 2.11
        let mut lines = vec![
            "cherry".to_string(),
            "apple".to_string(),
            "banana".to_string(),
        ];
        let params = SortParams {
            column_range: None,
            direction: SortDirection::Ascending,
            scope: SortScope::AllVisible,
        };
        let record = SortCommand::execute(&mut lines, &params, None).unwrap();
        // Original order was: cherry(0), apple(1), banana(2)
        // After sort: apple(1), banana(2), cherry(0)
        assert_eq!(record.original_order, vec![1, 2, 0]);
    }

    #[test]
    fn parse_args_no_arguments() {
        let result = SortCommand::parse_args(&[]);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.column_range, None);
        assert_eq!(params.direction, SortDirection::Ascending);
        assert_eq!(params.scope, SortScope::AllVisible);
    }

    #[test]
    fn parse_args_column_range_and_direction() {
        let result = SortCommand::parse_args(&["5", "20", "D"]);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.column_range, Some((5, 20)));
        assert_eq!(params.direction, SortDirection::Descending);
    }

    #[test]
    fn parse_args_tagged_scope() {
        let result = SortCommand::parse_args(&["TAGGED"]);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.scope, SortScope::Tagged);
    }

    #[test]
    fn parse_args_invalid_argument() {
        let result = SortCommand::parse_args(&["INVALID"]);
        assert!(result.is_err());
    }
}
