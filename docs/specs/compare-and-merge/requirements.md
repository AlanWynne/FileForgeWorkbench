# Requirements Document

## Introduction

This feature specifies the **Compare and Merge** subsystem for FileForgeWorkbench (`ff-compare` crate). The compare-and-merge system provides a COMPARE primary command, LCS-based line differencing, side-by-side and inline diff views, diff navigation, merge operations (accept left/right/both), three-way merge support, and VFS-aware resource comparison across any registered provider.

The compare-and-merge subsystem is fully **VFS-aware** (FFW-ARCH-001): any two resources addressable by URI — local files, dataset catalog members, or future remote resources — can be compared without the user needing to know or care about the underlying provider. The diff engine operates on the document model's line abstraction and supports configurable comparison options (ignore whitespace, ignore case) and binary detection.

The subsystem integrates with the workbench's **command framework** for invocation, the **layout-and-docking** system for split-panel diff rendering, the **theme-and-appearance** system for diff highlighting colours, and the **workflow engine** for three-way merge as a structured workflow. Merge operations that modify documents flow through the **edit-operations** subsystem to preserve undo/redo integration.

The subsystem also supports convenience comparison workflows: compare the active document with its last-saved version (detect unsaved changes visually), compare the active document with clipboard content, and compare two text selections within the editor. A diff export facility produces standard unified diff format for interoperability with external tools and version control systems.

**Source references:**
- **[FFE-COMPARE]** = FileForgeEditor compare-and-merge feature (planned — COMPARE command, basic diff, merge)
- **[WB]** = Workbench Platform Architecture Brief (VFS-aware operations, workflow integration, command-driven architecture)
- **[SCI]** = Scintilla diff concepts (change markers, indicator rendering adapted to Rust/egui)

## Glossary

- **COMPARE_Command**: The primary command (`compare.execute`) that initiates a comparison between two resources. Invoked via command line (`COMPARE path1 path2`), context menu, or keyboard shortcut. [FFE-COMPARE, WB]
- **Diff_Engine**: The core comparison engine that computes the set of differences between two text sequences using an LCS-based algorithm. Operates on lines of text, independent of rendering. [FFE-COMPARE]
- **LCS**: Longest Common Subsequence — the foundational algorithm used to determine the optimal alignment between two sequences of lines, minimising the reported differences. [FFE-COMPARE]
- **Diff_Result**: The structured output of the diff engine: a sequence of diff hunks describing insertions, deletions, and changes between two inputs. [FFE-COMPARE]
- **Diff_Hunk**: A contiguous region of difference — describes a range of lines in the left input and a corresponding range in the right input that differ. Types: Added, Removed, Changed. [FFE-COMPARE]
- **Inline_Change**: A character-level or word-level difference within a changed line pair, enabling fine-grained highlighting of exactly what changed within a line. [FFE-COMPARE]
- **Side_By_Side_View**: A split-panel rendering mode showing the left resource in one panel and the right resource in another panel, with aligned lines and diff highlighting. [FFE-COMPARE]
- **Inline_View**: A unified diff rendering mode showing both resources merged into a single panel with added/removed/changed lines interleaved and colour-coded. [FFE-COMPARE]
- **Diff_Highlight**: The visual presentation of differences using background colours and text decorations: added lines (green background), removed lines (red background), changed lines (yellow/orange background), with inline character differences emphasised. [FFE-COMPARE, WB]
- **Diff_Navigation**: The ability to jump between difference hunks sequentially (next diff, previous diff) without manual scrolling. [FFE-COMPARE]
- **Merge_Operation**: An action that resolves a difference by accepting content from one side (left, right, or both) and applying it to a merge result document. [FFE-COMPARE]
- **Three_Way_Merge**: A merge mode involving a common base version plus two divergent versions (left and right), enabling conflict detection where both sides modified the same region. [FFE-COMPARE, WB]
- **Merge_Conflict**: A region where both the left and right versions have modified the same lines relative to the base, requiring manual resolution. [FFE-COMPARE]
- **Compare_Session**: The stateful context of an active comparison, holding references to both resources, the computed diff result, current navigation position, and merge state. [FFE-COMPARE]
- **Resource_URI**: The unified resource identifier (`vfs://provider/path`) used to address any resource for comparison. [WB]
- **Diff_Statistics**: A summary of comparison results: total lines added, removed, changed, and unchanged. [FFE-COMPARE]
- **Binary_Comparison**: A comparison mode for non-text resources that reports whether resources are identical or different at the byte level, without producing line-level diff hunks. [FFE-COMPARE]
- **Compare_Output_Panel**: A dockable panel displaying comparison results, statistics, and navigation controls when the diff view is not appropriate (e.g., binary files, summary mode). [FFE-COMPARE, WB]
- **Myers_Diff**: The default diff algorithm (Eugene Myers, 1986) that finds the shortest edit script between two sequences in O(ND) time where N is total input length and D is the edit distance. Produces minimal, optimal diffs. [FFE-COMPARE]
- **Patience_Diff**: An alternative diff algorithm that first anchors on unique matching lines then fills between them, producing more readable diffs for structured code where standard LCS may match non-semantic lines. [FFE-COMPARE]
- **Compare_With_Saved**: A convenience comparison mode that diffs the active editor document's current in-memory content against its last-persisted version loaded from VFS, visualising unsaved changes. [FFE-COMPARE]
- **Compare_With_Clipboard**: A convenience comparison mode that diffs the active document (or selection) against text content currently on the system clipboard. [FFE-COMPARE]
- **Selection_Comparison**: A comparison mode that diffs two arbitrary text selections (from the same or different documents) without requiring separate files. [FFE-COMPARE]
- **Unified_Diff_Format**: The standard patch format (POSIX/git-compatible) using `---`/`+++` headers, `@@ ... @@` range indicators, and `+`/`-`/space-prefixed lines to represent diffs in a portable text format. [FFE-COMPARE]

