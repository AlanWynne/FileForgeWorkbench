# CR-CH-071 Implementation Plan -- ISPF Save/End/Cancel/Return/Refresh for editable Contexts

TDD-first implementation plan for CR-CH-071 (command-environments Requirement
20a; command-configurator Req 5.9 / 5.14 amended). The design gate is APPROVED
(all five recommended decisions confirmed). This plan is grounded in the live
source read in the worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crch071-save-cancel-refresh`
(branch `crch071-save-cancel-refresh`). Pass `-C <worktree>` on every git call.

All paths below are RELATIVE to the worktree root
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crch071-save-cancel-refresh`
unless written absolute.

## Design decisions recorded (grounded in code read, not re-opened)

These were settled in the gate; recorded here with the one-line rationale and the
exact code site each lands at, so the implementer does not re-decide.

- D1. The one new abstraction is a focused-editable-Context SEAM with three
  operations: `persist() -> Result<(), String>`, `discard()`, `reload()`. It is
  realised NOT as a free-standing `dyn`-object trait but as a small dispatch
  helper on `WorkbenchShell` keyed by the focused tab's `KindTag`, mirroring the
  existing `apply_<editor>_action` / `pending_action` pattern (the config editors
  are render-entangled panels on `WorkbenchShell`, not standalone objects -- same
  reason FFEDIT is a thin object over shell state). Rationale: design.md "the seam
  is realised the SAME way FFEDIT is ... a small dispatch helper on the shell";
  avoids reshaping any public framework type (framework-conformance).
- D2. "Is the focused Context an editable NON-buffer Context?" is decided by
  matching `self.tabs.active_tab().kind.tag()` against the set
  `{ AliasEditor, ThemeEditor, MenusEditor, KeysEditor, KindsEditor, ConfigPanel }`.
  Verified: each of these `KindTag`s maps via `BuiltinKind::from_tab_kind`
  (`crates/ff-desktop/src/workspace_kind/mod.rs:152-181`) to a NON-`Editor`
  BuiltinKind, so `active_name` returns `FFCMD`, never `FFEDIT`. The editor buffer
  kinds (`FileEditor`/`Untitled` -> `BuiltinKind::Editor` -> `FFEDIT`) are
  therefore NEVER in this set, which is exactly the active-wins boundary 20a.7
  needs. v1 WIRES only `AliasEditor`; the other five are the fast-follow set (see
  Task 9 decision).
- D3. The FFCMD-base handler lives in the ladder segment `try_commands_a`
  (`crates/ff-desktop/src/shell/commands_ladder_a.rs`), where END / RETURN / X
  already live, reached AFTER the active-env (FFEDIT) claim in `run_command_ladder`
  (`crates/ff-desktop/src/shell/commands.rs:150-205`). Active-wins is already the
  override -- no new machinery (20a.7). SAVE / CANCEL / REFRESH get new arms; the
  EXISTING `END` and `RETURN` arms are AMENDED to consult the seam on an editable
  Context, falling back to today's plain `nav_end` / `nav_return` otherwise
  (20a.1, 20a.6, 20a.14).
