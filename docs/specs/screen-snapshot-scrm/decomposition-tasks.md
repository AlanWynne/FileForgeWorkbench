# ff-desktop Decomposition -- Sliced Task Breakdown (Req 19.3/19.4, Wave 4+)

Companion to `tasks.md`. The SCRM build (Waves 0-3) is DONE. This file tracks the
behaviour-preserving PANEL/MODULE-EXTRACTION waves (design.md Section 8, "Waves
4+"). Each wave is a pure REFACTOR (no requirements gate) that:

- moves pure code (and its unit tests) out of the `ff-desktop` binary crate into a
  separately-compiled library crate;
- leaves a THIN adapter file in `ff-desktop` that `pub use`s the new crate's
  types, so every existing `crate::<module>::*` reference resolves UNCHANGED;
- adds the new crate as a path dep in `crates/ff-desktop/Cargo.toml`;
- keeps behaviour identical (no observable change);
- converts any non-ASCII comment characters to ASCII per `documentation.md`;
- is scoped-gated by Kiro (`cargo fmt --check`, `cargo clippy -p <crates> -- -D
  warnings`, `cargo nextest run -p <crates>`) and then handed off to the OWNER for
  the full `verify.ps1` gate;
- appends a before/after row to
  `docs/project-management/ffdesktop-decomposition-baseline.md` and updates the
  TCR Req 19.3/19.4 row.

Waves 1-2 are DONE (ff-theme-editor, ff-toolchain-panel). The remaining candidates
are sliced below into SMALL, independently-verifiable pieces, ordered so each
extraction's dependencies are already extracted (leaf modules first, panels last).

## Slicing principle

Extract the dependency SUBSTRATE before its dependents. A large panel
(catalog_manager_dialog, explorer_view, editor_panel, files_panel) couples to
smaller model/leaf modules (catalog_registry, nav_model, context_menu,
scroll_amount, exclude_manager). Extracting a leaf into its own crate first turns
the panel's later `crate::<leaf>` coupling into a clean `ff_<leaf>` crate
dependency, so the panel extraction is then a self-contained move. Each numbered
sub-task is one crate created + one thin adapter + one scoped gate -- small enough
to verify and hand off on its own.

## Wave 3 -- ff-catalog-registry (model substrate, egui-free)

`catalog_registry.rs` (801 lines): ZERO `crate::` coupling; external deps
`ff_dscatalog`, `ff_vfs`, `ff_file_tree`, `serde`. Consumed by session_manager,
shell/mod, shell/commands, shell/render, shell/reset_bare, files_panel,
catalog_manager_dialog -- all via `crate::catalog_registry::*`, so a thin
`pub use` adapter keeps them unchanged. Lowest-risk next slice; unblocks Wave 4.

- [ ] 11. Create the `ff-catalog-registry` crate (pure model, no egui)
  - [x] 11.1 Add `crates/ff-catalog-registry/` with `Cargo.toml` (deps:
        ff-dscatalog, ff-vfs, ff-file-tree, serde) and register it in the
        workspace `members` list. (Req 19.1)
  - [x] 11.2 Move the `catalog_registry.rs` body (types `CatalogRegistry`,
        `CatalogType`, `VirtualCatalog`, the `dataset_node` helper, and its unit
        tests) into `src/lib.rs`; convert any non-ASCII comment chars to ASCII.
        (Req 19.1, 19.3)
    - DONE: lib.rs holds the full model body + all 22 unit tests (ASCII-only
      comments). `cargo test -p ff-catalog-registry` = 22/22 pass.
  - [x] 11.3 Replace `ff-desktop/src/catalog_registry.rs` with a thin adapter:
        `pub use ff_catalog_registry::*;` so `crate::catalog_registry::*` resolves
        unchanged. Add `ff-catalog-registry` path dep in ff-desktop Cargo.toml.
        (Req 19.2)
    - DONE: adapter is `pub use ff_catalog_registry::*;`; ff-desktop compiles and
      its catalog_manager_dialog / files_panel / dataset_alloc_dialog tests pass
      unchanged against the re-export.
  - [x] 11.4 Scoped gate: `cargo nextest run -p ff-catalog-registry -p ff-desktop`,
        `cargo clippy -p ff-catalog-registry -p ff-desktop -- -D warnings`,
        `cargo fmt --check`. Record before/after lines; update baseline wave log +
        TCR Req 19.3/19.4. Hand off full verify.ps1 to owner. (Req 19.3, 19.4)
    - DONE (scoped CLEAN 2026-09-30): `cargo fmt --check` exit 0;
      `clippy -p ff-catalog-registry -p ff-desktop -- -D warnings` exit 0 (0 warnings);
      `nextest run -p ff-catalog-registry -p ff-desktop` 1213/1213 passed, 0 skipped
      (the 5 B048 config/history/theme tests pass under process-isolated nextest).
      catalog_registry.rs 801 -> 13; ff-desktop ~53,576 -> ~52,788. Owner full
      verify.ps1 still the hand-off to mark Wave 3 DONE (owner-confirmed).

## Wave 4 -- ff-catalog-dialog (depends on Wave 3)

`catalog_manager_dialog.rs` (1292 lines; ~667 non-test): only `crate::` dep is
catalog_registry (extracted in Wave 3); external `eframe::egui`. Three logical
sub-dialogs. Sliced by sub-dialog so each move is small and independently green.

- [ ] 12. Create the `ff-catalog-dialog` crate scaffold (empty lib + Cargo.toml)
  - [ ] 12.1 Add `crates/ff-catalog-dialog/` (deps: ff-catalog-registry, egui) and
        register in workspace `members`. Empty `src/lib.rs`. (Req 19.1)
- [ ] 13. Move the New-Catalog sub-dialog into the crate
  - [ ] 13.1 Move `NewCatalogForm` + `DialogOutcome` + `validate` + `build_catalog`
        + `render` + `render_mainframe/posix/native_fields` + `catalog_type_label`
        and their unit tests; ASCII comments. (Req 19.1, 19.3)
- [ ] 14. Move the Edit-Catalog sub-dialog into the crate
  - [ ] 14.1 Move `EditCatalogForm` + `validate_edit` + `render_edit` and tests.
        (Req 19.1, 19.3)
- [ ] 15. Move the Delete-Catalog sub-dialog into the crate
  - [ ] 15.1 Move `DeleteChoice` + `DeleteCatalogConfirm` + `render_delete` +
        `execute_delete` and tests. (Req 19.1, 19.3)
- [ ] 16. Thin-adapter + gate for the catalog dialog
  - [ ] 16.1 Replace `ff-desktop/src/catalog_manager_dialog.rs` with
        `pub use ff_catalog_dialog::*;`; add path dep. Verify files_panel.rs,
        shell/render.rs, shell/update.rs references resolve unchanged. (Req 19.2)
  - [ ] 16.2 Scoped gate (`-p ff-catalog-dialog -p ff-desktop`); baseline row +
        TCR update; hand off full verify.ps1. (Req 19.3, 19.4)

## Wave 5 -- explorer_view substrate + panel

`explorer_view.rs` (1150 lines) couples to `nav_model` (833 lines) and
`context_menu` (317 lines). Extract both leaves first, then the view.

- [ ] 17. Extract `context_menu` -> `ff-context-menu` crate (leaf: FileClass +
        classify_file). Thin adapter + scoped gate. (Req 19.1-19.4)
- [ ] 18. Extract `nav_model` -> `ff-nav-model` crate (NavModel on ff-file-tree).
        Confirm its own `crate::` coupling first; extract any leaf it needs.
        Thin adapter + scoped gate. (Req 19.1-19.4)
- [ ] 19. Extract `explorer_view` -> `ff-explorer-view` crate (depends on
        ff-nav-model + ff-context-menu + ff-vfs + egui). Thin adapter + scoped
        gate. (Req 19.1-19.4)

## Wave 6 -- editor_panel substrate + panel (hardest; TabState stays)

`editor_panel.rs` (1650 lines) couples to `scroll_amount` (184, leaf),
`exclude_manager` (300, leaf), and `tab_state` (401). `tab_state` is
shell-entangled (TabId/TabState/UndoEntry are the shell's runtime tab model) and
STAYS in ff-desktop; the editor crate takes it as a generic/param or the render
free-function keeps its `&mut TabState` signature via a re-exported type.

- [ ] 20. Extract `scroll_amount` -> `ff-scroll-amount` crate (leaf). Thin adapter
        + scoped gate. (Req 19.1-19.4)
- [ ] 21. Extract `exclude_manager` -> `ff-exclude-manager` crate (leaf). Thin
        adapter + scoped gate. (Req 19.1-19.4)
- [ ] 22. Extract the pure editor render/scroll helpers from `editor_panel` into
        `ff-editor-panel`, keeping `tab_state`-entangled glue in ff-desktop as the
        adapter. Slice by helper group if the move exceeds one reviewable step.
        Thin adapter + scoped gate. (Req 19.1-19.4)

## Wave 7 -- files_panel (last; depends on Waves 3-4)

`files_panel.rs` (2163 lines) pulls `catalog_manager_dialog` (Wave 4),
`catalog_registry` (Wave 3), and `dataset_alloc_dialog` (836). Extract
dataset_alloc_dialog first, then the panel.

- [ ] 23. Extract `dataset_alloc_dialog` -> `ff-dataset-alloc-dialog` crate
        (depends on ff-catalog-registry + egui). Thin adapter + scoped gate.
        (Req 19.1-19.4)
- [ ] 24. Extract the pure Files Panel body -> `ff-files-panel`, keeping the
        shell-entangled parts (dialog state machine, resolve_and_open_dataset
        shell wiring) as the adapter. Slice by concern (tree render / dialog
        dispatch / dataset resolution) if the move exceeds one reviewable step.
        Thin adapter + scoped gate. (Req 19.1-19.4)

## Notes

- Each `[ ]` sub-task is ONE crate + ONE thin adapter + ONE scoped gate = one
  reviewable, independently-verifiable piece (the "smaller pieces" slicing).
- These are REFACTORS: no `requirements.md` change, no new criteria. They are
  covered by the EXISTING Req 19.3 (behaviour-preserving) / 19.4 (measured
  before/after) criteria in `requirements.md`.
- Before extracting each leaf, re-run the `crate::` coupling check on it (as done
  for catalog_registry, explorer_view, editor_panel) in case a later change
  introduced a new coupling; extract any newly-discovered leaf dependency first.
- Line counts are the 2026-09-26 baseline; re-measure at extraction time.
