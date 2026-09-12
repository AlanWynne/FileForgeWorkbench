use ff_display_line_mapping::DisplayLineMapping;

use crate::error::ExcludeFilterError;
use crate::text_matcher::TextMatcher;
use crate::types::*;

use super::{DocumentAccess, ExclusionEngine};

impl<D: DisplayLineMapping, A: DocumentAccess> ExclusionEngine<D, A> {
    // === SHOW Command Operations ========================================

    /// Execute a SHOW/INCLUDE command with the given arguments.
    ///
    /// Addresses: Requirement 3
    pub fn execute_show(&mut self, args: &ShowArgs) -> Result<ShowResult, ExcludeFilterError> {
        match args {
            ShowArgs::All => Ok(self.show_all_lines()),
            ShowArgs::Excluded => Ok(self.show_excluded()),
            ShowArgs::NonExcluded => Ok(ShowResult::non_excluded_noop()),
            ShowArgs::Text { pattern } => self.show_text(pattern),
            ShowArgs::Regex { pattern } => self.show_regex(pattern),
        }
    }

    /// SHOW ALL -- clears excluded flag on every line.
    ///
    /// Addresses: Requirement 3 AC 1
    fn show_all_lines(&mut self) -> ShowResult {
        let count = self.excluded_line_count();
        self.display_mapping.show_all();
        ShowResult::new(count)
    }

    /// SHOW EXCLUDED -- clears excluded flag on all excluded lines.
    ///
    /// Addresses: Requirement 3 AC 2
    fn show_excluded(&mut self) -> ShowResult {
        let count = self.excluded_line_count();
        self.display_mapping.show_all();
        ShowResult::new(count)
    }

    /// SHOW 'text' -- show excluded lines containing literal text.
    ///
    /// Addresses: Requirement 3 AC 4
    fn show_text(&mut self, pattern: &str) -> Result<ShowResult, ExcludeFilterError> {
        let matcher = TextMatcher::literal(pattern);
        let total = self.display_mapping.lines_in_doc();
        let mut count = 0usize;

        for line in 0..total {
            if !self.is_excluded(line) {
                continue;
            }
            if let Some(content) = self.document.line_content(line) {
                if matcher.matches_line(content) && self.show_line(line) {
                    count += 1;
                }
            }
        }
        Ok(ShowResult::new(count))
    }

    /// SHOW REGEX 'pattern' -- show excluded lines matching regex.
    ///
    /// Addresses: Requirement 3 AC 5
    fn show_regex(&mut self, pattern: &str) -> Result<ShowResult, ExcludeFilterError> {
        let matcher = TextMatcher::regex(pattern, "show")?;
        let total = self.display_mapping.lines_in_doc();
        let mut count = 0usize;

        for line in 0..total {
            if !self.is_excluded(line) {
                continue;
            }
            if let Some(content) = self.document.line_content(line) {
                if matcher.matches_line(content) && self.show_line(line) {
                    count += 1;
                }
            }
        }
        Ok(ShowResult::new(count))
    }

    // === RESET Command Operations =======================================

    /// Execute a RESET command variant.
    ///
    /// Addresses: Requirement 4
    pub fn execute_reset(&mut self, variant: ResetVariant) -> ResetResult {
        match variant {
            ResetVariant::Default | ResetVariant::Excluded | ResetVariant::All => {
                let count = self.excluded_line_count();
                self.display_mapping.show_all();
                self.notify_change(count, 0, count);
                ResetResult::new(count)
            }
        }
    }

    // === Line Command Operations ========================================

    /// Process a resolved X/Xn/XX line command.
    ///
    /// Addresses: Requirement 5
    pub fn execute_line_command(
        &mut self,
        command: &LineCommandExclude,
    ) -> Result<ExcludeResult, ExcludeFilterError> {
        let total = self.display_mapping.lines_in_doc();
        match *command {
            LineCommandExclude::Single { line } => {
                if line >= total {
                    return Err(ExcludeFilterError::LineOutOfRange {
                        operation: "X".to_string(),
                        line,
                        total,
                    });
                }
                self.exclude_line(line);
                Ok(ExcludeResult::new(1))
            }
            LineCommandExclude::Count { line, count } => {
                let end = line + count - 1;
                if end >= total {
                    return Err(ExcludeFilterError::LineOutOfRange {
                        operation: "Xn".to_string(),
                        line: end,
                        total,
                    });
                }
                let affected = self.exclude_range(line, end);
                Ok(ExcludeResult::new(affected))
            }
            LineCommandExclude::Block { start, end } => {
                if end >= total {
                    return Err(ExcludeFilterError::LineOutOfRange {
                        operation: "XX".to_string(),
                        line: end,
                        total,
                    });
                }
                let affected = self.exclude_range(start, end);
                Ok(ExcludeResult::new(affected))
            }
        }
    }

    // === Placeholder / Block Model ======================================

    /// Enumerate all contiguous exclusion blocks in the document.
    ///
    /// Addresses: Requirement 6 AC 1
    pub fn exclusion_blocks(&self) -> Vec<ExclusionBlock> {
        let total = self.display_mapping.lines_in_doc();
        let mut blocks = Vec::new();
        let mut i = 0;

        while i < total {
            if self.is_excluded(i) {
                let start = i;
                while i < total && self.is_excluded(i) {
                    i += 1;
                }
                blocks.push(ExclusionBlock::new(start, i - 1));
            } else {
                i += 1;
            }
        }
        blocks
    }

    /// Number of exclusion blocks currently in the document.
    ///
    /// Addresses: Requirement 6 AC 6
    pub fn block_count(&self) -> usize {
        self.exclusion_blocks().len()
    }

    /// Get the exclusion block containing a specific document line.
    /// Returns None if the line is not excluded.
    ///
    /// Addresses: Requirement 6 AC 7
    pub fn block_at_doc_line(&self, doc_line: usize) -> Option<ExclusionBlock> {
        if !self.is_excluded(doc_line) {
            return None;
        }
        let total = self.display_mapping.lines_in_doc();
        // Walk backwards to find start
        let mut start = doc_line;
        while start > 0 && self.is_excluded(start - 1) {
            start -= 1;
        }
        // Walk forwards to find end
        let mut end = doc_line;
        while end + 1 < total && self.is_excluded(end + 1) {
            end += 1;
        }
        Some(ExclusionBlock::new(start, end))
    }

    // === Notifications ==================================================

    /// Emit an exclusion-changed notification to all listeners.
    fn notify_change(&self, total_excluded: usize, block_count: usize, lines_changed: usize) {
        let event = ExclusionChanged {
            total_excluded,
            block_count,
            lines_changed,
        };
        for listener in &self.listeners {
            listener.on_exclusion_changed(&event);
        }
    }
}
