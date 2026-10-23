# CR-NR-109 Implementation Plan -- Locale-aware command resolution + menu-kind options via localised aliases

Source of truth: localization `requirements.md` Requirement 11 (10 criteria) and
the Req 4.2 / 7.3 amendment notes; `design.md` section "Locale-aware alias
resolution + menu-kind options via localised aliases (CR-NR-109, Requirement
11)"; `tasks.md` Phase 4 Task 11.1-11.7; `docs/quality/TCR.md` the 10 CR-NR-109
rows (Req 11.1-11.10, currently NOT COVERED).

All work happens in the worktree
`c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr109-locale-alias`. Paths
below are RELATIVE to that worktree root unless stated otherwise.

TDD is mandatory: write the failing test first, each carrying a
`// Validates: localization Requirement 11.x` annotation. Agents COMPILE-VERIFY
(`cargo check -p ff-desktop`, `cargo clippy -p ff-desktop -- -D warnings`,
`cargo fmt -p ff-desktop -- --check`) and run the single new SCOPED test where
quick (`cargo test -p ff-desktop <test_name>`); the OWNER runs the full gate /
suite. No source `.rs` file may exceed 400 non-test lines (excluding
`#[cfg(test)]`). ASCII-only in docs and pattern-matched source.

---

## STEP 0 (BLOCKING PRE-REQ) -- bring the approved gate artifacts into the worktree

Investigation finding (verified): the CR-NR-109 gate artifacts -- Requirement 11
and the Req 4.2 / 7.3 amendment notes in `requirements.md`, the design section
in `design.md`, the Phase 4 Task 11 list in `tasks.md`, and the 10 CR-NR-109
rows in `docs/quality/TCR.md` -- are present ONLY as UNCOMMITTED working-tree
edits in the MAIN checkout (`c:\workspace\apps\FileForgeWorkbench\...`). They are
ABSENT from this worktree (the worktree's `requirements.md` stops at Requirement
10 / out-of-scope; its `TCR.md` has no CR-NR-109 rows). The worktree HEAD and
`main` both point at commit `1d10e13`; the spec deltas live only in main's dirty
working tree.

This is NOT a re-gate. The gate is complete and owner-approved. STEP 0 only
MOVES the already-approved documentation into the worktree so the implementation
and its TCR updates have their source of truth locally.

- What to do: copy the four approved docs from the MAIN checkout into the
  worktree, preserving content byte-for-byte (docs are ASCII):
  - `docs/specs/localization/requirements.md` (adds Req 11 + the Req 4.2 / 7.3
    amendment notes)
  - `docs/specs/localization/design.md` (adds the CR-NR-109 design section)
  - `docs/specs/localization/tasks.md` (adds Phase 4 Task 11.1-11.7)
  - `docs/quality/TCR.md` (adds the 10 CR-NR-109 rows, Req 11.1-11.10)
  Then diff the worktree copies against the main copies to confirm Req 11, both
  amendment notes, the design section, the Phase 4 tasks, and all 10 TCR rows are
  present.
- Files: the four docs above in the worktree.
- Verify: `Select-String` (via the pwsh7 wrapper) for `Requirement 11` in the
  worktree `requirements.md` returns the heading; for `CR-NR-109` in the worktree
  `TCR.md` returns 10 rows; for the CR-NR-109 design heading in `design.md`
  returns one hit. (If the owner prefers the worktree branch be rebased onto a
  commit that already carries these docs, that substitutes for the copy -- the
  requirement is only that the worktree contains the approved spec before code
  starts.)

If STEP 0 cannot be completed (the main-checkout content is unavailable), STOP
and surface it -- do not reconstruct the spec from this plan.

---

## Architecture findings that shape the plan (verified in code)

1. `AliasTable` (`crates/ff-desktop/src/shell/environment.rs`) stores
   `entries: Vec<(String, String)>` = `(surface-UPPERCASED, canonical)`. Its only
   constructor is `ffedit_english()`; `canonical_verb(surface)` and
   `apply_locale_overlay(env_name, rows)` exist. The file is ~300 lines
   production (well under 400) -- NO split needed.
2. `apply_locale_overlay` + its `AliasOverlayError::Collision` discipline and the
   `AliasCatalogue` in-memory type already exist in
   `crates/ff-desktop/src/shell/alias_overlay.rs` (whole module is
   `#![allow(dead_code)]` -- NO production caller yet). This file is ~170
   production lines -- NO split needed. Task 11.2's reader is its FIRST production
   caller.
