# RESUME: DECOMP Wave 7 (files_panel -- FINAL decomposition wave)

**Written:** 2026-10-01
**Author:** Kiro
**Branch to work on:** `decomp-and-scrm-wave` (at `3d75dff`, pushed to origin).
This contains DECOMP Waves 1-6 + egui-0.33 fix + CR-NR-099 log.

---

## STATE

- Waves 1-6 DONE, owner-verified (full verify.ps1 clean), pushed to origin at
  `3d75dff`. decomp-wave6 worktree + branch cleaned up.
- Wave 7 is the LAST wave in docs/specs/screen-snapshot-scrm/decomposition-tasks.md
  (tasks 23-24).

## VERIFIED COUPLING FACTS (checked at 3d75dff)

- `crates/ff-desktop/src/dataset_alloc_dialog.rs` -- ZERO `crate::` references.
  TRUE CLEAN LEAF. Task 23 = verbatim move + re-export shim (like Task 20).
  Depends only on external crates + egui (confirm exact deps at implementation).
- `crates/ff-desktop/src/files_panel.rs` -- 2163 lines. Couples to:
  - `crate::catalog_manager_dialog::{DeleteCatalogConfirm, EditCatalogForm,
    NewCatalogForm}` -- ALREADY a shim over `ff-catalog-dialog` (Wave 4).
  - `crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog}` --
    ALREADY a shim over `ff-catalog-registry` (Wave 3).
  - `crate::dataset_alloc_dialog::{AllocDatasetForm, AllocParams, Dsorg, Recfm}`
    -- becomes `ff-dataset-alloc-dialog` after Task 23.
  - defines its own `ContentEntry` type (`crate::files_panel::ContentEntry`).
  - IMPORTANT: `catalog_registry` and `catalog_manager_dialog` are ff-desktop
    `mod`s in main.rs BUT their bodies are thin re-export shims over the Wave 3/4
    crates -- so the Wave 7 crate can depend on `ff-catalog-registry` +
    `ff-catalog-dialog` + `ff-dataset-alloc-dialog` + egui. NO unplanned
    prerequisite crate extraction is needed.
  - Still TO CONFIRM at plan time: how shell-entangled the files_panel body is
    (the spec mentions a dialog state machine + `resolve_and_open_dataset` shell
    wiring). Likely needs the Task-21/22 clean-seam/adapter treatment (keep the
    shell-entangled glue in ff-desktop as the adapter; move the pure
    tree-render / dialog-form logic into the crate). TabState/shell types STAY in
    ff-desktop.

## HOUSEKEEPING

- Terminal was intermittently DEGRADED (mangled echo, Exit Code -1). The OWNER
  has a reliable terminal and runs the authoritative scoped gate + commit. Kiro
  makes code edits via file tools and hands off the gate/commit to the owner.
- Scoped gate per task (NEVER --workspace / verify.ps1 -- owner's manual step):
  - `cargo fmt -p <new-crate> -p ff-desktop -- --check`
  - `cargo clippy -p <new-crate> -p ff-desktop --tests -- -D warnings`
  - `cargo nextest run -p <new-crate> -p ff-desktop` (nextest, NOT plain test --
    B048 env-var process isolation).
- rust-standards: no .rs file >400 non-test lines (split the adapter if needed,
  as editor_panel became editor_panel/{mod,input,paint}.rs); ASCII-only .rs.
- Behaviour-preserving refactor -- NO gate, weaken no test, keep every
  `// Validates:` annotation.

## WAVE 7 TASKS (decomposition-tasks.md)

- [ ] 23. `dataset_alloc_dialog` -> `ff-dataset-alloc-dialog` (clean leaf;
      verbatim move + shim).
- [ ] 24. `files_panel` pure body -> `ff-files-panel`; keep shell-entangled parts
      (dialog state machine, dataset resolution) as the ff-desktop adapter. Slice
      by concern (tree render / dialog dispatch / dataset resolution) if the move
      exceeds one reviewable step.

## FINALIZE / PUSH (owner-run, learned from Wave 6)

- Finalize = fast-forward `decomp-and-scrm-wave` to the wave branch. The
  `.agents/tasks/decomp-wave7/*` mirror files are untracked in the main checkout
  AND committed on the wave branch -> `git merge --ff-only` ABORTS on "untracked
  working tree files would be overwritten". Fix: `del` the untracked mirror
  copies in the main checkout, then re-run `git merge --ff-only <wave-branch>`.
- Any uncommitted `docs/status/change-log.md` CR entry must be committed on
  `decomp-and-scrm-wave` before pushing so the branch is self-consistent.
- Owner runs the full `verify.ps1` before declaring Wave 7 DONE; then push.
