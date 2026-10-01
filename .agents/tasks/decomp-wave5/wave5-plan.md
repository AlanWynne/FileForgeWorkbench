# DECOMP Wave 5 -- explorer_view substrate + panel (Implementation Plan)

Behaviour-preserving REFACTOR (no observable behaviour change). No requirements
gate: covered by existing Req 19.3 (behaviour-preserving) / 19.4 (measured
before/after) in `docs/specs/screen-snapshot-scrm/requirements.md`.

- Code worktree: `c:\workspace\VSC\FileForgeWorkbench\.worktrees\decomp-wave5`
  (all source edits happen HERE). Git ops: `git -C c:\workspace\VSC\FileForgeWorkbench\.worktrees\decomp-wave5 ...`.
- Main repo root: `c:\workspace\VSC\FileForgeWorkbench` (this plan lives here).
- Spec: `docs/specs/screen-snapshot-scrm/decomposition-tasks.md`, section
  "Wave 5", tasks 17-19 (ordered leaves-first: context_menu, nav_model,
  explorer_view).

## Key decisions resolved during exploration

### DECISION 1 (PosixProvider) -- extract `posix_provider` into its OWN leaf crate FIRST

The task framed a choice: have `ff-nav-model` tests use `ff_vfs::posix_provider::PosixProvider`
directly, OR share/extract ff-desktop's `PosixProvider` if it differs materially.

Finding (verified in the worktree):
- `ff-vfs` does NOT export a `PosixProvider`. It exports `PosixNativeProvider`
  (`crates/ff-vfs/src/posix_provider.rs`) with a DIFFERENT, infallible
  constructor `PosixNativeProvider::new(impl Into<PathBuf>, bool) -> Self`.
- `ff-desktop`'s `PosixProvider` (`crates/ff-desktop/src/posix_provider.rs`) is a
  DIFFERENT type: fallible `PosixProvider::new(PathBuf, bool) -> Result<Self, VfsError>`,
  wrapping `ff_connector_local_fs::LocalFsProvider` with POSIX normalisation +
  root-jail. The `nav_model` tests call `PosixProvider::new(path, false).expect(...)`
  (the FALLIBLE form) and depend on its root-jail/normalisation behaviour.

Therefore using `ff_vfs::...::PosixNativeProvider` would CHANGE what the tests
assert (different type, different constructor shape, different behaviour) -- NOT
allowed (tests must assert the SAME behaviour; never weaken/delete a nav_model
test). A leaf crate also cannot dev-depend on `ff-desktop` (the crate we extract
FROM) without a dependency cycle.

Lowest-risk option chosen: **extract `ff-desktop/src/posix_provider.rs` into a new
leaf crate `ff-posix-provider` as Wave 5 Task 0 (BEFORE nav_model)**, leave a thin
`pub use ff_posix_provider::*;` adapter in ff-desktop (so the existing
`crate::posix_provider::PosixProvider` refs in `shell/render.rs` AND the
`nav_model` tests keep resolving unchanged), and give `ff-nav-model` a
dev-dependency on `ff-posix-provider`, with its tests rewritten from
`crate::posix_provider::PosixProvider` to `ff_posix_provider::PosixProvider`
(identical type, identical behaviour). `posix_provider.rs` has ZERO `crate::`
coupling (external deps only: `ff_connector_local_fs`, `ff_vfs`, `async_trait`),
so it is a clean leaf extraction following the established Wave 1-4 pattern.

### DECISION 2 (members list style) -- EXPLICIT list, insert each new crate

The workspace `members` in `.worktrees/decomp-wave5/Cargo.toml` is an EXPLICIT
array of path strings (NOT a glob). Each new crate MUST be added by hand. Insert
the three (then four) new leaf crates in the block just above `"crates/ff-desktop"`,
beside the other extracted leaves (`ff-catalog-registry`, `ff-catalog-dialog`,
`ff-dataset-alloc-dialog`). Cargo does not require dependency ordering in
`members`, but placing leaves before `ff-desktop` matches the existing convention.

