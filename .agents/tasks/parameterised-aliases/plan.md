# Implementation Plan -- CR-NR-112 Parameterised Command Aliases

Implements command-semantics Requirement 14 (14.1-14.10) and command-framework
Requirement 19 (19.1-19.6). Approved, gated change request; the gate is NOT
re-opened here. All work is in the worktree:

`c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr112-parameterised-aliases`

Use ABSOLUTE paths. Step cwd defaults to the MAIN workspace, so always pass
`-C <absolute worktree>` to git and target the worktree crate paths below.

## Design decisions (grounded in the worktree source; not re-deciding the gate)

- The extended alias record is an ADDITIVE `command_template: Option<String>` on
  `AliasEntry` (`crates/ff-desktop/src/shell/environment.rs`) and an additive
  `#[serde(default)] command_template` on the on-disk row `AliasRowToml`
  (`crates/ff-desktop/src/shell/alias_reader.rs`). Absent/empty deserialises to
  `None`; `None`/empty preserves today's pure verb-rewrite byte-for-byte
  (Req 14.6, 19.3). The existing `ffedit_english()` and `identity_for()` seed rows
  set the field to `None`.
- The substitution grammar + algorithm live in a NEW FOCUSED MODULE
  `crates/ff-desktop/src/shell/alias_template.rs` with a single pure function
  `expand_template(root: &str, template: &str, arg: &str) -> String` and its own
  `#[cfg(test)]` tests. Pure, deterministic, no I/O, no shell state. This isolates
  the one piece of genuinely new logic and keeps every touched file under the
  400-line rule.
- The template value is carried through the overlay machinery by widening the
  `KeyedAliasRow` tuple from `(surface, canonical, Option<key>)` to
  `(surface, canonical, Option<key>, Option<template>)` in
  `crates/ff-desktop/src/shell/alias_overlay.rs`. This is the single
  change-pattern that ripples to the few call sites that construct or destructure
  that tuple (`alias_reader.rs`, `environment.rs::apply_locale_overlay_keyed`,
  the keyless shim `apply_locale_overlay`, and `overlay_row_for_test`). Rationale:
  a tuple widening is the smallest additive change that threads the field with no
  new type; the keyless `apply_locale_overlay` path passes `None` for both the
  key and the template, so the pre-CR-NR-109 keyless tests stay byte-identical.
- The pre-pass `rewrite_verb_to_root` (`crates/ff-desktop/src/shell/locale_resolver.rs`)
  is extended to look up the matched `AliasEntry` (not just its canonical verb)
  via a new `AliasTable::resolve_alias_entry(surface) -> Option<&AliasEntry>`
  accessor. When the entry carries a non-empty template it returns
  `expand_template(entry.canonical, template, arg)`; otherwise it returns today's
  verb-only result unchanged. `canonical_verb` is RETAINED (menu display + other
  callers still use it). Single-pass: the returned string is NOT re-fed
  (Req 14.7, 19.5). Both call sites (`dispatch.rs` typed line, `commands_menu.rs`
  menu/key seam) already call `rewrite_verb_to_root`, so NEITHER call site changes
  shape (Req 19.1, 19.2, 19.6).
- `resolve_locale_verb` / `LocaleResolveOutcome` are verb-only and stay as-is for
  the menu display path; the template expansion is added to `rewrite_verb_to_root`
  only, which is the string-rewrite seam both execution call sites use.
- Forward-compat guards (max expansion DEPTH limit, VISITED-SET of alias
  surfaces) are DOCUMENTED in the `alias_template.rs` and `locale_resolver.rs`
  doc comments as the additive path to a future chaining CR. They are NOT wired
  (v1 is single-pass, no loop) -- see Req 14.7 / 19.5.
- All authored source is plain ASCII with ASCII section separators
  (`// === ... ===`). Template VALUES in TOML are DATA (out of ASCII scope).

## Verification commands (discovered from the worktree)

Agents prove COMPILE + LINT + FORMAT clean, scoped to the touched crate, and run
the SCOPED new tests where quick. The OWNER runs the full gate / whole suite.
Capture each to a log and READ it back (terminal-discipline.md); verify by EFFECT,
not the reported exit code. Run each as its OWN wrapped single-job command:

- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo check -p ff-desktop *> tools\logs\crnr112-check.log"` (cwd = worktree root)
- `cargo clippy -p ff-desktop -- -D warnings *> tools\logs\crnr112-clippy.log`
- `cargo fmt -p ff-desktop -- --check *> tools\logs\crnr112-fmt.log`
- Scoped unit tests for a module, quick: `cargo test -p ff-desktop alias_template *> tools\logs\crnr112-tmpl.log` (and similarly by test-name filter). If a scoped test risks a long/quiet run, write it, compile-verify, and leave the suite run to the owner.