---

## Requirements

### Requirement 1: COMPARE Primary Command

**User Story:** As a workbench user, I want a COMPARE command that I can invoke from the command line, context menu, or keyboard shortcut, so that I can compare any two resources with a consistent, discoverable interface.

**Source:** [FFE-COMPARE] COMPARE command; [WB] command-driven architecture.

#### Acceptance Criteria

1. THE command framework SHALL register a command with ID `compare.execute` that initiates a resource comparison.
2. WHEN `COMPARE path1 path2` is entered in the command line, THE command SHALL resolve both paths as Resource_URIs (bare paths resolve via the default provider) and initiate a comparison between the two resources.
3. WHEN `COMPARE path1` is entered with only one path and an active editor document exists, THE command SHALL compare the specified resource against the currently active document.
4. IF `COMPARE` is entered with no arguments and an active editor document exists, THE command SHALL prompt the user to select a second resource via a file picker dialog (VFS-aware browse).
5. IF `COMPARE` is entered with no arguments and no active editor document exists, THE command SHALL return an error result with the message "No active document. Specify two paths to compare." and display the error in the command output area.
6. WHEN both resource URIs are resolved, THE command SHALL verify that both resources exist via the VFS `exists()` method; IF either resource does not exist, THEN THE command SHALL return a `VfsError::NotFound` error identifying the missing resource URI.
7. THE `compare.execute` command SHALL accept optional parameters: `ignore_whitespace` (bool, default false), `ignore_case` (bool, default false), and `view_mode` (enum: `side_by_side` | `inline`, default `side_by_side`).
8. THE `compare.execute` command metadata SHALL include: display name "Compare Files", category "compare", description "Compare two files or resources side by side", and a default keyboard shortcut (configurable).
9. ALL COMPARE command invocations SHALL be routed through the command framework dispatch — no UI code SHALL directly invoke the diff engine without going through `compare.execute`.
10. THE COMPARE command SHALL support Resource_URIs from different VFS providers in a single comparison (e.g., comparing `vfs://local/file.txt` with `vfs://catalog/HLQ.DATA.MEMBER`).

---

### Requirement 2: Diff Algorithm — Myers / Patience Line Comparison

**User Story:** As a workbench developer, I want a well-defined diff algorithm that produces minimal, optimal difference sets between two text inputs, so that comparison results are accurate, readable, and deterministic.

**Source:** [FFE-COMPARE] diff algorithm; [WB] GUI-independent core.

#### Acceptance Criteria