## Established extraction pattern (Waves 1-4), applied per module X -> ff-X

1. Create `crates/ff-X/Cargo.toml` -- `[package]` name `ff-X`, with
   `version.workspace = true`, `edition.workspace = true`,
   `authors.workspace = true`, `license.workspace = true`, a `description`, and
   the module's real deps (matching sibling manifests such as
   `ff-catalog-registry` / `ff-catalog-dialog`).
2. Create `crates/ff-X/src/lib.rs` holding the MOVED module body INCLUDING its
   `#[cfg(test)] mod tests`. Convert any non-ASCII comment characters to ASCII
   (`.rs` is strict ASCII per `documentation.md` / `rust-standards.md`; box-drawing
   is NOT allowed in `.rs`).
3. Replace `crates/ff-desktop/src/X.rs` with a THIN adapter: `pub use ff_x::*;`
   (plus any extra re-exports needed so `crate::X::<Item>` paths elsewhere still
   resolve). Keep the `mod X;` declaration in `main.rs` so `crate::X::...` resolves
   via the re-export.
4. Add `ff-X = { path = "../ff-X" }` to `crates/ff-desktop/Cargo.toml` `[dependencies]`.
5. Add `"crates/ff-X"` to the root `Cargo.toml` `members` list (explicit list).

## Pre-flight (once, before Task 0)

- [ ] 0a. Confirm a clean baseline for the touched crates in the worktree.
      Files: none (read-only check).
      Verify: `cargo check -p ff-desktop` (from the worktree root) succeeds, so the
      starting point compiles before any move.

---

## Task 0 (prereq for Task 18): extract `posix_provider` -> `ff-posix-provider`

Leaf: `PosixProvider` (+ `resolve_posix_path`, `to_posix_path`). ZERO `crate::`
coupling. Before: `crates/ff-desktop/src/posix_provider.rs` ~370 total lines,
~224 non-test (test module `#[cfg(test)]` at line 225). Non-ASCII to fix on move:
U+2014 em dashes, U+2013 en dashes (`7.1-7.7`, `4-10`), and box-drawing comment
separators (`// -- Path helpers --`, `// -- Provider --`, `// -- Tests --`) -> ASCII
`// === Section ===` form. Consumers to keep working via the adapter:
`crate::posix_provider::PosixProvider` in `shell/render.rs` (5 sites) and in the
`nav_model` tests (rewired in Task 18).

- [ ] 0.1 Create `crates/ff-posix-provider/Cargo.toml`.
      Deps:
      `ff-connector-local-fs = { path = "../ff-connector-local-fs" }`,
      `ff-vfs = { path = "../ff-vfs" }`,
      `async-trait = "0.1"`.
      Dev-deps: `tokio = { workspace = true }`, `pretty_assertions = { workspace = true }`
      (the `#[tokio::test]` cases need the tokio runtime).
      `description = "POSIX VFS provider (scheme posix, root-jailed local FS) for FileForgeWorkbench"`.
      Files (create): `c:\workspace\VSC\FileForgeWorkbench\.worktrees\decomp-wave5\crates\ff-posix-provider\Cargo.toml`
      Verify: file exists; TOML parses (checked by the `cargo` build in 0.4).

- [ ] 0.2 Move the full body of `ff-desktop/src/posix_provider.rs` (the
      `#![allow(dead_code)]` is NOT needed in a library crate -- drop it; the items
      are `pub`/`pub(crate)` and used by tests/consumers) into
      `crates/ff-posix-provider/src/lib.rs`, INCLUDING its `#[cfg(test)] mod tests`.
      Change `pub(crate) fn resolve_posix_path` and `pub(crate) fn to_posix_path`
      to `pub fn` so the re-export exposes them unchanged (they are only referenced
      within this module today, so widening visibility is behaviour-neutral).
      Convert all non-ASCII comment chars to ASCII (em/en dashes, box-drawing).
      Files (create): `...\crates\ff-posix-provider\src\lib.rs`
      Verify: covered by 0.4.