---

## Ordered implementation items

- [ ] 1. Add the focused substitution module `alias_template.rs` with `expand_template` + tests (TDD first).
      Create `crates/ff-desktop/src/shell/alias_template.rs`: `pub(crate) fn expand_template(root: &str, template: &str, arg: &str) -> String` implementing the Req 14 algorithm (pre-scan highest referenced `&N`; left-to-right scan; `&N` = Nth whitespace token 1-based, longest-digit-run so `&12` is token 12; `&*` = tokens after the highest referenced `&N` joined by single spaces, or ALL tokens when no `&N`; unsupplied `&N` -> empty; literal text incl. bare `&` copied verbatim; surplus-append of tokens after the highest referenced `&N` when NO `&*`; result `format!("{root} {out}")`). Document the forward-compat depth-limit + visited-set guards in the module doc comment (NOT wired). Register the module in `crates/ff-desktop/src/shell/mod.rs`. Write `#[cfg(test)]` tests FIRST (red), each annotated `// Validates: ...`, covering every design.md worked-example row and edge case: `ecob helloW` -> `EDIT FFWB.COBOL.MASTER(helloW)` (14.2, 14.9); `ecob` (empty arg) -> `EDIT FFWB.COBOL.MASTER()` (14.4); `ecob a b` -> `EDIT FFWB.COBOL.MASTER(a) b` surplus append (14.5); `&1 TO &*` with `[a,b,c]` -> `... a TO b c` no repetition (14.3); no-`&N` template with `&*` -> all tokens (14.3); two-digit `&12` (14.3); bare `&` literal + no-placeholder literal copy (14.2); whitespace tokenisation only (14.10).
      Files: `crates/ff-desktop/src/shell/alias_template.rs`, `crates/ff-desktop/src/shell/mod.rs`
      Verify: `cargo test -p ff-desktop alias_template *> tools\logs\crnr112-tmpl.log` -- all new tests pass; read the log.

- [ ] 2. Add the `command_template` field to `AliasEntry` and the entry accessor on `AliasTable`.
      In `crates/ff-desktop/src/shell/environment.rs`: add `pub(crate) command_template: Option<String>` to `AliasEntry` (derive `Debug, Clone, PartialEq, Eq` already present), doc-commented as the optional parameter template (None/empty = pure verb alias). Set it to `None` in every seed construction (`ffedit_english`, `identity_for`) and in `apply_locale_overlay_keyed` row construction (updated in item 3). Add `pub(crate) fn resolve_alias_entry(&self, surface: &str) -> Option<&AliasEntry>` mirroring `canonical_verb`'s case-insensitive match but returning the whole entry. Keep `canonical_verb` unchanged. Add `#[cfg(test)]` tests: a `None`-template entry round-trips; `resolve_alias_entry` finds the entry case-insensitively and returns `None` for an unknown surface.
      Files: `crates/ff-desktop/src/shell/environment.rs`
      Verify: `cargo test -p ff-desktop environment:: *> tools\logs\crnr112-env.log` -- existing + new tests pass; read the log.

- [ ] 3. Widen `KeyedAliasRow` to carry the template and thread it through the overlay machinery.
      In `crates/ff-desktop/src/shell/alias_overlay.rs`: change `pub(crate) type KeyedAliasRow = (String, String, Option<String>);` to `(String, String, Option<String>, Option<String>)` (surface, canonical, localisation_key, command_template); update the doc comment. In `crates/ff-desktop/src/shell/environment.rs::apply_locale_overlay_keyed`, destructure the 4th element and set `command_template` on the created/updated `AliasEntry` (an update also refreshes the template). In the keyless shim `apply_locale_overlay`, map each `(s,c)` to `(s, c, None, None)`. Fix the `overlay_row_for_test` helper (`alias_reader.rs`) and any `with_environment_keyed` test rows to the 4-tuple (template `None` unless the test needs one). Update the existing keyed overlay unit tests in `environment.rs`/`alias_overlay.rs` to the 4-tuple shape (template `None`), asserting no behaviour change. Add one new test: a keyed overlay row WITH a template produces an `AliasEntry` whose `command_template` is `Some(...)` and `canonical_verb`/`resolve_alias_entry` still resolve.
      Files: `crates/ff-desktop/src/shell/alias_overlay.rs`, `crates/ff-desktop/src/shell/environment.rs`, `crates/ff-desktop/src/shell/alias_reader.rs`
      Verify: `cargo test -p ff-desktop alias_overlay *> tools\logs\crnr112-overlay.log` and `cargo test -p ff-desktop alias_reader *> tools\logs\crnr112-reader.log` -- pass; read both logs.