1. THE Diff_Engine SHALL implement a Myers diff algorithm (greedy LCS-based shortest edit script) as the default differencing strategy, producing an optimal edit script minimising the total number of changed lines.
1a. THE Diff_Engine SHALL additionally support a patience diff algorithm variant (using unique-line anchoring for improved hunk readability on structured code), selectable via a `diff_algorithm` option (enum: `myers` | `patience`, default `myers`).
2. THE Diff_Engine SHALL operate on sequences of lines (as `&[&str]` or equivalent) — it SHALL be independent of the document model's internal buffer representation and SHALL NOT require a Document handle.
3. THE Diff_Engine SHALL produce a `Diff_Result` containing an ordered sequence of `Diff_Hunk` entries, where each hunk is one of: `Equal { left_start, right_start, count }`, `Added { right_start, count }`, `Removed { left_start, count }`, or `Changed { left_start, left_count, right_start, right_count }`.
4. WHEN two identical inputs are compared, THE Diff_Engine SHALL return a Diff_Result containing a single `Equal` hunk spanning all lines, with no difference hunks.
5. WHEN one input is empty and the other is non-empty, THE Diff_Engine SHALL return a Diff_Result containing a single `Added` or `Removed` hunk spanning all lines of the non-empty input.
6. THE Diff_Engine SHALL support an `ignore_whitespace` option: WHEN enabled, leading and trailing whitespace on each line SHALL be excluded from comparison, and lines differing only in whitespace SHALL be reported as `Equal`.
7. THE Diff_Engine SHALL support an `ignore_case` option: WHEN enabled, line comparison SHALL use Unicode case-folded equality, and lines differing only in case SHALL be reported as `Equal`.
8. THE Diff_Engine SHALL perform inline change detection for `Changed` hunks: within each pair of changed lines, the engine SHALL identify the specific character ranges that differ, producing `Inline_Change` markers for fine-grained highlighting.
9. THE Diff_Engine output SHALL be deterministic: given the same two inputs and the same options, the engine SHALL always produce the same Diff_Result.
10. THE Diff_Engine SHALL handle large inputs efficiently — comparison of two 100,000-line files SHALL complete within 2 seconds on a modern desktop CPU (single-threaded).

---

### Requirement 3: Side-by-Side Diff View

**User Story:** As a user, I want to see the two compared resources displayed in a split panel with aligned lines and colour-coded differences, so that I can visually identify what changed between the two versions.

**Source:** [FFE-COMPARE] side-by-side diff view; [WB] layout-and-docking integration.

#### Acceptance Criteria

1. WHEN a comparison is initiated with `view_mode = side_by_side`, THE compare subsystem SHALL open a split panel in the center dock area showing the left resource in the left pane and the right resource in the right pane.
2. THE side-by-side view SHALL align corresponding lines vertically: where one side has added or removed lines, the other side SHALL display blank placeholder lines to maintain alignment.
3. LINES identified as `Added` (present only in the right input) SHALL be highlighted with the theme colour token `diff.added_background` in the right pane and display blank placeholders in the left pane.
4. LINES identified as `Removed` (present only in the left input) SHALL be highlighted with the theme colour token `diff.removed_background` in the left pane and display blank placeholders in the right pane.
5. LINES identified as `Changed` SHALL be highlighted with the theme colour token `diff.changed_background` in both panes, with inline character differences additionally highlighted using `diff.inline_change_background`.
6. LINES identified as `Equal` SHALL be displayed without diff highlighting in both panes.
7. THE side-by-side view SHALL synchronize vertical scrolling between the two panes: scrolling one pane SHALL scroll the other pane by the same number of aligned display lines.
8. THE side-by-side view SHALL display line numbers in both panes, reflecting the original line numbers from each resource (not the aligned display line numbers).
9. THE side-by-side view SHALL display a summary header showing the Resource_URI of each resource (left and right) and the Diff_Statistics.
10. THE split panel SHALL be resizable via a draggable splitter between the two panes, with a default 50/50 split and a minimum pane width of 100 logical pixels.
11. THE side-by-side view SHALL integrate with the layout-and-docking system as a Tab_Group split, using the standard Tab_Group splitter behaviour defined in the `layout-and-docking` spec.

---

### Requirement 4: Inline Diff View (Unified)

**User Story:** As a user, I want an alternative unified diff view that shows all differences in a single panel with interleaved added and removed lines, so that I can see the complete change narrative in a compact format.

**Source:** [FFE-COMPARE] inline/unified diff view; [WB] layout-and-docking.

#### Acceptance Criteria

1. WHEN a comparison is initiated with `view_mode = inline`, THE compare subsystem SHALL open a single panel in the center dock area showing a unified representation of both resources.
2. IN the inline view, `Equal` lines SHALL be displayed once without special highlighting.
3. IN the inline view, `Removed` lines SHALL be displayed with the theme colour token `diff.removed_background` and prefixed with a gutter marker (e.g., `−` or red indicator).
4. IN the inline view, `Added` lines SHALL be displayed with the theme colour token `diff.added_background` and prefixed with a gutter marker (e.g., `+` or green indicator).
5. IN the inline view, `Changed` line pairs SHALL display the removed version followed by the added version, each highlighted with appropriate diff colours and inline change highlighting.
6. THE inline view SHALL display two line-number columns in the gutter: one for the left resource line numbers and one for the right resource line numbers, with blank entries where a line does not correspond to that side.
7. THE inline view SHALL display a summary header showing both Resource_URIs and the Diff_Statistics.
8. THE user SHALL be able to switch between side-by-side and inline view modes for an active Compare_Session via a toggle command (`compare.toggle_view_mode`) without re-running the diff computation.

