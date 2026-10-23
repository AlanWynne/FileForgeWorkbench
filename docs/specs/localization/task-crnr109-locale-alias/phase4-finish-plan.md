# CR-NR-109 Phase 4 (Task 11) -- FINISH plan (implementer, worktree in place)

Worktree (ALL work here, in place -- do NOT `git worktree add`):
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr109-locale-alias`
Branch: `feature/crnr109-locale-alias` (based on `main`).
Cargo cwd for every scoped command = the worktree root
(`--manifest-path <worktree>\Cargo.toml` or run with that cwd).

This plan covers the REMAINING Task 11 work only. The mechanism (11.1-11.6) is
already implemented as uncommitted working-tree code and COMPILES CLEAN
(per the assessment). This plan does NOT re-implement it; it (1) applies the
three verification-pass fixes, (2) completes/validates Task 11.7, (3) finishes
the TCR + tasks.md docs, and (4) lands three conventional commits.

## IMPORTANT verified state correction (do not trust the stale assessment here)

The assessment marked Task 11.7 as NOT-STARTED. That is now STALE. Reading the
real files confirms the Task 11.7 DATA and verb keys ALREADY EXIST in the
working tree, and the reader test is ALREADY written:

- `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` exists, with the reviewed row
  `surface = "SOEK"`, `canonical = "FIND"`, `localisation_key = "verb-find"`.
- `crates/ff-i18n/i18n/af/ff-desktop-verbs.ftl` exists, with `verb-find = SOEK`.
- `crates/ff-i18n/i18n/en/ff-desktop-verbs.ftl` exists, with `verb-find = FIND`.
- The reader test `af_poc_alias_catalogue_loads_and_resolves` ALREADY EXISTS in
  `crates/ff-desktop/src/shell/alias_reader.rs` `#[cfg(test)]`, carrying
  `// Validates: localization Requirement 11.1, 5.1, 5.5`, and asserts exactly:
  SOEK -> FIND, FIND -> FIND (Identity_Base retained), and
  `display_surface_for_canonical("FIND") == Some("SOEK")` under the shipped af
  catalogue.
- The catalogue loader `ff-i18n/src/catalogue.rs::build_bundle` SCANS every
  `.ftl` file in `i18n/<locale>/` (directory merge), so `ff-desktop-verbs.ftl`
  is picked up AUTOMATICALLY -- NO loader registration is needed, and the
  `aliases/` subdir is correctly ignored by the `.ftl` scan (it is read
  separately by `alias_reader.rs`).

Therefore Task 11.7's remaining work is NOT authoring from scratch; it is: run
the already-written test SCOPED to confirm it now passes (the data it needs is
present), decide the FFEDIT POC file question (below), and commit the data +
keys + test together. Step 1 below handles this.

## HOUSE RULES (bake into every step)

- VERIFICATION SPLIT: agents COMPILE-VERIFY only. Allowed:
  `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`,
  `cargo fmt -p ff-desktop -- --check`, and the SINGLE scoped new test
  `cargo test -p ff-desktop af_poc_alias_catalogue_loads_and_resolves` (quick).
  Agents MUST NOT run full `cargo test` / `nextest` / `cargo gate` /
  `full-gate`. The OWNER runs the authoritative full gate + suite.
