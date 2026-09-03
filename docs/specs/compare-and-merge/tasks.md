# Implementation Plan: Compare and Merge (`ff-compare-merge`)

## Overview

This plan covers the complete implementation of the `ff-compare-merge` crate — the compare-and-merge subsystem for FileForgeWorkbench. The subsystem provides the COMPARE primary command, LCS-based line differencing (Myers and Patience algorithms), side-by-side and inline diff views, diff navigation, merge operations (accept left/right/both), three-way merge support, VFS-aware resource comparison across any registered provider, convenience comparison workflows (compare with saved, clipboard, selections), and unified diff export.

This is a **Wave 14 (File Explorer)** sub-project. It depends on `ff-vfs` (Wave 3), `ff-document-model` (Wave 4), `ff-command-framework` (Wave 2), `ff-workflow` (Wave 2), `ff-layout-docking` (Wave 2), `ff-theme` (Wave 2), `ff-undo-redo` (Wave 4), and `ff-edit-ops` (Wave 4).

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-compare-merge/Cargo.toml` with dependencies (ff-vfs, ff-document-model, ff-command-framework, ff-workflow, ff-layout-docking, ff-theme, ff-undo-redo, ff-edit-ops, thiserror, serde, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-compare-merge/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `diff_engine.rs`, `myers.rs`, `patience.rs`, `diff_types.rs`, `inline_change.rs`, `session.rs`, `merge.rs`, `three_way.rs`, `navigation.rs`, `view_side_by_side.rs`, `view_inline.rs`, `highlight.rs`, `statistics.rs`, `output_panel.rs`, `compare_command.rs`, `compare_saved.rs`, `compare_clipboard.rs`, `compare_selections.rs`, `export.rs`, `binary.rs`, `options.rs`, `error.rs`
  - [x] 1.4 Add `ff-compare-merge` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Core diff types and options
  - [x] 2.1 Define `DiffHunk` enum with variants: `Equal { left_start, right_start, count }`, `Added { right_start, count }`, `Removed { left_start, count }`, `Changed { left_start, left_count, right_start, right_count }`
  - [x] 2.2 Define `InlineChange` struct with character ranges for fine-grained highlighting within changed line pairs
  - [x] 2.3 Define `DiffResult` struct containing ordered `Vec<DiffHunk>` and associated `DiffStatistics`
  - [x] 2.4 Define `DiffStatistics` struct: `lines_added`, `lines_removed`, `lines_changed`, `lines_unchanged`, `hunks_count`
  - [x] 2.5 Define `DiffOptions` struct: `ignore_whitespace` (enum: None/LeadingTrailing/All), `ignore_case` (bool), `diff_algorithm` (enum: Myers/Patience)
  - [x] 2.6 Define `CompareError` enum with thiserror: VfsError, BinaryResource, NoActiveDocument, EmptyClipboard, NoMarkedSelection, SessionNotActive
  - [x] 2.7 Write unit tests for type construction, default values, and Display/Debug impls
  - Covers: Requirement 2 (AC 2.3, 2.6, 2.7), Requirement 11 (AC 11.1, 11.2, 11.3), Requirement 12 (AC 12.1)

- [x] 3. Myers diff algorithm implementation
  - [x] 3.1 Implement Myers greedy LCS-based shortest edit script operating on `&[&str]` line sequences
  - [x] 3.2 Implement line normalisation for `ignore_whitespace` option (leading/trailing and all modes)
  - [x] 3.3 Implement Unicode case-folded equality for `ignore_case` option
  - [x] 3.4 Implement `DiffResult` construction from the edit script with correct hunk classification
  - [x] 3.5 Implement optimisation for identical inputs (single Equal hunk) and empty input handling (single Added/Removed hunk)
  - [x] 3.6 Write unit tests: identical inputs, empty vs non-empty, single line diff, multi-hunk diff, whitespace/case options
  - Covers: Requirement 2 (AC 2.1, 2.2, 2.4, 2.5, 2.6, 2.7, 2.9, 2.10)

- [x] 4. Patience diff algorithm implementation
  - [x] 4.1 Implement patience diff unique-line anchoring strategy operating on `&[&str]` line sequences
  - [x] 4.2 Implement fallback to Myers for regions between anchors
  - [x] 4.3 Implement algorithm selection via `DiffOptions::diff_algorithm` field
  - [x] 4.4 Write unit tests: structured code diffs showing improved readability vs Myers, algorithm selection dispatch
  - Covers: Requirement 2 (AC 2.1a, 2.2, 2.9)

- [x] 5. Inline change detection
  - [x] 5.1 Implement character-level diff within changed line pairs to produce `InlineChange` markers
  - [x] 5.2 Implement word-boundary-aware splitting for semantic inline change grouping
  - [x] 5.3 Attach `InlineChange` markers to `Changed` hunks in the `DiffResult`
  - [x] 5.4 Write unit tests: single char change, word substitution, multiple inline changes per line pair
  - Covers: Requirement 2 (AC 2.8)

- [x] 6. Diff statistics computation
  - [x] 6.1 Implement `DiffStatistics` calculation from a `DiffResult` (lines added, removed, changed, unchanged, hunks count)
  - [x] 6.2 Implement percentage calculation (unchanged/total, added/total, removed/total, changed/total)
  - [x] 6.3 Implement formatted summary string output (e.g., "+42 −17 ~8 unchanged: 1,203")
  - [x] 6.4 Write unit tests: empty diff stats, all-equal stats, mixed hunk stats, percentage math
  - Covers: Requirement 12 (AC 12.1, 12.2, 12.3, 12.4)

- [x] 7. Compare session management
  - [x] 7.1 Define `CompareSession` struct holding: left/right Resource_URIs, loaded content, `DiffResult`, current navigation index, merge state, comparison options, view mode
  - [x] 7.2 Implement session creation from two resource URIs with VFS content loading
  - [x] 7.3 Implement session re-computation when options change (without re-loading resources)
  - [x] 7.4 Implement session refresh when external file changes are detected (re-load from VFS)
  - [x] 7.5 Implement current-diff-index tracking and update on navigation/scroll
  - [x] 7.6 Write unit tests: session creation, option change re-diff, index tracking
  - Covers: Requirement 1 (AC 1.6, 1.7), Requirement 6 (AC 6.7), Requirement 9 (AC 9.7), Requirement 11 (AC 11.4)

- [x] 8. COMPARE primary command registration
  - [x] 8.1 Register `compare.execute` command with the command framework (display name "Compare Files", category "compare", keyboard shortcut)
  - [x] 8.2 Implement two-path invocation: resolve both paths as Resource_URIs, verify existence via VFS `exists()`, create session
  - [x] 8.3 Implement single-path invocation: compare specified resource against active editor document
  - [x] 8.4 Implement no-argument invocation with active document: prompt file picker for second resource
  - [x] 8.5 Implement no-argument invocation without active document: return error "No active document. Specify two paths to compare."
  - [x] 8.6 Implement cross-provider comparison support (resources from different VFS providers)
  - [x] 8.7 Implement optional command parameters: `ignore_whitespace`, `ignore_case`, `view_mode`
  - [x] 8.8 Write unit tests: all invocation modes, error cases, parameter handling
  - Covers: Requirement 1 (AC 1.1–1.10)

- [x] 9. VFS-aware resource loading and binary detection
  - [x] 9.1 Implement resource URI resolution from bare paths via default provider
  - [x] 9.2 Implement content loading via VFS `read()` / `read_stream()` — no direct filesystem access
  - [x] 9.3 Implement binary detection heuristic: null bytes in first 8 KB or provider metadata
  - [x] 9.4 Implement encoding normalisation to UTF-8 via encoding-and-characters subsystem
  - [x] 9.5 Implement error handling for VfsError::NotFound, VfsError::PermissionDenied
  - [x] 9.6 Implement mixed binary/text resource warning and fallback to binary mode
  - [x] 9.7 Write unit tests: URI resolution, binary detection, encoding normalisation, error propagation
  - Covers: Requirement 9 (AC 9.1–9.6), Requirement 10 (AC 10.1, 10.5)

- [x] 10. Binary comparison mode
  - [x] 10.1 Implement byte-level comparison with streaming chunk processing (matching VFS `read_stream()`)
  - [x] 10.2 Implement result reporting: Identical or Different with first divergence offset, sizes, percentage similarity
  - [x] 10.3 Implement large file support without loading both files entirely into memory
  - [x] 10.4 Write unit tests: identical binary, different binary with offset, large file streaming, mixed text/binary
  - Covers: Requirement 10 (AC 10.1–10.6)

- [x] 11. Side-by-side diff view
  - [x] 11.1 Implement split panel rendering in center dock area (left resource / right resource)
  - [x] 11.2 Implement line alignment with blank placeholder lines for added/removed regions
  - [x] 11.3 Implement diff highlighting using theme colour tokens: `diff.added_background`, `diff.removed_background`, `diff.changed_background`, `diff.inline_change_background`
  - [x] 11.4 Implement synchronised vertical scrolling between panes
  - [x] 11.5 Implement original line number display in both panes
  - [x] 11.6 Implement summary header with Resource_URIs and DiffStatistics
  - [x] 11.7 Implement resizable splitter (default 50/50, minimum 100px pane width)
  - [x] 11.8 Implement integration with layout-and-docking as Tab_Group split
  - [x] 11.9 Write unit tests: alignment calculation, line number mapping, splitter constraints
  - Covers: Requirement 3 (AC 3.1–3.11)

- [x] 12. Inline (unified) diff view
  - [x] 12.1 Implement single-panel unified rendering in center dock area
  - [x] 12.2 Implement interleaved display: Equal lines once, Removed lines with gutter marker, Added lines with gutter marker, Changed pairs (removed then added)
  - [x] 12.3 Implement dual line-number columns (left and right) with blanks for non-corresponding lines
  - [x] 12.4 Implement diff highlighting using theme colour tokens with gutter indicators
  - [x] 12.5 Implement summary header with both Resource_URIs and DiffStatistics
  - [x] 12.6 Implement view mode toggle command `compare.toggle_view_mode` without re-running diff
  - [x] 12.7 Write unit tests: line interleaving logic, dual line-number generation, toggle state preservation
  - Covers: Requirement 4 (AC 4.1–4.8)

- [x] 13. Diff highlighting and theme integration
  - [x] 13.1 Define diff-specific colour tokens in the theme system: `diff.added_background`, `diff.added_foreground`, `diff.removed_background`, `diff.removed_foreground`, `diff.changed_background`, `diff.changed_foreground`, `diff.inline_change_background`, `diff.gutter_added`, `diff.gutter_removed`, `diff.gutter_changed`, `diff.conflict_background`
  - [x] 13.2 Implement alpha transparency blending for diff backgrounds (preserve syntax highlighting)
  - [x] 13.3 Implement theme hot-reload: re-render diff view on theme change without re-running comparison
  - [x] 13.4 Implement high-contrast mode with text decorations (underlines, borders) alongside colour
  - [x] 13.5 Implement gutter markers using `diff.gutter_*` tokens distinct from standard line numbers
  - [x] 13.6 Write unit tests: token resolution, alpha blending math, theme change event handling
  - Covers: Requirement 5 (AC 5.1–5.6)

- [x] 14. Diff navigation
  - [x] 14.1 Register `compare.next_diff` command: advance to next hunk, wrap to first if at end
  - [x] 14.2 Register `compare.prev_diff` command: move to previous hunk, wrap to last if at beginning
  - [x] 14.3 Implement viewport scrolling to centre the target hunk on navigation
  - [x] 14.4 Implement wrap notification via status bar ("navigation wrapped to beginning/end")
  - [x] 14.5 Implement current-diff-index maintenance and visual indicator on focused hunk
  - [x] 14.6 Implement status bar display "Diff N of M" for current position
  - [x] 14.7 Write unit tests: sequential navigation, wrap-around, index tracking, empty diff navigation
  - Covers: Requirement 6 (AC 6.1–6.9)

- [x] 15. Two-way merge operations
  - [x] 15.1 Register merge commands: `compare.accept_left`, `compare.accept_right`, `compare.accept_both`
  - [x] 15.2 Implement accept_left: replace hunk content in merge result with left version
  - [x] 15.3 Implement accept_right: replace hunk content in merge result with right version
  - [x] 15.4 Implement accept_both: insert left then right content sequentially at hunk position
  - [x] 15.5 Implement edit transaction creation for undo-redo integration (each merge accept individually undoable)
  - [x] 15.6 Implement hunk resolution tracking: unresolved, resolved-left, resolved-right, resolved-both, resolved-custom
  - [x] 15.7 Implement visual marking of resolved hunks (dimmed highlight, check gutter indicator)
  - [x] 15.8 Register `compare.accept_all_left` and `compare.accept_all_right` bulk resolution commands
  - [x] 15.9 Implement merge completion detection and status bar notification with save prompt
  - [x] 15.10 Implement merge result as new Document (editable, saveable via VFS, discardable — originals unmodified)
  - [x] 15.11 Write unit tests: individual accepts, bulk accept, undo integration, completion detection, result document state
  - Covers: Requirement 7 (AC 7.1–7.10)

- [x] 16. Three-way merge
  - [x] 16.1 Register `compare.three_way_merge` command accepting base, left, right Resource_URIs
  - [x] 16.2 Implement dual diff computation: base-to-left and base-to-right
  - [x] 16.3 Implement region classification: unchanged, left-only-change, right-only-change, conflict
  - [x] 16.4 Implement automatic resolution of non-conflict regions (unchanged, left-only, right-only)
  - [x] 16.5 Implement conflict highlighting with `diff.conflict_background` theme token
  - [x] 16.6 Implement conflict view showing all three versions (base, left, right) with labels
  - [x] 16.7 Implement conflict resolution actions: accept left, accept right, accept both, manual edit
  - [x] 16.8 Implement workflow integration via workflow-engine: load-resources → compute-diffs → auto-resolve → present-conflicts → await-resolution → save-result
  - [x] 16.9 Implement cancellation support at any workflow step with partial-result save/discard prompt
  - [x] 16.10 Write unit tests: region classification, auto-resolution, conflict detection, workflow step transitions, cancellation
  - Covers: Requirement 8 (AC 8.1–8.10)

- [x] 17. Comparison options and persistence
  - [x] 17.1 Implement `ignore_whitespace` three-mode support: none, leading_trailing, all
  - [x] 17.2 Implement `ignore_case` with Unicode case-folded equality (shared rules with find-and-replace)
  - [x] 17.3 Register toggle commands: `compare.toggle_ignore_whitespace`, `compare.toggle_ignore_case`
  - [x] 17.4 Implement live re-diff on option change for active CompareSession (no re-invoke needed)
  - [x] 17.5 Implement option display in diff view header/toolbar
  - [x] 17.6 Implement option persistence as user preferences via configuration-system
  - [x] 17.7 Write unit tests: option toggle state, re-diff triggering, preference load/save round-trip
  - Covers: Requirement 11 (AC 11.1–11.6)

- [x] 18. Compare Output Panel
  - [x] 18.1 Register dockable panel (panel_id: `compare_output`, default dock zone: Bottom) implementing `DockablePanel` trait
  - [x] 18.2 Implement comparison operation log with timestamps, Resource_URIs, options, and statistics summary
  - [x] 18.3 Implement binary comparison result display (identical/different, sizes, divergence offset)
  - [x] 18.4 Implement error display (resource not found, permission denied, load errors)
  - [x] 18.5 Implement selectable entries: re-open previous comparison on activation
  - [x] 18.6 Register `compare.clear_output` command for history clearing
  - [x] 18.7 Implement panel toggle via standard layout show/hide mechanism
  - [x] 18.8 Write unit tests: log entry creation, entry selection, clear operation
  - Covers: Requirement 13 (AC 13.1–13.7)

- [x] 19. Compare with saved version
  - [x] 19.1 Register `compare.with_saved` command
  - [x] 19.2 Implement fresh content load from VFS for persisted version of active document
  - [x] 19.3 Implement error for unsaved new document: "Document has not been saved. No saved version to compare against."
  - [x] 19.4 Implement no-changes shortcut: status bar notification "No unsaved changes — document matches saved version." (skip diff view)
  - [x] 19.5 Implement pane labelling: left = "Saved: {name}", right = "Unsaved Changes: {name}"
  - [x] 19.6 Implement read-only mode for both panes (no merge operations available)
  - [x] 19.7 Write unit tests: command routing, error cases, label generation, read-only enforcement
  - Covers: Requirement 14 (AC 14.1–14.6)

- [x] 20. Compare with clipboard
  - [x] 20.1 Register `compare.with_clipboard` command
  - [x] 20.2 Implement clipboard text reading as right-side input, active document as left-side input
  - [x] 20.3 Implement error for empty/non-text clipboard: "Clipboard does not contain text content."
  - [x] 20.4 Implement error for no active document: "No active document. Open a file before comparing with clipboard."
  - [x] 20.5 Implement pane labelling: left = "{resource_name}", right = "Clipboard Content"
  - [x] 20.6 Implement selection-aware mode: compare only selected text when selection is active, label left as "Selection in {resource_name}"
  - [x] 20.7 Implement clipboard content as temporary unnamed resource (no URI, no external-change monitoring)
  - [x] 20.8 Write unit tests: command routing, error cases, selection mode, label generation
  - Covers: Requirement 15 (AC 15.1–15.7)

- [x] 21. Compare selections
  - [x] 21.1 Register `compare.mark_selection_for_compare` command to store Selection A with source label
  - [x] 21.2 Register `compare.selections` command to compare Selection A against current selection
  - [x] 21.3 Register `compare.clear_marked_selection` command
  - [x] 21.4 Implement status bar indication when a selection is marked for comparison
  - [x] 21.5 Implement error for missing Selection A: "No selection marked for comparison. Use 'Mark Selection for Compare' first."
  - [x] 21.6 Implement error for empty current selection: "No text selected. Select text to compare against the marked selection."
  - [x] 21.7 Implement pane labelling: left = "Selection A: {source_label}", right = "Selection B: {source_label}" (with document name + line range)
  - [x] 21.8 Implement Selection A persistence across document switches (cleared only on explicit clear or new mark)
  - [x] 21.9 Write unit tests: mark/compare workflow, error cases, persistence across switches, label formatting
  - Covers: Requirement 16 (AC 16.1–16.8)

- [x] 22. Diff export (unified diff format)
  - [x] 22.1 Register `compare.export_diff` command (only available when CompareSession is active)
  - [x] 22.2 Implement unified diff header generation: `--- {left_path}` / `+++ {right_path}` with optional timestamps
  - [x] 22.3 Implement hunk formatting: `@@ -L,S +L,S @@` range header with context/removed/added line prefixes
  - [x] 22.4 Implement configurable context lines (default 3, range 0–999)
  - [x] 22.5 Implement "No newline at end of file" indicator (`\ No newline at end of file`)
  - [x] 22.6 Implement output destinations: copy to clipboard, save to file (VFS picker), open as new document
  - [x] 22.7 Implement options comment header when ignore_whitespace or ignore_case are active
  - [x] 22.8 Implement command disabled state when no active session
  - [x] 22.9 Write unit tests: header formatting, hunk output, context lines, no-newline handling, options header
  - Covers: Requirement 17 (AC 17.1–17.8)

- [x] 23. Property-based tests
  - [x] 23.1 Write PBT: Myers diff optimality property
  - [x] 23.2 Write PBT: diff determinism property
  - [x] 23.3 Write PBT: diff statistics consistency property
  - [x] 23.4 Write PBT: ignore-whitespace equivalence property
  - [x] 23.5 Write PBT: three-way merge region classification completeness property
  - [x] 23.6 Write PBT: unified diff export round-trip property
  - [x] 23.7 Write PBT: navigation wrap-around invariant property
  - [x] 23.8 Write PBT: merge resolution completeness property
  - Covers: All requirements via invariant verification

---

## Property-Based Test Definitions

### Property 1: Myers Diff Optimality

**Validates: Requirement 2.1**

- **Statement:** For any two sequences of lines A and B, the Myers diff algorithm SHALL produce a minimal edit script — the total number of added + removed lines SHALL be less than or equal to that of any other valid edit script transforming A into B.
- **Strategy:** Generate:
  - Left lines: Vec<String> of length [0, 100], each line from a small alphabet of [3, 8] unique lines (to create realistic overlap)
  - Right lines: Vec<String> of length [0, 100], drawn from the same alphabet
- **Invariant:** `edit_distance(result) == lcs_based_minimum(left, right)` where edit_distance = lines_added + lines_removed + 2 * lines_changed

### Property 2: Diff Determinism

**Validates: Requirement 2.9**

- **Statement:** For any two inputs and any set of DiffOptions, running the diff engine multiple times SHALL always produce the identical DiffResult.
- **Strategy:** Generate:
  - Left lines: Vec<String> of length [0, 50]
  - Right lines: Vec<String> of length [0, 50]
  - Options: random DiffOptions (algorithm, ignore_whitespace mode, ignore_case)
  - Run count: 3
- **Invariant:** `diff(left, right, opts) == diff(left, right, opts)` for all runs

### Property 3: Diff Statistics Consistency

**Validates: Requirement 12.1**

- **Statement:** For any DiffResult, the sum of lines_added + lines_removed + lines_changed + lines_unchanged SHALL equal the total number of lines in the longer input, and hunks_count SHALL equal the number of non-Equal hunks in the result.
- **Strategy:** Generate:
  - Left lines: Vec<String> of length [0, 80]
  - Right lines: Vec<String> of length [0, 80]
  - Compute DiffResult and DiffStatistics
- **Invariant:** `stats.lines_unchanged + stats.lines_changed == lines in left matching Equal+Changed hunks` ∧ `stats.lines_added == total lines in Added hunks` ∧ `stats.lines_removed == total lines in Removed hunks` ∧ `stats.hunks_count == result.hunks.iter().filter(|h| !h.is_equal()).count()`

### Property 4: Ignore-Whitespace Equivalence

**Validates: Requirement 2.6, Requirement 11.1**

- **Statement:** For any two inputs where lines differ only in whitespace, enabling `ignore_whitespace = All` SHALL produce a DiffResult with zero non-Equal hunks; enabling `ignore_whitespace = LeadingTrailing` SHALL treat lines differing only in leading/trailing whitespace as equal.
- **Strategy:** Generate:
  - Base lines: Vec<String> of length [1, 50]
  - Modified lines: same content with random whitespace injected (leading, trailing, internal)
  - Options: ignore_whitespace = All, then LeadingTrailing
- **Invariant:** With `All`: `result.hunks.iter().all(|h| h.is_equal())`. With `LeadingTrailing`: lines with only leading/trailing ws changes are Equal; lines with internal ws changes may be Changed.

### Property 5: Three-Way Merge Region Classification Completeness

**Validates: Requirement 8.3**

- **Statement:** For any three inputs (base, left, right), every line region SHALL be classified as exactly one of: unchanged, left-only-change, right-only-change, or conflict. No region SHALL be unclassified, and the union of all classified regions SHALL cover every line in the base.
- **Strategy:** Generate:
  - Base lines: Vec<String> of length [1, 60]
  - Left lines: base with random edits (insertions, deletions, changes) at random positions
  - Right lines: base with independent random edits at random positions
- **Invariant:** `regions.iter().map(|r| r.base_line_count()).sum() == base.len()` ∧ each region has exactly one classification ∧ no gaps between regions

### Property 6: Unified Diff Export Round-Trip

**Validates: Requirement 17.1, 17.3**

- **Statement:** For any DiffResult, exporting to unified diff format and parsing the export back SHALL reproduce an equivalent set of hunks (same line ranges and change types).
- **Strategy:** Generate:
  - Left lines: Vec<String> of length [1, 40] (no embedded diff markers)
  - Right lines: Vec<String> of length [1, 40]
  - Context lines: [0, 5]
  - Compute diff, export to unified format, parse back
- **Invariant:** `parse_unified_diff(export(diff_result)).hunks == diff_result.hunks` (modulo context expansion)

### Property 7: Navigation Wrap-Around Invariant

**Validates: Requirement 6.3, 6.4, 6.5, 6.6**

- **Statement:** For any CompareSession with N difference hunks (N >= 1), invoking `next_diff` N times from the first hunk SHALL visit every hunk exactly once then wrap to the first; invoking `prev_diff` N times from the last hunk SHALL visit every hunk exactly once then wrap to the last.
- **Strategy:** Generate:
  - Number of hunks: [1, 50]
  - Starting position: random index in [0, N-1]
  - Navigation direction: Next or Prev
  - Navigation count: N (full cycle)
- **Invariant:** After N next_diff calls from hunk 0: `visited == {0, 1, ..., N-1}` ∧ `final_position == 0`. After N prev_diff calls from hunk N-1: `visited == {N-1, N-2, ..., 0}` ∧ `final_position == N-1`.

### Property 8: Merge Resolution Completeness

**Validates: Requirement 7.8, 7.9**

- **Statement:** For any CompareSession with N non-Equal hunks, resolving each hunk (via accept_left, accept_right, or accept_both chosen randomly) SHALL result in all hunks being marked as resolved, and the session SHALL report merge-complete status.
- **Strategy:** Generate:
  - Number of non-Equal hunks: [1, 30]
  - Resolution choice per hunk: uniform from {Left, Right, Both}
  - Resolution order: random permutation of hunk indices
- **Invariant:** After all resolutions: `session.all_resolved() == true` ∧ `session.unresolved_count() == 0` ∧ `session.is_merge_complete() == true`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Algorithms", "tasks": ["2", "3", "4", "5", "6"], "dependsOn": [0] },
    { "id": 2, "label": "Session and Resource Loading", "tasks": ["7", "8", "9", "10", "17"], "dependsOn": [1] },
    { "id": 3, "label": "Diff Views and Highlighting", "tasks": ["11", "12", "13"], "dependsOn": [2] },
    { "id": 4, "label": "Navigation and Merge", "tasks": ["14", "15", "16"], "dependsOn": [3] },
    { "id": 5, "label": "Convenience Commands and Export", "tasks": ["18", "19", "20", "21", "22"], "dependsOn": [4] },
    { "id": 6, "label": "Property-Based Tests", "tasks": ["23"], "dependsOn": [5] }
  ]
}
```

