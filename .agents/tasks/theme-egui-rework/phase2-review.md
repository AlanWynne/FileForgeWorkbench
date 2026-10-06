# Solarized built-ins, toned Legacy band, and Legacy Soft (CR-CH-056 Phase 2 / Task 30)

Phase 2 rebuilds the two colour-mode defaults as Solarized instances while keeping
their names (`Default Dark` / `Default Light`) and `VisualMode`s, tones the Legacy
primary option-menu band from the full ISPF blue `#0000AA` to a muted navy
`#000060`, restores the ISPF slider track/handle colours inside the Legacy
ChromeStyle (resolving the Phase 1 slider regression without a seam hack), and adds
a fifth built-in `Legacy Soft` plus a `legacy soft` command shorthand. The colour
data lives in two new sibling modules (`defaults_solarized.rs`, `defaults_legacy_soft.rs`)
so `defaults.rs` does not grow. All Solarized text pairs clear WCAG AA and High
Contrast is untouched at AAA.

Watch for: the Phase 2 commit (be1ec7e) does not compile on its own -- it references
`palette.gutter` and `palette.chrome_style`, which exist only in UNCOMMITTED
working-tree changes (the Phase 1 model). The full reviewable state is commit +
working tree together (confirmed). Also, the committed test file bundles unrelated
`handle_command` -> `dispatch_command_string` renames from a different change (CR-CH-052 /
dispatch-unify). The Light body foreground was deepened from the literal Solarized
base00 to `#4E5F64` to satisfy Req 23.11 (the literal pair is 4.13:1, below AA) --
a documented, verified deviation from the "base00 body" wording.

**Verdict**: APPROVED

## High-level view

The Solarized palettes are byte-correct against Ethan Schoonover's specification:
every base tone and all eight accent hues match the brief's required hex values,
and the name/mode identity is retained so the built-in slot semantics do not shift.
The three-level background hierarchy, accent focus ring, accent-tinted active tab,
and accent-tinted title band (Req 23.7-23.10) are each asserted by a dedicated test
on palette data, and I independently recomputed the WCAG ratios behind the Req 23.11
"no new below-AA pair" claim -- they match the implementer's numbers to two decimals.

The Legacy retrofit is surgical: the only Legacy colour that changes is the
primary-menu band (`#0000AA` -> `#000060`), guarded by a byte-identity test that
pins every other `ui` field and the editor group to the historical ISPF values.
The Phase 1 slider regression is fixed the right way -- the turquoise track / yellow
handle now come from the Legacy instance's own `ui.input_border` / `editor.accent`
through `ChromeStyle::from_palette_parts`, so each Legacy variant carries its own
tone and no `apply`-seam special case remains.

Legacy Soft is a well-formed fifth built-in: softened phosphor values in every
domain group, the toned `#000060` band kept, `VisualMode::Legacy`, and it is wired
into `BUILTIN_THEME_NAMES`, `builtin_themes()`, and `builtin_palette_by_name`. The
`legacy soft` shorthand is ordered before the bare `legacy` match so it cannot be
shadowed, and both are tested across separator variants.

The phase boundaries are respected: no file-format, serialiser-format, versioning,
or `base`-resolution change (the gutter group still serialises under `[chrome]`),
no Theme Editor rebuild, B081 untouched, and built-in palettes stay compiled and
read-only. The one real caveat is git hygiene, not theme correctness: Phase 1 sits
uncommitted alongside the Phase 2 commit, and an unrelated dispatch rename rides in
the same commit's test file.

<details>
<summary>Issues (3)</summary>

