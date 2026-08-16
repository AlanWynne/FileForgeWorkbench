//! Help search engine — keyword search across all loaded topics.
//!
//! Provides case-insensitive substring matching with relevance ranking:
//! title matches score highest, followed by heading matches, then body matches.

use crate::topic::HelpTopic;
use crate::topic_key::TopicKey;

/// Where within a topic the search match was found — used for ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchLocation {
    /// Exact match on the topic title (highest relevance).
    Title,
    /// Match in a section heading within the body.
    Heading,
    /// Match in the topic body text (lowest relevance).
    Body,
    /// Match on a TopicKey alias.
    Alias,
}

impl MatchLocation {
    /// Returns the base relevance score for this match location.
    pub fn base_score(&self) -> u32 {
        match self {
            Self::Title => 100,
            Self::Heading => 50,
            Self::Alias => 40,
            Self::Body => 10,
        }
    }
}

/// A single result from a help topic search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matching topic's key.
    pub key: TopicKey,
    /// The matching topic's title.
    pub title: String,
    /// A brief excerpt showing the matching context.
    pub excerpt: String,
    /// Relevance score for ranking (higher = more relevant).
    pub relevance: u32,
    /// Where the match was found.
    pub match_location: MatchLocation,
}

/// Keyword search across all loaded help topics with relevance ranking.
///
/// Performs case-insensitive substring matching across topic titles, body text,
/// and aliases. Results are ranked by match location: title > heading > alias > body.
pub struct HelpSearch {
    /// Minimum query length required (default: 2 characters).
    min_query_length: usize,
}

impl HelpSearch {
    /// Create a new search engine with default minimum query length of 2.
    pub fn new() -> Self {
        Self {
            min_query_length: 2,
        }
    }

    /// Search the given topics for matches against the query.
    ///
    /// Returns an empty `Vec` if the query is shorter than the minimum length.
    /// Results are sorted by descending relevance score.
    pub fn query(&self, topics: &[HelpTopic], query: &str) -> Vec<SearchResult> {
        let query_trimmed = query.trim();
        if query_trimmed.len() < self.min_query_length {
            return Vec::new();
        }

        let query_lower = query_trimmed.to_lowercase();
        let mut results = Vec::new();

        for topic in topics {
            if let Some(result) = self.match_topic(topic, &query_lower) {
                results.push(result);
            }
        }

        // Sort by descending relevance
        results.sort_by_key(|r| std::cmp::Reverse(r.relevance));
        results
    }

    /// Match a single topic against the query. Returns the best match or None.
    fn match_topic(&self, topic: &HelpTopic, query_lower: &str) -> Option<SearchResult> {
        // Check title first (highest priority)
        let title_lower = topic.title().to_lowercase();
        if title_lower.contains(query_lower) {
            return Some(SearchResult {
                key: topic.key().clone(),
                title: topic.title().to_string(),
                excerpt: Self::extract_excerpt(topic.title(), query_lower),
                relevance: MatchLocation::Title.base_score(),
                match_location: MatchLocation::Title,
            });
        }

        // Check aliases
        for alias in topic.aliases() {
            if alias.to_lowercase().contains(query_lower) {
                return Some(SearchResult {
                    key: topic.key().clone(),
                    title: topic.title().to_string(),
                    excerpt: format!("Alias: {alias}"),
                    relevance: MatchLocation::Alias.base_score(),
                    match_location: MatchLocation::Alias,
                });
            }
        }

        // Check headings in body (lines starting with #)
        let body = topic.body();
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let heading_text = trimmed.trim_start_matches('#').trim();
                if heading_text.to_lowercase().contains(query_lower) {
                    return Some(SearchResult {
                        key: topic.key().clone(),
                        title: topic.title().to_string(),
                        excerpt: Self::extract_excerpt(heading_text, query_lower),
                        relevance: MatchLocation::Heading.base_score(),
                        match_location: MatchLocation::Heading,
                    });
                }
            }
        }

        // Check body text
        let body_lower = body.to_lowercase();
        if body_lower.contains(query_lower) {
            return Some(SearchResult {
                key: topic.key().clone(),
                title: topic.title().to_string(),
                excerpt: Self::extract_excerpt(body, query_lower),
                relevance: MatchLocation::Body.base_score(),
                match_location: MatchLocation::Body,
            });
        }

        None
    }

    /// Extract a context excerpt around the first match position (±40 chars).
    fn extract_excerpt(text: &str, query_lower: &str) -> String {
        let text_lower = text.to_lowercase();
        if let Some(pos) = text_lower.find(query_lower) {
            let start = pos.saturating_sub(40);
            let end = (pos + query_lower.len() + 40).min(text.len());

            let mut excerpt = String::new();
            if start > 0 {
                excerpt.push_str("...");
            }
            excerpt.push_str(&text[start..end]);
            if end < text.len() {
                excerpt.push_str("...");
            }
            excerpt
        } else {
            // Fallback: return first 80 chars
            let end = 80.min(text.len());
            let mut excerpt = text[..end].to_string();
            if end < text.len() {
                excerpt.push_str("...");
            }
            excerpt
        }
    }
}

