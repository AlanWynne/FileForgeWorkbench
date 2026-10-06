# CR-CH-052 Implementation Plan -- Uniform navigation / exit model (two roots)

Branch: `decomp-and-scrm-wave` (main checkout, NO worktree). Gate APPROVED +
COMMITTED (58cac12). This is the IMPLEMENTATION. All anchors below were
re-grepped against the live tree this session; the files drift, so each edit
item re-states the enclosing function and the exact match string to find rather
than trusting a bare line number.

## Design decisions (confirmed against the committed gate, not re-decided)

The design is fixed by the committed specs; this plan only sequences it. The one
decision that needed grounding in the code (not spelled out verbatim in the
spec) is HOW the Tab_Visual_Root is reconstructed, since no tab field stores it:

- **Tab_Visual_Root = the BOTTOM of the active tab's `nav_stack`**, i.e.
  `nav_stack.first()`. When the stack is EMPTY the tab is already AT its visual
  root (its current Context is the root). Verified from the code:
  - `nav_to_kind` / `navigate_to(push=true)` push the previous Context's
    descriptor, so an in-place navigation leaves the ORIGIN descriptor as
    `nav_stack[0]` (the bottom). For a POM->CONFIG navigation `nav_stack == [Menu{pom}]`.
  - `start_new_workspace` for `START <ctx>` navigates then CLEARS the stack
    (`nav_stack.rs` `start_new_workspace`), so a `START SETTINGS` tab has an EMPTY
    stack and its current Settings Context IS its visual root -> bare `X` closes
    it immediately (matches menu-workspace Req 14 glossary + design delta).
  - Therefore `nav_collapse_to_visual_root()` = IF `nav_stack` non-empty:
    reconstruct `nav_stack[0]` in place, then `nav_stack.clear()`. IF empty: it is
    a no-op (already at root); callers only invoke it on the non-empty branch.
- **`=` reinitialise-to-POM = clear stack + reconstruct the POM** (FFCMD_Root),
  NOT the per-tab root. This is the global POM via `set_active_tab_home()` (which
  already clears `menu_workspace` and re-seeds pom.toml / barebones). Confirmed in
  `nav_reconstruct.rs` `set_active_tab_home`.
- `close_workspace_or_exit()` (`nav_stack.rs`) already encodes "exit only when
  `tabs.len() <= 1`"; reuse unchanged for X-at-root / `=X` / END-at-empty /
  RETURN-at-root. No new exit mechanism.
- `X` stays a ladder/prelude verb (like EXIT/END/RETURN), NOT a registered
  Command_ID (wiring-standard Known Caveat).

### Current-behaviour facts the plan relies on (re-verified)
- Front door `dispatch_command_string` (`dispatch.rs`): `run_command_prelude` ->
  `resolve_target` -> `run_command_ladder`. `handle_command` (`commands.rs`) runs
  prelude + ladder but NOT `resolve_target`.
- `try_exit_family` (`commands_ladder_a.rs`) matches
  `EXIT|QUIT|=X|X|LOGOFF` -> `file.exit`. The EXIT-family gate in
  `run_command_prelude` (`commands.rs`) is
  `if (!editor_env_active || is_equals_prefixed) && self.try_exit_family(upper)`.
- `resolve_pom_option_key` and `try_chained_fastpath` (`commands_fastpath.rs`)
  both strip a leading `=` themselves today; the chained fastpath also does the
  origin `insert_pom_tab` pop. These become consumers of an already-stripped
  remainder.
- The three nav callers that bypass the front door call `self.handle_command`:
  `commands.rs` POM option-key recursion (`self.handle_command(&pom_command)`);
  `commands_fastpath.rs` chained loop (`self.handle_command(segment)`);
  `nav_stack.rs apply_start_command` (`self.handle_command(&resolved)`).
- `nav_return` TODAY targets the POM (CR-CH-038), despite the stale "collapse to
  ROOT" comment on the RETURN arm in `commands_ladder_a.rs`. It must be repointed.
- `should_close` is the shared flag set by `FileExitHandler` for `file.exit`;
  tests assert app-exit via `*shell.should_close.lock()`.

