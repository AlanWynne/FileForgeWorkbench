# Requirements Document

## Introduction

This feature specifies the **Custom File Viewer** framework for FileForgeWorkbench (`ff-viewers` crate). The viewer framework provides an extensible registry of file viewers, a `PREVIEW` command for activation, a `FileViewer` trait that all viewers implement, and a set of built-in viewers for common file types. Viewers display file content in specialised visual representations (ASA report rendering, hex display, image placeholders, CSV table grids) without modifying the underlying document.

The framework is architecturally significant because it connects multiple platform subsystems:
- **Plugin Architecture** (`ff-plugin`): Plugins register viewers at runtime via the `FileViewer` trait.
- **Layout & Docking** (`ff-layout`): Viewer output is rendered in a `DockablePanel` that participates in the dock zone / tab group system.
- **Command Framework** (`ff-command`): The `PREVIEW` command (and its sub-commands) is registered in the command registry.
- **Virtual File System** (`ff-vfs`): Viewers read content through the VFS abstraction, never directly from the filesystem.

The key principle is **view-only rendering**: a viewer is always a read-only representation of a resource. Editing is performed in the standard editor; the viewer refreshes to reflect changes.

**Source references:**
- **FFE** = FileForgeEditor `custom-file-viewers` specification (6 requirements — all incorporated and adapted)
- **WB** = Workbench Platform Architecture Brief (plugin model, layout-as-data, command-driven architecture)

**Cross-references:**
- `plugin-architecture` — `FileForgePlugin` trait, `PluginContext`, capability registration
- `layout-and-docking` — `DockablePanel` trait, Panel_Registry, dock zones, tab groups
- `asa-report-preview` — Built-in ASA report viewer (separate spec, registered here)
- `hex-display` — Built-in hex viewer (separate spec, registered here)
- `command-framework` — Command_Registry, Command_ID `"viewer.preview"`, `"viewer.preview_list"`
- `virtual-file-system` — Resource_URI, VfsProvider, async content reads

---

## Glossary

- **FileViewer**: The core trait that all viewer implementations must implement. Defines methods for rendering, supported content types, panel integration, and refresh behaviour. Viewers are always read-only. [FFE, WB]
- **Viewer_Registry**: The platform-owned registry of all available FileViewer implementations, populated at startup from built-in viewers and at runtime from plugin contributions. Indexed by Viewer_Key. [FFE, WB]
- **Viewer_Key**: A unique, lowercase, ASCII identifier string for a viewer (e.g., `"asa-report"`, `"hex"`, `"image"`, `"csv-table"`). Used in commands, configuration, and language profile declarations. [FFE]
- **Viewer_Panel**: A `DockablePanel` implementation that hosts the output of the active FileViewer. Renders in the center dock zone (typically beside or replacing the editor tab). [FFE, WB]
- **Built_In_Viewer**: A FileViewer implementation compiled directly into the `ff-viewers` crate. Always available without additional plugins. [FFE]
- **Plugin_Viewer**: A FileViewer implementation contributed by a plugin at runtime. Registered through the `PluginContext` capability advertisement mechanism. [WB]
- **PREVIEW**: The primary command (`viewer.preview`) that activates, switches, and deactivates viewers. Registered in the Command_Registry. [FFE, WB]
- **Viewer_Mode**: The state in which a Viewer_Panel is active and rendering content for a resource. Viewer_Mode is always read-only — the viewer displays but does not modify. [FFE]
- **Content_Match**: The mechanism by which a viewer declares what content it can render — via file extensions, MIME types, content-sniffing predicates, or explicit user selection. [FFE, WB]

---

## Requirements

### Requirement 1: Viewer Registry

**User Story:** As a platform developer, I want a central viewer registry that maps Viewer_Keys to FileViewer implementations, so that viewers from built-in code and plugins are discoverable and activatable through a single unified mechanism.

**Source:** FFE custom-file-viewers Req 1, adapted for plugin architecture and VFS integration. [FFE, WB]

#### Acceptance Criteria