1. **Split commit state (non-blocking)** -- confirmed: commit be1ec7e references `palette.gutter`/`chrome_style` that exist only in uncommitted working-tree Phase 1 changes, so the commit alone will not build. The owner should stage Phase 1 + Phase 2 together (or commit Phase 1 first) before the full gate so the history is bisectable.
2. **Unrelated renames bundled in the commit (non-blocking)** -- confirmed: `crates/ff-desktop/src/shell/tests_menu_workspace.rs` in the Phase 2 commit carries ~30 `handle_command` -> `dispatch_command_string` edits and a `menu_bar_has_help_and_excludes_terminate_option` rename that belong to CR-CH-052 / dispatch-unify, not the theme work. Consider separating them; they do not affect theme behaviour.
3. **Light body fg deviates from the literal base00 (non-blocking, documented)** -- confirmed: the brief lists Light fg as base00 `#657B83`, but the body/chrome foreground is `#4E5F64` (deepened base01). The literal base00-on-base3 pair is 4.13:1 (below AA); the deepening is required to satisfy Req 23.11 and is documented in code and the note. base00/base01 are retained for the >=3:1 UI/muted roles.

</details>

<details>
<summary>Details</summary>

### Solarized values and identity (Req 18.3)

Both builders live in `defaults_solarized.rs` and the shared accent/base constants
are declared once at module top. Checked against the brief: `SOL_BASE03 #002B36`,
`SOL_BASE02 #073642`, `SOL_BASE01 #586E75`, `SOL_BASE00 #657B83`, `SOL_BASE0 #839496`,
`SOL_BASE1 #93A1A1`, `SOL_BASE2 #EEE8D5`, `SOL_BASE3 #FDF6E3`; accents yellow
`#B58900`, orange `#CB4B16`, red `#DC322F`, magenta `#D33682`, violet `#6C71C4`,
blue `#268BD2`, cyan `#2AA198`, green `#859900`. All match. Dark editor background is
base03 and foreground is base0; Light editor background is base3. Names stay
`Default Dark` / `Default Light` and modes stay Dark / Light, so the built-in slot
identity is preserved (`solarized_dark_has_expected_identity`,
`solarized_light_has_expected_identity`). `SOL_VIOLET` is used (syntax macro role),
so there is no dead-constant clippy risk.

The one deviation from the brief's literal wording is the Light body foreground.
The brief says "fg base00 `#657B83`", but `SOL_L_FG = #4E5F64` is used for
`editor.foreground`, `panel_fg`, button/input text, and `default_text`. The literal
base00-on-base3 pair is 4.13:1 (I recomputed it: 4.13), which fails the AA 4.5:1
bar that Req 23.11 forbids regressing. The implementer deepened base01 to `#4E5F64`
for the body role and kept base00 (`#657B83`) for muted / gutter line numbers /
inactive-tab text (the >=3:1 UI-element pairs). This is the correct resolution of
the tension between "retrofit Solarized" (18.3) and "no new below-AA pair" (23.11),
and it is documented at the constant and in the note.

### egui-Visuals chrome contract (Req 23.7-23.10)

Each clause has a direct palette-level assertion:
- 23.7 three-level hierarchy: `panel_bg` (window) != `button_bg` (raised) !=
  `input_bg` (inset), and all differ from `editor.background` (base). Dark uses
  `#002029` / base02 / `#001820` over base03; Light uses `#E7E1CF` / base2 /
  `#FBF3DC` over base3. Asserted by the two `*_three_level_background_hierarchy_is_distinct`
  tests and the mirror tests in `defaults.rs`.
- 23.8 focus ring == accent: `ui.focus_ring == editor.accent == #268BD2` for both,
  asserted by `solarized_focus_ring_is_the_accent` and `dark_and_light_focus_ring_is_accent`.
- 23.9 active tab accent-tinted and distinct: `tab_bar.active_bg != inactive_bg`
  (`#06303F` vs `#002029` dark; `#D9E6F2` vs base2 light).
- 23.10 title band distinct from base: `primary_menu_bg != panel_bg` and `!= editor.background`
  (`#062A3A` dark, `#D2E2F1` light).

These are palette-data assertions rather than rendered `egui_kittest` checks. That
is defensible here: the clauses are properties of the Theme's colour instance (which
`ChromeStyle::from_palette_parts` turns into the `egui::Style`), not of a focus walk
or widget interaction, and the chrome Style's propagation through `apply_to_egui` is
separately covered in `chrome_style.rs` tests (full-visuals-surface, dark_mode sync,
serde round-trip). The chrome contract is instance data, so testing the instance is
appropriate; no `egui_kittest` gap worth blocking on.

