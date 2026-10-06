# CR-CH-056 Phase 4 (Task 32) -- Implementation Note

Versioned TOML theme format + v1->v2 backward-compat load + version-tolerant
embedded egui `Style` + RESOLVED `base` inheritance with cycle detection.
First iteration, TDD. Scope: `crates/ff-theme` only (plus one compile-only
`cargo check -p ff-desktop` to confirm the workspace still builds).

## Files changed / added

- **NEW `crates/ff-theme/src/format_version.rs`** (~145 non-test lines + tests):
  format version constants and `base` resolution primitives.
  - `THEME_FORMAT_VERSION: u32 = 2` (current egui-native format),
    `LEGACY_FORMAT_VERSION: u32 = 1` (pre-version / absent-`version` files).
  - `EMBEDDED_EGUI_VERSION: &str = "0.33"` -- records the egui version the
    embedded `Style` blob was written against (Req 25.1).
  - `builtin_palette_by_name(name)` -- resolves a built-in base NAME to its
    compiled palette WITHIN `ff-theme` (mirrors the shell helper of the same
    name so the loader can resolve a built-in base without depending on
    `ff-desktop`).
  - `resolve_base_palette(base_name, user_resolver)` -- built-in first, then an
    optional user-theme resolver, else `None`.
  - `base_chain_has_cycle(start, next_base)` -- bounded (MAX_BASE_DEPTH = 16)
    cycle detector over a `base` chain.
  - `warn_unresolvable_base` / `warn_base_cycle` -- WARN via `ff_logging::log`.
- **NEW `crates/ff-theme/src/loader_parse.rs`** (~330 lines, no tests): the
  per-group `parse_*` TOML helpers MOVED out of `loader.rs` (a pure REFACTOR, no
  behaviour change) so `loader.rs` stays under the 400-line non-test limit after
  the Phase-4 additions. Functions are now `pub(crate)` and unchanged otherwise.
- **CHANGED `crates/ff-theme/src/loader.rs`** (now 283 non-test lines):
  - `load_from_toml` is now a thin wrapper over the new
    `load_from_toml_with_base_resolver(toml, mode, user_resolver)`.
  - reads the top-level integer `version` (absent => v1) for format branching;
  - resolves `base` via `resolve_fallback_palette` (new): the per-token fallback
    palette is the resolved base when `base` resolves, else the mode default;
  - reads an embedded `[chrome_style]` sub-table VERSION-TOLERANTLY via
    `read_embedded_chrome_tolerant` + `deep_merge_table` (new) -- never fails the
    load;
  - adds `load_from_sources(name, sources, mode)` (new, public): loads a theme by
    name from a map of raw TOML sources, resolving its `base` chain against that
    map (and built-ins) WITH cycle detection; a cyclic chain is broken (base
    stripped) with a WARN;
  - `strip_base_field` (new helper) removes a `base = "..."` line to break a
    detected cycle.
- **CHANGED `crates/ff-theme/src/serialiser.rs`** (now 366 non-test lines):
  writes top-level `version = 2` and `egui_version = "0.33"`, and appends the
  embedded chrome `Style` as a correctly-nested `[chrome_style]` sub-table
  (`[chrome_style.style]` for the egui `Style`) via the new
  `serialise_embedded_chrome` helper (wraps the `ChromeStyle` in a single-key
  table so egui's serde nesting is correct). The embed is omitted (never a hard
  failure) if egui's `Style` cannot be represented.
- **CHANGED `crates/ff-theme/src/lib.rs`**: declares `mod loader_parse;` and
  `pub mod format_version;`, re-exports `builtin_palette_by_name`,
  `EMBEDDED_EGUI_VERSION`, `LEGACY_FORMAT_VERSION`, `THEME_FORMAT_VERSION`.

## Chosen format (version, Style embedding, authoritative source)

- **Version**: top-level integer `version`. `2` = egui-native format (this CR).
  No `version` field or `version = 1` = legacy; both load through the SAME flat
  authoring groups + per-token default-fill path.
- **egui version recorded**: top-level `egui_version = "0.33"` string, plus the
  `EMBEDDED_EGUI_VERSION` constant, so a future egui upgrade that changes the
  `Style` serde shape is detectable (Req 25.1).
