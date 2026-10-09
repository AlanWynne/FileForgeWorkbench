# Behaviour-preserving file-size split of three shell modules

Three `crates/ff-desktop/src/shell/` files that had crept back over the 400
non-test-line rule (`construct.rs` 453, `commands.rs` 446, `state.rs` 432) were
each brought under the limit by relocating cohesive item groups into new sibling
modules and grouping flat `WorkbenchShell` fields into Default-derived
sub-structs. The moves are verbatim: no signature changes, no renamed public
items, no logic edits -- only relocation plus the mechanical
`use` / `pub(super)` / `mod` wiring and a field-access-path rewrite
(`self.nav_selection` -> `self.nav_ui.nav_selection`, etc.). All five
new/extended modules are declared in `shell/mod.rs`, and the resume note records
the pass.

Watch for: nothing blocking. The only non-mechanical delta is one deliberate
visibility widen (`build_live_provider_registry` `fn` -> `pub(super) fn`, needed
because its new home is a sibling module), which is correct and does not change
behaviour (confirmed). The large working-tree diff also contains unrelated
volume/catalog spec and status-doc edits from the parallel worktree; those are
out of scope for this split and were not reviewed as part of it.

**Verdict**: APPROVED

## High-level view

The relocation is clean. Each moved item -- the free fn
`build_live_provider_registry`, the four built-in command registrations (now the
`register_builtin_commands` helper), the `dispatch_to_environment` environment
addressing method, and the `run_command_line` / `begin_command_line` /
`finish_command_line` trio -- appears byte-for-byte in its new file with the same
body the diff removed from the old file. Signatures and `pub(super)` visibility
are preserved on everything except the one necessary widen noted above.

The `state.rs` change groups three field clusters (`NavUiState`, `HelpState`,
`PendingTabActions`) into sub-structs in `state_groups.rs`, mirroring the
existing `FocusState` / `DetachSplitState` pattern. Every former flat field keeps
its exact type and doc comment; only the access path changes. All call sites
across the eight render/update/help modules were rewritten to the grouped path,
and the diff shows those rewrites are purely `self.X` -> `self.group.X` with no
behavioural edits.

Call-site resolution holds: `register_builtin_commands` takes `&CommandRegistry`,
`&PendingOpen`, `&Arc<Mutex<bool>>` and is called with exactly those; the
`PendingOpen` alias it imports is `pub(super)` in `state.rs` and reachable from
the sibling module. The notification channel, `ShellServices`, and
`ShellRequest` surfaces are not touched by any file in this split.

Line counts land where the verification note claims: construct.rs 348,
commands.rs 330, state.rs 399, state_groups.rs 171, and the four new files
57 / 83 / 67 / 71 -- all under 400, none newly over. All affected `.rs` files are
ASCII-only.

<details>
<summary>Issues (0)</summary>

No blocking or actionable concerns. The split is behaviour-preserving.

</details>

<details>
<summary>Details</summary>

### Verbatim relocation of free functions and methods

The four extracted items were compared body-to-body between the removed hunks in
the old files and the added files:

- `build_live_provider_registry` in `construct_provider.rs` matches the block
  deleted from `construct.rs` exactly, including the two `log_warn!` strings and
  the `runtime.enter()` guard comment. Signature is identical except `fn` ->
  `pub(super) fn`, which is required for `construct.rs` to call it across the
  module boundary and is the one widen the verification note discloses. No
  behaviour change.
- `register_builtin_commands` in `construct_commands.rs` wraps the four
  `registry.register(...)` blocks (file.open, file.exit, menu.open, config.open)
  verbatim, parameterised on `&CommandRegistry`, `&PendingOpen`,
  `&Arc<Mutex<bool>>`. `construct.rs` now calls
  `register_builtin_commands(&registry, &pending_open, &should_close)` in place of
  the inlined blocks. The `CommandId` / `CommandMetadata` imports and the four
  handler imports moved with the code; `construct.rs` dropped the now-unused
  imports (`ShellContextProvider` and `CommandRegistry` retained, as needed).
- `dispatch_to_environment` in `commands_environment.rs` is identical to the
  removed method, keeping `pub(super)` and the full `RegisteredEnv` match arms
  (FfEdit / HostFsPlaceholder / FfCmdBase). It re-imports `CommandEnvironment`
  (for the `claim` trait method) and `RegisteredEnv`; `commands.rs` dropped those
  two now-unused imports and kept `EnvDispatchOutcome` (still used by
  `run_command_ladder`).
