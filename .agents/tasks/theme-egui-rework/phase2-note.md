# CR-CH-056 Phase 2 (Task 30) -- Implementation Note

egui-native theme model rework, PHASE 2 ONLY (Task 30, sub-tasks 30.1-30.6):
built-in colour-data instances + the Legacy Soft shorthand + the built-in count
4 -> 5. No file-format / serialiser / version / base changes (Phase 4), no Theme
Editor rebuild (Phase 3), no B081. Builds ON the Phase-1 model (ChromeStyle +
wholesale `apply_to_egui` seam, `gutter` domain group); no framework change.

## What was built (per sub-task)

- **30.1 Default Dark -> Solarized Dark.** `dark_palette()` now delegates to a new
  `solarized_dark_palette()` in `defaults_solarized.rs`. NAME "Default Dark" and
  `VisualMode::Dark` retained. Backgrounds base03 `#002B36` / base02 `#073642`;
  foregrounds base0 `#839496` / base1 `#93A1A1`; muted base01 `#586E75`; shared
  Solarized accents (yellow `#B58900`, orange `#CB4B16`, red `#DC322F`, magenta
  `#D33682`, violet `#6C71C4`, blue `#268BD2`, cyan `#2AA198`, green `#859900`).
- **30.2 Default Light -> Solarized Light.** `light_palette()` delegates to
  `solarized_light_palette()`. NAME "Default Light", `VisualMode::Light`.
  Backgrounds base3 `#FDF6E3` / base2 `#EEE8D5`; shared accents.
- **30.3 Default Legacy retrofit.** (a) The primary option-menu / title band
  background is toned from the full ISPF blue `#0000AA` to the muted navy
  `#000060` (new `ISPF_MENU_BAND` constant; `legacy_ui_colours().primary_menu_bg`).
  ALL other Legacy colours are byte-identical (asserted by
  `legacy_non_band_colours_are_byte_identical`). `default_legacy_palette()` still
  equals `legacy_palette()` for every non-name field (existing
  `default_legacy_matches_legacy_colours` still green). (b) The Legacy SLIDER
  colours are restored as part of the Legacy instance's ChromeStyle, NOT a seam
  hack: `ChromeStyle::from_palette_parts` now, for `VisualMode::Legacy`, sets the
  slider widget fills from palette data -- track = `ui.input_border` (ISPF
  turquoise `#00AAAA`), handle stroke = `editor.accent` (ISPF yellow `#FFFF00`),
  active fill = `ui.input_fg`. This reproduces the former hand-written seam
  injection but sourced from the palette, so each Legacy variant carries its own
  tone and no seam branch remains.
- **30.4 Legacy Soft (NEW built-in).** `legacy_soft_palette()` in
  `defaults_legacy_soft.rs` (`VisualMode::Legacy`). Keeps the ISPF semantic roles
  in all domain groups but softens the harshest pure-saturated values: body green
  `#00FF00` -> `#33FF66`, turquoise `#00FFFF` -> `#4FD6D6`, yellow `#FFFF00` ->
  `#E6D25A`, red `#FF0000` -> `#F25A5A`, pink `#FF00FF` -> `#E06CC8`, white
  `#FFFFFF` -> `#E8E8E8`, electric blue muted to `#6E9BFF`/`#3C559E`. Keeps the
  toned `#000060` menu band. Added to `discovery::BUILTIN_THEME_NAMES` and
  `builtin_themes()` (set 4 -> 5), and to `theme_defaults::builtin_palette_by_name`
  so it loads as a code-only built-in. Because its slider track/handle come from
  its own `ui.input_border` (`#4FD6D6`) / `editor.accent` (`#E6D25A`), its sliders
  carry the softened turquoise/yellow automatically.
- **30.5 `legacy soft` shorthand.** `resolve_theme_arg` maps
  `legacy soft` / `legacy-soft` / `legacy_soft` -> `Legacy Soft`, matched BEFORE
  the bare `legacy` -> `Default Legacy` shorthand.

## Final Solarized chrome values chosen

Three-level background hierarchy (window/base -> raised -> inset), accent, tab,
title band. Chrome accent = Solarized blue `#268BD2` for both.

Dark:
- window/base `ui.panel_bg` = `#002029`; content `editor.background` = base03
  `#002B36`; raised `ui.button_bg` = base02 `#073642`; inset `ui.input_bg` =
  `#001820`.