1. THE `ff-viewers` crate SHALL maintain a Viewer_Registry that maps Viewer_Key strings to `Box<dyn FileViewer>` implementations, where Viewer_Key is a non-empty string containing only lowercase ASCII letters, digits, and hyphens.
2. THE Viewer_Registry SHALL be thread-safe (safe to read and write from any thread without requiring the caller to acquire an external lock).
3. WHEN the application starts, THE Viewer_Registry SHALL be populated with all Built_In_Viewers before any plugin initialization occurs.
4. THE Viewer_Registry SHALL support runtime registration of Plugin_Viewers via the `PluginContext` interface defined in `plugin-architecture`, allowing plugins to register viewers during their `initialize` lifecycle phase.
5. THE Viewer_Registry SHALL support deregistration of Plugin_Viewers during a plugin's `shutdown` lifecycle phase, cleanly removing the viewer and closing any active Viewer_Panels that were displaying through that viewer.
6. WHEN a registration is attempted with a Viewer_Key that already exists in the Viewer_Registry, THE registry SHALL reject the registration and return an error indicating the duplicate key — without modifying the existing registration.
7. THE Viewer_Registry SHALL support runtime discovery: listing all registered viewers with their Viewer_Key, display name, description, and supported content types.
8. WHEN a Viewer_Key is referenced in a command or language profile that does not exist in the Viewer_Registry, THE system SHALL log a warning identifying the unknown Viewer_Key and fall back to raw text display in the editor.

---

### Requirement 2: FileViewer Trait

**User Story:** As a viewer developer (core or plugin), I want a well-defined trait with clear rendering and metadata methods, so that I can implement a viewer without understanding the platform's internal rendering pipeline.

**Source:** FFE custom-file-viewers conceptual viewer API, elevated to a formal trait for workbench extensibility. [FFE, WB]

#### Acceptance Criteria

1. THE `ff-viewers` crate SHALL define a `FileViewer` trait with the following methods:
   - `viewer_key(&self) -> &str` — returns the unique Viewer_Key identifier.
   - `display_name(&self) -> &str` — returns a human-readable display name (1 to 128 characters).
   - `description(&self) -> &str` — returns a brief description of what the viewer renders.
   - `supported_extensions(&self) -> &[&str]` — returns file extensions this viewer handles (e.g., `["lst", "rpt", "spool"]`).
   - `supported_mime_types(&self) -> &[&str]` — returns MIME types this viewer handles (e.g., `["text/csv"]`).
   - `can_render(&self, uri: &ResourceUri, content_sample: &[u8]) -> bool` — returns whether this viewer can render the given resource, using URI metadata and/or a content sample for sniffing.
   - `render(&self, content: &[u8], ui: &mut egui::Ui)` — renders the content into the provided egui UI region.
   - `on_content_changed(&mut self, new_content: &[u8])` — called when the underlying document changes, allowing the viewer to refresh its internal state.
2. THE `FileViewer` trait SHALL be object-safe, allowing the platform to store viewers as trait objects (`Box<dyn FileViewer>`).
3. ALL methods on `FileViewer` SHALL be non-mutating except `on_content_changed`, which receives `&mut self` to update internal render state.
4. THE `render` method SHALL produce read-only output — it SHALL NOT provide any mechanism for the user to modify the underlying document content through the viewer.
5. THE `FileViewer` trait SHALL NOT require implementors to manage their own panel lifecycle — the platform's Viewer_Panel wrapper handles docking, visibility, and focus.

---

### Requirement 3: PREVIEW Command

**User Story:** As a workbench user, I want a unified `PREVIEW` command to activate, switch, list, and deactivate viewers, so that I have a consistent command-driven interface regardless of which viewer is being used.

**Source:** FFE custom-file-viewers Req 3, adapted for workbench command framework. [FFE, WB]

#### Acceptance Criteria