---

### Requirement 5: Diff Highlighting and Theme Integration

**User Story:** As a user, I want differences highlighted with clear, accessible colours that follow the active theme, so that I can distinguish added, removed, and changed content at a glance in both dark and light modes.

**Source:** [FFE-COMPARE] diff highlighting; [WB] theme-and-appearance integration.

#### Acceptance Criteria

1. THE theme system SHALL define the following diff-specific colour tokens: `diff.added_background`, `diff.added_foreground`, `diff.removed_background`, `diff.removed_foreground`, `diff.changed_background`, `diff.changed_foreground`, `diff.inline_change_background`, `diff.gutter_added`, `diff.gutter_removed`, `diff.gutter_changed`.
2. ALL diff highlighting colours SHALL be defined in each built-in theme (dark, light, high-contrast), ensuring readable contrast ratios in all visual modes.
3. THE diff highlighting SHALL support alpha transparency: diff background colours SHALL blend with the editor background rather than fully replacing it, preserving syntax highlighting visibility within diff regions.
4. WHEN the active theme changes (including hot-reload), THE diff view SHALL immediately re-render using the new diff colour tokens without requiring the user to re-run the comparison.
5. THE diff gutter markers (added/removed/changed indicators in the line-number gutter) SHALL use the corresponding `diff.gutter_*` colour tokens and SHALL be visually distinct from standard line-number rendering.
6. IN high-contrast mode, THE diff highlighting SHALL additionally use text decorations (underlines, borders) alongside colour to ensure differences are perceivable without relying solely on colour differentiation.

---

### Requirement 6: Diff Navigation

**User Story:** As a user, I want to jump between differences sequentially without scrolling manually, so that I can review all changes efficiently.

**Source:** [FFE-COMPARE] navigation between differences.

#### Acceptance Criteria

1. THE compare subsystem SHALL register a command `compare.next_diff` that moves the viewport and cursor to the next Diff_Hunk in the current Compare_Session.
2. THE compare subsystem SHALL register a command `compare.prev_diff` that moves the viewport and cursor to the previous Diff_Hunk in the current Compare_Session.
3. WHEN `compare.next_diff` is invoked and the cursor is positioned before or within the last hunk, THE compare subsystem SHALL advance to the start of the next hunk and scroll the viewport to centre that hunk.
4. WHEN `compare.next_diff` is invoked and the cursor is at or past the last hunk, THE compare subsystem SHALL wrap to the first hunk and notify the user via the status bar that navigation has wrapped to the beginning.
5. WHEN `compare.prev_diff` is invoked and the cursor is after the first hunk, THE compare subsystem SHALL move to the start of the previous hunk and scroll the viewport to centre that hunk.
6. WHEN `compare.prev_diff` is invoked and the cursor is at or before the first hunk, THE compare subsystem SHALL wrap to the last hunk and notify the user via the status bar that navigation has wrapped to the end.
7. THE Compare_Session SHALL maintain a current-diff-index indicating which hunk the user is currently viewing, updated by navigation and by manual scrolling when the viewport aligns with a hunk.
8. THE diff view SHALL visually indicate the currently focused hunk (e.g., a border, thicker highlight, or gutter marker) to distinguish it from other highlighted hunks.
9. THE status bar SHALL display the current hunk position as "Diff N of M" where N is the current-diff-index (1-based) and M is the total number of difference hunks.

---

### Requirement 7: Merge Operations

**User Story:** As a user, I want to resolve differences by accepting content from either side (or both), so that I can produce a merged result incorporating the changes I want to keep.

**Source:** [FFE-COMPARE] merge operations; [WB] edit-operations integration.

#### Acceptance Criteria

