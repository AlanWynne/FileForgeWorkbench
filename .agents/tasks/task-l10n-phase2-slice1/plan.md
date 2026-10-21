# Implementation Plan -- Localisation Phase 2, Slice 1 (CR-NR-103 / L10N.4)

Scope: FEAT-002 (menu title + option descriptions localised at render time) then
FEAT-003 (help content localised via per-locale directories). FEAT-001 is DONE
and merged to local main -- do NOT redo it (ff-i18n is wired, the English menu
keys exist in `crates/ff-i18n/i18n/en/menus.ftl`, the `ui.locale` reload callback
is registered in `main.rs`).

All work happens INSIDE the existing worktree:
`c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice1`
(branch `feature/l10n-phase2-slice1`). A RELATIVE path lands in the parent
workspace -- always use absolute paths, and `git -C <that worktree path>` for git.
Commit locally on the branch; NEVER push.

## Ground rules (apply to every item)

- TDD: write the failing test FIRST, each with a `// Validates: Requirement X.Y`
  comment, named by behaviour, deterministic. Then the minimum implementation.
- Behaviour-neutral at `ui.locale = en` (Req 10.2): existing egui_kittest /
  focus / title_line tests MUST stay green unchanged. The owner proves this with
  the full suite; agents prove the English baseline by construction (the catalogue
  values are byte-identical to the parsed English prose).
- Build ON existing seams; add NO new mechanism (Req 10.1).
- ASCII only in all `.rs` and docs (documentation-charset.md). NOTE: existing
  doc-comment lines in `content_loader.rs` already contain em-dashes -- do NOT
  touch those lines; keep every NEW line ASCII (`--`, `->`).
- No source `.rs` over 400 NON-test lines. `render.rs` is ALREADY ~487 non-test
  lines (over the limit, pre-existing) -- FEAT-002 must NOT add logic there.

## VERIFICATION POLICY (non-negotiable, machine-wide steering)

Agents do SCOPED compile/lint/fmt verification ONLY, each written to a log that
is then READ back (the reported `Exit Code: -1` is a capture artifact -- judge by
the log contents, never by the exit line). Run every command through the clean
pwsh7 wrapper, one command per invocation, never `;`-chained:

```
C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<cmd> *> tools\logs\<name>.log"
```

DO NOT run `cargo test` broadly, `cargo nextest`, `cargo gate`, or
`cargo full-gate` -- those hang this machine. A SINGLE named scoped unit test is
allowed ONLY when it returns in a couple of seconds (the pure key-builders in
FEAT-002, the pure loader resolver in FEAT-003). The OWNER runs the full
gate/test suite MANUALLY, outside Kiro, and is the authority on a green run.

---

## FEAT-002 -- Localise built-in menu title + option descriptions (task 5)

Satisfies Req 7.1, 7.2, 7.3, 7.4, 4.2, 10.2. Depends on FEAT-001 (catalogue
installed; `menu-pom-*` / `menu-settings-*` keys present). The `command` VALUE is
never localised (Req 7.3/4.2); built-in menus stay code-only, no translated TOML
on disk (Req 7.2); a missing key falls back to the parsed English prose (Req 7.4).

Key convention (lowercase slug = lowercase the input, collapse runs of
non-alphanumeric chars to a single `-`, trim leading/trailing `-`):
- Title:       `menu-<menu-slug>-title`            e.g. `menu-pom-title`
- Description: `menu-<menu-slug>-opt-<command-slug>-desc`
               e.g. `menu-settings-opt-reset-bare-desc` (command "Reset bare").

- [ ] 1. Create the i18n key module with failing unit tests first.
      Create `crates/ff-desktop/src/menu_workspace/i18n_keys.rs` with a
      `#[cfg(test)]` module FIRST asserting: `menu_title_key("pom") ==
      "menu-pom-title"`; `menu_option_desc_key("settings", "Reset bare") ==
      "menu-settings-opt-reset-bare-desc"`; the slug collapses non-alphanumeric
      runs (e.g. a command like "Reset  bare!" -> "reset-bare"); `t_or(key,
      fallback)` returns `fallback` when the key is absent (miss sentinel = key
      returned) and the catalogue value when present. Annotate each
      `// Validates: Requirement 7.1` / `7.4` / `4.2` as appropriate.
      Then implement: `pub(crate) fn t_or(key: &str, fallback: &str) -> String`
      (returns `ff_i18n::t(key)` unless it equals `key`, else `fallback.to_string()`);
      a private `slug(&str) -> String`; `pub(crate) fn menu_title_key(menu_name:
      &str) -> String`; `pub(crate) fn menu_option_desc_key(menu_name: &str,
      command: &str) -> String`; and the display helper `pub(crate) fn
      localised_option_description(menu_name: &str, option: &super::MenuOption)
      -> String` = `t_or(&menu_option_desc_key(menu_name, &option.command),
      &option.description)`. Keep ASCII; functions <= ~40 lines; no unwrap/expect.
      Files: crates/ff-desktop/src/menu_workspace/i18n_keys.rs
      Verify: `cargo test -p ff-desktop i18n_keys *> tools\logs\feat002-i18nkeys.log`
      then READ the log -- the key/slug/fallback tests pass (fast, pure).

