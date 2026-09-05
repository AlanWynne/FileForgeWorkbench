//! Global search engine -- file enumeration and per-file search delegation.
//!
//! Addresses: Requirement 2, 3

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ff_find_and_replace::indexer::SliceIndexer;
use ff_find_and_replace::scope::AllLinesFilter;
use ff_find_and_replace::types::BytePosition;
use ff_find_and_replace::CharacterIndexer;
use ff_find_and_replace::{FindEngine, FindOutcome, FindRequest, SearchMode, WordMatchMode};
use tokio::sync::mpsc;

use crate::error::GlobalSearchError;
use crate::result::{FileMatches, SearchEvent, SearchResult};

/// Options controlling what and how to search.
///
/// Addresses: Requirement 2.1, 2.2, 2.3, 2.5
#[derive(Debug, Clone)]
pub struct GlobalSearchRequest {
    /// The search query text.
    pub query: String,
    /// Literal, Regex, or WholeWord.
    pub mode: SearchMode,
    /// Whether the search is case-sensitive.
    pub case_sensitive: bool,
    /// Whether to match whole words only.
    pub whole_word: bool,
    /// Glob patterns to include (empty = all files).
    pub include_globs: Vec<String>,
    /// Glob patterns to exclude.
    pub exclude_globs: Vec<String>,
    /// Root directories to search.
    pub roots: Vec<String>,
}

impl GlobalSearchRequest {
    /// Create a simple literal search request over the given roots.
    pub fn literal(query: &str, roots: Vec<String>) -> Self {
        Self {
            query: query.to_string(),
            mode: SearchMode::Literal,
            case_sensitive: false,
            whole_word: false,
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            roots,
        }
    }
}

/// Cancellation token for a running search.
///
/// Addresses: Requirement 3.4
#[derive(Clone)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    /// Create a new, uncancelled token.
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Signal cancellation.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Returns true if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestrates global search across multiple files.
pub struct GlobalSearchEngine;

impl GlobalSearchEngine {
    /// Run a global search, streaming `SearchEvent`s via `tx`.
    ///
    /// Designed to be spawned as a Tokio task.
    ///
    /// Addresses: Requirement 3.1, 3.3, 3.4, 3.5, 3.6
    pub async fn search(
        request: GlobalSearchRequest,
        tx: mpsc::Sender<SearchEvent>,
        cancel: CancellationToken,
    ) -> Result<(), GlobalSearchError> {
        if request.query.is_empty() {
            let _ = tx
                .send(SearchEvent::Completed {
                    total_files: 0,
                    total_matches: 0,
                })
                .await;
            return Ok(());
        }

        let mut files_scanned: u64 = 0;
        let mut total_matches: u64 = 0;
        let mut binary_skipped: u64 = 0;

        for root in &request.roots {
            if cancel.is_cancelled() {
                let _ = tx.send(SearchEvent::Cancelled).await;
                return Ok(());
            }

            let mut walker = ignore::WalkBuilder::new(root);
            walker.hidden(false);

            // Apply exclude globs.
            for pat in &request.exclude_globs {
                let mut override_builder = ignore::overrides::OverrideBuilder::new(root);
                let _ = override_builder.add(&format!("!{pat}"));
                if let Ok(ov) = override_builder.build() {
                    walker.overrides(ov);
                }
            }

            for entry in walker.build() {
                if cancel.is_cancelled() {
                    let _ = tx.send(SearchEvent::Cancelled).await;
                    return Ok(());
                }

                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    continue;
                }

                let path = entry.path().to_string_lossy().into_owned();

                // Apply include globs if specified.
                if !request.include_globs.is_empty() {
                    let file_name = entry
                        .path()
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let matches_include = request
                        .include_globs
                        .iter()
                        .any(|pat| glob_match(pat, &file_name) || glob_match(pat, &path));
                    if !matches_include {
                        continue;
                    }
                }

                // Read file content.
                let content = match std::fs::read(&path) {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                // Skip binary files -- Addresses: Requirement 3.5
                if is_binary(&content) {
                    binary_skipped += 1;
                    continue;
                }

                let text = match String::from_utf8(content) {
                    Ok(s) => s,
                    Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
                };

                files_scanned += 1;

                let matches = search_in_text(&text, &request);
                if !matches.is_empty() {
                    let count = matches.len() as u64;
                    total_matches += count;
                    let fm = FileMatches {
                        file_path: path,
                        matches,
                    };
                    if tx.send(SearchEvent::MatchFound(fm)).await.is_err() {
                        return Ok(());
                    }
                }

                // Periodic progress -- Addresses: Requirement 3.2
                if files_scanned.is_multiple_of(50) {
                    let _ = tx
                        .send(SearchEvent::Progress {
                            files_scanned,
                            matches_found: total_matches,
                        })
                        .await;
                }
            }
        }

        let _ = tx
            .send(SearchEvent::Completed {
                total_files: files_scanned + binary_skipped,
                total_matches,
            })
            .await;
        Ok(())
    }
}

