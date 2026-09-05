# Design -- Global Search

## Architectural Decisions

### 1. New crate: ff-global-search

Global Search introduces a new library crate `ff-global-search` that owns the search
orchestration logic: file enumeration, per-file search delegation, result aggregation,
and the replace pipeline. It depends on `ff-find-and-replace` (reuses FindEngine),
`ff-bgio` (background execution), and `ff-file-ops` (file read/write).

The Search Results panel UI lives in `ff-desktop`.

### 2. Background execution via ff-bgio

Search runs as a Tokio task spawned via `ff-bgio`. Results are streamed back to the UI
via an `mpsc::channel` -- the panel polls the receiver each frame and appends new matches
to the displayed list. This keeps the egui render loop unblocked.

### 3. File enumeration

File enumeration uses the `ignore` crate (already a transitive dependency via `ff-bgio`)
which respects `.gitignore` and supports glob include/exclude patterns natively.

### 4. Per-file search

Each file is read into a `String` buffer and passed to `FindEngine::find_all()` from
`ff-find-and-replace`. This reuses all existing literal/regex/case-fold logic without
duplication.

### 5. Replace pipeline

Cross-file replace reads each file, applies substitutions via `FindEngine::replace_all()`,
and writes the result via `ff-file-ops`. Each file write is wrapped in a single undo
transaction for that file's editor tab (if open). Files not open in an editor are written
directly to disk with no undo record (the user can re-run the search to verify).

### 6. Search Results panel

A new `TabKind::SearchResults` is added to `ff-desktop`. The panel state holds:
- The current query and options
- A `Vec<FileMatches>` (file path + `Vec<SearchResult>`)
- A `Receiver<SearchEvent>` for streaming results from the background task
- Replace input state

## New Crate

```
crates/ff-global-search/
  Cargo.toml   -- deps: ff-find-and-replace, ff-bgio, ff-file-ops, ignore, tokio
  src/
    lib.rs
    search.rs  -- GlobalSearchRequest, GlobalSearchEngine, search()
    replace.rs -- GlobalReplaceEngine, replace_all()
    result.rs  -- SearchResult, FileMatches, SearchEvent
```

## Module Layout (ff-desktop)

```
ff-desktop/src/
  search_results_panel/
    mod.rs     -- SearchResultsPanelState
    render.rs  -- render(), render_file_section(), render_match_row()
    state.rs   -- query, options, results, receiver
```
