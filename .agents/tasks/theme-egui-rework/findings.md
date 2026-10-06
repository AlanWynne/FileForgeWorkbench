# Theme Model -> egui-native Rework: Scoping Findings

**Read-only investigation. No source changed.** Scope: `crates/ff-theme/`,
`crates/ff-theme-editor/`, `crates/ff-desktop/src/` (palette read sites + egui style
seam), `docs/specs/theme-and-appearance/`.

## Scope summary (framework-change magnitude; hybrid vs full; phase count)

This is a genuine FRAMEWORK CHANGE per `framework-conformance.md`: it reshapes the
public `ThemePalette` type (a load-bearing type consumed app-wide via `self.palette`)
and the on-disk Theme_File TOML format. It therefore requires explicit owner
confirmation and a full requirements-gate pass before any code.

The magnitude is **moderate and well-contained**, not a rewrite. The reason: the
palette is already split into semantic groups, and the single egui application seam is
ONE function -- `WorkbenchShell::apply_theme` in
`crates/ff-desktop/src/shell/render_theme.rs:18`. Today that function maps only a
handful of palette fields onto `egui::Visuals` and leaves most egui appearance at
`Visuals::dark()` defaults. The gap is "egui is under-driven," not "egui is wrongly
modelled."

**Recommendation: HYBRID, not full replacement.** Split the model into (1) a
**chrome layer** that mirrors egui's own `Style`/`Visuals`/`WidgetVisuals` primitives
(window/panel fills, per-state widget fills+strokes, rounding, selection, hyperlink,
extreme/faint bg, text styles, spacing) and (2) **domain groups that egui does NOT
model** -- `syntax`, `chrome` (editor gutter), `file_tree`, `decorations`, `indicators`,
`style_slots` -- which stay as first-class palette groups read directly by the editor /
file-tree / syntax subsystems. ISPF/Legacy becomes ONE instance built on the general
chrome model (its ISPF semantics live in the domain groups, exactly as now). A full
replacement would needlessly throw away the domain groups egui cannot express.

**Backward-compatible load is feasible** via a `version` field + the loader's existing
default-and-fill behaviour (every token already falls back to the mode default when
absent), so old `themes/*.toml` keep loading. There are **no user theme files shipped
in-repo** (`themes/` is created empty at first launch), so migration risk is limited to
end-user files.

**Rough phase count: 5** -- (1) new chrome model type + egui mapping; (2) built-in
instances incl. Solarized Dark/Light + retrofitted Legacy/Legacy Soft; (3) Theme Editor
token-set rebuild; (4) TOML version + migration/defaulting; (5) docs rewrite (ISPF-first
-> egui-native with ISPF retrofit).

---

## A. CURRENT MODEL -- `ThemePalette` groups: chrome/UI vs domain

`ThemePalette` is defined at `crates/ff-theme/src/palette.rs:181-213`. Fields:

| Field | Type | Classification |
|-------|------|----------------|
| `name: String` | - | metadata |
| `mode: VisualMode` | `crate::mode` | metadata (Dark/Light/HighContrast/Legacy) |
| `editor: EditorColours` | palette.rs:19-35 | **MIXED**: `background`/`foreground`/`accent`/`muted` are chrome-adjacent; `current_line_background`, `selection_secondary_background`, `modified_indicator` are editor-domain |
| `syntax: SyntaxColours` | palette.rs:38-59 | **DOMAIN** (egui does not model token colours) |
| `file_tree: FileTreeColours` | palette.rs:62-77 | **DOMAIN** (file-category colours) |
| `tab_bar: TabBarColours` | palette.rs:80-96 | **CHROME** (egui-paintable; app paints it manually today) |
| `chrome: ChromeColours` | palette.rs:99-116 | **DOMAIN** (editor gutter: line numbers, fold margin, cursor row/col, margin separator -- egui does not model an editor gutter) |
| `decorations: DecorationColours` | palette.rs:119-137 | **DOMAIN** (search/error/warning underlines, change markers, bookmark) |
| `indicators: IndicatorColours` | palette.rs:140-156 | **DOMAIN** (find/brace match + 32 user-defined) |
| `ui: UiColours` | palette.rs:159-196 | **CHROME** (panel/button/input/scrollbar/tooltip/menu/focus-ring -- the core egui-paintable set) |
| `style_slots: StyleSlotTable` | style_slot.rs | **DOMAIN** (256 Scintilla-style syntax slots) |
| `fonts: FontConfig` | font.rs | **CHROME-adjacent** (egui `Style`/`FontDefinitions` has text styles + sizes) |
| `design: DesignTokens` | design_tokens.rs | **CHROME-adjacent** (spacing/radius/shadow/animation -> egui `Spacing`, `Rounding`, `Shadow`) |
| `elements: ElementColourMap` | element.rs | **DOMAIN** (Scintilla element colours: selection/caret/whitespace/fold, with alpha) |