- D4. SAVE / CANCEL / REFRESH register as FFCMD roots by appending to
  `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (`crates/ff-desktop/src/command_config/mod.rs:218`).
  Verified this list is unioned into `builtin_ffcmd_roots()`
  (`crates/ff-desktop/src/shell/alias_reader.rs:151-188`), which feeds
  `EnvironmentRegistry::root_commands("FFCMD")` / `contains_root`
  (`crates/ff-desktop/src/shell/environment_registry.rs:300-345`). END / RETURN
  are already in the list and UNCHANGED (20a.5).
- D5. The no-shadow consequence (20a.12) needs NO new logic: both the save-side
  `validate_user_alias_rows` (`crates/ff-desktop/src/shell/alias_editor.rs`, the
  `(d)` reserved-root check) and the load-side `apply_user_rows`
  (`crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs`) already
  consult `builtin_ffcmd_roots()`. Adding the three verbs to the root source
  (D4) makes a user alias named SAVE / CANCEL / REFRESH rejected-at-save and
  skipped-with-WARN-at-load automatically. Only confirming tests are required.
- D6. The Alias Editor seam maps: `persist` = the EXISTING
  `WorkbenchShell::save_user_aliases` (unchanged); `discard` = clear
  `self.alias_editor_panel.rows` (and clear `error`) so unsaved edits cannot
  linger; `reload` = re-run the open-time `load_user_alias_rows()` +
  `self.alias_editor_panel.load_rows(rows)` (20a.2 / 20a.3 / 20a.4). Note
  `load_user_alias_rows` is currently a private `fn` on the `impl` in
  `alias_editor.rs`; it stays usable from the seam because the seam is a method on
  the same shell impl -- no visibility change needed.
- D7. Retiring the private interpreter (20a.10): in
  `crates/ff-desktop/src/shell/alias_editor_line_commands.rs::apply_alias_command_line`,
  the `"SAVE"` and `"CANCEL" | "END"` match arms are REMOVED. A non-FILTER /
  non-RESET primary verb is handed to the shared front door
  `self.dispatch_command_string(&text)` so the typed `Command ===>` line reaches
  the SAME FFCMD-base handler F3 / a typed END reaches. FILTER / RESET stay
  panel-local; the `apply_pending_line_commands()` pass still runs FIRST (Req 5.22).
- D8. REFRESH (not UNDO) is the context-revert verb (20a.9) -- a naming decision
  already baked into D4 (the root literal is `"REFRESH"`). No UNDO code is touched.
- D9. EXIT / QUIT / LOGOFF are UNCHANGED (20a.13): the `try_exit_family` arm
  (`commands_ladder_a.rs:32-56`) is NOT modified. END (and F3) is the
  save-then-leave verb, not EXIT.

## File-size / standards watch (rust-standards.md 400-line rule)

- `commands_ladder_a.rs` is ~300 lines today; adding the SAVE/CANCEL/REFRESH arms
  + the editable-Context dispatch helper could approach 400. DECISION: put the
  seam dispatch helper + the per-kind routing in a NEW focused module
  `crates/ff-desktop/src/shell/editable_context.rs` (the `persist`/`discard`/
  `reload` seam resolution keyed by `KindTag`), and keep ONLY the thin verb arms
  (which call the helper) in `commands_ladder_a.rs`. If `commands_ladder_a.rs`
  still crosses 400 after the arms, split the whole SAVE/CANCEL/REFRESH/END/RETURN
  editable-verb block into `editable_context.rs` as a `try_editable_context_verb`
  helper the ladder calls, leaving a one-line call site. Re-check line counts
  after Task 4 and split if needed (this split is a refactor, no gate).
- Keep each new `#[cfg(test)] mod tests` under ~200 lines; if the Req-20a tests
  exceed that, place them in a dedicated `editable_context` test module /
  `tests_*` file rather than bloating one file.
- No `unwrap`/`expect` in non-test code; derive `Debug` on any new public type;
  plain ASCII source + `// === ... ===` separators.

## Verification commands (scoped, per gate-policy.md + testing.md split)

The AGENT verifies by COMPILE + scoped tests only; the owner runs the full gate.
Run each via the clean pwsh7 wrapper, redirect to a log, and READ the log back
(terminal-discipline.md -- do not trust the Exit Code line):

- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo check -p ff-desktop *> tools\logs\crch071-check.log"`
- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo clippy -p ff-desktop -- -D warnings *> tools\logs\crch071-clippy.log"`
- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo fmt -p ff-desktop -- --check *> tools\logs\crch071-fmt.log"`
- Scoped tests (quick, per touched area):
  `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo test -p ff-desktop <test_name_or_module> *> tools\logs\crch071-test.log"`

Run `cargo` from the worktree root (`cwd` = the worktree). `tools\logs\` is
git-ignored -- NEVER stage it.

---

# Implementation Plan

- [ ] 1. Write the FAILING root-enumeration tests (TDD red) for SAVE/CANCEL/REFRESH as FFCMD roots.
      Add tests asserting `EnvironmentRegistry::contains_root("FFCMD", v)` is TRUE for each of
      `SAVE`, `CANCEL`, `REFRESH`, and that `root_commands("FFCMD")` contains all three. Place them
      in the existing registry test module (co-locate with the CR-CH-067 enumeration tests).
      Annotate `// Validates: command-environments Requirement 20a.5`.
      Files: crates/ff-desktop/src/shell/environment_registry.rs (test module), or a sibling
      `tests_*` module if the inline test block nears 200 lines.
      Verify: `cargo test -p ff-desktop contains_root` -- the three new assertions FAIL (red),
      proving they are not yet roots.

