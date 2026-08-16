//! Wrap indent mode for continuation lines.
//!
//! Controls how continuation sub-lines are indented relative to the
//! first sub-line of the wrapped document line.

/// Wrap indent mode for continuation lines.
///
/// Controls how continuation sub-lines are indented relative to the
/// first sub-line of the wrapped document line.
///
/// Addresses: Requirement 5 (Wrap Indent for Continuation Lines)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum WrapIndentMode {
    /// Indent by a fixed number of characters from the left margin.
    /// Amount is defined by `wrap_indent_amount` config value.
    #[default]
    Fixed,

    /// Align with the first non-whitespace character of the first sub-line
    /// (matching the source line's indentation level).
    Same,

    /// Same as `Same` plus one additional indent level.
    Indent,

    /// Same as `Same` plus two additional indent levels.
    DeepIndent,
}

impl WrapIndentMode {
    /// Compute the continuation line indent offset in characters.
    ///
    /// # Arguments
    ///
    /// - `indent_amount` — The fixed indent amount (used only in `Fixed` mode).
    /// - `first_non_ws_col` — Column of the first non-whitespace character on the line.
    /// - `indent_width` — The width of one indent level (e.g., 4 spaces).
    ///
    /// Addresses: Requirement 5 AC 2–5
    pub fn compute_indent(
        self,
        indent_amount: u8,
        first_non_ws_col: usize,
        indent_width: u8,
    ) -> usize {
        match self {
            Self::Fixed => indent_amount as usize,
            Self::Same => first_non_ws_col,
            Self::Indent => first_non_ws_col + indent_width as usize,
            Self::DeepIndent => first_non_ws_col + (indent_width as usize * 2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_indent_mode_is_fixed() {
        // Validates: Requirement 5.6
        assert_eq!(WrapIndentMode::default(), WrapIndentMode::Fixed);
    }

    #[test]
    fn fixed_mode_uses_indent_amount() {
        // Validates: Requirement 5.2
        assert_eq!(WrapIndentMode::Fixed.compute_indent(4, 8, 4), 4);
        assert_eq!(WrapIndentMode::Fixed.compute_indent(0, 8, 4), 0);
    }

    #[test]
    fn same_mode_uses_first_non_ws_col() {
        // Validates: Requirement 5.3
        assert_eq!(WrapIndentMode::Same.compute_indent(0, 8, 4), 8);
        assert_eq!(WrapIndentMode::Same.compute_indent(0, 0, 4), 0);
    }

    #[test]
    fn indent_mode_adds_one_level() {
        // Validates: Requirement 5.4
        assert_eq!(WrapIndentMode::Indent.compute_indent(0, 8, 4), 12);
    }

    #[test]
    fn deep_indent_mode_adds_two_levels() {
        // Validates: Requirement 5.5
        assert_eq!(WrapIndentMode::DeepIndent.compute_indent(0, 8, 4), 16);
    }
}