- `run_command_line` / `begin_command_line` / `finish_command_line` in
  `commands_line.rs` are byte-identical to the removed trio, including the inner
  `use crate::shell::command_line_outcome::CommandLineOutcome;` and the
  `CommandLineOutcome` match. All keep `pub(super)`.

### Field grouping in state.rs mirrors the existing sub-struct pattern

`state_groups.rs` gains three containers with ASCII `// === Name ===` separators:
`NavUiState` (`#[derive(Debug, Default)]`), `HelpState`, and `PendingTabActions`
(`#[derive(Debug, Default)]`). `HelpState` is not `Default`-derived because its
`registry` / `context_panel` carry no default -- it is constructed explicitly in
`construct.rs` with the real registry, matching the former flat initialisation.
Each field retained its original doc comment verbatim. `state.rs` replaces the
former flat fields with three grouped fields (`nav_ui`, `help`,
`pending_tab_actions`) whose doc comments point at the sub-structs.

The access-path rewrites in render_nav.rs, render_nav_ops.rs, update_input.rs,
help.rs, render_body.rs, render_body_arms.rs, render_tab_bar.rs, and update.rs are
each a one-for-one `self.<old_field>` -> `self.<group>.<field>` substitution with
no change to surrounding logic (the diff shows the control flow around each edit
is untouched; the only multi-line reflows are rustfmt wrapping the longer access
path).

### Scope containment and surfaces

No file outside `crates/ff-desktop/src/shell/` changed as part of this split
except `docs/status/RESUME-ffdesktop-simplification.md` (an accurate record of
the pass). `shell/mod.rs` adds exactly the four new `mod` lines
(`commands_environment`, `commands_line`, `construct_commands`,
`construct_provider`); `state_groups` was already declared. The notification
channel, `ShellServices`, and `ShellRequest` surfaces are not referenced by any
changed file. Catalog crates are untouched. The remaining spec/status-doc edits
in the working tree are the parallel volume/catalog worktree's work, outside this
split's scope.

### Evidence basis

Reviewed the coder's `verification.md`, the full `shell/` diff, the four new
files in full, the `state_groups.rs` additions, and the RESUME note diff. Spot
checks run: ASCII scan of the five new/extended files and the RESUME note (no
matches outside the allowed box-drawing range); `PendingOpen` type lookup
(confirms the `pub(super)` alias resolves for `construct_commands.rs`); and a
direct line count of the four target/extended files (348 / 330 / 399 / 171),
matching the note. The scoped suites (fmt / check / clippy / test /
check_line_limits) were NOT re-run per the step instruction; the known B048
multithreaded shared-env-var test flake is pre-existing and not a regression.

</details>

<details>
<summary>File map</summary>

In-scope (this split):
- `shell/construct.rs` -- 453 -> 348; inlined registrations and the provider
  builder replaced by calls into the two new helpers; field inits updated to
  grouped sub-structs.
- `shell/commands.rs` -- 446 -> 330; `dispatch_to_environment` and the
  command-line trio removed (moved out); unused imports dropped.
- `shell/state.rs` -- 432 -> 399; flat `nav_*` / `help_*` / `pending_*` fields
  replaced by three grouped sub-struct fields.
- `shell/construct_provider.rs` (new, 57) -- `build_live_provider_registry`.
- `shell/construct_commands.rs` (new, 83) -- `register_builtin_commands`.
- `shell/commands_environment.rs` (new, 67) -- `dispatch_to_environment`.
- `shell/commands_line.rs` (new, 71) -- command-line field lifecycle trio.
- `shell/state_groups.rs` -- extended to 171; adds `NavUiState`, `HelpState`,
  `PendingTabActions`.
- `shell/mod.rs` -- four new `mod` declarations.
- `shell/help.rs`, `render_body.rs`, `render_body_arms.rs`, `render_nav.rs`,
  `render_nav_ops.rs`, `render_tab_bar.rs`, `update.rs`, `update_input.rs` --
  grouped field-access-path rewrites only.
- `docs/status/RESUME-ffdesktop-simplification.md` -- resume note update.

Out of scope (parallel worktree, not reviewed here): dataset-allocator /
idcams-emulator / virtual-catalog-manager specs, project-master/tasks.md,
TCR.md, bugs.md, change-log.md, workflow.md, RESUME-volume-model.md, plan.md.

Full diff: `git diff -- crates/ff-desktop/src/shell/` in the workspace root.

</details>