### Req 23.11 contrast -- independently verified

The AA claim rests on `check_theme_contrast` returning zero warnings for both
Solarized palettes (`solarized_palettes_have_no_contrast_warnings`,
`dark_and_light_palettes_have_no_contrast_warnings`) and High Contrast being
unchanged (`high_contrast_unchanged_no_warnings`). Because the darkened tab/band
tones were a late AA adjustment, I recomputed the WCAG 2.x ratios for the pairs the
note calls out:

```
base1 on dark active tab  #93A1A1 / #06303F : 5.23  (note: 5.23)
base1 on dark title band  #93A1A1 / #062A3A : 5.61  (note: 5.61)
base0 on dark body        #839496 / #002B36 : 4.75
Lfg on light body         #4E5F64 / #FDF6E3 : 6.19
Lfg on light raised       #4E5F64 / #EEE8D5 : 5.45
title fg on light band    #1A3A52 / #D2E2F1 : 8.97  (note: 8.97)
base00 on base3 (literal) #657B83 / #FDF6E3 : 4.13  (below AA -> reason to deepen)
```

Every body/chrome text pair clears 4.5:1 and the literal base00 body pair is indeed
4.13:1, confirming both the AA claim and the stated reason for the Light body
deepening. High Contrast colours are untouched in `defaults.rs`, so its AAA status
holds.

### Legacy retrofit and byte-identity (Req 13.2, 18.1)

`ISPF_MENU_BAND = #000060` is the one changed Legacy colour; `legacy_ui_colours().primary_menu_bg`
now returns it. `legacy_non_band_colours_are_byte_identical` pins the editor group
(bg black, fg `#00FF00`, accent `#FFFF00`, muted `#0000AA`) and every other `ui`
field (panel_fg turquoise-hi, button_fg yellow, input_fg turquoise-hi, menu_bar_fg
white-hi, focus_ring yellow-hi) to the historical constants, with the band as the
sole change. `legacy_primary_menu_band_is_toned_navy` asserts `#000060` and that it
is not the old `#0000AA`. `default_legacy_matches_legacy_colours` still compares all
non-name groups, so `Default Legacy` tracks `legacy_palette()` including the toned
band. This is a faithful, minimal retrofit.

### Legacy slider restoration via ChromeStyle (Req 23.5, finding 1 from Phase 1)

In `chrome_style.rs::from_palette_parts`, a `matches!(mode, VisualMode::Legacy)`
block overrides the slider widget visuals after the base derivation: inactive/hovered
`bg_fill` + `weak_bg_fill` take `ui.input_border` (turquoise), active takes
`ui.input_fg`, and all three `fg_stroke` take 2px of `editor.accent` (yellow). For
Default Legacy that yields track `#00AAAA` / handle `#FFFF00`; for Legacy Soft it
yields `#4FD6D6` / `#E6D25A` automatically, because the values are sourced from each
instance's own palette rather than hardcoded. The colours live in the Theme's stored
`egui::Style`, so `apply_to_egui` stays a wholesale copy with no Legacy branch at the
seam. `legacy_chrome_style_carries_ispf_slider_colours` and
`legacy_slider_colours_come_from_the_chrome_style_not_the_seam` assert both the exact
colours and that the track is non-black (the regression the Phase 1 review flagged).
This is the correct, framework-conformant fix.

### Legacy Soft built-in (Req 18.3) and shorthand (Req 17.2)

`defaults_legacy_soft.rs` keeps the ISPF semantic roles in every domain group while
softening the harshest pure values exactly as the brief lists: body `#00FF00` ->
`#33FF66`, turquoise `#00FFFF` -> `#4FD6D6`, yellow `#FFFF00` -> `#E6D25A`, red
`#FF0000` -> `#F25A5A`, pink `#FF00FF` -> `#E06CC8`, white `#FFFFFF` -> `#E8E8E8`,
electric blue muted to `#6E9BFF` / `#3C559E`. The toned `#000060` band is kept and
the mode is `VisualMode::Legacy`. It is added to `BUILTIN_THEME_NAMES` (now five),
`builtin_themes()`, and `builtin_palette_by_name`. `discovery.rs` count tests move
4 -> 5 (and 5 -> 6 with a user file), de-dup and absent-dir tests follow, and the
full-shell `full_shell_theme_list_has_five_builtins` asserts the five names
including Legacy Soft and the continued absence of `Legacy (ISPF 3270)`.