3. CRITICAL: the `AliasTable` is NOT currently live in dispatch. `ffedit_english()`
   is called ONLY from tests; `EnvironmentRegistry`
   (`crates/ff-desktop/src/shell/environment_registry.rs`) holds NO `AliasTable`.
   There is NO `ffcmd_english()` constructor. So the resolver (11.4) and the menu
   render (11.5) have NO existing live per-environment alias table to read. The
   plan therefore introduces a shell-owned, locale-overlaid alias store
   (populated by the 11.2 reader + 11.3 synthesis) that both entry points consult.
4. The `ff_i18n::t(key)` seam (`crates/ff-i18n/src/seam.rs`) resolves against the
   process-wide active catalogue; the locale is implicit in the installed
   catalogue (swapped on `ui.locale` reload). "Scoped to localisation key L" is
   realised as: the active locale's overlay rows carry `localisation_key`, and the
   displayed/typeable surface for such a row is `t(key)` under the active
   catalogue. No per-row locale tag (localization Req 5.4).
5. The `ui.locale` hot-reload already re-runs `install_catalogue`
   (`crates/ff-desktop/src/shell/i18n_bootstrap.rs`) and reloads help
   (`crates/ff-desktop/src/shell/help.rs::reload_help_registry_for_locale`) via a
   callback registered in `crates/ff-desktop/src/main.rs`. Task 11.2 adds the
   alias-catalogue reload onto the SAME callback / the same construction seam -- no
   new mechanism (localization Req 1.5).
6. Command-line entry: `run_command_line`
   (`crates/ff-desktop/src/shell/commands_line.rs`) -> `dispatch_command_string`
   (`crates/ff-desktop/src/shell/dispatch.rs`). The `=` strip runs first, then
   the prelude, then `resolve_target`, then the ladder. The unified resolver
   (11.4) runs at the TOP of `dispatch_command_string` (after the `=` strip,
   BEFORE the prelude/`resolve_target`), rewriting the entered verb token to its
   ROOT and then letting the UNCHANGED front door dispatch the root.
7. Menu-selection entry: a click sets `pending_menu_option`
   (`crates/ff-desktop/src/shell/render_body_arms.rs`); `update.rs` drains it via
   `activate_menu_option` (`crates/ff-desktop/src/shell/commands_menu.rs`), which
   (absent an inline target) calls `resolve_and_dispatch_command` ->
   `handle_command`. 11.4 routes the option's ROOT `command` through the SAME
   `resolve_locale_verb` before dispatch so a clicked option and a typed verb
   take one path.
8. Menu render: `render_menu_workspace`
   (`crates/ff-desktop/src/menu_workspace/render.rs`) is the ONE renderer for all
   menu kinds. It paints `option.command` via `option_prefix_job` and localises
   only the description via `i18n_keys::localised_option_description`. MEASURED:
   production code runs to line ~492 (the `// === Tests` separator is at line 494,
   `#[cfg(test)]` at 496); the only `#[cfg(test)]`-gated production item is
   `format_option_row` (~10 lines). Net non-test ~485 lines => render.rs is ALREADY
   OVER the 400-line limit and MUST be split BEFORE Task 11.5 grows it (see Task
   11.5 step A).
9. A menu-kind Context resolves to the FFCMD environment
   (`active_name(KindTag::MenuWorkspace, true) == FFCMD_NAME`). Menu option roots
   (CATALOGS, FILES, SETTINGS, ...) are NOT FFEDIT verbs, so the FFCMD alias table
   the menu render consults is built from the 11.3 synthesised identity rows
   (one per menu-option root / registered root command) plus any af overlay.

### SEQUENCING CAVEAT (call-out, affects Task 11.7 end-to-end exercise)

Req 11 aliases a localised SURFACE to a root verb and resolves the DISPLAYED
surface through `ff_i18n::t(localisation_key)`. VERIFIED: there are currently NO
verb-level alias keys (e.g. `verb-find`) in ANY `.ftl` catalogue under
`crates/ff-i18n/i18n/` -- only UI-chrome / menu-title / option-description keys
exist, because the full ff-desktop string extraction (localization L10N.4 / Task
10.x) is not complete. Consequences:

