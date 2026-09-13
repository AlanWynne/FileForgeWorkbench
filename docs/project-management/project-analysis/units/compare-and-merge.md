# Analysis Record: compare-and-merge (W5.11)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-compare-merge` (COMPARE command, LCS/Myers/Patience line diff,
  side-by-side + inline diff views, navigation, merge accept-left/right/both, three-way
  merge, VFS-aware resource comparison, binary compare, diff stats/export)
- **Spec files**: requirements.md (384 lines, 17 requirements), tasks.md
  (183 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + cap (PA-STD-066)

17 reqs / 384 lines. The crate is cohesive (one compare/merge concern), so not a strong
CRATE-split candidate -- BUT two files are at/over the cap AND one is mis-housed:

- `session.rs` = 557 non-test lines -- and its module doc says "Diff engine: Myers and
  Patience algorithms operating on line slices" (`pub struct DiffEngine`). So the file
  named `session.rs` actually IS the DIFF ENGINE (Req 2), not session state. That is both
  a cap violation AND a misleading filename.
- `merge.rs` = 392 (near cap).

Recorded PA-STD-066 (MEDIUM -- cap + naming): rename/extract the diff engine out of
session.rs into `diff_engine.rs` (Myers) + `patience.rs` (or a `diff/` submodule),
leaving genuine session/navigation state in a correctly-named file; split merge.rs if it
grows. REFACTOR, no behaviour change. (The file-vs-content name mismatch is worth fixing
regardless of size.)

---

## 2. Cross-unit consistency -- CLEAN (wired, clean-seam, VFS-aware by interface)

### WIRED (NOT an orphan)

`ff-compare-merge` is referenced 156x outside the crate (COMPARE command + diff views +
CompareSession). NOT an orphan -- another Wave-5 positive after the ff-viewers /
language-service orphans.

### Clean-seam views (data model, shell renders)

Despite Reqs 3/4/13 (side-by-side + inline + output panel), the crate has NO egui dep
(sole dep thiserror). The diff views are DATA MODELS the shell renders -- the same
clean-seam pattern as ff-asa (W5.7). GUI-independent + testable. Positive.

### VFS-aware by INTERFACE (0 raw fs) -- positive

Req 9 (VFS-aware resource comparison across any registered provider) is handled via
URI-based `CompareSource` / `CompareSession` (error carries `uri`) with content provided
to the engine -- so it is VFS-aware BY INTERFACE without a hard ff-vfs dep or raw fs
(0 std::fs). Clean (contrast the JES raw-fs bypass PA-CONFLICT-015). Compare-with-saved
(Req 14), compare-with-clipboard (Req 15), compare-selections (Req 16) are consumer-side
integrations over the same engine.

### No duplication

LCS/Myers/Patience diff + merge is sole-owned by `ff-compare-merge`. No other crate does
line differencing (distinct from syntax tokenizing). No overlap. (Note: unlike ff-viewers
W5.10, this crate does NOT reimplement a peer -- it is the single diff/merge owner.)

### Naming drift (PA-DOC-009)

Spec calls it `ff-compare`; the crate dir is `ff-compare-merge`. Naming-reconciliation
set. Docs only.

### Cross-reference integrity

Cross-refs (COMPARE command, theme integration Req 5, clipboard Req 15, document saved
version Req 14) resolve as sub-projects and are wired (156 refs).

---

## 3. Completeness

Tracking: all 183 sub-tasks `[x]`. Implementation present across all 17 reqs (COMPARE,
Myers/Patience diff, side-by-side + inline views, navigation, merge + three-way, VFS-aware
compare, binary mode, ignore-whitespace options, stats, output panel, compare-with-saved/
clipboard/selections, unified-diff export). WIRED. Genuinely complete. No PA-INCOMPLETE.

### TCR gap (PA-TCR-033)

TCR.md has 1 row for ff-compare-merge across 17 reqs -- thin, despite 183 tasks + tests.
Recorded PA-TCR-033.

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 0; `ff-logging` Cargo dep: ABSENT (not dead).
- `std::fs`: 0.

Diff/merge is a deterministic, pure-transform engine (inputs -> diff/merge result);
errors surface via CompareError. Modest logging value (compare invoked on what, three-way
merge conflict counts, binary-vs-text decision). Recorded PA-LOG-049 (LOW): optional
dev-logging on compare invocation + merge-conflict outcomes under the `dev-logging` gate.
Low priority (pure transform, like ff-asa PA-LOG-045).

---

## 5. Task revision proposals

- **PA-STD-066 (MEDIUM -- cap + naming)**: extract the diff engine out of the mis-named
  session.rs (557, actually DiffEngine) into diff_engine.rs / patience.rs; fix the
  filename; split merge.rs (392) if it grows. REFACTOR.
- **PA-DOC-009 (naming)**: reconcile spec `ff-compare` to crate dir `ff-compare-merge`.
  Docs only.
- **PA-TCR-033**: enumerate per-requirement TCR rows (1 for 17 reqs). No code.
- **PA-STD-067 (ASCII, runtime strings)**: 3 non-comment non-ASCII -- em-dashes (U+2014)
  in error.rs strings and a MINUS SIGN (U+2212) in result.rs:136 diff-stats format
  ("+{} <minus>{} ~{}"). Replace with `--` and ASCII `-`. REFACTOR, no gate.
- **PA-LOG-049 (LOW)**: optional compare/merge dev-logging.

No PA-INCOMPLETE (complete + wired). No orphan. No raw-fs. No cross-unit duplication.

---

## Summary

compare-and-merge (`ff-compare-merge`) is a cohesive, complete, WIRED diff/merge subsystem
(COMPARE command, Myers/Patience LCS diff, side-by-side + inline views, navigation, merge +
three-way, VFS-aware compare, binary mode, ignore-whitespace, stats, unified-diff export;
183/183, 156 external refs). Another Wave-5 positive: clean-seam views (NO egui dep -- diff
views are data models the shell renders, like ff-asa), VFS-aware BY INTERFACE (URI-based
CompareSource, 0 raw fs -- contrast JES PA-CONFLICT-015), single diff/merge owner (no
duplication -- contrast ff-viewers W5.10). Findings are hygiene: PA-STD-066 (session.rs is
557 lines AND is actually the DiffEngine, not session state -- a cap violation + misleading
filename; extract diff_engine.rs; merge.rs 392 near); PA-DOC-009 (name drift ff-compare vs
ff-compare-merge); PA-TCR-033 (thin, 1/17); PA-STD-067 (em-dashes + a minus-sign U+2212 in
diff-stats/error strings); PA-LOG-049 (LOW -- optional dev-logging on a pure transform). The
crate is sound; no orphan, no incompleteness, no conflict.