The `legacy soft` / `legacy-soft` / `legacy_soft` shorthand maps to `Legacy Soft`
and is matched BEFORE the bare `legacy` -> `Default Legacy` arm, so it is not
shadowed; `resolve_theme_arg_legacy_soft_shorthand` covers all three separator
spellings and confirms bare `legacy` still resolves to `Default Legacy`.

### Phase scope and framework conformance

No file-format change: the serialiser still emits the gutter group under the
`[chrome]` TOML section and the loader still parses it there (`parse_gutter_colours`
reads `[chrome]`), so persisted files are unchanged -- the rename is in-memory only,
deferring the real format work to Phase 4. No `version` field, no embedded egui
`Style` sub-table in the file, no `base` resolution added. The Theme Editor is not
rebuilt and B081 is not touched. Built-ins remain compiled and read-only
(`ensure_default_theme_files` still writes nothing). The chrome layer is built on the
Phase 1 `ChromeStyle` + wholesale `apply_to_egui` seam with no new mechanism, so
framework-conformance holds.

### Standards and testing

The new modules are within the 400-line rule (solarized ~354 non-test, legacy soft
~198 non-test per the note) and `defaults.rs` shrank by delegating the Dark/Light
builders out, so the pre-existing data-table size was not made worse. No `unwrap`/
`expect` in library code (builders are total). ASCII-only source. Tests precede the
data per the stated TDD flow, each new test carries a `// Validates: Requirement X.Y`
annotation, and the former Catppuccin / count-4 tests were updated in place (renamed
`full_shell_theme_list_has_four_builtins` -> `_five_builtins`, serialiser bg assertion
re-pointed to `#002B36`, chrome-contract tests re-pointed to Req 23.x) rather than
weakened or deleted. The 3 `ff-desktop` parallel failures are the documented B048
process-global env/config flake class (they pass single-threaded and under the
owner's nextest process isolation) and do not touch theme code.

</details>

<details>
<summary>File map</summary>

- `crates/ff-theme/src/defaults_solarized.rs` (new) -- Solarized Dark/Light builders + chrome tones + 8 tests.
- `crates/ff-theme/src/defaults_legacy_soft.rs` (new) -- Legacy Soft builder + 3 tests.
- `crates/ff-theme/src/defaults.rs` -- Dark/Light delegate to Solarized; Catppuccin builders removed; `ISPF_MENU_BAND #000060`; `legacy_soft_palette()` wrapper; byte-identity + chrome-contract + slider tests.
- `crates/ff-theme/src/chrome_style.rs` (new in commit) -- Phase 1 egui-native chrome layer incl. the Legacy slider override; tests.
- `crates/ff-theme/src/discovery.rs` -- `Legacy Soft` added to `BUILTIN_THEME_NAMES` (4 -> 5); count/de-dup tests updated.
- `crates/ff-theme/src/serialiser.rs` -- gutter serialised under `[chrome]`; test asserts `#002B36`.
- `crates/ff-theme/src/lib.rs` -- new modules declared and `ChromeStyle` re-exported.
- `crates/ff-desktop/src/theme_defaults.rs` -- `Legacy Soft` arm + `legacy soft` shorthand + tests.
- `crates/ff-desktop/src/shell/tests_menu_workspace.rs` -- five-builtins test; bundles unrelated dispatch renames.
- UNCOMMITTED working tree (Phase 1 substrate): `palette.rs` (`chrome`->`gutter`, `chrome_style` field), `loader.rs`, `contrast.rs`, `Cargo.toml` (egui dep), ff-theme tests, and the ff-desktop shell `dispatch_command_string` changes.

Full diff: `git show be1ec7e` plus `git diff` for the uncommitted Phase 1 working-tree changes.

</details>
