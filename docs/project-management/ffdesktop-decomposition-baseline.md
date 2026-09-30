# ff-desktop Decomposition -- Before/After Measurement (Req 19.4)

Living record of the ff-desktop crate size and incremental-rebuild cost, captured
BEFORE and AFTER each panel-extraction wave (screen-snapshot-scrm Requirement 19.4;
CR-NR-098 goal 1). Each wave is a behaviour-preserving REFACTOR: verify.ps1 (full)
must stay green with no observable behaviour change.

All measurements are on the same machine, debug profile, warm target dir.

## Method

- Line census: `C:\tools\python\python.exe tools\python\measure_crate_lines.py crates\ff-desktop\src <log>`
  (raw `.rs` line count incl. tests and blanks; sorted largest-first).
- Incremental rebuild: warm the build (`cargo build -p ff-desktop`), then touch a
  single source file's mtime and time `cargo build -p ff-desktop`. This is the cost
  a developer pays for editing one ff-desktop file (lib rebuild + relink `ffwb`).

## Baseline -- BEFORE any extraction (2026-09-26)

Captured after SCRM Waves 0-3 landed (SCRM wiring lives in ff-desktop by design).

| Metric | Value |
|--------|-------|
| ff-desktop `.rs` files | 91 |
| ff-desktop total lines (incl. tests + blanks) | 54,545 |
| Incremental rebuild after touching one lib file | 5.17 s |

Largest files (baseline):

| Lines | File |
|-------|------|
| 10,999 | shell/tests.rs |
| 2,720 | shell/render.rs |
| 2,620 | shell/commands.rs |
| 2,403 | tab_manager.rs |
| 2,163 | files_panel.rs |
| 1,730 | shell/mod.rs |
| 1,650 | editor_panel.rs |
| 1,565 | shell/update.rs |
| 1,292 | catalog_manager_dialog.rs |
| 1,150 | explorer_view.rs |
| 1,097 | menu_workspace/render.rs |
| 908 | main.rs |
| 836 | dataset_alloc_dialog.rs |
| 481 | theme_editor_panel.rs |
| 559 | toolchain_panel.rs |

Note: the dominant developer-time cost is NOT the ~5 s lib touch-rebuild but
recompiling `shell/tests.rs` (~11k lines) when tests change, and the fact that
every panel currently lives in the single binary crate so any panel edit
recompiles + relinks the whole `ffwb`. Extractions move panel code (and its unit
tests) into separately-compiled library crates, so editing a panel recompiles only
that small crate; `ff-desktop` recompiles only when the thin wiring changes.

## Candidate ranking (outbound `crate::` coupling)

Analysed 2026-09-26. Fewer internal couplings = cleaner first extraction.

| Candidate | Lines | `crate::` deps | External deps | Boundary |
|-----------|-------|----------------|---------------|----------|
| theme_editor_panel.rs | 481 | 0 | ff_theme, egui | pure `render -> ThemeEditorAction`; WorkspaceContext adapter + `apply_theme_editor_action` in shell |
| toolchain_panel.rs | 559 | 0 | ff_toolchain_api, egui, mpsc | live channel/plugin state + `show_toolchain_panel` bool |
| catalog_manager_dialog.rs | 1292 | catalog_registry | egui | dialog forms |
| explorer_view.rs | 1150 | nav_model | ff_vfs, egui | tree render |
| editor_panel.rs | 1650 | exclude_manager, tab_state | ff-* editor stack, egui | entangled with TabState |
| files_panel.rs | 2163 | catalog_manager_dialog, catalog_registry, dataset_alloc_dialog | egui | pulls 3 modules along |

FIRST TARGET: `theme_editor_panel.rs` -- zero internal `crate::` deps, already a
pure `render -> action` + `WorkspaceContext` (owned-panel swap) design, only
`ff_theme` + egui in its core. Extract the pure part (state/tokens/action/render +
unit tests) into a new `ff-theme-editor` crate; keep the `WorkspaceContext` impl
(the shell-typed adapter) and `apply_theme_editor_action` (side effects) in
ff-desktop.

## Wave log

(rows appended after each extraction wave)