1. THE compare subsystem SHALL register the following merge commands: `compare.accept_left` (accept the left version for the current hunk), `compare.accept_right` (accept the right version for the current hunk), `compare.accept_both` (concatenate both versions — left then right — for the current hunk).
2. WHEN `compare.accept_left` is invoked on a `Changed` or `Added` hunk, THE compare subsystem SHALL replace the hunk's content in the merge result with the left version's content for that region.
3. WHEN `compare.accept_right` is invoked on a `Changed` or `Removed` hunk, THE compare subsystem SHALL replace the hunk's content in the merge result with the right version's content for that region.
4. WHEN `compare.accept_both` is invoked on a `Changed` hunk, THE compare subsystem SHALL insert both versions sequentially (left content followed by right content) into the merge result at that position.
5. ALL merge operations SHALL create edit transactions routable through the command framework, integrating with the undo-redo system so that merge accepts can be undone individually.
6. WHEN a hunk is resolved (accepted), THE diff view SHALL visually mark it as resolved (e.g., dimmed highlight, check indicator in the gutter) and advance the current-diff-index to the next unresolved hunk.
7. THE compare subsystem SHALL register a command `compare.accept_all_left` that resolves all remaining unresolved hunks by accepting the left version, and `compare.accept_all_right` that resolves all remaining hunks with the right version.
8. THE Compare_Session SHALL track resolution status per hunk: unresolved, resolved-left, resolved-right, resolved-both, or resolved-custom.
9. WHEN all hunks in a Compare_Session are resolved, THE compare subsystem SHALL display a notification in the status bar indicating the merge is complete and prompt the user to save the merged result.
10. THE merge result document SHALL be a new Document (via the document-model) that the user can edit, save (through VFS), or discard — the original compared resources SHALL NOT be modified unless the user explicitly saves back to one of them.

---

### Requirement 8: Three-Way Merge

**User Story:** As a user, I want to perform three-way merges using a common base version plus two divergent versions, so that I can resolve conflicts arising from concurrent modifications to the same file.

**Source:** [FFE-COMPARE] three-way merge; [WB] workflow-engine integration.

#### Acceptance Criteria

1. THE compare subsystem SHALL support three-way merge by accepting three Resource_URIs: base (common ancestor), left (first modified version), and right (second modified version).
2. THE compare subsystem SHALL register a command `compare.three_way_merge` that accepts parameters: `base`, `left`, `right` (all Resource_URIs), and optional `ignore_whitespace` and `ignore_case` options.
3. THE three-way merge SHALL compute diffs from base-to-left and base-to-right independently, then classify each region as: unchanged (same in all three), left-only-change (left differs from base, right matches base), right-only-change (right differs from base, left matches base), or conflict (both left and right differ from base in the same region).
4. REGIONS classified as unchanged SHALL be included in the merge result automatically without user intervention.
5. REGIONS classified as left-only-change SHALL be automatically resolved in favour of the left version in the merge result.
6. REGIONS classified as right-only-change SHALL be automatically resolved in favour of the right version in the merge result.
7. REGIONS classified as conflict SHALL be marked as unresolved in the merge result and highlighted with the theme colour token `diff.conflict_background`, requiring manual resolution by the user.
8. FOR conflict regions, THE merge view SHALL display all three versions (base, left, right) with clear labels, enabling the user to accept left, accept right, accept both, or manually edit the conflict region.
9. THE three-way merge SHALL be modelled as a workflow (via the workflow-engine): steps include load-resources, compute-diffs, auto-resolve-non-conflicts, present-conflicts, await-user-resolution, and save-result.
10. THE three-way merge workflow SHALL support cancellation at any step — if cancelled during conflict resolution, THE compare subsystem SHALL offer to save the partially resolved result or discard it.

---

### Requirement 9: VFS-Aware Resource Comparison

**User Story:** As a user, I want to compare any two resources from any registered VFS provider without worrying about where they are stored, so that I can compare a local file against a dataset catalog member or a future remote resource seamlessly.

**Source:** [WB] FFW-ARCH-001 all content through VFS; [FFE-COMPARE].

#### Acceptance Criteria

1. THE compare subsystem SHALL resolve all resource paths to Resource_URIs via the VFS abstraction before initiating comparison — bare paths SHALL be resolved via the default provider (local filesystem).
2. THE compare subsystem SHALL support comparing resources from different VFS providers in a single comparison session (e.g., `vfs://local/file.txt` vs. `vfs://catalog/HLQ.DATA(MEMBER)`).
3. THE compare subsystem SHALL load resource content by calling the VFS `read()` or `read_stream()` method on each resource, using the provider resolved from the Resource_URI — no direct filesystem access is permitted.
4. IF a resource cannot be loaded (VfsError::NotFound, VfsError::PermissionDenied, or other error), THEN THE compare subsystem SHALL display the error in the Compare_Output_Panel and SHALL NOT attempt to display a partial diff.
5. THE compare subsystem SHALL query VFS capabilities for each resource: IF a provider declares the resource is binary (no text content-type) or if automatic binary detection identifies the resource as binary, THEN THE compare subsystem SHALL switch to binary comparison mode (Requirement 10).
6. THE compare subsystem SHALL support comparing resources of different encodings by normalising both to UTF-8 (via the encoding-and-characters subsystem) before feeding content to the Diff_Engine.
7. THE VFS file-watch integration SHALL monitor both compared resources for external changes: IF an external modification occurs during an active Compare_Session, THEN THE compare subsystem SHALL notify the user and offer to refresh the comparison with updated content.