- TERMINAL DISCIPLINE: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive
  -Command "<one command>"`; ONE command per invocation (NO `;` chaining, the
  guard hard-blocks it); redirect to a log `*> tools\logs\<name>.log` and READ
  the log back with the read tool; verify by EFFECT, never trust the `Exit Code`
  line (`-1` is a capture artifact, not failure). Put any multi-step logic in a
  `tools/powershell/*.ps1` script and run THAT, reading its log.
- RUST STANDARDS: no `.rs` over 400 NON-test lines (excluding `#[cfg(test)]`);
  `thiserror` in libs; no `unwrap`/`expect` in non-test code.
- FRAMEWORK CONFORMANCE: build ON the single dispatch front door
  (`resolve_target` / `dispatch_command_target` / `dispatch_command_string`),
  the existing per-environment `AliasTable` + `apply_locale_overlay_keyed`
  collision machinery, and the `ff_i18n::t()` seam. The resolver REWRITES the
  typed verb to its ROOT; the ROOT is what dispatches / persists / records. Add
  NO second dispatcher, nav stack, persistence format, or focus mechanism.
- WORKSPACE CONFORMANCE: keep the menu-kind first-Tab focus `egui_kittest` test
  (`full_shell_localised_menu_first_tab_still_focuses_first_interior`, and the
  pre-existing POM first-Tab test) GREEN; do NOT alter the `InteriorFocus`
  contract.
- DOCS/SOURCE ASCII-only (no em dash, curly quotes, arrows, logic glyphs; use
  `--`, `'`, `"`, `->`).
- HEARTBEAT: at the START of each step and after each verify, append a
  timestamped line to
  `tools\logs\crnr109-phase4-progress.txt` (e.g.
  `2024-XX-XX HH:MM Step N: <what> done`). Create the file/dir if missing.

## Verified current line counts (all under the 400 non-test rule)

From `tools/logs/crnr109-linecount.log` (first `#[cfg(test)]` marks the test
boundary; NonTest = lines before it):

| File | Total | NonTest |
|------|-------|---------|
| `shell/environment.rs` | 629 | 352 |
| `shell/environment_contract.rs` | 56 | 56 |
| `shell/alias_reader.rs` | 556 | 250 |
| `shell/alias_overlay.rs` | 529 | 214 |
| `shell/locale_resolver.rs` | 247 | 170 |
| `shell/construct.rs` | 360 | 360 |
| `shell/dispatch.rs` | 136 | 120 |
| `shell/commands_menu.rs` | 353 | 353 |
| `menu_workspace/render.rs` | 831 | ~485 (test module at the `// === Tests` boundary; the line-18 `#[cfg(test)] use` is a test-only import, NOT the test module -- confirm by reading, see Step 4) |
| `menu_workspace/render_layout.rs` | 388 | ~85 prod + layout tests |
| `menu_workspace/i18n_keys.rs` | 231 | 120 |

No split is expected to be required (see Step 4 for the render.rs confirmation).

---

# Ordered implementation plan

- [ ] 1. Validate + finalise Task 11.7 (af POC alias data + minimal verb keys + reader test).
      The DATA and verb keys already exist in the working tree (see the state
      correction above); this step confirms they are correct, resolves the
      FFEDIT-file question, and proves the already-written reader test passes.
      Actions:
      - Confirm (read) the three data files are byte-correct and ASCII-only:
        `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` (SOEK -> FIND, key
        `verb-find`), `crates/ff-i18n/i18n/af/ff-desktop-verbs.ftl`
        (`verb-find = SOEK`), `crates/ff-i18n/i18n/en/ff-desktop-verbs.ftl`
        (`verb-find = FIND`). Fluent key syntax is lowercase-hyphen
        (`verb-find`) -- already correct.
      - FFEDIT POC file: the POC verb FIND is surfaced through the FFCMD menu-kind
        context (the 11.6 full-shell test uses a FIND menu option; the reader test
        loads FFCMD). FFEDIT.toml is therefore NOT required for the POC and is
        OUT of scope for this run -- do NOT create
        `crates/ff-i18n/i18n/af/aliases/FFEDIT.toml`. Record this decision in the
        commit message. (Rationale: the plan's Task 11.7 says "FFEDIT.toml if an
        editor verb is in the POC"; no editor verb is in this one-row POC.)
      - No `i18n/en/aliases/` tree is needed: the reader treats a missing
        per-environment file as identity-only, and the `en` display surface is
        the root itself via `verb-find = FIND`. Do NOT create `en/aliases/`.
      - The reader test `af_poc_alias_catalogue_loads_and_resolves` already exists
        with the exact `// Validates: localization Requirement 11.1, 5.1, 5.5`
        annotation and calls the real FFCMD identity-table API:
        `read_catalogue_from_dir(&af_aliases, &[FFCMD_NAME])` ->
        `AliasCatalogue::keyed_rows_for` / `load_into_keyed(FFCMD_NAME, &mut
        table)` on `AliasTable::identity_for(&["FIND","CATALOGS","FILES","SETTINGS"])`,
        asserting `canonical_verb("SOEK") == Some("FIND")`,
        `canonical_verb("FIND") == Some("FIND")`, and
        `display_surface_for_canonical("FIND") == Some("SOEK")`. Do NOT rewrite
        it; it is the TDD red-then-green test for the DATA (it fails without the
        shipped af files, passes with them).
      Files (verify only, create nothing new): the three data files above; the
      test in `crates/ff-desktop/src/shell/alias_reader.rs`.
      Verify: run the SINGLE scoped test to a log and read it back ---
      `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo
      test -p ff-desktop --manifest-path <worktree>\Cargo.toml
      af_poc_alias_catalogue_loads_and_resolves *>
      tools\logs\crnr109-afpoc-test.log"` then READ
      `tools\logs\crnr109-afpoc-test.log` and confirm `test result: ok. 1 passed`.
      Expected: PASS (the data is present). If it FAILS, the failure names the
      missing/incorrect file -- fix that file, do not weaken the test.

- [ ] 2. Verification-pass fix (a): FFCMD root-resolution SEED COVERAGE (the single
      most important correctness check for Req 11.6.1.2.1).
      Verified mechanism: the FFCMD identity seed is built by
      `crates/ff-desktop/src/shell/alias_reader.rs::builtin_ffcmd_roots()`, which
      derives roots ONLY from the option `command` values of the two compiled
      built-in menus (`menu_workspace::defaults::recovery_pom_menu()` and
      `recovery_settings_menu()`). It is consumed in
      `crates/ff-desktop/src/shell/construct.rs` (the `locale_alias_store` block,
      ~line 170) via `LocaleAliasStore::seed_and_overlay_for_locale(&root_refs,
      &help_locale)`. The seeded FFCMD roots are therefore exactly:
      `SETTINGS, CATALOGS, FILES, HELP, X, CONFIG, THEME, MENUS, KEYS, KINDS,
      "RESET BARE"` (uppercased; note the multi-word `RESET BARE`).
      The resolver `resolve_locale_verb` (`shell/locale_resolver.rs`) does a
      SINGLE `canonical_verb` lookup against this table and returns
      `Unknown(verb)` on a miss. `rewrite_verb_to_root` leaves an `Unknown`
      string UNCHANGED, so the existing front door
      (`dispatch_command_string` -> prelude/`resolve_target`/ladder) still
      dispatches an unseeded bare root normally -- functional dispatch is NOT
      broken for unseeded roots. The GAP is narrower but real:
        - A legitimate bare root that is NOT a POM/Settings option root (e.g.
          `EDIT`, `BROWSE`, and any registered command verb or user-menu option
          root) resolves to `Unknown`, so Req 11.6 branch 1.2.1 ("a bare root
          resolves to Root") is NOT formally satisfied for it, and the menu-render
          display path `display_surface_for_canonical(root)` returns `None` ->
          falls back to the raw root (acceptable per Req 11.8 but it means that
          root can never carry a localised surface in a menu).
        - Localised SURFACES of unseeded roots DO still work, because the overlay
          row is loaded into the table from the TOML regardless of the seed.
      Decision (choose the registry-fallback approach, which matches the impl
      plan's Task 11.4 step-2 "root command table" branch and is the least
      invasive): extend the FFCMD seed so every legitimate root resolves to
      `Root`. Preferred implementation: in `builtin_ffcmd_roots()`, ALSO fold in
      the registered root command verbs from the command registry
      (`cmd_registry` / `command_store.definitions`) in addition to the two
      built-in menus' option roots, deduplicated case-insensitively. This keeps a
      SINGLE lookup seam (the store table) -- no second dispatcher -- and makes a
      bare registered root self-resolve via its synthesised identity row (Req
      11.3/11.6.1.2.1). If threading the registry into `builtin_ffcmd_roots()` at
      construction is awkward (it is a free function), instead compute the root
      set in `construct.rs` where `cmd_registry` / `command_store` are in scope:
      union `builtin_ffcmd_roots()` with the registry's root verb ids, then pass
      the union to `seed_and_overlay_for_locale`. Keep `builtin_ffcmd_roots()`
      for the menu-derived subset and add the registry union at the call site.
      TDD: add a failing unit test FIRST (in `shell/locale_resolver.rs`
      `#[cfg(test)]` using `make_shell`, or in `alias_reader.rs` against a seeded
      store) named
      `every_registered_ffcmd_root_resolves_to_itself`
      `// Validates: localization Requirement 11.6, 11.3` -- assert that a
      registered root verb that is NOT a POM/Settings option root (pick a real
      one discovered by reading the registry, e.g. a navigation/editor-open verb)
      resolves `resolve_locale_verb(root, FFCMD_NAME) == Root(root)`, not
      `Unknown`. Confirm the test FAILS before the seed change and PASSES after.
      Scope the change so the `en`/no-overlay behaviour stays identity-only and
      byte-neutral (Req 10.2) -- adding identity rows for roots does not change
      dispatch, only makes the bare root self-resolve.
      Files: `crates/ff-desktop/src/shell/alias_reader.rs` (extend
      `builtin_ffcmd_roots` or add a registry-union helper) and/or
      `crates/ff-desktop/src/shell/construct.rs` (union at the call site);
      the new test in `shell/locale_resolver.rs`.
      Verify: `cargo check -p ff-desktop` + `cargo clippy -p ff-desktop -- -D
      warnings` + `cargo fmt -p ff-desktop -- --check`, each redirected to a log
      under `tools\logs\` and READ back (confirm no errors/warnings). The new
      scoped unit test is quick -- run `cargo test -p ff-desktop
      every_registered_ffcmd_root_resolves_to_itself *>
      tools\logs\crnr109-seed-test.log` and read the log for `1 passed`. Do NOT
      run the full suite.

- [ ] 3. Verification-pass fix (b): NARROW the blanket `#![allow(dead_code)]` now
      that production callers exist.
      Verified: two module-level blanket allows exist in the worktree:
      `crates/ff-desktop/src/shell/alias_reader.rs` line 39
      (`#![allow(dead_code)]`) and `crates/ff-desktop/src/shell/alias_overlay.rs`
      line 57. `alias_reader.rs` now has real production callers
      (`builtin_ffcmd_roots`, `LocaleAliasStore::seed_and_overlay_for_locale`,
      `publish_active_alias_store`, `reload_active_alias_store_for_locale` are
      called from `construct.rs`/`main.rs`; `table_for` and
      `display_surface_for_canonical` are called from `locale_resolver.rs`), so
      the blanket allow masks genuinely-dead items. `environment.rs` also carries
      item-level `#[allow(dead_code)]` on `apply_locale_overlay` /
      `apply_locale_overlay_keyed` -- `apply_locale_overlay_keyed` now has a
      production caller chain (via the reader's `load_into_keyed`), so its allow
      is likely removable; `apply_locale_overlay` (the keyless shim) may still be
      test-only.
      Actions:
      - Remove the blanket `#![allow(dead_code)]` from `alias_reader.rs`. Run
        clippy `-D warnings` (scoped) and read the log. For each item clippy
        flags as truly unused, EITHER delete it if it has no caller and no near-
        term consumer, OR replace the blanket allow with a NARROW item-level
        `#[allow(dead_code)]` carrying a one-line reason comment (e.g. the
        `ffcmd_roots` private helper if only the reload path uses it). Prefer
        deletion of anything with zero callers.
      - Repeat for `alias_overlay.rs`: remove the blanket allow; keep only the
        narrow item-level allows clippy proves are still needed (ahead-of-consumer
        API), each with a reason comment.
      - Re-check the two item-level allows in `environment.rs`: drop the one on
        any method that now has a production caller; keep (with reason) only those
        still test-only.
      - Do NOT add new `#[allow]`s to silence warnings introduced by Step 2; fix
        the root cause instead.
      Files: `crates/ff-desktop/src/shell/alias_reader.rs`,
      `crates/ff-desktop/src/shell/alias_overlay.rs`,
      `crates/ff-desktop/src/shell/environment.rs`.
      Verify: `cargo clippy -p ff-desktop -- -D warnings *>
      tools\logs\crnr109-clippy.log` then READ the log -- it must end with
      `Finished` and contain NO `warning:`/`error:` lines. Also
      `cargo check -p ff-desktop` and `cargo fmt -p ff-desktop -- --check` to
      logs, read back clean.

- [ ] 4. Verification-pass fix (c): confirm the 400-line rule holds; split only if needed.
      Verified counts (Step-0 table): `environment.rs` 352, `environment_contract.rs`
      56, `locale_resolver.rs` 170, `render_layout.rs` ~85 prod -- all well under
      400. The one to CONFIRM by reading is `menu_workspace/render.rs`: total 831,
      but the first `#[cfg(test)]` is at line 18 and is a TEST-ONLY `use`
      (`#[cfg(test)] use super::render_layout::option_font;`), NOT the test module.
      Action: READ `menu_workspace/render.rs` and locate the REAL test module
      boundary (the `// === Tests` separator / the `#[cfg(test)] mod tests`
      block). Count non-test production lines = lines before that module. The impl
      plan measured ~485 for the pre-split file and the split into
      `render_layout.rs` already landed, so the production body should now be
      under 400. If the production body is confirmed < 400, NO split -- record the
      measured number in the progress log. If it is >= 400, split as a PURE
      REFACTOR (no behaviour change, no gate): move the next cohesive block (the
      frame render loop helpers) into a new `menu_workspace/render_frame.rs`,
      register `mod render_frame;` in `menu_workspace/mod.rs`, fix imports, and
      confirm both files land under 400 non-test lines. Only split if the
      measurement requires it.
      Files: read `menu_workspace/render.rs`; (only if over) new
      `menu_workspace/render_frame.rs` + `menu_workspace/mod.rs` edit.
      Verify: re-run the line-count script
      `tools\powershell\crnr109-linecount.ps1` to
      `tools\logs\crnr109-linecount.log` and read it; confirm every listed `.rs`
      shows NonTest < 400. If a split was done, `cargo check -p ff-desktop` +
      `cargo fmt -p ff-desktop -- --check` to logs, read back clean.

- [ ] 5. COMMIT 1 -- Task 11.7 (af POC data + verb keys + reader test).
      Stage ONLY these files by name (never `git add -A`/`git add .`):
      `crates/ff-i18n/i18n/af/aliases/FFCMD.toml`,
      `crates/ff-i18n/i18n/af/ff-desktop-verbs.ftl`,
      `crates/ff-i18n/i18n/en/ff-desktop-verbs.ftl`,
      `crates/ff-desktop/src/shell/alias_reader.rs` (the reader test -- only if it
      is not already in a prior commit; if `alias_reader.rs` also carries Step 2/3
      changes, commit it in COMMIT 2 instead and keep COMMIT 1 to the data+keys,
      noting the test travels with the module in COMMIT 2).
      Conventional-commit message, e.g.:
      `feat(localization): add Afrikaans POC command-alias data + verb keys (CR-NR-109 Task 11.7)`
      Body: name the SOEK -> FIND row, the `verb-find` keys in en/af, note FFEDIT
      POC file intentionally omitted (no editor verb in the one-row POC), and that
      the reader test `af_poc_alias_catalogue_loads_and_resolves` proves it.
      Do NOT push/PR/merge.
      Verify: `git status` + `git show --stat HEAD` to a log, read back -- confirm
      ONLY the intended files are in the commit.

- [ ] 6. COMMIT 2 -- verification-pass fixes (seed coverage + dead_code narrowing + any 400 split).
      Stage ONLY the files touched by Steps 2-4 by name, e.g.:
      `crates/ff-desktop/src/shell/alias_reader.rs`,
      `crates/ff-desktop/src/shell/construct.rs`,
      `crates/ff-desktop/src/shell/locale_resolver.rs`,
      `crates/ff-desktop/src/shell/alias_overlay.rs`,
      `crates/ff-desktop/src/shell/environment.rs`, and (only if split in Step 4)
      `crates/ff-desktop/src/menu_workspace/render.rs` +
      `crates/ff-desktop/src/menu_workspace/render_frame.rs` +
      `crates/ff-desktop/src/menu_workspace/mod.rs`.
      Conventional-commit message, e.g.:
      `fix(localization): seed all FFCMD roots + narrow dead_code allows (CR-NR-109 Task 11 verification)`
      Body: note every registered FFCMD root now self-resolves (Req 11.6.1.2.1),
      the blanket `#![allow(dead_code)]` on alias_reader/alias_overlay narrowed or
      removed, clippy `-D warnings` clean, and the 400-line rule confirmed.
      Do NOT push/PR/merge.
      Verify: `git show --stat HEAD` to a log, read back -- only intended files.

- [ ] 7. COMMIT 3 -- TCR + tasks.md doc updates.
      7a. Flip the 10 CR-NR-109 rows in `docs/quality/TCR.md` (Req 11.1-11.10,
      currently the `[DOT]` NOT COVERED rows after the CR-NR-109 comment near line
      3844) from NOT COVERED to PASS, each with the ACTUAL backing test name as
      evidence (format matching the existing passing rows:
      `| `ff-desktop` | <pass-marker> | `<module>::tests::<test_fn>` | localization
      Req 11.x ... |`). Use the status marker the file's other PASS rows use (read
      an adjacent PASS row to copy the exact marker glyph/text; the file uses a
      status-emoji convention that is the one allowed non-ASCII exception for
      TCR.md). Map each row to the VERIFIED test name below (only flip a row whose
      test actually exists -- all listed below were confirmed present by reading):
        - Req 11.1 -> `shell::environment::tests::display_surface_resolves_localisation_key_else_raw_surface`
          (and exercised by `shell::alias_reader::tests::af_poc_alias_catalogue_loads_and_resolves`)
        - Req 11.2 -> `shell::alias_reader::tests::reader_parses_toml_rows_into_catalogue`
        - Req 11.3 -> `shell::environment::tests::synthesised_identity_row_exists_for_every_root`
          (plus `shell::locale_resolver::tests::every_registered_ffcmd_root_resolves_to_itself` from Step 2)
        - Req 11.4 -> `shell::environment::tests::overlay_row_replaces_synthesised_default_without_changing_root`
          (and `..::overlay_row_matching_identity_surface_updates_without_collision`)
        - Req 11.5 -> `shell::alias_reader::tests::reader_parses_toml_rows_into_catalogue`
          (reader parse+load; also `..::missing_environment_file_leaves_that_environment_identity_only`)
        - Req 11.6 -> `shell::locale_resolver::tests::resolve_locale_verb_follows_the_req_11_6_order`
          (plus `..::resolution_is_scoped_to_the_active_environment` and the full-shell
          `shell::tests_locale_menu::full_shell_menu_option_shows_localised_surface_and_dispatches_root`)
        - Req 11.7 -> `shell::tests_locale_menu::full_shell_menu_option_shows_localised_surface_and_dispatches_root`
          (menu display + dispatch; helper `menu_workspace::i18n_keys::tests::localised_option_command_resolves_alias_surface_else_root`)
        - Req 11.8 -> `menu_workspace::i18n_keys::tests::localised_option_command_resolves_alias_surface_else_root`
          (helper root fallback) + full-shell `shell::tests_locale_menu::full_shell_menu_option_en_shows_root_verb_unchanged`
        - Req 11.9 -> `shell::locale_resolver::tests::root_command_is_returned_regardless_of_surface`
          + full-shell `shell::tests_locale_menu::full_shell_menu_option_shows_localised_surface_and_dispatches_root`
        - Req 11.10 -> `shell::alias_reader::tests::reader_rejects_within_environment_collision_and_retains_base`
      IMPORTANT: a row is flipped to PASS only because its test EXISTS in the
      tree; per the VERIFICATION SPLIT the OWNER's full-suite run is the
      authoritative green. State in the commit body that the evidence is the
      named test and that the owner's gate confirms the suite. Confirm each exact
      test-module path by reading the file before writing it (module path =
      `shell::<file stem>::tests::<fn>` etc.) so the evidence string is accurate.
      7b. In `docs/specs/localization/tasks.md`, tick subtasks 11.1 through 11.7
      from `[ ]` to `[x]` (and the parent `- [ ] 11.` to `- [x] 11.` once all
      seven are done). Use ONLY `[ ]`/`[x]` markers; keep each descriptive title
      verbatim; put any status note on an indented sub-bullet, never inside the
      brackets. ASCII only.
      Files: `docs/quality/TCR.md`, `docs/specs/localization/tasks.md`.
      Stage ONLY those two by name. Conventional-commit message, e.g.:
      `docs(localization): flip CR-NR-109 Req 11.1-11.10 TCR rows to PASS + tick Task 11 subtasks`
      Do NOT push/PR/merge.
      Verify: read the edited TCR rows and tasks lines back; `git show --stat HEAD`
      to a log, read back -- only the two docs in the commit.

- [ ] 8. Final compile-verify + heartbeat close-out.
      Run the scoped trio to logs and read each back clean:
      `cargo fmt -p ff-desktop -- --check *> tools\logs\crnr109-fmt.log`,
      `cargo check -p ff-desktop *> tools\logs\crnr109-check.log`,
      `cargo clippy -p ff-desktop -- -D warnings *> tools\logs\crnr109-clippy-final.log`.
      Confirm the three commits exist on `feature/crnr109-locale-alias`
      (`git log --oneline -3` to a log, read back). Append the final timestamped
      line to `tools\logs\crnr109-phase4-progress.txt`. Hand off to the OWNER for
      the authoritative full `cargo gate` / `egui_kittest` suite run (agents do
      NOT run it). Do NOT push/PR/merge.

## Ordering and dependencies

1 (validate 11.7 data+test) is independent and first (smallest, proves the data).
2 (seed coverage) adds a test + touches the seed; 3 (dead_code) must run AFTER 2
so clippy sees the final set of callers; 4 (400-line) is a read-only confirm
unless a split is forced. Commits 5/6/7 follow their respective work in that
order; 8 is the final compile-verify + hand-off. Each step leaves the crate
compiling, lint/format clean, with its scoped new test (where any) passing.

## Known gaps / assumptions (flagged, with reasonable defaults taken)

- The FFEDIT af POC file is intentionally omitted (no editor verb in the one-row
  POC); if the owner later wants an FFEDIT surface in the POC, add
  `af/aliases/FFEDIT.toml` + the matching verb key then.
- Step 2 assumes the command registry exposes its root verb ids at construction
  (`cmd_registry` / `command_store.definitions` are already in scope in
  `construct.rs`). If the registry's root-verb set is not readily enumerable, the
  fallback is to keep the single-lookup design and instead add a registry
  `is_root_command` check as resolver step 2 (as the impl plan's Task 11.4
  originally described) so a bare unseeded root returns `Root(verb)` without
  seeding -- functionally equivalent for Req 11.6.1.2.1. Choose whichever the
  registry API makes clean; both satisfy the criterion and keep ONE dispatch seam.
- TCR status-marker glyph: copy the exact PASS marker from an adjacent passing
  row in TCR.md (the file's established emoji convention is the one allowed
  non-ASCII exception for that file); do not invent a new marker.
