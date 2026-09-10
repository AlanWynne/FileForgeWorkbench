# Analysis Record: document-model

- **Wave**: 0 (foundation -- text storage layer under the whole editor)
- **Backing crate**: `ff-document-model`
- **Spec folder**: `docs/specs/document-model/`
- **Analysed**: Wave 0, task W0.9 (CR-NR-057 re-baseline)
- **Verdict**: COMPLETE (140/140 tasks `[x]`, TCR PASS). NOT a split candidate.
  VFS-only verified. Zero-log defensible. One file-size WATCH (document.rs at cap).
- **CR-NR-057 impact**: NONE (document-model specs untouched). Re-verified.

---

## 1. Scope summary

`ff-document-model` is the GUI-independent text-storage foundation. 10
requirements: Req 1 gap-buffer storage (u64, >2GB); Req 2 insert/delete +
read-only; Req 3 line index (O(log n) bidirectional); Req 4 streaming VFS load
(async, cancellable); Req 5 line-end modes; Req 6 lifecycle + shared ownership
(DocumentHandle = Arc<RwLock<Document>>); Req 7 DocumentWatcher; Req 8
encoding-aware char navigation; Req 9 viewport top_line + clamped scroll;
Req 10 save-point / modification state.

248 req lines, 10 requirements, single backing crate. Cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 248 lines; 10 reqs | No |
| 3+ distinct responsibilities | storage/lineindex/streaming/nav/lifecycle are layers of ONE text model; adjacent concerns already separate crates (edit-operations, display-line-mapping, undo-redo) | No |
| 2+ crates | one crate | No |
| low-cohesion clusters | high cohesion | No |
| file-size pressure | document.rs 399 non-test (AT cap); text_buffer.rs 354 | Watch (not a split driver) |

0 split criteria. **NOT a split candidate.** Well factored (16 modules).

## 3. Consistency / conflict

- Public types owned here (GapBuffer, TextBuffer, Document, DocumentHandle,
  LineIndex, SparseLineIndex, StreamingFileReader, LineEndMode, BytePosition,
  LineNumber, CharacterExtracted, DocumentWatcher, TopLine, SplitView,
  DocumentError) -- sole owner. Added to consistency-matrix.
- VFS (ff-vfs): all I/O via `ff_vfs` (Req 4.8). VERIFIED no std::fs/tokio::fs.
  Correct direction (FFW-ARCH-001).
- undo-redo (insert/delete notify undo); display-line-mapping consumes LineIndex;
  edit-operations consumes insert/delete primitives; encoding-and-characters owns
  broad encoding while this crate owns UTF-8 nav; LineEndMode owned here,
  validated by encoding-and-characters. Boundaries match spec Cross-References. No
  conflict.
- Req 2.9 restates command-framework mutation-routing (consistent with
  command-framework Req 2.7). No conflict.

## 4. Completeness

- Tasks: 140 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-document-model` PASS (integration_tests.rs, property_tests.rs).
- Req 4.8 VFS-only VERIFIED in code. Streaming-failure path surfaces errors via
  DocumentError / LoadingProgress::Failed (no silent swallow). unwrap/expect: 0 in
  lib code (all in test modules).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` declared (wired, unused). DEFENSIBLE: pure
  GUI-independent library; all errors surface via Result / LoadingProgress::Failed
  / watcher notifications. No requirement mandates a log record. One of the 56
  zero-log crates (PA-LOG-002) but a legitimate case.
- Optional enhancement (PA-LOG-003, carried): a `log_warn!` on streaming-load
  failure (Req 4.6) and `log_debug!` on read-only modify-attempt (Req 7.4) would
  aid bug reporting. Owner-gated; not a completeness blocker.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a pure library)**.

## 6. Findings logged

- **PA-WATCH-002** (file-size WATCH): `document.rs` is 399 non-test lines --
  exactly at the 400 cap. Any further non-test addition requires a concurrent
  split (e.g. `document_nav.rs` / `document_scroll.rs`). REFACTOR on next touch;
  no gate.
- **PA-LOG-003** (LOGGING ENHANCEMENT, optional): zero log calls despite the
  `ff-logging` dep; defensible for a pure library. Optional: log the
  streaming-load failure (Req 4.6) + read-only modify-attempt (Req 7.4).
  Owner-gated; not a blocker.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-document-model contributes 21
  matches, MOSTLY legitimate Unicode test fixtures (U+00E9, U+0085 NEL, U+2028 LS,
  U+2029 PS used to test line-end detection). The project-wide cleanup MUST
  exclude genuine Unicode test/data literals here. Rolled into PA-LOG-001 with
  this caveat.