1. THE `ff-viewers` crate SHALL register a command with Command_ID `"viewer.preview"` in the Command_Registry during platform startup, accepting an optional `action` parameter with values: a Viewer_Key string, `"on"`, `"off"`, `"list"`, or no argument (toggle).
2. WHEN `PREVIEW` is issued with no argument, THE command SHALL toggle the viewer: activating the default viewer for the current resource if no viewer is active, or deactivating the active viewer if one is showing.
3. WHEN `PREVIEW ON` is issued with no viewer argument, THE command SHALL activate the default viewer for the current resource's content type (determined by extension match or language profile `default_viewer`). IF no default viewer is defined, THE command SHALL display a message listing available viewers.
4. WHEN `PREVIEW <viewer-key>` is issued (e.g., `PREVIEW asa-report`, `PREVIEW csv-table`), THE command SHALL activate the named viewer regardless of the language profile default.
5. WHEN `PREVIEW OFF` is issued, THE command SHALL deactivate the active viewer and close or hide the Viewer_Panel, returning focus to the editor.
6. WHEN `PREVIEW LIST` is issued, THE command SHALL display the Viewer_Key, display name, and description of all registered viewers in the message/output area.
7. THE active Viewer_Key SHALL be displayed in the status bar when Viewer_Mode is active (e.g., `Viewer: asa-report`).
8. THE `PREVIEW` command SHALL be valid regardless of whether the active resource is in browse mode or edit mode.
9. THE `PREVIEW` state change SHALL NOT produce an Undo_Record — it is a non-undoable display state change that does not modify document content.

---

### Requirement 4: Built-In Viewers

**User Story:** As a workbench user, I want a set of built-in viewers for common file types available immediately without installing plugins, so that mainframe reports, binary files, images, and tabular data are viewable out of the box.

**Source:** FFE custom-file-viewers Req 1.2 (asa-report, hex), extended with image placeholder and CSV table for workbench scope. [FFE, WB]

#### Acceptance Criteria

1. THE `ff-viewers` crate SHALL include a Built_In_Viewer with Viewer_Key `"asa-report"` that renders ASA carriage control report files. The rendering logic is defined in the separate `asa-report-preview` spec; this registration makes it discoverable via the Viewer_Registry and activatable via `PREVIEW asa-report`.
2. THE `ff-viewers` crate SHALL include a Built_In_Viewer with Viewer_Key `"hex"` that renders binary file content in a hex dump format (offset + hex bytes + ASCII decode). The rendering logic is defined in the separate `hex-display` spec; this registration provides discoverability via `PREVIEW LIST`. The canonical activation commands for hex mode remain `HEX ON` / `HEX OFF` as defined in `hex-display`; `PREVIEW hex` is accepted as an alias.
3. THE `ff-viewers` crate SHALL include a Built_In_Viewer with Viewer_Key `"image"` that renders image files (PNG, JPEG, GIF, BMP, WEBP) as a scaled preview within the Viewer_Panel. IF the image cannot be decoded, THE viewer SHALL display a placeholder with the filename, dimensions (if available from headers), and an error description.
4. THE `ff-viewers` crate SHALL include a Built_In_Viewer with Viewer_Key `"csv-table"` that renders CSV/TSV files as a formatted table grid with column headers (from the first row), aligned columns, row numbering, and horizontal scrolling for wide tables.
5. EACH Built_In_Viewer SHALL implement the `FileViewer` trait fully and be registered in the Viewer_Registry before any plugin initialization runs.

---

### Requirement 5: Plugin-Provided Viewers

**User Story:** As a plugin developer, I want to register custom viewers for my file types through the plugin architecture, so that I can extend the viewer system without modifying the core `ff-viewers` crate.

**Source:** FFE custom-file-viewers Req 1.3, adapted for workbench plugin architecture. [FFE, WB]

#### Acceptance Criteria