| Wave | Extracted | ff-desktop lines before -> after | Incr rebuild before -> after | verify.ps1 | Date |
|------|-----------|-----------------------------------|------------------------------|-----------|------|
| -- | (baseline) | 54,545 | 5.17 s | CLEAN (9452) | 2026-09-26 |
| 1 | ff-theme-editor (Theme Editor Context) | 54,545 -> 54,120 (-425) | not remeasured (see note) | CLEAN (owner-confirmed full verify.ps1) | 2026-09-29 |
| 2 | ff-toolchain-panel (Toolchain bottom-dock panel) | 54,120 -> 53,576 (-544) | not remeasured (see note) | scoped CLEAN; full verify.ps1 = owner hand-off | 2026-09-29 |
| 3 | ff-catalog-registry (catalog registry model) | 53,576 -> ~52,788 (-788) | not remeasured (see note) | CLEAN (owner-confirmed full verify.ps1: 9452/9452) | 2026-09-30 |
| 4 | ff-catalog-dialog (New/Edit/Delete catalog dialogs) | ~52,788 -> ~51,508 (-1280) | not remeasured (see note) | scoped CLEAN (nextest 1191/1191); full verify.ps1 = owner hand-off | 2026-09-30 |

### Wave 1 -- ff-theme-editor (2026-09-29)

Extracted the pure Theme Editor (the `EditableToken` model, `ThemeEditorAction`,
`ThemeEditorState` + its methods, and the `render` free function, plus its 4 unit
tests) into a new `ff-theme-editor` crate depending only on `ff_theme` + `egui`.
Editing the Theme Editor now recompiles only that ~430-line crate, not the whole
`ff-desktop` binary.

What STAYED in `ff-desktop` (by design, per the candidate-ranking note): the
`WorkspaceContext` trait impl (the trait is local to `ff-desktop`, so the impl
must live here) and `apply_theme_editor_action` / `open_theme_editor` /
`refresh_theme_editor_list` (the shell-side side effects: file writes, palette
swap, config persist). `theme_editor_panel.rs` is now a 56-line thin adapter that
`pub use`s the crate's types (so every existing `crate::theme_editor_panel::*`
reference resolves unchanged) and holds the `WorkspaceContext` impl.

Behaviour-preserving: no observable change. `theme_editor_panel.rs` 481 -> 56
lines; `ff-desktop` 54,545 -> 54,120 lines (file count unchanged at 91 -- the
adapter file remains; the implementation moved out of the crate).

NOTE (fixup): the crate had previously been added to the workspace `members` list
with a `Cargo.toml` but NO `src/`, which made the whole workspace fail to load
(`no targets specified in the manifest`). Wave 1 also fixes that break by
supplying `src/lib.rs`.

NOTE (metrics): the incremental-rebuild figure was not remeasured this session
(it needs a warm-target touch-rebuild cycle on the owner's machine; the line
delta is the primary Req 19.4 metric). The full `verify.ps1` gate is the owner's
manual step; it was run by the owner and reported CLEAN (empty ai-review.log) on
2026-09-29, so Wave 1 is DONE (owner-confirmed), not merely scoped-clean.

