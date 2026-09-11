# Analysis Record: external-modification (W4.14)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-external-mod` (external file-modification detection --
  detects open files modified/renamed/deleted by external tools and offers the user
  handling options)
- **Spec files**: requirements.md (216 lines, 10 requirements), tasks.md
  (157 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 10 reqs, 216 lines. Cohesive concern (external-change
detection: watch registry, change events, batch coalescing, focus-gained re-check,
config). 11 files, no file over 400. No split.

---

## 2. Cross-unit consistency

### PA-CONFLICT-014 (FOURTH orphan crate + unresolved placement) -- MEDIUM

`ff-external-mod` provides a full detection model: `ExternalModificationDetector`,
`WatchRegistry`, `ChangeType`, `ExternalChange`, `BatchCoalescer`,
`FocusGainedChecker`, `DocumentState`, `ExternalModConfig`. But:

- It is referenced by NO crate (grep all Cargo.toml + all src: only its own; 0
  `ff_external_mod` refs elsewhere). ORPHAN -- the FOURTH (w/ ff-file-tree
  PA-CONFLICT-011, ff-idle-processing PA-CONFLICT-012, ff-large-file-performance
  PA-CONFLICT-013).
- The spec's OWN intro leaves placement UNRESOLVED: "the `ff-external-modification`
  module (part of the `ff-file-operations` crate OR a standalone crate depending on
  final architecture)". So the orphan status is partly a never-finalized
  architectural decision -- is external-mod its own crate (as built) or folded into
  file-operations (W4.15)? The detector is not wired into the shell either way.

Recorded PA-CONFLICT-014 (owner-gated, MEDIUM): (a) resolve the placement decision
the spec deferred (keep `ff-external-mod` standalone -- recommended, it is cleanly
decomposed -- OR fold into ff-file-operations), and (b) WIRE the detector into the
shell/document lifecycle (register open documents with the WatchRegistry; surface
ChangeType via the notification system on external change / focus-gained re-check).
Cross-reference W4.15 file-operations for the placement half.

### Naming drift (PA-DOC-006) -- spec name vs crate dir

Spec + module docs call it `ff-external-modification`; the actual crate dir is
`ff-external-mod`. Same crate-name-drift pattern as ff-hex/ff-select/ff-tabmask/
ff-forge/ff-keys/ff-completion/ff-wrap/ff-seqnum/ff-dscatalog. Recorded PA-DOC-006
(add to the naming-reconciliation set): reconcile spec text to `ff-external-mod` (or
rename the crate) so the docs match the built artifact.

### FFW-ARCH-001 compliance -- CLEAN (VFS-mediated, 0 real fs)

lib.rs documents FFW-ARCH-001 ("all filesystem interaction flows through ff-vfs -- no
std::fs / tokio::fs for watching or stat"). Verified: the 2 grep hits for
std::fs/tokio::fs are BOTH inside that doc-comment (the prohibition text itself);
actual fs usage is 0 -- watching + stat go through ff-vfs. Exemplary VFS discipline
(counter-example to any raw-fs finding). Consistent with connector/VFS layering.

### Notification integration

External-change handling (Req: present the user options) is a natural notification-
system (W4.7) + dialog consumer. The detector produces `ExternalChange`/`ChangeType`;
the SHELL should route these to notifications/dialogs. Currently unwired (part of
PA-CONFLICT-014). Consistent design (detector produces events; shell presents).

### Public types and ownership

- `ExternalModificationDetector`, `WatchRegistry`, `ChangeType`, `ExternalChange`,
  `BatchCoalescer`, `FocusGainedChecker`, `DocumentState`, `ExternalModConfig` --
  sole-owned by `ff-external-mod` but ORPHANED (PA-CONFLICT-014). No duplication.

### Cross-reference integrity

Cross-refs (ff-vfs, document-model, config, command-framework) resolve as
sub-projects and are actually used (VFS for watching, document-model for
DocumentState). file-operations placement cross-ref open (PA-CONFLICT-014).

---

## 3. Completeness

Tracking: all 157 sub-tasks `[x]`. Crate complete + tested (detector, watch registry,
change events, batch coalescing, focus-gained re-check, config). FUNCTIONALLY complete
AS A CRATE -- but not wired to the shell/document lifecycle (PA-CONFLICT-014), so
external-change detection is not delivered to the user in the running app. No
PA-INCOMPLETE for the crate itself; the gap is the missing shell wiring +
unresolved placement (architectural).

### TCR gap (PA-TCR-026) -- TOTAL ABSENCE

TCR.md has ZERO rows for ff-external-mod (grep = 0) across 10 reqs / ~80 criteria,
despite the crate's tests. Recorded PA-TCR-026. (Third orphan infra crate in a row
with total TCR absence -- idle PA-TCR-024, large-file PA-TCR-025, now this.)

---

## 4. Logging audit

Scan of `crates/ff-external-mod/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0 (the 2 hits are the FFW-ARCH-001 prohibition doc-comment)

A file-watcher is exactly where dev-logging pays off: watch register/unregister,
raw fs event -> ChangeType classification, batch-coalesce decisions, and
focus-gained re-check outcomes are all hard to debug blind (races, missed/duplicate
events). Recorded PA-LOG-039 (MEDIUM -- higher than the usual LOW): resolve the dead
ff-logging dep AND add dev-logging on watch lifecycle + event classification +
coalescing under the `dev-logging` gate. This is a strong CR-NR-058 candidate.

---

## 5. Task revision proposals

- **PA-CONFLICT-014 (owner-gated, MEDIUM)**: FOURTH orphan + unresolved placement.
  (a) resolve the spec's deferred decision (standalone `ff-external-mod` -- recommended
  -- vs fold into ff-file-operations); (b) wire the detector into the shell/document
  lifecycle + notification system. Cross-ref W4.15. Code + owner decision.
- **PA-DOC-006 (naming)**: reconcile spec `ff-external-modification` to crate dir
  `ff-external-mod` (add to the naming-reconciliation set). Docs only.
- **PA-STD-054 (ASCII, runtime strings)**: 90 non-ASCII bytes (6 non-comment) --
  em-dashes in `#[error]` strings (error.rs:23,33,43,52,77,95). Replace with `--`.
  REFACTOR, no gate.
- **PA-LOG-039 (MEDIUM)**: resolve dead ff-logging dep + add watch-lifecycle /
  event-classification / coalescing dev-logging (strong CR-NR-058 candidate).
- **PA-TCR-026**: add TCR rows (0 for 10 reqs). No code.

No PA-STD size item (no file over 400). No requirement CHANGE proposed; the crate is
correct -- the issues are placement (spec-deferred) + missing shell wiring.

---

## Summary

external-modification (`ff-external-mod`) is a clean, complete, well-decomposed
external-change detector (watch registry, change events, batch coalescing,
focus-gained re-check; 157/157, no file over 400) with EXEMPLARY VFS discipline --
0 real fs, all watching/stat through ff-vfs per the documented FFW-ARCH-001 (the 2
std::fs grep hits are the prohibition doc-comment itself). The headline finding is
PA-CONFLICT-014 (MEDIUM): it is the FOURTH orphan crate -- referenced by no crate --
AND its placement was never resolved (the spec intro literally says "part of
ff-file-operations OR a standalone crate depending on final architecture"). Resolve
the placement (keep standalone -- recommended) and wire the detector into the shell/
document lifecycle + notification system so external-change handling reaches the user.
Naming drift PA-DOC-006 (ff-external-modification vs ff-external-mod). Logging is the
notable functional gap: a file-watcher benefits strongly from dev-logging (watch
lifecycle, event classification, coalescing) yet ff-logging is a DEAD dep -- recorded
PA-LOG-039 at MEDIUM (a strong CR-NR-058 candidate). Minor: total TCR absence
(PA-TCR-026), runtime-string ASCII (PA-STD-054). The crate itself is sound.
