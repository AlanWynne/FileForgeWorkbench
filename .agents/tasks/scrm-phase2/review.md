# Review -- SCRM Phase 2 surfacing (feat/scrm-phase2-surfacing)

Verdict: APPROVED

## Scope reviewed
`git diff main...HEAD` on the worktree. Changed files:
- `crates/ff-desktop/src/menu_workspace/defaults.rs` (new `DEFAULT_MENU_BAR_TOML`, `default_menubar_menu()` reparse, tests)
- `crates/ff-desktop/src/shell/render_chrome.rs` (`dynamic_menu_options` adds `SNAPSHOT`; new `scrm_menu_bar_options()`)
- `crates/ff-desktop/src/shell/tests_menu_workspace.rs` (3 new tests)
- `docs/specs/screen-snapshot-scrm/requirements.md` (new Req 21)
- `docs/specs/screen-snapshot-scrm/tasks.md` (Wave S / task 14)
- `docs/project-management/project-master/tasks.md` (SCRM.8)

## Evidence checked against the task

### 1. Menu-bar entry, not a POM option, not typed-only -- PASS
`DEFAULT_MENU_BAR_TOML` adds a top-level `S` / `SNAPSHOT` option to the compiled
default Menu_Bar. `default_menubar_menu()` now parses the new constant instead of
returning `recovery_pom_menu()`, so the bar is decoupled from the POM. The entry
is rendered on the bar (confirmed: `render_menu_bar_into_ui` draws each
`show_in_menu_bar` option, and the full-shell test locates the live `SNAPSHOT`
button via `get_by_label`).

### 2. Command value is the EXISTING SCRM verb; routes through the single path -- PASS
`scrm_menu_bar_options()` yields children whose `command` is an existing verb
(`SNAPSHOT TEXT/ANSI/MARKDOWN/HTML/YAML`, `CAPTURE START/STOP/SCREEN/STATUS/LIST/
PURGE/REPLAY`). `render_menu_bar_into_ui` dispatches both the dynamic children and
any direct top-level command through `self.dispatch_command_string(...)` -- the
single front door (`shell/dispatch.rs`), which runs prelude -> `resolve_target`
-> ladder. These verbs are the pre-existing ones handled by
`commands_ladder_b.rs` -> `handle_snapshot` / `handle_capture`
(`commands_scrm.rs`). A clicked verb therefore follows the identical path as a
typed verb.

Note on the FORWARD-PLAN phrasing "resolve_target -> dispatch_command_target":
the SNAPSHOT/CAPTURE verbs are not yet migrated to `CommandTarget`, so
`resolve_target` returns `Err` for them and they fall through to the ladder.
This is the pre-existing dispatch state of SCRM (not changed by this diff); the
important invariant -- click and typed line share ONE front door
(`dispatch_command_string`) -- holds. No new dispatcher is introduced.

### 3. Content is code-only built-in, no new verb / intercept / dispatcher / nav stack -- PASS
- Menu content lives in the code-only `DEFAULT_MENU_BAR_TOML` constant
  (menu_workspace/defaults.rs), never written to disk (CR-CH-021). Conforms to
  framework-conformance mechanism 4.
- The `SNAPSHOT` dropdown reuses the EXISTING dynamic-source mechanism
  (`dynamic_menu_options`), the same pattern as `THEME LIST`. No bespoke
  `if upper == "..."` intercept was added; the one `if trimmed.eq_ignore_ascii_
  case("SNAPSHOT")` is a dynamic-source classifier inside the existing helper,
  not a dispatch intercept.
- No new verb (all verbs pre-exist in commands_ladder_b.rs). No second
  dispatcher, no second navigation stack. Conforms to framework-conformance
  mechanisms 1-3 and wiring-standard (reuses the single dispatch seam; UI
  affordance invokes the command rather than a handler directly).

### 4. Bare-core POM stays EXACTLY 0 Settings / 3 Help / X (CR-CH-073) -- PASS
`DEFAULT_POM_TOML` is NOT in the diff (confirmed by reading the source: still
exactly `0` Settings / `3` Help / `X`). The new `DEFAULT_MENU_BAR_TOML` is a
separate constant. Test `scrm_surfacing_does_not_expand_the_pom` asserts the POM
has 3 options, no SNAPSHOT, and `DEFAULT_POM_TOML` does not mention SNAPSHOT. The
pre-existing `menu_bar_omits_catalogs_and_files` test remains valid (bar is now
Settings/Help/SNAPSHOT, still no catalogs/files).

### 5. Tests + TDD annotations -- PASS
- `defaults.rs`: `default_menubar_differs_from_barebones_pom_by_scrm_entry`,
  `default_menubar_bar_visible_options_are_settings_help_snapshot`,
  `scrm_surfacing_does_not_expand_the_pom`, `default_menubar_has_scrm_snapshot_entry`,
  `default_menu_bar_toml_is_valid_ascii_toml` -- each carries a `// Validates:`
  annotation.
- `tests_menu_workspace.rs`: `menu_bar_dynamic_scrm_source_lists_existing_verbs`
  (verb set), `scrm_menu_bar_item_dispatch_is_command_parity` (bar seam ==
  typed), and `full_shell_scrm_menu_bar_entry_is_live_and_dispatches` (full-shell
  egui_kittest: asserts the live `SNAPSHOT` bar button is rendered + front-door
  dispatch parity). Each carries a `// Validates:` / doc annotation.
- The full-shell test documents that egui nested-menu popups are not reliably
  walkable in egui_kittest and therefore asserts activation at the exact dispatch
  seam the render arm invokes (`dispatch_command_string`). This is an honest,
  acceptable "closest harness-expressible" proof and is permitted by the task
  note and testing.md's manual/limitation exception; the rendered-button presence
  is still asserted via `get_by_label`. The parity assertion is meaningful:
  `CAPTURE START` -> `scrm.start(...)` sets `scrm.is_active()`.

### 6. Deferred add-ons NOT implemented -- PASS
No digital signature, protected-PDF evidence, DIDL export, conditional-capture-
rule wiring, replay filter UI, or off-frame async write appears in the diff. The
diff only surfaces the existing verbs.

### 7. Bookkeeping -- PASS
- Spec `requirements.md`: Req 21 (21.1-21.5) added as the gate acceptance
  criteria.
- Spec `tasks.md`: Wave S / task 14 added and marked complete with sub-tasks.
- `project-master/tasks.md`: SCRM.8 added (DONE) closing the SCRM.7 surfacing
  bookkeeping for Req 21. (SCRM.7 itself remains `[ ]` IN PROGRESS for the
  separate decomposition reduction work -- correctly not claimed done by this
  change.)

## Verification evidence (read from commit message, not re-run)
Scoped, proxy result files, returncode 0:
- `cargo check -p ff-desktop`: clean
- `cargo clippy -p ff-desktop -- -D warnings`: clean
- `cargo fmt -p ff-desktop -- --check`: clean
- `cargo test -p ff-desktop --bin ffwb scrm_`: 17 passed, 0 failed
- `cargo test -p ff-desktop --bin ffwb menu_bar`: 18 passed, 0 failed
- `cargo test -p ff-desktop --bin ffwb defaults::tests`: 35 passed, 0 failed
- `full_shell_tab_reaches_settings_as_first_menu_item`: 1 passed

This matches the gate-policy agent scope (scoped compile/clippy/fmt + quick
scoped tests); the authoritative full gate is the owner's manual run before merge.

## Findings
No blockers.

ASCII-clean, additive, builds on the framework, bare-core POM invariant
preserved, deferred add-ons excluded, tests present with annotations, bookkeeping
closed.