---

### Requirement 10: Binary Comparison Mode

**User Story:** As a user, I want to compare binary (non-text) resources and see whether they are identical or different, so that I can detect changes even when line-based diffing is not meaningful.

**Source:** [FFE-COMPARE] binary comparison.

#### Acceptance Criteria

1. WHEN both resources are detected as binary (via content-type heuristic: presence of null bytes in the first 8 KB, or provider metadata indicating binary), THE compare subsystem SHALL switch to binary comparison mode.
2. IN binary comparison mode, THE compare subsystem SHALL compare the raw byte content of both resources and report one of: `Identical` (byte-for-byte equal) or `Different` (at least one byte differs).
3. WHEN resources are `Different` in binary mode, THE compare subsystem SHALL report: the first byte offset where they diverge, the total size of each resource in bytes, and a percentage similarity estimate (matching bytes / max size × 100).
4. THE binary comparison result SHALL be displayed in the Compare_Output_Panel as a summary (not a line-level diff view), including file sizes, match status, and divergence offset.
5. IF one resource is detected as binary and the other as text, THEN THE compare subsystem SHALL display a warning in the Compare_Output_Panel indicating a mixed comparison and SHALL fall back to binary comparison mode.
6. THE binary comparison SHALL support large files by comparing in streaming chunks (matching the VFS `read_stream()` interface) rather than loading both files entirely into memory.

---

### Requirement 11: Comparison Options — Ignore Whitespace and Ignore Case

**User Story:** As a user, I want options to ignore whitespace differences or case differences during comparison, so that I can focus on meaningful content changes and filter out formatting noise.

**Source:** [FFE-COMPARE] comparison options.

#### Acceptance Criteria

1. THE compare subsystem SHALL accept an `ignore_whitespace` option that applies to the diff computation: WHEN enabled, lines differing only in leading whitespace, trailing whitespace, or internal whitespace runs SHALL be treated as equal.
2. THE `ignore_whitespace` option SHALL support three modes: `none` (default — all whitespace significant), `leading_trailing` (ignore only leading and trailing whitespace), and `all` (ignore all whitespace differences including internal).
3. THE compare subsystem SHALL accept an `ignore_case` option that applies to the diff computation: WHEN enabled, line comparison SHALL use Unicode case-folded equality (using the same case-folding rules as the find-and-replace subsystem).
4. WHEN comparison options are changed for an active Compare_Session, THE compare subsystem SHALL re-run the diff computation with the new options and update the diff view without requiring the user to re-invoke the COMPARE command.
5. THE current comparison options SHALL be displayed in the diff view header/toolbar and SHALL be togglable via commands: `compare.toggle_ignore_whitespace` and `compare.toggle_ignore_case`.
6. THE comparison options SHALL be persisted as user preferences (via the configuration-system) so that the user's preferred defaults are applied to subsequent comparisons.

---

### Requirement 12: Diff Statistics

**User Story:** As a user, I want to see a summary of the comparison results (lines added, removed, changed, unchanged), so that I can quickly assess the scope of differences without reviewing every hunk.

**Source:** [FFE-COMPARE] diff statistics.

#### Acceptance Criteria

1. THE Diff_Result SHALL include a `Diff_Statistics` struct containing: `lines_added` (total lines present only in right), `lines_removed` (total lines present only in left), `lines_changed` (total line pairs that differ), `lines_unchanged` (total lines identical in both), and `hunks_count` (total number of diff hunks).
2. THE diff view (both side-by-side and inline) SHALL display the Diff_Statistics in the view header, formatted as a concise summary (e.g., "+42 −17 ~8 unchanged: 1,203").
3. THE Compare_Output_Panel SHALL display a detailed statistics breakdown when the comparison completes, including percentages (e.g., "93% unchanged, 4% added, 2% removed, 1% changed").
4. WHEN comparison options change (ignore_whitespace, ignore_case) and the diff is recomputed, THE statistics SHALL be recalculated and the display SHALL update immediately.

---

### Requirement 13: Compare Output Panel

**User Story:** As a user, I want a dedicated output panel that shows comparison summaries, binary comparison results, and error messages, so that I have a persistent record of comparison operations and results.

**Source:** [FFE-COMPARE] compare output; [WB] layout-and-docking.

#### Acceptance Criteria