- The MECHANISM (entry model, reader, synthesis, resolver, render refactor,
  behaviour-neutral `en` baseline) is fully testable NOW: unit tests install a
  small test catalogue (as `tests_common` already does) that defines the one or
  two verb keys the test needs; the full-shell `egui_kittest` test (11.6) installs
  a catalogue carrying the af key for its one option.
- The af POC DATA (11.7) can be exercised END-TO-END only if the matching verb
  key(s) exist in BOTH the `en` Identity_Base and the `af` catalogue. Task 11.7
  MUST therefore ALSO author the small set of `verb-*` keys it uses (at minimum
  `verb-find` = `FIND` in `en`, `verb-find` = `SOEK` in `af`) in the existing
  `.ftl` files under `crates/ff-i18n/i18n/en/` and `crates/ff-i18n/i18n/af/`. This
  is NOT the full extraction (out of scope, localization "Out of Scope"); it is
  the minimal key set the POC alias rows reference. With those keys present, the
  af POC exercises end-to-end; without them, `t(key)` returns the key string and
  the surface falls back to the root (still correct per Req 11.8, just not
  localised).

---

## Task 11.1 -- Grow the `AliasTable` entry to carry an optional localisation key

Satisfies Req 11.1, 11.2; behaviour-neutral at `en` (Req 10.2). Turns TCR rows
Req 11.1 and Req 11.2 green.

- Split needed? NO. `environment.rs` is ~300 production lines.
- Design decision: change `entries: Vec<(String, String)>` to
  `entries: Vec<AliasEntry>` where
  `struct AliasEntry { surface: String, canonical: String, localisation_key: Option<String> }`
  (private to the module). Keep `canonical_verb` and `apply_locale_overlay`
  signatures as they are for the existing `(surface, canonical)` row callers by
  constructing `AliasEntry { localisation_key: None, .. }` internally, so the
  existing `apply_locale_overlay(&[(String,String)])` callers and all current
  tests remain unchanged and byte-identical at `en`. Add an overloaded/extended
  overlay entry point that carries the key (used by the 11.2 reader):
  `apply_locale_overlay_keyed(env, rows: &[(String, String, Option<String>)])`
  (or extend the row tuple), reusing the SAME collision machinery.
- Failing test first (in `environment.rs` `#[cfg(test)]`):
  `display_surface_resolves_localisation_key_else_raw_surface`
  `// Validates: localization Requirement 11.1, 11.2` -- install a tiny test
  catalogue (reuse `i18n_bootstrap::install_catalogue_for_tests` or set a
  catalogue with a single `verb-find = SOEK` key), build a table with an entry
  `(surface="FIND", canonical="FIND", localisation_key=Some("verb-find"))`, and
  assert the new `display_surface(&entry)` accessor returns `"SOEK"` (the `t(key)`
  resolution) while an entry with `localisation_key=None` returns its raw surface.
  Also assert `canonical_verb` for the typeable localised surface ("SOEK")
  resolves to "FIND" once the keyed overlay is applied (Req 11.2 one source of
  truth).
- Minimum implementation: the `AliasEntry` struct; a `display_surface(&self,
  entry) -> String` (or `display_surface_for(surface) -> String`) accessor
  returning `ff_i18n::t(key)` when `localisation_key` is `Some`, else the raw
  `surface`; the keyed overlay entry point; `ffedit_english()` builds entries with
  `localisation_key: None` (byte-identical `en`).
- Verify: `cargo test -p ff-desktop display_surface_resolves_localisation_key`
  passes; the pre-existing `alias_overlay` / `environment` tests still pass;
  `cargo check/clippy/fmt -p ff-desktop` clean.

## Task 11.2 -- On-disk alias-catalogue reader + wiring

Satisfies Req 11.5, 11.10; unblocks localization Task 10.2 / 10.3. Turns TCR rows
Req 11.5 and Req 11.10 green.

- Split needed? NO new split for `alias_overlay.rs` (~170 lines). Put the reader
  in a NEW sibling module `crates/ff-desktop/src/shell/alias_reader.rs` (keeps
  `alias_overlay.rs` as the pure in-memory loader and stays well under 400 lines);
  register `mod alias_reader;` in `crates/ff-desktop/src/shell/mod.rs`.
