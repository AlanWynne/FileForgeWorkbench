# CR-CH-056 Phase 1 (Task 29) -- Implementation Note

egui-native theme model rework, PHASE 1 ONLY (model + egui mapping + read-site
migration). No built-in colour changes (Phase 2), no editor rebuild (Phase 3),
no file-format/version/base work (Phase 4), no doc rewrite (Phase 5).

## What was built (per sub-task)

- **29.1 egui `serde` feature.** Enabled on the workspace egui dependency
  (`Cargo.toml`): `egui = { version = "0.33", features = ["serde"] }`. Confirmed
  the feature name is exactly `serde` for egui 0.33.3 (the crate compiled with it;
  eframe/egui_kittest still build -- ff-desktop and its egui_kittest tests all
  compile and run). Added `egui = { workspace = true }` to
  `crates/ff-theme/Cargo.toml` (ff-theme previously had no egui dep; the chrome
  layer requires it).

- **29.2 `ChromeStyle` chrome-layer type.** New module
  `crates/ff-theme/src/chrome_style.rs` (262 non-test lines). `ChromeStyle` is
  BACKED BY `egui::Style` (`pub style: egui::Style`, serialised via egui's own
  serde derives) plus the FFWB-only chrome colours egui's `Style` does NOT model,
  carried as `ColourRGBA` extras: `title_band_bg`/`title_band_fg` (the
  Title_Line / primary-menu header band), `tab_active_bg`/`tab_inactive_bg`/
  `tab_active_text`/`tab_inactive_text` (the tab bar), `accent`, `focus_ring`,
  and `foreground` (chrome body text for the F-key label bar). The stored
  `egui::Style` carries the FULL themable Visuals/WidgetVisuals surface: per-state
  widget fills (incl. the previously-unset `open` state) + `weak_bg_fill` +
  `fg_stroke`, selection fill+stroke, hyperlink, extreme/faint/code bg, warn/error,
  window/menu corner radius, window/popup shadow, and `dark_mode`.

- **29.3 `apply_to_egui`.** `ChromeStyle::apply_to_egui(&self, &mut egui::Style)`
  is the single WHOLESALE apply: it overwrites the whole `Style` with the Theme's
  stored `egui::Style` and keeps `visuals.dark_mode` in sync. The DesignTokens
  spacing/rounding/shadow are wired into the stored Style inside
  `ChromeStyle::from_palette_parts` (so they reach egui -- Req 23.6): radii ->
  widget + window + menu corner radius, shadows -> window/popup shadow, spacing ->
  `Style.spacing` (item_spacing / button_padding / menu_margin / indent).

- **29.4 `ThemePalette` reshape.** Added `pub chrome_style: ChromeStyle`. RENAMED
  the `chrome` domain group to `gutter` (type `ChromeColours` -> `GutterColours`),
  repointing the `colour()` match arms (the `ColourToken::Chrome*` VARIANT NAMES
  are retained so the token API and every editor/syntax read site are unchanged).
  RETAINED `syntax`/`file_tree`/`decorations`/`indicators`/`style_slots`/`elements`/
  `fonts`/`design`/`mode`/`name`. **UiColours / TabBarColours / EditorColours were
  KEPT (not cut over)** as the Phase-1 authoring surface; `chrome_style` is DERIVED
  from them via `ChromeStyle::from_palette_parts(ui, tab_bar, editor, design, mode)`
  in both `defaults.rs` (each built-in) and `loader.rs` (every loaded theme). This
  is the "thin shim" option the task permits; it keeps Phase-1 appearance
  byte-identical and keeps ff-theme-editor + the ColourToken API + ~2 ff-desktop
  field tests compiling unchanged. The flat groups will be cut over / the editor
  rebuilt in Phase 3.

- **29.5 Seam rewrite.** `WorkbenchShell::apply_theme`
  (`crates/ff-desktop/src/shell/render_theme.rs`) is now:
  `let mut style = (*ctx.style()).clone(); self.palette.chrome_style.apply_to_egui(&mut style); ctx.set_style(style);`
  The former hand-written ~15-field body is gone, and the **hardcoded Legacy
  slider-colour injection at the seam is REMOVED**. `visuals.dark_mode` is set by
  `apply_to_egui` to match the Theme's VisualMode (Light => false, Dark/Legacy/
  HighContrast => true).