1. THE compare subsystem SHALL register a dockable panel with the layout system (panel_id: `compare_output`, default dock zone: `Bottom`), implementing the `DockablePanel` trait.
2. THE Compare_Output_Panel SHALL display a log of comparison operations with timestamps: each entry SHALL show the two Resource_URIs compared, the comparison options used, and the Diff_Statistics summary.
3. WHEN a binary comparison completes, THE Compare_Output_Panel SHALL display the binary comparison result (identical/different, sizes, divergence offset).
4. WHEN a comparison fails (resource not found, permission denied, load error), THE Compare_Output_Panel SHALL display the error with the Resource_URI and error description.
5. THE Compare_Output_Panel entries SHALL be selectable: WHEN the user clicks/activates a previous comparison entry, THE compare subsystem SHALL offer to re-open that comparison in a diff view (re-loading resources if needed).
6. THE Compare_Output_Panel SHALL support clearing its history via a `compare.clear_output` command.
7. THE Compare_Output_Panel SHALL be toggleable via the standard panel show/hide mechanism in the layout system.

---

### Requirement 14: Compare Active File with Saved Version

**User Story:** As a user, I want to compare the current (possibly modified) document in the active editor against its last-saved version on disk, so that I can review my unsaved changes visually before deciding to save or revert.

**Source:** [FFE-COMPARE] compare with saved; [WB] VFS-aware operations.

#### Acceptance Criteria

1. THE compare subsystem SHALL register a command `compare.with_saved` that compares the active editor document's current in-memory content against the persisted version of the same resource (loaded fresh from VFS).
2. WHEN `compare.with_saved` is invoked, THE compare subsystem SHALL load the last-saved content from the resource's VFS provider (using the document's Resource_URI) and compare it against the document model's current line content.
3. IF the active document has no associated Resource_URI (unsaved new document), THEN THE command SHALL return an error result with the message "Document has not been saved. No saved version to compare against."
4. IF the active document has no unsaved modifications (is_modified == false), THE command SHALL notify the user via the status bar with the message "No unsaved changes — document matches saved version." and SHALL NOT open a diff view.
5. THE compare-with-saved diff view SHALL label the left pane as "Saved: {resource_name}" and the right pane as "Unsaved Changes: {resource_name}", clearly identifying which version is which.
6. THE compare-with-saved view SHALL be read-only in both panes — merge operations SHALL NOT be available since the purpose is review, not merge.

---

### Requirement 15: Compare Active File with Clipboard

**User Story:** As a user, I want to compare the active editor document (or its current selection) against text on the clipboard, so that I can quickly diff content I've copied from another source against my current work.

**Source:** [FFE-COMPARE] compare with clipboard; [WB] desktop integration.

#### Acceptance Criteria

1. THE compare subsystem SHALL register a command `compare.with_clipboard` that compares the active editor document's content against the current text content of the system clipboard.
2. WHEN `compare.with_clipboard` is invoked and the clipboard contains text content, THE compare subsystem SHALL use the clipboard text as the right-side input and the active document content as the left-side input, opening a diff view.
3. IF the clipboard does not contain text content (empty or non-text data), THEN THE command SHALL return an error result with the message "Clipboard does not contain text content."
4. IF no active editor document exists, THEN THE command SHALL return an error result with the message "No active document. Open a file before comparing with clipboard."
5. THE compare-with-clipboard diff view SHALL label the left pane as "{resource_name}" (the active document) and the right pane as "Clipboard Content".
6. THE clipboard content SHALL be treated as a temporary, unnamed resource — it SHALL NOT have a Resource_URI and SHALL NOT be monitored for external changes.
7. WHEN the user has a text selection active in the editor, THE `compare.with_clipboard` command SHALL compare only the selected text against clipboard content (not the entire document), labelling the left pane as "Selection in {resource_name}".

---

### Requirement 16: Compare Selections

**User Story:** As a user, I want to compare two text selections, so that I can diff arbitrary sections of text without needing them to be in separate files.

**Source:** [FFE-COMPARE] compare selections.

#### Acceptance Criteria