- Design decision: parse `i18n/<locale>/aliases/<environment>.toml` with `[[alias]]`
  rows `{ surface, canonical, localisation_key? }` using the `toml` crate (already
  a dependency of the menu loader) into `AliasCatalogue` (extended in 11.1 to carry
  the optional key per row). Resolve the `i18n/` root with the SAME resolver the
  catalogue bootstrap uses (`i18n_bootstrap::resolve_i18n_root`, made
  `pub(crate)` if not already) so alias data sits beside the `.ftl` tree under
  `crates/ff-i18n/i18n/`. Load each environment's rows via the EXISTING
  `AliasTable::apply_locale_overlay_keyed`; a within-environment collision returns
  `Err` and the reader logs a WARN and RETAINS the Identity_Base for that
  environment (Req 11.10), other environments still load.
- Wiring: establish the shell-owned, per-environment alias store (the home the
  resolver/render read). Add a grouped field (NOT a flat field on the god-struct)
  e.g. `shell.locale_aliases: LocaleAliasStore` holding the overlaid
  `AliasTable` per environment name (at least FFCMD and FFEDIT). Populate it in
  shell construction (`crates/ff-desktop/src/shell/construct.rs`) by seeding
  identity rows (11.3) then applying the active locale's on-disk overlay, and
  re-populate it from the `ui.locale` reload path (extend the callback wired in
  `main.rs` / the construction seam, mirroring the help-registry reload in
  `help.rs`). Behaviour-neutral at `en` (no `i18n/en/aliases/*` required; absence
  => identity-only table).
- Failing tests first (in `alias_reader.rs` `#[cfg(test)]`):
  - `reader_parses_toml_rows_into_catalogue`
    `// Validates: localization Requirement 11.5` -- write a temp
    `aliases/FFEDIT.toml` with two rows (one with a `localisation_key`, one
    without), parse it, assert the resulting `AliasCatalogue.rows_for("FFEDIT")`
    carries both rows with the key preserved, and that loading into an
    `AliasTable::ffedit_english()` makes the localised surface resolve to its
    root via `canonical_verb`.
  - `reader_rejects_within_environment_collision_and_retains_base`
    `// Validates: localization Requirement 11.10` -- a TOML with a surface
    mapping to two roots is rejected (reader returns/logs the diagnostic) and the
    base table is unchanged.
- Minimum implementation: the TOML row structs (`serde::Deserialize`), the
  parse+load function, the `LocaleAliasStore` type and its populate/reload method,
  and the construction + reload wiring.
- Verify: `cargo test -p ff-desktop reader_parses_toml_rows` and
  `reader_rejects_within_environment_collision` pass; `cargo check/clippy/fmt -p
  ff-desktop` clean.

## Task 11.3 -- Auto-synthesised, editable identity rows

Satisfies Req 11.3, 11.4. Turns TCR rows Req 11.3 and Req 11.4 green.

- Split needed? NO.
- Design decision: when building a per-environment `AliasTable` for the store
  (11.2), SEED an identity row `(surface==canonical, localisation_key=None)` for
  every root command of that environment BEFORE applying the on-disk overlay. The
  root-command source per environment:
  - FFEDIT: the canonical verbs already in `ffedit_english()`.
  - FFCMD (menu-kind Context env): the set of menu-option root commands plus any
    registered root verbs the menu kinds use. Derive the FFCMD identity set from
    the parsed built-in menus (`DEFAULT_*_TOML` option `command` values) and the
    command registry (`cmd_registry` / `command_store.definitions`) so every root
    a menu option can carry is resolvable. (A new `AliasTable::identity_for(roots:
    &[&str])` constructor builds an identity-only table from a root list.)
  An authored/overlaid row for the SAME canonical REPLACES the synthesised
  identity default: the overlay apply already updates an existing surface's row,
  and for the Russian-root case the overlay row keeps `canonical=<russian-root>`
  with `surface="FIND"` + `localisation_key="verb-find"`, leaving the ROOT
  unchanged.