### Sequencing rationale
Behaviour-change items (1-6) land FIRST and are mutually dependent (they share
the dispatch path and nav stack); the reroute (7) must precede the dead-arm
deletion (8) so the arms are truly unreachable before removal. Tests (9-10) are
written red-first per item where practical, but because items 1-8 are a single
coherent behaviour switch, the realistic TDD unit is "write/rewrite the affected
tests, see them fail against old behaviour, implement 1-8, see them pass". Each
numbered item below still leaves the crate BUILDABLE (`cargo check -p ff-desktop`).

### File-size (400-line) watch
- `dispatch.rs` is 338 lines; the `=` front-door step adds ~8 lines -> ~346. OK.
- `commands.rs` is 344; item 1 adds a small `X` prelude branch (~12 lines) and
  item 8 DELETES the bare-THEME branch (~18 lines) -> net ~ -6. OK.
- `nav_stack.rs` is 251; new `nav_collapse_to_visual_root` (~15 lines) + edits to
  `nav_return` -> ~ +10. OK.
- `commands_ladder_a.rs`, `commands_ladder_b.rs` only SHRINK (deletions). OK.
- No file is expected to cross 400. If `dispatch.rs` does after item 1, split the
  FFEDIT router note or move the `=` helper body to `nav_stack.rs` (the helper
  `reinitialise_active_tab_to_pom` naturally belongs there).

---

# Implementation Plan

- [ ] 1. Add the single front-door `=` reinitialise-to-POM step + a nav helper.
      In `shell/nav_stack.rs` add `pub(super) fn reinitialise_active_tab_to_pom(&mut self)`
      that does `self.tabs.active_tab_mut().nav_stack.clear();` then
      `self.set_active_tab_home();` (POM = FFCMD_Root; `set_active_tab_home` already
      re-seeds the pom menu). In `shell/dispatch.rs`, at the TOP of
      `dispatch_command_string` (BEFORE `run_command_prelude`), add:
      `if let Some(rest) = raw.trim_start().strip_prefix('=') { self.reinitialise_active_tab_to_pom(); let rest = rest.trim().to_string(); self.dispatch_command_string(&rest); return; }`
      (clone `rest` to a `String` first to avoid borrowing `raw` across the
      `&mut self` call). This makes `=` apply EXACTLY ONCE at the front door,
      before `resolve_target` and before the Active_Environment. Validates:
      command-framework Req 10.2 (revised), 10.14.
      Files: crates/ff-desktop/src/shell/nav_stack.rs,
      crates/ff-desktop/src/shell/dispatch.rs
      Verify: `cargo check -p ff-desktop` compiles.

- [ ] 2. Make the three ad-hoc `=` sites consume an already-stripped remainder.
      Because item 1 strips `=` at the front door, no `=` reaches these on the
      typed/front-door path. (a) In `commands_ladder_a.rs` `try_exit_family`,
      remove the `=X` literal (done in item 3). (b) In `commands_fastpath.rs`
      `try_chained_fastpath`, the leading-`=` origin pop (`is_origin` ->
      `insert_pom_tab` + `ensure_pom_menu_loaded`) is now redundant for the
      front-door path; leave the `strip_prefix('=')` body-parse in place as a
      harmless no-op (the front door already stripped it) but DELETE the
      `is_origin` insert_pom_tab block, since the stack was already reinitialised
      to the POM. (c) In `resolve_pom_option_key`, the `strip_prefix('=')` becomes
      a no-op on the front-door path; keep it only so direct `handle_command`
      callers that pass a stale `=key` still resolve (defensive). Add a comment at
      each site pointing to the single front-door `=` step.
      NOTE: items 1-3 are interdependent; implement together, then verify.
      Files: crates/ff-desktop/src/shell/commands_fastpath.rs
      Verify: `cargo check -p ff-desktop` compiles.