- [ ] 0.3 Replace `ff-desktop/src/posix_provider.rs` with the thin adapter
      `pub use ff_posix_provider::*;` (one line + a short module doc comment). Keep
      `mod posix_provider;` in `main.rs` so `crate::posix_provider::PosixProvider`
      resolves via the re-export. Add `ff-posix-provider = { path = "../ff-posix-provider" }`
      to `ff-desktop/Cargo.toml` `[dependencies]`, and add `"crates/ff-posix-provider"`
      to root `Cargo.toml` `members` (above `"crates/ff-desktop"`).
      Files (edit): `...\crates\ff-desktop\src\posix_provider.rs`,
      `...\crates\ff-desktop\Cargo.toml`, `...\Cargo.toml`
      Verify: covered by 0.4.

- [ ] 0.4 Scoped gate for Task 0.
      Files: none.
      Verify (run from the worktree root):
      `cargo fmt --check`;
      `cargo clippy -p ff-posix-provider -p ff-desktop -- -D warnings` (0 warnings);
      `cargo nextest run -p ff-posix-provider -p ff-desktop` -- the moved
      posix_provider tests (7+ cases) pass in the new crate AND the ff-desktop
      suite still passes unchanged (shell/render.rs resolves `PosixProvider` via
      the adapter). Record before/after lines (posix_provider.rs ~370 -> ~3-line
      adapter) in `docs/project-management/ffdesktop-decomposition-baseline.md`.

---

## Task 17: extract `context_menu` -> `ff-context-menu`

Leaf: `FileClass`, `EXTERNAL_EXTENSIONS`, `classify_extension`, `classify_file`,
`is_text_bytes`, `launch_default_app`, `reveal_in_explorer`. ZERO `crate::`
coupling; external deps = std only (no egui, no ff-* crates). Before:
`crates/ff-desktop/src/context_menu.rs` 317 total lines, ~156 non-test
(`#[cfg(test)]` at line 157). Non-ASCII to fix on move: U+2014 em dashes in the
module doc comment and several inline comments / `FileClass` variant docs.

CAUTION -- `reveal_in_explorer` is `pub(crate)`. For `crate::context_menu::reveal_in_explorer`
(if referenced in ff-desktop) to keep resolving through `pub use ff_context_menu::*;`,
it MUST be `pub` in the new crate. Change it to `pub fn reveal_in_explorer` on the
move (behaviour-neutral; glob re-export then re-exposes it crate-locally).

- [ ] 17.1 Create `crates/ff-context-menu/Cargo.toml`.
      Deps: none (std only).
      Dev-deps: `pretty_assertions = { workspace = true }` (optional; current tests
      use `assert_eq!`/`assert!` only, so dev-deps may be omitted entirely -- match
      whichever keeps clippy/fmt clean; prefer NO dev-deps if unused).
      `description = "File classification + OS-integration helpers for the File Explorer (FileForgeWorkbench)"`.
      Files (create): `...\crates\ff-context-menu\Cargo.toml`
      Verify: covered by 17.4.

- [ ] 17.2 Move the full body of `ff-desktop/src/context_menu.rs` into
      `crates/ff-context-menu/src/lib.rs`, INCLUDING its `#[cfg(test)] mod tests`.
      Promote `reveal_in_explorer` from `pub(crate)` to `pub`. Keep `#[allow(dead_code)]`
      on `FileClass` if clippy still flags unused variants in the library context
      (verify in 17.4; remove it if the move makes the items used/exported such
      that the lint no longer fires). Convert em dashes to `--` / `-`.
      Files (create): `...\crates\ff-context-menu\src\lib.rs`
      Verify: covered by 17.4.

