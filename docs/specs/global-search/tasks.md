# Tasks -- Global Search

## Task 1. ff-global-search crate scaffold (Req 1, 2, 3)

- [x] 1.1 Create `crates/ff-global-search/Cargo.toml` with deps: `ff-find-and-replace`,
  `ff-bgio`, `ff-file-ops`, `ignore`, `tokio`, `thiserror`
  - Satisfies: Req 3.1
- [x] 1.2 Define `GlobalSearchRequest` struct: query, mode (Literal/Regex), case_sensitive,
  whole_word, include_globs, exclude_globs, roots
  - Satisfies: Req 2.1, 2.2, 2.3, 2.5
- [x] 1.3 Define `SearchResult` struct: file_path, line_number, col_start, col_end, line_text
  - Satisfies: Req 4.2
- [x] 1.4 Define `FileMatches` struct: file_path, matches: Vec<SearchResult>
  - Satisfies: Req 4.1
- [x] 1.5 Define `SearchEvent` enum: `MatchFound(FileMatches)`, `Progress { files_scanned, matches_found }`,
  `Completed { total_files, total_matches }`, `Cancelled`
  - Satisfies: Req 3.2, 3.3, 3.7

## Task 2. Search engine implementation (Req 2, 3)

- [x] 2.1 Implement file enumeration using `ignore::WalkBuilder` with include/exclude glob support;
  skip binary files (null byte in first 8 KB)
  - Satisfies: Req 3.5, 3.6, 2.2, 2.3
- [x] 2.2 Implement per-file search: read file to String, delegate to `FindEngine::find_all()`,
  collect `SearchResult` entries
  - Satisfies: Req 2.5, 3.3
- [x] 2.3 Implement `GlobalSearchEngine::search()` as async function: enumerate files, search each,
  send `SearchEvent` messages via `mpsc::Sender`
  - Satisfies: Req 3.1, 3.3
- [x] 2.4 Implement cancellation via `CancellationToken`; send `SearchEvent::Cancelled` on abort
  - Satisfies: Req 3.4
- [x] 2.5 Write unit tests: literal search finds matches, regex search, binary file skipped,
  exclude glob respected, cancellation stops enumeration
  - Satisfies: Req 2.5, 3.4, 3.5, 3.6

## Task 3. Replace engine implementation (Req 5)

- [x] 3.1 Implement `GlobalReplaceEngine::replace_all()`: for each file with matches, read content,
  apply `FindEngine::replace_all()`, write via `ff-file-ops`
  - Satisfies: Req 5.3
- [x] 3.2 Check for open editor tabs with unsaved changes before replacing; return conflict list
  - Satisfies: Req 5.6
- [x] 3.3 Write unit tests: replace modifies file content, unsaved-changes conflict detected
  - Satisfies: Req 5.3, 5.6

## Task 4. Search Results panel -- state and rendering (Req 1, 4)

- [x] 4.1 Add `TabKind::SearchResults` variant to tab kind enum in `ff-desktop`
  - Satisfies: Req 1.1
- [x] 4.2 Create `SearchResultsPanelState`: query, options, results: Vec<FileMatches>,
  receiver: Option<Receiver<SearchEvent>>, replace_input, is_searching
  - Satisfies: Req 4.1
- [x] 4.3 Implement `render_search_results_panel()`: search input, option toggles, include/exclude
  fields, results list grouped by file with collapsible sections
  - Satisfies: Req 2.1, 4.1, 4.2, 4.4
- [x] 4.4 Render progress indicator while search is running; render summary on completion
  - Satisfies: Req 3.2, 3.7
- [x] 4.5 Render match rows with line number and highlighted match text; click opens file at line
  - Satisfies: Req 4.2, 4.3
- [x] 4.6 Implement keyboard navigation: Up/Down between rows, Enter opens match, Left/Right
  collapses/expands file sections
  - Satisfies: Req 4.5
- [x] 4.7 Poll `SearchEvent` receiver each frame; append new matches to results list
  - Satisfies: Req 3.3

## Task 5. Replace UI and activation (Req 1, 5, 6)

- [x] 5.1 Add expandable replace input field and Replace All / per-file Replace buttons
  - Satisfies: Req 5.1
- [x] 5.2 Implement Replace_Preview confirmation dialog showing file/match counts
  - Satisfies: Req 5.2
- [x] 5.3 Wire Replace All: spawn replace task via ff-bgio, show summary on completion
  - Satisfies: Req 5.3, 5.4
- [x] 5.4 Wire Ctrl+Shift+F activation and `GSEARCH`/`SEARCH` command routing
  - Satisfies: Req 1.1, 1.2
- [x] 5.5 Add `Search > Find in Files` menu item
  - Satisfies: Req 1.4
- [x] 5.6 Implement search history dropdown (last 20 queries, persisted in session)
  - Satisfies: Req 6.1, 6.2, 6.3
- [x] 5.7 Write integration tests: search finds matches across multiple files, replace modifies files
  - Satisfies: Req 3.7, 5.4
