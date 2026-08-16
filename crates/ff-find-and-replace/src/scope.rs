//! Scope filtering: line eligibility, column ranges, and bounds.
//!
//! Addresses: Requirement 2 AC 1–8, Requirement 7 AC 4–6

use crate::types::LineNumber;

/// Scope filter controlling which lines are eligible for search.
///
/// Addresses: Requirement 2 AC 1–4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScopeModifier {
    /// Search all lines regardless of state (FIND ALL).
    #[default]
    All,
    /// Search only visible lines (default for FIND without ALL).
    Visible,
    /// Search only excluded (hidden) lines.
    Excluded,
    /// Search only tagged lines.
    Tagged,
    /// Search only non-tagged lines.
    NonTagged,
}

impl ScopeModifier {
    /// Parse from a command token string (case-insensitive).
    pub fn from_token(token: &str) -> Option<Self> {
        match token.to_ascii_uppercase().as_str() {
            "ALL" => Some(Self::All),
            "VISIBLE" => Some(Self::Visible),
            "EXCLUDED" => Some(Self::Excluded),
            "TAGGED" => Some(Self::Tagged),
            "NONTAGGED" => Some(Self::NonTagged),
            _ => None,
        }
    }
}

/// Trait for querying line visibility and tag state.
///
/// Implemented by the exclude-show-filter or display-line-mapping layer.
///
/// Addresses: Requirement 2 AC 1–4
pub trait ScopeFilterProvider: Send + Sync {
    /// Whether the line is visible (not excluded).
    fn is_visible(&self, line: LineNumber) -> bool;

    /// Whether the line is excluded (hidden).
    fn is_excluded(&self, line: LineNumber) -> bool;

    /// Whether the line is tagged.
    fn is_tagged(&self, line: LineNumber) -> bool;
}

/// A scope filter that accepts all lines (no filtering).
///
/// Used as the default when no scope constraints are active.
#[derive(Debug, Clone, Copy)]
pub struct AllLinesFilter;

impl ScopeFilterProvider for AllLinesFilter {
    fn is_visible(&self, _line: LineNumber) -> bool {
        true
    }

    fn is_excluded(&self, _line: LineNumber) -> bool {
        false
    }

    fn is_tagged(&self, _line: LineNumber) -> bool {
        false
    }
}

/// Check whether a line is eligible under a given scope modifier.
///
/// Addresses: Requirement 2 AC 1–4, AC 8
pub fn line_passes_scope(
    scope: ScopeModifier,
    line: LineNumber,
    filter: &dyn ScopeFilterProvider,
) -> bool {
    match scope {
        ScopeModifier::All => true,
        ScopeModifier::Visible => filter.is_visible(line),
        ScopeModifier::Excluded => filter.is_excluded(line),
        ScopeModifier::Tagged => filter.is_tagged(line),
        ScopeModifier::NonTagged => !filter.is_tagged(line),
    }
}

/// An optional column range restricting search to a horizontal slice.
///
/// Columns are 1-based and refer to byte positions within a line.
///
/// Addresses: Requirement 2 AC 5–7, Requirement 7 AC 4–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRange {
    /// Start column (1-based, inclusive).
    pub start: u32,
    /// End column (1-based, inclusive).
    pub end: u32,
}

impl ColumnRange {
    /// Create a new column range, returning None if start > end or start is 0.
    pub fn new(start: u32, end: u32) -> Option<Self> {
        if start == 0 || start > end {
            None
        } else {
            Some(Self { start, end })
        }
    }

    /// Compute the intersection of two column ranges.
    /// Returns None if they don't overlap.
    ///
    /// Addresses: Requirement 7 AC 6
    pub fn intersect(&self, other: &ColumnRange) -> Option<ColumnRange> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start <= end {
            Some(ColumnRange { start, end })
        } else {
            None
        }
    }

    /// Convert to 0-based byte offsets within a line.
    /// Returns (start_offset, end_offset) where end_offset is exclusive.
    pub fn to_byte_offsets(&self) -> (u64, u64) {
        ((self.start as u64) - 1, self.end as u64)
    }
}

/// Active BOUNDS settings affecting find operations.
///
/// Addresses: Requirement 2 AC 5–6
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    /// Left boundary column (1-based, inclusive).
    pub left: u32,
    /// Right boundary column (1-based, inclusive).
    pub right: u32,
}

impl Bounds {
    /// Convert bounds to a ColumnRange.
    pub fn to_column_range(&self) -> ColumnRange {
        ColumnRange {
            start: self.left,
            end: self.right,
        }
    }
}