- **Style embedding**: ADDITIVE. The serialiser still writes every flat
  authoring group (`[editor]`/`[syntax]`/`[ui]`/`[tab_bar]`/`[chrome]`/
  `[decorations]`/`[indicators]`/`[font.*]`/`[design.*]`/`[style_slots.*]`) AND
  additionally embeds the egui chrome `Style` (+ FFWB chrome extras) under
  `[chrome_style]`, serialised via egui's own serde derives on `egui::Style`.
- **Authoritative source decision (documented)**: the FLAT AUTHORING GROUPS are
  AUTHORITATIVE; the chrome `egui::Style` is DERIVED on load via
  `ChromeStyle::from_palette_parts(...)` (the Phase-1 derive-on-load design). The
  embedded `[chrome_style]` is a round-trip-stable SNAPSHOT for the record and
  forward-compatibility -- it is read tolerantly but NEVER overrides the derived
  chrome. This keeps built-in appearance byte-identical to Phases 1-3 (nothing in
  the chrome layer changes) and makes the round-trip trivially stable (the derive
  is deterministic from the flat groups, so re-serialising a loaded palette
  yields byte-identical TOML -- asserted by
  `v2_round_trip_preserves_theme_including_embedded_style_and_metadata`).

## v1 backward-compat + version-tolerant Style deserialise

- **v1 (Req 25.2)**: a file with no `version` key is treated as v1 and loads via
  the existing flat-group parse with per-token default-fill; no field is
  required, so an old file NEVER fails. (`v1_file_without_version_loads_with_default_fill`.)
