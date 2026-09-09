# Analysis Record: document-model

- **Wave**: 0 (foundation -- text storage layer under the whole editor)
- **Backing crate**: `ff-document-model`
- **Spec folder**: `docs/specs/document-model/`
- **Analysed**: Wave 0, task W0.9
- **Verdict**: COMPLETE (all 140 tasks `[x]`, TCR PASS). NOT a split candidate.
  One WATCH (document.rs at 399/400 line cap) and one optional logging
  enhancement (zero log calls on failure paths).

---

## 1. Scope summary

`ff-document-model` is the GUI-independent text-storage foundation. 10
requirements:

- Req 1 gap-buffer storage (u64 positions, >2GB); Req 2 insert/delete with line
  tracking + read-only; Req 3 line index (O(log n) bidirectional); Req 4
  streaming VFS load (async, cancellable, sparse index); Req 5 line-end modes
  (Default / Unicode LS/PS/NEL); Req 6 lifecycle + shared ownership
  (DocumentHandle = Arc<RwLock<Document>>); Req 7 DocumentWatcher notifications;
  Req 8 encoding-aware char navigation; Req 9 viewport top_line + clamped
  scroll arithmetic; Req 10 save-point / modification state.

248 req lines, 10 requirements, single backing crate. Cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 248 lines; 10 reqs | No |
| 3+ distinct responsibilities | storage/lineindex/streaming/navigation/lifecycle are layers of ONE text model, tightly coupled through TextBuffer/Document | No |
| 2+ crates | one crate `ff-document-model`; higher edit/display/undo concerns are already SEPARATE crates (edit-operations, display-line-mapping, undo-redo) | No |
| low-cohesion clusters | high cohesion -- every module feeds Document | No |
| file-size pressure | document.rs 399 non-test (AT the 400 cap); text_buffer.rs 354 | Watch (not a split driver) |

0 split criteria met. **NOT a split candidate.** The model is already well
factored (16 focused modules), and adjacent concerns are separate crates.

## 3. Consistency / conflict

- Public types owned here (GapBuffer, TextBuffer, Document, DocumentHandle,
  LineIndex, SparseLineIndex, StreamingFileReader, LineEndMode, BytePosition,
  LineNumber, CharacterExtracted, DocumentWatcher, TopLine, SplitView,
  DocumentError) -- sole owner `ff-document-model`. Added to consistency-matrix.
- Cross-refs are correctly one-directional:
  - VFS (ff-vfs): document-model consumes `ff_vfs::{Vfs, ResourceUri}` for ALL
    I/O (Req 4.8). VERIFIED: no `std::fs` / `tokio::fs` anywhere in src.
    Correct architecture direction (FFW-ARCH-001).
  - undo-redo-transactions: insert/delete notify undo; that spec is authoritative
    for transaction semantics -- no duplication here.
  - display-line-mapping consumes LineIndex; edit-operations consumes
    insert/delete primitives; encoding-and-characters owns broad encoding while
    this crate owns UTF-8 nav. Boundaries are stated in the spec Cross-References
    section and match the actual crate split. No conflict.
- Req 2.9 restates the command-framework routing rule (mutations routable
  through ff-command). Consistent with command-framework Req 2.7. No conflict.

## 4. Completeness

- Tasks: 140 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-document-model` row PASS (integration_tests.rs, property_tests.rs
  covering insert/delete, line counting, byte positions, streaming load).
- Req 4.8 (VFS-only I/O) verified in code. Req 4.6 streaming-failure path
  verified: VFS error -> `DocumentError::VfsIo`; cancellation ->
  `DocumentError::LoadCancelled`; read error -> `LoadingProgress::Failed {
  reason, bytes_loaded }` (partial content preserved, error surfaced). No error
  is silently swallowed.
- unwrap/expect scan: 45 hits, ALL inside `#[cfg(test)]` modules (test setup);
  none in lib code. No silent-error risk.
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Log-macro call sites: ZERO. `ff-document-model` is one of the 56 zero-log
  crates flagged in PA-LOG-002. BUT:
  - `ff-logging` IS a declared dependency (wired, unused).
  - Every failure path surfaces via `Result` / `LoadingProgress::Failed` /
    watcher notifications (Req 4.6, 7.4). A pure GUI-independent library
    legitimately defers logging to its consumers (background-io, ff-desktop),
    which is idiomatic and consistent with the crate's stated GUI-independence.
  - Therefore zero-log is DEFENSIBLE, not a requirements violation. No spec
    criterion mandates a log record from this crate (contrast ff-config /
    ff-logging which have explicit WARN criteria).
- Optional enhancement (PA-LOG-003): because the streaming-load failure path
  (Req 4.6) and read-only modify-attempt (Req 7.4) are prime bug-report signals,
  a low-risk improvement would be a `log_warn!` on VfsIo/Failed and a
  `log_debug!` on modify-attempt. Optional, owner-gated; NOT required for
  completeness.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a pure library)**.

## 6. Findings logged

- **PA-WATCH-002** (file-size watch): `document.rs` is 399 non-test lines --
  exactly at the rust-standards.md 400-line cap. Any further non-test addition
  requires a concurrent split (candidate split targets per rust-standards: a
  `document_nav.rs` for the encoding-nav delegation or a `document_scroll.rs`
  for viewport methods). REFACTOR when next touched; no gate.
- **PA-LOG-003** (optional logging enhancement): `ff-document-model` has zero log
  calls despite depending on `ff-logging`. Defensible for a pure library, but
  adding `log_warn!` on the streaming-load failure path (Req 4.6) and a
  `log_debug!` on read-only modify-attempt (Req 7.4) would aid bug reporting.
  Optional, owner-gated; not a completeness blocker.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-document-model contributes
  21 non-ASCII matches (mostly U+00E9 and Unicode line-separator literals in
  navigation tests, plus doc-comment prose). The test-data code points (U+00E9,
  U+0085, U+2028, U+2029) are legitimate test fixtures; documentation.md's ASCII
  rule targets prose/separators, not intentional Unicode test inputs. Recommend
  the project-wide cleanup EXCLUDE genuine Unicode test literals here. Rolled
  into PA-LOG-001 with this caveat; not fixed here.
