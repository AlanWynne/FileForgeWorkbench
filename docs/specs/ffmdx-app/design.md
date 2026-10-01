# Design Document

## Overview

This design rebuilds the standalone Markdown Explorer `ffmdx` (`ff-mdx-app`) as a cut-down sibling of FileForgeWorkbench, and introduces a shared startup crate, **`ff-app-bootstrap`**, that both `ffwb` and `ffmdx` call so their common startup cannot drift. `ffmdx` gains its own configuration environment (`ff-config`), inherits FFWB's theme (`ff-theme`), logs through `ff-logging`, persists session state (`ff-session`), exposes a cut-down Help/About surface (`ff-help`), and renders Markdown through the markdown-rendering subsystem (`ff-md-style`, CR-NR-101).

The investigation confirmed the three foundation crates are GUI/editor-free and safe to reuse: `ff-config` (deps `ff-core` + `ff-logging`), `ff-logging` (no workbench deps), `ff-theme` (deps `ff-config` + `ff-logging`). The existing `ffwb` startup in `crates/ff-desktop/src/main.rs` is an ordered sequence (profile arg -> two-phase logging -> `ff_config::init` -> schema registration -> `apply_logging_config` -> theme palette resolution -> `eframe::run_native`). The bootstrap crate factors the common, non-shell portion of that sequence so `ffwb` keeps identical behaviour and `ffmdx` reuses it.

## Architecture

```text
                 ff-app-bootstrap  (NEW shared crate)
                 - AppBootstrap::run(BootstrapOptions) -> BootstrapResult
                 - init logging -> load config -> load theme palette
                 - apply fonts + palette to egui Context (pre-first-frame hook)
                        |                         |
          +-------------+                         +-------------+
          v                                                     v
   ffwb (ff-desktop/main.rs)                           ffmdx (ff-mdx-app)
   - supplies ffwb identity                            - supplies ffmdx identity
   - WorkbenchShell (full)                             - MdxApp (cut-down)
                                                        - ff-md-style (egui + CSS)
                                                        - ff-session (window/folder/file)
                                                        - ff-help (About/Help)
```

### New crate: `ff-app-bootstrap`

- **Responsibility** (Requirement 1.1): perform, in order, (1) initialise `ff-logging`, (2) load configuration via `ff-config`, (3) resolve the active theme palette via `ff-theme`, and (4) provide a hook that applies the resolved fonts and palette to an `egui::Context` before the first frame.
- **Dependencies**: `ff-logging`, `ff-config`, `ff-theme`, and `egui`/`eframe` only for the context-application hook. It MUST NOT depend on the editor, command-dispatch, VFS, toolchain, plugin, or shell crates (Requirement 1.5, 5), so it stays reusable by a cut-down app.
- **API shape** (illustrative):

```rust
pub struct BootstrapOptions {
    pub app_name: String,            // "ffwb" | "ffmdx" -- identity (Req 1.4)
    pub config_home: ConfigHome,     // which config environment (Req 1.4, 2.1)
    pub window_title: String,
    pub default_window_size: [f32; 2],
    pub profile: Option<String>,     // --profile passthrough
}

pub struct BootstrapResult {
    pub config: ConfigHandle,
    pub logging_status: LoggingStatus,
    pub palette: Arc<ThemePalette>,
}

impl AppBootstrap {
    pub fn run(opts: BootstrapOptions) -> anyhow::Result<BootstrapResult>;
    /// Apply resolved fonts + palette to the egui context (call from the
    /// eframe creation closure, before the first frame -- Req 3.2).
    pub fn apply_to_egui(result: &BootstrapResult, ctx: &egui::Context);
}
```

- **`ffwb` refactor** (Requirement 1.3, behaviour-preserving): `ff-desktop/main.rs` replaces its inline logging-init + config-init + palette-resolution block with `AppBootstrap::run`, passing the `ffwb` identity and config home. The ffwb-specific steps that are NOT common (tokio runtime, `WorkbenchApp`, `register_builtin_schema` for the full key set, CLI/batch/fftest handling, `WorkbenchShell`) stay in `ff-desktop`. The schema-registration seam is parameterised: the bootstrap calls back into a consumer-supplied schema-registration closure so each app registers its own keys. Behaviour must be byte-identical for `ffwb`; this is verified by the existing `ff-desktop` startup tests continuing to pass plus a new assertion that the bootstrap path produces the same effective logging/theme config.

### `ff-mdx-app` rebuild