Colour access is centralised through `ThemePalette::colour(token: ColourToken)` --
one exhaustive match (`palette.rs:231-298`) over `ColourToken`. `DesignTokens`
(`design_tokens.rs`) already carries spacing/border-radius/shadow/animation scales that
map directly onto egui's `Spacing`/`Rounding`/`Shadow`, but are **not applied** at the
seam (see B).

**Hybrid line (recommended):** chrome layer = `ui` + `tab_bar` + the chrome-adjacent
parts of `editor` (`background`/`foreground`/`accent`) + `fonts` + `design`. Domain
layer (unchanged) = `syntax`, `chrome` (gutter), `file_tree`, `decorations`,
`indicators`, `style_slots`, `elements`, and the editor-domain `editor` fields.

## B. EGUI APPLICATION SEAM -- what is set vs the gap

The sole seam is `WorkbenchShell::apply_theme(&self, ctx)` in
`crates/ff-desktop/src/shell/render_theme.rs:18-55`. It starts from
`egui::Visuals::dark()` (render_theme.rs:20) and sets ONLY:

- `panel_fill` <- `editor.background` (:20)
- `window_fill` <- `ui.panel_bg` (:21)
- `window_stroke` <- `ui.panel_border` (:22)
- `override_text_color` <- `ui.menu_bar_fg` (:26) -- a global text-colour override
- `widgets.noninteractive.bg_fill`/`fg_stroke` (:27,:29)
- `widgets.inactive.bg_fill`/`fg_stroke` (:30,:32)
- `widgets.hovered.bg_fill`/`fg_stroke` (:33,:35)
- `widgets.active.bg_fill`/`fg_stroke` (:36,:38)
- `selection.bg_fill` (accent * 0.35) + `selection.stroke` (:39,:40)
- Legacy-only hardcoded slider track/handle override (:44-52) -- a smell: hardcoded
  ISPF colours injected at the seam instead of coming from the palette.

Applied via `ctx.set_visuals(visuals)` (:53). The follow-OS block reads
`ctx.style().visuals.dark_mode` (`shell/update.rs:242`).

**egui appearance NOT currently themed (left at `Visuals::dark()` defaults) -- the gap:**

- `widgets.open.*` (the open-combo/menu state) -- never set.
- Per-`WidgetVisuals` `bg_stroke`, `rounding`/`corner_radius`, `expansion`,
  `weak_bg_fill` -- only `bg_fill` + `fg_stroke` are set; borders/rounding/expansion are
  default for every state.
- `hyperlink_color` -- default.
- `extreme_bg_color` (text-edit/`TextEdit` background), `faint_bg_color` (striped rows),
  `code_bg_color` -- default; so themed input backgrounds are inconsistent with
  `ui.input_bg`.
- `warn_fg_color`, `error_fg_color` -- default.
- `window_rounding`/`window_corner_radius`, `window_shadow`, `menu_rounding`,
  `popup_shadow`, `menu_corner_radius` -- default (DesignTokens radius/shadow unused).
- `selection.stroke` is set but `selection.bg_fill` uses a hardcoded 0.35 multiply
  rather than a palette-driven selection colour.
- `resize_corner_size`, `clip_rect_margin`, `button_frame`, `collapsing_header_frame`,
  `indent_has_left_vline`, `striped`, `slider_trailing_fill` -- default.
- `Style.spacing` (`item_spacing`, `button_padding`, `window_margin`, `indent`,
  `scroll`) -- **never set** from `design.spacing`; DesignTokens spacing is dead at the
  seam.
