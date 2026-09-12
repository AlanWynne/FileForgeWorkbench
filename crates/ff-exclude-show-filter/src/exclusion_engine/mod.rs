//! Core exclusion engine -- orchestrates all visibility operations.
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
/// Addresses: Requirements 1-10
pub struct ExclusionEngine<D: DisplayLineMapping, A: DocumentAccess> {
    /// The display-line-mapping for visibility storage.
    pub(super) display_mapping: D,
    /// Document content accessor for text matching.
    pub(super) document: A,
    /// Registered exclusion-change listeners.
    pub(super) listeners: Vec<Box<dyn ExclusionListener>>,
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

    // === Query Methods ==================================================

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

    // === Scope Iterators ================================================

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

    // === Low-Level Mutation Methods =====================================

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

    // === EXCLUDE Command Operations =====================================

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

    /// EXCLUDE 'text' -- excludes lines containing literal text.
    ///
    /// Addresses: Requirement 2 AC 1-2
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

    /// EXCLUDE REGEX 'pattern' -- excludes lines matching regex.
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

    /// EXCLUDE ALL -- excludes every line in the document.
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

    /// EXCLUDE TAGGED -- excludes lines with tagged flag.
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

    /// EXCLUDE n m -- excludes document lines in range (1-based inclusive).
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
}

mod ops;
