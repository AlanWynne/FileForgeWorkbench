//! Scope resolution — priority-ordered algorithm for determining target lines/columns.
//!
//! The scope resolver determines which lines (and optionally which column range)
//! a command targets, using a defined priority order from explicit ranges down to
//! entire-document fallback.

/// A filter applied to scope lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeFilter {
    /// Include only visible (non-excluded) lines.
    Visible,
    /// Include only excluded (hidden) lines.
    Excluded,
    /// Include all lines regardless of visibility.
    All,
    /// Include only tagged lines.
    Tagged,
    /// Include only non-tagged lines.
    NonTagged,
}

/// How the scope was determined (for diagnostics and priority tracking).
///
/// Variants are ordered by priority (highest first). The `Ord` implementation
/// reflects this ordering for comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeSource {
    /// Priority 1: Explicit line range in command arguments.
    ExplicitRange = 1,
    /// Priority 2: Block line command pair (CC/CC, MM/MM, etc.).
    BlockSource = 2,
    /// Priority 3: Single line command.
    SingleLineCommand = 3,
    /// Priority 4: TAGGED/NONTAGGED modifier.
    TaggedModifier = 4,
    /// Priority 5: VISIBLE/EXCLUDED/ALL modifier.
    VisibilityModifier = 5,
    /// Priority 6: Cursor line.
    CursorLine = 6,
    /// Priority 7: Entire document (default for commands that allow it).
    EntireDocument = 7,
}

/// Which lines are included in the scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeLines {
    /// A specific contiguous range of lines (inclusive, 0-based).
    Range { start: u64, end: u64 },
    /// The cursor line only.
    CursorLine(u64),
    /// The entire document.
    EntireDocument,
    /// Lines matching a visibility/tag filter.
    Filtered {
        /// Base range to filter within.
        base: Box<ScopeLines>,
        /// The filter to apply.
        filter: ScopeFilter,
    },
}

/// Column boundaries for column-sensitive operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnBounds {
    /// Left bound (1-based column number, inclusive).
    pub left: u32,
    /// Right bound (1-based column number, inclusive).
    pub right: u32,
}

/// The resolved target scope for a command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScope {
    /// The lines targeted by this scope.
    pub lines: ScopeLines,
    /// Optional column bounds restriction.
    pub column_bounds: Option<ColumnBounds>,
    /// The source that determined this scope (for diagnostics).
    pub source: ScopeSource,
}

/// Input representing a potential scope source for the resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeCandidate {
    /// The source type (determines priority).
    pub source: ScopeSource,
    /// The lines this candidate would resolve to.
    pub lines: ScopeLines,
}

/// Resolves the target scope for a command using the priority algorithm.
pub struct ScopeResolver;

impl ScopeResolver {
    /// Resolve scope from a set of candidates using priority ordering.
    ///
    /// The candidate with the highest priority (lowest `ScopeSource` ordinal) wins.
    /// If no candidates are provided and `allows_whole_document` is true, returns
    /// `EntireDocument` scope. Otherwise returns an error.
    ///
    /// # Errors
    ///
    /// Returns `ScopeError::NoScope` if no candidates resolve and the command
    /// does not allow whole-document scope.
    pub fn resolve(
        candidates: &[ScopeCandidate],
        column_bounds: Option<ColumnBounds>,
        allows_whole_document: bool,
    ) -> Result<ResolvedScope, crate::error::ScopeError> {
        if candidates.is_empty() {
            if allows_whole_document {
                return Ok(ResolvedScope {
                    lines: ScopeLines::EntireDocument,
                    column_bounds,
                    source: ScopeSource::EntireDocument,
                });
            }
            return Err(crate::error::ScopeError::NoScope);
        }

        // Find highest-priority candidate (lowest ordinal value)
        let best = candidates
            .iter()
            .min_by_key(|c| c.source)
            .expect("candidates is non-empty");

        Ok(ResolvedScope {
            lines: best.lines.clone(),
            column_bounds,
            source: best.source,
        })
    }

