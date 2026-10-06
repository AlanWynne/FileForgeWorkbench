# CR-CH-056 Phase 1 (Task 29) -- egui-native theme model: chrome layer + apply seam

Phase 1 introduces an egui-native CHROME layer (`ChromeStyle`) in `ff-theme`,
backed by `egui::Style` and serialised through egui's own serde derives, carries
it on `ThemePalette`, renames the old `chrome` domain group to `gutter`, collapses
the hand-written `apply_theme` seam into a single wholesale `apply_to_egui`, and
migrates the ~33 chrome read sites in the shell to read the chrome layer. The work
is deliberately a thin-shim slice: the flat `UiColours`/`TabBarColours`/
`EditorColours` groups are KEPT as the authoring surface and the chrome layer is
DERIVED from them, so Phase 1 produces no intended visible change. Built-in colour
VALUES are untouched (no Solarized yet), the on-disk TOML format is unchanged
(gutter still persists under `[chrome]`), and no versioning/base work was done --
all correctly deferred to Phases 2-4.

Watch for: one spec-sanctioned behavioural change -- the Legacy slider's
turquoise/yellow seam injection is removed, so Legacy sliders now take the Legacy
`ui` dark-blue washes until Phase 2 finalises the Legacy chrome Style (confirmed,
required by Req 23.5). The wholesale `set_style` seam now resets the whole `Style`
each frame rather than only `Visuals`; this is behaviour-neutral here because the
app never installs custom `text_styles` (confirmed). The working tree also carries
a large, UNRELATED CR-CH-052 (nav-ladder) change set co-mingled with this Phase 1
work (confirmed) -- a commit-hygiene concern, not a Phase 1 defect.

**Verdict**: APPROVED

## High-level view

The model reshape matches Requirement 23 exactly. `ChromeStyle` stores a real
`egui::Style` plus the handful of FFWB-only chrome colours egui genuinely does not
model (the Title_Line band, per-tab fills/text, accent, focus ring, chrome body
text). The stored `Style` is driven across the full themable `Visuals`/
`WidgetVisuals` surface -- including the open-state widgets, strokes, extreme/
faint/code backgrounds, hyperlink, warn/error, and selection that the old seam
left at `Visuals::dark()` defaults -- and the DesignTokens spacing/radii/shadows
are wired in (Req 23.6), closing the "egui under-driven" gap the findings
identified. The `chrome` -> `gutter` domain-group rename is mechanical and
complete, and the retained domain groups and the `ColourToken::Chrome*` variant
names (deliberately kept) leave the token API and every editor/syntax read site
untouched.

The apply seam is now the single wholesale path Req 23.5 asks for:
`apply_theme` clones the live style, calls `chrome_style.apply_to_egui`, and sets
it. The hand-written ~15-field body and the hardcoded Legacy slider branch are
both gone; `dark_mode` is set from the Theme's VisualMode inside the chrome layer.

The ~33 chrome read-site migration is done and consistent: tab bar, split regions,
command line (Title_Line band), status bar, and the focus-ring indicator all read
`palette.chrome_style.*` accessors; the `to_egui_color` helper and its last import
were removed once every caller migrated. Domain read sites (file-tree
`category_colour`, syntax/gutter via `ColourToken`) are correctly left alone.