- **Legacy slider colours preserved WITHOUT the seam hack.** The old seam injected
  ISPF turquoise/yellow slider colours for Legacy because the Legacy slider
  (inactive/hovered/active widget bg) was near-black on black. In the new model
  the Legacy `ChromeStyle` sources those widget fills from the Legacy `ui` group
  (`button_bg`/`input_bg`/`button_hover` = dark-blue washes) which are non-black
  and visible; the chrome Style carries them, so Legacy sliders stay visible with
  no seam-side special case. A unit test asserts the Legacy inactive widget bg
  comes from the chrome Style and is not `Color32::BLACK`.
  NOTE: the former seam used pure turquoise/yellow for the slider; the new model
  uses the Legacy `ui` surface values. This keeps Legacy sliders visible (the
  bug the hack addressed) without a hardcoded seam branch; the exact Legacy
  slider palette is finalised when the Legacy built-in is rebuilt in Phase 2.
  Phase 1 does NOT rebuild the Legacy built-in.

- **29.7 Read-site migration.** The chrome read sites in
  `crates/ff-desktop/src/shell/render_*.rs` now read the chrome layer
  (`self.palette.chrome_style.*` accessors returning `egui::Color32`) instead of
  flat `palette.ui.*` / `palette.tab_bar.*` / `palette.editor.{accent,foreground,
  background}`:
  - `render_tab_bar.rs`: active/inactive tab bg+text + modified accent.
  - `render_split_region.rs`: per-leaf tab bg+text, focus-highlight accent, drop
    accent, region command-field accent.
  - `render_command_line.rs`: Title_Line band bg+fg; detached command-field accent.
  - `render_status.rs`: key-bar accent + body text; zoom/modified/CAPS accents.
  - `render.rs`: `render_focus_indicator` focus-ring colour (was via ColourToken).
  DOMAIN read sites LEFT UNCHANGED as instructed: the file-tree `category_colour`
  in `ff-explorer-view`, and syntax/gutter colours read via the `ColourToken` API.

## Files changed

- `Cargo.toml` -- egui `serde` feature.
- `crates/ff-theme/Cargo.toml` -- egui dependency.
- `crates/ff-theme/src/chrome_style.rs` -- NEW (ChromeStyle + apply_to_egui + 5 tests).
- `crates/ff-theme/src/lib.rs` -- `pub mod chrome_style;` + `pub use ChromeStyle`.
- `crates/ff-theme/src/palette.rs` -- `chrome_style` field; `chrome`->`gutter`
  (type `GutterColours`); `colour()` arms repointed.
- `crates/ff-theme/src/defaults.rs` -- derive `chrome_style` in all 4 built-ins;
  `*_chrome_colours` -> `*_gutter_colours`; two tests updated to `.gutter`.
- `crates/ff-theme/src/loader.rs` -- derive `chrome_style`; `parse_chrome_colours`
  -> `parse_gutter_colours` (still reads the `[chrome]` TOML section -- file format
  UNCHANGED in Phase 1).
- `crates/ff-theme/src/serialiser.rs` -- `palette.chrome.*` -> `palette.gutter.*`
  (still writes the `[chrome]` section header -- file format UNCHANGED); 1 test
  updated.
- `crates/ff-theme/src/contrast.rs` -- `.chrome` -> `.gutter` on the gutter pair.
- `crates/ff-theme/tests/property_tests.rs`, `tests/integration_tests.rs` --
  `.chrome` -> `.gutter`.
- `crates/ff-desktop/src/shell/render_theme.rs` -- wholesale seam; removed Legacy
  slider injection; dropped now-unused `to_egui_color` import.
- `crates/ff-desktop/src/shell/render_tab_bar.rs`, `render_split_region.rs`,
  `render_command_line.rs`, `render_status.rs`, `render.rs` -- read-site migration
  + unused-import cleanup.
- `crates/ff-desktop/src/shell/helpers.rs` -- removed the now-unused
  `to_egui_color` helper and its `ColourRGBA` import (all callers migrated).

## File-format / serialiser touch (Phase 4 deferral)

The ThemePalette reshape required loader/serialiser edits ONLY to keep them
compiling with the renamed group and the new field: the gutter group is still
persisted under the `[chrome]` TOML section (the on-disk format is UNCHANGED),
and `chrome_style` is DERIVED at load time -- it is NOT serialised. No `version`
field, no `base` resolution, no embedded egui `Style` sub-table were added. Those
are Phase 4 (Task 32). Existing round-trip tests still pass unchanged in behaviour.