- `main.rs` builds `BootstrapOptions` for the `ffmdx` identity (own config home, title "Markdown Explorer"), calls `AppBootstrap::run`, then `eframe::run_native` with a creation closure that calls `AppBootstrap::apply_to_egui` and constructs `MdxApp`.
- `MdxApp` (today in `app.rs`) gains fields for the `ConfigHandle`, the active `Arc<ThemePalette>`, the `MarkdownStyleHandle` (from `ff-md-style`), and a `SessionState` (from `ff-session`). Its `update` loop re-applies the palette on theme change and re-renders the viewer on markdown-style change.
- The viewer (`viewer.rs`) replaces the bare `CommonMarkViewer::new()` with the `ff-md-style` egui mapping applied first (`EguiStyleMapping::apply` + `configure_viewer`).
- Chrome (`app.rs` toolbar, file tree, status bar) replaces hardcoded `RichText` styling with theme-token-derived colours applied through the egui `Visuals` set by the bootstrap (Requirement 3.3) -- the panels then inherit theme colours automatically, with the few explicit colours sourced from the palette.

### File size and structure (`rust-standards`)

- `ff-app-bootstrap`: `lib.rs` (API + docs), `run.rs` (the sequence), `egui_apply.rs` (fonts + palette -> context), `options.rs` (option/result types). Each < 400 non-test lines.
- `ff-mdx-app`: existing split (`app.rs`, `viewer.rs`, `file_tree.rs`, `helpers.rs`, `main.rs`) retained; add `config.rs` (ffmdx config home + schema keys), `session.rs` (session wiring), `help.rs` (About/Help surface). Keep each file under 400 non-test lines; split `app.rs` if the additions push it over.

## Configuration Environment (Requirement 2)

- `ffmdx` resolves its own config home via `ff-config`, distinct from `ffwb`. Mechanism: `ff-config`'s `ConfigInitOptions` with an ffmdx-specific application directory (and `ff-session::set_active_profile` left at default). The two apps therefore never write each other's config home (Requirement 2.6).
- ffmdx registers a minimal schema (its own `register_schema` closure passed through the bootstrap): `theme.active`, `theme.active_name`, `markdown.style_file`, `logging.level` (+ the logging dir/rotation keys ffmdx honours). Each has a documented default (Requirement 2.5); absent files/keys fall back with a DEBUG log (Requirement 2.3); invalid TOML logs a warning and retains defaults (Requirement 2.4).

## Theme Inheritance (Requirement 3)

- The bootstrap resolves the active palette with the same `ff-theme` loader `ffwb` uses, and `apply_to_egui` builds `egui::Visuals` from the palette (the proven `apply_theme` pattern in `ff-desktop/src/shell/render_chrome.rs`: `panel_fill`, `override_text_color`, widget fills/strokes, selection colours from palette tokens via a `to_egui_color` helper) and sets the resolved monospace/proportional fonts (`FontConfig` base sizes) before the first frame (Requirement 3.2).
- `ffmdx` chrome uses the ambient `Visuals` set this way; no hardcoded colours remain in its rendering code (Requirement 3.3).
- Visual_Mode switching at runtime (Requirement 3.4, 3.5): ffmdx exposes a mode selector (toolbar menu); changing it writes `theme.active` through `ff-config` and re-resolves the palette, re-applying `apply_to_egui` on the next frame. Theme-file discovery (Requirement 3.6) reuses the `ff-theme` discovery model against ffmdx's themes directory.

## Logging (Requirement 4)