- [ ] 17.3 Replace `ff-desktop/src/context_menu.rs` with the thin adapter
      `pub use ff_context_menu::*;`. Keep `mod context_menu;` (or `pub(crate) mod
      context_menu;` -- preserve whichever declaration `main.rs` currently uses) so
      `crate::context_menu::{classify_file, FileClass, ...}` still resolve. Add
      `ff-context-menu = { path = "../ff-context-menu" }` to `ff-desktop/Cargo.toml`
      `[dependencies]`; add `"crates/ff-context-menu"` to root `Cargo.toml` members.
      Files (edit): `...\crates\ff-desktop\src\context_menu.rs`,
      `...\crates\ff-desktop\Cargo.toml`, `...\Cargo.toml`
      Verify: covered by 17.4.

- [ ] 17.4 Scoped gate for Task 17.
      Files: none.
      Verify (worktree root):
      `cargo fmt --check`;
      `cargo clippy -p ff-context-menu -p ff-desktop -- -D warnings`;
      `cargo nextest run -p ff-context-menu -p ff-desktop` -- the moved
      context_menu tests (~15 cases) pass in the new crate AND ff-desktop
      (explorer_view's `crate::context_menu::classify_file`/`FileClass` refs still
      resolve via the adapter until Task 19 rewires them). Append the before/after
      row (context_menu.rs 317 -> ~3-line adapter) to the baseline doc; update the
      TCR Req 19.3/19.4 row.

---

## Task 18: extract `nav_model` -> `ff-nav-model` (depends on Task 0)

`NavModel` on `ff-file-tree`, using `ff_vfs::ResourceUri`. Non-test code is
`crate::`-free. Its `#[cfg(test)] mod tests` uses
`crate::posix_provider::PosixProvider::new(...)` in 5 sites (lines 444, 473, 497,
548, 572). Before: `crates/ff-desktop/src/nav_model.rs` ~843 total lines, ~303
non-test (`#[cfg(test)]` at line 304). ASCII already clean (no non-ASCII found).

- [ ] 18.1 Create `crates/ff-nav-model/Cargo.toml`.
      Deps:
      `ff-file-tree = { path = "../ff-file-tree" }`,
      `ff-vfs = { path = "../ff-vfs" }`,
      `tokio = { workspace = true }` (non-test `list_via_provider` takes
      `&tokio::runtime::Runtime`).
      Dev-deps:
      `ff-posix-provider = { path = "../ff-posix-provider" }` (Task 0),
      `async-trait = "0.1"` (the in-test `MockProvider` impls `#[async_trait::async_trait]`),
      `tempfile = { workspace = true }`,
      `pretty_assertions = { workspace = true }`.
      `description = "File Explorer navigation model (NavModel over ff-file-tree) for FileForgeWorkbench"`.
      Files (create): `...\crates\ff-nav-model\Cargo.toml`
      Verify: covered by 18.4.

- [ ] 18.2 Move the full body of `ff-desktop/src/nav_model.rs` into
      `crates/ff-nav-model/src/lib.rs`, INCLUDING its `#[cfg(test)] mod tests`.
      In the test module ONLY, rewrite the 5 occurrences of
      `crate::posix_provider::PosixProvider::new(...)` to
      `ff_posix_provider::PosixProvider::new(...)` (identical type + signature +
      behaviour -- the SAME impl now lives in ff-posix-provider). Make NO other
      change to any test (do not weaken or delete assertions). The module-level
      `#[allow(dead_code)]`-style attributes on individual methods (`uri_count`)
      stay as-is unless clippy requires otherwise (verify in 18.4).
      Files (create): `...\crates\ff-nav-model\src\lib.rs`
      Verify: covered by 18.4.

