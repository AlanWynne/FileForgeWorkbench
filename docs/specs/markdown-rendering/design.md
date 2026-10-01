# Design Document

## Overview

The markdown-rendering sub-project introduces a new crate, **`ff-md-style`**, that owns the Markdown presentation configuration (`Markdown_Style_Config`), its built-in default, loading through the configuration-system, resolution of colour references against the active `ff-theme` palette, and the two output adapters: an **egui style mapping** for the `egui_commonmark` render path and a **CSS emitter** for the `ff-md-viewer` HTML render path. Both FileForgeWorkbench and the standalone `ffmdx` consume this crate, and the HTML/PDF export paths consume its CSS emission.

The central design constraint, confirmed by investigation, is that the two render engines have **different styling fidelity**:

- The **HTML path** (`ff-md-viewer::render_to_html`, pulldown-cmark) emits standard HTML elements, so a CSS stylesheet can style every element precisely. This path has **full fidelity**.
- The **egui path** (`egui_commonmark::CommonMarkViewer` 0.22) does **not** expose per-heading-size or per-element-colour setters. Its builder exposes only `default_width`, `indentation_spaces`, `max_image_width`, `syntax_theme_dark` / `syntax_theme_light`, `show_alt_text_on_hover`, and image/HTML/math hooks. Heading sizes and text colours come from the **ambient egui `Style` / `Visuals`** (the `TextStyle::Heading` size entries and `override_text_color`). This path is therefore an **approximation**: `ff-md-style` configures the ambient egui `Style` (heading `TextStyle` sizes, body colour) and the viewer builder knobs before the viewer renders.

This asymmetry is exactly what markdown-rendering Requirement 9.4 anticipated ("apply the nearest supported approximation rather than silently ignoring the attribute"). The design makes the single `Markdown_Style_Config` the source for both adapters so they cannot drift, while documenting the per-path capability map.

## Architecture

```text
                 markdown.toml (Markdown_Style_File, user-editable)
                          |  (ff-config: load, layered override, hot-reload)
                          v
   ff-theme palette  -->  ff-md-style
   (Colour_Tokens)        - MarkdownStyleConfig (schema structs)
                          - built-in default (compiled)
                          - loader (ff-config)
                          - ColourTokenRef resolution (-> ThemePalette)
                          - change notification
                          |
            +-------------+--------------------------+
            v                                        v
   EguiStyleMapping                           CssEmitter
   (sets egui Style + Visuals,                (emits a self-contained
    returns CommonMarkViewer knobs)            <style> stylesheet)
            |                                        |
            v                                        v
   egui_commonmark::CommonMarkViewer         ff-md-viewer::render_to_html
   (ff-mdx-app, in-shell viewer)             -> ff-html-export / ff-pdf-export
```

### New crate: `ff-md-style`

- **Dependencies**: `ff-config` (load, hot-reload), `ff-theme` (palette + Colour_Token resolution + `VisualMode`), `ff-logging` (warnings), `serde`, `toml`. For the egui mapping, an optional `egui` dependency behind a feature so the crate's pure-config core stays GUI-free and unit-testable without a UI. Proposed layout:
  - `egui` feature OFF: schema, default, loader, colour resolution, CSS emitter (all pure, headless-testable).
  - `egui` feature ON: adds the `EguiStyleMapping` adapter.
- **Rationale**: keeps the colour/typography model, loader, and CSS emitter testable with no GUI, mirroring how `ff-theme` keeps itself GUI-free. The egui adapter is additive.

### File size and structure (`rust-standards`)

`ff-md-style` is split to keep every file under 400 non-test lines:

- `lib.rs` -- re-exports, crate docs (thin coordinator).
- `config.rs` -- `MarkdownStyleConfig` and its element sub-structs (`HeadingScale`, `HeadingStyle`, `BodyStyle`, `InlineStyle`, `ListStyle`, `BlockquoteStyle`, `CodeBlockTheme`, `TableStyle`, `RuleStyle`, `LinkStyle`, `LayoutTokens`), `serde` derives, `Default`.
- `colour_ref.rs` -- `ColourTokenRef` newtype, parse/validate, and resolution against a `ThemePalette`.
- `defaults.rs` -- the compiled `Built_In_Default` config constructor.
- `loader.rs` -- TOML load via `ff-config`, partial-definition merge over defaults, per-value validation + clamping + warnings.
- `css.rs` -- the CSS emitter (`emit_css(&MarkdownStyleConfig, &ThemePalette) -> String`).
- `egui_map.rs` -- (feature `egui`) `EguiStyleMapping` building the egui `Style`/`Visuals` deltas and the `CommonMarkViewer` knob set.
- Tests split into `_tests.rs` siblings where a module's test block alone exceeds 200 lines.

## Data Model

