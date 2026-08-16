//! Search types and fallback search implementation for the VFS abstraction layer.
//!
//! Defines `SearchQuery`, `SearchOptions`, and `VfsSearchResult` used by the
//! provider search method and the VFS search facade.
//!
//! Also provides [`fallback_search`] — a generic search implementation that
//! enumerates files via `list`, reads content via `read_stream`, and matches
//! line-by-line. Used when a provider lacks native search capability.

use std::pin::Pin;
use std::sync::Arc;

use futures_core::Stream;
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;

use crate::provider::VfsProvider;
use crate::types::VfsEntryType;
use crate::uri::ResourceUri;

/// A search query specifying what to look for.
///
/// Addresses: Requirement 8 AC 1
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SearchQuery {
    /// Search for content within files matching the given pattern.
    Content(String),
    /// Search for filenames matching the given pattern.
    Filename(String),
}

/// Options controlling search behaviour.
///
/// Addresses: Requirement 8 AC 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOptions {
    /// Whether the search is case-sensitive.
    pub case_sensitive: bool,
    /// Whether to match whole words only.
    pub whole_word: bool,
    /// Maximum number of results to return (0 = unlimited).
    pub max_results: usize,
    /// Whether to search recursively into subdirectories.
    pub recursive: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            max_results: 0,
            recursive: true,
        }
    }
}

/// A single search result returned by the provider.
///
/// Addresses: Requirement 8 AC 3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VfsSearchResult {
    /// The URI of the resource where the match was found.
    pub uri: ResourceUri,
    /// The 1-based line number where the match occurred (if applicable).
    pub line: Option<u32>,
    /// The 0-based column offset where the match starts (if applicable).
    pub column: Option<u32>,
    /// A preview of the matched content (the line or filename).
    pub preview: String,
}

/// Perform a fallback search by enumerating files via `list` and reading
/// content via `read_stream`, matching line-by-line.
///
/// Used when a provider does not have native search capability.
/// Results are emitted as an async stream. The search respects the
/// `CancellationToken` and stops producing results when cancelled.
///
/// Addresses: Requirement 8 AC 4, AC 5
pub async fn fallback_search(
    provider: Arc<dyn VfsProvider>,
    root_path: &str,
    root_scheme: &str,
    query: &SearchQuery,
    options: &SearchOptions,
    cancel_token: CancellationToken,
) -> Pin<Box<dyn Stream<Item = VfsSearchResult> + Send>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<VfsSearchResult>(64);

    let root_path = root_path.to_string();
    let root_scheme = root_scheme.to_string();
    let query = query.clone();
    let options = options.clone();

    tokio::spawn(async move {
        let mut result_count: usize = 0;
        let max = options.max_results;

        // Collect all file paths recursively
        let mut file_paths: Vec<String> = Vec::new();
        let mut dirs_to_visit = vec![root_path.clone()];

        while let Some(dir) = dirs_to_visit.pop() {
            if cancel_token.is_cancelled() {
                return;
            }

            let entries = match provider.list(&dir).await {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries {
                if cancel_token.is_cancelled() {
                    return;
                }

                let child_path = if dir.ends_with('/') {
                    format!("{}{}", dir, entry.name)
                } else {
                    format!("{}/{}", dir, entry.name)
                };

                match entry.entry_type {
                    VfsEntryType::Directory => {
                        if options.recursive {
                            dirs_to_visit.push(child_path);
                        }
                    }
                    VfsEntryType::File => {
                        file_paths.push(child_path);
                    }
                    _ => {}
                }
            }
        }

        // Process each file
        for file_path in file_paths {
            if cancel_token.is_cancelled() {
                return;
            }
            if max > 0 && result_count >= max {
                return;
            }

            match &query {
                SearchQuery::Filename(pattern) => {
                    let filename = file_path.rsplit('/').next().unwrap_or(&file_path);

                    if matches_pattern(filename, pattern, &options) {
                        let uri = ResourceUri::new(&root_scheme, &file_path);
                        let result = VfsSearchResult {
                            uri,
                            line: None,
                            column: None,
                            preview: filename.to_string(),
                        };
                        result_count += 1;
                        if tx.send(result).await.is_err() {
                            return;
                        }
                        if max > 0 && result_count >= max {
                            return;
                        }
                    }
                }
                SearchQuery::Content(pattern) => {
                    // Read the file content
                    let mut reader = match provider.read_stream(&file_path).await {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                    let mut content = Vec::new();
                    if reader.read_to_end(&mut content).await.is_err() {
                        continue;
                    }

                    let text = match String::from_utf8(content) {
                        Ok(t) => t,
                        Err(_) => continue, // skip binary files
                    };

                    for (line_idx, line_content) in text.lines().enumerate() {
                        if cancel_token.is_cancelled() {
                            return;
                        }
                        if max > 0 && result_count >= max {
                            return;
                        }

                        if let Some(col) = find_match(line_content, pattern, &options) {
                            let uri = ResourceUri::new(&root_scheme, &file_path);
                            let result = VfsSearchResult {
                                uri,
                                line: Some((line_idx + 1) as u32),
                                column: Some(col as u32),
                                preview: line_content.to_string(),
                            };
                            result_count += 1;
                            if tx.send(result).await.is_err() {
                                return;
                            }
                        }
                    }
                }
            }
        }
    });

    Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
}

/// Check if a filename matches the given pattern, respecting search options.
fn matches_pattern(text: &str, pattern: &str, options: &SearchOptions) -> bool {
    if options.whole_word {
        if options.case_sensitive {
            text == pattern
        } else {
            text.eq_ignore_ascii_case(pattern)
        }
    } else if options.case_sensitive {
        text.contains(pattern)
    } else {
        text.to_ascii_lowercase()
            .contains(&pattern.to_ascii_lowercase())
    }
}

/// Find the first match of `pattern` in `line`, returning the column offset if found.
/// Respects case sensitivity and whole-word options.
fn find_match(line: &str, pattern: &str, options: &SearchOptions) -> Option<usize> {
    if options.whole_word {
        find_whole_word_match(line, pattern, options.case_sensitive)
    } else if options.case_sensitive {
        line.find(pattern)
    } else {
        line.to_ascii_lowercase()
            .find(&pattern.to_ascii_lowercase())
    }
}

/// Find a whole-word match of `pattern` in `line`.
/// A word boundary is a transition between alphanumeric/underscore and non-alphanumeric chars.
fn find_whole_word_match(line: &str, pattern: &str, case_sensitive: bool) -> Option<usize> {
    let line_bytes = if case_sensitive {
        line.to_string()
    } else {
        line.to_ascii_lowercase()
    };
    let pat = if case_sensitive {
        pattern.to_string()
    } else {
        pattern.to_ascii_lowercase()
    };

    let mut start = 0;
    while start + pat.len() <= line_bytes.len() {
        if let Some(pos) = line_bytes[start..].find(&pat) {
            let abs_pos = start + pos;
            let before_ok = abs_pos == 0
                || !line_bytes.as_bytes()[abs_pos - 1].is_ascii_alphanumeric()
                    && line_bytes.as_bytes()[abs_pos - 1] != b'_';
            let after_pos = abs_pos + pat.len();
            let after_ok = after_pos >= line_bytes.len()
                || !line_bytes.as_bytes()[after_pos].is_ascii_alphanumeric()
                    && line_bytes.as_bytes()[after_pos] != b'_';

            if before_ok && after_ok {
                return Some(abs_pos);
            }
            start = abs_pos + 1;
        } else {
            break;
        }
    }
    None
}