- [ ] 4. Parse the on-disk `command_template` field in the TOML reader (loader round-trip).
      In `crates/ff-desktop/src/shell/alias_reader.rs`: add `#[serde(default)] command_template: Option<String>` to `AliasRowToml`; in `parse_alias_toml` carry it through into the 4-tuple `KeyedAliasRow` (trim; map empty to `None`, mirroring the `localisation_key` treatment). Add `#[cfg(test)]` tests FIRST: a row WITHOUT `command_template` parses to template `None` (back-compat, 14.6); a row WITH `command_template = "FFWB.COBOL.MASTER(&1)"` parses to `Some("FFWB.COBOL.MASTER(&1)")` with surface/canonical uppercased and template preserved verbatim (14.1).
      Files: `crates/ff-desktop/src/shell/alias_reader.rs`
      Verify: `cargo test -p ff-desktop alias_reader *> tools\logs\crnr112-reader2.log` -- new loader round-trip tests pass; read the log.

- [ ] 5. Extend `rewrite_verb_to_root` to expand a template entry in the existing pre-pass (single-pass).
      In `crates/ff-desktop/src/shell/locale_resolver.rs`: after `split_verb_arg`, resolve the matched `AliasEntry` via the active environment table's `resolve_alias_entry` (add a small `resolve_alias_entry_for(verb, env_name)` helper alongside `resolve_locale_verb`, reading under the store lock, mirroring its structure). If the entry has a non-empty `command_template`, return `super::alias_template::expand_template(entry.canonical, template, arg)`. Otherwise preserve TODAY's exact behaviour: when the resolved root differs from the surface return `root` (empty arg) or `format!("{root} {arg}")`, else return `raw` unchanged (14.6, 19.3). The returned string is NOT re-submitted (single-pass, 14.7/19.5). Document that both call sites (`dispatch.rs`, `commands_menu.rs`) are unchanged in shape. Add `#[cfg(test)]` tests driving `rewrite_verb_to_root` through a `make_shell()` with a template alias overlaid (via `overlay_row_for_test` extended, or a direct keyed overlay): `ecob helloW` -> `EDIT FFWB.COBOL.MASTER(helloW)`; `ecob` -> `EDIT FFWB.COBOL.MASTER()`; and a back-compat assertion that a templateless alias (e.g. `X`->`EXCLUDE` in FFEDIT) yields the identical pre-CR-NR-112 string for the same surface+arg.
      Files: `crates/ff-desktop/src/shell/locale_resolver.rs` (and `crates/ff-desktop/src/shell/alias_reader.rs` if `overlay_row_for_test` gains an optional template param)
      Verify: `cargo test -p ff-desktop locale_resolver *> tools\logs\crnr112-resolver.log` -- pass; read the log. Confirm `locale_resolver.rs` stays under 400 non-test lines (split the test module to `locale_resolver_tests.rs` if needed, per rust-standards.md).

- [ ] 6. Add the full-shell end-to-end integration test (worked example through the single front door).
      Add a `harness_shell()` egui_kittest `build_eframe` test (model on `crates/ff-desktop/src/shell/tests_locale_menu.rs`; put it in that file or a new `tests_alias_template.rs` registered in `mod.rs`). Seed an FFEDIT (editor environment) template alias `ECOB`/root `EDIT`/template `FFWB.COBOL.MASTER(&1)` into the shell's alias store before dispatch. Open an editor Context so the active environment is FFEDIT. Drive the typed command `ecob helloW` through the command field (handle_command / dispatch_command_string) and assert the OBSERVABLE effect of the EXISTING EDIT ladder arm: an editor tab for `ffwb.cobol.master(helloW)` opens (assert via the opened tab's title/path or the `file.open` effect used by existing EDIT tests), proving the expanded string flowed through `resolve_target`/the ladder unchanged with no new dispatch path (14.8, 14.9, 19.1, 19.2). Annotate `// Validates: command-semantics Requirement 14.9; command-framework Requirement 19.1`.
      Files: `crates/ff-desktop/src/shell/tests_alias_template.rs` (new) or `crates/ff-desktop/src/shell/tests_locale_menu.rs`, `crates/ff-desktop/src/shell/mod.rs`
      Verify: `cargo test -p ff-desktop alias_template_end_to_end *> tools\logs\crnr112-e2e.log` (by test name). If the full-shell harness risks a long/quiet run, compile-verify and leave the run to the owner; note that in the item's findings.