- Failing tests first (in `alias_reader.rs` or `environment.rs` tests):
  - `synthesised_identity_row_exists_for_every_root`
    `// Validates: localization Requirement 11.3` -- build the FFCMD identity
    table from a root list `["CATALOGS","FILES","SETTINGS"]`; assert each resolves
    to itself via `canonical_verb`.
  - `overlay_row_replaces_synthesised_default_without_changing_root`
    `// Validates: localization Requirement 11.4` -- seed identity row for a
    (fictional Russian) root `NAYTI`; apply an `en` overlay row
    `(surface="FIND", canonical="NAYTI", localisation_key="verb-find")`; assert
    `canonical_verb("FIND") == Some("NAYTI")` (English surface reaches the Russian
    root), the root `NAYTI` still resolves to itself, and the displayed surface for
    the row is `t("verb-find")`. (Install a test catalogue with `verb-find = FIND`
    so the display assert is meaningful.)
- Minimum implementation: `AliasTable::identity_for(roots)`; the FFCMD root
  derivation helper reading the built-in menus + registry; overlay-replaces-default
  behaviour (confirm/extend `apply_locale_overlay_keyed` so a row whose surface
  equals an existing identity surface but introduces a key updates that row rather
  than erroring -- this is NOT a collision because the canonical is unchanged or
  the surface differs).
- Verify: both scoped tests pass; existing overlay tests still pass; clean
  check/clippy/fmt.

## Task 11.4 -- Unified locale-aware resolver `resolve_locale_verb(V, L)`

Satisfies Req 11.6, 11.9, 10.3. Turns TCR rows Req 11.6 and Req 11.9 green.

- Split needed? NO. `dispatch.rs` is ~120 production lines. Put the resolver in a
  NEW sibling `crates/ff-desktop/src/shell/locale_resolver.rs` (keeps dispatch.rs
  as the thin router) and register the module; call it from dispatch.rs and
  commands_menu.rs.
- Design decision: `resolve_locale_verb(&self, verb: &str, env_name: &str) ->
  LocaleResolveOutcome` where the active locale is implicit (the overlaid store +
  active catalogue). Order (identical for both entry points):
  1. look up `verb` (uppercased) in the store's `AliasTable` for `env_name` via
     `canonical_verb` -- this table already carries the active locale's overlay
     surfaces, so a localised surface (SOEK) resolves to its root (FIND) ->
     return `Root(canonical)` (Req 11.6.1.1);
  2. else look up `verb` in the root command table (the registry /
     `is_root_command`) -> return `Root(verb)` (Req 11.6.1.2.1);
  3. else return `Unknown(verb)` (Req 11.6.1.2.2) -- the caller issues the
     existing unknown-command message.
  Because step 1's table ALSO holds the synthesised identity rows (11.3), a bare
  English root entered under a localised locale still resolves (Req 11.6/5.5). The
  returned ROOT is what the UNCHANGED front door dispatches and what
  history/session/macro record (Req 11.9, 10.3) -- the resolver only REWRITES the
  verb token to its root; it never dispatches.
- Call sites:
  - `dispatch_command_string` (dispatch.rs): after the `=` strip, split the raw
    into verb + argument (reuse `shell::helpers::split_verb_arg`), run
    `resolve_locale_verb(verb, active_env_name)`, and if it returns a root
    DIFFERENT from the entered surface, reassemble `root + arg` and continue the
    EXISTING prelude/`resolve_target`/ladder path with the root; on `Unknown`, let
    the existing not-found path run unchanged. Behaviour-neutral at `en`/no overlay
    (verb == root).
  - `activate_menu_option` (commands_menu.rs): the option already carries the ROOT
    `command`; feed it through `resolve_locale_verb` for symmetry (root in -> same
    root out) so the single-path property holds, then dispatch as today.
- Failing tests first (table-driven, in `locale_resolver.rs` `#[cfg(test)]`):
  `resolve_locale_verb_follows_the_req_11_6_order`
  `// Validates: localization Requirement 11.6` covering: localised surface ->
  root (1.1); bare root -> root (1.2.1); unknown -> `Unknown` (1.2.2); and
  `root_command_is_returned_regardless_of_surface`
  `// Validates: localization Requirement 11.9, 10.3` asserting the returned value
  is always the ROOT (e.g. SOEK and FIND both -> FIND). Build the store with an af
  overlay (SOEK->FIND) for the FFCMD/FFEDIT env under test.
- Minimum implementation: `LocaleResolveOutcome` enum; `resolve_locale_verb`; the
  two call-site hooks; a small `is_root_command` check against the registry.
- Verify: scoped resolver tests pass; the existing `typed_submit_routes_through_
  dispatch_command_string` and command tests still pass; clean check/clippy/fmt.

