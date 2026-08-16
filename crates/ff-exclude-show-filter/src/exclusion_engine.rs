//! Core exclusion engine — orchestrates all visibility operations.
//!
//! The `ExclusionEngine` delegates visibility storage to the
//! `DisplayLineMapping` trait and provides the public API for
//! EXCLUDE/SHOW/RESET operations.

use ff_display_line_mapping::{DisplayLineMapping, DocLine};

use crate::error::ExcludeFilterError;
use crate::text_matcher::TextMatcher;
use crate::types::*;

/// Trait for accessing document line content.
/// Implemented by adapters over `ff-document-model`'s Document type.
///
/// Addresses: Requirement 2 (text matching needs line content)
pub trait DocumentAccess: Send + Sync {
    /// Get the text content of a specific document line (0-based index).
    fn line_content(&self, line: usize) -> Option<&str>;

    /// Total number of lines in the document.
    fn line_count(&self) -> usize;

    /// Check if the line is tagged.
    fn is_tagged(&self, line: usize) -> bool;
}

/// Trait for receiving exclusion-change events.
///
/// Addresses: Requirement 7 AC 5
pub trait ExclusionListener: Send + Sync {
    /// Called when exclusion state changes.
    fn on_exclusion_changed(&self, event: &ExclusionChanged);
}

/// The top-level exclusion engine orchestrating all visibility operations.
///
/// Holds a mutable reference to a `DisplayLineMapping` and a reference to
/// document content for text matching. This is owned per-editor-session.
///
/// Addresses: Requirements 1–10
pub struct ExclusionEngine<D: DisplayLineMapping, A: DocumentAccess> {
    /// The display-line-mapping for visibility storage.
    display_mapping: D,
    /// Document content accessor for text matching.
    document: A,
    /// Registered exclusion-change listeners.
    listeners: Vec<Box<dyn ExclusionListener>>,
}

impl<D: DisplayLineMapping, A: DocumentAccess> ExclusionEngine<D, A> {
    /// Create a new ExclusionEngine with the given display mapping and document.
    pub fn new(display_mapping: D, document: A) -> Self {
        Self {
            display_mapping,
            document,
            listeners: Vec::new(),
        }
    }

    /// Register an exclusion-change listener.
    pub fn add_listener(&mut self, listener: Box<dyn ExclusionListener>) {
        self.listeners.push(listener);
    }

    /// Get a reference to the underlying display mapping.
    pub fn display_mapping(&self) -> &D {
        &self.display_mapping
    }

    /// Get a mutable reference to the underlying display mapping.
    pub fn display_mapping_mut(&mut self) -> &mut D {
        &mut self.display_mapping
    }

    /// Get a reference to the document accessor.
    pub fn document(&self) -> &A {
        &self.document
    }

    // ─── Query Methods ──────────────────────────────────────────────────

    /// Check if a specific document line is excluded.
    /// Delegates to display_line_mapping.get_visible(doc_line) == false.
    ///
    /// Addresses: Requirement 1 AC 4
    pub fn is_excluded(&self, doc_line: usize) -> bool {
        !self.display_mapping.get_visible(DocLine(doc_line))
    }

    /// Check if any lines in the document are currently excluded.
    ///
    /// Addresses: Requirement 1 AC 5
    pub fn has_excluded_lines(&self) -> bool {
        self.display_mapping.hidden_lines()
    }

    /// Return the total count of currently excluded lines.
    ///
    /// Addresses: Requirement 1 AC 7
    pub fn excluded_line_count(&self) -> usize {
        let total = self.display_mapping.lines_in_doc();
        (0..total).filter(|&line| self.is_excluded(line)).count()
    }

    /// Total number of lines in the document.
    pub fn line_count(&self) -> usize {
        self.display_mapping.lines_in_doc()
    }

    // ─── Scope Iterators ────────────────────────────────────────────────

