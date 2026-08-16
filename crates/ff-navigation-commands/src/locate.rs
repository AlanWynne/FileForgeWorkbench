//! LOCATE command implementation.
//!
//! Jumps the viewport and cursor to a specific line number or named label.

use crate::error::NavigationError;

use ff_viewport_scrolling::{CursorModel, ViewportModel};

/// A registry of named labels mapped to document line numbers.
pub trait LabelRegistry {
    /// Resolve a label name to a line number (1-based).
    fn resolve_label(&self, name: &str) -> Option<u64>;
}

/// LOCATE command executor.
///
/// Provides methods to jump to a specific line number or named label,
/// updating both the viewport and cursor position.
pub struct LocateCommand;

impl LocateCommand {
    /// Execute LOCATE with a line number target.
    ///
    /// Scrolls viewport so target line is top, updates cursor_line to target,
    /// and resets cursor_column to 1.
    ///
    /// # Errors
    ///
    /// Returns `NavigationError::LineOutOfRange` if `target_line` is less than 1
    /// or greater than `doc_line_count`.
    pub fn locate_line(
        viewport: &mut ViewportModel,
        cursor: &mut CursorModel,
        target_line: u64,
        doc_line_count: u64,
    ) -> Result<(), NavigationError> {
        if target_line < 1 || target_line > doc_line_count {
            return Err(NavigationError::LineOutOfRange);
        }

        viewport.scroll_to_line(target_line, cursor);
        cursor.set_position(target_line, 1);
        Ok(())
    }

    /// Execute LOCATE with a label target.
    ///
    /// Resolves the label to a line number, then navigates as with `locate_line`.
    ///
    /// # Errors
    ///
    /// Returns `NavigationError::LabelNotFound` if the label does not exist.
    /// Returns `NavigationError::LineOutOfRange` if the resolved line is invalid.
    pub fn locate_label(
        viewport: &mut ViewportModel,
        cursor: &mut CursorModel,
        label: &str,
        label_registry: &dyn LabelRegistry,
        doc_line_count: u64,
    ) -> Result<(), NavigationError> {
        let target_line =
            label_registry
                .resolve_label(label)
                .ok_or_else(|| NavigationError::LabelNotFound {
                    label: label.to_string(),
                })?;

        Self::locate_line(viewport, cursor, target_line, doc_line_count)
    }

    /// Parse a LOCATE argument string into either a line number or label.
    ///
    /// Returns `Ok(target_line)` for numeric arguments, `Err(label_string)` for
    /// non-numeric arguments.
    pub fn parse_argument(arg: &str) -> Result<u64, &str> {
        arg.trim().parse::<u64>().map_err(|_| arg.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestLabels {
        labels: Vec<(&'static str, u64)>,
    }

    impl LabelRegistry for TestLabels {
        fn resolve_label(&self, name: &str) -> Option<u64> {
            self.labels
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, line)| *line)
        }
    }

    #[test]
    fn locate_line_sets_viewport_and_cursor() {
        // Validates: Requirement 1.1, 1.6
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        cursor.set_position(5, 10);

        let result = LocateCommand::locate_line(&mut viewport, &mut cursor, 50, 100);
        assert!(result.is_ok());
        assert_eq!(cursor.cursor_line(), 50);
        assert_eq!(cursor.cursor_column(), 1);
    }

    #[test]
    fn locate_line_out_of_range_too_high() {
        // Validates: Requirement 1.2
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();

        let result = LocateCommand::locate_line(&mut viewport, &mut cursor, 101, 100);
        assert_eq!(result, Err(NavigationError::LineOutOfRange));
        // Viewport and cursor unchanged
        assert_eq!(cursor.cursor_line(), 1);
    }

    #[test]
    fn locate_line_out_of_range_zero() {
        // Validates: Requirement 1.2
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();

        let result = LocateCommand::locate_line(&mut viewport, &mut cursor, 0, 100);
        assert_eq!(result, Err(NavigationError::LineOutOfRange));
    }

    #[test]
    fn locate_label_found() {
        // Validates: Requirement 1.3, 1.6
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        let labels = TestLabels {
            labels: vec![("TOP_SECTION", 25)],
        };

        let result =
            LocateCommand::locate_label(&mut viewport, &mut cursor, "TOP_SECTION", &labels, 100);
        assert!(result.is_ok());
        assert_eq!(cursor.cursor_line(), 25);
        assert_eq!(cursor.cursor_column(), 1);
    }

    #[test]
    fn locate_label_not_found() {
        // Validates: Requirement 1.4
        let mut viewport = ViewportModel::with_line_count(100);
        viewport.set_visible_count(20);
        let mut cursor = CursorModel::new();
        let labels = TestLabels { labels: vec![] };

        let result =
            LocateCommand::locate_label(&mut viewport, &mut cursor, "MISSING", &labels, 100);
        assert_eq!(
            result,
            Err(NavigationError::LabelNotFound {
                label: "MISSING".to_string()
            })
        );
        assert_eq!(cursor.cursor_line(), 1);
    }

    #[test]
    fn parse_argument_numeric() {
        assert_eq!(LocateCommand::parse_argument("42"), Ok(42));
        assert_eq!(LocateCommand::parse_argument(" 100 "), Ok(100));
    }

    #[test]
    fn parse_argument_label() {
        assert_eq!(LocateCommand::parse_argument("MY_LABEL"), Err("MY_LABEL"));
        assert_eq!(LocateCommand::parse_argument("top"), Err("top"));
    }
}