## Task 11.5 -- Menu-kind render refactor (command column shows the locale surface)

Satisfies Req 11.7, 11.8, 4.2-amended, 7.3-amended. Turns TCR row Req 11.7 green
(Req 11.8 shares coverage with 11.6).

- Split needed? YES -- `menu_workspace/render.rs` is ~485 non-test lines (OVER
  400) and Task 11.5 adds to it. Split FIRST as a pure REFACTOR (no behaviour
  change, no gate): move the pure measurement/layout helpers
  (`MenuColours`, `command_column_width`, `format_option_row`,
  `option_font`, `option_prefix_job`, `option_prefix_text`,
  `natural_width_from_rows`, `natural_option_list_width`, the width constants) and
  their unit tests into a new `crates/ff-desktop/src/menu_workspace/render_layout.rs`;
  keep `render_menu_workspace`, `MenuRenderResult`, and the frame render loop in
  `render.rs`. Register `mod render_layout;` (or `pub(crate)` re-exports) in
  `menu_workspace/mod.rs` and fix imports. Confirm BOTH files land under 400
  non-test lines. Verify the split alone compiles and the moved tests pass BEFORE
  adding 11.5 behaviour.
- Design decision: add `i18n_keys::localised_option_command(environment, option,
  locale_store)` mirroring `localised_option_description`: resolve the alias
  surface for the option's ROOT command in the active locale via the shell's
  `LocaleAliasStore` (the overlaid FFCMD `AliasTable`'s `display_surface_for(root)`),
  falling back to the option's root `command` string when no localised alias
  exists (Req 11.7, 11.8). `render_menu_workspace` calls it where
  `option_prefix_job` currently formats `option.command`. Because the store is
  shell-owned, thread the needed data in: either pass the resolved display verb
  into `render_menu_workspace` (compute it in `render_body_arms.rs` where `&self`
  is available, building a `Vec`/map of `root -> display surface` for the menu's
  options) OR pass a borrow of the FFCMD alias table. Preferred: compute the
  per-option display surface in `render_body_arms.rs` (shell side) and pass it
  through `MenuWorkspaceState` / a render arg, keeping `render_menu_workspace`
  free of shell coupling. Selecting an option STILL yields `MenuOption` with the
  ROOT `command` (unchanged), fed through 11.4's resolver (Req 11.9).
- Failing test first: a unit test for the helper in `i18n_keys.rs`
  `localised_option_command_resolves_alias_surface_else_root`
  `// Validates: localization Requirement 11.7, 11.8` -- with a store carrying
  SOEK->FIND (key `verb-find`) for FFCMD and a test catalogue defining
  `verb-find = SOEK`, assert the helper returns "SOEK" for a FIND option, and
  returns the raw root (e.g. "CATALOGS") when no alias/key exists (fallback).
- Minimum implementation: the split; the `localised_option_command` helper; the
  render call-site change (command column uses the display surface); the shell-side
  wiring in `render_body_arms.rs` to compute the surfaces. The selection path is
  already root-carrying; no change to persistence.
- Verify: `cargo test -p ff-desktop localised_option_command_resolves_alias_surface`
  passes; the moved layout tests pass; clean check/clippy/fmt.

## Task 11.6 -- Full-shell `egui_kittest` behaviour test + behaviour-neutral baseline

Satisfies Req 11.7, 11.9, 10.2. Turns TCR rows Req 11.8 and Req 11.9 green (shared
with 11.4/11.5), and anchors the Req 10.2 behaviour-neutral baseline.

- Split needed? NO (test file). Add tests to the full-shell harness area
  (`crates/ff-desktop/src/shell/tests_command.rs` or a new
  `tests_locale_menu.rs` module registered under the shell test tree), using
  `harness_shell()` / `build_eframe`.
- Failing tests first:
  - `full_shell_menu_option_shows_localised_surface_and_dispatches_root`
    `// Validates: localization Requirement 11.7, 11.9` -- install a test catalogue
    + alias store with a non-`en` locale overlay (af: SOEK->FIND, key verb-find,
    on a menu that carries a FIND option; or reuse the POM with a seeded option);
    run the harness, assert the option's command column galley/text shows the
    localised surface (SOEK), then activate it and assert the recorded command /
    history entry is the ROOT (FIND), not the surface.
  - `full_shell_menu_option_en_shows_root_verb_unchanged`
    `// Validates: localization Requirement 10.2` -- with `en` / no overlay, the
    command column shows the root verb exactly as today and dispatch is unchanged.
  - Keep the EXISTING menu-kind first-Tab focus `egui_kittest` test GREEN (do not
    modify the focus contract; `workspace-conformance.md`). Re-run it as part of
    verification.
- Minimum implementation: test-only catalogue + store setup helpers (extend
  `tests_common.rs`); assertions read the shell's recorded command / history. No
  production change beyond what 11.1-11.5 added.
- Verify: `cargo test -p ff-desktop full_shell_menu_option` passes; the existing
  menu first-Tab focus test still passes; clean check/clippy/fmt. (The OWNER runs
  the full `egui_kittest` suite.)

## Task 11.7 -- Author the Afrikaans (`af`) POC alias data + minimal verb keys

Satisfies Req 5.1, 5.5, 11.1. Keeps the Req 11 rows' end-to-end story real.

- Split needed? NO (data).
- Design decision: author `crates/ff-i18n/i18n/af/aliases/<environment>.toml`
  (at least FFCMD, and FFEDIT if an editor verb is in the POC) with reviewed
  Afrikaans rows, e.g.:
  ```toml
  [[alias]]
  surface = "SOEK"
  canonical = "FIND"
  localisation_key = "verb-find"
  ```
  Per the SEQUENCING CAVEAT, ALSO author the minimal `verb-*` keys the POC rows
  reference in the EXISTING `.ftl` files so the surface resolves end-to-end:
  `verb-find = FIND` in `crates/ff-i18n/i18n/en/<file>.ftl` (Identity_Base) and
  `verb-find = SOEK` in `crates/ff-i18n/i18n/af/<file>.ftl`. Place them in a
  focused catalogue file (e.g. a new `ff-desktop-verbs.ftl`) consistent with the
  existing per-area `.ftl` convention; register it with the catalogue loader if
  the loader enumerates files explicitly. fr/de alias data are OUT of scope for
  this run (localization Task 10.2/10.3 DATA work).
- Failing test first: a reader test
  `af_poc_alias_catalogue_loads_and_resolves`
  `// Validates: localization Requirement 11.1, 5.1, 5.5` -- load the shipped
  `af/aliases/FFCMD.toml` via the 11.2 reader into the FFCMD identity table and
  assert SOEK resolves to FIND, FIND still resolves to FIND (Identity_Base
  retained, Req 5.5), and the display surface for the row is `t("verb-find")` =
  "SOEK" under an `af` catalogue.
- Minimum implementation: the `.toml` data file(s) + the `verb-find` keys in
  `en`/`af` `.ftl`.
- Verify: `cargo test -p ff-desktop af_poc_alias_catalogue_loads` passes; the
  `ftl_locale_check` tool (if run) still reports key parity; clean check/clippy/fmt.

---

## Ordering and dependencies

STEP 0 (docs into worktree) -> 11.1 (entry model) -> 11.2 (reader + store wiring)
-> 11.3 (identity synthesis; depends on the store from 11.2) -> 11.4 (resolver;
reads the store) -> 11.5 (menu render; split render.rs first, reads the store) ->
11.6 (full-shell test; exercises 11.4+11.5) -> 11.7 (af data + verb keys; proven
by the reader test and the 11.6 full-shell test).

Each task leaves the crate compiling, lint/format clean, with its scoped new test
passing. The OWNER runs the authoritative full `cargo gate` / `egui_kittest`
suite.

## TCR row mapping (all currently NOT COVERED -> target PASS)

- Req 11.1 -> Task 11.1 (and exercised by 11.7)
- Req 11.2 -> Task 11.1
- Req 11.3 -> Task 11.3
- Req 11.4 -> Task 11.3
- Req 11.5 -> Task 11.2
- Req 11.6 -> Task 11.4
- Req 11.7 -> Task 11.5 (+ full-shell 11.6)
- Req 11.8 -> Task 11.6 (+ helper fallback in 11.5)
- Req 11.9 -> Task 11.4 (+ full-shell 11.6)
- Req 11.10 -> Task 11.2

Update `docs/quality/TCR.md` rows to PASS with the test name as evidence as each
task completes.
