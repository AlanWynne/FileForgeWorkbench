# Implementation Plan: Display Line Mapping (`ff-display-line-mapping`)

## Overview

This plan implements the `ff-display-line-mapping` crate — the core editor infrastructure component that maintains the bidirectional mapping between document lines (logical lines in the buffer) and display lines (visual lines rendered in the viewport). The crate supports line exclusion, code folding, word wrap, lazy allocation, large document indexing, and O(log n) lookup performance.

**Crate path:** `crates/ff-display-line-mapping`

**Upstream dependency:** `ff-document-model` (line count, insert/delete notifications)

**Consumers:** `ff-viewport-and-scrolling`, `ff-exclude-show-filter`, `ff-idle-processing`, `ff-line-wrap-toggle`

---

## Tasks

- [x] 1. Crate scaffold and core types
  - [x] 1.1 Create `crates/ff-display-line-mapping/Cargo.toml` with dependencies on `ff-document-model`, `thiserror`, and dev-dependencies on `proptest`, `pretty_assertions`
  - [x] 1.2 Create `src/lib.rs` with crate-level documentation and public re-exports
  - [x] 1.3 Create `src/types.rs` with newtype definitions: `DocLine(usize)`, `DisplayLine(usize)`, `SubLine(usize)`, `LineHeight(usize)`
  - [x] 1.4 Create `src/error.rs` with `DisplayLineMappingError` enum (line out of range, invalid range, mode mismatch)
  - [x] 1.5 Create `src/traits.rs` with the `DisplayLineMapping` public trait defining the full lookup and mutation API (Requirement 7 AC 10)

- [x] 2. Partitioning data structure (Fenwick tree / prefix-sum)
  - [x] 2.1 Create `src/partitioning/mod.rs` re-exporting partition types
  - [x] 2.2 Create `src/partitioning/fenwick_tree.rs` implementing a Fenwick tree (Binary Indexed Tree) over `usize` values supporting O(log n) prefix-sum queries and O(log n) point updates
  - [x] 2.3 Implement `prefix_sum(index)` — returns cumulative sum of heights for indices [0, index)
  - [x] 2.4 Implement `find_prefix_sum(target)` — returns the largest index whose prefix sum is ≤ target (inverse lookup for doc_from_display), O(log n)
  - [x] 2.5 Implement `point_update(index, delta)` — adds `delta` to the value at `index`, O(log n)
  - [x] 2.6 Implement `insert(index, value)` and `delete(index)` for dynamic resizing (rebuild on structural change)
  - [x] 2.7 Write unit tests for Fenwick tree operations (sum, find, update, insert, delete)

- [x] 3. One-to-one mode and lazy allocation
  - [x] 3.1 Create `src/contraction_state.rs` with `ContractionState` struct containing `line_count: usize` and `data: Option<FullTrackingData>` for lazy allocation (Requirement 9 AC 1)
  - [x] 3.2 Implement `FullTrackingData` struct holding: visibility Vec<bool>, expanded Vec<bool>, heights Vec<usize>, fold_display_texts Vec<Option<String>>, and the Fenwick tree partitioning
  - [x] 3.3 Implement `ensure_data()` private method that lazily allocates `FullTrackingData` on first non-trivial operation (Requirement 9 AC 2)
  - [x] 3.4 Implement one-to-one mode fast paths for `display_from_doc`, `doc_from_display`, `get_visible`, `get_expanded`, `get_height` (Requirement 9 AC 5)
  - [x] 3.5 Implement `insert_lines` and `delete_lines` in one-to-one mode (line count update only, Requirement 9 AC 6)
  - [x] 3.6 Write unit tests for one-to-one mode identity mapping and lazy allocation trigger