Scoped gate (Kiro-run, cargo nextest -- the project's real runner):
- `cargo nextest run -p ff-desktop` -> 1225 tests run, 1225 passed, 0 skipped.
- `cargo clippy -p ff-desktop --all-targets -- -D warnings` -> exit 0.
- `cargo fmt -p ff-theme-editor -p ff-desktop -- --check` -> exit 0.
- `cargo test -p ff-theme-editor` -> 4 passed (crate in isolation).

Note on `cargo test` vs `nextest`: under plain thread-parallel `cargo test
-p ff-desktop`, 5 unrelated config/history/env-var-backed tests fail
(`close_workspace_removes_settings_from_config`,
`exit_saves_command_history_and_reloads`,
`startup_loads_persisted_command_history`,
`startup_missing_or_corrupt_history_is_empty_no_panic`,
`theme_follow_os_can_be_set_to_true`) due to the known B048 env-var/config
isolation issue; all 5 PASS under process-isolated nextest. Not caused by this
extraction (a pure code move that touches no config/history/session code).

### Wave 2 -- ff-toolchain-panel (2026-09-29)

Extracted the pure Toolchain Panel (the `ToolchainEntry` + `ToolchainPanelState`
state model, their methods, the `render` free function, and the 10 unit tests)
into a new `ff-toolchain-panel` crate depending only on `ff-toolchain-api`,
`ff-gcc-toolchain`, `ff-rust-toolchain`, and `egui`. Editing the Toolchain Panel
now recompiles only that ~330-line crate, not the whole `ff-desktop` binary.

Unlike Wave 1, this panel has NO `WorkspaceContext` impl to keep behind: the
Toolchain Panel is a bottom dock, so ALL of it moved out. `toolchain_panel.rs`
becomes a 15-line thin adapter that `pub use`s `ff_toolchain_panel::{render,
ToolchainPanelState}` so every existing `crate::toolchain_panel::*` reference in
the shell (`shell/mod.rs` owns the `ToolchainPanelState` field + the
`show_toolchain_panel` flag; `shell/render.rs` calls `render` in the bottom
dock) resolves unchanged. The shell still owns the state field, the visibility
flag, and the editor-navigation side effect driven by `render`'s return value.

Behaviour-preserving: no observable change. `toolchain_panel.rs` 559 -> 15 lines.
En route this also cleaned up the pre-extraction file's non-ASCII box-drawing
section separators and em-dashes; the extracted crate uses ASCII `// === ... ===`
separators per documentation.md.

NOTE (metrics): the incremental-rebuild figure was not remeasured this session
(same reasoning as Wave 1 -- the line delta is the primary Req 19.4 metric). The
full `verify.ps1` gate is the owner's manual step; Kiro ran the scoped gate CLEAN
and handed off.

Scoped gate (Kiro-run, cargo nextest):
- `cargo fmt --check` -> exit 0.
- `cargo clippy -p ff-toolchain-panel -p ff-desktop --all-targets -- -D warnings`
  -> exit 0.
- `cargo nextest run -p ff-toolchain-panel -p ff-desktop` -> 1225 tests run,
  1225 passed, 0 skipped (the 10 Toolchain Panel tests moved with the code from
  ff-desktop into ff-toolchain-panel; the combined scoped total is unchanged).

### Wave 3 -- ff-catalog-registry (2026-09-30)

Extracted the pure catalog-registry model (the `CatalogType`, `VirtualCatalog`,
`CatalogRegistry`, and `RegistryError` types, their methods, the `dataset_node`
free function, plus all 22 unit tests) into a new `ff-catalog-registry` crate
depending only on `ff-dscatalog`, `ff-vfs`, `ff-file-tree`, `serde`, and `toml`
(egui-free, zero `crate::` coupling). Editing the catalog registry now recompiles
only that ~800-line crate, not the whole `ff-desktop` binary.

Like Wave 2 and unlike Wave 1, this module has NO `WorkspaceContext` impl to keep
behind: the registry is a pure model consumed by session_manager, shell/mod,
shell/commands, shell/render, shell/reset_bare, files_panel, and
catalog_manager_dialog via `crate::catalog_registry::*`. `catalog_registry.rs`
becomes a 13-line thin adapter that `pub use`s `ff_catalog_registry::*`, so every
existing reference resolves unchanged.

Behaviour-preserving: no observable change. `catalog_registry.rs` 801 -> 13 lines;
`ff-desktop` ~53,576 -> ~52,788 lines (file count unchanged -- the adapter file
remains; the implementation moved out into the crate). The crate scaffold
(Cargo.toml + workspace `members` entry + the lib.rs model body) pre-existed from
an earlier session; this wave supplied the missing `#[cfg(test)] mod tests` module
and finished the adapter + ff-desktop path dependency.

NOTE (metrics): the incremental-rebuild figure was not remeasured this session
(same reasoning as Waves 1-2 -- the line delta is the primary Req 19.4 metric).
The exact ff-desktop total (~52,788) is an estimate from the file-level delta and
should be confirmed by the owner's measurement at gate time.

Verification done this session (early, before the terminal broke):
- `cargo test -p ff-catalog-registry` -> 22 passed (crate in isolation).
- `cargo test -p ff-desktop` -> compiled cleanly (19.44 s); 1186 passed, 5 failed.
  The 5 failures (close_workspace_removes_settings_from_config,
  exit_saves_command_history_and_reloads,
  full_shell_theme_unknown_leaves_theme_unchanged,
  startup_missing_or_corrupt_history_is_empty_no_panic,
  theme_follow_os_can_be_set_to_true) contain NO catalog_registry reference and
  are the known B048 env-var config-isolation class that passes under
  process-isolated nextest -- identical to the Wave 1-2 note. All
  catalog-touching tests (catalog_manager_dialog, files_panel,
  dataset_alloc_dialog) passed, confirming the adapter re-export resolves.

Scoped gate (Kiro-run, cargo nextest -- run 2026-09-30 after a terminal recovery;
logs in tools\logs\decomp3-fmt.txt / decomp3-clippy.txt / decomp3-nextest.txt):
- `cargo fmt --check` -> exit 0.
- `cargo clippy -p ff-catalog-registry -p ff-desktop -- -D warnings` -> exit 0
  (both crates checked, 0 warnings).
- `cargo nextest run -p ff-catalog-registry -p ff-desktop` -> 1213 tests run,
  1213 passed, 0 skipped (the new crate's 22 tests plus the ff-desktop suite; the
  5 B048 config/history/theme tests that fail only under thread-parallel
  `cargo test` pass here under process isolation).

(Earlier in the session the interactive terminal was non-functional env-wide --
every command, including a trivial `echo`, returned Exit Code -1 with no output --
so the scoped gate was deferred; it ran clean once the terminal recovered.)

The OWNER ran the full `verify.ps1` gate on 2026-09-30 and reported CLEAN (FULL
nextest: 9452 run, 9452 passed, 0 failed; empty ai-review.log). Wave 3 is DONE
(owner-confirmed), not merely scoped-clean.

### Wave 4 -- ff-catalog-dialog (2026-09-30)

Extracted the New / Edit / Delete virtual-catalog modal dialogs from
`ff-desktop/src/catalog_manager_dialog.rs` (1292 lines) into a new
`ff-catalog-dialog` crate. The dialogs' only `crate::` dependency was
`catalog_registry`, extracted in Wave 3, so the move is clean: the crate depends
on `ff-catalog-registry`, `ff-dscatalog` (repository initialisation on create),
and `egui`. Editing the catalog dialogs now recompiles only this crate, not the
whole `ffwb` binary.

Sliced by sub-dialog into one module each (also satisfying the 400-line source
limit):
- `new_dialog.rs`: `NewCatalogForm`, `DialogOutcome`, `validate`, `build_catalog`,
  `render` + the three type-specific field renderers, and 25 tests.
- `edit_dialog.rs`: `EditCatalogForm`, `validate_edit`, `render_edit`, and 9 tests.
- `delete_dialog.rs`: `DeleteChoice`, `DeleteCatalogConfirm`, `render_delete`,
  `execute_delete` (with Home-catalog protection), and 8 tests.
- `lib.rs`: the shared `pub(crate) catalog_type_label` helper (used by New + Edit)
  and the public re-exports that reproduce the old flat module surface.

`ff-desktop/src/catalog_manager_dialog.rs` is now a 12-line thin adapter that
`pub use`s `ff_catalog_dialog::*`, so every existing
`crate::catalog_manager_dialog::*` reference (files_panel, shell/render,
shell/update) resolves unchanged. `ff-catalog-dialog` added as a path dep in
`ff-desktop/Cargo.toml`.

Behaviour-preserving: no observable change. `catalog_manager_dialog.rs`
1292 -> 12 lines; `ff-desktop` ~52,788 -> ~51,508 lines (estimate from the
file-level delta; owner to confirm exact total at gate time).

Scoped gate (Kiro-run, cargo nextest -- logs in tools\logs\decomp4-*.txt):
- `cargo fmt -p ff-catalog-dialog -p ff-desktop -- --check` -> exit 0.
- `cargo clippy -p ff-catalog-dialog -p ff-desktop -- -D warnings` -> exit 0
  (both crates checked, 0 warnings).
- `cargo nextest run -p ff-catalog-dialog -p ff-desktop` -> 1191 tests run,
  1191 passed, 0 skipped (the crate's 44 tests plus the ff-desktop suite; the
  catalog_manager_dialog consumer tests pass unchanged against the re-export).

NOTE (unrelated pre-existing gate blockers, FIXED this session): a workspace-wide
`cargo fmt --check` had been failing on formatting drift in the prior-batch
markdown-viewer crates (`ff-md-viewer`, `ff-mdx-app`, `ff-mdx-installer`,
`ff-mdx-plugin`) and on an unresolved feature-gated module in `ff-pdf-export`
(`#[cfg(feature = "protected")] pub mod protected;` with no `protected.rs`).
Both were resolved as an owner-approved cleanup alongside Wave 4: a GENERIC
`ff-pdf-export/src/protected.rs` was written (owner-password encryption operating
on plain PDF bytes/pages, decoupled from SCRM types; lopdf aligned to 0.36; 7
tests pass, clippy-clean under the feature), and the viewer-crate drift was
absorbed with `cargo fmt`. Workspace `cargo fmt --check` is now exit 0.

The owner's full `verify.ps1` gate remains the hand-off that marks Wave 4 DONE
(owner-confirmed).
