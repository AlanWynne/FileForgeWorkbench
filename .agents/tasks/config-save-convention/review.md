# Review: config-save-convention (B081, project-master Requirement 11 -- Config/Settings fix)

Verdict: APPROVED

## Scope reviewed
Diff `main...HEAD` on worktree `config-save-convention` (10 files, +575/-23):
- `config_panel/commit.rs`, `config_panel/mod.rs`, `config_panel/render.rs`, `config_panel/widgets.rs`
- `shell/commands_config_save.rs` (new), `shell/editable_context.rs`, `shell/mod.rs`
- `shell/tests_config_save_convention.rs` (new)
- `docs/project-management/project-master/requirements.md`, `docs/specs/configuration-system/requirements.md`

Verification evidence read from the commit message (not re-run, per instructions):
`cargo check -p ff-desktop` OK; `cargo clippy -p ff-desktop --tests -- -D warnings` 0 warnings; `cargo fmt --check` clean; `cargo test -p ff-desktop config_save_convention` 8 passed; `config_panel::` 20 passed (no regression); focus-contract tests passed. Full gate deferred to owner per gate-policy.

## Requirement conformance

Working-copy buffer on entry (11.1): `ConfigPanelState` gains `working: HashMap<String, WorkingEdit>` and `persist_error`. `working_value` overlays staged `Set`/falls through to the persisted effective value, so with no edits the displayed value equals persisted. Test `config_working_value_defaults_to_persisted` confirms. ff-config layered-store semantics untouched -- buffer lives entirely in the panel.

No autosave on a bare edit (11.2): `commit_value` now validates then calls `state.stage_set` (buffer only); the former `config.set_user_value` write is removed. Reset click calls `stage_reset` (buffer only); the former `config.remove_user_value` is removed. Widgets read/edit `working_value`. Verified by `config_bare_edit_does_not_write_store`.

SAVE persists + stays open (11.3): SAVE in `commands_ladder_a.rs` calls `editable_persist()` and does not navigate. `config_panel_persist` takes the buffer and flushes each `Set`/`Reset` via `set_user_value`/`remove_user_value`, empties the buffer on success, clears `persist_error`. Verified by `config_save_command_persists_and_stays_open` (persisted == 7, Context open, not dirty).

END persists then exits; stays open on persist error (11.4): END branch calls `editable_persist()`, on `Ok` navigates (`nav_end`), on `Err` stays put. `config_panel_persist` returns `Err` and sets `persist_error` on the first write failure, restoring the unflushed edits. Verified by `config_end_persists_then_exits` and `config_end_on_persist_error_stays_open` (locks the key so `set_user_value` returns KeyLocked; asserts Context stays open and `persist_error` is set).

RETURN discards, store unchanged (11.5): RETURN branch calls `editable_discard()` -> `config_panel_discard` -> `clear_working` (in-memory only, no store write) then `nav_return`. Verified by `config_return_discards_buffered_edits_and_exits` (store == entry value, Context exited) and `config_return_discards_staged_reset` (a staged Reset is dropped; persisted user value survives).

Save affordance shares the SAVE command path (11.8): the in-panel Save button is enabled only when dirty and pushes `ShellRequest::Command("SAVE")`. `render.rs` drains requests via `handle_command(&cmd)`, i.e. the exact typed-SAVE dispatch seam -- no bespoke intercept, no divergent save path. Test `config_save_affordance_matches_save_command_path` exercises the shared `editable_persist` path. Confirmed `ShellServices.requests` field exists and `services.config` is `&self.config_handle` (the same handle persist writes to), so persisted values display correctly and hot-reload/provenance keep reading live from the store.

Seam wiring: `EditableKind::ConfigPanel` added; `editable_context_kind` maps `KindTag::ConfigPanel`; persist/discard/reload dispatch to `config_panel_persist`/`_discard`/`_reload`. `config_panel_reload` re-snapshots by clearing the buffer (effective values read live). New `commands_config_save.rs` follows the established split pattern mirroring the Theme/Menus/Keys/Kinds seams.

## Scope guard
- ff-config crate: no files in the diff -- store semantics unchanged.
- Sibling editors (Alias/Theme/Menus/Keys/Kinds) and Command Configurator: no files touched -- byte-identical.
- Dispatch: via the single `editable_*` seam; Save button routes through `handle_command("SAVE")`. No parallel dispatcher.
- 400-line rule: `config_panel/mod.rs` (~182 lines) and `render.rs` (non-test ~296) within limit; new files small. OK.
- ASCII-only: Select-String non-ASCII scan of the new/changed source returned no matches (box/glyphs are Rust `\u{...}` escapes = ASCII bytes).
- Master Req 11 Config/Settings conformance row updated to COMPLIANT; configuration-system requirements.md gains the cross-cutting save-convention section.

## Tests and annotations
All required cases present and annotated `// Validates: project-master Requirement 11.x`: 11.1, 11.2, 11.3, 11.4 (incl. persist-error), 11.5 (incl. staged-Reset discard), 11.8.

## Non-blocking observations (not findings)
- On a mid-flush persist failure, `config_panel_persist` restores the FULL taken buffer (including already-flushed keys). Re-flushing an already-set value is idempotent and the partial-consistency behaviour is documented; not a correctness blocker.
- `config_save_affordance_matches_save_command_path` asserts via `editable_persist()` directly rather than driving the enqueued SAVE command; the shared-path claim is nonetheless substantiated by the render enqueue code plus the typed-SAVE test hitting the same persist. Acceptable.

No blockers found.