1. THE `PluginContext` (defined in `plugin-architecture`) SHALL expose a `register_viewer(viewer: Box<dyn FileViewer>) -> Result<(), PluginError>` method that plugins call during their `initialize` phase to contribute a viewer to the Viewer_Registry.
2. WHEN a plugin registers a viewer, THE Viewer_Registry SHALL validate the viewer's Viewer_Key for uniqueness and format compliance before accepting the registration.
3. WHEN a plugin enters its `shutdown` lifecycle phase, THE platform SHALL automatically deregister all viewers contributed by that plugin from the Viewer_Registry.
4. IF an active Viewer_Panel is displaying content through a plugin-provided viewer when that plugin shuts down, THE platform SHALL close the Viewer_Panel gracefully and display a message indicating that the viewer is no longer available.
5. THE `PluginContext` SHALL expose a `deregister_viewer(viewer_key: &str) -> Result<(), PluginError>` method allowing a plugin to remove its viewer proactively (e.g., during reconfiguration).
6. Plugin-provided viewers SHALL have the same capabilities as built-in viewers: they appear in `PREVIEW LIST`, are activatable via `PREVIEW <key>`, and can be referenced in language profile `default_viewer` declarations.

---

### Requirement 6: Viewer Selection

**User Story:** As a workbench user, I want the system to automatically suggest the best viewer for a file based on its extension or content, while still allowing me to manually choose any viewer via the PREVIEW command.

**Source:** FFE custom-file-viewers Req 2 (language profile viewer declaration), extended with content-sniffing. [FFE, WB]

#### Acceptance Criteria

1. WHEN a resource is opened and a registered FileViewer's `supported_extensions` list matches the resource's file extension, THE system SHALL record that viewer as the auto-detected default for the resource.
2. WHEN a resource is opened and the active language profile defines a `default_viewer` key, THE value of `default_viewer` SHALL take precedence over extension-based auto-detection for that resource.
3. WHEN auto-detection identifies a default viewer, THE system SHALL display a non-blocking status bar notification offering to activate that viewer (e.g., `ASA report detected — type PREVIEW or press F4 to view`). The user SHALL be able to dismiss the notification without activating the viewer.
4. WHEN no auto-detection matches and the user issues `PREVIEW ON`, THE system SHALL invoke the `can_render` method on all registered viewers with the resource URI and a content sample, selecting the first viewer that returns `true`. IF no viewer matches, THE system SHALL display a message indicating no suitable viewer is available.
5. THE user SHALL always be able to override auto-detection by issuing `PREVIEW <viewer-key>` to activate any registered viewer, regardless of file type.
6. WHEN the user declines a viewer offer (dismisses the notification), THE system SHALL NOT prompt again for that resource during the same session.

---

### Requirement 7: Viewer Panel as DockablePanel

**User Story:** As a workbench user, I want the viewer output to appear in a dockable panel that I can position, resize, split, and float like any other panel, so that it integrates seamlessly with my workspace layout.

**Source:** NEW — workbench layout integration requirement. [WB]

#### Acceptance Criteria

1. THE Viewer_Panel SHALL implement the `DockablePanel` trait defined in `layout-and-docking`, with panel_id `"viewer"`, default dock zone `Center`, and a title that includes the active Viewer_Key (e.g., `"Preview: asa-report"`).
2. THE Viewer_Panel SHALL be registered in the Panel_Registry during `ff-viewers` crate initialization, making it available for docking, floating, and persona configurations.
3. WHEN the user activates a viewer via `PREVIEW`, THE Viewer_Panel SHALL become visible in the center dock zone (or its last known position if previously moved), rendering the active viewer's output.
4. WHEN the user deactivates the viewer via `PREVIEW OFF`, THE Viewer_Panel SHALL be hidden (removed from view) while preserving its dock position in the Layout_State for future reactivation.
5. THE Viewer_Panel SHALL support being placed in a tab group alongside editor tabs, or in a split view beside the active editor, allowing side-by-side editing and previewing of the same resource.
6. THE Viewer_Panel SHALL support floating as an independent OS-level window via the standard drag-to-float or undock mechanism provided by the Layout_Engine.
7. THE Viewer_Panel's dock state (position, size, visibility) SHALL be included in persona serialization, allowing users to define personas that include or exclude the viewer panel.

---

### Requirement 8: Viewer Read-Only Constraint

**User Story:** As a workbench developer, I want a platform-level guarantee that viewers never modify document content, so that the view-only principle is enforced regardless of viewer implementation quality.