- [x] 4. Document-to-display mapping (Requirement 1)
  - [x] 4.1 Implement `display_from_doc(doc_line)` using Fenwick tree prefix sum (Requirement 1 AC 1)
  - [x] 4.2 Implement `display_from_doc_sub(doc_line, sub_line)` with sub-line clamping (Requirement 1 AC 2)
  - [x] 4.3 Implement `display_last_from_doc(doc_line)` returning last display line of a doc line (Requirement 1 AC 3)
  - [x] 4.4 Implement `doc_from_display(display_line)` using Fenwick tree `find_prefix_sum` (Requirement 1 AC 4)
  - [x] 4.5 Implement clamping for out-of-range display line lookups (Requirement 1 AC 5, AC 6)
  - [x] 4.6 Implement `lines_in_doc()` and `lines_displayed()` accessors (Requirement 1 AC 7, AC 8)
  - [x] 4.7 Write unit tests for forward/reverse mapping with various visibility and height patterns

- [x] 5. Line exclusion and hiding (Requirement 2)
  - [x] 5.1 Implement `set_visible(start_line, end_line, is_visible)` with range validation and Fenwick tree updates (Requirement 2 AC 1)
  - [x] 5.2 Implement `get_visible(doc_line)` returning boolean visibility (Requirement 2 AC 2)
  - [x] 5.3 Implement Display_Line_Count adjustment on visibility changes (Requirement 2 AC 3, AC 4)
  - [x] 5.4 Implement `hidden_lines()` returning whether any line is hidden (Requirement 2 AC 5)
  - [x] 5.5 Implement `show_all()` — deallocate tracking, return to one-to-one mode (Requirement 2 AC 6, Requirement 9 AC 3)
  - [x] 5.6 Implement boundary validation for invalid ranges (Requirement 2 AC 7)
  - [x] 5.7 Write unit tests for hide/show operations and display count invariant (Requirement 2 AC 8)

- [x] 6. Code folding state (Requirement 3)
  - [x] 6.1 Implement `set_expanded(doc_line, is_expanded)` storing fold state per line (Requirement 3 AC 1)
  - [x] 6.2 Implement `get_expanded(doc_line)` returning fold state (Requirement 3 AC 2)
  - [x] 6.3 Implement `expand_all()` setting all folds to expanded (Requirement 3 AC 3)
  - [x] 6.4 Implement `contracted_next(start_line)` finding next collapsed fold header (Requirement 3 AC 4)
  - [x] 6.5 Implement `set_fold_display_text(doc_line, text)` and `get_fold_display_text(doc_line)` (Requirement 3 AC 7, AC 8)
  - [x] 6.6 Write unit tests for fold state management, expand_all, contracted_next

- [x] 7. Word wrap mapping (Requirement 4)
  - [x] 7.1 Implement `set_height(doc_line, height)` with Fenwick tree delta updates for visible lines (Requirement 4 AC 1, AC 5)
  - [x] 7.2 Implement `get_height(doc_line)` returning stored height or 1 in one-to-one mode (Requirement 4 AC 2)
  - [x] 7.3 Implement hidden-line height storage without display count impact (Requirement 4 AC 6)
  - [x] 7.4 Implement range validation for set_height (Requirement 4 AC 7)
  - [x] 7.5 Write unit tests for wrap height changes and sub-line contiguity (Requirement 4 AC 8)

- [x] 8. Incremental updates (Requirement 6)
  - [x] 8.1 Implement `insert_lines(doc_line, count)` — insert entries into visibility, expanded, heights arrays and rebuild/update Fenwick tree (Requirement 6 AC 1)
  - [x] 8.2 Implement `delete_lines(doc_line, count)` — remove entries, adjust display count, rebuild/update Fenwick tree (Requirement 6 AC 2)
  - [x] 8.3 Optimize insert/delete to O(count × log n) by batched Fenwick rebuilds (Requirement 6 AC 3, AC 4)
  - [x] 8.4 Write unit tests for insert/delete maintaining the display count invariant (Requirement 6 AC 7)