1. THE compare subsystem SHALL register a command `compare.selections` that compares two user-provided text selections.
2. THE `compare.selections` workflow SHALL be: (a) user selects text in an editor, (b) invokes `compare.mark_selection_for_compare` to mark it as "Selection A", (c) user selects different text (same or different document), (d) invokes `compare.selections` to compare Selection A against the current selection (Selection B).
3. WHEN `compare.mark_selection_for_compare` is invoked, THE compare subsystem SHALL store the selected text and its source label (document name + line range) as Selection A, and indicate in the status bar that a selection is marked for comparison.
4. WHEN `compare.selections` is invoked and a marked Selection A exists and the current selection is non-empty, THE compare subsystem SHALL compare Selection A (left) against the current selection (right) in a diff view.
5. IF `compare.selections` is invoked without a previously marked Selection A, THEN THE command SHALL return an error result with the message "No selection marked for comparison. Use 'Mark Selection for Compare' first."
6. IF `compare.selections` is invoked and the current selection is empty, THEN THE command SHALL return an error result with the message "No text selected. Select text to compare against the marked selection."
7. THE compare-selections diff view SHALL label the left pane as "Selection A: {source_label}" and the right pane as "Selection B: {source_label}", where source_label includes the document name and line range.
8. THE marked Selection A SHALL persist until explicitly cleared (via `compare.clear_marked_selection`) or until a new selection is marked, surviving document switches and editor navigation.

---

### Requirement 17: Diff Export (Unified Diff Format)

**User Story:** As a user, I want to export comparison results in standard unified diff format, so that I can share diffs with version control tools, email them to colleagues, or apply them as patches.

**Source:** [FFE-COMPARE] diff export; [WB] interoperability.

#### Acceptance Criteria

1. THE compare subsystem SHALL register a command `compare.export_diff` that generates the active Compare_Session's diff result in standard unified diff format (as specified by POSIX and used by `diff -u`, `git diff`).
2. THE unified diff output SHALL include the standard header lines: `--- {left_resource_path}` and `+++ {right_resource_path}`, with optional timestamp metadata.
3. THE unified diff output SHALL represent each Diff_Hunk as a unified diff chunk with the `@@ -L,S +L,S @@` range header, followed by context lines (prefixed with space), removed lines (prefixed with `-`), and added lines (prefixed with `+`).
4. THE diff export SHALL include 3 lines of context (unchanged lines) around each hunk by default, configurable via a `context_lines` parameter (range 0–999).
5. WHEN `compare.export_diff` is invoked, THE command SHALL offer the following output destinations: (a) copy to clipboard, (b) save to a file (via VFS-aware file picker), or (c) open as a new unnamed document in the editor.
6. THE unified diff export SHALL correctly handle the "No newline at end of file" indicator (`\ No newline at end of file`) when either resource does not end with a newline character.
7. THE exported diff SHALL reflect the current comparison options (ignore_whitespace, ignore_case) — if options are active, the diff export SHALL note this in a comment header (e.g., `# Options: ignore_whitespace=leading_trailing`).
8. THE `compare.export_diff` command SHALL only be available when a Compare_Session is active; IF no session is active, THE command SHALL be disabled (greyed out in menus, returns error if invoked programmatically).

---

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `virtual-file-system` | **Dependency** | All resource loading for comparison flows through the VFS. Resource_URIs are resolved and content is read via VFS provider methods. [WB] |
| `document-model` | **Dependency** | The merge result is a Document instance. Line content for diff input is extracted from the document model's line abstraction. Compare-with-saved reads current document state. [FFE-COMPARE] |
| `layout-and-docking` | **Integration** | The side-by-side diff view uses Tab_Group splits. The Compare_Output_Panel registers as a DockablePanel in the Bottom dock zone. [WB] |
| `command-framework` | **Integration** | All compare/merge commands (compare.execute, compare.next_diff, compare.prev_diff, compare.accept_left, compare.with_saved, compare.with_clipboard, compare.selections, compare.export_diff, etc.) are registered with and dispatched through the command framework. [WB] |
| `theme-and-appearance` | **Consumer** | All diff highlighting colours are obtained via theme colour tokens (`diff.*`). The compare subsystem never uses hardcoded colours. [WB] |
| `undo-redo-transactions` | **Integration** | Merge accept operations create undoable edit transactions on the merge result document. Each merge operation is independently undoable. [FFE-COMPARE] |
| `edit-operations` | **Integration** | Merge accept operations produce edit transactions on the merge result document, integrating with undo/redo. [FFE-COMPARE] |
| `workflow-engine` | **Integration** | Three-way merge is modelled as a workflow with defined steps, cancellation support, and progress reporting. [WB] |
| `encoding-and-characters` | **Dependency** | Resources with different encodings are normalised to UTF-8 before diff computation. [FFE-COMPARE] |
| `find-and-replace` | **Reference** | The ignore_case option uses the same Unicode case-folding rules as find-and-replace. [FFE-COMPARE] |
| `clipboard-operations` | **Dependency** | Compare-with-clipboard reads text content from the system clipboard via the clipboard subsystem. [FFE-COMPARE] |