- [ ] 7. Ship a sample parameterised alias row as DATA (optional authoring example) and confirm back-compat.
      If a shipped sample is warranted, add an `ECOB` row with `command_template` to a per-locale FFEDIT aliases TOML (e.g. under `crates/ff-i18n/i18n/<locale>/aliases/FFEDIT.toml`) ONLY if the gate/tasks call for shipped data; otherwise the template path is exercised purely by tests (preferred for v1 -- do NOT invent shipped data not required by the spec). Either way, confirm an EXISTING templateless row (e.g. `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` `SOEK`->`FIND`) still loads and resolves byte-identically (14.6, 19.3).
      Files: (conditional) `crates/ff-i18n/i18n/<locale>/aliases/FFEDIT.toml`
      Verify: `cargo test -p ff-desktop alias *> tools\logs\crnr112-data.log` -- the templateless rows and the template path both resolve; read the log.

- [ ] 8. Whole-crate compile/lint/format gate (agent verification-done).
      Run the three scoped gate commands below, each as its own wrapped single-job command, and READ each log. Confirm `ff-desktop` compiles clean, clippy is clean with `-D warnings`, and `cargo fmt --check` passes. Confirm no touched non-test `.rs` file exceeds 400 lines (split per rust-ffwb-layout.md if any does). Fix anything the logs report, re-stage, re-run.
      Files: (whole `ff-desktop` crate)
      Verify: read `tools\logs\crnr112-check.log`, `tools\logs\crnr112-clippy.log`, `tools\logs\crnr112-fmt.log` -- all clean. The authoritative full `cargo gate` / whole-suite run is the OWNER's manual step.

- [ ] 9. Flip the TCR.md coverage rows for CR-NR-112.
      In `c:\workspace\apps\FileForgeWorkbench\docs\quality\TCR.md`, under the "Phase (parameterised-aliases) -- CR-NR-112" table, set each of the 16 rows below to its landed status (Test column = the test fn name that proves it, Status -> green) as each test lands. Do NOT touch the separate CR-NR-113 "alias-editor-workspace" table (that is a later gated CR, out of scope here). Rows to flip:
      - command-semantics Req 14.1 (four-part alias record; loader round-trip + `AliasEntry` field) -- items 2, 4
      - command-semantics Req 14.2 (template -> root+space+substituted; literal preserved) -- item 1
      - command-semantics Req 14.3 (`&N` Nth token; `&*` after highest referenced) -- item 1
      - command-semantics Req 14.4 (unsupplied `&N` -> empty, no alias error) -- item 1 + item 5
      - command-semantics Req 14.5 (surplus args appended when no `&*`) -- item 1
      - command-semantics Req 14.6 (absent/empty template byte-identical to today) -- items 4, 5
      - command-semantics Req 14.7 (single-pass; not re-fed) -- item 5
      - command-semantics Req 14.8 (pure text sub; parameterised == typed; observable) -- item 6
      - command-semantics Req 14.9 (worked example end-to-end via existing EDIT routing) -- item 6
      - command-semantics Req 14.10 (whitespace tokenisation; quoted out of scope) -- item 1
      - command-framework Req 19.1 (expanded in pre-pass before resolve_target; same classifier) -- item 6
      - command-framework Req 19.2 (string->string upstream; no 2nd dispatcher; params not routed) -- items 5, 6
      - command-framework Req 19.3 (templateless byte-identical) -- items 4, 5
      - command-framework Req 19.4 (ROOT recorded/persisted; surface+template pre-resolution only) -- item 6 (assert recorded identity is the expanded ROOT)
      - command-framework Req 19.5 (single-pass; future chaining additive via depth+visited) -- items 1, 5 (guards documented, not wired)
      - command-framework Req 19.6 (key-bound merges then expands; menu-option arg supply out of scope) -- item 5/6 (assert menu-option activation of a template surface expands with empty args per 14.4)
      Files: `c:\workspace\apps\FileForgeWorkbench\docs\quality\TCR.md`
      Verify: read the TCR.md CR-NR-112 table back and confirm all 16 rows reference a real landed test and status; no CR-NR-113 row touched.

## Notes / assumptions

- Menu-option argument supply is OUT OF SCOPE (Req 19.6). A template alias used
  as a menu option's `command` expands with an EMPTY argument list (every `&N`
  -> empty per 14.4) because `activate_menu_option` supplies no field; this is
  the documented consequence, not a new mechanism. Item 6 may add a small
  assertion for this if quick, but no routing of the command-line field into
  menu activation is added.
- Chaining is explicitly NOT enabled. The depth-limit + visited-set are comments
  describing the additive future path only.
- CR-NR-113 (Alias Editor workspace) has a hard build-order dependency on this CR
  but is a separate gated change request and is NOT implemented here.
