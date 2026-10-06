# CR-CH-056 Phase 4 (Task 32) -- Semantic Review

Versioned theme file format, v1 backward-compat load, version-tolerant embedded
egui `Style`, and RESOLVED `base` inheritance with cycle detection

The change makes the theme file format self-describing and future-proof. It adds
a top-level integer `version` plus the egui version the embedded `Style` was
written against, embeds the egui chrome `Style` as an additive `[chrome_style]`
TOML sub-table alongside the retained flat authoring groups, and -- most
substantively -- turns the previously discarded `base` key into real
inheritance: a child theme now inherits unspecified tokens from its named base
(built-in or another user theme) before falling through to the mode default.
Base chains are cycle-guarded and unresolvable bases degrade to a WARN plus the
mode default without failing the load. The flat authoring groups remain the
authoritative chrome source (the Phase-1 derive-on-load design), so built-in
appearance is unchanged and the embedded `Style` is a round-trip-stable snapshot
read tolerantly but never allowed to override the derived chrome.

Watch for: the uncommitted working tree carries Phase 1 (Req 23 gutter rename +
`chrome_style`/`rederive_chrome_style`) and the Req 24 import guard
(`parse_native_theme`) ALONGSIDE Phase 4, because HEAD is still at Phase 2 (Task
30). Those earlier-phase changes are correct and in-scope for THEIR phases, and
Phase 4 does not re-touch built-in colours or Theme Editor behaviour, so the
Phase-scope limit (criterion 8) holds -- but a reviewer skimming `git diff HEAD`
will see more than Task 32. (confirmed)

**Verdict**: APPROVED

## High-level view

The versioned format is implemented exactly as specified. `format_version.rs`
owns `THEME_FORMAT_VERSION = 2`, `LEGACY_FORMAT_VERSION = 1`, and
`EMBEDDED_EGUI_VERSION = "0.33"`; the serialiser writes `version`,
`egui_version`, and a correctly-nested `[chrome_style.style]` sub-table; the
loader reads `version` for provenance/branching. The authoritative-source
decision (flat groups authoritative, chrome derived, embed is a read-only
snapshot) is stated consistently in the note, the loader, and the serialiser.

Backward compatibility is structural rather than conditional: v1 (absent
`version`) and v2 both flow through the same flat-group parse with per-token
default-fill, so an old file cannot fail. The embedded `[chrome_style]` is read
only when present, and only to validate -- it never overrides the derived
chrome, so a v1 file with no embed and a v2 file with a foreign embed behave
identically for appearance.

Version tolerance is achieved by serialising the derived chrome to a TOML table,
deep-merging the embedded fields over it (missing fields keep the derived value),
then deserialising; egui's `Style` does not `deny_unknown_fields`, so extra keys
are dropped. Both the missing-field and the extra-field case have dedicated
tests, and any deserialise error is caught, WARNed, and swallowed.

Base resolution is the real behaviour change. `resolve_fallback_palette` makes
the per-token fallback the resolved base palette instead of the bare mode
default, with a built-in-first then user-resolver lookup. Cycles are detected up
front by `base_chain_has_cycle` (bounded at depth 16) in `load_from_sources`,
which breaks the loop by stripping the base and WARNing; an unresolvable base
WARNs (naming it) and falls back. Each of these has a test, including one that
proves inheritance comes from the BASE (Default Legacy) and not the passed mode
default (Dark).

Round-trip is strengthened, not weakened: the existing colour/font/mode
round-trip tests remain, and a new v2 test asserts the groups, the derived
`Style`, the chrome extras, AND byte-identical re-serialisation
(`toml_str == serialise(&round_tripped)`).

<details>
<summary>Issues (0)</summary>

No blocking or non-blocking findings. The implementation satisfies every
blocking criterion; the observations below are context, not action items.

</details>

<details>
<summary>Details</summary>

## Scope context: all three phases are uncommitted together

