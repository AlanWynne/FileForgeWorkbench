# Analysis Record: clipboard-operations (W4.11)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-clipboard`
- **Spec files**: requirements.md (482 lines, 20 requirements), tasks.md
  (222 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 20 reqs, 482 lines (above thresholds), but ONE cohesive
concern (clipboard: copy/cut/paste, COPY command, file-insert, rectangular +
multi-caret). Well-decomposed into 15 files (copy/paste/entry/splitter/file_insert/
config/...); no file over 400. Below a confident split (1 signal: volume). No split.

---

## 2. Cross-unit consistency

### PA-CONFLICT-002 (transaction model) -- CLEAN SEAM (returns ClipboardEntry data)

Despite DECLARING both `ff-edit-operations` AND `ff-undo-redo` as Cargo deps,
ff-clipboard does NOT use their transaction types:
- `EditorTransaction` usages: 0. `ScrapStack` usages: 0. No non-doc
  `ff_undo_redo::` / `ff_edit_operations::` references (only doc-comment examples).
- It defines its OWN `ClipboardEntry` (stream / rectangular / multi_caret -- copy.rs)
  as the clipboard content model, and PRODUCES it. The CALLER applies the paste as
  an edit (wrapping it in whatever transaction the shell uses).

So clipboard is a clean-seam crate (return data, caller owns the transaction) --
same pattern as sequence-numbers / auto-indent / hex-display -- and stays OUT of
PA-CONFLICT-002. Recorded as a CLEAN row.

### PA-DEP-003 (NEW) -- over-declared (dead) Cargo deps: ff-edit-operations + ff-undo-redo

The FLIP SIDE of the clean seam: ff-clipboard's Cargo.toml declares
`ff-edit-operations` + `ff-undo-redo` (+ ff-line-commands, ff-document-model,
ff-command, ff-vfs, ff-config, ff-logging) but the code uses `EditorTransaction`/
`ScrapStack`/undo-redo 0 times. So at least ff-edit-operations + ff-undo-redo appear
to be DECLARED-BUT-UNUSED dependencies (dead deps, like the dead ff-logging pattern
but for edit/undo). This over-declares the coupling (and slows the build) and
contradicts the clean-seam design (if it returns ClipboardEntry data, it shouldn't
need the transaction crates). Recorded PA-DEP-003 (LOW): audit + prune the unused
ff-clipboard deps (confirm ff-edit-operations / ff-undo-redo / ff-line-commands are
actually used; if only for a trait/type, keep; if unused, remove). Also ties the
dead-ff-logging pattern (PA-LOG below).

### Copy/cut/paste content model -- sole-owned

`ClipboardEntry` (stream/rectangular/multi_caret) -- the clipboard content model,
sole-owned by ff-clipboard. Rectangular + multi-caret clipboard (Req config
`rectangular_paste_adds_lines`) coexists with the edit-operations rectangular/
multi-caret selection model (W1.5) -- clipboard holds the copied SEGMENTS; edit-ops
owns the selection. No duplication (different layer: content vs selection).

### System clipboard + file-insert

COPY command routing (Req) via command-framework (Command_Target). File-insert
(Req) reads a file via ff-vfs (0 raw fs -- clean, VFS-mediated). System clipboard
integration is the shell's (the crate models the content; the OS clipboard bridge
is caller/shell -- consistent with GUI-independence). Consistent.

### Public types and ownership

- `ClipboardEntry` (+ stream/rectangular/multi_caret), copy/cut/paste engines,
  LineSplitter, FileInsertHandler, COPY command -- sole-owned by ff-clipboard. No
  duplication. 0 fs (file-insert via ff-vfs).

### Cross-reference integrity

Cross-refs (document-model, edit-operations, command-framework, undo-redo, vfs,
config, line-commands) resolve as sub-projects -- but several are over-declared
(PA-DEP-003).

---

## 3. Completeness

Tracking: all 222 sub-tasks `[x]`. Implementation present across all 20 reqs
(copy/cut/paste, COPY command, file-insert, rectangular, multi-caret, line-split,
config). Tests in-file. No PA-INCOMPLETE. Complete.

### TCR gap (PA-TCR-023)

TCR.md has 1 row for ff-clipboard against 20 reqs / ~150 criteria. Thin (a 20-req
crate). Recorded PA-TCR-023.

---

## 4. Logging audit

Scan of `crates/ff-clipboard/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 0 (file-insert via ff-vfs)

Clipboard operations are user actions dispatched through command-framework (PA-W0.2
covers COPY). Empty-clipboard / paste-mismatch (paste.rs:104) surface via
ClipboardError. Zero-log largely defensible. Recorded PA-LOG-036 (LOW): resolve the
dead ff-logging dep; optional dev-logging on copy/cut/paste + file-insert. Low
priority.

---

## 5. Task revision proposals

- **PA-DEP-003 (LOW)**: audit + prune over-declared ff-clipboard Cargo deps --
  `ff-edit-operations` + `ff-undo-redo` have 0 usages (EditorTransaction/ScrapStack
  0); confirm ff-line-commands usage. Remove unused deps (the clean-seam design
  means they should not be needed). Also resolve the dead ff-logging dep.
- **PA-STD-051 (ASCII, runtime string)**: 54 non-ASCII bytes (1 non-comment) --
  em-dash in a runtime string/comment (paste.rs:104 "mismatch -- caller should use
  browse"). Replace with `--`. REFACTOR, no gate.
- **PA-LOG-036 (LOW)**: resolve dead ff-logging dep; optional copy/cut/paste
  dev-logging.
- **PA-TCR-023**: enumerate per-requirement TCR rows (1 row for 20 reqs). No code.

No PA-STD size item (no file over 400). No requirement CHANGE proposed.

---

## Summary

clipboard-operations (`ff-clipboard`) is a cohesive, complete clipboard subsystem
(copy/cut/paste, COPY command, file-insert, rectangular + multi-caret), well-sized
(no file over 400). It stays OUT of PA-CONFLICT-002 via the clean-seam pattern: it
produces its own `ClipboardEntry` content model (stream/rectangular/multi_caret) and
the caller applies the paste -- 0 EditorTransaction/ScrapStack usages. The flip side
is PA-DEP-003 (NEW, LOW): it OVER-DECLARES Cargo deps -- `ff-edit-operations` +
`ff-undo-redo` are declared but UNUSED (dead deps, like the dead-ff-logging pattern
but for edit/undo), contradicting the clean-seam design; prune them. File-insert is
VFS-mediated (0 raw fs -- clean). Minor: runtime-string ASCII (PA-STD-051), dead
ff-logging dep (PA-LOG-036, LOW), thin TCR (PA-TCR-023). No conflict, no split.
