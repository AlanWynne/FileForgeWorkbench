# Requirements Document

## Introduction

This feature specifies the rebuild of the standalone **Markdown Explorer** binary `ffmdx` (`ff-mdx-app`) as a **cut-down sibling** of FileForgeWorkbench (`ffwb`). Today `ffmdx` is a bare `eframe` application: it has no configuration environment, no theme, no logging, hardcoded window chrome, and renders Markdown through a bare `egui_commonmark::CommonMarkViewer` with no styling. The result is a plain look and no shared behaviour with the main workbench.

This feature rebuilds `ffmdx` so it reuses the same self-contained foundation crates FFWB uses, rather than reimplementing them:
- `ff-logging` -- the same structured logging subsystem (log location, levels, dev/prod build gate).
- `ff-config` -- its OWN configuration environment (a dedicated config home, layered TOML, hot-reload).
- `ff-theme` -- inherit FFWB's palette, fonts, and Visual_Modes, applied to the `ffmdx` egui context.
- `ff-session` -- remember the last folder, window size and position, and last-opened file.
- `ff-help` -- a cut-down About / Help surface.
- `markdown-rendering` (CR-NR-101) -- the Markdown element styling, consumed by `ffmdx`'s egui and export paths.

The identical startup wiring (initialise logging, load configuration, load theme, apply fonts and palette to the egui context) is factored into a NEW shared crate (proposed `ff-app-bootstrap`, name confirmed in `design.md`) called by BOTH `ffwb` and `ffmdx`. This shared bootstrap is the structural mechanism that makes `ffmdx` a cut-down FFWB rather than a parallel reimplementation, and prevents the two startup paths from drifting.

The dependency survey confirms these foundation crates are GUI/editor-free and reusable: `ff-config` depends only on `ff-core` + `ff-logging`; `ff-logging` has no workbench dependencies; `ff-theme` depends only on `ff-config` + `ff-logging`. None drags in the editor, VFS, command dispatch, or shell.

**Out of scope for this round (deferred, not dropped):**
- `ff-keys` configurable keybindings. `ffmdx`'s shortcut surface is tiny (Ctrl+O, F5); configurable keys are deferred. See Requirement 7.
- The full editor / command-dispatch / VFS / catalog / toolchain / plugin / layout-docking stack. `ffmdx` is a viewer, not an editor; pulling these would defeat the cut-down goal.

**Source references:**
- **[OWNER-FFMDX]** = Owner prompt (Phase ffmdx-parity): "the ffmdx application should inherit theme attributes from FFWB ... it must create its own configuration environment ... perhaps it should also inherit FFWB's logging options ... FFMDX should be built very similarly to FFWB, essentially a cut down version."
- **[THEME]** = `theme-and-appearance` requirements -- theme loading, palette, fonts, Visual_Modes.
- **[CFG]** = `configuration-system` requirements -- TOML loading, config home, layered overrides, hot-reload.
- **[LOG]** = `logging-subsystem` requirements -- log location, levels, build-profile gate.
- **[SESS]** = `startup-and-session` requirements -- session persistence (window geometry, recent folder/file).
- **[HELP]** = `context-help` requirements -- help content and display.
- **[MDR]** = `markdown-rendering` requirements (CR-NR-101) -- Markdown element styling consumed here.

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `markdown-rendering` | **Dependency** | `ffmdx` applies the Markdown_Style_Config to its egui `CommonMarkViewer` and its HTML/PDF export. |
| `theme-and-appearance` | **Dependency** | `ffmdx` loads the active theme through `ff-theme` and applies palette and fonts to its egui context. |
| `configuration-system` | **Dependency** | `ffmdx` has its own configuration home loaded through `ff-config`. |
| `logging-subsystem` | **Dependency** | `ffmdx` initialises `ff-logging` at startup, sharing log location and level conventions. |
| `startup-and-session` | **Dependency** | `ffmdx` persists and restores its session (window geometry, last folder, last file) through `ff-session`. |
| `context-help` | **Dependency** | `ffmdx` exposes a cut-down Help / About surface via `ff-help`. |
| `fileforge-integration` | **Related** | `ffmdx` and `ffwb` share a common startup via the shared bootstrap crate; `ffwb` is refactored to call it (behaviour-preserving). |

## Glossary