## UiColours / TabBarColours / EditorColours decision

KEPT as thin authoring shims (NOT cut over). Rationale: the clean cut-over would
churn the ColourToken API, the ff-theme-editor `EditableToken` get/set, the
serialiser/loader key mapping, and ~25-30 tests that assert the flat fields --
most of which is explicitly Phase 3 (editor rebuild) and Phase 4 (format) work.
The shim keeps Phase 1 to "model + seam + read-site" with zero visible change.
Only the `chrome`->`gutter` rename forced test updates (see below).

## Tests

New (ff-theme `chrome_style`):
- `apply_to_egui_sets_full_visuals_surface_not_just_a_subset` (Req 23.1, 23.5)
- `apply_to_egui_wires_design_tokens_spacing_rounding_shadow` (Req 23.6)
- `apply_to_egui_sets_dark_mode_matching_visual_mode` (Req 23.1)
- `legacy_slider_colours_come_from_the_chrome_style_not_the_seam` (Req 23.5)
- `chrome_style_round_trips_through_egui_serde` (Req 23.2) -- asserts the
  round-trip SUCCEEDS and themable fields are preserved; a whole-struct `==` is
  NOT used because `egui::Style` carries a `number_formatter` closure wrapper that
  never compares equal across a serde round-trip.

Updated (chrome -> gutter rename): `serialise_round_trip_preserves_colours`,
`default_legacy_matches_legacy_colours`,
`legacy_blue_on_black_uses_bright_blue_for_legibility` (defaults.rs),
`theme_serialisation_round_trip_correctness` (property_tests.rs),
`partial_theme_file_inherits_missing_tokens_from_defaults` (integration_tests.rs).
`.mode` / `.name` THEME-command tests left unchanged. The two ff-desktop tests
reading `palette.ui.*` (tests_focus.rs focus_ring, tests_menu_workspace.rs
panel_bg) were left as-is because the shim keeps `ui` present.

## Scoped checks run (exact commands + results)

Run from `c:\workspace\VSC\FileForgeWorkbench` via the pwsh7 wrapper:

- `cargo fmt` -- clean (no output).
- `cargo check -p ff-theme -p ff-desktop -p ff-theme-editor` -- clean (0 warnings
  after unused-import cleanup).
- `cargo clippy -p ff-theme -p ff-desktop -p ff-theme-editor` -- clean (0 warnings).
- `cargo test -p ff-theme -p ff-desktop -p ff-theme-editor --no-fail-fast`:
  - ff-theme lib: **113 passed, 0 failed**
  - ff-theme integration: **7 passed, 0 failed**
  - ff-theme property: **7 passed, 0 failed**
  - ff-theme doc-tests: **2 passed, 0 failed**
  - ff-theme-editor lib: **4 passed, 0 failed**
  - ff-desktop bin: **975 passed, 2 failed** under the default parallel runner.
- Re-ran ff-desktop single-threaded (`cargo test -p ff-desktop --bin ffwb --
  --test-threads=1`): **977 passed, 0 failed**. The two failures above
  (`shell::tests_session::exit_saves_command_history_and_reloads`,
  `shell::tests_menu_workspace::close_workspace_removes_settings_from_config`)
  each PASS in isolation and under single-threaded run -- they are a PRE-EXISTING
  parallel-execution flake (shared process-global env/config state between
  unrelated session/config tests under thread-based `cargo test`; the owner's
  gate uses cargo-nextest which process-isolates per test, per testing.md B048).
  Neither test touches theme chrome; this change does not alter session/config
  code. Not a regression from this change.

## Deferred to later phases (NOT done here, by design)

- Solarized Dark/Light built-ins, Legacy tone-down (#000060), Legacy Soft (Phase 2).
- Theme Editor rebuild / import-export / B081 (Phase 3).
- `version` field, v1->v2 load, embedded egui `Style` sub-table, `base` resolution
  (Phase 4).
- Theme-authoring doc rewrite + TCR status flips (Phase 5).
- Cutting over UiColours/TabBarColours entirely + migrating the two remaining
  ff-desktop `palette.ui.*` field tests (Phase 3, when the editor is rebuilt).

Per the task: docs/status/bugs.md, TCR, the acceptance plan, and tasks.md `[x]`
marks were NOT touched.