- `Style.text_styles` / `Style.override_font_id` -- fonts are applied elsewhere (not in
  `apply_theme`); the text-style map (Heading/Body/Monospace/Button/Small sizing) is not
  driven by the theme here.
- `visuals.dark_mode` boolean -- not set to match the theme's mode.

Net: the current model exposes maybe ~15 of egui's ~50 themable `Visuals`/`Style`
fields. The rework's chrome layer should map the full set (or an explicit curated
subset) so a theme can drive egui's real appearance range.

## C. PALETTE READ SITES (blast radius)

Palette is read as `self.palette.<group>.<field>` in `ff-desktop` and as
`palette.<group>` in sibling view crates. Grouped non-test read sites:

**ff-desktop (chrome groups -- the ones the rework touches most):**
- `ui.*` -- `render_theme.rs` (apply_theme, ~10 reads), `render_command_line.rs:58-59`
  (`primary_menu_bg`, `menu_bar_fg`), `shell/render.rs:331` (`UiFocusRing` via token).
  ~12 reads.
- `editor.*` (accent/foreground/background) -- `render_theme.rs:20,29,39,40`,
  `render_status.rs:31,32,151,160,165`, `render_split_region.rs:158,177,288`,
  `render_command_line.rs:200`, `render_tab_bar.rs:32`. ~13 reads (mostly
  `editor.accent` used as a generic chrome accent).
- `tab_bar.*` -- `render_tab_bar.rs:28-31` and `render_split_region.rs:57-60` (4 each).
  ~8 reads.
- `mode` -- `render_theme.rs:44,173` (Legacy branch + POM colour choice),
  `shell/update.rs` (follow-OS). ~3 reads.
- `name` -- theme commands + tests (`commands_theme.rs:36`, several tests).

**ff-desktop (domain groups -- read indirectly):** editor syntax/gutter colours are
read through the token API `palette.colour(ColourToken::...)` (e.g.
`primary_option_menu.rs:123` `from_palette`, `shell/render.rs:331`), not via
`self.palette.syntax.*` directly, so the editor paint path is insulated by the
`ColourToken` enum.

**Sibling view crates (DOMAIN):**
- `ff-explorer-view/src/lib.rs:524-535` -- `palette.file_tree.{binary,structured,text,
  unknown,directory,symlink}` in `category_colour`. 6 reads, 1 site.
- `ff-syntax-highlighting` -- consumes `style_slots` / `ColourToken` syntax colours.
- `ff-text-decorations`, `ff-caret-selection`, `ff-whitespace`/guides -- documented
  consumers (requirements Cross-References) of `decorations`/`indicators`/`elements`.

**Rough totals (non-test):** `ui` ~12, `editor` ~13, `tab_bar` ~8, `file_tree` 6,
`mode`/`name` ~6, plus the token-API path for syntax/gutter. The **chrome groups**
(`ui`/`editor`/`tab_bar`) are where a model change bites -- ~33 call sites, almost all
in `crates/ff-desktop/src/shell/render_*.rs`. The domain groups are read in few,
localized places (one file-tree site; syntax via the token API), so a hybrid that keeps
domain groups intact leaves those call sites untouched.

## D. ON-DISK FORMAT + MIGRATION

**Serialiser** (`crates/ff-theme/src/serialiser.rs:14`) writes a flat-ish TOML with a
header comment, top-level `name` + `mode`, then sections:
`[editor]`, `[syntax]`, `[file_tree]`, `[tab_bar]`, `[chrome]`, `[decorations]`,
`[indicators]` (incl. a `user_defined = [...]` 32-array), `[ui]`, `[font.monospace]`,
`[font.proportional]`, `[design.spacing]`, `[design.border_radius]`, and only-if-custom
`[style_slots.N]` tables. Colours are `#RRGGBB` / `#RRGGBBAA` (serialiser.rs:316).

**Loader** (`crates/ff-theme/src/loader.rs:28`): parses the TOML table; the effective
`mode` is the file's `mode` field if present else the caller's argument
(loader.rs:42-48, B063). Every section is parsed with per-key fallback to the mode
default (`parse_*_colours` helpers), so **any omitted token inherits the built-in
default** -- this is the key to backward compatibility.

