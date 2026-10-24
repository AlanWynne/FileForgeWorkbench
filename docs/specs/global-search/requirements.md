# Requirements Document -- Global Search

## Introduction

This spec defines Global Search (cross-file search and replace) for FileForgeWorkbench.
Global Search allows the user to search for text or regex patterns across all files in the
active workspace roots or selected catalog paths, view results in a dedicated Search Results
panel, and perform cross-file replace operations with preview and undo support.

Global Search is the most significant functional gap relative to comparable editors. It is
implemented as a new `ff-global-search` library crate plus a Search Results panel in
`ff-desktop`. It reuses the existing `ff-find-and-replace` FindEngine for per-file matching.

### Scope note -- SEARCH as the FFCMD canonical multi-file verb (CR-NR-110)

CR-NR-110 draws a firm line between two string-search commands that live in two
different Command Environments (see `docs/specs/command-environments/`):

- FIND is an FFEDIT (editor) command -- it searches for a string WITHIN the current
  editor buffer. Its behaviour is owned by `docs/specs/find-and-replace/`; it is NOT a
  workbench/global verb.
- SEARCH is the FFCMD (workbench) command this spec owns -- it searches for a string
  across a SET OF FILES spanning ALL VFS backends (mainframe datasets, POSIX files,
  NTFS files), routed through FFWB's own VFS / catalog abstraction, and surfaces results
  in the existing Search Results Context.

Requirements 1 to 6 above define the Search Results Context, its panel, background
execution, result display, cross-file replace, and history. Requirements 7 to 9 (added
by CR-NR-110) formalise SEARCH as the FFCMD CANONICAL ROOT verb, pin GREP as an alias
surface for it, define the cross-backend search as a behaviour CONTRACT, and record what
is DEFERRED. The earlier `GSEARCH`/`SEARCH` activation criterion (Requirement 1.2) is the
activation seam; CR-NR-110 makes SEARCH the canonical identifier and GREP its alias, and
makes the cross-backend scope explicit.

The concrete internal search ENGINE (whether and how a grep engine is embedded for the
POSIX/NTFS backends) and the mainframe DATASET traversal / record-search semantics are
DEFERRED until the Volume Model (CR-NR-105) and the dataset-catalog resolution mechanisms
are finalised. CR-NR-110 specifies the VERB SPLIT, the environment ownership, and the
user-facing cross-backend CONTRACT -- NOT the engine. See Requirement 9 and the design
Deferred section.

**Source references:**
- **WB** = Workbench Architecture Brief
- **GAP** = Phase BQ gap-analysis.md section 3.3 (Workspace Search, High priority)
- **EXEC** = Phase BQ executive-assessment.md Recommendation 3
- **FFE** = find-and-replace spec (reuses FindEngine, SearchMode, FindRequest)
- **CR-NR-110** = the FIND-vs-SEARCH verb split: FIND = FFEDIT in-buffer, SEARCH = FFCMD
  cross-backend canonical root (GREP alias). Owner decision, this gate.

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
- **SEARCH_Verb**: The FFCMD canonical ROOT verb for cross-backend multi-file string
  search. It is the identifier that dispatches, records, and persists; GREP is an alias
  surface that resolves to it. [CR-NR-110]
- **GREP_Alias**: An additional surface form (`grep`) that resolves to the canonical
  SEARCH_Verb through the existing FFCMD AliasTable; it is command-line input convenience
  only and carries no behaviour of its own. [CR-NR-110]
- **VFS_Backend**: One of the storage backends reachable through FFWB's VFS / catalog
  abstraction -- a mainframe catalog/dataset backend, a POSIX filesystem, or an NTFS
  filesystem. SEARCH's scope is a set of files spanning these backends. [CR-NR-110]
- **Search_Engine**: The internal mechanism that performs the byte/line matching for a
  SEARCH across a backend (potentially an embedded grep engine for POSIX/NTFS). Its
  concrete form is DEFERRED by CR-NR-110 pending CR-NR-105 and dataset-catalog. [CR-NR-110]

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

---

### Requirement 7: SEARCH as the FFCMD Canonical Root Verb (CR-NR-110)

**User Story:** As a workbench user, I want SEARCH to be the one canonical workbench
command for finding a string across many files, resolved through the same single
command-dispatch front door as every other workbench verb, so that it behaves
consistently with the rest of the command system and is distinct from the editor's FIND.

**Source:** CR-NR-110

#### Acceptance Criteria

1. THE SEARCH_Verb SHALL be an FFCMD (workbench) command, owned by the FFCMD base
   environment, and SHALL be resolved through the single command-dispatch front door
   (`resolve_target` / `dispatch_command_target`); it SHALL NOT introduce a second
   dispatcher or a parallel command path.

2. THE SEARCH_Verb SHALL be the CANONICAL ROOT verb for cross-backend multi-file string
   search: it is the identifier that dispatches, is recorded in command history, and is
   persisted. WHEN any alias surface (Requirement 8) is typed, THE recorded and persisted
   verb SHALL be the canonical SEARCH, never the alias.

3. WHEN the user types `SEARCH <string>` in the `Command ===>` field from any context, THE
   workbench SHALL activate cross-backend search (Requirement 9) and surface results in the
   Search Results Context (TabKind::SearchResults, Requirement 1).

