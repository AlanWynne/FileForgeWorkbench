# Theme Authoring Guide (egui-Native Theme Model)

This guide explains how a FileForgeWorkbench (FFWB) **Theme** works and how to
author, copy, save, import, and export one. It describes the model that shipped
with CR-CH-056. It is a USER- and AUTHOR-facing companion to the approved
specification; the binding criteria live in
`docs/specs/theme-and-appearance/requirements.md` (Requirements 23, 24, 25) and
the architecture in `docs/specs/theme-and-appearance/design.md`
(section "CR-CH-056 egui-Native Theme Model"). Where this guide and those files
differ, those files win.

Content was rephrased for clarity; the authoritative wording is in the spec.

---

## 1. What a Theme is

FFWB is a Windows egui application, not an ISPF terminal. A **Theme** is the
user-facing concept: a named, selectable, versioned set of visual settings. The
ISPF / mainframe "Legacy" look is just ONE retrofitted instance of the general
model, not the shape the model is built around.

egui itself has NO concept of a "theme". Its vocabulary is `egui::Style`, which
contains `Visuals` (colours per widget state), `WidgetVisuals`, `Spacing`, text
styles, and corner-radius / shadow fields. So internally a Theme PRODUCES an
`egui::Style` for the application chrome, PLUS a set of domain colour groups that
egui does not model.

A Theme therefore has two parts:

1. A **chrome layer** -- the egui-native part. It configures the full themable
   surface of `egui::Style` / `Visuals` / `WidgetVisuals`: window and panel
   fills, per-state widget fills and strokes, selection, hyperlink colour,
   extreme/faint/code backgrounds, warn/error colours, corner radius, shadows,
   and the `dark_mode` flag. This is what paints the real look of the window.
2. The **domain groups** -- the FFWB-specific colours egui does NOT model, kept
   as first-class groups: `syntax`, `gutter`, `file_tree`, `decorations`,
   `indicators`, `style_slots`, and `elements`. The editor, file tree, syntax
   highlighter, and decoration subsystems read these directly.

A Theme also carries metadata: a `name` and a `VisualMode`
(Dark / Light / HighContrast / Legacy). The application never shows egui's
`Style` / `Visuals` vocabulary as the user-facing concept -- the word shown in
the UI and used in commands (`THEME <name>`) is always "Theme".

### Terminology (Theme vs egui Style / Visuals)

