# Stream SAVE: Command Configurator + Config/Settings ISPF explicit-save, Help END guards

Two commits bring the last two autosaving editable Contexts (Config/Settings and the Command Configurator) onto the shared CR-CH-071 editable-Context seam, add regression guards for the already-fixed Help F3/END key path, and flip the project-master Requirement 11 conformance rows to COMPLIANT. Config/Settings gains a per-key working-copy buffer (`ConfigPanelState.working`); the Command Configurator gains a whole-store working copy (`CommandConfiguratorState.working: Option<Vec<CommandDefinition>>`) with delete deferred to persist. Both route SAVE/END/RETURN/REFRESH through the single dispatch seam; neither introduces a new dispatcher or an autosave path.

Watch for: (confirmed) S.1 landed as a fresh fix commit (abc991b6), not a literal cherry-pick of 0bcddf3 as the brief worded it — behaviorally equivalent, same 8 tests. (confirmed) The Command Configurator convention tests cover Req 11.2-11.5 but not 11.1 or 11.8 explicitly, where the brief asked for "11.1-11.5, 11.8"; 11.8 has no configurator analogue (no in-panel Save button) and 11.1 is implicitly exercised. (likely) `config_panel_persist` leaves the store partially written if a mid-flush key write fails — the commit message acknowledges this; it is a documented design choice, not a regression.

**Verdict**: APPROVED

## High-level view

The Command Configurator retrofit is the substantive piece and it is done correctly. Add/edit/delete all mutate a cloned whole-store working copy via `working_mut`, taken from the persisted definitions on first edit; the former per-action `command_store.save()` calls are gone. Persist swaps the working copy into the store and writes once, rolling the in-memory store back and retaining the working copy on write failure so END stays open and RETURN still works. Delete is correctly deferred — the test proves a delete click leaves both definitions on disk until SAVE.

Config/Settings follows the same shape with a per-key `HashMap<String, WorkingEdit>` buffer instead of a whole-store clone, because it overlays a live `ConfigHandle` rather than owning the store. Widget edits stage via `commit_value` -> `stage_set`; Reset stages a `WorkingEdit::Reset`; the in-panel Save button enqueues the `SAVE` command rather than persisting directly, so it shares the one seam. The `ConfigHandle` layered-store semantics are untouched.

The Help END fix (S.3) is not a code change to the dispatch path — the branch confirms via live repro that B080 dispatch unification already fixed it, and adds two regression guards. I verified this independently: `dispatch_bound_command` resolves then falls through to `dispatch_command_string`, which runs the ladder's `try_commands_a`; the END arm treats a non-editable Context (Help: `editable_context_kind()` is None) as pure `nav_end` navigation. The key path and the `handle_command` path converge on the same END handler, so the guards are meaningful.

Conformance docs and i18n are ASCII-clean in the lines this change touched. All changed source files are well under the 400-line limit (largest touched non-test file is `command_config/render.rs` at ~350 total, test module included).

<details>
<summary>Issues (4)</summary>

1. **S.1 delivered as a fix commit, not a cherry-pick** (confirmed) — The brief asked to cherry-pick 0bcddf3; the branch instead carries a standalone fix commit (abc991b6) with the same 8 `config_save_convention` tests and the same file set. Behaviorally equivalent; confirm the owner is fine with the history shape since nothing is merged/pushed.
2. **Configurator tests omit Req 11.1 and 11.8** (confirmed) — `tests_configurator_save_convention.rs` has 7 tests covering 11.2/11.3/11.4/11.5. The brief listed "11.1-11.5, 11.8". 11.8 (Save affordance shares the path) has no configurator counterpart because the configurator has no in-panel Save button; 11.1 (entry = working copy of persisted) is only implicitly covered. Consider a note in the test file or one explicit 11.1 assertion for parity with the config suite.
3. **Partial-flush leaves the config store half-written** (likely) — `config_panel_persist` commits staged keys in sorted order and, on the first `set_user_value`/`remove_user_value` error, restores the unflushed edits but leaves already-written keys in the store. The store is left partially consistent. The commit message calls this out as intentional; flag it so the owner confirms that partial commit is acceptable rather than all-or-nothing.
4. **Full workspace gate deferred** (confirmed) — Per gate-policy.md the agent ran scoped check/clippy/fmt and scoped tests only (results recorded in the commit messages: 16 + 63 + 38 passing across the targeted filters). The authoritative full `cargo gate` is the owner's manual run before merge and has not been run.

</details>

<details>
<summary>Details</summary>

### Command Configurator whole-store working copy (S.2)

`CommandConfiguratorState` gains `working: Option<Vec<CommandDefinition>>` plus `persist_error`, with `is_dirty` / `working_mut` / `displayed` / `clear_working` helpers. `working_mut` takes the clone from the persisted definitions on first edit, which is the right lazy-snapshot point — a Context that is opened and closed without editing never allocates a buffer and `is_dirty` returns false.

