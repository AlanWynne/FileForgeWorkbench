//! State for the Global Search Results panel.
//!
//! Addresses: global-search Requirement 1, 2, 4, 5, 6

use ff_find_and_replace::SearchMode;
use ff_global_search::search::GlobalSearchRequest;
use ff_global_search::{CancellationToken, FileMatches, SearchEvent, SearchResult};
use tokio::sync::mpsc;

/// Options controlling the current search.
///
/// Addresses: Requirement 2.1, 2.2, 2.3
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub include_globs: String,
    pub exclude_globs: String,
}

/// State of a running or completed search.
pub enum SearchPhase {
    /// No search has been run yet.
    Idle,
    /// Search is running -- receiver is live.
    Running {
        receiver: mpsc::Receiver<SearchEvent>,
        cancel: CancellationToken,
        files_scanned: u64,
        matches_found: u64,
    },
    /// Search completed.
    Done {
        total_files: u64,
        total_matches: u64,
    },
    /// Search was cancelled.
    Cancelled,
}

/// Confirmation state for Replace All.
///
/// Addresses: Requirement 5.2
pub enum ReplaceConfirm {
    /// No confirmation pending.
    None,
    /// Waiting for user to confirm replace of `file_count` files / `match_count` matches.
    Pending {
        file_count: usize,
        match_count: usize,
    },
}

/// All state for the Search Results panel.
///
/// Addresses: Requirement 4.1, 4.2, 5.1, 6.1
pub struct SearchResultsPanelState {
    /// Current search query text.
    pub query: String,
    /// Current search options.
    pub options: SearchOptions,
    /// Replace input text (empty = replace not expanded).
    pub replace_text: String,
    /// Whether the replace input row is visible.
    pub replace_expanded: bool,
    /// Accumulated results grouped by file.
    pub results: Vec<FileMatches>,
    /// Current search phase.
    pub phase: SearchPhase,
    /// Index of the currently keyboard-selected match (file_idx, match_idx).
    pub selected: Option<(usize, usize)>,
    /// File sections that have been manually collapsed by the user.
    pub collapsed_files: std::collections::HashSet<String>,
    /// Inline regex error message.
    pub regex_error: Option<String>,
    /// Replace confirmation state.
    pub replace_confirm: ReplaceConfirm,
    /// Search history (most recent first, max 20).
    ///
    /// Addresses: Requirement 6.1, 6.2
    pub history: Vec<String>,
    /// Whether the history dropdown is open.
    pub history_open: bool,
}

impl SearchResultsPanelState {
    /// Create a new, idle panel state.
    pub fn new() -> Self {
        Self {
            query: String::new(),
            options: SearchOptions::default(),
            replace_text: String::new(),
            replace_expanded: false,
            results: Vec::new(),
            phase: SearchPhase::Idle,
            selected: None,
            collapsed_files: std::collections::HashSet::new(),
            regex_error: None,
            replace_confirm: ReplaceConfirm::None,
            history: Vec::new(),
            history_open: false,
        }
    }

    /// Restore search history from session.
    ///
    /// Addresses: Requirement 6.2
    pub fn restore_history(&mut self, history: Vec<String>) {
        self.history = history;
    }

    /// Record a query in the history list (most recent first, capped at 20).
    ///
    /// Addresses: Requirement 6.1
    pub fn push_history(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        self.history.retain(|q| q != query);
        self.history.insert(0, query.to_string());
        self.history.truncate(20);
    }

    /// Poll the receiver for new events (called every frame while Running).
    ///
    /// Addresses: Requirement 3.3, 4.7
    pub fn poll_events(&mut self) {
        let mut done = false;
        let mut total_files = 0u64;
        let mut total_matches = 0u64;
        let mut cancelled = false;

        if let SearchPhase::Running {
            ref mut receiver,
            ref mut files_scanned,
            ref mut matches_found,
            ..
        } = self.phase
        {
            // Drain up to 64 events per frame to stay responsive.
            for _ in 0..64 {
                match receiver.try_recv() {
                    Ok(SearchEvent::MatchFound(fm)) => {
                        *matches_found += fm.matches.len() as u64;
                        self.results.push(fm);
                    }
                    Ok(SearchEvent::Progress {
                        files_scanned: fs,
                        matches_found: mf,
                    }) => {
                        *files_scanned = fs;
                        *matches_found = mf;
                    }
                    Ok(SearchEvent::Completed {
                        total_files: tf,
                        total_matches: tm,
                    }) => {
                        total_files = tf;
                        total_matches = tm;
                        done = true;
                        break;
                    }
                    Ok(SearchEvent::Cancelled) => {
                        cancelled = true;
                        break;
                    }
                    Err(_) => break,
                }
            }
        }

        if done {
            self.phase = SearchPhase::Done {
                total_files,
                total_matches,
            };
        } else if cancelled {
            self.phase = SearchPhase::Cancelled;
        }
    }