- [ ] 2. Register the new module.
      Add `pub mod i18n_keys;` to the module list in
      `crates/ff-desktop/src/menu_workspace/mod.rs` (next to `pub mod render;`).
      Files: crates/ff-desktop/src/menu_workspace/mod.rs
      Verify: `cargo check -p ff-desktop *> tools\logs\feat002-check.log` then
      READ the log -- compiles clean.

- [ ] 3. Localise the two DESCRIPTION display sites in render.rs (NO new logic in
      render.rs -- it is already over 400 non-test lines).
      In `crates/ff-desktop/src/menu_workspace/render.rs`, inside
      `render_menu_workspace`, derive the menu-name ONCE from the backing state
      (`state.file_path.file_stem()` lowered -- the same stem `menu_name_label()`
      uses, but lowercased for the key). Then replace BOTH description bindings
      with the helper from item 1:
        - the row `let desc = option.description.clone();` becomes
          `let desc = super::i18n_keys::localised_option_description(&menu_name, option);`
        - in the keyboard-focus branch, the `result.focused_option` description
          field (currently `option.description.clone()`) becomes the same
          `localised_option_description(&menu_name, option)` call.
      Leave `option.command` and every other field untouched (Req 7.3). This is a
      call-site swap only -- it does not grow render.rs's logic.
      Files: crates/ff-desktop/src/menu_workspace/render.rs
      Verify: `cargo check -p ff-desktop *> tools\logs\feat002-check.log` then
      READ the log -- compiles clean. Confirm render.rs non-test line count did
      not materially grow (still ~487; the swap is line-neutral).

- [ ] 4. Localise the TITLE in the Title_Line derivation.
      In `crates/ff-desktop/src/shell/mod_helpers.rs`, in `title_line_text`'s
      `TabKind::MenuWorkspace(ref mw)` arm: when a menu is loaded, compute the
      raw title (`mw.menu_title()`) and the menu-name (`mw.menu_name_label()`
      lowered), and return `crate::menu_workspace::i18n_keys::t_or(
      &crate::menu_workspace::i18n_keys::menu_title_key(&name), &raw_title)`.
      Preserve the existing fallback to `tab.title.clone()` when no menu is
      loaded. Do NOT change any other arm. Keep the arm small -- if it would grow
      past a few lines, factor a tiny private helper in the same file.
      Files: crates/ff-desktop/src/shell/mod_helpers.rs
      Verify: `cargo check -p ff-desktop *> tools\logs\feat002-check.log` then
      READ the log -- compiles clean.

- [ ] 5. Add the catalogue-backed behaviour test (installed catalogue).
      In the `i18n_keys.rs` test module, add a test that installs the shipped
      English catalogue (same pattern as `i18n_bootstrap.rs` tests:
      `Catalogue::load_identity_base(root)` then `ff_i18n::set_catalogue(..)`
      using `env!("CARGO_MANIFEST_DIR")/../ff-i18n/i18n`) and asserts: a COVERED
      option (e.g. pom "settings" via its real command value) resolves to the
      catalogue English text, and an UNCOVERED option returns its parsed English
      fallback unchanged. Annotate `// Validates: Requirement 7.1, 7.4`.
      (Behaviour-neutrality, Req 10.2: the catalogue values are byte-identical to
      the parsed English, so this also demonstrates the en baseline is unchanged.)
      Files: crates/ff-desktop/src/menu_workspace/i18n_keys.rs
      Verify: `cargo test -p ff-desktop i18n_keys *> tools\logs\feat002-i18nkeys.log`
      then READ the log -- the covered/uncovered test passes.

- [ ] 6. Final scoped gate for FEAT-002 + commit.
      Run, reading each log: `cargo check -p ff-desktop *> tools\logs\feat002-check.log`;
      `cargo clippy -p ff-desktop -- -D warnings *> tools\logs\feat002-clippy.log`
      (zero warnings); `cargo fmt -p ff-desktop -- --check *> tools\logs\feat002-fmt.log`
      (no diff). Confirm i18n_keys.rs, render.rs, mod_helpers.rs each under 400
      NON-test lines (render.rs must not have grown). Set FEAT-002 status to
      `completed` and record findings. Commit locally:
      `git -C c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice1 add -- <changed files>`
      then a descriptive commit. NEVER push.
      Files: the FEAT-002 files above + features/FEAT-002.json
      Verify: the three logs are clean; `git -C <worktree> status` shows the
      intended files committed, no stray changes.

