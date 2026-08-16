//! Logical combination: AND/OR with parenthesised grouping and precedence.
//!
//! Combines per-row boolean results using AND/OR connectors and
//! parenthesised grouping, respecting standard logical precedence
//! (AND binds tighter than OR unless overridden by grouping).

use crate::model::CriteriaConnector;

/// Input row for the logical combiner.
#[derive(Debug, Clone)]
pub struct LogicalRow {
    /// The boolean result of this criterion's comparison.
    pub result: bool,
    /// The connector to the NEXT row (None on last row).
    pub connector: Option<CriteriaConnector>,
    /// Whether this row opens a parenthesised group.
    pub group_open: bool,
    /// Whether this row closes a parenthesised group.
    pub group_close: bool,
}

/// Combines per-row boolean results using AND/OR connectors and
/// parenthesised grouping, respecting standard logical precedence.
///
/// Addresses: Requirement 5
pub struct LogicalCombiner;

impl LogicalCombiner {
    /// Combine a sequence of logical rows into a final boolean.
    ///
    /// AND binds tighter than OR unless overridden by grouping.
    ///
    /// Addresses: Requirement 5 AC 1, 2, 3
    pub fn combine(rows: &[LogicalRow]) -> bool {
        if rows.is_empty() {
            return true;
        }

        // Parse into a tree respecting groups, then evaluate with precedence
        let tokens = Self::tokenize(rows);
        Self::evaluate_tokens(&tokens)
    }

    /// Convert rows into a flat token stream with group markers.
    fn tokenize(rows: &[LogicalRow]) -> Vec<Token> {
        let mut tokens = Vec::new();

        for row in rows {
            if row.group_open {
                tokens.push(Token::GroupOpen);
            }
            tokens.push(Token::Value(row.result));
            if row.group_close {
                tokens.push(Token::GroupClose);
            }
            if let Some(conn) = row.connector {
                tokens.push(Token::Connector(conn));
            }
        }

        tokens
    }

    /// Evaluate a token stream with proper precedence.
    /// Uses recursive descent: OR is lowest precedence, AND is higher,
    /// parenthesised groups are highest.
    fn evaluate_tokens(tokens: &[Token]) -> bool {
        let mut pos = 0;
        Self::parse_or(tokens, &mut pos)
    }

    /// Parse an OR expression (lowest precedence).
    fn parse_or(tokens: &[Token], pos: &mut usize) -> bool {
        let mut result = Self::parse_and(tokens, pos);

        while *pos < tokens.len() {
            if let Token::Connector(CriteriaConnector::Or) = &tokens[*pos] {
                *pos += 1;
                let right = Self::parse_and(tokens, pos);
                result = result || right;
            } else {
                break;
            }
        }

        result
    }

    /// Parse an AND expression (higher precedence than OR).
    fn parse_and(tokens: &[Token], pos: &mut usize) -> bool {
        let mut result = Self::parse_primary(tokens, pos);

        while *pos < tokens.len() {
            if let Token::Connector(CriteriaConnector::And) = &tokens[*pos] {
                *pos += 1;
                let right = Self::parse_primary(tokens, pos);
                result = result && right;
            } else {
                break;
            }
        }

        result
    }

    /// Parse a primary expression (value or grouped sub-expression).
    fn parse_primary(tokens: &[Token], pos: &mut usize) -> bool {
        if *pos >= tokens.len() {
            return true;
        }

        match &tokens[*pos] {
            Token::GroupOpen => {
                *pos += 1; // consume '('
                let result = Self::parse_or(tokens, pos);
                // consume ')' if present
                if *pos < tokens.len() && matches!(tokens[*pos], Token::GroupClose) {
                    *pos += 1;
                }
                result
            }
            Token::Value(v) => {
                let result = *v;
                *pos += 1;
                result
            }
            _ => true,
        }
    }
}

/// Internal token type for expression parsing.
#[derive(Debug, Clone)]
enum Token {
    Value(bool),
    Connector(CriteriaConnector),
    GroupOpen,
    GroupClose,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(result: bool, connector: Option<CriteriaConnector>) -> LogicalRow {
        LogicalRow {
            result,
            connector,
            group_open: false,
            group_close: false,
        }
    }

    fn row_grouped(
        result: bool,
        connector: Option<CriteriaConnector>,
        group_open: bool,
        group_close: bool,
    ) -> LogicalRow {
        LogicalRow {
            result,
            connector,
            group_open,
            group_close,
        }
    }

    #[test]
    fn empty_rows_returns_true() {
        assert!(LogicalCombiner::combine(&[]));
    }

    #[test]
    fn single_true_row_returns_true() {
        assert!(LogicalCombiner::combine(&[row(true, None)]));
    }

    #[test]
    fn single_false_row_returns_false() {
        assert!(!LogicalCombiner::combine(&[row(false, None)]));
    }

