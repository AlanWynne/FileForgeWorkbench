# Review -- CR-NR-112 Parameterised Command Aliases

Branch: `crnr112-parameterised-aliases` (worktree), forked from `main`.
Verdict: **APPROVED** -- no blocking findings.

## Scope reviewed

Branch diff vs `main` (8 files, +772/-22):

- `crates/ff-desktop/src/shell/alias_template.rs` (new, 213 non-test lines)
- `crates/ff-desktop/src/shell/tests_alias_template.rs` (new, full-shell e2e)
- `crates/ff-desktop/src/shell/locale_resolver.rs` (206 non-test lines)
- `crates/ff-desktop/src/shell/environment.rs` (399 non-test lines)
- `crates/ff-desktop/src/shell/alias_reader.rs` (355 non-test lines)
- `crates/ff-desktop/src/shell/alias_overlay.rs` (type widening)
- `crates/ff-desktop/src/shell/mod.rs` (module registration)
- `docs/quality/TCR.md` (CR-NR-112 table, 16 rows)

Reviewed against command-semantics Requirement 14 (14.1-14.10), command-framework
Requirement 19 (19.1-19.6), the plan, and framework-conformance.

## Framework conformance -- PASS

- Template expansion lives INSIDE the existing `rewrite_verb_to_root` pre-pass
  (`locale_resolver.rs`), strictly UPSTREAM of `resolve_target`. It is NOT moved
  into `resolve_target`. Confirmed via a new helper `resolve_template_entry`
  returning `(root, template)`; when present it returns
  `alias_template::expand_template(&root, &template, arg)`, otherwise it falls
  through to today's verb-only rewrite.
- Both call sites -- `dispatch.rs:87` (typed line) and `commands_menu.rs:186`
  (menu/key seam) -- call `rewrite_verb_to_root` and are UNCHANGED in shape
  (neither file appears in the diff). No second dispatcher, no parallel resolver,
  no change to `CommandTarget` / `resolve_target` / `dispatch_command_target` /
  nav stack. (Req 19.1, 19.2 -- PASS.)
- Single-pass: the expanded string is returned and not re-fed. Proven by
  `rewrite_verb_to_root_is_single_pass`. Depth-limit (`MAX_EXPANSION_DEPTH`) and
  the visited-set are DOCUMENTED in the module doc comment and NOT wired.
  (Req 14.7, 19.5 -- PASS.)

## Grammar correctness -- PASS

Verified `expand_template` against Req 14.3/14.4/14.5/14.10:

- `&N` -> Nth whitespace token, 1-based; maximal digit run (`&12` = token 12) via
  `parse_positional`. Confirmed by `two_digit_positional_is_token_twelve`.
- `&*` -> tokens AFTER the highest referenced `&N` (`start = highest_ref`),
  joined by single spaces; ALL tokens when no `&N` (`highest_ref == 0`). No
  repetition when mixing `&1` and `&*`. Confirmed by
  `rest_placeholder_takes_tokens_after_highest_positional_no_repeat` and
  `rest_placeholder_without_positional_takes_all_tokens`.
- Unsupplied `&N` -> empty string, no alias-layer error. Confirmed by
  `unsupplied_positional_substitutes_empty` and
  `rewrite_verb_to_root_empty_arg_substitutes_empty`.
- Surplus args appended (space-separated, single leading space) when no `&*`,
  keyed off the highest REFERENCED index -- including the non-contiguous case
  (`&2` only). The `!has_rest && tokens.len() > highest_ref` guard prevents
  double-append when `&*` is present. Confirmed by
  `surplus_tokens_are_appended_when_no_rest_placeholder` and
  `non_contiguous_template_does_not_reappend_lower_unreferenced_token`. The
  non-contiguous behaviour is the recorded limitation in Req 14.5 -- matches.
- Bare `&` and literal template text copied verbatim; whitespace-only
  tokenisation. Confirmed by `bare_ampersand_and_no_placeholder_are_literal`,
  `tokenisation_is_whitespace_only`.

## Back-compat -- PASS

Absent/empty `command_template` behaves byte-identical to today's pure verb
rewrite. Empty-to-None normalisation applied at BOTH the loader
(`parse_alias_toml`) and the overlay (`apply_locale_overlay_keyed`). Tested by
`reader_row_without_template_parses_to_none` and
`rewrite_verb_to_root_templateless_alias_is_byte_identical` (verb-only rewrite,
argument preserved exactly). (Req 14.6, 19.3 -- PASS.)

## Additive data model -- PASS

- `command_template: Option<String>` added to `AliasEntry` (Debug/Clone/PartialEq/Eq
  already derived; set to `None` in every seed and overlay-insert path).
- `#[serde(default)] command_template: Option<String>` on `AliasRowToml`; carried
  verbatim (trimmed, empty -> None) through `parse_alias_toml`.
- `KeyedAliasRow` widened from 3-tuple to 4-tuple; the keyless shim
  `apply_locale_overlay` maps `(s,c)` -> `(s,c,None,None)` so pre-CR-NR-109
  keyless callers are byte-identical.
- New accessor `AliasTable::resolve_alias_entry` returns the whole entry
  (case-insensitive); `canonical_verb` retained for display/verb-only callers.

## Worked example end-to-end -- PASS

`alias_template_end_to_end_opens_expanded_edit_target` drives `ecob helloW`
through `run_command_line` -> `dispatch_command_string` -> pre-pass ->
`resolve_target` -> the existing EDIT ladder arm, asserting `pending_open.path ==
"FFWB.COBOL.MASTER(helloW)"`. `alias_template_with_no_argument_expands_empty_and_routes`
proves the empty-arg menu-activation consequence (`EDIT FFWB.COBOL.MASTER()`),
Req 19.6. (Req 14.8, 14.9, 19.1, 19.4 -- PASS.)

## rust-standards -- PASS

- File sizes (non-test): alias_template 213, locale_resolver 206, alias_reader
  355, environment 399 -- all <= 400.
- No `unwrap`/`expect` in non-test code: the resolver uses `.lock().ok()?` and
  `.as_ref()?`; expansion uses `saturating_mul/add` and `checked_sub`. The only
  `expect`/`unwrap` occurrences are inside `#[cfg(test)]`.
- Public types derive `Debug` (`AliasEntry`, `LocaleResolveOutcome`).
- Plain ASCII confirmed for alias_template.rs and locale_resolver.rs (no non-ASCII
  matches). Template VALUES are DATA (out of ASCII scope).

## TCR -- PASS

16 CR-NR-112 rows added under the "Phase (parameterised-aliases) -- CR-NR-112"
table, all flipped green, each citing a real landed test fn (cross-checked
against the passing test logs). The CR-NR-113 alias-editor-workspace table was
NOT added/touched. Status glyphs use the established coverage-report emoji
convention (allowed by documentation-charset.md).

## Verification evidence (read, not re-run)

- `tools/logs/crnr112-check.log`: `cargo check -p ff-desktop` Finished, clean.
- `tools/logs/crnr112-clippy.log`: `cargo clippy -p ff-desktop -- -D warnings`
  Finished, clean (no warnings).
- `tools/logs/crnr112-fmt.log`: empty (fmt --check prints nothing when clean).
- `tools/logs/crnr112-tmpl.log`: 11 passed (9 alias_template unit + 2 full-shell).
- `tools/logs/crnr112-env.log`: 9 passed (incl. the 2 new template tests).

Evidence is present, consistent with the impl-summary and commit, and sufficient.
The authoritative full `cargo gate` / whole-suite run is the owner's manual step.

## Findings

None (no blockers).