    /// Build a `GlobalSearchRequest` from the current panel state and roots.
    ///
    /// Returns `Err` with an inline message if the regex is invalid.
    ///
    /// Addresses: Requirement 2.5, 2.6
    pub fn build_request(&mut self, roots: Vec<String>) -> Result<GlobalSearchRequest, String> {
        self.regex_error = None;
        let mode = if self.options.use_regex {
            // Validate regex before spawning.
            if let Err(e) = regex::Regex::new(&self.query) {
                let msg = format!("Invalid regex: {e}");
                self.regex_error = Some(msg.clone());
                return Err(msg);
            }
            SearchMode::Regex
        } else {
            SearchMode::Literal
        };

        let include_globs = self
            .options
            .include_globs
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let exclude_globs = self
            .options
            .exclude_globs
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(GlobalSearchRequest {
            query: self.query.clone(),
            mode,
            case_sensitive: self.options.case_sensitive,
            whole_word: self.options.whole_word,
            include_globs,
            exclude_globs,
            roots,
        })
    }

    /// Total match count across all result files.
    pub fn total_match_count(&self) -> usize {
        self.results.iter().map(|fm| fm.matches.len()).sum()
    }

    /// Navigate to the next match (Down arrow).
    ///
    /// Addresses: Requirement 4.5
    pub fn select_next(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let (fi, mi) = self.selected.unwrap_or((0, 0));
        let file_count = self.results.len();
        let match_count = self.results[fi].matches.len();
        if mi + 1 < match_count {
            self.selected = Some((fi, mi + 1));
        } else if fi + 1 < file_count {
            self.selected = Some((fi + 1, 0));
        }
    }

    /// Navigate to the previous match (Up arrow).
    ///
    /// Addresses: Requirement 4.5
    pub fn select_prev(&mut self) {
        let (fi, mi) = match self.selected {
            None => return,
            Some(s) => s,
        };
        if mi > 0 {
            self.selected = Some((fi, mi - 1));
        } else if fi > 0 {
            let prev_fi = fi - 1;
            let prev_mi = self.results[prev_fi].matches.len().saturating_sub(1);
            self.selected = Some((prev_fi, prev_mi));
        }
    }

    /// Return the currently selected `SearchResult` if any.
    pub fn selected_result(&self) -> Option<(&FileMatches, &SearchResult)> {
        let (fi, mi) = self.selected?;
        let fm = self.results.get(fi)?;
        let sr = fm.matches.get(mi)?;
        Some((fm, sr))
    }
}

impl Default for SearchResultsPanelState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_global_search::SearchResult;

    fn make_fm(path: &str, count: usize) -> FileMatches {
        FileMatches {
            file_path: path.to_string(),
            matches: (0..count)
                .map(|i| SearchResult {
                    line_number: i as u64 + 1,
                    col_start: 0,
                    col_end: 3,
                    line_text: "foo".to_string(),
                })
                .collect(),
        }
    }

    // Validates: Requirement 6.1 -- history capped at 20, most recent first
    #[test]
    fn push_history_caps_at_20_and_deduplicates() {
        let mut state = SearchResultsPanelState::new();
        for i in 0..25 {
            state.push_history(&format!("query{i}"));
        }
        assert_eq!(state.history.len(), 20);
        assert_eq!(state.history[0], "query24");
        // Push a duplicate -- should move to front, not grow.
        state.push_history("query24");
        assert_eq!(state.history.len(), 20);
        assert_eq!(state.history[0], "query24");
    }

    // Validates: Requirement 4.5 -- keyboard navigation wraps across files
    #[test]
    fn select_next_advances_across_files() {
        let mut state = SearchResultsPanelState::new();
        state.results.push(make_fm("a.rs", 2));
        state.results.push(make_fm("b.rs", 1));
        state.selected = Some((0, 0));
        state.select_next();
        assert_eq!(state.selected, Some((0, 1)));
        state.select_next();
        assert_eq!(state.selected, Some((1, 0)));
        // At last match -- should not advance further.
        state.select_next();
        assert_eq!(state.selected, Some((1, 0)));
    }

    // Validates: Requirement 4.5 -- select_prev goes backwards
    #[test]
    fn select_prev_goes_backwards_across_files() {
        let mut state = SearchResultsPanelState::new();
        state.results.push(make_fm("a.rs", 2));
        state.results.push(make_fm("b.rs", 1));
        state.selected = Some((1, 0));
        state.select_prev();
        assert_eq!(state.selected, Some((0, 1)));
        state.select_prev();
        assert_eq!(state.selected, Some((0, 0)));
    }

    // Validates: Requirement 2.6 -- invalid regex returns error
    #[test]
    fn build_request_returns_error_for_invalid_regex() {
        let mut state = SearchResultsPanelState::new();
        state.query = "[invalid".to_string();
        state.options.use_regex = true;
        let result = state.build_request(vec![]);
        assert!(result.is_err());
        assert!(state.regex_error.is_some());
    }

    // Validates: Requirement 2.5 -- valid regex builds request
    #[test]
    fn build_request_succeeds_for_valid_regex() {
        let mut state = SearchResultsPanelState::new();
        state.query = r"\d+".to_string();
        state.options.use_regex = true;
        let result = state.build_request(vec!["/some/root".to_string()]);
        assert!(result.is_ok());
        assert!(state.regex_error.is_none());
    }
}
