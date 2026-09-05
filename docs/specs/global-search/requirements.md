# Requirements Document -- Global Search

## Introduction

This spec defines Global Search (cross-file search and replace) for FileForgeWorkbench.
Global Search allows the user to search for text or regex patterns across all files in the
active workspace roots or selected catalog paths, view results in a dedicated Search Results
panel, and perform cross-file replace operations with preview and undo support.

Global Search is the most significant functional gap relative to comparable editors. It is
implemented as a new `ff-global-search` library crate plus a Search Results panel in
`ff-desktop`. It reuses the existing `ff-find-and-replace` FindEngine for per-file matching.

**Source references:**
- **WB** = Workbench Architecture Brief
- **GAP** = Phase BQ gap-analysis.md section 3.3 (Workspace Search, High priority)
- **EXEC** = Phase BQ executive-assessment.md Recommendation 3
- **FFE** = find-and-replace spec (reuses FindEngine, SearchMode, FindRequest)

## Glossary

- **Global_Search**: A search operation that scans multiple files across workspace roots
  or catalog paths for a text or regex pattern.
- **Search_Scope**: The set of files to be searched, defined by root paths and optional
  include/exclude glob patterns.
- **Search_Result**: A single match: file path, line number, column range, and the
  matching line text with the match highlighted.
- **Search_Results_Panel**: The dedicated panel (tab kind `SearchResults`) that displays
  all matches grouped by file.
- **Replace_Preview**: A diff-style view showing the proposed replacements before they
  are applied to disk.
- **Global_Replace**: A cross-file replace operation that applies a substitution to all
  or selected matches across multiple files.

---

## Requirements

### Requirement 1: Search Activation

**User Story:** As a developer, I want to open a global search panel quickly so that I can
start searching across files without navigating menus.

**Source:** GAP 3.3

#### Acceptance Criteria

1. WHEN the user presses Ctrl+Shift+F from any context, THE workbench SHALL open the
   Search Results panel as a tab (TabKind::SearchResults) with the search input focused.

2. WHEN the user types `GSEARCH` or `SEARCH` in the `Command ===>` field, THE workbench
   SHALL open the Search Results panel.

3. WHEN the Search Results panel is already open, Ctrl+Shift+F SHALL focus the search
   input field in the existing panel rather than opening a duplicate tab.

4. THE Search Results panel SHALL be accessible from the menu bar via `Search > Find in Files`.

---

### Requirement 2: Search Input and Options

**User Story:** As a developer, I want to specify a search pattern, scope, and options
(case sensitivity, whole word, regex) so that I can precisely control what is searched.

**Source:** GAP 3.3, FFE

#### Acceptance Criteria

1. THE Search Results panel SHALL provide a search input field for the query text, a
   replace input field (collapsed by default, expandable), and the following option
   toggles: Case Sensitive (default: off), Whole Word (default: off), Use Regex (default: off).

2. THE Search Results panel SHALL provide an Include Files field accepting glob patterns
   (e.g., `**/*.rs`, `src/**`) to restrict the search to matching file paths; an empty
   field means search all files.

3. THE Search Results panel SHALL provide an Exclude Files field accepting glob patterns
   to exclude matching file paths from the search (e.g., `**/target/**`, `**/*.lock`).

4. WHEN a workspace is active, THE default Search_Scope SHALL be all files under all
   Workspace_Roots; WHEN no workspace is active, THE default scope SHALL be all files
   under all mounted Native catalogs.

5. THE search query SHALL support the same literal, whole-word, and regex modes as the
   per-file FindEngine (reusing `ff-find-and-replace` SearchMode and matching logic).

6. WHEN Use Regex is enabled and the query is an invalid regex pattern, THE panel SHALL
   display an inline error message adjacent to the search field and SHALL NOT execute
   the search.

---

### Requirement 3: Search Execution

**User Story:** As a developer, I want search to run in the background so that the UI
remains responsive while large codebases are being scanned.

**Source:** GAP 3.3, WB