/// Search for all matches of `request` within `text`.
///
/// Returns one `SearchResult` per match.
pub(crate) fn search_in_text(text: &str, request: &GlobalSearchRequest) -> Vec<SearchResult> {
    let mut engine = FindEngine::new();
    let filter = AllLinesFilter;
    let mut results = Vec::new();

    let word_match = if request.whole_word {
        WordMatchMode::WholeWord
    } else {
        WordMatchMode::None
    };

    let mut cursor = BytePosition::ZERO;
    let indexer = SliceIndexer::from_str(text);
    let end = BytePosition(indexer.length());

    loop {
        if cursor >= end {
            break;
        }
        let req = FindRequest {
            term: request.query.clone(),
            mode: request.mode,
            direction: ff_find_and_replace::SearchDirection::Next,
            scope: ff_find_and_replace::scope::ScopeModifier::All,
            case_sensitive: request.case_sensitive,
            word_match,
            column_range: None,
            cursor_position: cursor,
        };
        match engine.find(&req, &indexer, &filter, None) {
            Ok(FindOutcome::Found(r)) => {
                let match_start = r.match_range.start.0 as usize;
                let match_end = r.match_range.end.0 as usize;
                // Determine line number and column.
                let before = &text[..match_start];
                let line_number = before.bytes().filter(|&b| b == b'\n').count() as u64 + 1;
                let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
                let line_end = text[match_start..]
                    .find('\n')
                    .map(|p| match_start + p)
                    .unwrap_or(text.len());
                let line_text = text[line_start..line_end]
                    .trim_end_matches('\r')
                    .to_string();
                let col_start = match_start - line_start;
                let col_end = (match_end - line_start).min(line_text.len());
                results.push(SearchResult {
                    line_number,
                    col_start,
                    col_end,
                    line_text,
                });
                // Advance past this match; avoid infinite loop on zero-length match.
                let next = if match_end > match_start.max(cursor.0 as usize) {
                    match_end
                } else {
                    match_start + 1
                };
                cursor = BytePosition(next as u64);
            }
            _ => break,
        }
    }
    results
}

/// Returns true if the first 8 KB of `data` contains a null byte.
///
/// Addresses: Requirement 3.5
pub(crate) fn is_binary(data: &[u8]) -> bool {
    data.iter().take(8192).any(|&b| b == 0)
}

/// Simple glob match: supports `*` (any chars) and `?` (one char).
fn glob_match(pattern: &str, text: &str) -> bool {
    // Delegate to the `ignore` crate's glob via regex for correctness.
    // Build a simple regex from the glob pattern.
    let mut regex_pat = String::from("(?i)^");
    for ch in pattern.chars() {
        match ch {
            '*' => regex_pat.push_str(".*"),
            '?' => regex_pat.push('.'),
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => {
                regex_pat.push('\\');
                regex_pat.push(ch);
            }
            c => regex_pat.push(c),
        }
    }
    regex_pat.push('$');
    regex::Regex::new(&regex_pat)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(query: &str) -> GlobalSearchRequest {
        GlobalSearchRequest {
            query: query.to_string(),
            mode: SearchMode::Literal,
            case_sensitive: false,
            whole_word: false,
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            roots: Vec::new(),
        }
    }

    // Validates: Requirement 2.5 -- literal search finds all matches
    #[test]
    fn literal_search_finds_all_matches() {
        let text = "foo bar foo baz foo";
        let results = search_in_text(text, &req("foo"));
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[0].col_start, 0);
        assert_eq!(results[1].col_start, 8);
        assert_eq!(results[2].col_start, 16);
    }

    // Validates: Requirement 2.5 -- case-insensitive search
    #[test]
    fn case_insensitive_search_matches_mixed_case() {
        let text = "Hello HELLO hello";
        let results = search_in_text(text, &req("hello"));
        assert_eq!(results.len(), 3);
    }

    // Validates: Requirement 2.5 -- multiline search returns correct line numbers
    #[test]
    fn multiline_search_returns_correct_line_numbers() {
        let text = "alpha\nbeta\nalpha\n";
        let results = search_in_text(text, &req("alpha"));
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);
    }

    // Validates: Requirement 2.5 -- no match returns empty
    #[test]
    fn no_match_returns_empty_vec() {
        let text = "hello world";
        let results = search_in_text(text, &req("xyz"));
        assert!(results.is_empty());
    }

    // Validates: Requirement 3.5 -- binary detection
    #[test]
    fn binary_detection_triggers_on_null_byte() {
        assert!(is_binary(b"hello\x00world"));
        assert!(!is_binary(b"hello world"));
    }

    // Validates: Requirement 3.5 -- binary detection only checks first 8 KB
    #[test]
    fn binary_detection_only_checks_first_8kb() {
        let mut data = vec![b'a'; 8193];
        data[8192] = 0; // null byte beyond 8 KB window
        assert!(!is_binary(&data));
    }

    // Validates: Requirement 2.5 -- whole-word matching
    #[test]
    fn whole_word_search_does_not_match_substrings() {
        let text = "foobar foo foobaz";
        let mut r = req("foo");
        r.whole_word = true;
        let results = search_in_text(text, &r);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].col_start, 7);
    }

    // Validates: Requirement 3.4 -- cancellation token
    #[test]
    fn cancellation_token_starts_uncancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    // Validates: Requirement 3.1 -- empty query completes immediately
    #[tokio::test]
    async fn empty_query_completes_immediately() {
        let (tx, mut rx) = mpsc::channel(16);
        let req = GlobalSearchRequest::literal("", vec![]);
        let cancel = CancellationToken::new();
        GlobalSearchEngine::search(req, tx, cancel).await.unwrap();
        let event = rx.recv().await.unwrap();
        assert!(matches!(
            event,
            SearchEvent::Completed {
                total_files: 0,
                total_matches: 0
            }
        ));
    }
}