- [ ] 18.3 Replace `ff-desktop/src/nav_model.rs` with the thin adapter
      `pub use ff_nav_model::*;`. Keep the `mod nav_model;` declaration so
      `crate::nav_model::{NavModel, split_catalog_uri_path, ...}` still resolve
      (used by `explorer_view.rs` until Task 19 and by `shell/render.rs` via
      `crate::nav_model::split_catalog_uri_path`). Add
      `ff-nav-model = { path = "../ff-nav-model" }` to `ff-desktop/Cargo.toml`
      `[dependencies]`; add `"crates/ff-nav-model"` to root `Cargo.toml` members.
      Files (edit): `...\crates\ff-desktop\src\nav_model.rs`,
      `...\crates\ff-desktop\Cargo.toml`, `...\Cargo.toml`
      Verify: covered by 18.4.

- [ ] 18.4 Scoped gate for Task 18.
      Files: none.
      Verify (worktree root):
      `cargo fmt --check`;
      `cargo clippy -p ff-nav-model -p ff-posix-provider -p ff-desktop -- -D warnings`;
      `cargo nextest run -p ff-nav-model -p ff-posix-provider -p ff-desktop` -- all
      moved nav_model tests (including the 5 provider-backed cases now using
      `ff_posix_provider::PosixProvider`) pass in the new crate AND ff-desktop
      still passes (shell/render.rs + explorer_view resolve `crate::nav_model::*`
      via the adapter). Append the before/after row (nav_model.rs ~843 -> ~3-line
      adapter) to the baseline doc; update the TCR Req 19.3/19.4 row.

---

## Task 19: extract `explorer_view` -> `ff-explorer-view` (depends on 17 + 18)

Panel + interaction core. Couples to `crate::nav_model::NavModel` (import at
line 24; doc-comment reference at line 4) and `crate::context_menu::{classify_file,
FileClass}` (3 refs at lines 505-509, in the `resolve_open` free fn). External
deps: `eframe::egui`, `ff_file_tree`, `ff_theme`, `ff_vfs`. Before:
`crates/ff-desktop/src/explorer_view.rs` ~1086 total lines, ~767 NON-TEST
(`#[cfg(test)]` at line 768). ASCII already clean. The test module uses only
`super::*` (no `crate::` refs), so no test rewrite is required beyond what the
`use` rewrite in the parent module already provides.

NOTE (400-line rule): after the move, `ff-explorer-view/src/lib.rs` will hold
~767 non-test lines, which EXCEEDS the 400 non-test-line guideline. This is a
pure behaviour-preserving MOVE; per the Wave-5 slicing note, do NOT re-slice
beyond trivial. Record the over-limit fact in the baseline doc as a known
follow-up (a future `_state.rs`/`_render.rs` split is a separate refactor), but
do not block this task on it.

- [ ] 19.1 Create `crates/ff-explorer-view/Cargo.toml`.
      Deps:
      `ff-nav-model = { path = "../ff-nav-model" }`,
      `ff-context-menu = { path = "../ff-context-menu" }`,
      `ff-file-tree = { path = "../ff-file-tree" }`,
      `ff-theme = { path = "../ff-theme" }`,
      `ff-vfs = { path = "../ff-vfs" }`,
      `egui = { workspace = true }`,
      `eframe = { workspace = true }` (the module imports `eframe::egui`; confirm in
      19.4 whether `egui` alone suffices -- the current source does `use eframe::egui;`,
      so keep `eframe` unless the move is changed to `use egui;` which would alter
      the import, so KEEP `eframe` to stay behaviour/source-preserving).
      Dev-deps: `pretty_assertions = { workspace = true }` (optional; tests use
      `assert_eq!`/`panic!` only -- include only if used, else omit to stay clippy-clean).
      `description = "NavModel-backed File Explorer view + interaction core (egui) for FileForgeWorkbench"`.
      Files (create): `...\crates\ff-explorer-view\Cargo.toml`
      Verify: covered by 19.4.