- **Ffmdx**: The standalone Markdown Explorer application and its binary `ffmdx` (`ff-mdx-app`). [OWNER-FFMDX]
- **App_Bootstrap**: The NEW shared crate (proposed `ff-app-bootstrap`) that performs the common startup sequence -- initialise logging, load configuration, load theme, apply fonts and palette to the egui context -- called by both `ffwb` and `ffmdx`. [OWNER-FFMDX]
- **Config_Home**: The configuration directory owned by `ffmdx` (distinct from FFWB's), where its TOML configuration, theme selection, and session state live. [CFG]
- **Foundation_Crate**: One of the GUI/editor-free crates reused by `ffmdx`: `ff-logging`, `ff-config`, `ff-theme`, `ff-session`, `ff-help`. [OWNER-FFMDX]
- **Session_State**: The persisted `ffmdx` session: window size and position, the last-opened folder, and the last-opened file. [SESS]
- **Cut_Down_Surface**: The deliberately minimal `ffmdx` feature set (open folder, browse tree, filter, view Markdown, reload, export) that excludes the editor, command dispatch, VFS, toolchains, and plugins. [OWNER-FFMDX]

## Requirements

### Requirement 1: Shared Application Bootstrap

**User Story:** As a workbench developer, I want `ffmdx` and `ffwb` to share one startup path, so that the standalone app is a cut-down FFWB rather than a parallel reimplementation, and the two cannot drift.

**Source:** [OWNER-FFMDX] "FFMDX should be built very similarly to FFWB".

#### Acceptance Criteria

1. THE project SHALL provide a shared App_Bootstrap crate that performs, in order: initialise `ff-logging`, load configuration through `ff-config`, load the active theme through `ff-theme`, and apply the resolved fonts and palette to the egui context before the first frame.
2. THE `ffmdx` binary SHALL perform its startup by invoking the App_Bootstrap crate, not by reimplementing the sequence inline.
3. THE `ffwb` binary SHALL be refactored to perform its common startup through the SAME App_Bootstrap crate, with no observable change to `ffwb` behaviour (behaviour-preserving refactor).
4. THE App_Bootstrap SHALL accept per-application parameters (at minimum: application name, Config_Home selection, window title and default size) so that each binary supplies its own identity while sharing the sequence.
5. THE App_Bootstrap SHALL NOT depend on the editor, command-dispatch, VFS, toolchain, or shell crates, so that `ffmdx` remains a Cut_Down_Surface.

---

### Requirement 2: Configuration Environment

**User Story:** As a user of the standalone Markdown Explorer, I want `ffmdx` to have its own configuration directory and TOML settings, so that I can configure it independently of the main workbench.

**Source:** [OWNER-FFMDX] "it must create its own configuration environment"; [CFG].

#### Acceptance Criteria

1. THE `ffmdx` application SHALL establish its own Config_Home directory (distinct from the FFWB config home) using `ff-config`, creating it on first launch if absent.
2. THE `ffmdx` application SHALL load its settings from TOML files in the Config_Home through the `ff-config` API, participating in the layered-override and hot-reload model.
3. WHEN the Config_Home or a configuration file is absent, THE application SHALL use documented defaults and continue operating, logging the fallback at DEBUG level.
4. WHEN a configuration file contains invalid TOML, THE application SHALL log a warning identifying the file and parse error, retain previous or default values, and continue operating.
5. THE `ffmdx` configuration SHALL include at minimum: the active theme selection, the active Visual_Mode, the active Markdown_Style_File selection, and logging options (level), each falling back to a documented default when unset.
6. THE `ffmdx` application SHALL NOT write to the FFWB config home, and FFWB SHALL NOT write to the `ffmdx` Config_Home.

---

### Requirement 3: Theme Inheritance

**User Story:** As a user of the standalone Markdown Explorer, I want `ffmdx` to use the same themes as FFWB, so that the two applications look consistent and I can switch Visual_Modes.

**Source:** [OWNER-FFMDX] "inherit theme attributes from FFWB"; [THEME].

#### Acceptance Criteria

1. THE `ffmdx` application SHALL load the active theme through `ff-theme`, obtaining the same palette, Font_Stacks, and Visual_Modes available to FFWB.
2. THE `ffmdx` application SHALL apply the resolved monospace and proportional fonts and the palette to its egui context before the first frame is rendered.
3. THE `ffmdx` application SHALL render all of its own chrome (toolbar, file-tree panel, status bar, separators, placeholder text) using theme Colour_Tokens, with no hardcoded colour values in its rendering code.
4. THE `ffmdx` application SHALL support switching the active Visual_Mode (dark / light / high-contrast / legacy) at runtime, applying the change within the theme system's frame budget.
5. WHEN a theme hot-reload or Visual_Mode change occurs, THE `ffmdx` application SHALL re-apply the updated palette and fonts without an application restart.
6. THE `ffmdx` application SHALL be able to discover and select any theme file available in its Config_Home themes directory, in addition to the built-in themes, consistent with the theme system's discovery model.

---

### Requirement 4: Logging

**User Story:** As a developer supporting `ffmdx`, I want the standalone app to log through the same subsystem as FFWB, so that diagnostics are consistent and the production build strips development logging.

**Source:** [OWNER-FFMDX] "inherit FFWB's logging options"; [LOG].

#### Acceptance Criteria

1. THE `ffmdx` application SHALL initialise `ff-logging` at startup (via the App_Bootstrap), using the same log-record format and level conventions as FFWB.
2. THE `ffmdx` application SHALL honour the `ff-logging` build-profile feature gate (`dev-logging`), so that a production release build strips TRACE/DEBUG call sites exactly as FFWB does.
3. THE `ffmdx` application SHALL replace its ad-hoc status-string-only feedback with logged records at appropriate levels (INFO for folder open / file view / export, WARN for recoverable errors, ERROR for failures) in addition to any user-facing status line.
4. THE active logging level SHALL be configurable through the `ffmdx` configuration (Requirement 2.5).
5. WHEN logging initialisation fails, THE application SHALL continue to run with logging disabled and SHALL surface a single non-fatal notice, rather than aborting startup.

---

### Requirement 5: Session Persistence

**User Story:** As a user of the standalone Markdown Explorer, I want `ffmdx` to remember my last folder, window placement, and last-opened file, so that it reopens where I left off.

**Source:** [OWNER-FFMDX] recommendation (session); [SESS].

#### Acceptance Criteria

1. THE `ffmdx` application SHALL persist its Session_State through `ff-session`, including window size, window position, the last-opened folder, and the last-opened file.
2. WHEN `ffmdx` starts with no folder argument on the command line, THE application SHALL restore the last-opened folder from Session_State if it still exists.
3. WHEN `ffmdx` starts with no file argument AND the restored folder contains the last-opened file, THE application SHALL re-open that file.
4. WHEN a persisted folder or file no longer exists, THE application SHALL start in the empty state and log the missing path at DEBUG level, without error.
5. WHEN `ffmdx` is given a folder path on the command line, THE command-line argument SHALL take precedence over the persisted folder.
6. THE application SHALL update Session_State when the folder, open file, or window geometry changes, and SHALL persist it on a clean exit.

---

### Requirement 6: Markdown Rendering via the Shared Configuration

**User Story:** As a user of the standalone Markdown Explorer, I want `ffmdx` to render Markdown using the configurable element styling, so that documents look good and match FFWB.

**Source:** [OWNER-FFMDX]; [MDR].

#### Acceptance Criteria

1. THE `ffmdx` application SHALL render Markdown through the markdown-rendering subsystem (CR-NR-101), applying the active Markdown_Style_Config to its egui `CommonMarkViewer`.
2. THE `ffmdx` application SHALL apply the Markdown_Style_Config's CSS_Emission to its HTML export (and PDF export, where applicable), so that exported output matches the on-screen appearance.
3. WHEN the Markdown_Style_Config changes (edit, hot-reload, or Visual_Mode change), THE `ffmdx` application SHALL re-render the open document using the updated configuration without a restart.
4. THE `ffmdx` application SHALL NOT define its own Markdown element styling independent of the markdown-rendering subsystem.

---

### Requirement 7: Help and About, and Deferred Keybindings

**User Story:** As a user of the standalone Markdown Explorer, I want a basic Help / About surface, while accepting that configurable keybindings are a later addition.

**Source:** [OWNER-FFMDX] recommendation (help included, keys deferred); [HELP].

#### Acceptance Criteria

1. THE `ffmdx` application SHALL expose a cut-down Help / About surface (at minimum: application name and version, a short usage summary, and the active keyboard shortcuts) using `ff-help` where it fits the cut-down scope.
2. THE Help / About surface SHALL be reachable from the `ffmdx` toolbar or menu.
3. THE `ffmdx` keyboard shortcuts (Ctrl+O to open a folder, F5 to refresh) SHALL remain hardcoded for this round; configurable keybindings via `ff-keys` are explicitly DEFERRED and SHALL be recorded as out of scope in this requirements document and in the design.
4. THE DEFERRAL of `ff-keys` SHALL be revisited only as a separate gated change request; this requirement's scope SHALL NOT include integrating configurable keybindings.

---

### Requirement 8: Cut-Down Surface Boundary

**User Story:** As a workbench maintainer, I want a clear boundary on what `ffmdx` includes, so that it stays a lightweight viewer and does not accrete the full workbench.

**Source:** [OWNER-FFMDX] "essentially a cut down version".

#### Acceptance Criteria

1. THE `ffmdx` application SHALL depend only on the Foundation_Crates, the markdown-rendering crate, the export crates (`ff-html-export`, `ff-pdf-export`), the Markdown scanning/watching crate (`ff-md-viewer`), and the egui/eframe UI crates; it SHALL NOT depend on the editor, command-dispatch, VFS, catalog, toolchain, plugin, or layout-docking crates.
2. THE `ffmdx` feature set SHALL remain the Cut_Down_Surface: open a folder, browse the Markdown file tree, filter, view a rendered Markdown document, live-reload on external change, and export to HTML (and PDF where supported).
3. WHEN a future requirement proposes adding an excluded capability to `ffmdx`, THAT addition SHALL be a separate gated change request that explicitly re-evaluates the Cut_Down_Surface boundary.
4. THE rebuild SHALL preserve the existing `ffmdx` behaviours that already pass acceptance tests (folder scan, F5 rescan, drag-and-drop folder open, filter, HTML export), changing only their styling and the addition of the Foundation_Crate integrations.