HEAD is `be1ec7e ... Phase 2 (Task 30)`. `git diff HEAD -- crates/ff-theme`
therefore shows Phase 1 (Req 23) and the Req 24 import guard in addition to
Phase 4, because none of Phases 1/3/4 are committed yet. Specifically the diff
includes the `ChromeColours -> GutterColours` rename and the `chrome_style`
field + `rederive_chrome_style` on `ThemePalette` (palette.rs), the
`chrome.*  -> gutter.*` follow-through in contrast.rs and the two test files, and
`parse_native_theme` in discovery.rs. These are NOT Task 32 deliverables; they
belong to earlier phases of the same CR and are correct for those phases. The
Phase-4-specific files are `format_version.rs` (new), `loader_parse.rs` (new,
pure refactor), the Phase-4 additions in `loader.rs`, and the version/embed
additions in `serialiser.rs`. The review below judges Task 32 against those, and
confirms Phase 4 did not reach back into built-in colours or the Theme Editor.

## Criterion 1 -- Versioned format (Req 25.1, 25.6)

Satisfied (confirmed). `format_version.rs:26` defines `THEME_FORMAT_VERSION:
u32 = 2`; `:33` `LEGACY_FORMAT_VERSION = 1`; `:42` `EMBEDDED_EGUI_VERSION =
"0.33"`. The serialiser writes a top-level integer `version` and
`egui_version` string (serialiser.rs, the `out.push_str(&format!("version =
{}\n", ...))` / `egui_version` block) and embeds the egui `Style` ADDITIVELY as
`[chrome_style]` with the egui `Style` nested at `[chrome_style.style]`
(`serialise_embedded_chrome` wraps `palette.chrome_style` in a single-key table
so the nesting is correct; test `serialise_embeds_chrome_style_subtable` asserts
`chrome_style.style` is a table). The flat authoring groups are NOT removed --
every `[editor]`/`[syntax]`/`[ui]`/... group is still written above the embed.

