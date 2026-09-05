//! Result types for global search operations.
//!
//! Addresses: Requirement 3.2, 3.3, 3.7, 4.1, 4.2

/// A single match within a file.
///
/// Addresses: Requirement 4.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// 1-based line number of the match.
    pub line_number: u64,
    /// 0-based byte column of the match start within the line.
    pub col_start: usize,
    /// 0-based byte column of the match end (exclusive) within the line.
    pub col_end: usize,
    /// The full text of the matching line (without trailing newline).
    pub line_text: String,
}

/// All matches found within a single file.
///
/// Addresses: Requirement 4.1
#[derive(Debug, Clone)]
pub struct FileMatches {
    /// Absolute path to the file.
    pub file_path: String,
    /// All matches within this file, in line order.
    pub matches: Vec<SearchResult>,
}

/// Events streamed from the background search task to the UI.
///
/// Addresses: Requirement 3.2, 3.3, 3.7
#[derive(Debug)]
pub enum SearchEvent {
    /// One file's worth of matches found.
    MatchFound(FileMatches),
    /// Periodic progress update.
    Progress {
        files_scanned: u64,
        matches_found: u64,
    },
    /// Search completed normally.
    Completed {
        total_files: u64,
        total_matches: u64,
    },
    /// Search was cancelled by the user.
    Cancelled,
}