**Inheritance / version:** there is a `base` key -- but it is **read and discarded**:
`let _base_name = table.get("base")...` (loader.rs:57) with a `_` binding; inheritance
from a named base is NOT actually resolved in the loader (requirements 14.4/15.5 specify
it; the code only does default-fill). **There is NO `version` field** in the format
today.

**Shipped user themes:** none. `themes/` is created empty on first launch (Req 19.1);
built-ins are compiled-in and never written to disk (Req 18.2 / 19.2, CR-CH-019). A
file search found no `themes/*.toml` in-repo (the only in-repo TOMLs are Cargo/config
files). So migration only concerns end-user-authored files.

**Migration if the model changes:** add a top-level `version` key. The loader branches
on it: absent/`1` = legacy section layout (parse as today, map old `[ui]`/`[editor]`
keys into the new chrome layer); `2` = new egui-native layout. Because every token
already defaults when absent, an old v1 file loads into a v2 palette by mapping the keys
it has and defaulting the new chrome fields. No destructive migration needed; optionally
re-serialise to v2 on next Save.

## E. THEME EDITOR COUPLING

`crates/ff-theme-editor/src/lib.rs` derives its editable list from a **fixed**
`EditableToken` enum with a hand-written `EditableToken::ALL` array (lib.rs:52-67) -- it
is NOT reflected from the palette. Currently 14 tokens, all in the `ui` + `editor`
chrome groups (lib.rs:34-49), deliberately the "visible chrome" subset (Req 20.3). Each
token has hand-written `label()` (lib.rs:70), `get()` (lib.rs:88), and `set()`
(lib.rs:108) matches against palette fields. State is positional: `hex_buffers: Vec<String>`
indexed by `EditableToken::ALL` order (lib.rs:197-203). A test asserts
`EditableToken::ALL.len() == 14` (lib.rs end).

**Rebuild cost if the token set changes:** moderate but mechanical. The editor would
need: (1) a widened/replaced `EditableToken` enum + `ALL` + three match arms per token
(label/get/set); (2) the `== 14` test updated; (3) the full-shell first-Tab egui_kittest
test (workspace-conformance) re-pointed if the first control changes. The render loop
(lib.rs:226) iterates `EditableToken::ALL` generically, so adding tokens is additive --
the loop, swatch, validation, and advisory code need no structural change. If the rework
exposes the full egui chrome set, consider generating `EditableToken` from the chrome
layer's fields (reflection/macro) to avoid a 50-arm hand-written match.

## F. SPEC + TEST FOOTPRINT

**Requirements written in the current (ISPF-first / fixed-group) model that an
egui-native model would SUPERSEDE or REWORD** (`docs/specs/theme-and-appearance/requirements.md`):

- **Req 1** (Theme Configuration File) -- 1.6 partial-inherit stays; wording tied to
  current sections may need the `version`/chrome-layer note. REWORD.
- **Req 2** (Theme Palette Structure) -- enumerates the exact groups/fields (editor,
  syntax, file_tree, tab_bar, chrome, decorations, indicators, ui). The chrome groups
  (`tab_bar`, `ui`, chrome parts of `editor`) are SUPERSEDED by the egui-native chrome
  layer; domain groups stay. Major REWORD.
- **Req 5** (Visual Modes / contrast) -- 5.6 AAA contrast contract stays but is
  re-expressed against the new chrome fields. REWORD.
- **Req 6** (Design System Tokens) -- becomes "map onto egui `Spacing`/`Rounding`/
  `Shadow`"; currently unused at the seam (B). REWORD + newly WIRE.
- **Req 8** (Replacing Hardcoded Colours) -- 8.1-8.6 enumerate per-group sourcing; the
  chrome subset is restated in egui terms. REWORD.
- **Req 13** (Legacy ISPF semantics) -- RETAINED but RE-FRAMED as "the Legacy instance
  of the general chrome model," per the owner's directive to retrofit ISPF onto an
  egui-native base. REWORD (keep the colour contract).
- **Req 14** (user-configurable tokens / custom themes) -- "every colour token
  overridable" restated for the new field set; `base` inheritance (14.4) still only
  default-filled in code (see D). REWORD.