    /// Apply a visibility or tag filter to an existing scope.
    pub fn apply_filter(scope: ResolvedScope, filter: ScopeFilter) -> ResolvedScope {
        ResolvedScope {
            lines: ScopeLines::Filtered {
                base: Box::new(scope.lines),
                filter,
            },
            column_bounds: scope.column_bounds,
            source: scope.source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.1
    #[test]
    fn resolve_selects_highest_priority_candidate() {
        let candidates = vec![
            ScopeCandidate {
                source: ScopeSource::CursorLine,
                lines: ScopeLines::CursorLine(5),
            },
            ScopeCandidate {
                source: ScopeSource::ExplicitRange,
                lines: ScopeLines::Range { start: 0, end: 10 },
            },
            ScopeCandidate {
                source: ScopeSource::BlockSource,
                lines: ScopeLines::Range { start: 2, end: 8 },
            },
        ];

        let result = ScopeResolver::resolve(&candidates, None, false).unwrap();
        assert_eq!(result.source, ScopeSource::ExplicitRange);
        assert_eq!(result.lines, ScopeLines::Range { start: 0, end: 10 });
    }

    // Validates: Requirement 2.9
    #[test]
    fn resolve_ignores_lower_priority_without_error() {
        let candidates = vec![
            ScopeCandidate {
                source: ScopeSource::BlockSource,
                lines: ScopeLines::Range { start: 2, end: 8 },
            },
            ScopeCandidate {
                source: ScopeSource::VisibilityModifier,
                lines: ScopeLines::EntireDocument,
            },
        ];

        let result = ScopeResolver::resolve(&candidates, None, false).unwrap();
        assert_eq!(result.source, ScopeSource::BlockSource);
    }

    // Validates: Requirement 2.8
    #[test]
    fn resolve_no_candidates_without_whole_document_returns_error() {
        let result = ScopeResolver::resolve(&[], None, false);
        assert!(result.is_err());
    }

    // Validates: Requirement 2.1
    #[test]
    fn resolve_no_candidates_with_whole_document_returns_entire_document() {
        let result = ScopeResolver::resolve(&[], None, true).unwrap();
        assert_eq!(result.source, ScopeSource::EntireDocument);
        assert_eq!(result.lines, ScopeLines::EntireDocument);
    }

    // Validates: Requirement 2.7
    #[test]
    fn resolve_passes_through_column_bounds() {
        let candidates = vec![ScopeCandidate {
            source: ScopeSource::CursorLine,
            lines: ScopeLines::CursorLine(3),
        }];
        let bounds = Some(ColumnBounds { left: 5, right: 40 });

        let result = ScopeResolver::resolve(&candidates, bounds, false).unwrap();
        assert_eq!(
            result.column_bounds,
            Some(ColumnBounds { left: 5, right: 40 })
        );
    }

    // Validates: Requirement 2.1
    #[test]
    fn resolve_priority_is_independent_of_presentation_order() {
        let candidates_a = vec![
            ScopeCandidate {
                source: ScopeSource::CursorLine,
                lines: ScopeLines::CursorLine(5),
            },
            ScopeCandidate {
                source: ScopeSource::SingleLineCommand,
                lines: ScopeLines::Range { start: 3, end: 3 },
            },
        ];

        let candidates_b = vec![
            ScopeCandidate {
                source: ScopeSource::SingleLineCommand,
                lines: ScopeLines::Range { start: 3, end: 3 },
            },
            ScopeCandidate {
                source: ScopeSource::CursorLine,
                lines: ScopeLines::CursorLine(5),
            },
        ];

        let result_a = ScopeResolver::resolve(&candidates_a, None, false).unwrap();
        let result_b = ScopeResolver::resolve(&candidates_b, None, false).unwrap();
        assert_eq!(result_a.source, result_b.source);
        assert_eq!(result_a.source, ScopeSource::SingleLineCommand);
    }

    // Validates: Requirement 2.2
    #[test]
    fn apply_filter_wraps_scope_with_all_filter() {
        let scope = ResolvedScope {
            lines: ScopeLines::EntireDocument,
            column_bounds: None,
            source: ScopeSource::EntireDocument,
        };
        let filtered = ScopeResolver::apply_filter(scope, ScopeFilter::All);
        match filtered.lines {
            ScopeLines::Filtered { filter, .. } => assert_eq!(filter, ScopeFilter::All),
            _ => panic!("expected Filtered variant"),
        }
    }

    // Validates: Requirement 2.3
    #[test]
    fn apply_filter_visible() {
        let scope = ResolvedScope {
            lines: ScopeLines::Range { start: 0, end: 50 },
            column_bounds: None,
            source: ScopeSource::ExplicitRange,
        };
        let filtered = ScopeResolver::apply_filter(scope, ScopeFilter::Visible);
        match filtered.lines {
            ScopeLines::Filtered { filter, .. } => assert_eq!(filter, ScopeFilter::Visible),
            _ => panic!("expected Filtered variant"),
        }
    }

    // Validates: Requirement 2.4
    #[test]
    fn apply_filter_excluded() {
        let scope = ResolvedScope {
            lines: ScopeLines::Range { start: 0, end: 50 },
            column_bounds: None,
            source: ScopeSource::ExplicitRange,
        };
        let filtered = ScopeResolver::apply_filter(scope, ScopeFilter::Excluded);
        match filtered.lines {
            ScopeLines::Filtered { filter, .. } => assert_eq!(filter, ScopeFilter::Excluded),
            _ => panic!("expected Filtered variant"),
        }
    }

    // Validates: Requirement 2.5
    #[test]
    fn apply_filter_tagged() {
        let scope = ResolvedScope {
            lines: ScopeLines::CursorLine(10),
            column_bounds: None,
            source: ScopeSource::CursorLine,
        };
        let filtered = ScopeResolver::apply_filter(scope, ScopeFilter::Tagged);
        match filtered.lines {
            ScopeLines::Filtered { filter, .. } => assert_eq!(filter, ScopeFilter::Tagged),
            _ => panic!("expected Filtered variant"),
        }
    }

    // Validates: Requirement 2.6
    #[test]
    fn apply_filter_non_tagged() {
        let scope = ResolvedScope {
            lines: ScopeLines::CursorLine(10),
            column_bounds: None,
            source: ScopeSource::CursorLine,
        };
        let filtered = ScopeResolver::apply_filter(scope, ScopeFilter::NonTagged);
        match filtered.lines {
            ScopeLines::Filtered { filter, .. } => assert_eq!(filter, ScopeFilter::NonTagged),
            _ => panic!("expected Filtered variant"),
        }
    }
}