---

## FEAT-003 -- Localise help via per-locale content directories (task 6)

Satisfies Req 6.1, 6.2, 6.3, 6.4, 10.2. Independent of FEAT-002 but depends on
FEAT-001's `ui.locale` reload wiring. NO change to parsing, `discover_files`,
`ContentParser`, `HelpTopic`, or `TopicKey` -- Topic_Key and cross-references come
from parsed content, not the path, so they stay unlocalised (Req 6.3).

DESIGN DECISION (hot-reload, Req 6.4), grounded in the read of the code:
`HelpState.registry` (state_groups.rs) and `HelpContextPanel.model`
(help_context.rs) both hold `Arc<HelpTopicRegistry>` clones sharing ONE inner
`Arc<RwLock<HashMap>>`. The registry has no `clear()`, and `register_file_topic`
never removes -- so re-loading a new locale into the same registry would leave
stale topics. Chosen fix (no new mechanism, Req 10.1): add an additive
`replace_file_topics` to the registry that removes FileBased topics in place and
bulk-inserts the new locale's set, mutating the SHARED inner map so both Arc
holders see the swap and the panel's Arc stays valid. The reload closure (chained
onto the existing one in main.rs) re-resolves + reloads via that method.

- [ ] 7. Add locale-aware directory resolution to the loader, test-first.
      In `crates/ff-help/src/content_loader.rs` add a `#[cfg(test)]` test FIRST
      (temp dirs): `resolve_help_directory_for_locale` with a `help/fr` present
      picks `help/fr`; with `help/fr` absent falls back to `help`; for locale
      `"en"` resolves the same base dir `load_all`/`resolve_help_directory` would
      pick. Annotate `// Validates: Requirement 6.1, 6.2, 10.2`.
      Then implement `pub fn resolve_help_directory_for_locale(&self, locale:
      &str) -> Option<PathBuf>`: for each `base` in `search_paths`, probe
      `base.join(locale)` first, then `base` itself; return the first existing.
      And `pub fn load_all_for_locale(&self, locale: &str) ->
      Result<ContentLoadResult, HelpError>` that resolves via the locale-aware
      method then reuses the EXISTING `discover_files` + `ContentParser` pipeline
      UNCHANGED. Keep `load_all()` as-is. New lines ASCII only.
      Files: crates/ff-help/src/content_loader.rs
      Verify: `cargo test -p ff-help resolve_help_directory_for_locale *> tools\logs\feat003-locale.log`
      then READ the log -- the three resolution tests pass; existing
      `resolve_help_directory_*` / `discover_files_*` tests still present.
      (content_loader.rs is ~261 lines; if the additions push non-test code over
      400, split the locale-aware fns into a `content_loader_locale.rs` sibling.)

- [ ] 8. Add the in-place file-topic replacement method to the registry,
      test-first.
      In `crates/ff-help/src/registry.rs` add a test FIRST: load a set of
      FileBased topics, then `replace_file_topics` with a different set, and
      assert the old FileBased topics are GONE and the new ones present, while a
      CommandRegistry/Plugin-sourced topic for an unrelated key survives.
      Annotate `// Validates: Requirement 6.4`.
      Then implement `pub fn replace_file_topics(&self, topics_to_load:
      Vec<HelpTopic>)`: take the write lock, retain only non-FileBased topics
      (drop entries whose `source()` is FileBased), then insert the new set via
      the same priority rules as `register_file_topic`. This mutates the shared
      inner map so both Arc holders observe it.
      Files: crates/ff-help/src/registry.rs
      Verify: `cargo test -p ff-help replace_file_topics *> tools\logs\feat003-replace.log`
      then READ the log -- the replace test passes.