#### Acceptance Criteria

1. WHEN the user presses Enter in the search field or clicks the Search button, THE
   workbench SHALL start the global search as a background Tokio task via `ff-bgio`,
   keeping the UI fully interactive during the search.

2. WHILE a search is running, THE Search Results panel SHALL display a progress indicator
   showing the number of files scanned and the number of matches found so far.

3. THE search SHALL stream results to the panel incrementally -- matches SHALL appear in
   the panel as each file is scanned, not only after all files are complete.

4. WHEN the user clicks a Cancel button while a search is running, THE workbench SHALL
   abort the background search task and display the partial results found so far.

5. THE search SHALL skip binary files (files where the first 8 KB contains a null byte)
   and SHALL display a "N binary files skipped" count in the panel footer.

6. THE search SHALL respect the Exclude Files glob patterns and SHALL NOT open or scan
   excluded files.

7. WHEN a search completes, THE panel SHALL display the total match count and file count
   in the format "N matches in M files".

---

### Requirement 4: Search Results Display

**User Story:** As a developer, I want search results grouped by file with expandable
sections so that I can quickly navigate to any match.

**Source:** GAP 3.3

#### Acceptance Criteria

1. THE Search Results panel SHALL display results grouped by file path, with each file
   shown as a collapsible section header displaying the file name, relative path, and
   match count for that file.

2. EACH match within a file section SHALL display: the line number (right-aligned), and
   the matching line text with the matched portion visually highlighted.

3. WHEN the user clicks a match row, THE workbench SHALL open the file in an editor tab
   (or focus the existing tab if already open), scroll to the matching line, and highlight
   the match using the text-decorations system.

4. ALL file sections SHALL be expanded by default; the user SHALL be able to collapse
   individual file sections by clicking the section header.

5. THE Search Results panel SHALL support keyboard navigation: Up/Down arrows move between
   match rows; Enter opens the selected match; Left/Right arrows collapse/expand file sections.

6. WHEN a new search is executed, THE previous results SHALL be cleared and replaced with
   the new results.

---

### Requirement 5: Cross-File Replace

**User Story:** As a developer, I want to replace a pattern across multiple files with a
preview before committing, so that I can safely perform large-scale refactoring.

**Source:** GAP 3.3

#### Acceptance Criteria

1. WHEN the user expands the replace input field and enters a replacement string, THE
   Search Results panel SHALL display a Replace All button and per-file Replace buttons.

2. WHEN the user clicks Replace All, THE workbench SHALL display a Replace_Preview showing
   the number of files and matches that will be modified, and SHALL ask for confirmation
   before writing any changes to disk.

3. WHEN the user confirms Replace All, THE workbench SHALL apply the replacement to all
   matches across all files, writing each modified file to disk via the file-operations
   pipeline.

4. WHEN a cross-file replace completes, THE workbench SHALL display a summary: "Replaced
   N occurrences in M files".

5. THE cross-file replace operation SHALL be undoable as a single undo transaction per
   file -- undoing the replace in an open editor tab SHALL revert that file's changes.

6. WHEN a file to be replaced is currently open in an editor tab with unsaved changes,
   THE workbench SHALL warn the user and SHALL NOT overwrite the file until the user
   saves or discards the in-editor changes.

7. THE replace input SHALL support the same regex group substitution syntax as the
   per-file CHANGE command (`\1` through `\9` for captured groups).

---

### Requirement 6: Search History

**User Story:** As a developer, I want my recent search queries remembered so that I can
quickly re-run previous searches without retyping.

**Source:** GAP 3.3

#### Acceptance Criteria

1. THE Search Results panel SHALL maintain a history of the last 20 search queries,
   accessible via a dropdown on the search input field.

2. THE search history SHALL be persisted in the session state and restored on next launch.

3. WHEN the user selects a previous query from the history dropdown, THE search field
   SHALL be populated with that query and its associated options (case sensitivity, regex,
   include/exclude patterns).