- The bootstrap calls `ff_logging::init_default` then applies the configured level/dir (mirroring `ffwb`'s two-phase init), so ffmdx shares the record format and the `dev-logging` build-profile gate (Requirement 4.2) automatically.
- `ffmdx` replaces its status-string-only feedback with `log_info!`/`log_warn!`/`log_error!` records at the folder-open / file-view / export / error seams (Requirement 4.3), keeping the user-facing status line as well.
- Logging level is a config key (Requirement 4.4). Init failure degrades to disabled logging with a non-fatal notice, never aborting startup (Requirement 4.5).

## Session Persistence (Requirement 5)

- `ffmdx` persists `SessionState` via `ff-session`: `WindowGeometryState` (size/position), the last-opened folder, and the last-opened file. On start with no CLI folder, it restores the last folder if it still exists (`window_geometry::restore_geometry` + `clamp_to_display` for the window; folder/file existence checks for the rest), re-opening the last file when present (Requirements 5.1-5.4).
- CLI folder argument takes precedence over the persisted folder (Requirement 5.5), matching the existing `main.rs` arg handling.
- Session is updated on folder/file/geometry change and saved on clean exit (Requirement 5.6), using `ff-session`'s session-file persistence.
- Note: ffmdx uses a focused subset of `ff-session` (geometry + recent folder/file), not the full tab/layout session model, consistent with the cut-down surface.

## Markdown Rendering (Requirement 6)

- ffmdx holds a `MarkdownStyleHandle` from `ff-md-style`, loaded from its `markdown.style_file` config key. The viewer applies `EguiStyleMapping` before `CommonMarkViewer::show`; HTML export calls `ff-html-export::build_standalone_html_with_css` with `ff-md-style::emit_css` (Requirements 6.1, 6.2).
- On markdown-style hot-reload or Visual_Mode change, ffmdx re-renders the open document from the updated config/palette without restart (Requirement 6.3). ffmdx defines no independent markdown styling (Requirement 6.4).

## Help / About and Deferred Keys (Requirement 7)

- A cut-down About/Help surface (app name + version, short usage, active shortcuts) is reached from the toolbar (Requirements 7.1, 7.2). Where it fits the cut-down scope, it reuses `ff-help` content primitives (`HelpTopicRegistry` / `ContentLoader` / `HelpPanelModel`); a minimal About dialog may be a plain egui window if a full help panel is heavier than warranted -- decided at implementation, bounded by "cut-down".
- Keybindings stay hardcoded (Ctrl+O, F5); `ff-keys` integration is explicitly deferred and recorded here and in the requirements as out of scope (Requirements 7.3, 7.4). No `ff-keys` dependency is added.

## Cut-Down Surface Boundary (Requirement 8)

- `ff-mdx-app` dependency allow-list: `ff-app-bootstrap`, `ff-md-style`, `ff-html-export`, `ff-pdf-export`, `ff-md-viewer`, `ff-config`, `ff-theme`, `ff-logging`, `ff-session`, `ff-help`, `egui`, `eframe`, `rfd`, `anyhow`. It MUST NOT depend on the editor, command-dispatch, VFS, catalog, toolchain, plugin, or layout-docking crates (Requirement 8.1).
- The feature set stays the Cut_Down_Surface (open folder, browse tree, filter, view, live-reload, export) (Requirement 8.2). Existing passing behaviours (folder scan, F5 rescan, drag-drop open, filter, HTML export) are preserved, changing only styling + foundation integration (Requirement 8.4).
- Adding any excluded capability later is a separate gated change request that re-evaluates the boundary (Requirement 8.3).

## Error Handling

- `ffmdx` is application/binary code: `anyhow` with `.context(...)` across module boundaries. Startup degrades gracefully -- a missing config home, theme file, or session file never aborts launch (bootstrap + ffmdx each fall back with a logged warning), consistent with `ff-session`'s graceful-degradation guarantee.

## Testing Strategy (per `testing.md`)

- **`ff-app-bootstrap`**: headless unit tests for `run` (logging status, config handle, palette resolution with a tempdir config home) and an `egui_kittest` `build_eframe`/`build_ui` test that `apply_to_egui` sets expected `Visuals`/fonts. A behaviour-preservation test asserts `ffwb`'s existing startup tests still pass after the refactor.
- **`ff-mdx-app`**: extend the existing `egui_kittest` `build_eframe` tests (F5 rescan, empty-state, loaded-title already exist) with: theme applied to chrome (assert a palette-derived colour on a known widget), markdown style applied to the viewer (assert the mapping ran), session restore (tempdir: persisted folder re-opened; CLI arg precedence), and About/Help reachable. Pixel-exact appearance stays MANUAL (justified) but applied-value presence is harness-asserted.
- All new tests carry `// Validates: Requirement X.Y`.

## Design Decisions and Deferrals

1. `ff-app-bootstrap` is the enforcement mechanism for "ffmdx is a cut-down ffwb"; the `ffwb` refactor is behaviour-preserving and gated by its existing tests.
2. Schema registration is a consumer-supplied closure so each app owns its keys while sharing the sequence.
3. ffmdx uses a focused `ff-session` subset (geometry + recent folder/file), not the full tab/layout model.
4. About/Help may be a minimal surface rather than the full `ff-help` panel if the full panel exceeds the cut-down scope; bounded decision at implementation.
5. `ff-keys` deferred; no dependency added. Revisited only as a separate CR.
6. No framework mechanism (command dispatch, navigation stack, WorkspaceContext) is touched; ffmdx consumes foundation crates, not the shell.