- [ ] 19.2 Move the full body of `ff-desktop/src/explorer_view.rs` into
      `crates/ff-explorer-view/src/lib.rs`, INCLUDING its `#[cfg(test)] mod tests`.
      Rewrite the crate-local couplings so the crate is self-contained:
      - `use crate::nav_model::NavModel;` -> `use ff_nav_model::NavModel;` (line 24).
      - The 3 `crate::context_menu::` refs in `resolve_open` ->
        `ff_context_menu::` (`ff_context_menu::classify_file(...)`,
        `ff_context_menu::FileClass::{Text, FfwbStructured, External}`).
      - The doc-comment reference `[`crate::nav_model::NavModel`]` (line 4) ->
        `[`ff_nav_model::NavModel`]` (cosmetic; keeps the doc link valid).
      Make NO other change (no test edits -- tests use `super::*`).
      Files (create): `...\crates\ff-explorer-view\src\lib.rs`
      Verify: covered by 19.4.

- [ ] 19.3 Replace `ff-desktop/src/explorer_view.rs` with the thin adapter
      `pub use ff_explorer_view::*;`. Keep the `mod explorer_view;` declaration so
      every `crate::explorer_view::*` reference in ff-desktop (shell render/update,
      etc.) still resolves. Add `ff-explorer-view = { path = "../ff-explorer-view" }`
      to `ff-desktop/Cargo.toml` `[dependencies]`; add `"crates/ff-explorer-view"`
      to root `Cargo.toml` members.
      Files (edit): `...\crates\ff-desktop\src\explorer_view.rs`,
      `...\crates\ff-desktop\Cargo.toml`, `...\Cargo.toml`
      Verify: covered by 19.4.

- [ ] 19.4 Scoped gate for Task 19.
      Files: none.
      Verify (worktree root):
      `cargo fmt --check`;
      `cargo clippy -p ff-explorer-view -p ff-nav-model -p ff-context-menu -p ff-posix-provider -p ff-desktop -- -D warnings`;
      `cargo nextest run -p ff-explorer-view -p ff-nav-model -p ff-context-menu -p ff-posix-provider -p ff-desktop`
      -- all moved explorer_view tests (~20 cases) pass in the new crate AND the
      full ff-desktop suite still passes (every `crate::explorer_view::*` consumer
      resolves via the adapter). Append the before/after row (explorer_view.rs ~1086
      -> ~3-line adapter; note the ~767 non-test lines now in ff-explorer-view
      exceed the 400 guideline -- known follow-up). Update the TCR Req 19.3/19.4 row.

---

## Full-gate hand-off (after Task 19 scoped-clean)

Kiro runs ONLY the scoped `-p` checks above. After Task 19's scoped gate is
clean, STOP and hand off to the owner to run the full gate manually, OUTSIDE Kiro:
`powershell -ExecutionPolicy Bypass -File tools\powershell\verify.ps1`
Do NOT run `verify.ps1` or any `--workspace` build/test from Kiro. The task is
"code-complete pending full gate" until the owner reports a clean run (empty
`ai-review.log`).

## Order of execution (dependency-correct, strictly sequential)

0 (ff-posix-provider) -> 17 (ff-context-menu) -> 18 (ff-nav-model, needs 0)
-> 19 (ff-explorer-view, needs 17 + 18). Tasks 0 and 17 are mutually independent
leaves; 17 may run before 0 if preferred, but 0 MUST precede 18 and 17+18 MUST
precede 19.

## Files summary (new crates created)

- `crates/ff-posix-provider/` (Cargo.toml + src/lib.rs) -- Task 0
- `crates/ff-context-menu/` (Cargo.toml + src/lib.rs) -- Task 17
- `crates/ff-nav-model/` (Cargo.toml + src/lib.rs) -- Task 18
- `crates/ff-explorer-view/` (Cargo.toml + src/lib.rs) -- Task 19

Each leaves a thin `pub use ff_<name>::*;` adapter at the original
`crates/ff-desktop/src/<module>.rs` path, keeps the `mod <module>;` decl, adds a
path dep in `ff-desktop/Cargo.toml`, and registers the crate in the explicit root
`Cargo.toml` members list.