- focus ring `ui.focus_ring` = `#268BD2` (== accent).
- active tab `tab_bar.active_bg` = `#06303F` (!= inactive `#002029`); active text
  base1 `#93A1A1` (5.23:1).
- title band `ui.primary_menu_bg` = `#062A3A` (!= base panel); band text base1
  `#93A1A1` (5.61:1).
  (The initial `#0A3F55` tab / `#0C445E` band were darkened to `#06303F` /
  `#062A3A` so base1 text clears WCAG AA, Req 23.11.)

Light:
- window/base `ui.panel_bg` = `#E7E1CF`; content `editor.background` = base3
  `#FDF6E3`; raised `ui.button_bg` = base2 `#EEE8D5`; inset `ui.input_bg` =
  `#FBF3DC`.
- focus ring = `#268BD2` (== accent).
- active tab `tab_bar.active_bg` = `#D9E6F2` (!= inactive base2 `#EEE8D5`).
- title band `ui.primary_menu_bg` = `#D2E2F1` (!= base panel); band text
  `#1A3A52` (8.97:1).
- Body/chrome foreground = `#4E5F64` (the Solarized base01 `#586E75` role tone,
  deepened just enough that every checked text pair clears WCAG AA on the lighter
  raised surfaces, Req 23.11). base00 `#657B83` stays for muted / gutter line
  numbers / inactive-tab text (the >= 3:1 UI-element pairs). Solarized Light's
  literal base00-on-base3 body pair is ~4.13:1 (below AA), which is why the body
  tone is the deepened base01; this is the single deliberate AA deepening.

## Legacy slider restoration

In `chrome_style.rs::from_palette_parts`, after the base derivation, a
`matches!(mode, VisualMode::Legacy)` block overrides the slider widget visuals:
`widgets.inactive/hovered.bg_fill` and `weak_bg_fill` = `ui.input_border` (ISPF
turquoise), `widgets.active.bg_fill`/`weak_bg_fill` = `ui.input_fg`, and all three
`fg_stroke` = 2px of `editor.accent` (ISPF yellow). This lives in the Theme's
stored `egui::Style` (the chrome layer), so the seam stays a wholesale
`apply_to_egui` with no Legacy branch. Default Legacy -> turquoise `#00AAAA` /
yellow `#FFFF00`; Legacy Soft -> `#4FD6D6` / `#E6D25A`.

## Contrast-check results

`check_theme_contrast` (WCAG AA advisory, Req 23.11): Solarized Dark and Light
each produce ZERO warnings (asserted by `solarized_palettes_have_no_contrast_warnings`
and `dark_and_light_palettes_have_no_contrast_warnings`). High Contrast is
UNCHANGED and still AAA / zero warnings (`high_contrast_unchanged_no_warnings`,
`high_contrast_fg_bg_pairs_meet_wcag_aaa`). All checked text pairs clear 4.5:1 and
all UI-element pairs clear 3.0:1 (verified numerically before coding).

## Files changed

- `crates/ff-theme/src/lib.rs` -- `pub mod defaults_solarized;` +
  `pub mod defaults_legacy_soft;`.
- `crates/ff-theme/src/defaults_solarized.rs` -- NEW (Solarized Dark/Light
  builders + 8 tests). 354 non-test lines.
- `crates/ff-theme/src/defaults_legacy_soft.rs` -- NEW (Legacy Soft builder +
  3 tests). 198 non-test lines.
- `crates/ff-theme/src/defaults.rs` -- `dark_palette`/`light_palette` delegate to
  Solarized; removed the Catppuccin Dark/Light private builders; `legacy` band
  toned `#000060` via new `ISPF_MENU_BAND`; added `legacy_soft_palette()`
  wrapper; updated/added tests (band toned, byte-identical guard, Legacy slider
  colours in chrome Style, chrome-contract tests re-pointed to Req 23.x). File
  SHRANK 897 -> 825 total lines (non-test ~568 -> 496); the pre-existing
  >400 data-table violation was NOT made worse.
- `crates/ff-theme/src/chrome_style.rs` -- Legacy slider widget-visuals override
  inside `from_palette_parts` (part of the chrome Style, not the seam).
- `crates/ff-theme/src/discovery.rs` -- `Legacy Soft` added to
  `BUILTIN_THEME_NAMES` (4 -> 5); count tests updated (4->5, 5->6 with user file).