- [ ] 2. Register SAVE/CANCEL/REFRESH as FFCMD canonical roots (make Task 1 green).
      Append `"SAVE"`, `"CANCEL"`, `"REFRESH"` to `BUILTIN_FFCMD_LADDER_VERB_ROOTS` with an
      inline comment tagging them CR-CH-071 editable-Context verbs. Confirm the existing drift-guard
      test `builtin_ffcmd_ladder_verb_roots_are_disjoint_from_classifier_and_carry_file_open`
      still passes (the three are NOT classifier verbs, so disjointness holds).
      Files: crates/ff-desktop/src/command_config/mod.rs
      Verify: `cargo test -p ff-desktop -- command_config environment_registry` -- Task 1 tests and
      the drift guard all pass; then `cargo check -p ff-desktop` clean.

- [ ] 3. Create the focused-editable-Context SEAM module + the per-kind resolution (TDD).
      Write FIRST a failing unit test that, with the active tab driven to the Alias Editor and
      `alias_editor_panel.rows` holding one pending edit, calls the new seam ops and asserts:
      `discard()` leaves zero pending rows; `reload()` repopulates rows from the last-saved
      `user.toml` (use a `dir_overrides.user_aliases` temp path as existing alias tests do).
      Then implement a new module `crates/ff-desktop/src/shell/editable_context.rs` exposing
      `pub(super)` shell methods: `editable_context_kind() -> Option<EditableKind>` (returns Some
      only for the 6 editable-non-buffer KindTags; v1 only AliasEditor is wired, others return a
      `NotWiredYet` marker or `None` per Task 9 decision), and `editable_persist()/_discard()/_reload()`
      that route by the active `KindTag` to the owning Context's methods. Declare the module in
      `shell/mod.rs`. Annotate tests `// Validates: command-environments Requirement 20a.6, 20a.11`.
      Files: crates/ff-desktop/src/shell/editable_context.rs (new), crates/ff-desktop/src/shell/mod.rs
      Verify: `cargo test -p ff-desktop editable_context` -- the discard/reload tests pass;
      `cargo check -p ff-desktop` and `cargo clippy -p ff-desktop -- -D warnings` clean.

- [ ] 4. Wire the Alias Editor into the seam (persist/discard/reload) (TDD).
      Write FIRST failing tests that the Alias Editor seam: `persist` writes `user.toml` via
      `save_user_aliases` and surfaces a validation error without writing on a bad row; `discard`
      clears `rows` and `error`; `reload` re-reads `user.toml`. Then implement the AliasEditor arm
      of the Task 3 helper: persist -> `self.save_user_aliases()`; discard -> clear
      `self.alias_editor_panel.rows` + set `error = None`; reload -> `let rows = self.load_user_alias_rows();
      self.alias_editor_panel.load_rows(rows);`. (persist must return the Result-shaped outcome the
      handler needs for END's save-then-leave-vs-stay decision -- surface the existing
      `alias_editor_panel.error` as the Err signal, or refactor `save_user_aliases` to return
      `Result<(), String>` and keep the error-field set at the call site; prefer the latter so END
      can branch on ok/err without reading panel state.)
      Files: crates/ff-desktop/src/shell/editable_context.rs, crates/ff-desktop/src/shell/alias_editor.rs
      Verify: `cargo test -p ff-desktop -- editable_context alias_editor` -- all green;
      `cargo check -p ff-desktop` clean.