4. THE SEARCH_Verb SHALL NOT be an FFEDIT (editor-buffer) verb; the in-buffer string search
   is the separate FIND command owned by `docs/specs/find-and-replace/`. WHEN an editor
   Context is focused and the user types SEARCH, THE command SHALL still resolve to the
   FFCMD cross-backend SEARCH (the active FFEDIT environment does not own SEARCH and declines
   it, so the FFCMD base receives it), not to the editor's in-buffer FIND.

5. THE existing `GSEARCH`/`SEARCH` activation seam (Requirement 1.2) SHALL remain valid:
   SEARCH opens the Search Results Context, and this requirement makes SEARCH the canonical
   identifier for cross-backend search rather than only a panel-open keyword.

---

### Requirement 8: GREP Alias Surface for SEARCH (CR-NR-110)

**User Story:** As a user accustomed to grep, I want to type `grep <string>` and have it
behave exactly like `SEARCH <string>`, so that a familiar name works without creating a
second, divergent command.

**Source:** CR-NR-110

#### Acceptance Criteria

1. THE workbench SHALL provide `GREP` as an ALIAS surface for the canonical SEARCH_Verb,
   recorded through the EXISTING FFCMD AliasTable alias mechanism (surface form -> canonical
   verb); it SHALL NOT be a separate command with its own handler.

2. WHEN the user types `GREP <string>` (case-insensitively), THE workbench SHALL resolve it
   to the canonical SEARCH_Verb and take the IDENTICAL code path and produce the IDENTICAL
   result as `SEARCH <string>` (the alias is behaviour-neutral).

3. THE canonical SEARCH SHALL be what is recorded in command history and persisted when the
   GREP alias is used (typing `grep` records/persists `SEARCH`), so saved state and macros
   do not depend on the alias surface.

4. THE GREP alias registration SHALL follow the same case-insensitive matching and
   load-time collision discipline as the existing FFCMD alias entries; an alias that
   collides with another FFCMD canonical verb or alias SHALL be rejected at load.

---

### Requirement 9: Cross-Backend Search Contract (CR-NR-110; engine deferred)

**User Story:** As a user working across mainframe datasets and local files, I want SEARCH
to look for my string across a set of files that may span mainframe, POSIX, and NTFS
storage through FFWB's own file abstraction, so that one command covers all my files
without my shelling out to an external tool.

**Source:** CR-NR-110

#### Acceptance Criteria

1. THE SEARCH scope SHALL be a SET OF FILES that MAY span multiple VFS_Backends (mainframe
   catalog/dataset, POSIX filesystem, NTFS filesystem), resolved through FFWB's own VFS /
   catalog abstraction. SEARCH SHALL NOT be implemented as a raw shell-out to an external
   process as its primary mechanism.

2. WHEN a SEARCH spans multiple VFS_Backends, THE matches from all backends SHALL be
   surfaced together in the one Search Results Context (Requirement 1, Requirement 4),
   grouped by file as for any other global search.

3. THE SEARCH scope SHALL be expressible using the existing scope and include/exclude
   controls of the Search Results Context (Requirement 2) where those apply to a backend;
   WHERE a backend (for example a mainframe catalog) requires backend-specific scope
   addressing, that addressing is part of the DEFERRED engine work (criterion 9.5) and is
   not specified by this gate.

4. THE concrete internal Search_Engine -- whether and how a grep engine is embedded for the
   POSIX/NTFS backends, and how per-backend matching is performed -- is DEFERRED and SHALL
   be specified in a follow-on gate; this requirement fixes only the user-facing CONTRACT
   (scope = a set of files across VFS backends; results in the Search Results Context).

5. THE mainframe DATASET traversal and record-search semantics (how datasets are enumerated
   and how records within a dataset are matched) are DEFERRED pending the Volume Model
   (CR-NR-105, `docs/specs/volume-model/`) and the dataset-catalog resolution mechanisms
   (`docs/specs/dataset-catalog/`); this gate SHALL NOT specify that traversal, and SHALL
   record the dependency explicitly (see Out of Scope and the design Deferred section).

---

## Out of Scope (CR-NR-110)

The following are explicitly DEFERRED by CR-NR-110 and are NOT delivered by this gate:

- The concrete SEARCH internal engine: whether a grep engine (for example the `grep`/`ignore`
  crate family already used by this spec's per-file scan) is embedded for the POSIX/NTFS
  backends, and the per-backend matching implementation. Dependency: the follow-on engine
  gate.
- Mainframe DATASET traversal and record-level search semantics (dataset enumeration across
  volumes, record framing, RECFM/LRECL-aware matching). Dependency: Volume Model (CR-NR-105,
  `docs/specs/volume-model/`) AND dataset-catalog resolution (`docs/specs/dataset-catalog/`)
  must be finalised first.
- Any backend-specific scope-addressing syntax for datasets (criterion 9.3). It belongs with
  the deferred engine/traversal work.

CR-NR-110 delivers the VERB SPLIT (FIND = FFEDIT, SEARCH = FFCMD canonical, GREP alias), the
environment ownership, the cross-backend CONTRACT, and the user-facing semantics only.