`apply_configurator_action` and `commit_configurator_form` now mutate the working copy only; the two former `command_store.save()` calls (one in Delete, one at the end of the form commit) are removed. Duplicate-id rejection was reworked to check against the persisted store first and then the working copy, which is correct for the add path. Delete is `working.retain(|d| d.id != id)` against the buffer — deferred, as the `configurator_delete_is_deferred_to_save` test proves by asserting both definitions remain on disk until SAVE.

`command_configurator_persist` does `mem::replace` of the store definitions with the working copy, then `command_store.save()`. On `Err` it restores `previous` and keeps the working copy so END stays open and RETURN can still discard — the `configurator_end_on_persist_error_stays_open` test confirms both the error surfacing and the retained working copy. This whole-store replace is genuinely atomic (write succeeds or the in-memory store is rolled back), which is cleaner than the config panel's per-key flush.

### Config/Settings per-key buffer (S.1)

The config panel can't own a whole-store clone the way the configurator does because it reads effective values live from the layered `ConfigHandle` and only overlays user-layer edits. So the working copy is a `HashMap<key, WorkingEdit>` where `WorkingEdit::Set(v)` stages a value and `WorkingEdit::Reset` stages a user-override removal. `working_value` overlays a staged Set for display; a staged Reset deliberately falls through to the persisted value until flush, which matches the comment and the `config_return_discards_staged_reset` test.

`config_panel_persist` takes the buffer, sorts keys for deterministic flush order, and applies each as `set_user_value` / `remove_user_value`. The partial-write behavior on mid-flush failure (issue 3) is the one asymmetry versus the configurator's atomic replace — here the config store has no single-write commit point, so a true all-or-nothing flush would need buffering at the `ConfigHandle` layer, which this change explicitly keeps out of scope.

### Help END key path is already fixed; guards added (S.3)

No dispatch code changed for S.3. I traced the live path: `resolve_function_key_command` (F3 default = END) -> `dispatch_key_command` -> `dispatch_bound_command` -> `resolve_and_dispatch_command`, which on `ResolveOutcome::FallThrough` calls `dispatch_command_string` — the same front door the typed/ladder path uses. The ladder's `try_commands_a` END arm checks `editable_context_kind()`; Help returns None, so END is `nav_end` (pop the Navigation_Stack), not persistence. That is exactly what S.3 requires.

The two new tests are well-targeted: `tests_nav.rs::f3_end_from_help_context_navigates_back_without_error_key_path` drives `dispatch_key_command("END")` (not `handle_command`), and `tests_focus.rs::full_shell_f3_in_help_navigates_back_without_error` presses a physical F3 through the eframe harness. Both assert `open_error.is_none()` to guard against the "command not yet implemented" regression. Each carries a `// Validates:` annotation.

### Seam wiring and framework conformance

`EditableKind::ConfigPanel` and `EditableKind::CommandConfigurator` are added and wired into `editable_context_kind` / `editable_persist` / `editable_discard` / `editable_reload`, replacing the prior TODO fall-through. This follows the Menus/Keys/Kinds pattern already on main, so SAVE/END/RETURN/REFRESH reach both Contexts through the single shared seam with no bespoke intercept. The config panel's Save button pushes `ShellRequest::Command("SAVE")` rather than calling persist directly — correct per framework-conformance (route through the dispatch seam).

</details>

<details>
<summary>File map</summary>

- `command_config/render.rs` — working-copy state + helpers on `CommandConfiguratorState`; table renders from `displayed`; unsaved/persist-error banners.
- `shell/configurator.rs` — add/edit/delete mutate the working copy; former per-action store writes removed.
- `shell/commands_configurator_save.rs` (new) — configurator persist/discard/reload seam ops (atomic store replace + rollback).
- `shell/commands_config_save.rs` (new) — config panel persist/discard/reload seam ops (per-key flush).
- `config_panel/mod.rs` — `WorkingEdit` enum, `working` buffer, stage/clear/take helpers; `WorkspaceContext::render` passes `requests`.
- `config_panel/commit.rs` — `commit_value` stages instead of writing.
- `config_panel/render.rs` — Save button enqueues SAVE; Reset stages; dirty indicator + persist-error banner.
- `config_panel/widgets.rs` — widgets display the working-copy value.
- `shell/editable_context.rs` — `EditableKind::ConfigPanel` + `CommandConfigurator` wired into the seam.
- `shell/mod.rs` — register the two new seam modules + two new test modules.
- `shell/tests_config_save_convention.rs` (new) — 8 tests (Req 11.1-11.5, 11.8).
- `shell/tests_configurator_save_convention.rs` (new) — 7 tests (Req 11.2-11.5).
- `shell/tests_nav.rs`, `shell/tests_focus.rs` — two Help END key-path regression guards.
- `crates/ff-i18n/i18n/en/ff-desktop-command-config.ftl` — unsaved-changes string.
- `docs/project-management/project-master/requirements.md` — Req 11 rows flipped to COMPLIANT.
- `docs/specs/configuration-system/requirements.md` — Cross-Cutting Save Convention section added.

Full diff: `git -C c:/workspace/apps/FileForgeWorkbench/.worktrees/stream-save diff main...HEAD`

</details>
