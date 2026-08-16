//! Prefix matching algorithm for completion filtering.

/// Performs a prefix match: returns true if `candidate` starts with `query`.
///
/// When `case_sensitive` is false, both strings are compared case-insensitively.
/// An empty query matches everything.
///
/// # Examples
///
/// ```
/// use ff_completion::matching::prefix_match;
///
/// assert!(prefix_match("fi", "FIND", false));
/// assert!(prefix_match("FI", "find", false));
/// assert!(!prefix_match("FI", "find", true));
/// assert!(prefix_match("", "anything", false));
/// ```
pub fn prefix_match(query: &str, candidate: &str, case_sensitive: bool) -> bool {
    if query.is_empty() {
        return true;
    }
    if case_sensitive {
        candidate.starts_with(query)
    } else {
        let q_lower = query.to_lowercase();
        let c_lower = candidate.to_lowercase();
        c_lower.starts_with(&q_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.2 (case-insensitive prefix match)
    #[test]
    fn case_insensitive_prefix_match_basic() {
        assert!(prefix_match("fi", "FIND", false));
        assert!(prefix_match("FI", "find", false));
        assert!(prefix_match("Find", "FIND", false));
    }

    // Validates: Requirement 6.2 (strict prefix match)
    #[test]
    fn case_sensitive_prefix_match() {
        assert!(prefix_match("FI", "FIND", true));
        assert!(!prefix_match("fi", "FIND", true));
        assert!(prefix_match("fi", "find", true));
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(prefix_match("", "anything", false));
        assert!(prefix_match("", "", false));
        assert!(prefix_match("", "FIND", true));
    }

    #[test]
    fn non_matching_prefix_returns_false() {
        assert!(!prefix_match("xyz", "FIND", false));
        assert!(!prefix_match("ind", "FIND", false)); // not a prefix
    }

    #[test]
    fn exact_match_returns_true() {
        assert!(prefix_match("FIND", "FIND", false));
        assert!(prefix_match("find", "find", true));
    }

    #[test]
    fn query_longer_than_candidate_returns_false() {
        assert!(!prefix_match("FINDING", "FIND", false));
    }

    #[test]
    fn empty_candidate_only_matches_empty_query() {
        assert!(prefix_match("", "", false));
        assert!(!prefix_match("a", "", false));
    }
}