- `crates/ff-theme/src/serialiser.rs` -- `serialise_opaque_colour_uses_rrggbb`
  now asserts the Solarized Dark bg `#002B36`.
- `crates/ff-desktop/src/theme_defaults.rs` -- `Legacy Soft` arm in
  `builtin_palette_by_name`; `legacy soft` shorthand in `resolve_theme_arg`;
  `builtins()` test helper + new `resolve_theme_arg_legacy_soft_shorthand` test.
- `crates/ff-desktop/src/shell/tests_menu_workspace.rs` -- the full-shell theme
  list test renamed/updated to assert FIVE built-ins incl. Legacy Soft.

## New / updated test names

New: `defaults_solarized::solarized_dark_has_expected_identity`,
`solarized_light_has_expected_identity`,
`solarized_dark_three_level_background_hierarchy_is_distinct`,
`solarized_light_three_level_background_hierarchy_is_distinct`,
`solarized_focus_ring_is_the_accent`,
`solarized_active_tab_distinct_from_inactive`,
`solarized_title_band_distinct_from_base_panel`,
`solarized_palettes_have_no_contrast_warnings`;
`defaults_legacy_soft::legacy_soft_has_expected_identity`,
`legacy_soft_keeps_toned_menu_band`,
`legacy_soft_softens_pure_saturated_values`;
`defaults::legacy_primary_menu_band_is_toned_navy`,
`legacy_non_band_colours_are_byte_identical`,
`legacy_chrome_style_carries_ispf_slider_colours`;
`theme_defaults::resolve_theme_arg_legacy_soft_shorthand`.
Renamed: `full_shell_theme_list_has_four_builtins` ->
`full_shell_theme_list_has_five_builtins`.
Updated to new expected values (not weakened): `legacy_palette_is_complete`,
`legacy_retains_look_and_feel`, `dark_chrome_has_three_level_surface_hierarchy`,
`light_chrome_has_three_level_surface_hierarchy`,
`dark_and_light_focus_ring_is_accent`,
`dark_and_light_active_tab_is_distinct_and_accented`,
`builtin_themes_returns_five_entries`,
`list_all_themes_includes_builtins_when_dir_absent`,
`list_all_themes_includes_user_themes`,
`list_all_themes_dedups_builtin_named_user_file`,
`serialise_opaque_colour_uses_rrggbb`.

## Scoped checks run (exact commands + results)

From `c:\workspace\VSC\FileForgeWorkbench` via the pwsh7 non-interactive wrapper:

- `cargo fmt` -- clean (no output).
- `cargo check -p ff-theme -p ff-desktop` -- clean (0 warnings after assigning
  the formerly-unused `SOL_VIOLET` to the Solarized macro-name syntax role).
- `cargo clippy -p ff-theme -p ff-desktop` -- clean (0 warnings).
- `cargo test -p ff-theme` -- lib **127 passed, 0 failed**; integration **7/0**;
  property **7/0**; doc **2/0**.
- `cargo test -p ff-desktop --no-fail-fast` -- **975 passed, 3 failed** under the
  default parallel runner. The three failures
  (`shell::tests_menu_workspace::close_workspace_removes_settings_from_config`,
  `shell::tests_menu_workspace::full_shell_theme_shorthand_selects_default_builtins`,
  `shell::tests_session::exit_saves_command_history_and_reloads`) ALL PASS when
  re-run single-threaded:
  `cargo test -p ff-desktop --bin ffwb -- --test-threads=1 <names>` -> **3 passed,
  0 failed**. These are the KNOWN pre-existing parallel-execution flake class
  (shared process-global env/config state between unrelated session/config/theme
  tests under thread-based `cargo test`, B048; the owner's cargo-nextest gate
  process-isolates per test). The theme-shorthand one is the same follow_os/config
  global-state leak, not a regression -- this change only adds built-in colour
  data and a shorthand; it does not touch session/config code.

## Deferred (NOT done here, by design)

- Theme Editor rebuild / import-export / B081 (Phase 3).
- `version` field, v1->v2 load, embedded egui `Style` sub-table, `base`
  resolution (Phase 4).
- Theme-authoring doc rewrite + TCR status flips (Phase 5).
- Per the task: `docs/status/bugs.md`, TCR, the acceptance plan, and tasks.md
  `[x]` marks were NOT touched (owner does that after the full gate).
