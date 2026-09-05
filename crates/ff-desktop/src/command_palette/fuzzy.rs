//! Fuzzy match engine for the Command Palette.
//!
//! Implements subsequence matching with scoring by contiguity, word-boundary
//! position, and display-name length.
//!
//! Validates: Requirement 2.1, 2.2, 2.5 (command-palette)

/// Returns `true` when every character of `query` appears in `target` as a
/// subsequence (case-insensitive).
///
/// Validates: Requirement 2.1, 2.5
pub fn fuzzy_match(query: &str, target: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut target_chars = target.chars().flat_map(|c| c.to_lowercase());
    for qc in query.chars().flat_map(|c| c.to_lowercase()) {
        if !target_chars.any(|tc| tc == qc) {
            return false;
        }
    }
    true
}

/// Score a fuzzy match between `query` and `target` (higher = better).
///
/// Returns `i32::MIN` when `query` is not a subsequence of `target`.
///
/// Scoring bonuses (Req 2.2):
/// - +10 per character in a contiguous run of matched characters
/// - +5 per character matched at a word boundary (start of word)
/// - shorter `target` scores higher: penalty = `target.len() as i32`
///
/// Validates: Requirement 2.2
pub fn fuzzy_score(query: &str, target: &str) -> i32 {
    if query.is_empty() {
        return -(target.len() as i32);
    }

    let q_lower: Vec<char> = query.chars().flat_map(|c| c.to_lowercase()).collect();
    let t_lower: Vec<char> = target.chars().flat_map(|c| c.to_lowercase()).collect();
    let t_orig: Vec<char> = target.chars().collect();

    // Find match positions (greedy, left-to-right).
    let mut positions: Vec<usize> = Vec::with_capacity(q_lower.len());
    let mut ti = 0;
    for &qc in &q_lower {
        let found = (ti..t_lower.len()).find(|&i| t_lower[i] == qc);
        match found {
            Some(i) => {
                positions.push(i);
                ti = i + 1;
            }
            None => return i32::MIN,
        }
    }

    let mut score: i32 = 0;

    // Contiguous run bonus.
    let mut run = 1i32;
    for w in positions.windows(2) {
        if w[1] == w[0] + 1 {
            run += 1;
            score += 10 * run;
        } else {
            run = 1;
        }
    }

    // Word-boundary bonus: position 0 or preceded by a non-alphanumeric char.
    for &pos in &positions {
        let at_boundary = pos == 0
            || t_orig
                .get(pos.wrapping_sub(1))
                .map(|c| !c.is_alphanumeric())
                .unwrap_or(false);
        if at_boundary {
            score += 5;
        }
    }

    // Length penalty (shorter target = higher score).
    score -= target.len() as i32;

    score
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: Requirement 2.1 -- subsequence match returns true.
    #[test]
    fn fuzzy_match_subsequence_returns_true() {
        // Validates: command-palette Requirement 2.1
        assert!(fuzzy_match("fo", "file.open"));
        assert!(fuzzy_match("op", "file.open"));
        assert!(fuzzy_match("fop", "file.open"));
    }

    /// Validates: Requirement 2.1 -- non-subsequence returns false.
    #[test]
    fn fuzzy_match_non_subsequence_returns_false() {
        // Validates: command-palette Requirement 2.1
        assert!(!fuzzy_match("xyz", "file.open"));
        assert!(!fuzzy_match("zz", "file.open"));
    }

    /// Validates: Requirement 2.5 -- matching is case-insensitive.
    #[test]
    fn fuzzy_match_is_case_insensitive() {
        // Validates: command-palette Requirement 2.5
        assert!(fuzzy_match("FO", "file.open"));
        assert!(fuzzy_match("FILE", "File.Open"));
        assert!(fuzzy_match("fo", "FILE.OPEN"));
    }

    /// Validates: Requirement 2.1 -- empty query always matches.
    #[test]
    fn fuzzy_match_empty_query_always_matches() {
        // Validates: command-palette Requirement 2.1
        assert!(fuzzy_match("", "anything"));
        assert!(fuzzy_match("", ""));
    }

    /// Validates: Requirement 2.2 -- contiguous run scores higher than scattered.
    #[test]
    fn fuzzy_score_contiguous_run_scores_higher_than_scattered() {
        // Validates: command-palette Requirement 2.2a
        // "op" matches contiguously in "open" but scattered in "file.open"
        let contiguous = fuzzy_score("op", "open");
        let scattered = fuzzy_score("op", "file.open");
        assert!(
            contiguous > scattered,
            "contiguous={contiguous} scattered={scattered}"
        );
    }

    /// Validates: Requirement 2.2 -- word-boundary match scores higher than mid-word.
    #[test]
    fn fuzzy_score_word_boundary_scores_higher_than_mid_word() {
        // Validates: command-palette Requirement 2.2b
        // "o" at position 0 of "open" (boundary) vs mid-word in "close"
        let boundary = fuzzy_score("o", "open");
        let mid = fuzzy_score("o", "close");
        assert!(boundary > mid, "boundary={boundary} mid={mid}");
    }

    /// Validates: Requirement 2.2 -- shorter target scores higher for equal match quality.
    #[test]
    fn fuzzy_score_shorter_target_scores_higher() {
        // Validates: command-palette Requirement 2.2c
        let short = fuzzy_score("op", "open");
        let long = fuzzy_score("op", "open file dialog");
        assert!(short > long, "short={short} long={long}");
    }

    /// Validates: Requirement 2.2 -- no match returns i32::MIN.
    #[test]
    fn fuzzy_score_no_match_returns_min() {
        // Validates: command-palette Requirement 2.2
        assert_eq!(fuzzy_score("xyz", "file.open"), i32::MIN);
    }

    /// Validates: Requirement 2.5 -- score is case-insensitive.
    #[test]
    fn fuzzy_score_is_case_insensitive() {
        // Validates: command-palette Requirement 2.5
        let lower = fuzzy_score("op", "open");
        let upper = fuzzy_score("OP", "OPEN");
        assert_eq!(lower, upper);
    }
}
