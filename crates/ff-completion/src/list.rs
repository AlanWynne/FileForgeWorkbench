//! `CompletionList` — filtered and ranked completion items.

use std::collections::HashSet;

use crate::candidate::CompletionCandidate;
use crate::config::MatchingMode;
use crate::matching::{fuzzy_match, prefix_match};

/// A display-ready completion item after filtering and ranking.
///
/// Includes match highlight information for the popup renderer.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The original candidate data.
    pub candidate: CompletionCandidate,
    /// Computed relevance score (higher is better). Used for sort order.
    pub score: f64,
    /// Character positions in the label that matched the typed prefix.
    /// Used by the renderer to highlight matched characters.
    pub match_positions: Vec<usize>,
}

/// The complete set of filtered, ranked completion items for the current context.
///
/// Manages the candidate list, supports re-filtering, de-duplication,
/// and provides indexed access for the selection state.
#[derive(Debug, Clone)]
pub struct CompletionList {
    /// All raw candidates from providers (before filtering).
    all_candidates: Vec<CompletionCandidate>,
    /// The filtered and ranked items, in display order.
    filtered_items: Vec<CompletionItem>,
    /// The matching mode to use for filtering.
    matching_mode: MatchingMode,
    /// Whether matching is case-sensitive.
    case_sensitive: bool,
}

impl CompletionList {
    /// Creates a new list from raw candidates.
    ///
    /// De-duplicates candidates by `insert_text` (keeping higher relevance),
    /// then applies an initial empty filter (all items visible).
    pub fn new(
        candidates: Vec<CompletionCandidate>,
        matching_mode: MatchingMode,
        case_sensitive: bool,
    ) -> Self {
        let deduped = deduplicate(candidates);
        let filtered_items = deduped
            .iter()
            .map(|c| CompletionItem {
                candidate: c.clone(),
                score: c.base_relevance as f64,
                match_positions: vec![],
            })
            .collect();

        Self {
            all_candidates: deduped,
            filtered_items,
            matching_mode,
            case_sensitive,
        }
    }

    /// Filters the list against the given query string.
    ///
    /// Re-filters from the full candidate set. The filtered result replaces
    /// any previous filter state.
    pub fn filter(&mut self, query: &str) {
        if query.is_empty() {
            self.filtered_items = self
                .all_candidates
                .iter()
                .map(|c| CompletionItem {
                    candidate: c.clone(),
                    score: c.base_relevance as f64,
                    match_positions: vec![],
                })
                .collect();
            return;
        }

        self.filtered_items = match self.matching_mode {
            MatchingMode::Prefix => self.filter_prefix(query),
            MatchingMode::Fuzzy => self.filter_fuzzy(query),
        };

        // Sort by score descending, then by label length, then alphabetically
        self.filtered_items.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.candidate.label.len().cmp(&b.candidate.label.len()))
                .then_with(|| a.candidate.label.cmp(&b.candidate.label))
        });
    }

    /// Returns the number of filtered items.
    pub fn len(&self) -> usize {
        self.filtered_items.len()
    }

    /// Returns true if the filtered list is empty.
    pub fn is_empty(&self) -> bool {
        self.filtered_items.is_empty()
    }

    /// Returns a reference to the item at the given index in the filtered view.
    pub fn get(&self, index: usize) -> Option<&CompletionItem> {
        self.filtered_items.get(index)
    }

    /// Returns a slice of all filtered items.
    pub fn items(&self) -> &[CompletionItem] {
        &self.filtered_items
    }

    /// Returns items within a visible window (for scrolled rendering).
    pub fn visible_window(&self, scroll_offset: usize, max_visible: usize) -> &[CompletionItem] {
        let start = scroll_offset.min(self.filtered_items.len());
        let end = (start + max_visible).min(self.filtered_items.len());
        &self.filtered_items[start..end]
    }

    /// Returns the total number of candidates (before filtering).
    pub fn total_candidates(&self) -> usize {
        self.all_candidates.len()
    }

    fn filter_prefix(&self, query: &str) -> Vec<CompletionItem> {
        self.all_candidates
            .iter()
            .filter(|c| prefix_match(query, &c.label, self.case_sensitive))
            .map(|c| {
                let mut score = c.base_relevance as f64;
                // Exact match bonus
                if c.label.eq_ignore_ascii_case(query) {
                    score += 100.0;
                }
                // Shorter label bonus (closer match)
                score += 20.0 - (c.label.len() as f64).min(20.0);
                CompletionItem {
                    candidate: c.clone(),
                    score,
                    match_positions: (0..query.len()).collect(),
                }
            })
            .collect()
    }

    fn filter_fuzzy(&self, query: &str) -> Vec<CompletionItem> {
        self.all_candidates
            .iter()
            .filter_map(|c| {
                fuzzy_match(query, &c.label, self.case_sensitive).map(|result| {
                    let score = c.base_relevance as f64 + result.score as f64;
                    CompletionItem {
                        candidate: c.clone(),
                        score,
                        match_positions: result.matched_positions,
                    }
                })
            })
            .collect()
    }
}