**Source:** FFE custom-file-viewers principle (view vs edit separation), formalized as platform constraint. [FFE, WB]

#### Acceptance Criteria

1. THE `render` method of the `FileViewer` trait SHALL receive content as an immutable byte slice (`&[u8]`) — no mutable reference to the document or edit buffer SHALL be accessible from within a viewer's render method.
2. THE Viewer_Panel SHALL NOT expose any editing affordances (no cursor insertion, no text selection for editing, no keyboard input that modifies document state). Clipboard copy of displayed text is permitted.
3. WHEN Viewer_Mode is active, keyboard and mouse input directed at the Viewer_Panel SHALL NOT produce Undo_Records or modify the document's edit state in any way.
4. IF a viewer implementation attempts to invoke a document-mutating command through the command framework, THE Command_Dispatch SHALL reject the execution with a `ViewerReadOnlyViolation` error.
5. THE platform SHALL log a warning if a viewer's `on_content_changed` implementation takes longer than 100ms, indicating the viewer may need optimization — but SHALL NOT allow it to block the editor thread.

---

### Requirement 9: Viewer Refresh on Document Change

**User Story:** As a workbench user, I want the viewer to automatically update when I edit the document in the editor, so that I always see a current preview without manually refreshing.

**Source:** FFE custom-file-viewers Req 5.4 (split view real-time update), generalized for workbench document change events. [FFE, WB]

#### Acceptance Criteria

1. WHEN the document backing the currently viewed resource is modified in the editor, THE platform SHALL notify the active FileViewer by calling its `on_content_changed` method with the updated content.
2. THE refresh notification SHALL be debounced: after a document change, THE platform SHALL wait for a configurable quiet period (default: 300ms of no further edits) before invoking `on_content_changed`, preventing excessive re-renders during rapid typing.
3. THE debounce interval SHALL be configurable via the `[viewers]` section of the workbench configuration (key: `refresh_debounce_ms`, type: positive integer, default: 300).
4. WHEN the underlying resource is modified externally (detected via VFS file-watcher events from `virtual-file-system`), THE platform SHALL reload the content through the VFS and invoke `on_content_changed` on the active viewer.
5. IF `on_content_changed` returns an error or panics, THE platform SHALL catch the failure, log a warning, and display a stale-content indicator in the Viewer_Panel rather than crashing or hiding the panel.
6. THE viewer refresh SHALL occur on a background thread or async task — it SHALL NOT block the editor's UI thread or interrupt the user's typing.

---

### Requirement 10: Viewer Configuration

**User Story:** As a workbench user, I want to configure viewer behaviour globally (auto-activation, default split mode, refresh interval) so that the viewer system adapts to my workflow preferences.

**Source:** FFE custom-file-viewers Req 6, adapted for workbench TOML configuration system. [FFE, WB]

#### Acceptance Criteria

1. THE workbench configuration SHALL accept a `[viewers]` section with the following optional keys:
   - `auto_offer`: boolean, default `true` — whether to display the auto-detection notification when a resource with a matching viewer is opened.
   - `default_position`: string enum (`"split-right"`, `"split-bottom"`, `"tab"`, `"float"`), default `"split-right"` — where the Viewer_Panel opens relative to the editor when activated.
   - `split_ratio`: float 0.1–0.9, default `0.5` — default split ratio (viewer fraction) when `default_position` is a split variant.
   - `refresh_debounce_ms`: positive integer, default `300` — debounce interval for viewer refresh after document changes.
2. WHEN a `[viewers]` configuration key contains an invalid value, THE system SHALL emit a configuration warning via the logging subsystem and apply the default for that key.
3. THE `[viewers]` configuration SHALL support hot-reload: changes to the configuration file SHALL be picked up without restarting the application, applying to the next viewer activation.
4. INDIVIDUAL viewers MAY define their own configuration sub-sections under `[viewers.<viewer-key>]` (e.g., `[viewers.asa-report]`), which the platform passes to the viewer during initialization. The `FileViewer` trait SHALL include an optional `configure(&mut self, config: &toml::Value)` method with a default no-op implementation.