- [ ] 9. Build the startup help registry for the active locale, via an extracted
      helper (keep construct.rs from growing).
      Factor the existing help-registry build block in
      `crates/ff-desktop/src/shell/construct.rs` (~lines 160-190) into a helper,
      e.g. `pub(crate) fn build_help_registry_for_locale(locale: &str) ->
      Arc<ff_help::HelpTopicRegistry>` in a new/existing `shell/help.rs`
      (or adjacent module), reusing the SAME `search_paths` construction and
      calling `loader.load_all_for_locale(locale)` instead of `load_all()`. In
      construct.rs read `ui.locale` from `config_handle`
      (`get_string(ff_config::keys::ui::LOCALE)`, default `"en"`) and call the
      helper. With `en` the resolved content is identical to today (Req 10.2).
      Files: crates/ff-desktop/src/shell/construct.rs,
             crates/ff-desktop/src/shell/help.rs (new helper module; register it
             in shell/mod.rs if new)
      Verify: `cargo check -p ff-desktop *> tools\logs\feat003-check.log` then
      READ the log -- compiles clean. Confirm construct.rs did not grow past 400
      non-test lines (it is large; the extraction should keep it flat or smaller).

- [ ] 10. Chain the help reload onto the existing ui.locale callback (Req 6.4).
      In `crates/ff-desktop/src/main.rs`, where `_locale_reload_handle` registers
      `on_locale_reload` (currently calls `i18n_bootstrap::install_catalogue`),
      capture a clone of the shell-held help-registry `Arc` and the config handle,
      and inside the SAME closure (or a second `on_locale_reload` registration)
      re-read `ui.locale`, call `load_all_for_locale` via the loader, and apply
      `replace_file_topics` on the registry Arc so subsequently displayed topics
      use the new locale. Keep it a Fn closure (Send + Sync + 'static) matching
      the existing handle. If the registry Arc is created inside shell
      construction rather than main.rs, expose it (e.g. a getter on the shell or
      construct returning the Arc) so main.rs can clone it for the closure --
      prefer the smallest surface that keeps ONE registry identity shared with
      the shell and its context panel.
      Files: crates/ff-desktop/src/main.rs (+ a minimal accessor if needed in
             shell/construct.rs or shell/mod.rs)
      Verify: `cargo check -p ff-desktop *> tools\logs\feat003-check.log` then
      READ the log -- compiles clean.

- [ ] 11. Add a focused hot-reload test on the extracted helper / registry seam.
      Where a full headless shell harness is impractical, target the extracted
      pieces: a test that builds a registry for a temp `help/` then calls the
      reload path for a temp `help/fr/` and asserts the topic set switches (via
      `build_help_registry_for_locale` + `replace_file_topics`, or the loader +
      registry directly). Annotate `// Validates: Requirement 6.4`. If the test
      must target the helper rather than the live shell, record WHY in the FEAT
      findings.
      Files: crates/ff-desktop/src/shell/help.rs (or wherever the helper lands)
      Verify: `cargo test -p ff-help replace_file_topics *> tools\logs\feat003-replace.log`
      and, if a ff-desktop helper test is added and fast,
      `cargo test -p ff-desktop build_help_registry *> tools\logs\feat003-helper.log`
      then READ the logs -- the switch test passes.

- [ ] 12. Final scoped gate for FEAT-003 + commit.
      Run, reading each log: `cargo check -p ff-help -p ff-desktop *> tools\logs\feat003-check.log`;
      `cargo clippy -p ff-help -p ff-desktop -- -D warnings *> tools\logs\feat003-clippy.log`
      (zero warnings); `cargo fmt -p ff-help -p ff-desktop -- --check *> tools\logs\feat003-fmt.log`
      (no diff). Confirm no touched `.rs` exceeds 400 non-test lines. Set FEAT-003
      status to `completed` and record findings. Commit locally (NEVER push):
      `git -C c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice1 add -- <changed files>`
      then a descriptive commit.
      Files: the FEAT-003 files above + features/FEAT-003.json
      Verify: the three logs are clean; `git -C <worktree> status` shows the
      intended files committed, no stray changes.

---

## Integration / convergence (after both FEATs)

- [ ] 13. Cross-FEAT compile verification.
      From the worktree root run `cargo check -p ff-i18n -p ff-help -p ff-desktop`
      and `cargo clippy -p ff-i18n -p ff-help -p ff-desktop -- -D warnings` and
      `cargo fmt --check`, each `*>` to a log read back. Fix any seam issues.
      Do NOT run the full suite -- the owner does that manually.
      Verify: all logs clean; no file over 400 non-test lines.

## Open assumptions (resolved by reasonable default, flag if wrong)

- The menu-name used for keys is the file stem of the backing `menus/<name>.toml`
  (`pom`, `settings`), lowercased -- consistent with `menu_name_label()` (which
  uppercases) and with the shipped `menu-pom-*` / `menu-settings-*` keys.
- Hot-reload mutates the shared registry in place via `replace_file_topics`
  rather than swapping the Arc, because the context panel holds its own clone of
  the Arc; swapping would desync the panel. If the shell is later refactored to
  hold a single `Arc<Mutex<Arc<..>>>` indirection, revisit.
