# Implementation Plan -- shell file-size split (behaviour-preserving)

Pure file-split refactor. Zero observable behaviour change. No requirements
gate (per workflow.md "What does NOT need the gate" and rust-standards.md
Source File Size section). All edits are made DIRECTLY in the main workspace at
`c:\workspace\VSC\FileForgeWorkbench` -- NO worktree, NO branch, NO commit.
Leave all changes UNCOMMITTED.

Three files in `crates/ff-desktop/src/shell/` exceed the 400 non-test-line rule
(confirmed by `tools/python/check_line_limits.py`):
- `construct.rs` -- 453 non-test lines
- `commands.rs`  -- 446 non-test lines
- `state.rs`     -- 432 non-test lines

None of the three files currently contains a `#[cfg(test)]` module, so every
line counts. The new sibling modules are declared in `shell/mod.rs` with
`mod <name>;` lines placed in the existing alphabetical-ish block (same style as
`construct`, `commands`, `commands_ladder_a`, `state`, `state_groups`,
`mod_helpers`).

GLOBAL CONSTRAINTS for every item below:
- Move items VERBATIM: no logic edits, no renamed public items, no changed
  signatures, no reordered field initialisation.
- All methods are `impl WorkbenchShell` methods and keep their existing
  `pub(super)` / `pub(crate)` visibility; moving them to a sibling `impl
  WorkbenchShell` block in the same `shell` module keeps every `self.xxx()`
  call site resolving unchanged (methods resolve on the type, not the file).
- Free functions move with their existing visibility; where a mover is called
  from the origin file it is imported via `use super::<fn>;` (or a module path).
- Keep ALL `.rs` files ASCII-only (no em dash, curly quotes, or unicode arrows);
  use ASCII section separators `// === Name ===`.
- Do NOT touch the notification channel, `ShellServices`, or `ShellRequest`
  surfaces. Do NOT touch any file outside `crates/ff-desktop/src/shell/` except
  `shell/mod.rs`. Do NOT touch catalog crates (ff-dscatalog /
  ff-dataset-catalog).
- After each split BOTH the primary file and the new file must be under 400
  non-test lines.