/// Resolve the effective column range from explicit range and bounds.
///
/// If both are present, the intersection is used.
/// If only one is present, that one is used.
/// If neither is present, returns None (search full line).
///
/// Addresses: Requirement 2 AC 5–7, Requirement 7 AC 5–6
pub fn resolve_column_range(
    explicit: Option<&ColumnRange>,
    bounds: Option<&Bounds>,
    bounds_affect_find: bool,
) -> Option<ColumnRange> {
    let bounds_range = if bounds_affect_find {
        bounds.map(|b| b.to_column_range())
    } else {
        None
    };

    match (explicit, bounds_range) {
        (Some(e), Some(b)) => e.intersect(&b),
        (Some(e), None) => Some(*e),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_range_new_rejects_zero_start() {
        assert_eq!(ColumnRange::new(0, 10), None);
    }

    #[test]
    fn column_range_new_rejects_start_greater_than_end() {
        assert_eq!(ColumnRange::new(10, 5), None);
    }

    #[test]
    fn column_range_new_accepts_valid_range() {
        let cr = ColumnRange::new(1, 80).unwrap();
        assert_eq!(cr.start, 1);
        assert_eq!(cr.end, 80);
    }

    #[test]
    fn column_range_intersect_returns_overlap() {
        let a = ColumnRange::new(5, 20).unwrap();
        let b = ColumnRange::new(10, 30).unwrap();
        let result = a.intersect(&b).unwrap();
        assert_eq!(result.start, 10);
        assert_eq!(result.end, 20);
    }

    #[test]
    fn column_range_intersect_returns_none_when_disjoint() {
        let a = ColumnRange::new(1, 5).unwrap();
        let b = ColumnRange::new(10, 20).unwrap();
        assert_eq!(a.intersect(&b), None);
    }

    #[test]
    fn column_range_intersect_is_commutative() {
        let a = ColumnRange::new(3, 15).unwrap();
        let b = ColumnRange::new(10, 25).unwrap();
        assert_eq!(a.intersect(&b), b.intersect(&a));
    }

    #[test]
    fn column_range_to_byte_offsets_converts_to_zero_based() {
        let cr = ColumnRange::new(1, 80).unwrap();
        assert_eq!(cr.to_byte_offsets(), (0, 80));

        let cr2 = ColumnRange::new(5, 10).unwrap();
        assert_eq!(cr2.to_byte_offsets(), (4, 10));
    }

    #[test]
    fn scope_modifier_from_token_parses_case_insensitively() {
        assert_eq!(
            ScopeModifier::from_token("TAGGED"),
            Some(ScopeModifier::Tagged)
        );
        assert_eq!(
            ScopeModifier::from_token("excluded"),
            Some(ScopeModifier::Excluded)
        );
        assert_eq!(
            ScopeModifier::from_token("Visible"),
            Some(ScopeModifier::Visible)
        );
        assert_eq!(
            ScopeModifier::from_token("nontagged"),
            Some(ScopeModifier::NonTagged)
        );
        assert_eq!(ScopeModifier::from_token("all"), Some(ScopeModifier::All));
        assert_eq!(ScopeModifier::from_token("invalid"), None);
    }

    #[test]
    fn line_passes_scope_all_always_returns_true() {
        let filter = AllLinesFilter;
        assert!(line_passes_scope(
            ScopeModifier::All,
            LineNumber(0),
            &filter
        ));
        assert!(line_passes_scope(
            ScopeModifier::All,
            LineNumber(999),
            &filter
        ));
    }

    #[test]
    fn resolve_column_range_prefers_intersection_when_both_present() {
        let explicit = ColumnRange::new(5, 20).unwrap();
        let bounds = Bounds {
            left: 10,
            right: 30,
        };
        let result = resolve_column_range(Some(&explicit), Some(&bounds), true);
        assert_eq!(result, Some(ColumnRange { start: 10, end: 20 }));
    }

    #[test]
    fn resolve_column_range_ignores_bounds_when_bounds_affect_find_is_false() {
        let explicit = ColumnRange::new(5, 20).unwrap();
        let bounds = Bounds {
            left: 10,
            right: 30,
        };
        let result = resolve_column_range(Some(&explicit), Some(&bounds), false);
        assert_eq!(result, Some(explicit));
    }

    #[test]
    fn resolve_column_range_returns_none_when_no_constraints() {
        let result = resolve_column_range(None, None, true);
        assert_eq!(result, None);
    }
}