---

## Notes

- This is a Wave 14 (File Explorer) crate depending on VFS (Wave 3), document-model and edit-operations (Wave 4), command-framework and workflow-engine (Wave 2), and layout-and-docking and theme-and-appearance (Wave 2)
- The diff engine is GUI-independent: it operates on `&[&str]` line slices and produces pure data structures. All rendering is handled by the view layer.
- The Myers algorithm implementation targets O(ND) time complexity where N = total input length and D = edit distance
- The Patience algorithm anchors on unique matching lines first, then fills between anchors using Myers — producing more readable hunks for structured code
- Three-way merge is modelled as a workflow (via `ff-workflow`) with defined steps, cooperative cancellation, and progress reporting
- Merge operations create edit transactions on a new merge-result Document, integrating with the undo-redo system for individual accept rollback
- All resource access is via VFS — no direct `std::fs` calls permitted. Cross-provider comparison is a first-class use case.
- Binary detection uses a null-byte heuristic (first 8 KB) or provider content-type metadata
- The Compare_Output_Panel registers as a DockablePanel in the Bottom dock zone and logs all comparison operations with timestamps
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Diff export produces POSIX/git-compatible unified diff format suitable for `patch` utility consumption
- The UI rendering tasks (11, 12, 13) define the data model and integration contract; actual egui widget painting is done via the layout-and-docking panel API

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: COMPARE Primary Command | AC 1.1–1.2 | Task 8 |
| Req 1: COMPARE Primary Command | AC 1.3–1.5 | Task 8 |
| Req 1: COMPARE Primary Command | AC 1.6 | Tasks 7, 8 |
| Req 1: COMPARE Primary Command | AC 1.7 | Tasks 7, 8 |
| Req 1: COMPARE Primary Command | AC 1.8 | Task 8 |
| Req 1: COMPARE Primary Command | AC 1.9 | Task 8 |
| Req 1: COMPARE Primary Command | AC 1.10 | Tasks 8, 9 |
| Req 2: Diff Algorithm | AC 2.1 | Task 3 |
| Req 2: Diff Algorithm | AC 2.1a | Task 4 |
| Req 2: Diff Algorithm | AC 2.2 | Tasks 3, 4 |
| Req 2: Diff Algorithm | AC 2.3 | Task 2 |
| Req 2: Diff Algorithm | AC 2.4–2.5 | Task 3 |
| Req 2: Diff Algorithm | AC 2.6–2.7 | Tasks 2, 3 |
| Req 2: Diff Algorithm | AC 2.8 | Task 5 |
| Req 2: Diff Algorithm | AC 2.9 | Tasks 3, 4 |
| Req 2: Diff Algorithm | AC 2.10 | Task 3 |
| Req 3: Side-by-Side View | AC 3.1–3.11 | Task 11 |
| Req 4: Inline View | AC 4.1–4.7 | Task 12 |
| Req 4: Inline View | AC 4.8 | Task 12 |
| Req 5: Diff Highlighting | AC 5.1–5.6 | Task 13 |
| Req 6: Diff Navigation | AC 6.1–6.9 | Task 14 |
| Req 7: Merge Operations | AC 7.1–7.5 | Task 15 |
| Req 7: Merge Operations | AC 7.6–7.8 | Task 15 |
| Req 7: Merge Operations | AC 7.9–7.10 | Task 15 |
| Req 8: Three-Way Merge | AC 8.1–8.3 | Task 16 |
| Req 8: Three-Way Merge | AC 8.4–8.6 | Task 16 |
| Req 8: Three-Way Merge | AC 8.7–8.8 | Task 16 |
| Req 8: Three-Way Merge | AC 8.9–8.10 | Task 16 |
| Req 9: VFS-Aware Comparison | AC 9.1–9.3 | Task 9 |
| Req 9: VFS-Aware Comparison | AC 9.4 | Task 9 |
| Req 9: VFS-Aware Comparison | AC 9.5–9.6 | Task 9 |
| Req 9: VFS-Aware Comparison | AC 9.7 | Task 7 |
| Req 10: Binary Comparison | AC 10.1–10.2 | Task 10 |
| Req 10: Binary Comparison | AC 10.3–10.4 | Tasks 10, 18 |
| Req 10: Binary Comparison | AC 10.5 | Task 9 |
| Req 10: Binary Comparison | AC 10.6 | Task 10 |
| Req 11: Comparison Options | AC 11.1–11.3 | Tasks 2, 17 |
| Req 11: Comparison Options | AC 11.4 | Tasks 7, 17 |
| Req 11: Comparison Options | AC 11.5–11.6 | Task 17 |
| Req 12: Diff Statistics | AC 12.1 | Tasks 2, 6 |
| Req 12: Diff Statistics | AC 12.2 | Tasks 6, 11, 12 |
| Req 12: Diff Statistics | AC 12.3 | Tasks 6, 18 |
| Req 12: Diff Statistics | AC 12.4 | Tasks 6, 7 |
| Req 13: Compare Output Panel | AC 13.1–13.7 | Task 18 |
| Req 14: Compare With Saved | AC 14.1–14.6 | Task 19 |
| Req 15: Compare With Clipboard | AC 15.1–15.7 | Task 20 |
| Req 16: Compare Selections | AC 16.1–16.8 | Task 21 |
| Req 17: Diff Export | AC 17.1–17.8 | Task 22 |
