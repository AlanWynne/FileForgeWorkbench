//! FoldContext: helper structure for fold-level computation.

use crate::types::{FoldFlags, FoldLevel, LineNumber};

/// Helper structure for fold-level computation.
/// Provides access to line content and fold level setting during lexer fold computation.
/// Addresses: Requirement 8
pub struct FoldContext<'a> {
    /// The full document text.
    text: &'a str,
    /// Line start byte offsets.
    line_starts: &'a [usize],
    /// Start line of the range to process.
    start_line: LineNumber,
    /// End line of the range to process (exclusive).
    end_line: LineNumber,
    /// Fold levels being computed (mutable output).
    levels: Vec<(FoldLevel, FoldFlags)>,
}

impl<'a> FoldContext<'a> {
    /// Create a new FoldContext.
    pub fn new(
        text: &'a str,
        line_starts: &'a [usize],
        start_line: LineNumber,
        end_line: LineNumber,
    ) -> Self {
        let range_size = end_line.0.saturating_sub(start_line.0);
        Self {
            text,
            line_starts,
            start_line,
            end_line,
            levels: vec![(FoldLevel::MIN, FoldFlags::NONE); range_size],
        }
    }

    /// Set the fold level and flags for a line.
    /// Addresses: Requirement 8, criterion 8.1
    pub fn set_level(&mut self, line: LineNumber, level: FoldLevel, flags: FoldFlags) {
        if line.0 >= self.start_line.0 && line.0 < self.end_line.0 {
            let idx = line.0 - self.start_line.0;
            if idx < self.levels.len() {
                self.levels[idx] = (level, flags);
            }
        }
    }

    /// Get the current fold level for a line (from this computation pass).
    pub fn current_level(&self, line: LineNumber) -> FoldLevel {
        if line.0 >= self.start_line.0 && line.0 < self.end_line.0 {
            let idx = line.0 - self.start_line.0;
            self.levels
                .get(idx)
                .map(|(l, _)| *l)
                .unwrap_or(FoldLevel::MIN)
        } else {
            FoldLevel::MIN
        }
    }

    /// Get the text content of a line for analysis.
    pub fn line_text(&self, line: LineNumber) -> &str {
        if line.0 >= self.line_starts.len() {
            return "";
        }
        let start = self.line_starts[line.0];
        let end = if line.0 + 1 < self.line_starts.len() {
            self.line_starts[line.0 + 1]
        } else {
            self.text.len()
        };
        // Strip trailing newline for convenience
        let slice = &self.text[start..end];
        slice.trim_end_matches('\n').trim_end_matches('\r')
    }

    /// Get the range of lines to process.
    pub fn line_range(&self) -> (LineNumber, LineNumber) {
        (self.start_line, self.end_line)
    }

    /// Consume the context and return computed levels.
    pub fn into_levels(self) -> Vec<(FoldLevel, FoldFlags)> {
        self.levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_level() {
        // Validates: Requirement 8, criterion 8.1
        let text = "line1\nline2\nline3\n";
        let line_starts = vec![0, 6, 12];
        let mut ctx = FoldContext::new(text, &line_starts, LineNumber(0), LineNumber(3));

        ctx.set_level(LineNumber(0), FoldLevel::new(1), FoldFlags::NONE);
        ctx.set_level(LineNumber(1), FoldLevel::new(2), FoldFlags::FOLD_HEADER);

        assert_eq!(ctx.current_level(LineNumber(0)), FoldLevel::new(1));
        assert_eq!(ctx.current_level(LineNumber(1)), FoldLevel::new(2));
        assert_eq!(ctx.current_level(LineNumber(2)), FoldLevel::MIN);
    }

    #[test]
    fn line_text_returns_content() {
        let text = "hello\nworld\n";
        let line_starts = vec![0, 6];
        let ctx = FoldContext::new(text, &line_starts, LineNumber(0), LineNumber(2));

        assert_eq!(ctx.line_text(LineNumber(0)), "hello");
        assert_eq!(ctx.line_text(LineNumber(1)), "world");
    }

    #[test]
    fn line_range_returns_bounds() {
        let text = "a\nb\nc\n";
        let line_starts = vec![0, 2, 4];
        let ctx = FoldContext::new(text, &line_starts, LineNumber(1), LineNumber(3));
        assert_eq!(ctx.line_range(), (LineNumber(1), LineNumber(3)));
    }

    #[test]
    fn into_levels_returns_computed_data() {
        let text = "a\nb\n";
        let line_starts = vec![0, 2];
        let mut ctx = FoldContext::new(text, &line_starts, LineNumber(0), LineNumber(2));
        ctx.set_level(LineNumber(0), FoldLevel::new(1), FoldFlags::FOLD_HEADER);
        ctx.set_level(LineNumber(1), FoldLevel::new(0), FoldFlags::NONE);

        let levels = ctx.into_levels();
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0], (FoldLevel::new(1), FoldFlags::FOLD_HEADER));
        assert_eq!(levels[1], (FoldLevel::new(0), FoldFlags::NONE));
    }

    #[test]
    fn line_text_out_of_range_returns_empty() {
        let text = "hello\n";
        let line_starts = vec![0];
        let ctx = FoldContext::new(text, &line_starts, LineNumber(0), LineNumber(1));
        assert_eq!(ctx.line_text(LineNumber(5)), "");
    }
}