- [ ] 5. Add the FFCMD-base editable-Context verb handling in the ladder (TDD, the behaviour core).
      Write FIRST failing tests (drive the active tab to the Alias Editor via `open_alias_editor`,
      set a temp `dir_overrides.user_aliases`, then call `handle_command`/`dispatch_command_string`
      with each verb) asserting the full matrix:
        SAVE    -> persists + STAYS (tab kind unchanged, file written);
        REFRESH -> discards + reloads + STAYS (pending edits gone, rows == last-saved);
        CANCEL  -> discards + leaves via `nav_end` (no write);
        RETURN  -> discards + leaves via `nav_return` (no write);
        END     -> persists then `nav_end` on success; on a validation failure STAYS with an error
                   and the store is UNCHANGED;
        and that on a NON-editable Context (e.g. POM) SAVE/REFRESH are no-ops and END/RETURN are
        plain navigation (20a.14).
      Then amend `try_commands_a` (`commands_ladder_a.rs`): add `SAVE`/`CANCEL`/`REFRESH` arms and
      amend the existing `END`/`RETURN` arms to first check `editable_context_kind()`; if Some(wired),
      perform the seam sequence (SAVE=persist+stay; REFRESH=discard+reload+stay; CANCEL=discard+nav_end;
      RETURN=discard+nav_return; END=persist then nav_end on ok / stay+surface error on err); else fall
      through to today's behaviour (SAVE/REFRESH unresolved/no-op; END/RETURN plain nav). Keep the arms
      thin -- the logic lives in the Task 3 helper. (SAVE/REFRESH must NOT be claimed when the active
      env is FFEDIT: this is already guaranteed because the FFEDIT claim runs BEFORE `try_commands_a`
      in `run_command_ladder`, and an editor buffer's KindTag is not in the editable set.)
      Annotate `// Validates: command-environments Requirement 20a.1, 20a.2, 20a.3, 20a.4, 20a.6, 20a.14`.
      Files: crates/ff-desktop/src/shell/commands_ladder_a.rs (and editable_context.rs if the block is
      moved there per the file-size watch).
      Verify: `cargo test -p ff-desktop -- editable_context commands_ladder` -- the full matrix passes;
      `cargo check` + `cargo clippy -p ff-desktop -- -D warnings` clean. COMMIT POINT A.

- [ ] 6. Confirm the FFEDIT buffer SAVE active-wins override is byte-identical (TDD regression guard).
      Write a test that, with an editor-buffer tab focused (FileEditor/Untitled kind), a typed `SAVE`
      is claimed by FFEDIT FIRST (routes through `dispatch_to_environment` store-addressing) and does
      NOT reach the FFCMD editable-Context handler; and that REFRESH on an editor buffer is a no-op in
      v1 (FFEDIT does not claim it; the FFCMD base treats a buffer as non-editable). Reuse/extend the
      existing FFEDIT claim tests. No production change expected -- this is the guard that the new
      FFCMD roots did NOT shadow the buffer SAVE.
      Annotate `// Validates: command-environments Requirement 20a.7, 20a.8`.
      Files: crates/ff-desktop/src/shell/environment_ffedit.rs (tests) or the existing FFEDIT claim
      test module.
      Verify: `cargo test -p ff-desktop -- ffedit save` -- green (and the pre-existing
      `save_is_classified_store_affecting` / `in_buffer_verbs_are_not_store_affecting` still pass).

- [ ] 7. Retire the Alias Editor private SAVE/CANCEL/END interpreter -- one dispatch path (TDD).
      Write FIRST a failing test that a typed `Command ===>` END on the Alias Editor now SAVES-then-leaves
      (not a bare `nav_end`), CANCEL discards-then-leaves, and REFRESH discards+reloads+stays -- i.e.
      the typed line and F3/global END produce the SAME result. Then in
      `apply_alias_command_line` (`alias_editor_line_commands.rs`): REMOVE the `"SAVE"` and
      `"CANCEL" | "END"` arms; after the FILTER/RESET arms, change the fallthrough so a non-empty,
      non-FILTER/RESET primary verb is handed to `self.dispatch_command_string(&<original text>)`
      (preserving the uppercased-verb routing but passing the real command string). Keep the
      `apply_pending_line_commands()` first-pass and the FILTER/RESET arms intact (Req 5.15, 5.22).
      Update the existing `command_line_applies_line_commands_before_primary_command` test if its
      SAVE assumption changes (the I-then-SAVE ordering must still hold via the new path).
      Annotate `// Validates: command-environments Requirement 20a.10, command-configurator Req 5.9, 5.14`.
      Files: crates/ff-desktop/src/shell/alias_editor_line_commands.rs
      Verify: `cargo test -p ff-desktop -- alias_editor_line_commands editable_context` -- green;
      confirm no test still asserts the old END=bare-nav behaviour. COMMIT POINT B.

- [ ] 8. Add the no-shadow confirmation tests for SAVE/CANCEL/REFRESH-named user aliases (TDD).
      Write tests asserting: `validate_user_alias_rows` REJECTS a row whose surface is `SAVE`/`CANCEL`/
      `REFRESH` (save-side, reusing the `(d)` reserved-root path); and `apply_user_rows` SKIPS such a
      row with a WARN at load (load-side). No production change is expected (both already read
      `builtin_ffcmd_roots()`, now including the three verbs via Task 2) -- these tests lock the
      20a.12 consequence in place.
      Annotate `// Validates: command-environments Requirement 20a.12`.
      Files: crates/ff-desktop/src/shell/alias_editor.rs (validate tests),
      crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs (load tests).
      Verify: `cargo test -p ff-desktop -- validate_rejects no_shadow user_overlay` -- green.

- [ ] 9. Decide and record the sibling-editor (Settings/Menus/Theme/Keys/Kinds) path.
      DECISION for v1: do NOT wire the five siblings in v1. Rationale grounded in code read: each
      sibling has a DIFFERENT working-copy shape and save entry point -- Menus uses
      `apply_menus_editor_action` + `save_working_menu` with a `working: Option<MenuFile>` and a
      `selected` name; Keys uses `apply_keys_editor_action` + `save_keys_for_kind`; Theme uses
      `apply_theme_editor_action`; Kinds uses `apply_kinds_editor_action`; Config/Settings is the
      `ConfigPanel` with its own model. Wiring all five means five distinct discard/reload shapes and
      five first-Tab-safe reload paths -- real scope bloat that risks the v1 Alias Editor deliverable.
      So the Task 3 helper returns `None` (non-editable fallthrough) for the five sibling KindTags in
      v1, with a clearly-marked `// TODO(CR-CH-071 fast-follow, tasks 34.1-34.5)` per kind, and the
      seam is SHAPED so each sibling plugs in by adding its arm (persist = its existing save action;
      discard/reload = drop + re-open its working copy). Leave the follow-up tasks already captured in
      `docs/specs/command-environments/tasks.md` Tasks 34.1-34.5 untouched (they remain `[ ]`).
      Files: crates/ff-desktop/src/shell/editable_context.rs (TODO markers only).
      Verify: `cargo check -p ff-desktop` clean; confirm the five siblings still leave via plain
      `nav_end`/`nav_return` exactly as today (a quick test that END on the Menus Editor behaves
      unchanged guards against accidental regression).

- [ ] 10. Flip the CR-CH-071 TCR rows for every criterion now covered.
      In `docs/quality/TCR.md`, locate the `Phase (alias-editor-save-semantics)` section (the
      NOT-COVERED red rows added by the gate) and flip the rows whose tests now exist to the project's
      PASS/covered marker: 20a.1-20a.6 (Tasks 3,4,5), 20a.7-20a.8 (Task 6), 20a.9 (Task 5 REFRESH
      naming + design), 20a.10 (Task 7) + command-configurator 5.9 / 5.14 (Task 7), 20a.12 (Task 8),
      20a.13 (add/confirm an EXIT-unchanged test in Task 5's matrix), 20a.14 (Task 5 non-editable
      arm). Leave 20a.11 PARTIAL (Alias Editor done; siblings are fast-follow Tasks 34.x) with a note.
      Use the AGENT verification wording the repo's TCR convention uses (compile/clippy/fmt clean +
      scoped Req-20a tests green; full gate = owner's manual run), matching the CR-CH-070 TCR style.
      Files: docs/quality/TCR.md
      Verify: ASCII-only check on the edited file:
      `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "Select-String -Path docs\quality\TCR.md -Pattern '[^\x00-\x7F\u2500-\u257F]' *> tools\logs\crch071-tcr-ascii.log"`
      then READ the log -- only pre-existing allowed emoji in unrelated rows may match.

- [ ] 11. Update the tasks.md checkboxes to reflect completed work.
      In `docs/specs/command-environments/tasks.md`, mark Tasks 32.1-32.4, 33.1-33.3, 35.1-35.7 `[x]`
      as each lands (leave 34.1-34.5 `[ ]` fast-follow; 35.8 is the owner's full-gate run). Keep
      `[ ]`/`[x]` only, titles intact (task-format.md).
      Files: docs/specs/command-environments/tasks.md
      Verify: visual read-back that only `[ ]`/`[x]` markers appear and titles are intact.

- [ ] 12. Final scoped verification pass + record a summary.
      Run, reading each log back: `cargo fmt -p ff-desktop -- --check`, `cargo check -p ff-desktop`,
      `cargo clippy -p ff-desktop -- -D warnings`, and the full scoped Req-20a test set
      (`cargo test -p ff-desktop -- editable_context commands_ladder alias_editor environment_registry
      ffedit`). Re-check `commands_ladder_a.rs` and `editable_context.rs` line counts (< 400 non-test;
      split if needed per the file-size watch). Record a concise summary in
      `.agents/tasks/alias-editor-save-semantics/` (criteria implemented, files changed, commands run +
      results read from logs, the sibling-editor follow-up, and that the authoritative full `cargo gate`
      is the owner's manual run).
      Files: (verification only; summary note under .agents/tasks/alias-editor-save-semantics/)
      Verify: all four logs read clean (fmt ok, check ok, clippy 0 warnings, tests pass). COMMIT POINT C.

## Commit points (stage files BY NAME; never `git add -A`; never stage tools/logs)

- COMMIT POINT A (after Task 5): the FFCMD root registration + seam + Alias Editor wiring + ladder
  handler land together as one buildable, test-green unit. Stage exactly:
  `command_config/mod.rs`, `shell/editable_context.rs`, `shell/mod.rs`, `shell/alias_editor.rs`,
  `shell/commands_ladder_a.rs`, `shell/environment_registry.rs` (tests). Example:
  `git -C <worktree> add crates/ff-desktop/src/command_config/mod.rs crates/ff-desktop/src/shell/editable_context.rs crates/ff-desktop/src/shell/mod.rs crates/ff-desktop/src/shell/alias_editor.rs crates/ff-desktop/src/shell/commands_ladder_a.rs crates/ff-desktop/src/shell/environment_registry.rs`
  then `git -C <worktree> commit -m "feat(cmdenv): FFCMD Save/Cancel/Refresh editable-Context seam + Alias Editor (CR-CH-071 Req 20a.1-20a.8)"`.
- COMMIT POINT B (after Task 7): retire the private interpreter. Stage
  `shell/alias_editor_line_commands.rs` (+ any FFEDIT guard test file touched in Task 6).
  `git -C <worktree> commit -m "refactor(alias-editor): retire private SAVE/CANCEL/END interpreter -- one dispatch path (CR-CH-071 Req 20a.10)"`.
- COMMIT POINT C (after Task 12): docs + no-shadow tests + final verification. Stage
  `shell/alias_reader/alias_user_overlay.rs` (tests), `docs/quality/TCR.md`,
  `docs/specs/command-environments/tasks.md`.
  `git -C <worktree> commit -m "docs(cmdenv): flip CR-CH-071 TCR rows + tasks; no-shadow SAVE/CANCEL/REFRESH tests (Req 20a.12)"`.

Commit only when each unit is compile + scoped-test green (gate-policy.md). Do NOT push; do NOT run
the full `cargo gate` (the owner's manual run is authoritative). Never stage `tools/logs/` (git-ignored).

## Gaps / assumptions noted

- `save_user_aliases` currently returns `()` and sets `alias_editor_panel.error` on failure. Task 4
  prefers refactoring it to return `Result<(), String>` (keeping the error-field set at the call
  site) so END can branch save-then-leave vs stay-with-error without reading panel state. If that
  refactor is deemed too broad, the fallback is: after calling `persist()`, read
  `alias_editor_panel.error.is_none()` to decide END's leave-vs-stay. Either satisfies 20a.1/20a.2;
  the Result refactor is cleaner and is the recommended path.
- The seam's `EditableKind`/`NotWiredYet` marker (Task 3) is an internal enum; derive `Debug`. It is
  NOT a public framework type and must not leak into any public API (framework-conformance).
- `dispatch_command_string` is `pub(super)`; `apply_alias_command_line` is on the same shell impl, so
  the Task 7 reroute call compiles without a visibility change. Confirmed by read.