The no-visible-regression contract holds for colours: built-in palette values are
byte-identical, the format is unchanged, and `chrome_style` is derived -- not
serialised -- so round-trip tests keep passing. The one genuine behavioural delta
is the Legacy slider, which Req 23.5 explicitly mandates (the seam injection must
be removed and its values become part of the Legacy instance's chrome Style); the
exact ISPF slider palette is a Phase 2 built-in concern and is documented as such.

The TDD evidence is present and honest: five new chrome tests written against the
criteria, each annotated `// Validates: Requirement 23.x`, with the serde
round-trip test correctly asserting field preservation rather than a whole-struct
`==` (egui's `Style` carries a non-comparable `number_formatter` closure). The
rename-driven test updates adjust assertions in place without weakening them.

Scope is respected: no Solarized values, no editor rebuild, no version/base,
no doc rewrite, no `[x]` task marks, no TCR flips (TCR for Req 23 stays NOT
COVERED, matching the Phase 5 deferral).

<details>
<summary>Issues (4)</summary>

1. **Legacy slider appearance changes in Phase 1 (non-blocking, spec-sanctioned)** -- removing the seam injection means Legacy sliders render with the Legacy `ui` dark-blue surface instead of ISPF turquoise/yellow until Phase 2 rebuilds the Legacy chrome Style. Req 23.5 explicitly requires this removal; the note documents the deferral. No action required for Phase 1; ensure Phase 2 (Task 30.3) restores an acceptable Legacy slider palette.
2. **Unrelated CR-CH-052 changes co-mingled in the working tree (non-blocking, hygiene)** -- the diff carries nav-ladder/command-ladder/nav_stack/menu_workspace/tests_command/tests_nav edits and CR-CH-052 TCR PASS flips that are NOT part of CR-CH-056 Phase 1. Isolate the theme work into its own commit so the Phase 1 slice is reviewable and revertable on its own.
3. **`defaults.rs` exceeds the 400-line rule (non-blocking, pre-existing)** -- 734 non-test lines; it was already 897 total before this change, so the violation is pre-existing (a data-constant table), not introduced here. Consider splitting the built-in palette data in a later refactor; not a Phase 1 blocker.
4. **Pre-existing non-ASCII in touched `.rs` files (non-blocking)** -- `lib.rs` still contains em-dash and box-drawing separators on lines this change did not modify; the two added lines are clean ASCII and the new `chrome_style.rs` is fully ASCII. documentation.md's "replace when touched" is a soft directive; worth cleaning opportunistically.

</details>

<details>
<summary>Details</summary>

### Chrome layer type and the full Visuals surface (Req 23.1, 23.2)

`ChromeStyle` stores `pub style: egui::Style` plus the FFWB-only extras
(`title_band_bg/fg`, `tab_active/inactive_bg/text`, `accent`, `focus_ring`,
`foreground`). The split is exactly right: egui models no title band and no tab
bar, so those live as extras; everything egui does model lives in the embedded
`Style`. `from_palette_parts` drives the previously-defaulted surface -- the `open`
widget state, `weak_bg_fill` on every state, `fg_stroke`, `extreme_bg_color`/
`code_bg_color`/`faint_bg_color`, `hyperlink_color`, `warn_fg_color`/
`error_fg_color`, and `selection` fill+stroke -- so a Theme now reaches egui's real
range rather than ~15 fields. The serde requirement is satisfied by enabling egui's
`serde` feature on the workspace dependency and adding `egui` to `ff-theme`; the
chrome layer (de)serialises through egui's own derives.

The round-trip test is the right shape. A whole-struct `==` across serde would fail
on egui's `number_formatter` closure wrapper, so the test asserts the themable
fields (dark_mode, panel_fill, window_fill, active widget fill, window_shadow,
spacing.indent) and the FFWB extras survive the round trip. That proves Req 23.2
without a brittle assertion.

### DesignTokens wired at the seam (Req 23.6)

`from_palette_parts` maps `border_radius(Md)` onto every widget corner radius,
`border_radius(Lg)` onto the window corner radius, `Md` onto the menu corner radius,
`shadow(Lg)`/`shadow(Md)` onto window/popup shadow, and the spacing scale onto
`Style.spacing` (item_spacing, button_padding, menu_margin, indent). The accessor
and type names check out against `design_tokens.rs` (`border_radius(RadiusLevel)`,
`shadow(ShadowLevel) -> &ShadowDef`, `ShadowDef.blur_radius`, `SpacingScale.{xs,sm,
lg}`). The dedicated test asserts spacing.indent, item_spacing.x, window corner
radius, and window shadow all come from the tokens -- previously dead at the seam.

### The single wholesale apply seam (Req 23.5)

`apply_theme` is reduced to clone-live-style / `chrome_style.apply_to_egui` /
`set_style`. `apply_to_egui` overwrites the whole `Style` with the Theme's stored
`Style` and keeps `dark_mode` in sync. The former hand-written body and the Legacy
`if p.mode == Legacy { ... }` slider injection are both removed, and the unused
`to_egui_color` import was dropped from `render_theme.rs`.

One behaviour to be aware of: the old seam used `ctx.set_visuals`, which preserved
any `text_styles`/font config already on the context; the new seam replaces the
entire `Style` every frame with one whose `text_styles` are `Style::default()`'s.
I traced the shell's font path -- `font.rs` has no `apply_to_egui`, nothing calls
`set_fonts`/sets `Style.text_styles`, and all rendering uses explicit
`FontId::monospace(..)` / `TextStyle::Monospace` that resolve against egui's
default text-style map. So resetting `text_styles` to the egui defaults every frame
is behaviour-neutral. If a future phase introduces theme-driven text styles, they
must flow through the chrome `Style` (which is the correct place for them).

### Legacy slider: removed from the seam, sourced from the chrome Style (Req 23.5)

The old seam forced ISPF turquoise track / yellow handle for Legacy because the
Legacy slider widget fills were near-black on black. In the new model the Legacy
`ChromeStyle` sources the inactive/hovered/active widget `bg_fill` from the Legacy
`ui` group (button/input/hover washes), which are non-black, so sliders stay
visible with no seam branch. The dedicated test asserts the applied inactive widget
`bg_fill` comes from the chrome Style and is not `Color32::BLACK`. This is a real
appearance change from the previous turquoise/yellow, but it is exactly what Req
23.5 mandates and the note is explicit that the authentic Legacy slider palette is
finalised when the Legacy built-in is rebuilt in Phase 2. Acceptable for Phase 1.

### `chrome` -> `gutter` rename and retained domain groups (Req 23.3)

The rename is complete and mechanical across `palette.rs` (type `ChromeColours` ->
`GutterColours`, field `chrome` -> `gutter`), `defaults.rs` (all four builders +
two tests), `loader.rs` (`parse_gutter_colours`, still reading the `[chrome]` TOML
section so the format is unchanged), `serialiser.rs` (still writing `[chrome]`),
`contrast.rs`, and the integration/property tests. Crucially, the
`ColourToken::Chrome*` variant NAMES are retained and the `colour()` match arms
simply repoint to `self.gutter.*`, so the token API and every editor/syntax read
site are untouched. The retained groups (syntax/file_tree/decorations/indicators/
style_slots/elements/fonts/design/mode/name) are intact on `ThemePalette`.

### Read-site migration (Req 23.3, 23.5; Req 8.4/8.5 re-expressed)

All the chrome read sites listed in the task migrated to `chrome_style` accessors:
`render_tab_bar.rs` (active/inactive tab bg+text + modified accent),
`render_split_region.rs` (per-leaf tab colours, focus accent, drop accent, region
command-field accent), `render_command_line.rs` (Title_Line band bg+fg, detached
accent), `render_status.rs` (key-bar accent + body text, zoom/modified/CAPS
accents), and `render.rs` (`render_focus_indicator` focus-ring, previously via
`ColourToken::UiFocusRing`). The `to_egui_color` helper and its `ColourRGBA` import
were removed from `helpers.rs` once every caller migrated, and the `use
super::helpers::*;` imports were trimmed where they became unused. Domain read sites
(file-tree `category_colour`; syntax/gutter via `ColourToken`) are left unchanged
as instructed.

### No-visible-regression and scope

Built-in palette colour values are unchanged (the derivation reproduces the old
seam's mapping), so there is no colour regression beyond the sanctioned Legacy
slider. The on-disk format is unchanged: the gutter group still persists under
`[chrome]`, `chrome_style` is derived at load and not serialised, and the loader/
serialiser touches are purely the rename plus the derivation call -- round-trip
tests pass unchanged in behaviour. No `version` field, no `base` resolution, no
embedded `Style` sub-table (all Phase 4). No Solarized values, no Legacy tone-down,
no Legacy Soft (Phase 2). No editor rebuild (Phase 3). No doc rewrite (Phase 5).
`tasks.md` adds the Phase 1-5 breakdown but leaves Task 29 `[ ]`, and the TCR Req
23/24/25 rows stay NOT COVERED -- both consistent with the workflow's "flip after
owner gate" discipline and the Phase 5 TCR task.

### Framework conformance and rust-standards

Dispatch, navigation, and `WorkspaceContext` are untouched by the theme work; the
seam remains the single `apply_theme` path. `chrome_style.rs` is 262 non-test lines
(under 400) and pure ASCII; `render_theme.rs` shrank to 177 lines; `palette.rs` is
342 total. Library code in `chrome_style.rs` has no `unwrap()`/`expect()` outside
the `#[cfg(test)]` module. `defaults.rs` is over 400 non-test lines, but that
predates this change (it was 897 total at HEAD) and is a data-constant table; not
introduced here and reasonably deferrable.

### Verification evidence

The implementation note records the exact scoped commands and results: `cargo fmt`
clean; `cargo check`/`cargo clippy -p ff-theme -p ff-desktop -p ff-theme-editor`
clean (0 warnings); `cargo test` across the three crates green (ff-theme 113 lib +
7 integration + 7 property + 2 doc; ff-theme-editor 4; ff-desktop 977 under
single-threaded). The two ff-desktop failures under the parallel runner are a
documented pre-existing session/config env-isolation flake (B048) unrelated to the
theme chrome, and pass in isolation / single-threaded. The evidence is specific and
leaves no articulable doubt that required a re-run; I spot-checked only the
`DesignTokens` accessor names, the file sizes, and the font-path reasoning above,
all of which held.

</details>

<details>
<summary>File map</summary>

In-scope CR-CH-056 Phase 1 files (match the implementation note):

- `Cargo.toml` -- enable egui `serde` feature.
- `crates/ff-theme/Cargo.toml` -- add egui dependency.
- `crates/ff-theme/src/chrome_style.rs` -- NEW: `ChromeStyle` + `from_palette_parts` + `apply_to_egui` + accessors + 5 tests.
- `crates/ff-theme/src/lib.rs` -- `pub mod chrome_style;` + re-export.
- `crates/ff-theme/src/palette.rs` -- `chrome_style` field; `chrome` -> `gutter` (type `GutterColours`); `colour()` arms repointed.
- `crates/ff-theme/src/defaults.rs` -- derive `chrome_style` in all built-ins; `*_chrome_colours` -> `*_gutter_colours`; test updates.
- `crates/ff-theme/src/loader.rs` -- `parse_gutter_colours` (still reads `[chrome]`); derive `chrome_style`.
- `crates/ff-theme/src/serialiser.rs` -- `palette.gutter.*` (still writes `[chrome]`); test update.
- `crates/ff-theme/src/contrast.rs` -- `.chrome` -> `.gutter`.
- `crates/ff-theme/tests/{property_tests,integration_tests}.rs` -- `.chrome` -> `.gutter`.
- `crates/ff-desktop/src/shell/render_theme.rs` -- wholesale seam; Legacy injection removed.
- `crates/ff-desktop/src/shell/{render_tab_bar,render_split_region,render_command_line,render_status,render}.rs` -- chrome read-site migration.
- `crates/ff-desktop/src/shell/helpers.rs` -- removed `to_egui_color`.
- `docs/specs/theme-and-appearance/{requirements,design,tasks}.md`, `docs/quality/TCR.md`, `docs/status/change-log.md`, `docs/project-management/...` -- gate docs (Req 23/24/25, Task 29-33 breakdown).

Out-of-scope (CR-CH-052 nav-ladder, co-mingled in the working tree -- NOT part of this review's target): `shell/commands*.rs`, `shell/dispatch.rs`, `shell/nav_stack.rs`, `shell/mod.rs`, `menu_workspace/defaults.rs`, `shell/tests_{command,nav,focus,menu_workspace,session,...}.rs`, and the CR-CH-052 TCR PASS flips.

Full diff: `git diff HEAD` from the repo root.

</details>
