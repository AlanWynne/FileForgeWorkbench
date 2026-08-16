//! Work priority type and built-in priority constants.

/// Numeric priority for idle work sources. Lower values = higher priority.
///
/// Priority 0 is the highest possible; u32::MAX is the lowest.
///
/// # Examples
///
/// ```
/// use ff_idle_processing::WorkPriority;
/// assert!(WorkPriority::SYNTAX_HIGHLIGHT < WorkPriority::SEARCH_INDEX);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkPriority(pub u32);

impl WorkPriority {
    /// Syntax re-highlighting beyond viewport — highest built-in priority.
    pub const SYNTAX_HIGHLIGHT: Self = Self(10);

    /// Word-wrap height measurement.
    pub const WRAP_CALCULATION: Self = Self(20);

    /// Fold-level computation for collapsed regions.
    pub const FOLD_COMPUTATION: Self = Self(30);

    /// Search index building for find-all.
    pub const SEARCH_INDEX: Self = Self(40);

    /// Returns the raw numeric value.
    pub fn value(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_value_is_higher_priority() {
        // Validates: Requirement 4 AC 1
        assert!(WorkPriority::SYNTAX_HIGHLIGHT < WorkPriority::WRAP_CALCULATION);
        assert!(WorkPriority::WRAP_CALCULATION < WorkPriority::FOLD_COMPUTATION);
        assert!(WorkPriority::FOLD_COMPUTATION < WorkPriority::SEARCH_INDEX);
    }

    #[test]
    fn well_known_constants_have_correct_values() {
        // Validates: Requirement 4 AC 2
        assert_eq!(WorkPriority::SYNTAX_HIGHLIGHT.value(), 10);
        assert_eq!(WorkPriority::WRAP_CALCULATION.value(), 20);
        assert_eq!(WorkPriority::FOLD_COMPUTATION.value(), 30);
        assert_eq!(WorkPriority::SEARCH_INDEX.value(), 40);
    }

    #[test]
    fn custom_priority_between_builtins() {
        // Validates: Requirement 4 AC 5
        let custom = WorkPriority(25);
        assert!(WorkPriority::WRAP_CALCULATION < custom);
        assert!(custom < WorkPriority::FOLD_COMPUTATION);
    }
}
