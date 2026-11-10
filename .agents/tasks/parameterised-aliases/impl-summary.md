# CR-NR-112 Parameterised Command Aliases -- Implementation Summary

Implements command-semantics Requirement 14 (14.1-14.10) and command-framework
Requirement 19 (19.1-19.6). Work done on branch `crnr112-parameterised-aliases`
in the worktree `c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr112-parameterised-aliases`.

This run RESUMED substantial uncommitted draft work left on disk by a prior
coding run that dropped on a transient auth/session failure. The draft covered
plan items 1-6; this run verified it, fixed two defects found during
verification, completed the compile/lint/format gate and the TCR flip (items
7-9), and committed the result.

## Criteria implemented (all 16 TCR rows 🟢)

- 14.1 four-part alias entry (root, surface, optional localisation key, optional
  `command_template`) + TOML round-trip.
- 14.2 template expands to `root + space + substituted template`, literals verbatim.
- 14.3 `&N` Nth whitespace token (1-based, maximal digit run so `&12` = token 12);
  `&*` = tokens after the highest referenced `&N` (all tokens when no `&N`).
- 14.4 unsupplied `&N` -> empty string, no alias-layer error.
- 14.5 surplus tokens beyond the highest referenced `&N` appended when no `&*`.
- 14.6 absent/empty template byte-identical to the pre-CR-NR-112 verb rewrite.
- 14.7 single-pass -- expanded string is not re-fed.
- 14.8 pure text substitution; parameterised alias == equivalent typed command.
- 14.9 worked example `ecob helloW` -> `EDIT FFWB.COBOL.MASTER(helloW)` end-to-end
  through the existing EDIT ladder.
- 14.10 whitespace tokenisation only; quoted multi-word args out of scope.
- 19.1 expanded in the existing `rewrite_verb_to_root` pre-pass BEFORE
  `resolve_target`; same single classifier.
- 19.2 string-to-string rewrite upstream; no second dispatcher.
- 19.3 templateless aliases byte-identical.
- 19.4 ROOT (expanded) command is the dispatched/recorded identity.
- 19.5 single-pass; depth-limit (`MAX_EXPANSION_DEPTH`) + visited-set guards
  DOCUMENTED, not wired.
- 19.6 menu-option arg supply out of scope -- a template alias activated with no
  field expands with empty `&N`.

## Files changed (worktree)

New:
- `crates/ff-desktop/src/shell/alias_template.rs` -- pure `expand_template(root,
  template, arg) -> String` + 9 unit tests; forward-compat guards documented.
- `crates/ff-desktop/src/shell/tests_alias_template.rs` -- 2 full-shell
  (`make_shell`) end-to-end tests proving the worked example and the empty-arg
  menu-activation consequence route through the existing EDIT ladder.

Modified:
- `crates/ff-desktop/src/shell/environment.rs` -- `command_template: Option<String>`
  on `AliasEntry`; `AliasTable::resolve_alias_entry`; template threaded through
  `apply_locale_overlay_keyed`; 2 new tests.
- `crates/ff-desktop/src/shell/alias_overlay.rs` -- `KeyedAliasRow` widened to a
  4-tuple carrying the optional template.
- `crates/ff-desktop/src/shell/alias_reader.rs` -- `#[serde(default)]
  command_template` on `AliasRowToml`; carried through `parse_alias_toml`;
  `overlay_row_for_test_with_template` test helper; 2 new loader tests.
- `crates/ff-desktop/src/shell/locale_resolver.rs` -- `rewrite_verb_to_root`
  extended to expand a template entry via `resolve_template_entry` +
  `alias_template::expand_template` (single-pass); 5 new tests.
- `crates/ff-desktop/src/shell/mod.rs` -- registered `mod alias_template;` and
  `mod tests_alias_template;` (already present in the resumed draft).
- `docs/quality/TCR.md` -- added the CR-NR-112 Phase table with all 16 rows 🟢
  and their proving test fn names (CR-NR-113 table NOT added -- out of scope).

## Defects found and fixed during verification

1. `MAX_EXPANSION_DEPTH` used `#[cfg_attr(not(test), allow(dead_code))]`, which
   left a `dead_code` warning in the TEST build (the constant is never consumed
   by design). Changed to an unconditional `#[allow(dead_code)]` with a comment.
2. `two_digit_positional_is_token_twelve` asserted `CMD <t12>` for a 13-token
   argument, but token 13 is surplus beyond the highest referenced index (12) and
   is correctly appended per Req 14.5, so the real output is `CMD <t12> t13`. The
   IMPLEMENTATION was correct; the TEST expectation was wrong. Fixed the test to
   use exactly 12 tokens, isolating two-digit parsing from the (separately
   covered) surplus-append.

## Commands run + log results (scoped to ff-desktop, worktree manifest)

- `cargo check -p ff-desktop` -> `tools/logs/crnr112-check.log`: Finished, clean.
- `cargo fmt -p ff-desktop -- --check` -> `tools/logs/crnr112-fmt.log`: clean
  (after applying rustfmt to the two alias_template test assertions).
- `cargo clippy -p ff-desktop -- -D warnings` -> `tools/logs/crnr112-clippy.log`:
  Finished, clean (no warnings).
- `cargo test -p ff-desktop alias_template::` -> `crnr112-tmpl.log`: 11 passed.
- `cargo test -p ff-desktop locale_resolver::` -> `crnr112-resolver.log`: 9 passed.
- `cargo test -p ff-desktop shell::environment::` -> `crnr112-env.log`: 9 passed.
- `cargo test -p ff-desktop shell::alias_reader::` -> `crnr112-reader.log`: 7 passed.
- `cargo test -p ff-desktop shell::alias_overlay::` -> `crnr112-overlay.log`: 13 passed.

All scoped CR-NR-112 coverage is green. The authoritative full `cargo gate` /
whole-suite run is the owner's manual step (testing.md execution split).

## Known limitations / follow-ups

- Single-pass only; alias chaining is deliberately not enabled. Depth-limit +
  visited-set are documented, not wired.
- No shipped sample parameterised alias DATA (plan item 7): v1 exercises the
  template path purely by tests, as the plan preferred; existing templateless
  shipped rows (e.g. `af/aliases/FFCMD.toml`) still load and resolve, confirmed
  by `af_poc_alias_catalogue_loads_and_resolves`.
- Non-contiguous templates (referencing only `&2`, skipping `&1`) append keyed
  off the highest REFERENCED index -- a recorded limitation (Req 14.5), covered
  by `non_contiguous_template_does_not_reappend_lower_unreferenced_token`.
- CR-NR-113 (Alias Editor workspace) has a hard build-order dependency on this CR
  but is a separate gated change request, not implemented here.