Verification command (used by several items below):
`C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "C:\tools\python\python.exe tools/python/check_line_limits.py > tools/logs/line-limits.txt 2>&1"`
then read `tools/logs/line-limits.txt`.
Scoped build/lint (Kiro-run): `cargo check -p ff-desktop`,
`cargo clippy -p ff-desktop`, `cargo fmt`. (Full `cargo gate --build` is the
owner's manual step.)

---

- [ ] 1. Split `construct.rs`: move the free function `build_live_provider_registry` to a new sibling `construct_provider.rs`.
      This is the single standalone free fn at the bottom of `construct.rs` (its
      doc comment through the closing brace, including the `use
      ff_connector_local_fs::LocalFsProvider;` / `use ff_vfs::VfsProvider;` inner
      `use` lines inside the fn body). It is a self-contained ~77-line unit called
      exactly once, from `new_with_history_store`, as `build_live_provider_registry(&runtime)`.
      Steps:
      - Create `crates/ff-desktop/src/shell/construct_provider.rs` with a module
        doc comment (ASCII, explaining it holds the live Provider_Registry builder
        extracted from `construct.rs`, pure code movement). Add the top-level
        `use std::sync::Arc;` and `use tokio::runtime::Runtime;` it needs, plus the
        `use ff_vfs;` path references (the fn body uses `ff_vfs::ProviderRegistry`,
        `ff_logging::log_warn!`; these are crate-root paths needing no extra use).
        Paste the fn VERBATIM; keep its signature `fn build_live_provider_registry(runtime: &Runtime) -> Arc<ff_vfs::ProviderRegistry>`
        but change its visibility to `pub(super)` so `construct.rs` can call it.
      - In `construct.rs`: delete the moved fn and add `use super::construct_provider::build_live_provider_registry;`
        to the import block. The existing call site
        `let provider_registry = build_live_provider_registry(&runtime);` is
        unchanged. Remove now-unused imports from `construct.rs` ONLY if they were
        used solely by the moved fn (none expected -- `Arc`, `Runtime` are still
        used by the constructor; verify `cargo check` shows no unused-import
        warning and leave the rest intact).
      Files: create `crates/ff-desktop/src/shell/construct_provider.rs`; modify
      `crates/ff-desktop/src/shell/construct.rs`.
      Verify: `cargo check -p ff-desktop` compiles with no new warnings; the
      line-limit checker no longer lists `construct.rs` and does not list
      `construct_provider.rs`.

- [ ] 2. Declare the `construct_provider` module in `shell/mod.rs`.
      Add the line `mod construct_provider;` in the module-declaration block,
      immediately after the existing `mod construct;` line (keeps the
      construct-family declarations together and matches the file's ordering
      style).
      Files: modify `crates/ff-desktop/src/shell/mod.rs`.
      Verify: `cargo check -p ff-desktop` compiles (the new module resolves);
      no "file not found for module" error.

- [ ] 3. Split `commands.rs` part A: move the Command_Environment addressing method `dispatch_to_environment` to a new sibling `commands_environment.rs`.
      `dispatch_to_environment(&mut self, name: &str, raw: &str) -> EnvDispatchOutcome`
      is the self-contained ~45-line method (doc comment + body) at the top of the
      `impl WorkbenchShell` block in `commands.rs`. It references
      `EnvDispatchOutcome`, `RegisteredEnv`, and `super::environment_ffedit::FfEditEnvironment`.
      Steps:
      - Create `crates/ff-desktop/src/shell/commands_environment.rs` with an ASCII
        module doc comment (extracted environment-addressing method, pure code
        movement). Imports: `use super::environment::EnvDispatchOutcome;`,
        `use super::environment_registry::RegisteredEnv;`, and
        `use super::WorkbenchShell;`. Open `impl WorkbenchShell { ... }` and paste
        `dispatch_to_environment` VERBATIM, keeping `pub(super)` visibility and its
        signature. (The body's `super::environment_ffedit::FfEditEnvironment` and
        `self.host_fs_save()` references resolve unchanged.)
      - In `commands.rs`: delete the moved method. Its callers
        (`run_command_ladder` in the same file, calling
        `self.dispatch_to_environment(...)`) resolve on the type and need no
        change. Keep the existing `use super::environment::{CommandEnvironment,
        EnvDispatchOutcome};` and `use super::environment_registry::RegisteredEnv;`
        in `commands.rs` ONLY if still used by remaining code; if `RegisteredEnv`
        or `CommandEnvironment` becomes unused after the move, remove just the
        now-unused name (verify via `cargo check` unused-import warnings).
      Files: create `crates/ff-desktop/src/shell/commands_environment.rs`; modify
      `crates/ff-desktop/src/shell/commands.rs`.
      Verify: `cargo check -p ff-desktop` compiles with no new warnings.

- [ ] 4. Split `commands.rs` part B: move the command-line field lifecycle trio to the new `commands_environment.rs` OR a dedicated `commands_line.rs`.
      Decision: use a dedicated `commands_line.rs` so each new file holds ONE
      concern (environment addressing vs command-line field lifecycle), matching
      the one-concern-per-file convention and keeping both new files small.
      Move the three cohesive methods `run_command_line`, `begin_command_line`,
      and `finish_command_line` (contiguous ~55 lines incl. docs) that together
      own the `Command ===>` field disposition.
      Steps:
      - Create `crates/ff-desktop/src/shell/commands_line.rs` with an ASCII module
        doc comment. Imports: `use super::WorkbenchShell;`. `finish_command_line`
        uses `crate::shell::command_line_outcome::CommandLineOutcome` via a local
        `use` inside the fn body -- keep that inner `use` exactly as written.
        Open `impl WorkbenchShell { ... }` and paste the three methods VERBATIM,
        each keeping `pub(super)` visibility.
      - In `commands.rs`: delete the three moved methods. Callers resolve on the
        type: `run_command_line` is invoked from the render/update layer and from
        `handle_command`'s neighbours; `begin_command_line`/`finish_command_line`
        are invoked by `run_command_line` (now in the new file) and by
        `dispatch_key_command` (another sibling module) -- all `self.*()` calls
        resolve unchanged.
      Files: create `crates/ff-desktop/src/shell/commands_line.rs`; modify
      `crates/ff-desktop/src/shell/commands.rs`.
      Verify: `cargo check -p ff-desktop` compiles with no new warnings; the
      line-limit checker no longer lists `commands.rs` and lists neither
      `commands_environment.rs` nor `commands_line.rs`.

- [ ] 5. Declare the `commands_environment` and `commands_line` modules in `shell/mod.rs`.
      Add `mod commands_environment;` and `mod commands_line;` in the
      `commands*` declaration block (after `mod commands;`, among
      `commands_fastpath`, `commands_ladder_a`, etc.), preserving the existing
      ordering style.
      Files: modify `crates/ff-desktop/src/shell/mod.rs`.
      Verify: `cargo check -p ff-desktop` compiles; both modules resolve.

- [ ] 6. Decide the `state.rs` split approach (grouping, not method-extraction).
      `state.rs` is a single `WorkbenchShell` struct definition plus the small
      `PendingOpen` type alias -- it has no separable method or free-fn block to
      move. The behaviour-preserving way to shrink it is the pattern ALREADY in
      use in this exact file: group a cohesive cluster of currently-flat fields
      into a `Default`-derived sub-struct in `state_groups.rs` (the file already
      does this for `focus: state_groups::FocusState`, `detach_split:
      state_groups::DetachSplitState`, `dir_overrides: state_groups::DirOverrides`).
      This is a source-level access-path change (`self.nav_x` ->
      `self.nav_ui.nav_x`) with identical runtime values, so it is
      behaviour-preserving. Items 7-8 carry it out. (Extracting only the
      `PendingOpen` alias would save ~5 lines -- insufficient for the 32-line
      overage -- so it is NOT the chosen approach.)
      Files: (decision item -- implemented by items 7-8.)
      Verify: covered by items 7-8.

- [ ] 7. Group the modern-explorer navigation fields into a new `NavUiState` sub-struct in `state_groups.rs`, mirroring the existing `FocusState`/`DetachSplitState` grouping already used in `state.rs`.
      This is the established, behaviour-preserving simplification pattern in this
      codebase (the struct doc for `state.rs` explicitly documents it). Group the
      SIX contiguous `nav_*` UI fields that form one cohesive cluster:
      `nav_selection: crate::explorer_view::ExplorerSelection`,
      `nav_rename: Option<(ff_file_tree::NodeId, String)>`,
      `nav_delete: Option<(ff_file_tree::NodeId, String)>`,
      `nav_new: Option<(ff_file_tree::NodeId, bool, String)>`,
      `nav_focused: bool`,
      `nav_file_clipboard: Vec<ff_vfs::ResourceUri>`.
      (Keep `nav_model` on the struct -- it is accessed very widely and is a
      distinct heavier model; grouping only the six UI-interaction fields is the
      lowest-churn cohesive move and removes ~20 field+doc lines from `state.rs`.)
      Steps:
      - In `state_groups.rs`: add a new `// === NavUiState ===` section and a
        `#[derive(Debug, Default)] pub(crate) struct NavUiState { ... }` whose six
        fields carry the SAME types and the SAME doc comments moved verbatim from
        `state.rs`. Add the needed `use`/path references (`ff_file_tree`, `ff_vfs`,
        `crate::explorer_view` are crate-root paths; reference them fully-qualified
        in the field types exactly as in `state.rs`, so no new `use` is required
        beyond what the field types already name).
      - In `state.rs`: replace the six fields with a single
        `pub(crate) nav_ui: state_groups::NavUiState,` field (place it where the
        `nav_selection` field was, keeping the doc comment summarising the group,
        same style as the `focus` / `detach_split` fields). Confirm
        `state_groups` is already imported in `state.rs` (it is:
        `use super::{..., state_groups, ...};`).
      Files: modify `crates/ff-desktop/src/shell/state_groups.rs`; modify
      `crates/ff-desktop/src/shell/state.rs`.
      Verify: `cargo check -p ff-desktop` -- expected to FAIL first with
      access-path errors at every `self.nav_selection` / `self.nav_rename` /
      `self.nav_delete` / `self.nav_new` / `self.nav_focused` /
      `self.nav_file_clipboard` call site, which item 8 fixes. (Do not treat this
      intermediate failure as done.)

- [ ] 8. Update every call site of the six grouped nav fields to the `self.nav_ui.<field>` path, and update the `construct.rs` initialiser.
      This is one mechanical rename applied across all files under
      `crates/ff-desktop/src/shell/` (and only there) that read/write the six
      fields. Use the compiler error list from item 7 as the authoritative set of
      sites; also grep to confirm completeness.
      Steps:
      - In `construct.rs`: replace the six flat field initialisers in the `Self {
        ... }` literal
        (`nav_selection: ...default(), nav_rename: None, nav_delete: None,
        nav_new: None, nav_focused: false, nav_file_clipboard: Vec::new(),`) with a
        single `nav_ui: state_groups::NavUiState::default(),` (all six defaults are
        the type's `Default`, so this is behaviour-identical). `state_groups` is
        already imported in `construct.rs`.
      - Across the other shell siblings (render_nav.rs, render_nav_expand.rs,
        render_nav_ops.rs, update_input.rs, update_keys.rs, nav_* modules, etc.):
        rewrite `self.nav_selection` -> `self.nav_ui.nav_selection`,
        `self.nav_rename` -> `self.nav_ui.nav_rename`,
        `self.nav_delete` -> `self.nav_ui.nav_delete`,
        `self.nav_new` -> `self.nav_ui.nav_new`,
        `self.nav_focused` -> `self.nav_ui.nav_focused`,
        `self.nav_file_clipboard` -> `self.nav_ui.nav_file_clipboard`.
        Keep the field names INSIDE the sub-struct identical (so the only textual
        change is the inserted `nav_ui.`); this keeps the diff mechanical and the
        semantics identical.
      Files: modify `crates/ff-desktop/src/shell/construct.rs` and the shell
      sibling modules that reference the six fields (identified by the item-7
      compiler errors and by `grep_search` for `self.nav_selection`,
      `self.nav_rename`, `self.nav_delete`, `self.nav_new`, `self.nav_focused`,
      `self.nav_file_clipboard`).
      Verify: `cargo check -p ff-desktop` compiles with no new warnings;
      `cargo clippy -p ff-desktop` clean; the line-limit checker no longer lists
      `state.rs`.

- [ ] 9. Final conformance pass: ASCII check, formatting, scoped build/lint/tests, and line-limit re-confirmation.
      Steps:
      - Confirm the three new/changed files are ASCII-only and use `// === Name ===`
        separators (no em dash / curly quotes / unicode arrows). Grep:
        the documentation.md enforcement pattern over the touched `.rs` files.
      - Run `cargo fmt` (whole workspace fmt is fine; it only reformats).
      - Run the line-limit checker; confirm NONE of `construct.rs`, `commands.rs`,
        `state.rs`, `construct_provider.rs`, `commands_environment.rs`,
        `commands_line.rs`, `state_groups.rs` exceeds 400 non-test lines.
      - Run `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop`, and
        `cargo test -p ff-desktop` (scoped). All must pass -- behaviour is
        unchanged, so the existing shell test suite is the behaviour-preservation
        safety net.
      Files: none beyond those above (fmt may touch formatting only).
      Verify: `tools/logs/line-limits.txt` lists none of the shell files;
      `cargo check -p ff-desktop` + `cargo clippy -p ff-desktop` + `cargo test -p
      ff-desktop` all succeed. Then HAND OFF to the owner for the full
      `cargo gate --build`.

---

## Call-site resolution summary (why nothing breaks)

- Methods moved between sibling modules (`build_live_provider_registry` is a free
  fn; `dispatch_to_environment`, `run_command_line`, `begin_command_line`,
  `finish_command_line` are `impl WorkbenchShell` methods) all stay in the same
  `shell` module tree. `impl` methods resolve on the TYPE regardless of file, so
  every `self.method()` call site is unchanged. The one free fn is re-imported in
  its single caller via `use super::construct_provider::build_live_provider_registry;`.
- The `nav_ui` grouping is the ONLY call-site change, and it is a pure, mechanical
  path rewrite (`self.nav_x` -> `self.nav_ui.nav_x`) confined to `crates/ff-desktop/
  src/shell/`. The sub-struct derives `Default`, so the `Self { .. }` initialiser
  collapses six `Default`/`None`/`Vec::new()` entries into one `::default()` with
  identical runtime values. No field type, no public item, no signature changes.

## Assumptions / notes

- `state.rs` has no cleanly separable method/free-fn block (it is a single struct
  definition), so the field-grouping sub-struct approach -- already the
  documented, in-use pattern in this exact file -- is the correct
  behaviour-preserving way to bring it under 400. The chosen six-field cluster is
  the lowest-churn cohesive group (modern-explorer UI interaction state) and
  removes ~20 lines, clearing the 32-line overage with margin.
- If, after item 1, `construct.rs` is still at or near the limit (unlikely --
  removing ~77 lines takes 453 -> ~376), additionally move the two marker-handler
  registration blocks is NOT needed; re-check the log before adding more.