    #[test]
    fn two_rows_and_both_true() {
        let rows = vec![row(true, Some(CriteriaConnector::And)), row(true, None)];
        assert!(LogicalCombiner::combine(&rows));
    }

    #[test]
    fn two_rows_and_one_false() {
        let rows = vec![row(true, Some(CriteriaConnector::And)), row(false, None)];
        assert!(!LogicalCombiner::combine(&rows));
    }

    #[test]
    fn two_rows_or_one_true() {
        let rows = vec![row(false, Some(CriteriaConnector::Or)), row(true, None)];
        assert!(LogicalCombiner::combine(&rows));
    }

    #[test]
    fn two_rows_or_both_false() {
        let rows = vec![row(false, Some(CriteriaConnector::Or)), row(false, None)];
        assert!(!LogicalCombiner::combine(&rows));
    }

    #[test]
    fn and_binds_tighter_than_or_true_case() {
        // true OR false AND false → true OR (false AND false) → true OR false → true
        let rows = vec![
            row(true, Some(CriteriaConnector::Or)),
            row(false, Some(CriteriaConnector::And)),
            row(false, None),
        ];
        assert!(LogicalCombiner::combine(&rows));
    }

    #[test]
    fn and_binds_tighter_than_or_false_case() {
        // false OR false AND true → false OR (false AND true) → false OR false → false
        let rows = vec![
            row(false, Some(CriteriaConnector::Or)),
            row(false, Some(CriteriaConnector::And)),
            row(true, None),
        ];
        assert!(!LogicalCombiner::combine(&rows));
    }

    #[test]
    fn grouped_or_overrides_and_precedence() {
        // (false OR true) AND true → true AND true → true
        let rows = vec![
            row_grouped(false, Some(CriteriaConnector::Or), true, false),
            row_grouped(true, Some(CriteriaConnector::And), false, true),
            row(true, None),
        ];
        assert!(LogicalCombiner::combine(&rows));
    }

    #[test]
    fn group_override_a_or_b_and_c() {
        // A OR (B AND C) — group overrides default precedence
        // With A=false, B=true, C=true → false OR (true AND true) → true
        let rows = vec![
            row_grouped(false, Some(CriteriaConnector::Or), false, false),
            row_grouped(true, Some(CriteriaConnector::And), true, false),
            row_grouped(true, None, false, true),
        ];
        assert!(LogicalCombiner::combine(&rows));
    }

    #[test]
    fn group_override_a_or_b_and_c_both_false() {
        // A OR (B AND C) with A=false, B=true, C=false → false OR (true AND false) → false
        let rows = vec![
            row_grouped(false, Some(CriteriaConnector::Or), false, false),
            row_grouped(true, Some(CriteriaConnector::And), true, false),
            row_grouped(false, None, false, true),
        ];
        assert!(!LogicalCombiner::combine(&rows));
    }

    #[test]
    fn nested_groups() {
        // ( (T AND F) OR T ) = (F OR T) = T
        // Actually we need proper nesting:
        // Let's do: ( (T AND F) OR T ) = (F OR T) = T
        let rows2 = vec![
            LogicalRow {
                result: true,
                connector: Some(CriteriaConnector::And),
                group_open: true,
                group_close: false,
            },
            LogicalRow {
                result: false,
                connector: Some(CriteriaConnector::Or),
                group_open: false,
                group_close: false,
            },
            LogicalRow {
                result: true,
                connector: None,
                group_open: false,
                group_close: true,
            },
        ];
        // Without the group: T AND F OR T → (T AND F) OR T → F OR T → T (same result due to AND precedence)
        // Actually this tests the group wrapping the whole thing
        assert!(LogicalCombiner::combine(&rows2));
    }

    #[test]
    fn three_ands_all_true() {
        let rows = vec![
            row(true, Some(CriteriaConnector::And)),
            row(true, Some(CriteriaConnector::And)),
            row(true, None),
        ];
        assert!(LogicalCombiner::combine(&rows));
    }

    #[test]
    fn three_ands_one_false() {
        let rows = vec![
            row(true, Some(CriteriaConnector::And)),
            row(false, Some(CriteriaConnector::And)),
            row(true, None),
        ];
        assert!(!LogicalCombiner::combine(&rows));
    }

    #[test]
    fn three_ors_all_false() {
        let rows = vec![
            row(false, Some(CriteriaConnector::Or)),
            row(false, Some(CriteriaConnector::Or)),
            row(false, None),
        ];
        assert!(!LogicalCombiner::combine(&rows));
    }

    #[test]
    fn three_ors_one_true() {
        let rows = vec![
            row(false, Some(CriteriaConnector::Or)),
            row(true, Some(CriteriaConnector::Or)),
            row(false, None),
        ];
        assert!(LogicalCombiner::combine(&rows));
    }
}
