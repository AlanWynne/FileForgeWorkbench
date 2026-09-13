# Analysis Record: file-operations (W4.15)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-file-ops` (user-facing file commands -- New, Open, Save,
  Save As, Revert, Recent Files -- plus persistence: atomic rename-on-write, backup
  copies, read-only detection)
- **Spec files**: requirements.md (258 lines, 10 requirements), tasks.md
  (160 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 10 reqs, 258 lines. Cohesive concern (file lifecycle
commands + persistence). 19 files, no file over 400 (well-decomposed: commands,
save, backup, revert, guard, recent, resource_uri, ...). No split.

---

## 2. Cross-unit consistency

### NOT an orphan (breaks the Wave-4 streak)

`ff-file-ops` IS consumed -- `ff-global-search` depends on it. Unlike the four
Wave-4 orphans (ff-file-tree, ff-idle-processing, ff-large-file-performance,
ff-external-mod), file-ops is wired into at least one consumer. (Shell wiring of the
New/Open/Save commands is via command-framework dispatch -- PA-W0.2 family.)

### PA-CONFLICT-014 cross-ref (external-mod placement) -- RESOLVED to standalone by construction

The external-modification spec (W4.14) left its placement unresolved ("part of the
`ff-file-operations` crate OR a standalone crate"). Verified from THIS side:
`ff-file-ops` does NOT depend on `ff-external-mod` (0) and contains no external-change
detection. So external-mod was built STANDALONE, not folded into file-ops. This
CONFIRMS the PA-CONFLICT-014 recommendation (keep ff-external-mod standalone) is the
as-built reality -- the remaining work there is wiring, not re-housing. No new
conflict; cross-reference recorded.

### PA-CONFLICT-002 (transaction model) -- CLEAN SEAM (reads dirty state, no transactions)

file-ops declares `ff-undo-redo` but uses it 0 times (Transaction 0, no non-doc
`ff_undo_redo::`). It takes `is_dirty: bool` as a parameter (commands.rs
`is_save_enabled`) and reads `document.is_dirty()` (guard.rs, revert.rs) -- the dirty
flag is owned by document-model; file-ops does not drive the undo stack. Revert is a
fresh reload gated by `needs_revert_confirmation(is_dirty)`, not an undo-stack
operation. So file-ops stays OUT of PA-CONFLICT-002 (clean seam: reads dirty state,
does not own transactions). Consistent with the clean-seam crates (clipboard,
seqnum, ...).

### PA-DEP-004 (NEW) -- over-declared (dead) deps: ff-undo-redo (+ ff-logging)

The flip side of the clean seam (as with ff-clipboard PA-DEP-003): file-ops declares
`ff-undo-redo` but uses it 0 times -- a dead dep. Plus `ff-logging` declared, 0 uses
(dead). Recorded PA-DEP-004 (LOW): audit + prune -- confirm ff-undo-redo is unused and
remove; resolve the dead ff-logging dep (or start using it, see PA-LOG-040). Same
over-declaration family as PA-DEP-003 + the dead-ff-logging cluster.

### FFW-ARCH VFS discipline -- CLEAN (0 real fs)

lib.rs documents "all file I/O goes through the VFS abstraction (ff-vfs) -- no std::fs
calls". Verified: the 1 std::fs grep hit is that prohibition doc-comment; actual fs
usage is 0. Atomic rename-on-write, backup copies, and read-only detection all go
through ff-vfs (backup.rs uses `resource_uri::backup_uri_alongside`). Exemplary VFS
discipline (same as external-mod W4.14) -- important for a persistence crate.

### Public types and ownership

- File commands (New/Open/Save/Save As/Revert/Recent), `BackupConfig`/`BackupLocation`,
  atomic-rename + read-only persistence, resource_uri helpers -- sole-owned by
  `ff-file-ops`. Dirty flag READ from document-model (not owned here). No duplication.

### Cross-reference integrity

Cross-refs (ff-vfs, command-framework, document-model, config) resolve as
sub-projects and are used. ff-undo-redo declared-unused (PA-DEP-004). External-mod
placement confirmed standalone (cross-ref W4.14).

---

## 3. Completeness

Tracking: all 160 sub-tasks `[x]`. Implementation present across all 10 reqs
(New/Open/Save/Save As/Revert/Recent + atomic rename, backup, read-only detection).
Consumed by global-search. Tests in-file. No PA-INCOMPLETE. Complete.

### TCR gap (PA-TCR-027)

TCR.md has 2 rows for ff-file-ops across 10 reqs / ~90 criteria. Thin (better than
the four orphans' zero, but under-enumerated). Recorded PA-TCR-027.

---

## 4. Logging audit

Scan of `crates/ff-file-ops/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0 (the 1 hit is the VFS-prohibition doc-comment)

Persistence is a HIGH-VALUE logging site: Save / atomic rename-on-write / backup
creation / read-only detection / Revert are exactly the operations where a silent
failure or partial write is dangerous and hard to diagnose. ff-logging is a DEAD dep
here. Recorded PA-LOG-040 (MEDIUM -- above the usual LOW): resolve the dead dep AND add
dev-logging on save/atomic-rename/backup/read-only/revert outcomes under the
`dev-logging` gate. Strong CR-NR-058 candidate (data-safety-relevant), alongside
external-mod PA-LOG-039.

---

## 5. Task revision proposals

- **PA-DEP-004 (LOW)**: prune over-declared deps -- `ff-undo-redo` unused (Transaction
  0); resolve dead `ff-logging` (or use it, PA-LOG-040). Same family as PA-DEP-003.
- **PA-STD-055 (ASCII, runtime strings)**: 91 non-ASCII bytes (11 non-comment) --
  em-dashes in error.rs (9) + save.rs (2). Replace with `--`. REFACTOR, no gate.
- **PA-LOG-040 (MEDIUM)**: resolve dead ff-logging + add save/rename/backup/read-only/
  revert dev-logging (data-safety-relevant; strong CR-NR-058 candidate).
- **PA-TCR-027**: enumerate per-requirement TCR rows (2 for 10 reqs). No code.

No PA-STD size item (no file over 400). No PA-CONFLICT (file-ops itself is clean; it
CONFIRMS PA-CONFLICT-014's standalone recommendation). No requirement CHANGE.

---

## Summary

file-operations (`ff-file-ops`) is a cohesive, complete file-lifecycle + persistence
crate (New/Open/Save/Save As/Revert/Recent, atomic rename-on-write, backups,
read-only detection; 160/160, 19 files, no file over 400). It BREAKS the Wave-4 orphan
streak -- it is consumed by ff-global-search. It has EXEMPLARY VFS discipline (0 real
fs; the single std::fs hit is the prohibition doc-comment) and stays OUT of
PA-CONFLICT-002 via the clean-seam pattern (reads `document.is_dirty()`, does not own
transactions; Revert is a gated reload). It CONFIRMS PA-CONFLICT-014: file-ops does
NOT contain external-mod (0 dep), so ff-external-mod was built standalone -- the
remaining external-mod work is wiring, not re-housing. Findings are hygiene:
PA-DEP-004 (NEW, LOW) over-declared/dead deps (ff-undo-redo unused + dead ff-logging),
PA-LOG-040 (MEDIUM) -- persistence is a high-value, data-safety-relevant logging site
yet ff-logging is dead (strong CR-NR-058 candidate) -- plus PA-STD-055 (runtime-string
ASCII) and PA-TCR-027 (thin, 2/10). No conflict, no split.