Authoritative-source decision is stated and internally consistent: the flat
groups are authoritative and the chrome `Style` is DERIVED on load
(`ChromeStyle::from_palette_parts`), with the embed read only as a tolerant,
non-overriding snapshot. This is asserted in three places that agree -- the note
("the FLAT AUTHORING GROUPS are AUTHORITATIVE"), loader.rs
(`load_from_toml_with_base_resolver` doc comment and the `read_embedded_chrome_
tolerant` comment "NEVER overrides the derived chrome"), and serialiser.rs (the
`[chrome_style]` embed comment "derived ... on load; recorded here for the
record and forward-compatibility"). TOML-over-JSON is recorded (Req 25.6) and
the choice is flaggable; no concern.

## Criterion 2 -- Backward-compat loader (Req 25.2)

Satisfied (confirmed). The loader reads `version` with
`.unwrap_or(LEGACY_FORMAT_VERSION)` so an absent `version` is treated as v1, and
BOTH v1 and v2 take the identical flat-group parse + per-token default-fill path
-- the version only records provenance and gates whether an embedded
`[chrome_style]` read is attempted. Because every group parser fills from the
resolved fallback palette, no field is required and an old file cannot fail.
`v1_file_without_version_loads_with_default_fill` loads a no-`version` file,
asserts the one specified token is applied and an absent token is default-filled.
No destructive migration exists; the loader only reads.

## Criterion 3 -- Version-tolerant Style deserialise (Req 25.3)

Satisfied (confirmed). `read_embedded_chrome_tolerant` runs only when
`[chrome_style]` is present. It serialises the derived chrome to a table,
`deep_merge_table`s the embedded fields over it (so a MISSING embedded field
keeps the derived value), then attempts a `ChromeStyle` deserialise. egui's
`Style` does not use `deny_unknown_fields`, so EXTRA/unknown fields are dropped
by serde; any deserialise error is caught, WARNed via `ff_logging::log`, and the
derived chrome is used -- the load never fails. BOTH cases are proven:
`embedded_style_with_missing_field_loads_defaulted` (near-empty
`[chrome_style.style]`) and `embedded_style_with_unknown_field_is_ignored`
(includes `future_only_egui_field = 42` under `[chrome_style.style]` plus an
unknown top-level chrome field), each asserting a successful load.

## Criterion 4 -- Base resolution (Req 25.4, making 14.4/14.5/15.5 effective)

Satisfied (confirmed). The old `let _base_name` discard is gone;
`resolve_fallback_palette` now reads `base`, resolves it via
`format_version::resolve_base_palette` (built-in first, then the optional
user-resolver), and returns the resolved base palette as the per-token fallback.
Tokens absent from the file inherit the base's values; tokens absent from both
fall through to the mode default (the base palettes are themselves fully
default-filled). The decisive test is
`base_resolves_inherited_tokens_from_named_base_not_bare_default`: it bases a
child on "Default Legacy" while passing `VisualMode::Dark`, then asserts
`palette.syntax.keyword == legacy.syntax.keyword` and `editor.foreground ==
legacy.editor.foreground` -- i.e. the inherited tokens equal the BASE's values,
not the Dark mode default. `base_resolves_through_user_sources_when_acyclic`
additionally proves inheritance from another USER theme.

## Criterion 5 -- Base-cycle safety

Satisfied (confirmed). `base_chain_has_cycle` (format_version.rs) walks the
chain with a `seen` set, bounded by `MAX_BASE_DEPTH = 16`, returning `true` on a
repeat or on exceeding the bound. `load_from_sources` runs this guard BEFORE
resolving; on a cycle it WARNs (`warn_base_cycle`), strips the `base` line
(`strip_base_field`), and loads without re-entering the loop.
`base_cycle_terminates_with_warn_not_hang` builds A->B->A and asserts the load
returns successfully (proving termination) with A's own token applied. The
pure-function cycle detector is independently unit-tested
(`base_chain_cycle_is_detected`, `acyclic_base_chain_is_not_a_cycle`).

## Criterion 6 -- Unresolvable base (Req 25.5)

Satisfied (confirmed). `resolve_fallback_palette` calls
`warn_unresolvable_base(base_name, mode)` -- which logs a WARN naming the
unresolved base and the mode -- and returns the mode default when
`resolve_base_palette` yields `None`. The load does not fail.
`unresolvable_base_warns_and_falls_back_without_error` loads a theme with
`base = "NoSuchTheme"`, asserts the explicit token is applied and the absent
token falls back to the mode default, with a successful load.

## Criterion 7 -- Round-trip (Req 25.7 / Req 9.2)

Satisfied (confirmed). `v2_round_trip_preserves_theme_including_embedded_style_
and_metadata` serialises the dark palette, confirms `version == 2`, loads it
back, and asserts the authoring groups, the derived `Style` (`visuals.panel_fill`),
and a chrome extra (`title_band_bg`) all round-trip, THEN asserts byte-identical
re-serialisation (`toml_str == serialise(&round_tripped)`) -- the strongest
round-trip form. The pre-existing round-trip tests
(`serialise_round_trip_preserves_colours`, `..._fonts`,
`..._legacy_mode`, the property test `prop_assert_eq!` chain) remain and were
UPDATED only for the `chrome -> gutter` field rename (an identifier change, not
a weakened assertion). No assertion was loosened.

## Criterion 8 -- Phase-scope limits

Satisfied (confirmed). Phase 4 adds `format_version.rs`, `loader_parse.rs`, and
the version/embed/base logic in `loader.rs`/`serialiser.rs`. It does NOT touch
`defaults*.rs` built-in colour data (Phase 2) -- `dark_palette()` et al. are
only READ as fallbacks. It does NOT touch Theme Editor behaviour (Phase 3): the
`rederive_chrome_style` method on `ThemePalette` present in the diff is Phase 1
(Req 23.5/20.11) work, not a Phase 4 addition, and no editor render/handler code
is in the ff-theme diff. No docs rewrite (Phase 5 / Task 33) is included. The
`gutter` rename and `parse_native_theme` are earlier-phase changes carried in the
same uncommitted tree, not Phase 4 regressions.

## Criterion 9 -- Steering conformance

Satisfied (confirmed).

- TDD + annotation: every new test carries `// Validates: Requirement 25.x`
  (and 24.x for the import-guard tests). Tests precede implementation per the
  note's TDD account; the test names map 1:1 to the criteria.
- File size: `loader.rs` lib code ends at `strip_base_field` (~line 283) before
  `#[cfg(test)]`; `format_version.rs` lib code ends (~line 145) before its test
  module; `serialiser.rs` is ~366 non-test lines; `loader_parse.rs` is the
  extracted parse helpers. All under the 400 non-test-line limit, and the split
  (`loader_parse.rs`, `format_version.rs`) is exactly the sibling-split the rule
  endorses.
- ASCII: `grep` for non-ASCII across loader.rs / loader_parse.rs /
  format_version.rs / serialiser.rs returns no matches. (The `ΓöÇ` box-drawing
  artifact visible in discovery.rs is a pre-existing separator from an earlier
  phase, not Phase 4 code.)
- Error handling: library code uses `?`, `ok_or_else`, and `.ok()?`; the only
  `.unwrap()`/`.expect()` calls are inside `#[cfg(test)]`. Recoverable base
  cases (unresolvable base, cycle, bad embed) WARN via `ff_logging::log` rather
  than returning errors, matching Req 15.3/25.5.
- Framework conformance: no new persistence mechanism -- the versioned file IS
  the existing `WorkspaceDescriptor`-independent theme file, extended additively.
  `builtin_palette_by_name` is a deliberate ff-theme-local copy (so the loader
  resolves built-in bases without depending on `ff-desktop`); the note flags it
  as distinct from the existing shell helper, which is the correct call.

## Verification evidence

phase4-note.md records the exact scoped commands and results: baseline
`cargo test -p ff-theme` (131+7+7), then post-change `cargo check -p ff-theme`
clean, `cargo test -p ff-theme` 149 unit + 7 integration + 7 property PASS (0
failed, +18 unit), `cargo clippy -p ff-theme --all-targets -- -D warnings` clean,
`cargo fmt -- --check` clean, and a single compile-only `cargo check -p
ff-desktop` clean. This is the correct scoped set, and the note correctly defers
the heavy `ff-desktop`/workspace test run to the owner's manual gate (and flags
the specific `ff-desktop` risk: byte-exact theme-TOML string assertions will now
see the extra `version`/`egui_version`/`[chrome_style]` output). The evidence is
present and self-consistent, so no re-run or spot-check was warranted.

