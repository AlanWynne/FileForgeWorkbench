//! Fuzzy (subsequence) matching algorithm for completion filtering.

/// The result of a successful fuzzy match, containing match quality information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuzzyMatchResult {
    /// The character positions in the candidate that matched the query.
    /// Positions are strictly increasing.
    pub matched_positions: Vec<usize>,
    /// The overall match score. Higher is better.
    pub score: u32,
    /// Bonus points for consecutive matched characters.
    pub contiguity_bonus: u32,
}

/// Performs a fuzzy (subsequence) match: returns `Some(FuzzyMatchResult)` if all
/// characters of `query` appear in `candidate` in the same order (not necessarily
/// consecutively).
///
/// When `case_sensitive` is false, characters are compared case-insensitively.
/// An empty query matches everything with a score of 0.
///
/// # Scoring
///
/// The scoring algorithm rewards:
/// - Consecutive matches (contiguity bonus: +3 per consecutive pair)
/// - Matches at the start of the candidate (+5 bonus)
/// - Matches at word boundaries (after `_`, `.`, or uppercase transition) (+2 per boundary match)
/// - Shorter candidates (shorter = better match quality)
///
/// # Examples
///
/// ```
/// use ff_completion::matching::fuzzy_match;
///
/// let result = fuzzy_match("fs", "file.save", false);
/// assert!(result.is_some());
///
/// let result = fuzzy_match("xyz", "file.save", false);
/// assert!(result.is_none());
/// ```
pub fn fuzzy_match(query: &str, candidate: &str, case_sensitive: bool) -> Option<FuzzyMatchResult> {
    if query.is_empty() {
        return Some(FuzzyMatchResult {
            matched_positions: vec![],
            score: 0,
            contiguity_bonus: 0,
        });
    }

    let query_chars: Vec<char> = if case_sensitive {
        query.chars().collect()
    } else {
        query.chars().map(|c| c.to_ascii_lowercase()).collect()
    };

    let candidate_chars: Vec<char> = candidate.chars().collect();
    let candidate_lower: Vec<char> = if case_sensitive {
        candidate_chars.clone()
    } else {
        candidate_chars
            .iter()
            .map(|c| c.to_ascii_lowercase())
            .collect()
    };

    // Find matched positions using a greedy forward scan
    let mut matched_positions = Vec::with_capacity(query_chars.len());
    let mut candidate_idx = 0;

    for &qc in &query_chars {
        let mut found = false;
        while candidate_idx < candidate_lower.len() {
            if candidate_lower[candidate_idx] == qc {
                matched_positions.push(candidate_idx);
                candidate_idx += 1;
                found = true;
                break;
            }
            candidate_idx += 1;
        }
        if !found {
            return None;
        }
    }

    // Compute score
    let mut score: u32 = 0;
    let mut contiguity_bonus: u32 = 0;

    // Base score: query length matched
    score += query_chars.len() as u32;

    // Contiguity bonus: consecutive matched positions
    for i in 1..matched_positions.len() {
        if matched_positions[i] == matched_positions[i - 1] + 1 {
            contiguity_bonus += 3;
        }
    }
    score += contiguity_bonus;

    // Start-of-string bonus
    if !matched_positions.is_empty() && matched_positions[0] == 0 {
        score += 5;
    }

    // Word boundary bonus
    for &pos in &matched_positions {
        if pos > 0 {
            let prev = candidate_chars[pos - 1];
            if prev == '_'
                || prev == '.'
                || prev == '-'
                || prev == '/'
                || (prev.is_lowercase() && candidate_chars[pos].is_uppercase())
            {
                score += 2;
            }
        }
    }

    // Shorter candidates get a small bonus (inversely proportional to length)
    if !candidate_chars.is_empty() {
        score += (20_u32).saturating_sub(candidate_chars.len() as u32);
    }

    Some(FuzzyMatchResult {
        matched_positions,
        score,
        contiguity_bonus,
    })
}

/// Returns the highlight positions for a fuzzy match of `query` against `candidate_label`.
///
/// This is a convenience wrapper around `fuzzy_match` that extracts just the
/// positions for rendering purposes.
pub fn highlight_positions(query: &str, candidate_label: &str, case_sensitive: bool) -> Vec<usize> {
    fuzzy_match(query, candidate_label, case_sensitive)
        .map(|r| r.matched_positions)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 6.1 (fuzzy subsequence match)
    #[test]
    fn fuzzy_match_finds_subsequence() {
        let result = fuzzy_match("fs", "file.save", false);
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.matched_positions.len(), 2);
        // 'f' at 0, 's' at 5
        assert_eq!(r.matched_positions[0], 0);
        assert_eq!(r.matched_positions[1], 5);
    }

    #[test]
    fn fuzzy_match_case_insensitive() {
        let result = fuzzy_match("FS", "file.save", false);
        assert!(result.is_some());
    }

    #[test]
    fn fuzzy_match_case_sensitive_fails() {
        let result = fuzzy_match("FS", "file.save", true);
        assert!(result.is_none());
    }

    #[test]
    fn fuzzy_match_non_matching_returns_none() {
        assert!(fuzzy_match("xyz", "file.save", false).is_none());
        assert!(fuzzy_match("zf", "file.save", false).is_none()); // out of order
    }

    #[test]
    fn fuzzy_match_empty_query_matches_everything() {
        let result = fuzzy_match("", "anything", false);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.matched_positions.is_empty());
        assert_eq!(r.score, 0);
    }

    // Validates: Requirement 6.4 (contiguity bonus scoring)
    #[test]
    fn fuzzy_match_consecutive_chars_get_contiguity_bonus() {
        let result_consecutive = fuzzy_match("fin", "find", false).unwrap();
        let result_spread = fuzzy_match("fnd", "find", false).unwrap();
        // "fin" in "find" = positions [0,1,2] — 2 consecutive pairs = bonus 6
        assert_eq!(result_consecutive.contiguity_bonus, 6);
        // "fnd" in "find" = positions [0,2,3] — 1 consecutive pair = bonus 3
        assert_eq!(result_spread.contiguity_bonus, 3);
        assert!(result_consecutive.score > result_spread.score);
    }

    #[test]
    fn fuzzy_match_start_of_string_bonus() {
        let start_match = fuzzy_match("f", "find", false).unwrap();
        let mid_match = fuzzy_match("i", "find", false).unwrap();
        // Starting at position 0 gets +5 bonus
        assert!(start_match.score > mid_match.score);
    }

    // Validates: Requirement 6.3 (highlight positions)
    #[test]
    fn highlight_positions_returns_matched_indices() {
        let positions = highlight_positions("fs", "file.save", false);
        assert_eq!(positions, vec![0, 5]);
    }

    #[test]
    fn highlight_positions_empty_on_no_match() {
        let positions = highlight_positions("xyz", "file.save", false);
        assert!(positions.is_empty());
    }

    #[test]
    fn fuzzy_match_positions_are_strictly_increasing() {
        let result = fuzzy_match("abc", "aXbYc", false).unwrap();
        for i in 1..result.matched_positions.len() {
            assert!(result.matched_positions[i] > result.matched_positions[i - 1]);
        }
    }

    #[test]
    fn fuzzy_match_word_boundary_bonus() {
        // "s" after a dot gets word boundary bonus
        let result = fuzzy_match("s", "file.save", false).unwrap();
        // position 5 is after '.', should get boundary bonus
        assert!(result.score > 0);
    }
}