| Term | Meaning |
|------|---------|
| **Theme** | The FFWB user-facing concept: a named, selectable, versioned set of visual settings. Carries `name` + `VisualMode`; PRODUCES an `egui::Style` (chrome) plus the domain groups. |
| **egui `Style`** | egui's top-level appearance struct: `Visuals`, `Spacing`, text styles, corner radius, shadow. The closest egui equivalent to a theme; it is what the chrome layer configures and applies. |
| **`Visuals`** | The colour portion of `egui::Style`: fills, per-state `WidgetVisuals`, selection, hyperlink, extreme/faint/code backgrounds, warn/error colours, shadows, and `dark_mode`. |
| **`WidgetVisuals`** | Per-interaction-state (noninteractive / inactive / hovered / active / open) fills, strokes, corner radius, and expansion inside `Visuals`. |
| **Chrome layer** | The part of a Theme that configures `egui::Style` (serialised via egui's own serde). "Chrome" means the application's egui-painted surface. |
| **Domain groups** | The FFWB-specific colour groups egui does NOT model (syntax, gutter, file_tree, decorations, indicators, style_slots, elements). |
| **gutter group** | The editor line-number / fold-margin / cursor-row gutter colours. This is the domain group formerly named `chrome`, RENAMED to `gutter` so it does not clash with the new egui chrome layer. |

---

## 2. How a Theme reaches the screen

A Theme is applied to egui through a SINGLE seam,
`WorkbenchShell::apply_theme` (in
`crates/ff-desktop/src/shell/render_theme.rs`). The seam performs a WHOLESALE
`chrome.apply_to_egui(&mut egui::Style)` and then `ctx.set_style(style)`. There
is no per-field hand mapping and no special-case colour injection at the seam.

At that same seam the Design_Tokens (spacing, border radii, shadows) are wired
onto the egui `Style`:

- spacing -> `Style.spacing` (item spacing, button padding, menu margin, indent)
- border radii -> the corner-radius fields of `Visuals` / `WidgetVisuals`
- shadows -> `Visuals.window_shadow` / `popup_shadow`

The chrome layer also sets `visuals.dark_mode` to match the Theme's
`VisualMode` (Light produces `false`; Dark / Legacy / HighContrast produce
`true`).

The domain groups are read directly by their subsystems through the stable
`ColourToken` API or the groups themselves -- those read sites did not change
when the chrome layer was introduced.

---

## 3. The theme file format (versioned TOML, version 2)

Theme files are TOML. TOML was chosen for consistency with the existing
`menus/*.toml` and configuration files. A current theme file carries, at the top
level:

- `version = 2` -- the format version (an integer). An absent `version` is
  treated as version 1 (the legacy layout) and still loads.
- `egui_version = "0.33"` -- records the egui version the embedded chrome
  `Style` blob was written against, so a future egui upgrade is detectable.
- `name = "..."` -- the Theme name.
- `mode = "dark" | "light" | "high_contrast" | "legacy"` -- the `VisualMode`,
  persisted so a theme saved from Legacy reloads as Legacy.

### 3.1 The flat authoring groups are authoritative

The body of the file is a set of flat authoring groups, each a TOML table of
`#RRGGBB` / `#RRGGBBAA` colour values. These groups are AUTHORITATIVE: the chrome
`egui::Style` is DERIVED from them on load, and the editor edits them.

- `[editor]` -- content-area background / foreground / accent / muted plus the
  editor-domain fields (modified indicator, current-line background, secondary
  selection background).
- `[syntax]` -- token colours (keyword, comment, string, number, operator,
  type, function, macro, preprocessor, default text).
- `[file_tree]` -- file-category colours (binary, structured, text, unknown,
  directory, symlink).
- `[tab_bar]` -- active/inactive tab background and text, modified indicator,
  close button, drop target.
- `[chrome]` -- the editor GUTTER colours (line numbers, fold margins, cursor
  row/column, margin separator). NOTE: the TOML SECTION is still named
  `[chrome]` for file-format compatibility; the in-memory group it loads into is
  the renamed `gutter` group. This section is NOT the egui chrome layer.
- `[decorations]` -- search highlight, error/warning/info underlines, change
  markers, bookmark.
- `[indicators]` -- find match, brace match/mismatch, hotspot underline, and the
  user-defined indicator array.
- `[ui]` -- the general UI authoring colours (panel, button, input, scrollbar,
  tooltip, menu-bar foreground, primary-menu background). The chrome `egui::Style`
  (window/panel fills, per-state widget fills, focus ring, title band, tab fills)
  is derived from these `[ui]` / `[tab_bar]` / `[editor]` values on load.
- `[font.monospace]` / `[font.proportional]` -- font families and base size.
- `[design.spacing]` / `[design.border_radius]` -- the Design_Tokens.
- `[style_slots.N]` -- any defined 256-entry style slots beyond the default.

### 3.2 The embedded egui chrome Style

After the authoring groups, the file embeds the egui-native chrome `Style`
(plus the FFWB-only chrome extras the tab bar and title band need) as a
`[chrome_style]` sub-table, serialised through egui's OWN serde derives on
`egui::Style`. The nested egui `Style` appears as `[chrome_style.style]`.

This embed is ADDITIVE and is NOT the authoritative source. On load the chrome
is RE-DERIVED from the flat authoring groups; the embedded `[chrome_style]` is
read tolerantly as a snapshot for the record and for forward compatibility, and
it never overrides the derived chrome. Because the derive is deterministic from
the flat groups, re-serialising a loaded theme produces byte-identical TOML.

### 3.3 Load tolerance and backward compatibility

- A file with no `version` (version 1) loads backward-compatibly: the keys it
  has are mapped and every absent chrome/egui field is filled from the built-in
  default for the active `VisualMode`. An old file never fails to load.
- The embedded `[chrome_style]` sub-table is deserialised VERSION-TOLERANTLY:
  missing egui fields take egui's defaults (or the mode default), and extra or
  unrecognised egui fields are ignored. A `Style` written by a different egui
  version still loads; on any deserialise error a WARN is logged and the derived
  chrome is used.
- Unknown or extra TOML sections are ignored without error.

---

## 4. The `base` inheritance mechanism

A theme file may declare a parent with a top-level `base` key:

```toml
version = 2
name = "My Dark"
mode = "dark"
base = "Default Dark"

[syntax]
keyword = "#FF8800"
```

The loader RESOLVES `base`: every token not explicitly defined in the file is
inherited from the named base theme (a built-in, or a previously defined user
theme). Only tokens absent from BOTH the file and the base fall back to the mode
default. In the example above, every colour except `syntax.keyword` comes from
`Default Dark`.

- If a declared `base` cannot be resolved, the loader logs a WARN naming the
  unresolved base and falls back to the mode default for the unresolved tokens,
  without failing the load.
- A base chain is walked with cycle detection (bounded depth): a loop
  (A bases B, B bases A) is detected, the base is broken, and a WARN is emitted
  so the load always terminates.

---

## 5. The built-in themes (five instances of the model)

Every built-in is an INSTANCE of the general model (a chrome `Style` plus the
domain groups), compiled into the binary and code-only -- the built-ins are
never written to the themes directory. There are five:

- **Default Dark** -- a Solarized Dark instance. Backgrounds base03 `#002B36` /
  base02 `#073642`; foregrounds base0 `#839496` / base1 `#93A1A1`.
- **Default Light** -- a Solarized Light instance. Backgrounds base3 `#FDF6E3` /
  base2 `#EEE8D5`. (The body foreground is a slightly deepened Solarized base01
  tone, `#4E5F64`, so body text clears the WCAG AA advisory on the lighter
  surfaces; the literal base00-on-base3 pair is below AA.)
- **Default High Contrast** -- unchanged; meets the WCAG AAA 7:1 contract.
- **Default Legacy** -- the ISPF retrofit. Identical to the original ISPF
  palette except the primary option-menu / title band is toned from the full
  ISPF blue `#0000AA` to a muted navy `#000060`. The ISPF turquoise/yellow slider
  colours are part of this instance's chrome `Style` (not a seam hack).
- **Legacy Soft** -- a softer phosphor variant that keeps the ISPF semantic roles
  but tones the harshest pure-saturated values (for example, body green
  `#00FF00` becomes `#33FF66`).

Shared Solarized accents used by Default Dark and Default Light: yellow
`#B58900`, orange `#CB4B16`, red `#DC322F`, magenta `#D33682`, violet `#6C71C4`,
blue `#268BD2`, cyan `#2AA198`, green `#859900`. The chrome accent (focus ring,
active-tab tint, title band) is Solarized blue `#268BD2`.

The Dark and Light instances present, in egui `Visuals` terms:

- a three-level background hierarchy -- window/base (`panel_fill` / `window_fill`),
  a raised surface (widget `weak_bg_fill` / inactive `bg_fill`), and an inset
  level (`extreme_bg_color` for text inputs) -- all perceptibly distinct;
- the accent applied to the keyboard focus ring;
- an accent-tinted active tab distinct from the inactive tab;
- an accent-tinted primary-menu / title band distinct from the base panel fill.

All of these preserve the WCAG AA contrast advisory for the text pairs the
chrome introduces (intentionally muted inactive-tab text may use the >= 3:1
UI-element threshold).

---

## 6. Selecting a Theme

Switch the active Theme with the `THEME` command (typed on the command line, or
invoked by the Settings menu affordance -- the same code path):

- `THEME` with no argument reports the current mode.
- `THEME <mode>` switches to a built-in mode: `dark`, `light`,
  `high contrast` / `high_contrast`, `legacy`.
- `THEME legacy soft` (also `legacy-soft` / `legacy_soft`) selects the Legacy
  Soft built-in.
- `THEME <name>` selects a user theme by name.

User themes live as `.toml` files in the platform themes directory. New or
modified theme files there are picked up on hot-reload; `theme.active` applies a
new theme within one cycle.

---

## 7. The Theme Editor: Copy / Save / Save As / Import / Export

Open the Theme Editor with the `THEMES` command (or the Settings menu Themes
affordance -- same code path). It opens as a Workspace Context; END / RETURN
leaves it.

### 7.1 The editable surface is derived

The editor's list of editable controls is DERIVED from the egui `Style` /
`Visuals` chrome fields plus the retained domain groups -- it is NOT a
hand-maintained fixed token list. Each control is a labelled colour with a
`#RRGGBB` / `#RRGGBBAA` hex field. A chrome control (for example "Chrome: window
fill") edits the flat authoring field that PRODUCES the egui chrome field it is
labelled for; after each edit the shell re-derives the chrome `Style` so the live
preview reflects the egui chrome. Domain controls read and write the first-class
domain groups directly. Adding or changing a chrome authoring field or a domain
group colour surfaces in the editor automatically, without a bespoke per-field
edit.

Editing a value updates a working copy and live-previews it on the active
palette; each control round-trips (edit -> working copy -> serialise -> load)
without data loss.

### 7.2 Copy / Save / Save As

The new-name field is PRE-FILLED with a real, de-duplicated default name (not
hint text), so Copy / Save / Save As are enabled from the first frame. Saving a
built-in theme under a name creates a selectable USER theme in the themes
directory; it is one discoverable step and never a silent no-op. (This closed
bug B081, where Save was disabled behind a pre-filled-but-empty name field.)
Built-in themes are never overwritten on disk.

### 7.3 Import / Export

- **Export** writes the active (or selected) Theme to a native FFWB theme file
  (the versioned TOML format above) at a location you choose.
- **Import** reads a native FFWB theme file -- one produced by FFWB's own Export
  or Save -- and makes it a selectable user theme, without altering any built-in.
- An invalid, foreign, or non-FFWB file is REJECTED with a clear message that
  identifies the problem; the existing themes, the active theme, and the themes
  directory are left unchanged.
- Import and Export are each expressible as a command (`THEME IMPORT` /
  `THEME EXPORT`); the editor's Import/Export buttons invoke those same commands
  rather than calling the underlying logic directly (command parity).

Importing an EXTERNAL theme format (base16, VS Code, tmTheme, or any non-FFWB
format) is out of scope: the import/export loop is FFWB's own native format only.

---

## 8. Quick authoring checklist

1. Start from a built-in with `base = "Default Dark"` (or another built-in) and
   override only the tokens you want to change.
2. Set `version = 2`, a unique `name`, and the right `mode`.
3. Author colours as `#RRGGBB` (opaque) or `#RRGGBBAA` (translucent) in the flat
   authoring groups; the chrome `egui::Style` is derived from `[ui]` /
   `[tab_bar]` / `[editor]` on load.
4. Keep foreground/background pairs within the WCAG AA advisory (the editor
   surfaces contrast warnings); High Contrast must stay at AAA.
5. Prefer the Theme Editor (Copy / edit / Save As) over hand-editing; use Export
   to back up or share and Import to bring a file back.