</details>

## File map

<details>
<summary>Files changed</summary>

- `crates/ff-theme/src/format_version.rs` (NEW) -- format version constants,
  `EMBEDDED_EGUI_VERSION`, `builtin_palette_by_name`, `resolve_base_palette`,
  `base_chain_has_cycle`, WARN helpers; unit tests for each.
- `crates/ff-theme/src/loader_parse.rs` (NEW) -- per-group `parse_*` helpers
  moved out of `loader.rs` (pure refactor to stay under 400 lines).
- `crates/ff-theme/src/loader.rs` -- `version` read + branch, embedded-chrome
  tolerant read + deep-merge, `resolve_fallback_palette`, `load_from_sources`
  with cycle detection, `strip_base_field`; Req 25.2-25.5 tests.
- `crates/ff-theme/src/serialiser.rs` -- writes `version`/`egui_version` and the
  nested `[chrome_style.style]` embed; Req 25.1/25.6/25.7 tests.
- `crates/ff-theme/src/lib.rs` -- declares/re-exports `format_version`,
  `loader_parse`, `parse_native_theme`.
- `crates/ff-theme/src/palette.rs` (Phase 1) -- `GutterColours` rename,
  `chrome_style` field, `rederive_chrome_style`.
- `crates/ff-theme/src/contrast.rs`, `discovery.rs`, `tests/*.rs` (Phase 1/Req
  24) -- `gutter` rename follow-through, import guard + tests.
- `crates/ff-theme/Cargo.toml` -- adds `egui` dependency (Phase 1).

Full diff: `git diff HEAD -- crates/ff-theme` plus the two untracked files
`format_version.rs` and `loader_parse.rs`.

</details>