- [x] 9. Large document support (Requirement 8)
  - [x] 9.1 Implement generic `ContractionState<Idx>` parameterized over index type (`u32` or `u64`) or use `usize` with compile-time cfg (Requirement 8 AC 1)
  - [x] 9.2 Implement constructor with `large_document: bool` flag selecting internal index width (Requirement 8 AC 5)
  - [x] 9.3 Ensure public API uses `usize` regardless of internal representation (Requirement 8 AC 4)
  - [x] 9.4 Write unit test verifying 32-bit mode memory usage is less than 64-bit equivalent (Requirement 8 AC 6)

- [x] 10. Dual hiding mechanism support (Requirement 10)
  - [x] 10.1 Verify that visibility and expanded state are stored independently (Requirement 10 AC 1)
  - [x] 10.2 Ensure `set_visible` works for both ISPF exclusion and fold engine (Requirement 10 AC 2)
  - [x] 10.3 Verify `set_expanded` is orthogonal to visibility (Requirement 10 AC 3)
  - [x] 10.4 Implement `show_all()` resetting both exclusion and fold state (Requirement 10 AC 6)
  - [x] 10.5 Verify mapping layer does NOT store fold levels or region extents (Requirement 10 AC 7)
  - [x] 10.6 Write unit tests for dual-mechanism coexistence scenarios (ISPF exclusion + fold collapse on overlapping ranges)

- [x] 11. Integration points and trait implementation (Requirement 7)
  - [x] 11.1 Implement `DisplayLineMapping` trait on `ContractionState` (Requirement 7 AC 10)
  - [x] 11.2 Implement change notification mechanism (callback/observer pattern) for Display_Line_Count changes (Requirement 7 AC 9)
  - [x] 11.3 Create `src/watcher.rs` with `MappingChangeListener` trait and notification dispatch
  - [x] 11.4 Document integration contracts for viewport, scrollbar, gutter, and find subsystems (Requirement 7 AC 1–8)
  - [x] 11.5 Write integration test demonstrating viewport-style usage pattern (scroll → translate → render)

- [x] 12. Property-based tests for mapping invariants
  - [x] 12.1 Write property test: roundtrip invariant — for all visible lines d, `doc_from_display(display_from_doc(d)) == d` (Requirement 1 AC 10)
    - **Validates: Requirement 1.10**
  - [x] 12.2 Write property test: display count invariant — `lines_displayed() == sum(get_height(d) for all visible d)` (Requirement 6 AC 7)
    - **Validates: Requirement 6.7**
  - [x] 12.3 Write property test: hidden lines contribute zero display lines — hiding a line decreases display count by its height (Requirement 2 AC 8)
    - **Validates: Requirement 2.8**
  - [x] 12.4 Write property test: insert/delete line count consistency — after insert_lines(pos, n), lines_in_doc() increases by n; after delete_lines(pos, n), decreases by n (Requirement 6 AC 1, AC 2)
    - **Validates: Requirements 6.1, 6.2**
  - [x] 12.5 Write property test: set_height on visible line adjusts display count by exactly (new - old) (Requirement 4 AC 5)
    - **Validates: Requirement 4.5**
  - [x] 12.6 Write property test: one-to-one mode identity — when no lines hidden and all heights are 1, display_from_doc(n) == n and doc_from_display(n) == n (Requirement 1 AC 9)
    - **Validates: Requirement 1.9**
  - [x] 12.7 Write property test: sub-line contiguity — for a visible line with height h, display_from_doc_sub(d, 0..h-1) returns h contiguous values (Requirement 4 AC 8)
    - **Validates: Requirement 4.8**
  - [x] 12.8 Write property test: show_all restores one-to-one mode — after arbitrary hide/fold/wrap operations, show_all() returns display_from_doc(n) == n for all n (Requirement 2 AC 6)
    - **Validates: Requirement 2.6**