    /// Iterate over all currently visible line indices.
    ///
    /// Addresses: Requirement 8 AC 5
    pub fn visible_lines_iter(&self) -> impl Iterator<Item = usize> + '_ {
        let total = self.display_mapping.lines_in_doc();
        (0..total).filter(|&line| !self.is_excluded(line))
    }

    /// Iterate over all currently excluded line indices.
    ///
    /// Addresses: Requirement 8 AC 6
    pub fn excluded_lines_iter(&self) -> impl Iterator<Item = usize> + '_ {
        let total = self.display_mapping.lines_in_doc();
        (0..total).filter(|&line| self.is_excluded(line))
    }

    // ─── Low-Level Mutation Methods ─────────────────────────────────────

    /// Exclude a single line by index.
    ///
    /// Addresses: Requirement 1 AC 2
    pub fn exclude_line(&mut self, doc_line: usize) -> bool {
        self.display_mapping
            .set_visible(DocLine(doc_line), DocLine(doc_line), false)
    }

    /// Exclude a contiguous range of lines (inclusive, 0-based).
    ///
    /// Addresses: Requirement 1 AC 8
    pub fn exclude_range(&mut self, start: usize, end: usize) -> usize {
        if start > end {
            return 0;
        }
        let changed = self
            .display_mapping
            .set_visible(DocLine(start), DocLine(end), false);
        if changed {
            end - start + 1
        } else {
            0
        }
    }

    /// Show (un-exclude) a single line by index.
    ///
    /// Addresses: Requirement 1 AC 3
    pub fn show_line(&mut self, doc_line: usize) -> bool {
        self.display_mapping
            .set_visible(DocLine(doc_line), DocLine(doc_line), true)
    }

    /// Show a contiguous range of lines (inclusive, 0-based).
    pub fn show_range(&mut self, start: usize, end: usize) -> usize {
        if start > end {
            return 0;
        }
        let changed = self
            .display_mapping
            .set_visible(DocLine(start), DocLine(end), true);
        if changed {
            end - start + 1
        } else {
            0
        }
    }

    /// Show all lines (reset to fully visible).
    ///
    /// Addresses: Requirement 4 AC 4
    pub fn show_all(&mut self) {
        self.display_mapping.show_all();
    }

    // ─── EXCLUDE Command Operations ─────────────────────────────────────

    /// Execute an EXCLUDE command with the given arguments.
    ///
    /// Addresses: Requirement 2
    pub fn execute_exclude(
        &mut self,
        args: &ExcludeArgs,
    ) -> Result<ExcludeResult, ExcludeFilterError> {
        match args {
            ExcludeArgs::Text { pattern, scope } => self.exclude_text(pattern, *scope),
            ExcludeArgs::Regex { pattern, scope } => self.exclude_regex(pattern, *scope),
            ExcludeArgs::All => Ok(self.exclude_all()),
            ExcludeArgs::Tagged => Ok(self.exclude_tagged()),
            ExcludeArgs::Range {
                start_line,
                end_line,
            } => self.exclude_range_by_number(*start_line, *end_line),
        }
    }

    /// EXCLUDE 'text' — excludes lines containing literal text.
    ///
    /// Addresses: Requirement 2 AC 1–2
    fn exclude_text(
        &mut self,
        pattern: &str,
        scope: ExcludeScope,
    ) -> Result<ExcludeResult, ExcludeFilterError> {
        let matcher = TextMatcher::literal(pattern);
        let total = self.display_mapping.lines_in_doc();
        let mut count = 0usize;

        for line in 0..total {
            let in_scope = match scope {
                ExcludeScope::Visible => !self.is_excluded(line),
                ExcludeScope::All => true,
            };
            if !in_scope {
                continue;
            }
            if let Some(content) = self.document.line_content(line) {
                if matcher.matches_line(content) && self.exclude_line(line) {
                    count += 1;
                }
            }
        }
        Ok(ExcludeResult::new(count))
    }

    /// EXCLUDE REGEX 'pattern' — excludes lines matching regex.
    ///
    /// Addresses: Requirement 2 AC 3
    fn exclude_regex(
        &mut self,
        pattern: &str,
        scope: ExcludeScope,
    ) -> Result<ExcludeResult, ExcludeFilterError> {
        let matcher = TextMatcher::regex(pattern, "exclude")?;
        let total = self.display_mapping.lines_in_doc();
        let mut count = 0usize;

        for line in 0..total {
            let in_scope = match scope {
                ExcludeScope::Visible => !self.is_excluded(line),
                ExcludeScope::All => true,
            };
            if !in_scope {
                continue;
            }
            if let Some(content) = self.document.line_content(line) {
                if matcher.matches_line(content) && self.exclude_line(line) {
                    count += 1;
                }
            }
        }
        Ok(ExcludeResult::new(count))
    }

    /// EXCLUDE ALL — excludes every line in the document.
    ///
    /// Addresses: Requirement 2 AC 4
    fn exclude_all(&mut self) -> ExcludeResult {
        let total = self.display_mapping.lines_in_doc();
        if total == 0 {
            return ExcludeResult::new(0);
        }
        let last = total - 1;
        self.display_mapping
            .set_visible(DocLine(0), DocLine(last), false);
        ExcludeResult::new(total)
    }

    /// EXCLUDE TAGGED — excludes lines with tagged flag.
    ///
    /// Addresses: Requirement 2 AC 5
    fn exclude_tagged(&mut self) -> ExcludeResult {
        let total = self.display_mapping.lines_in_doc();
        let mut count = 0usize;
        for line in 0..total {
            if self.document.is_tagged(line) && self.exclude_line(line) {
                count += 1;
            }
        }
        ExcludeResult::new(count)
    }

    /// EXCLUDE n m — excludes document lines in range (1-based inclusive).
    ///
    /// Addresses: Requirement 2 AC 6
    fn exclude_range_by_number(
        &mut self,
        start_line: usize,
        end_line: usize,
    ) -> Result<ExcludeResult, ExcludeFilterError> {
        let total = self.display_mapping.lines_in_doc();
        if start_line == 0 || end_line == 0 {
            return Err(ExcludeFilterError::InvalidRange {
                start: start_line,
                end: end_line,
                total,
            });
        }
        if start_line > end_line {
            return Err(ExcludeFilterError::InvalidRange {
                start: start_line,
                end: end_line,
                total,
            });
        }
        if end_line > total {
            return Err(ExcludeFilterError::InvalidRange {
                start: start_line,
                end: end_line,
                total,
            });
        }
        // Convert 1-based to 0-based
        let start = start_line - 1;
        let end = end_line - 1;
        let count = self.exclude_range(start, end);
        Ok(ExcludeResult::new(count))
    }

    // ─── SHOW Command Operations ────────────────────────────────────────

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

    /// SHOW ALL — clears excluded flag on every line.
    ///
    /// Addresses: Requirement 3 AC 1
    fn show_all_lines(&mut self) -> ShowResult {
        let count = self.excluded_line_count();
        self.display_mapping.show_all();
        ShowResult::new(count)
    }

    /// SHOW EXCLUDED — clears excluded flag on all excluded lines.
    ///
    /// Addresses: Requirement 3 AC 2
    fn show_excluded(&mut self) -> ShowResult {
        let count = self.excluded_line_count();
        self.display_mapping.show_all();
        ShowResult::new(count)
    }

    /// SHOW 'text' — show excluded lines containing literal text.
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

    /// SHOW REGEX 'pattern' — show excluded lines matching regex.
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

    // ─── RESET Command Operations ───────────────────────────────────────

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

    // ─── Line Command Operations ────────────────────────────────────────

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

    // ─── Placeholder / Block Model ──────────────────────────────────────

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

    // ─── Notifications ──────────────────────────────────────────────────

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