- [ ] 3. Add the uniform `X` handler and `nav_collapse_to_visual_root`; drop
      `X`/`=X` from the EXIT family. In `shell/nav_stack.rs` add
      `pub(super) fn nav_collapse_to_visual_root(&mut self)`: if the active tab's
      `nav_stack` is non-empty, clone `nav_stack[0]` (bottom = Tab_Visual_Root),
      `reconstruct_context(&root)`, then `nav_stack.clear()` and set
      `focus.command_field_focus_requested = true` (match `nav_return`'s focus
      behaviour); if empty, no-op. Add `pub(super) fn nav_x(&mut self)`:
      `if self.tabs.active_tab().nav_stack.is_empty() { self.close_workspace_or_exit(); } else { self.nav_collapse_to_visual_root(); }`.
      In `commands_ladder_a.rs` `try_exit_family`, change the match to
      `upper == "EXIT" || upper == "QUIT" || upper == "LOGOFF"` (REMOVE `=X` and
      `X`). Add an `X` arm to the ladder in `try_commands_a` (place it with the
      END/RETURN arms): `if upper == "X" { self.nav_x(); self.open_error = None; return true; }`.
      Keep the EXIT-family gate in `run_command_prelude` as-is (it still protects
      EXIT/QUIT/LOGOFF; the editor-env skip still lets a bare `X` fall through to
      the FFEDIT claim, which is the documented EXCLUDE carve-out, Req 14.13).
      Validates: menu-workspace Req 14.13, 1g (revised); startup-and-session Req 14.46.
      Files: crates/ff-desktop/src/shell/nav_stack.rs,
      crates/ff-desktop/src/shell/commands_ladder_a.rs
      Verify: `cargo check -p ff-desktop` compiles.

- [ ] 4. `=X` falls out naturally -- confirm, do not special-case. With item 1
      stripping `=` and item 3 handling `X`, `=X` == reinitialise-to-POM then `X`
      at the (now empty) POM root -> `close_workspace_or_exit()`. There is NO
      literal `=X` arm anywhere after item 3. Add NO new code; this item is a
      review checkpoint only (its proof is the test in item 10).
      Files: (none)
      Verify: covered by item 10 tests.

- [ ] 5. END: confirm unchanged. `nav_end` (`nav_stack.rs`) already pops one /
      closes-at-empty / exits-when-last, and the END arm in `commands_ladder_a.rs`
      already calls it (after the split-unsplit guard). Make NO behaviour change;
      only confirm the arm and `nav_end` body are untouched.
      Files: (none)
      Verify: existing END tests still pass (item 9 re-runs them).

- [ ] 6. Repoint RETURN from collapse-to-POM to collapse-to-Tab_Visual_Root.
      In `shell/nav_stack.rs` `nav_return`: replace the body so that
      `if self.tabs.active_tab().nav_stack.is_empty() { self.close_workspace_or_exit(); } else { self.nav_collapse_to_visual_root(); }`
      (removing the `is_home` / `set_active_tab_home` POM jump). Update the
      `nav_return` doc comment (retire the CR-CH-038 POM-target note; cite
      CR-CH-052 Req 14.10 revised). Update the stale RETURN-arm comment in
      `commands_ladder_a.rs` to say "collapse to the Tab_Visual_Root (converges
      with bare X); at the root close/exit-when-last". RETURN now converges with
      bare `X` on a non-empty stack.
      Validates: menu-workspace Req 14.10 (revised).
      Files: crates/ff-desktop/src/shell/nav_stack.rs,
      crates/ff-desktop/src/shell/commands_ladder_a.rs
      Verify: `cargo check -p ff-desktop` compiles.