impl Default for HelpSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topic::TopicSource;
    use std::path::PathBuf;

    fn make_topic(key: TopicKey, title: &str, body: &str) -> HelpTopic {
        HelpTopic::new(
            key,
            title.to_string(),
            body.to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("test.help.md"),
            },
        )
    }

    // Validates: Requirement 4.1 — Search requires minimum 2 characters
    #[test]
    fn query_below_minimum_length_returns_empty() {
        let search = HelpSearch::new();
        let topics = vec![make_topic(TopicKey::command("FIND"), "FIND", "find text")];
        let results = search.query(&topics, "F");
        assert!(results.is_empty());
    }

    // Validates: Requirement 4.2 — Case-insensitive matching
    #[test]
    fn query_is_case_insensitive() {
        let search = HelpSearch::new();
        let topics = vec![make_topic(
            TopicKey::command("FIND"),
            "FIND Command",
            "searches for text",
        )];

        let results = search.query(&topics, "find");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, TopicKey::command("FIND"));
    }

    // Validates: Requirement 4.4 — Title matches ranked higher than body
    #[test]
    fn title_match_ranks_higher_than_body_match() {
        let search = HelpSearch::new();
        let topics = vec![
            make_topic(
                TopicKey::command("FIND"),
                "FIND Command",
                "The FIND command searches text.",
            ),
            make_topic(
                TopicKey::command("CHANGE"),
                "CHANGE Command",
                "Use FIND with CHANGE for replacements.",
            ),
        ];

        let results = search.query(&topics, "find");
        assert_eq!(results.len(), 2);
        // FIND title match should be first
        assert_eq!(results[0].key, TopicKey::command("FIND"));
        assert_eq!(results[0].match_location, MatchLocation::Title);
        // CHANGE body match should be second
        assert_eq!(results[1].key, TopicKey::command("CHANGE"));
        assert_eq!(results[1].match_location, MatchLocation::Body);
        assert!(results[0].relevance > results[1].relevance);
    }

    // Validates: Requirement 4.5 — No results for unmatched query
    #[test]
    fn query_with_no_matches_returns_empty() {
        let search = HelpSearch::new();
        let topics = vec![make_topic(
            TopicKey::command("FIND"),
            "FIND Command",
            "searches text",
        )];

        let results = search.query(&topics, "zzz_nonexistent");
        assert!(results.is_empty());
    }

    // Validates: Requirement 4.4 — Heading match ranks between title and body
    #[test]
    fn heading_match_ranks_between_title_and_body() {
        let search = HelpSearch::new();
        let topics = vec![
            make_topic(TopicKey::command("FIND"), "FIND Command", "body text only"),
            make_topic(
                TopicKey::command("CHANGE"),
                "CHANGE Command",
                "## Using FIND\nSome body text about finding.",
            ),
        ];

        let results = search.query(&topics, "find");
        assert_eq!(results.len(), 2);
        // Title match first
        assert_eq!(results[0].match_location, MatchLocation::Title);
        // Heading match second (CHANGE has "FIND" in a heading)
        assert_eq!(results[1].match_location, MatchLocation::Heading);
    }

    // Validates: Requirement 4.1 — Excerpt generation
    #[test]
    fn excerpt_is_generated_around_match() {
        let search = HelpSearch::new();
        let body = "The quick brown fox jumps over the lazy dog. FINDME is hidden in the middle of this text that is quite long.";
        let topics = vec![make_topic(TopicKey::command("TEST"), "Test Topic", body)];

        let results = search.query(&topics, "findme");
        assert_eq!(results.len(), 1);
        assert!(results[0].excerpt.contains("FINDME"));
    }
}
