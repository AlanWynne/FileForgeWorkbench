# Analysis Record: global-search (W5.12)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-global-search` (cross-file search + replace: activation, input/
  options, execution, results display, cross-file replace, search history)
- **Spec files**: requirements.md (209 lines, 6 requirements), tasks.md
  (27 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy

NOT a split candidate. 6 reqs, 209 lines, 5 files, no file over 350. Small, focused crate.
No split.

---

## 2. Cross-unit consistency

### Good layering -- REUSES ff-find-and-replace (no duplication)

`ff-global-search` REUSES the in-file find/replace engine: it imports
`ff_find_and_replace::{MutableSliceIndexer, ChangeRequest, ChangeOutcome, AllLinesFilter,
scope}` for the actual match/substitution logic. It does NOT reimplement the match engine
-- it wraps it across files. Positive layering (contrast the ff-viewers ASA/hex
duplication W5.10). WIRED (36 external refs -- search command + results panel).

### PA-CONFLICT-020 (NEW) -- cross-file REPLACE bypasses the ff-file-ops safe-write path

Req 5 (cross-file replace) writes with RAW std::fs, bypassing ff-file-ops:
- replace.rs:60 `std::fs::read_to_string(&fm.file_path)`, replace.rs:91
  `std::fs::write(&fm.file_path, ...)` -- read + write directly (search.rs:163 reads for
  matching; lines 119/154/177 are test fixtures).
- The doc comment (replace.rs:39) even says "writes back via ..." but the code uses raw
  `std::fs::write`.

This bypasses ff-file-ops' SAFE-WRITE guarantees (atomic rename-on-write, backup copies,
read-only detection -- W4.15). A BULK cross-file replace writing raw (no atomic rename, no
backup) is RISKIER than a single-file edit: a crash mid-replace can leave files partially
written with no backup. Recorded PA-CONFLICT-020 (MEDIUM-HIGH -- data-safety): route the
cross-file replace writes through ff-file-ops' atomic-rename + backup path (the dep is
already declared -- see PA-DEP-005). Cousin of JES raw-fs (PA-CONFLICT-015) but higher
stakes (bulk mutation of user files). Code.

### PA-DEP-005 (NEW) + correction to W4.15

`ff-file-ops` is DECLARED in ff-global-search's Cargo.toml but used 0 times in code
(0 `ff_file_ops` refs -- the replace path uses raw std::fs instead). So it is a DEAD dep
here (PA-DEP-005). IMPORTANT CORRECTION to W4.15: that record concluded ff-file-ops "is
NOT an orphan (consumed by ff-global-search)" based on the Cargo.toml DECLARATION -- but
the code does not actually use it. So ff-file-ops' only declared consumer is a dead-dep;
ff-file-ops is effectively closer to unused than W4.15 stated. (It is still declared, so
not a pure orphan, but the "consumed" claim was declaration-only.) Resolving PA-CONFLICT-020
(actually USE ff-file-ops for the replace writes) would also fix PA-DEP-005 and make the
W4.15 "consumed" claim true. Cross-referenced both records.

### raw-fs search READ (lower stakes)

search.rs:163 `std::fs::read` walks + reads files to search (with the `ignore` crate for
gitignore-aware directory walking). Read-only cross-file search over the real tree is
arguably legitimate (grep-like), lower-stakes than the replace WRITE. Folded into the
PA-CONFLICT-020 discussion (standardise read + write on ff-file-ops / VFS when wired);
not a separate hard finding.

### Public types and ownership

- GlobalSearchEngine, GlobalReplaceEngine, SearchResults, ConflictList, search history --
  sole-owned by `ff-global-search`; match engine BORROWED from ff-find-and-replace
  (correct). No duplication.

### Cross-reference integrity

Cross-refs: ff-find-and-replace USED (good); ff-file-ops DECLARED-unused (PA-DEP-005);
`ignore` + regex + tokio USED. Search command + results panel wired.

---

## 3. Completeness

Tracking: all 27 sub-tasks `[x]`. Implementation present across all 6 reqs (activation,
input/options, execution, results, cross-file replace, history) + WIRED (36 refs). No
PA-INCOMPLETE. Complete. TCR = 10 rows for 6 reqs -- GOOD coverage (no PA-TCR gap).

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 0; `ff-logging` Cargo dep: ABSENT (not dead).
- `std::fs`: search read + replace read/write (PA-CONFLICT-020).

Cross-file search + BULK REPLACE is a meaningful logging site: search scope + match count,
and especially the REPLACE (how many files modified, any write failures, read-only skips)
-- a bulk mutation should log what it changed. Recorded PA-LOG-050 (MEDIUM): add ff-logging
+ dev-logging on search execution (scope, match count) + replace (files modified, failures)
under the `dev-logging` gate. Pairs with the file-ops persistence logging (PA-LOG-040) --
if replace goes through ff-file-ops (PA-CONFLICT-020), that logging lands there.

---

## 5. Task revision proposals

- **PA-CONFLICT-020 (MEDIUM-HIGH -- data-safety)**: route cross-file replace writes
  (replace.rs:60/91) through ff-file-ops' atomic-rename + backup path instead of raw
  std::fs. Bulk mutation without atomic/backup is risky. Code.
- **PA-DEP-005 (LOW, ties PA-CONFLICT-020)**: ff-file-ops declared but unused (0 refs) --
  either use it (preferred, resolves PA-CONFLICT-020) or drop the dep. Also CORRECT the
  W4.15 "consumed by global-search" claim (declaration-only).
- **PA-LOG-050 (MEDIUM)**: add search + replace dev-logging (bulk mutation should log
  what it changed).

No split (no cap issue). No PA-TCR gap (10 rows). No orphan (wired). No ASCII item
(0 non-comment non-ASCII).

---

## Summary

global-search (`ff-global-search`) is a small, focused, WIRED cross-file search + replace
crate (6 reqs, 5 files, 27/27, 36 refs, 10 TCR rows) with GOOD layering -- it REUSES the
ff-find-and-replace match engine (no duplication). Two related findings. PA-CONFLICT-020
(MEDIUM-HIGH, data-safety): cross-file REPLACE writes with RAW std::fs (replace.rs:60/91),
bypassing ff-file-ops' atomic-rename + backup + read-only-detection safe-write path -- a
BULK mutation of user files with no atomic/backup guarantee is risky (a crash mid-replace
can corrupt files). PA-DEP-005 (LOW): ff-file-ops is DECLARED but used 0 times (the replace
path uses raw std::fs) -- a dead dep, AND a CORRECTION to W4.15, whose "ff-file-ops is not
an orphan (consumed by global-search)" conclusion was declaration-only (the code does not
use it). Routing replace through ff-file-ops fixes both + makes the W4.15 claim true. Also
PA-LOG-050 (MEDIUM -- a bulk replace should log what it changed). Otherwise clean: no split,
no ASCII issue, good coverage, correct reuse of the find engine.