- [ ] 7. Change the compiled POM default option `X` command from `RETURN` to `X`.
      In `crates/ff-desktop/src/menu_workspace/defaults.rs` `DEFAULT_POM_TOML`,
      the `[[options]]` with `key = "X"`: change `command = "Return"` to
      `command = "X"` (keep `show_in_menu_bar = false` and the description, or
      reword the description to "Collapse to the visual root / close workspace
      (exit when last)"). Code-only compiled default; NEVER written to disk
      (CR-CH-021 preserved). This file's unit tests
      (`default_pom_toml_terminate_is_x_return`, `recovery_pom_menu_has_barebones_options`,
      `default_pom_toml_has_recovery_baseline_options`, and the menu-bar
      `default_menubar_*` tests that assert the `Return` command) MUST be updated
      in the SAME item to expect `"X"` (rename `default_pom_toml_terminate_is_x_return`
      -> `default_pom_toml_terminate_is_x_x` or similar, asserting `Some("X")`;
      update the `commands` vec expectations from `"Return"` to `"X"`; the menu-bar
      visible-options tests still exclude the hidden X option so their assertions
      stay the same except any that look up `o.command == "Return"` must look up
      `"X"`). Validates: menu-workspace Req 1g (revised).
      Files: crates/ff-desktop/src/menu_workspace/defaults.rs
      Verify: `cargo test -p ff-desktop defaults::` -- the defaults module tests pass.

- [ ] 8. Reroute the three nav callers through the front door. Change
      `self.handle_command(...)` to `self.dispatch_command_string(...)` at:
      (a) `commands.rs` `run_command_prelude` POM option-key recursion
      (`self.handle_command(&pom_command)` inside
      `if pom_command.to_uppercase() != *upper`);
      (b) `commands_fastpath.rs` `try_chained_fastpath` segment loop
      (`for segment in segments { self.handle_command(segment); }` ->
      `self.dispatch_command_string(segment);`);
      (c) `nav_stack.rs` `apply_start_command` (`self.handle_command(&resolved)`).
      After this, `resolve_target` runs for these seams so an in-scope verb
      reaches the CustomWorkspace dispatch arm (same shell open method as the
      ladder arm). Recursion safety preserved: the Function terminal still
      re-enters via `handle_command`, not the front door. Update the surrounding
      comments at each site to state the reroute.
      Validates: command-framework Req 10.14; menu-workspace Req 14.7, 14.8/14.9.
      Files: crates/ff-desktop/src/shell/commands.rs,
      crates/ff-desktop/src/shell/commands_fastpath.rs,
      crates/ff-desktop/src/shell/nav_stack.rs
      Verify: `cargo test -p ff-desktop b080_menu_bar_front_door_matches_typed_path_for_in_scope_verbs`
      still passes (reroute is behaviour-preserving for in-scope verbs).

- [ ] 9. Delete the now-dead ladder arms (AFTER item 8, so they are unreachable).
      Delete these arms and their bodies, leaving the fall-through intact:
      - `commands_ladder_a.rs` `try_commands_a`: the `KEYS` arm (`verb_arg(cmd, "KEYS")`
        -> `open_keys_editor`) and the `KINDS` arm (`upper == "KINDS"` ->
        `open_kinds_editor`). KEEP: EXIT family (now EXIT/QUIT/LOGOFF only), EDIT,
        START, MENU/MENU.OPEN, CLOSE, DOCK, HELP, PFSHOW, END, RETURN, and the new X.
      - `commands_ladder_b.rs` `try_commands_b1`: the `CONFIG`, `=FILES`/`FILES`,
        `GSEARCH`/`SEARCH`, `COMMANDS`, `MENUS`, `LOG`, `FILE CATALOGS`/`CATALOGS`,
        `PLUGINS`, `MACROS` arms. KEEP: `COMMAND`/`COMMAND <pos>`, `SNAPSHOT`,
        `CAPTURE`, `RESET BARE`, `RETRIEVE`.
      - `commands.rs` `run_command_ladder`: the bare-`THEME` branch ONLY
        (`if arg.is_empty() { self.open_theme_editor(); ... }`). KEEP the
        `THEME <name>` apply branch (resolve_theme_arg / set_active_theme) -- restructure
        the `if let Some(arg) = verb_arg(cmd, "THEME")` so a bare arg is no longer
        special-cased to open the editor (it now resolves via the front door's
        CustomWorkspace("theme_editor")); a bare `THEME` reaching the ladder as a
        fallback should fall through, so remove the empty-arg branch and keep the
        non-empty apply.
      Update/REMOVE the SUPERSEDED-ARM NOTE comments in both ladder files to say
      the arms are DELETED (CR-CH-052), and remove the now-unused `verb_arg`/import
      lines if any arm removal makes an import dead (`cargo check` will flag).
      Also delete the now-unreachable `KEYS`/`KINDS` doc blocks.
      Validates: command-framework Req 8.4 (behaviour-equivalent via resolve_target).
      Files: crates/ff-desktop/src/shell/commands_ladder_a.rs,
      crates/ff-desktop/src/shell/commands_ladder_b.rs,
      crates/ff-desktop/src/shell/commands.rs
      Verify: `cargo check -p ff-desktop` compiles with no dead-code/unused-import
      warnings; `cargo clippy -p ff-desktop` clean.

- [ ] 10. Rewrite the existing tests that encode the OLD semantics.
      In `crates/ff-desktop/src/shell/tests_command.rs`:
      - `non_editor_bare_x_still_exits`: under CR-CH-052 `CONFIG` navigates in
        place and PUSHES the POM onto the stack, so bare `X` now COLLAPSES to the
        Tab_Visual_Root (POM), it does NOT exit. REWRITE: rename to
        `non_editor_bare_x_collapses_to_visual_root`; assert after `run_command_line("X")`
        that `should_close` is FALSE and `shell.tabs.active_tab().is_home` is TRUE
        (back at the POM) and `nav_stack` is empty.
      - `editor_equals_x_exits_not_exclude`: `shell_new_untitled` adds a tab, so
        `tabs.len()` may be > 1 and `=X` CLOSES the workspace rather than exits.
        REWRITE: rename to `editor_equals_x_closes_workspace_not_exclude`; after
        `run_command_line("=X")` assert the editor tab was CLOSED (tab count
        decreased OR, if it was the last tab, `should_close` is true). Prefer a
        deterministic setup: ensure a known tab count first, then assert
        close-not-exit when >1 and exit when ==1 (two cases, or set up a single-tab
        editor root and assert exit). Keep the intent: `=X` escapes FFEDIT EXCLUDE.
      - Search `tests_command.rs` / `tests_focus.rs` / `tests_*` for any test
        asserting POM `X` -> `RETURN` -> stay-Home, or `=X`/`X` ->
        `file.exit` unconditionally (grep `should_close` + `"X"`/`"=X"`), and the
        parametrised EXIT-family test near line ~1486 that asserts `X`/`=X` from a
        non-menu context dispatch `file.exit`; rewrite those to the uniform model
        (they must now assert close-to-visual-root or close-workspace, exit only
        when last). Repoint any in-scope-verb test that calls `handle_command`
        directly to `dispatch_command_string` where it was relying on the old arm.
      Validates: the suite exercises the live uniform path.
      Files: crates/ff-desktop/src/shell/tests_command.rs (and any other tests_*.rs
      the grep surfaces)
      Verify: `cargo test -p ff-desktop` -- the rewritten tests pass; no stale
      OLD-semantics test remains (grep shows none asserting unconditional `=X` exit
      or POM-X->RETURN).

- [ ] 11. Add full-shell `egui_kittest` uniform-behaviour tests (write failing
      first where the behaviour is new). Add to `crates/ff-desktop/src/shell/tests_focus.rs`
      (or a new `tests_nav_ladder.rs` module if tests_focus.rs nears 200 test
      lines added), modelled on `harness_shell` / `make_shell` /
      `b080_menu_bar_front_door_matches_typed_path_for_in_scope_verbs` and the
      existing END-at-root test (`should_close` assertion ~L653). Cover, across the
      POM AND a non-POM `START SETTINGS` workspace:
      - `bare_x_collapses_to_visual_root_when_above_root`: navigate POM->CONFIG
        (non-empty stack), run `X`, assert back at visual root (is_home / Settings
        kind), stack empty, not closed.
      - `bare_x_at_visual_root_closes_workspace`: at a visual root with >1 tab, run
        `X`, assert the tab count decreased and `should_close` is false.
      - `equals_x_closes_not_app_exit`: with >1 tab, run `=X`, assert tab closed and
        `should_close` false; with exactly 1 tab, run `=X`, assert `should_close` true.
      - `end_pops_one_level`: build a 2-deep stack (POM->CONFIG->KEYS via nav),
        run `END`, assert one level popped (back at CONFIG), stack depth decreased by 1.
      - `app_exits_only_on_last_close`: single tab at root, run `X`, assert
        `should_close` true.
      - `pom_not_special_cased`: assert bare `X` on the POM (after a POM->CONFIG
        nav) behaves identically to bare `X` on a Config/Files workspace (same
        collapse-to-root outcome) -- compare two shells.
      - `ffedit_bare_x_stays_exclude_but_equals_x_escapes`: on an editor Context
        (`shell_new_untitled`), bare `X ALL` does NOT close (FFEDIT EXCLUDE), but
        `=X` closes/exits via FFCMD.
      - `equals_1_works_from_a_non_pom_tab`: on a `START SETTINGS` tab, run `=1`,
        assert it resolves as POM option 1 (Catalogs -> FilesPanel kind), proving
        `=` reinitialises to the POM from any tab.
      - `detached_workspace_closes_via_same_path`: detach a workspace (DETACH),
        run `X` at its visual root (or `=X`), assert it closes via
        close-workspace-or-exit (tab removed; exit only when last). If detaching
        headlessly is impractical, assert the close path at the model level
        (floating tab removed) and mark the OS-window aspect MANUAL with a reason
        in TCR.
      Each test carries `// Validates: menu-workspace Req 14.13-14.16 / command-framework
      Req 10.2,10.14 / startup-and-session Req 14.46` as applicable. Use a bounded
      frame budget (`for _ in 0..4 { harness.run(); }`) and assert on model/focus,
      not pixels.
      Files: crates/ff-desktop/src/shell/tests_focus.rs (or new tests_nav_ladder.rs)
      Verify: `cargo test -p ff-desktop` -- all new tests pass; they fail if run
      against the pre-item-1..8 code (confirm red-first on at least the new
      behaviours).

- [ ] 12. Update `docs/quality/TCR.md`: set the CR-CH-052 NOT-COVERED rows
      (command-framework Req 10.2 revised / 10.14; menu-workspace Req 1g revised /
      14.10 revised / 14.13 / 14.14 / 14.15 / 14.16; startup-and-session Req 12
      revised / 14.46 / Req 20.3 revised) to PASS (or MANUAL with a stated reason
      for the detached-OS-window aspect only). Append-only: update status in place,
      do not remove rows.
      Files: docs/quality/TCR.md
      Verify: grep the CR-CH-052 rows show PASS/MANUAL (no remaining red for a
      covered criterion).

- [ ] 13. Mark the CR-CH-052 tasks done and run the scoped gate. Tick the
      completed boxes in `docs/specs/menu-workspace/tasks.md` (tasks 39-47) and
      `docs/specs/command-framework/tasks.md` (task 30) as each lands. Then run the
      scoped checks and HAND OFF (do NOT run the full `ffwb-gate.ps1` -- that is the
      owner's manual step).
      Files: docs/specs/menu-workspace/tasks.md, docs/specs/command-framework/tasks.md
      Verify: `cargo fmt -- --check`, `cargo check -p ff-desktop`,
      `cargo clippy -p ff-desktop -- -D warnings`, `cargo test -p ff-desktop` all
      clean; then prompt the owner to run
      `pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1`.

## Gaps / assumptions noted
- The detached-workspace headless close test may need a model-level assertion
  (floating tab removed) if a real OS viewport cannot be driven under
  `egui_kittest`; TCR row for the OS-window aspect may stay MANUAL with a reason
  (per testing.md's justified-exception list). This is the only likely MANUAL row.
- `editor_equals_x_closes_workspace_not_exclude` setup: confirm whether
  `shell_new_untitled` leaves 1 or 2 tabs at the point of the test by reading the
  tab count in the rewrite; assert the correct close-vs-exit branch accordingly.
- If `dispatch.rs` crosses 400 lines after item 1 (unlikely at ~346), move the
  `reinitialise_active_tab_to_pom` body entirely into `nav_stack.rs` (already the
  plan) and keep only the 4-line strip-and-redispatch guard in `dispatch.rs`.