```rust
// config.rs (illustrative shapes; exact fields finalised in implementation)

pub struct MarkdownStyleConfig {
    pub headings: HeadingScale,     // H1..H6
    pub body: BodyStyle,
    pub inline: InlineStyle,        // emphasis/strong/strikethrough/inline-code/link
    pub lists: ListStyle,
    pub blockquote: BlockquoteStyle,
    pub code_block: CodeBlockTheme,
    pub table: TableStyle,
    pub rule: RuleStyle,
    pub layout: LayoutTokens,
    pub code_highlighting_enabled: bool,
}

pub struct HeadingStyle {
    pub size_pt: f32,
    pub bold: bool,
    pub italic: bool,
    pub colour: Option<ColourTokenRef>,  // None -> body text colour (Req 2.3)
    pub underline_rule: bool,            // horizontal rule beneath (Req 2.6)
}

pub struct ColourTokenRef(String); // e.g. "editor.accent"; resolves via ThemePalette

pub struct LayoutTokens {
    pub max_content_width_px: f32,   // 0 => full width (Req 7.2)
    pub block_spacing_px: f32,
    pub list_indent_px: f32,
    pub list_item_spacing_px: f32,
    // table cell padding, code-block padding, blockquote padding ...
}
```

Colour values are **never** stored as literals in this config (Requirement 8.1); every colour is a `ColourTokenRef` resolved at render time against the active `ThemePalette` for the active `VisualMode`. Non-colour tokens (sizes, spacing, flags) are owned directly (Requirement 8.6).

### Colour token resolution (`colour_ref.rs`)

`ColourTokenRef::resolve(&self, palette: &ThemePalette) -> ColourRGBA` maps a dotted token name (e.g. `editor.accent`, `editor.background`, `syntax.keyword`) onto the corresponding palette field. An unknown name logs a warning and falls back to a documented default (`editor.foreground` for text, `editor.background` for backgrounds) per Requirement 8.4. The resolver is a thin match over the `ThemePalette` groups already defined by `theme-and-appearance` (editor, syntax, ui, etc.), so markdown-rendering adds **no** new palette tokens.

## Loading, Defaults, and Hot-Reload

### Config key and file

- A new config key `markdown.style_file` (string, default empty) names the active `Markdown_Style_File` relative to the config path; empty means "use the compiled default". Registered as a schema entry by the consumer at startup (FFWB in `register_builtin_schema`, ffmdx in its own schema registration) -- `ff-md-style` provides the key constant and default.
- Loading flow (`loader.rs`): read the file via `ff-config` -> parse TOML -> merge each present value over the `Built_In_Default` (partial definitions, Requirement 1.5) -> validate+clamp each value with a warning on failure (Requirement 1.4) -> produce a `MarkdownStyleConfig`.
- Invalid TOML (Requirement 1.3): log a warning, retain the previously active config (or default), continue.

### Hot-reload and theme-change (Requirement 10)

`ff-md-style` exposes a small holder, `MarkdownStyleHandle`, wrapping `Arc<RwLock<Arc<MarkdownStyleConfig>>>` plus a change-notification counter (an epoch), mirroring the atomic-swap pattern `ff-theme` uses for its palette:

- A `ff-config` hot-reload callback on `markdown.style_file` reloads and atomically swaps the config (Requirement 10.1, 10.2).
- The consumer re-resolves `ColourTokenRef`s on every `ff-theme` palette-change notification (Visual_Mode / theme switch) without a style-file change (Requirement 10.3), because colours are references, not stored values.
- The epoch bump lets consumers invalidate caches / request a repaint (Requirement 10.4).

Note: resolved colours are **not** cached inside `MarkdownStyleConfig` (which holds references only); the egui mapping and CSS emitter resolve against the live palette each time they run, so a mode change needs no config reload.

## Output Adapters

### CSS emitter (`css.rs`) -- full fidelity

`emit_css(config, palette) -> String` produces a self-contained stylesheet (Requirement 9.6, no external resources) targeting the standard elements pulldown-cmark emits (`h1`-`h6`, `p`, `code`, `pre`, `blockquote`, `ul`/`ol`/`li`, `table`/`th`/`td`, `hr`, `a`, `del`, `input[type=checkbox]`, `.footnote`). Colours are emitted as resolved `#RRGGBB(AA)` from the palette. `max_content_width_px` maps to `max-width` + `margin:0 auto` (centring, Requirement 7.4); `0` omits the constraint (full width, Requirement 7.2).