- **Req 18** (Default Legacy + built-ins) -- the FOUR built-ins change to incl.
  Solarized Dark/Light (replacing Catppuccin per user messages 6/7) + Legacy + Legacy
  Soft; the ISPF-attribute framing is retrofit. REWORD + new built-ins.
- **Req 20** (Theme Editor Context) -- 20.3 editable-token wording follows the new set
  (E). REWORD.
- **Req 21** (Non-Monochrome Chrome for Dark/Light) -- written entirely in current
  `ui`/`tab_bar` field terms (`ui.panel_bg`/`button_bg`/`input_bg` hierarchy,
  `tab_bar.active_bg`, `ui.primary_menu_bg`, `ui.focus_ring`). SUPERSEDED by the
  egui-native chrome layer (these become egui `Visuals` fields). Major REWORD.
- **Req 22** (Legacy blue-on-black legibility) -- names specific Legacy fields
  (`chrome.line_number_fg`, `syntax.comment`, etc.) in domain groups that mostly SURVIVE;
  minor REWORD only if field names change.

Supersede/major-reword set: **Req 2, 21** (chrome structure). Reword/retrofit:
**Req 1, 5, 6, 8, 13, 14, 18, 20**. Mostly-survives: **Req 22** and the domain parts of 2.

**Test churn (functions asserting specific palette group fields):**
- `crates/ff-theme/src/*` + `tests/*`: round-trip tests
  (`serialise_round_trip_preserves_colours`, `..._legacy_mode`, `..._fonts`,
  serialiser.rs), loader tests (`all_ui_colour_tokens_overridable_via_toml`,
  `base_inheritance_fills_missing_tokens`, `load_*` ~10 fns, loader.rs), property tests
  (`theme_serialisation_round_trip_correctness`, `high_contrast_mode_wcag_aaa_*`,
  `element_colour_alpha_enforcement_correctness`, property_tests.rs), defaults tests
  (`dark_palette_is_complete`, `high_contrast_fg_bg_pairs_meet_wcag_aaa`, defaults.rs).
  ~20+ fns reference specific group fields.
- `crates/ff-theme-editor/src/lib.rs`: `load_working_initialises_hex_buffers`,
  `editable_token_get_set_round_trips`, `all_editable_tokens_have_labels` (asserts
  `len()==14`). 3 fns.
- `crates/ff-desktop/src/...`: `tests_focus.rs:416` (`palette.ui.focus_ring`),
  `tests_menu_workspace.rs:1740` (`shell.palette.ui.panel_bg`), POM colour tests
  (`primary_option_menu.rs` `legacy_pom_colours_match_ispf_spec`,
  `pom_colours_inherited_uses_placeholder`), plus the many `shell.palette.mode/.name`
  THEME-command tests (`tests_menu_workspace.rs` ~10, `tests_misc.rs`). The `.mode`/
  `.name` tests are unaffected by a field-structure change; the `.ui.*` field tests need
  updating.

**Estimate:** ~25-30 test functions touch specific group FIELDS and would need updating;
the larger set of THEME-command tests (asserting `.mode`/`.name`) is unaffected by a
hybrid that keeps `mode`/`name` and the domain groups.

## G. RECOMMENDATION INPUTS

**(i) Hybrid vs full replacement -- HYBRID is cleaner.** egui's `Style`/`Visuals` has no
concept of syntax-token colours, an editor line-number gutter, file-type colours, text
decorations, or Scintilla style slots. Those domain groups (`syntax`, `chrome`,
`file_tree`, `decorations`, `indicators`, `style_slots`, `elements`) MUST remain
first-class palette groups regardless. The real problem (B) is that the chrome subset is
hand-mapped onto a tiny slice of egui and the rest of egui is left at defaults. So the
clean design is: a **chrome layer modelled on egui's own primitives** (ideally
serialise/deserialise an `egui::Style`-shaped struct, or a 1:1 mirror of it) applied
wholesale at the seam, PLUS the retained domain groups. Legacy/ISPF becomes one chrome
instance (its ISPF colours fill the chrome layer) with the ISPF semantics still carried
by the domain groups -- directly satisfying the owner's "retrofit ISPF onto egui" ask.