/// De-duplicates candidates by `insert_text`, keeping the one with higher `base_relevance`.
fn deduplicate(mut candidates: Vec<CompletionCandidate>) -> Vec<CompletionCandidate> {
    let mut seen = HashSet::new();
    let mut result = Vec::with_capacity(candidates.len());

    // Sort by relevance descending so we keep the highest-relevance duplicate
    candidates.sort_by_key(|c| std::cmp::Reverse(c.base_relevance));

    for candidate in candidates {
        if seen.insert(candidate.insert_text.clone()) {
            result.push(candidate);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::CompletionKind;

    fn make_candidates() -> Vec<CompletionCandidate> {
        vec![
            CompletionCandidate::new("FIND", "FIND", CompletionKind::Command),
            CompletionCandidate::new("FILTER", "FILTER", CompletionKind::Command),
            CompletionCandidate::new("FILE.SAVE", "FILE.SAVE", CompletionKind::Command),
            CompletionCandidate::new("SAVE", "SAVE", CompletionKind::Command),
            CompletionCandidate::new("SORT", "SORT", CompletionKind::Command),
        ]
    }

    // Validates: Requirement 1.6 (dynamic filtering)
    #[test]
    fn filter_narrows_results() {
        let mut list = CompletionList::new(make_candidates(), MatchingMode::Prefix, false);
        assert_eq!(list.len(), 5); // all visible initially

        list.filter("FI");
        assert_eq!(list.len(), 3); // FIND, FILTER, FILE.SAVE

        list.filter("FIN");
        assert_eq!(list.len(), 1); // FIND only
    }

    // Validates: Requirement 1.7 (empty on no match)
    #[test]
    fn filter_produces_empty_when_no_match() {
        let mut list = CompletionList::new(make_candidates(), MatchingMode::Prefix, false);
        list.filter("XYZ");
        assert!(list.is_empty());
    }

    // Validates: Requirement 1.4 (ranking: shorter names first)
    #[test]
    fn prefix_filter_ranks_shorter_names_first() {
        let mut list = CompletionList::new(make_candidates(), MatchingMode::Prefix, false);
        list.filter("FI");
        let labels: Vec<_> = list
            .items()
            .iter()
            .map(|i| i.candidate.label.as_str())
            .collect();
        // FIND (4) should come before FILTER (6) which should come before FILE.SAVE (9)
        assert_eq!(labels[0], "FIND");
        assert_eq!(labels[1], "FILTER");
        assert_eq!(labels[2], "FILE.SAVE");
    }

    // Validates: Requirement 6.1 (fuzzy matching)
    #[test]
    fn fuzzy_filter_matches_subsequence() {
        let mut list = CompletionList::new(make_candidates(), MatchingMode::Fuzzy, false);
        list.filter("fs");
        // Should match FILE.SAVE (f...s...)
        let labels: Vec<_> = list
            .items()
            .iter()
            .map(|i| i.candidate.label.as_str())
            .collect();
        assert!(labels.contains(&"FILE.SAVE"));
    }

    // Validates: Requirement 2.7 (de-duplication by insert_text)
    #[test]
    fn deduplication_by_insert_text_keeps_higher_relevance() {
        let candidates = vec![
            CompletionCandidate::new("FIND", "FIND", CompletionKind::Command).with_relevance(5),
            CompletionCandidate::new("FIND", "FIND", CompletionKind::Plugin).with_relevance(10),
        ];
        let list = CompletionList::new(candidates, MatchingMode::Prefix, false);
        assert_eq!(list.total_candidates(), 1);
        // The higher-relevance one (10) should be kept
        assert_eq!(list.items()[0].candidate.base_relevance, 10);
    }

    // Validates: Requirement 1.6 (filter idempotence)
    #[test]
    fn filter_same_query_twice_produces_same_result() {
        let mut list = CompletionList::new(make_candidates(), MatchingMode::Prefix, false);
        list.filter("FI");
        let first: Vec<_> = list
            .items()
            .iter()
            .map(|i| i.candidate.label.clone())
            .collect();
        list.filter("FI");
        let second: Vec<_> = list
            .items()
            .iter()
            .map(|i| i.candidate.label.clone())
            .collect();
        assert_eq!(first, second);
    }

    #[test]
    fn get_returns_item_at_index() {
        let list = CompletionList::new(make_candidates(), MatchingMode::Prefix, false);
        assert!(list.get(0).is_some());
        assert!(list.get(99).is_none());
    }

    #[test]
    fn visible_window_returns_correct_slice() {
        let list = CompletionList::new(make_candidates(), MatchingMode::Prefix, false);
        let window = list.visible_window(1, 2);
        assert_eq!(window.len(), 2);
    }

    #[test]
    fn empty_query_shows_all_candidates() {
        let mut list = CompletionList::new(make_candidates(), MatchingMode::Prefix, false);
        list.filter("FI");
        assert_eq!(list.len(), 3);
        list.filter("");
        assert_eq!(list.len(), 5);
    }
}