`ff-html-export` integration: add `build_standalone_html_with_css(title, body, css)` that injects the emitted stylesheet in place of the current hardcoded `<style>` block. The existing `build_standalone_html(title, body)` is retained (delegating to the new fn with the default config's CSS) for backward compatibility, so no existing caller breaks.

### egui style mapping (`egui_map.rs`, feature `egui`) -- approximation

`EguiStyleMapping::apply(config, palette, style: &mut egui::Style)` and a companion `configure_viewer(config, CommonMarkViewer) -> CommonMarkViewer`:

- **Heading sizes** -> set the `TextStyle::Heading`-family size entries in `style.text_styles` from `HeadingScale`. egui has one built-in `Heading` text style; the mapping registers named heading styles (H1-H6) as custom `TextStyle`s where the viewer honours them, and otherwise sets the single `Heading` to H1/H2 scale with documented degradation for H3-H6 (capability gap, Requirement 9.4).
- **Body colour** -> `visuals.override_text_color` from the body `ColourTokenRef`.
- **Links / code / blockquote colours** -> applied via `Visuals` fields where egui exposes them; where it does not, documented as approximated.
- **Max content width** -> `CommonMarkViewer::default_width`.
- **List indent** -> `CommonMarkViewer::indentation_spaces`.
- **Code-block syntax highlighting** -> `CommonMarkViewer::syntax_theme_dark` / `syntax_theme_light` selected from `code_highlighting_enabled` and the active `VisualMode`. See the decision below.
- **Max image width** -> `CommonMarkViewer::max_image_width`.

### Capability map (documented, Requirement 9.4)

| Attribute | HTML (CSS) | egui (CommonMarkViewer 0.22) |
|---|---|---|
| H1-H6 sizes | exact | approximated via egui TextStyle (H3-H6 may share Heading scale) |
| Body/link/code colours | exact | body via override_text_color; others approximated |
| Max content width | exact (`max-width`) | exact (`default_width`) |
| List indent | exact | approximated (`indentation_spaces`, integer) |
| Code-block background/padding | exact | approximated (ambient frame) |
| Code syntax highlighting | emitted token CSS classes | `syntax_theme_*` (syntect theme name) |
| Table borders / zebra | exact | approximated (ambient grid) |
| Blockquote border/background | exact | approximated |

## Code-Block Syntax Highlighting Decision (Requirement 5)

**Decision: use `egui_commonmark`'s built-in `syntax_theme_dark` / `syntax_theme_light` (syntect) for the egui path, and emit token-class CSS for the HTML path; do NOT reuse the editor's `ff-syntax-highlighting` engine for the viewer.**

Rationale:
- `egui_commonmark` already integrates syntect for fenced code blocks; wiring our own tokeniser into it is not supported by its 0.22 API and would fight the framework.
- The editor's `ff-syntax-highlighting` is built for the live editing surface (incremental, style-slot driven) and is heavier than a viewer needs; pulling it into `ff-md-style` / `ffmdx` would widen the dependency surface against the cut-down goal.
- `code_highlighting_enabled = false` (Requirement 5.5) selects a plain monospace render on both paths.
- The mapping from the theme `syntax.*` Colour_Tokens to a syntect theme name (egui) and to emitted CSS classes (HTML) is the one place the two paths approximate differently; this is captured in the capability map and is acceptable since both derive from the same `code_highlighting_enabled` flag and theme syntax group.

A future change request may revisit a shared highlighter once `ff-syntax-highlighting` is wired onto the editor render path (CR-NR-099); recorded as a follow-up, not built now.

## Consumer Integration (summary; detailed in each consumer's spec)

- **`ff-mdx-app`** (ffmdx-app spec): applies `EguiStyleMapping` before `CommonMarkViewer::show`, and `emit_css` for its HTML/PDF export.
- **In-shell viewer** (`custom-file-viewers` / `ff-mdx-plugin`): same two adapters, driven by FFWB's active palette and `markdown.style_file`.
- **`ff-html-export` / `ff-pdf-export`**: consume `build_standalone_html_with_css` so exported artefacts match on-screen (Requirement 9.2).

## Error Handling

- Library code in `ff-md-style` uses `thiserror` with a crate `MarkdownStyleError` for load/parse failures; all recoverable conditions degrade to the default with a `ff-logging` warning (no panics, no `unwrap` in lib code, per `rust-standards`).
- Invalid colour-token references, out-of-range sizes, and negative spacing each log one warning and fall back to the default for that token (Requirements 1.4, 2.5, 3.6, 4.4, 6.4, 7.5, 8.4).

## Testing Strategy (per `testing.md`)

- **Pure config/CSS/colour** (feature `egui` OFF): unit + property tests for load/merge/validate/clamp, round-trip (parse(serialise)==config, Requirement 1.7), colour-token resolution + fallback, and CSS emission (assert emitted selectors + resolved colours; self-contained, no external refs).
- **egui mapping** (feature `egui` ON): `egui_kittest` `build_ui` harness rendering a `CommonMarkViewer` with the mapping applied, asserting model-level effects (max width applied, heading text style sizes set, syntax theme selected). Pixel-exact appearance is MANUAL per `testing.md` (justified: appearance geometry), but the *presence* of the applied style values is harness-asserted.
- **Both-paths-driven** (Requirement 9.3): a test enumerating element classes and asserting each is addressed by both `emit_css` output and the egui mapping (no element styled by only one path).

## Design Decisions and Deferrals

1. New crate `ff-md-style` (name proposed; confirm at implementation) rather than extending `ff-theme` -- composes theme tokens, owns markdown typography/layout (framework-conformant).
2. egui path is an acknowledged approximation bounded by `egui_commonmark` 0.22; the capability map is the contract. If a future egui_commonmark version adds finer hooks, the mapping tightens with no config change.
3. Code-block highlighting via syntect (egui) + token CSS (HTML); shared-highlighter reuse deferred to a follow-up tied to CR-NR-099.
4. `build_standalone_html` retained for backward compatibility; new `_with_css` variant is the styled path.
5. No new palette tokens added to `ff-theme`; markdown colours are references only.