**(ii) Smallest set of new/changed public types in `ff-theme`:**
- NEW `ChromeStyle` (or `EguiChrome`) struct: an egui-native, serde-serialisable mirror
  of the themable subset of `egui::Visuals` + `Style.spacing` + rounding/shadow (fed
  from `DesignTokens`) + text styles. This replaces the chrome role of `UiColours` +
  `TabBarColours` + the chrome-adjacent `EditorColours` fields.
- CHANGE `ThemePalette`: add `chrome_style: ChromeStyle` (new chrome layer); keep
  `syntax`, `chrome` (rename to `gutter` to avoid clashing with the new chrome layer),
  `file_tree`, `decorations`, `indicators`, `style_slots`, `elements`, `fonts`,
  `design`, `mode`, `name`. Consider keeping `UiColours`/`TabBarColours` as deprecated
  shims during migration so the ~33 call sites can migrate incrementally.
- ADD a `version: u32` field (format versioning) and actually RESOLVE `base` in the
  loader (currently discarded, D).
- A single `apply_to_egui(&self, &mut egui::Style)` on `ChromeStyle` to centralise the
  seam, replacing the hand-written body of `apply_theme`.

**(iii) Backward-compatible load -- FEASIBLE.** The loader already fills every absent
token from the mode default, so an old v1 file (no `version`, old `[ui]`/`[editor]`
sections) loads by mapping present keys into the chrome layer and defaulting the new
egui fields. Gate it on the new `version` key: absent => v1 mapping path; `2` => native
path. No user files ship in-repo, so only end-user files are at risk, and they degrade
gracefully rather than failing. Optionally re-serialise to v2 on the next Save via the
Theme Editor.

**(iv) Rough phase breakdown (5):**
1. **Model + egui mapping** -- add `ChromeStyle` + `version`, implement
   `apply_to_egui`, rewrite `apply_theme` (render_theme.rs) to call it, resolve `base`
   in the loader. Keep domain groups unchanged.
2. **Built-in instances** -- Solarized Dark + Solarized Light (replacing Catppuccin per
   user messages 6/7), retrofitted Legacy (toned-down primary-menu per message 5/7) and
   new Legacy Soft; High-Contrast retained. Each is a `ChromeStyle` + domain groups.
3. **Theme Editor rebuild** -- widen/replace `EditableToken` for the new chrome field
   set (consider generating it), update the `len()` test and the first-Tab
   egui_kittest test.
4. **Migration** -- `version` branching in the loader, v1->v2 key mapping, round-trip
   tests for both versions, re-serialise-on-Save.
5. **Docs** -- rewrite `theme-and-appearance` requirements (Req 2/21 superseded;
   1/5/6/8/13/14/18/20 reworded) to describe the egui-native model with ISPF as a
   retrofit, plus a user-facing theme-authoring doc keyed to egui primitives.

---

### Evidence index (file:line)
- Palette groups + `colour()` match: `crates/ff-theme/src/palette.rs:19-298`
- egui seam: `crates/ff-desktop/src/shell/render_theme.rs:18-55`
- Follow-OS read of egui dark_mode: `crates/ff-desktop/src/shell/update.rs:242`
- Serialiser sections: `crates/ff-theme/src/serialiser.rs:14-317`
- Loader + discarded `base`, no `version`: `crates/ff-theme/src/loader.rs:28-85` (`:57`)
- DesignTokens (unused at seam): `crates/ff-theme/src/design_tokens.rs`
- Built-ins incl. ISPF constants: `crates/ff-theme/src/defaults.rs`
- Theme Editor fixed token list: `crates/ff-theme-editor/src/lib.rs:34-120,196-203`
- File-tree domain read site: `crates/ff-explorer-view/src/lib.rs:524-535`
- Chrome read sites: `render_tab_bar.rs:28-32`, `render_split_region.rs:57-60`,
  `render_status.rs:31-32`, `render_command_line.rs:58-59,200`, `shell/render.rs:331`
- Requirements: `docs/specs/theme-and-appearance/requirements.md` (Req 1,2,5,6,8,13,14,
  18,20,21,22)
```