- **Version-tolerant Style (Req 25.3)**: `read_embedded_chrome_tolerant` only
  runs when `[chrome_style]` is present. It serialises the DERIVED chrome to a
  TOML table, DEEP-MERGES the embedded fields over it (so any MISSING egui field
  keeps the derived value), then attempts a `ChromeStyle` deserialise; unknown
  fields are ignored by serde (egui's types do not `deny_unknown_fields`). On ANY
  deserialise error a WARN is logged and the derived chrome is used -- the load
  never fails. (`embedded_style_with_missing_field_loads_defaulted`,
  `embedded_style_with_unknown_field_is_ignored`.)

## base resolution + cycle detection + unresolvable-base fallback

- **Resolution (Req 25.4)**: when a file declares `base = "<name>"`, the per-token
  fallback palette becomes the resolved base (built-in via
  `builtin_palette_by_name`, or a previously-loaded user theme via the resolver /
  source map) instead of the bare mode default. Tokens absent from the file then
  inherit the base's values; tokens absent from BOTH fall through to the mode
  default (the base palettes are themselves fully default-filled).
  (`base_resolves_inherited_tokens_from_named_base_not_bare_default` bases a child
  on `Default Legacy` while passing `VisualMode::Dark` and asserts the inherited
  `syntax.keyword` / `editor.foreground` equal the LEGACY base, not the Dark
  default; `base_resolves_through_user_sources_when_acyclic` inherits from another
  user theme.)
- **Cycle detection (Req 25.4)**: `load_from_sources` walks the `base` chain with
  `base_chain_has_cycle` (bounded at depth 16) BEFORE resolving. A loop
  (A base B, B base A) is detected, the base is stripped so the load terminates,
  and a WARN is emitted -- it never recurses forever.
  (`base_cycle_terminates_with_warn_not_hang`.)
- **Unresolvable base (Req 25.5)**: a base naming a theme that cannot be found
  emits a WARN naming it (`warn_unresolvable_base`) and falls back to the mode
  default WITHOUT failing the load.
  (`unresolvable_base_warns_and_falls_back_without_error`.)

## Exact scoped commands run + results

(All via the clean non-interactive pwsh7 wrapper, output redirected to
`tools/logs/` and read back. The terminal reports `Exit Code: -1` due to known
PSReadLine echo mangling in this environment; the redirected logs are the source
of truth and show the real results below.)

- `cargo test -p ff-theme` (baseline, before changes): 131 unit + 7 integration +
  7 property PASS.
- `cargo check -p ff-theme`: Finished, clean.
- `cargo test -p ff-theme`: **149 unit + 7 integration + 7 property PASS, 0 failed**
  (`tools/logs/p4-lib.txt`, `p4-prop.txt`). +18 unit tests vs baseline.
- `cargo clippy -p ff-theme --all-targets -- -D warnings`: Finished, **no warnings**
  (`tools/logs/p4-clippy2.txt`).
- `cargo fmt -- --check`: clean, no diffs (`tools/logs/p4-fmtcheck2.txt` empty).
- `cargo check -p ff-desktop` (compile-only, once): **Finished, clean** -- the
  workspace still builds with the new serialiser/loader
  (`tools/logs/p4-desktop-check2.txt`).

Per the Phase-3 lesson, the heavy `ff-desktop` / full-suite TEST run was NOT run
in-agent (only the compile-only check). File-size rule: every changed/new .rs is
under 400 non-test lines (loader.rs 283, serialiser.rs 366, loader_parse.rs ~330,
format_version.rs ~145). All new/changed .rs are plain ASCII.

## New / updated test names

In `format_version.rs`: `format_version_is_two`, `embedded_egui_version_is_recorded`,
`builtin_base_resolves_to_compiled_palette`, `unknown_base_without_resolver_is_none`,
`user_base_resolves_via_resolver`, `base_chain_cycle_is_detected`,
`acyclic_base_chain_is_not_a_cycle`.

In `loader.rs` tests: `v1_file_without_version_loads_with_default_fill` (25.2),
`v2_file_with_embedded_style_loads` (25.2/25.3),
`embedded_style_with_missing_field_loads_defaulted` (25.3),
`embedded_style_with_unknown_field_is_ignored` (25.3),
`base_resolves_inherited_tokens_from_named_base_not_bare_default` (25.4),
`unresolvable_base_warns_and_falls_back_without_error` (25.5),
`base_cycle_terminates_with_warn_not_hang` (25.4),
`base_resolves_through_user_sources_when_acyclic` (25.4).

In `serialiser.rs` tests: `serialise_writes_version_and_egui_version` (25.1),
`serialise_embeds_chrome_style_subtable` (25.6),
`v2_round_trip_preserves_theme_including_embedded_style_and_metadata` (25.7).

All pre-existing serialiser/loader tests kept GREEN (round-trip tests now also
exercise the v2 `version`/`[chrome_style]` output without weakened assertions).

## HAND-OFF -- for the owner's full gate

Kiro ran ONLY the scoped `ff-theme` checks above plus a single compile-only
`cargo check -p ff-desktop`. The full `ffwb-gate.ps1` (fmt + clippy --workspace +
nextest the full suite) is the OWNER's manual step:

```
pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1
```

Things that MIGHT affect `ff-desktop` (please confirm in the full gate):
- The serialiser now writes `version`, `egui_version`, and a `[chrome_style]`
  sub-table. Any `ff-desktop` test that asserts on the EXACT serialised theme
  TOML text (export/Save output, snapshot of a theme file) will now see the extra
  version metadata + the embedded `[chrome_style]` block. The round-trip still
  loads correctly (verified in `ff-theme`), but a byte-exact string assertion in
  `ff-desktop` would need updating. `cargo check -p ff-desktop` passed (compiles),
  but its TESTS were not run in-agent.
- `ff_theme::loader::load_from_toml` keeps its signature and behaviour for files
  WITHOUT a `base`; files WITH a `base` now actually inherit (previously the base
  was discarded). If any `ff-desktop` test authored a theme with a `base` and
  expected the OLD discard-and-default behaviour, it will now see inherited
  tokens -- this is the intended Req 25.4 fix, not a regression.
- New public API: `ff_theme::loader::load_from_toml_with_base_resolver`,
  `ff_theme::loader::load_from_sources`, and `ff_theme::builtin_palette_by_name`
  (ff-theme's own copy; distinct from the existing
  `ff_desktop::theme_defaults::builtin_palette_by_name`). No existing call site
  changed.

No built-in COLOURS, Theme Editor behaviour, or v1 backward-compat were changed.
The on-disk format change is purely additive (v1 files still load; v2 adds
metadata + an embedded snapshot).