- [x] 13. Performance validation (Requirement 5)
  - [x] 13.1 Write benchmark test for `display_from_doc` on 1M-line document verifying sub-microsecond lookup (Requirement 5 AC 6)
  - [x] 13.2 Write benchmark test for `doc_from_display` on 1M-line document verifying sub-microsecond lookup (Requirement 5 AC 6)
  - [x] 13.3 Write benchmark test for `set_visible` range update verifying O(range × log n) scaling (Requirement 5 AC 5)
  - [x] 13.4 Write benchmark confirming one-to-one mode returns in O(1) with no allocation (Requirement 5 AC 4)

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered By Task(s) |
|---|---|---|
| 1 (Doc↔Display Mapping) | AC 1–10 | 4.1–4.7, 12.1, 12.6 |
| 2 (Line Exclusion) | AC 1–8 | 5.1–5.7, 12.3, 12.8 |
| 3 (Code Folding) | AC 1–10 | 6.1–6.6 |
| 4 (Word Wrap) | AC 1–8 | 7.1–7.5, 12.5, 12.7 |
| 5 (Performance) | AC 1–6 | 2.2–2.6, 13.1–13.4 |
| 6 (Incremental Updates) | AC 1–7 | 8.1–8.4, 12.2, 12.4 |
| 7 (Integration Points) | AC 1–10 | 11.1–11.5 |
| 8 (Large Document) | AC 1–6 | 9.1–9.4 |
| 9 (Lazy Allocation) | AC 1–7 | 3.1–3.6 |
| 10 (Dual Hiding) | AC 1–8 | 10.1–10.6 |

---

## Notes

- The Fenwick tree (Binary Indexed Tree) is chosen over a segment tree for its lower constant factor and simpler implementation while still providing O(log n) prefix-sum queries and updates
- The `design.md` for this crate may be generated concurrently; if API signatures differ from this plan, defer to design.md
- Task 9 (large document support) may use Rust generics over index width or a runtime enum; the approach should align with `ff-document-model`'s `LineNumber(u64)` pattern
- Property-based tests (task 12) use the `proptest` crate with a minimum of 100 cases per property
- Performance benchmarks (task 13) use `criterion` and are informational — they do not block task completion but regressions should be investigated
- The `DisplayLineMapping` trait (task 1.5 / 11.1) is the primary public interface for downstream consumers; the concrete `ContractionState` type may remain `pub(crate)` if desired
- This crate does NOT store fold levels, fold nesting depth, or fold region extents — those belong to the syntax/language layer (Requirement 10 AC 7)
- ISPF exclusion is flat (non-hierarchical), while code folding is hierarchical; the mapping layer treats both as boolean visibility per line

---

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 0,
      "label": "Crate scaffold and core types",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5"],
      "dependsOn": []
    },
    {
      "id": 1,
      "label": "Partitioning data structure (Fenwick tree)",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7"],
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "One-to-one mode and lazy allocation",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6"],
      "dependsOn": [0, 1]
    },
    {
      "id": 3,
      "label": "Document-to-display mapping",
      "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7"],
      "dependsOn": [2]
    },
    {
      "id": 4,
      "label": "Line exclusion and hiding",
      "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7"],
      "dependsOn": [3]
    },
    {
      "id": 5,
      "label": "Code folding state",
      "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6"],
      "dependsOn": [3]
    },
    {
      "id": 6,
      "label": "Word wrap mapping",
      "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5"],
      "dependsOn": [3]
    },
    {
      "id": 7,
      "label": "Incremental updates",
      "tasks": ["8.1", "8.2", "8.3", "8.4"],
      "dependsOn": [4, 6]
    },
    {
      "id": 8,
      "label": "Large document support",
      "tasks": ["9.1", "9.2", "9.3", "9.4"],
      "dependsOn": [2]
    },
    {
      "id": 9,
      "label": "Dual hiding mechanism support",
      "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6"],
      "dependsOn": [4, 5]
    },
    {
      "id": 10,
      "label": "Integration points and trait implementation",
      "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5"],
      "dependsOn": [7, 9]
    },
    {
      "id": 11,
      "label": "Property-based tests for mapping invariants",
      "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8"],
      "dependsOn": [7, 9]
    },
    {
      "id": 12,
      "label": "Performance validation",
      "tasks": ["13.1", "13.2", "13.3", "13.4"],
      "dependsOn": [10]
    }
  ]
}
```
