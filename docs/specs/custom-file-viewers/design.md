# Design Document: Custom File Viewers (`ff-viewers`)

## 1. Overview

The `ff-viewers` crate is the **extensible file viewing framework** for the FileForgeWorkbench platform. It provides a registry of content viewers, a `FileViewer` trait for viewer implementations, a unified `PREVIEW` command for activation, built-in viewers for common file types, and plugin-extensibility for custom viewer contributions.

### Purpose

- Define the `FileViewer` trait — the contract all viewer implementations fulfill
- Maintain a thread-safe `ViewerRegistry` mapping Viewer_Keys to viewer implementations
- Register the `PREVIEW` command (and sub-commands) in the Command_Registry
- Provide built-in viewers: `asa-report`, `hex`, `image`, `csv-table`
- Integrate with the plugin architecture for runtime viewer registration/deregistration
- Host viewer output in a `ViewerPanel` that implements `DockablePanel`
- Enforce the read-only constraint: viewers never modify document content
- Support debounced refresh when the underlying document changes
- Provide content-type matching (extensions, MIME types, content sniffing)

### Position in Architecture

```
Wave 12 — FileForge Domain

┌─────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)               │
│         Renders ViewerPanel; does NOT own viewer logic    │
├─────────────────────────────────────────────────────────┤
│  ff-viewers (THIS CRATE) │ ff-document-model │ ff-file-ops│
│  Viewer framework, registry, panel, built-in viewers     │
├─────────────────────────────────────────────────────────┤
│  ff-layout │ ff-command │ ff-plugin │ ff-vfs │ ff-config  │
│              (Platform Architecture — Wave 2/3)           │
├─────────────────────────────────────────────────────────┤
│                     ff-logging (Wave 0)                   │
└─────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **View-Only Rendering**: Viewers NEVER modify document content — enforced at the API level via immutable byte slices
- **Plugin Architecture (Req 5)**: Plugins register viewers via `PluginContext::register_viewer()` at runtime
- **Command-Driven (Req 3)**: All viewer operations are invoked through the `PREVIEW` command
- **DockablePanel Integration (Req 7)**: Viewer output renders in a panel that participates in the dock system
- **VFS Access Only (Req 9)**: Content is read through `ff-vfs` — viewers never access the filesystem directly
- **Multi-Crate Workspace**: Crate at `crates/ff-viewers`
- **Error Message Standards**: Errors follow `[viewers] operation: description` format

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Shell
        DESKTOP[ff-desktop<br/>GUI Shell / Renderer]
    end

    subgraph ff-viewers
        VR[ViewerRegistry<br/>viewer storage + lookup]
        VP[ViewerPanel<br/>DockablePanel impl]
        CS[ContentSelector<br/>auto-detection + matching]
        RC[RefreshController<br/>debounced change notify]
        CMD[PreviewCommand<br/>PREVIEW handler]
        BV[Built-in Viewers<br/>asa-report, hex, image, csv-table]
    end

    subgraph Upstream [Platform Crates]
        PLUGIN[ff-plugin<br/>PluginContext]
        LAYOUT[ff-layout<br/>PanelRegistry + DockablePanel]
        COMMAND[ff-command<br/>CommandRegistry]
        VFS[ff-vfs<br/>ResourceUri + content reads]
        CONFIG[ff-config<br/>viewer settings]
    end

    DESKTOP -->|renders| VP
    CMD -->|activates/deactivates| VP
    CMD -->|queries| VR
    CS -->|selects viewer| VR
    VP -->|calls render()| VR
    RC -->|calls on_content_changed()| VR
    BV -->|registered in| VR
    PLUGIN -->|register_viewer()| VR
    VP -->|implements| LAYOUT
    CMD -->|registered in| COMMAND
    RC -->|reads content via| VFS
    CS -->|reads config| CONFIG
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **ViewerRegistry** | Central map of Viewer_Key → `Box<dyn FileViewer>`; thread-safe; handles registration/deregistration |
| **ViewerPanel** | `DockablePanel` implementation; hosts active viewer's rendered output; manages visibility and dock state |
| **ContentSelector** | Determines which viewer (if any) should handle a given resource via extension, MIME, sniffing, or language profile |
| **RefreshController** | Debounces document changes and VFS watch events; calls `on_content_changed` on the active viewer |
| **PreviewCommand** | Command handler for `viewer.preview` — dispatches on/off/list/toggle/<viewer-key> actions |
| **Built-in Viewers** | Four `FileViewer` implementations compiled into the crate: asa-report, hex, image, csv-table |

---

## 3. Module Structure

```
crates/ff-viewers/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── traits.rs               # FileViewer trait definition
│   ├── registry.rs             # ViewerRegistry — registration, lookup, listing
│   ├── panel.rs                # ViewerPanel — DockablePanel impl, render dispatch
│   ├── selector.rs             # ContentSelector — auto-detection, matching logic
│   ├── refresh.rs              # RefreshController — debounce timer, change notification
│   ├── command.rs              # PreviewCommand — PREVIEW command handler registration
│   ├── key.rs                  # ViewerKey newtype — validation and parsing
│   ├── content_match.rs        # ContentMatch struct — extensions, MIME, sniffing results
│   ├── config.rs               # ViewerConfig — TOML [viewers] section parsing
│   ├── error.rs                # ViewerError enum
│   ├── builtin/
│   │   ├── mod.rs              # Built-in viewer re-exports
│   │   ├── asa_report.rs       # AsaReportViewer — ASA carriage control rendering
│   │   ├── hex.rs              # HexViewer — hex dump rendering
│   │   ├── image.rs            # ImageViewer — image preview rendering
│   │   └── csv_table.rs        # CsvTableViewer — CSV/TSV table grid rendering
│   └── integration.rs          # Startup wiring: registry population, command registration
└── tests/
    ├── registry_tests.rs       # Registry property tests
    ├── selector_tests.rs       # Content matching property tests
    ├── refresh_tests.rs        # Debounce property tests
    ├── command_tests.rs        # PREVIEW command property tests
    ├── key_tests.rs            # ViewerKey validation property tests
    ├── config_tests.rs         # Configuration parsing property tests
    └── integration.rs          # End-to-end viewer activation scenarios
```

---

## 4. Key Data Models and Types

### ViewerKey

```rust
/// A validated, unique identifier for a viewer. Non-empty string containing only
/// lowercase ASCII letters, digits, and hyphens. Examples: "asa-report", "hex",
/// "csv-table".
///
/// Addresses: Requirement 1, criterion 1
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ViewerKey(String);

impl ViewerKey {
    /// Parse and validate a viewer key string.
    /// Returns error if empty, contains invalid characters, or exceeds 64 chars.
    pub fn new(key: &str) -> Result<Self, ViewerError>;

    /// Returns the key as a string slice.
    pub fn as_str(&self) -> &str;
}
```

### FileViewer Trait

```rust
/// The core trait that all viewer implementations must implement.
/// Defines methods for rendering, supported content types, panel integration,
/// and refresh behaviour. Viewers are always read-only.
///
/// Addresses: Requirement 2, all criteria
pub trait FileViewer: Send + Sync {
    /// Returns the unique ViewerKey identifier.
    /// Addresses: Requirement 2, criterion 1 (viewer_key)
    fn viewer_key(&self) -> &str;

    /// Returns a human-readable display name (1 to 128 characters).
    /// Addresses: Requirement 2, criterion 1 (display_name)
    fn display_name(&self) -> &str;

    /// Returns a brief description of what this viewer renders.
    /// Addresses: Requirement 2, criterion 1 (description)
    fn description(&self) -> &str;

    /// Returns file extensions this viewer handles (e.g., ["lst", "rpt", "spool"]).
    /// Addresses: Requirement 2, criterion 1 (supported_extensions)
    fn supported_extensions(&self) -> &[&str];

    /// Returns MIME types this viewer handles (e.g., ["text/csv"]).
    /// Addresses: Requirement 2, criterion 1 (supported_mime_types)
    fn supported_mime_types(&self) -> &[&str];

    /// Returns whether this viewer can render the given resource, using URI
    /// metadata and/or a content sample for sniffing.
    /// Addresses: Requirement 2, criterion 1 (can_render)
    fn can_render(&self, uri: &ResourceUri, content_sample: &[u8]) -> bool;

    /// Renders the content into the provided egui UI region.
    /// Content is received as an immutable byte slice — no mutation is possible.
    /// Addresses: Requirement 2, criterion 1 (render); Requirement 8, criterion 1
    fn render(&self, content: &[u8], ui: &mut egui::Ui);

    /// Called when the underlying document changes, allowing the viewer to
    /// refresh its internal state (e.g., re-parse, update cached render data).
    /// Addresses: Requirement 2, criterion 1 (on_content_changed)
    fn on_content_changed(&mut self, new_content: &[u8]);

    /// Optional configuration method. Called during initialization and when
    /// the `[viewers.<key>]` configuration section changes at runtime.
    /// Default implementation is a no-op.
    /// Addresses: Requirement 10, criterion 4
    fn configure(&mut self, _config: &toml::Value) {}
}
```

### ViewerRegistry

```rust
/// Thread-safe registry mapping ViewerKeys to FileViewer implementations.
/// Populated at startup with built-in viewers, extended at runtime by plugins.
///
/// Addresses: Requirement 1, all criteria
pub struct ViewerRegistry {
    /// Map of viewer_key → viewer instance (behind Arc<RwLock> for thread safety)
    viewers: Arc<RwLock<HashMap<ViewerKey, ViewerEntry>>>,
}

/// An entry in the viewer registry, tracking the viewer and its provenance.
#[derive(Debug)]
struct ViewerEntry {
    /// The viewer implementation
    viewer: Box<dyn FileViewer>,
    /// Whether this is a built-in or plugin-contributed viewer
    source: ViewerSource,
    /// Display name cached for listing
    display_name: String,
    /// Description cached for listing
    description: String,
}

/// Identifies the origin of a registered viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerSource {
    /// Compiled into the ff-viewers crate
    BuiltIn,
    /// Contributed by a plugin at runtime
    Plugin,
}
```

### ViewerPanel

```rust
/// The DockablePanel implementation that hosts the active viewer's rendered output.
/// Manages the currently active viewer, content buffer, and panel lifecycle.
///
/// Addresses: Requirement 7, all criteria
pub struct ViewerPanel {
    /// The currently active viewer key (None if no viewer is active)
    active_viewer_key: Option<ViewerKey>,
    /// Cached content bytes for the current resource
    content_buffer: Vec<u8>,
    /// The ResourceUri of the currently viewed resource
    current_resource: Option<ResourceUri>,
    /// Whether the panel is currently visible
    visible: bool,
    /// Last known dock position for reactivation
    last_position: DockZone,
    /// Stale-content indicator (set when on_content_changed fails)
    stale: bool,
}
```

### ContentMatch

```rust
/// Describes how a viewer matches a given resource — used by ContentSelector
/// to rank and select the best viewer for auto-detection.
///
/// Addresses: Requirement 6, criteria 1/2/3/4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentMatch {
    /// The viewer key that matched
    pub viewer_key: ViewerKey,
    /// How the match was determined
    pub match_method: MatchMethod,
    /// Confidence score (higher = better match)
    pub confidence: MatchConfidence,
}

/// The method by which a viewer was matched to a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMethod {
    /// Matched via language profile `default_viewer` key
    LanguageProfile,
    /// Matched via file extension in `supported_extensions()`
    Extension,
    /// Matched via MIME type in `supported_mime_types()`
    MimeType,
    /// Matched via `can_render()` content sniffing
    ContentSniff,
    /// Explicit user selection via `PREVIEW <key>`
    UserExplicit,
}

/// Confidence level for content matching, used to rank multiple matching viewers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchConfidence {
    /// Low confidence — content sniff heuristic
    Low,
    /// Medium confidence — extension or MIME match
    Medium,
    /// High confidence — language profile explicit declaration
    High,
    /// Highest — user explicitly requested this viewer
    Explicit,
}
```

### ViewerConfig

```rust
/// Parsed representation of the `[viewers]` TOML configuration section.
///
/// Addresses: Requirement 10, all criteria
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViewerConfig {
    /// Whether to display auto-detection notifications. Default: true
    pub auto_offer: bool,
    /// Where the ViewerPanel opens relative to the editor. Default: SplitRight
    pub default_position: ViewerPosition,
    /// Split ratio (viewer fraction) for split positions. Default: 0.5
    pub split_ratio: f32,
    /// Debounce interval in milliseconds for viewer refresh. Default: 300
    pub refresh_debounce_ms: u64,
}

/// Panel opening position relative to the active editor.
/// Addresses: Requirement 10, criterion 1 (default_position)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViewerPosition {
    /// Open in a vertical split to the right of the editor
    SplitRight,
    /// Open in a horizontal split below the editor
    SplitBottom,
    /// Open as a tab alongside editor tabs
    Tab,
    /// Open as a floating window
    Float,
}
```

### RefreshController

```rust
/// Manages debounced refresh notifications from document changes and VFS watch events.
/// Ensures viewers are not overwhelmed by rapid edits.
///
/// Addresses: Requirement 9, all criteria
pub struct RefreshController {
    /// Debounce interval (from ViewerConfig)
    debounce_ms: u64,
    /// Timer handle for the pending refresh (reset on each new change)
    pending_timer: Option<TimerHandle>,
    /// Whether a refresh is currently in-flight on a background task
    refresh_in_flight: bool,
}

### ContentSelector

```rust
/// Determines which viewer should handle a given resource.
/// Implements the priority chain: language profile > extension > MIME > sniff.
///
/// Addresses: Requirement 6, all criteria
pub struct ContentSelector {
    /// Reference to the viewer registry for querying available viewers
    registry: Arc<RwLock<HashMap<ViewerKey, ViewerEntry>>>,
    /// Set of resource URIs that the user has dismissed the offer for (per-session)
    dismissed_offers: HashSet<ResourceUri>,
}
```

### PreviewCommandAction

```rust
/// Parsed action from a PREVIEW command invocation.
///
/// Addresses: Requirement 3, criteria 1–6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewCommandAction {
    /// Toggle viewer: activate default if off, deactivate if on
    Toggle,
    /// Activate the default viewer for the current resource
    On,
    /// Deactivate the active viewer
    Off,
    /// List all registered viewers
    List,
    /// Activate a specific viewer by key
    Activate(ViewerKey),
}
```

---

## 5. Public API Surface

### ViewerRegistry — Construction and Lifecycle

```rust
impl ViewerRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self;

    /// Register a built-in viewer. Called during crate initialization.
    /// Validates viewer_key format and uniqueness.
    /// Addresses: Requirement 1, criteria 1/3/6
    pub fn register_builtin(
        &self,
        viewer: Box<dyn FileViewer>,
    ) -> Result<(), ViewerError>;

    /// Register a plugin-provided viewer. Called from PluginContext.
    /// Validates viewer_key format and uniqueness.
    /// Addresses: Requirement 1, criteria 4/6; Requirement 5, criteria 1/2
    pub fn register_plugin(
        &self,
        viewer: Box<dyn FileViewer>,
    ) -> Result<(), ViewerError>;

    /// Deregister a viewer by key. Returns true if the viewer existed.
    /// Addresses: Requirement 1, criterion 5; Requirement 5, criteria 3/5
    pub fn deregister(&self, key: &ViewerKey) -> bool;

    /// Look up a viewer by key.
    pub fn get(&self, key: &ViewerKey) -> Option<&dyn FileViewer>;

    /// Returns whether a viewer key is currently registered.
    pub fn is_registered(&self, key: &ViewerKey) -> bool;

    /// List all registered viewers with metadata (key, display_name, description).
    /// Addresses: Requirement 1, criterion 7
    pub fn list_all(&self) -> Vec<ViewerInfo>;

    /// Returns all viewers whose supported_extensions include the given extension.
    pub fn viewers_for_extension(&self, ext: &str) -> Vec<ViewerKey>;

    /// Returns all viewers whose supported_mime_types include the given MIME type.
    pub fn viewers_for_mime(&self, mime: &str) -> Vec<ViewerKey>;

    /// Returns the source (BuiltIn or Plugin) for a given viewer key.
    pub fn viewer_source(&self, key: &ViewerKey) -> Option<ViewerSource>;
}

/// Summary information for a registered viewer (used in LIST output).
#[derive(Debug, Clone)]
pub struct ViewerInfo {
    pub key: ViewerKey,
    pub display_name: String,
    pub description: String,
    pub extensions: Vec<String>,
    pub mime_types: Vec<String>,
    pub source: ViewerSource,
}
```

### ContentSelector — Auto-Detection

```rust
impl ContentSelector {
    /// Create a new content selector backed by the given registry.
    pub fn new(registry: Arc<RwLock<HashMap<ViewerKey, ViewerEntry>>>) -> Self;

    /// Determine the best viewer for a resource based on the priority chain:
    /// 1. Language profile `default_viewer`
    /// 2. Extension match
    /// 3. MIME type match
    /// 4. Content sniffing via `can_render()`
    ///
    /// Returns None if no viewer matches.
    /// Addresses: Requirement 6, criteria 1/2/4
    pub fn select_viewer(
        &self,
        uri: &ResourceUri,
        content_sample: &[u8],
        language_profile_viewer: Option<&str>,
    ) -> Option<ContentMatch>;

    /// Record that the user dismissed the viewer offer for a resource.
    /// Addresses: Requirement 6, criterion 6
    pub fn dismiss_offer(&mut self, uri: &ResourceUri);

    /// Check whether the offer was already dismissed for this resource.
    pub fn is_offer_dismissed(&self, uri: &ResourceUri) -> bool;
}
```

### ViewerPanel — DockablePanel Implementation

```rust
impl DockablePanel for ViewerPanel {
    /// Returns panel_id "viewer".
    /// Addresses: Requirement 7, criterion 1
    fn panel_id(&self) -> &str;

    /// Returns DockZone::Center as the default zone.
    /// Addresses: Requirement 7, criterion 1
    fn default_dock_zone(&self) -> DockZone;

    /// Renders the active viewer's output. If no viewer is active, renders
    /// a placeholder message. If stale, renders a stale-content indicator.
    /// Addresses: Requirement 7, criterion 3; Requirement 8, criterion 2
    fn render(&mut self, ui: &mut egui::Ui);

    /// Returns title including active viewer (e.g., "Preview: asa-report").
    /// Addresses: Requirement 7, criterion 1
    fn title(&self) -> &str;

    /// Handles dock state transitions (visibility preserved).
    /// Addresses: Requirement 7, criterion 4
    fn on_dock_state_changed(&mut self, state: DockState);
}

impl ViewerPanel {
    /// Create a new ViewerPanel (initially hidden, no active viewer).
    pub fn new() -> Self;

    /// Activate a viewer for the given resource.
    /// Loads content via VFS and calls the viewer's render method.
    /// Addresses: Requirement 3, criteria 2/3/4; Requirement 7, criterion 3
    pub fn activate(
        &mut self,
        viewer_key: &ViewerKey,
        resource: &ResourceUri,
        content: Vec<u8>,
    );

    /// Deactivate the current viewer and hide the panel.
    /// Addresses: Requirement 3, criterion 5; Requirement 7, criterion 4
    pub fn deactivate(&mut self);

    /// Returns whether a viewer is currently active.
    pub fn is_active(&self) -> bool;

    /// Returns the currently active viewer key (if any).
    pub fn active_viewer_key(&self) -> Option<&ViewerKey>;

    /// Update content after a debounced document change.
    /// Sets stale indicator on failure.
    /// Addresses: Requirement 9, criteria 1/5
    pub fn refresh_content(&mut self, new_content: Vec<u8>);

    /// Mark content as stale (viewer failed to process update).
    /// Addresses: Requirement 9, criterion 5
    pub fn mark_stale(&mut self);

    /// Clear the stale indicator after successful refresh.
    pub fn clear_stale(&mut self);
}
```

### PreviewCommand — Command Handler

```rust
impl PreviewCommand {
    /// Create and register the PREVIEW command in the Command_Registry.
    /// Command_ID: "viewer.preview"
    /// Addresses: Requirement 3, criterion 1
    pub fn register(
        command_registry: &CommandRegistry,
        viewer_registry: Arc<ViewerRegistry>,
        viewer_panel: Arc<Mutex<ViewerPanel>>,
    ) -> Result<(), ViewerError>;

    /// Parse the command action parameter into a PreviewCommandAction.
    /// Accepts: None (toggle), "on", "off", "list", or a viewer-key string.
    pub fn parse_action(params: &CommandParams) -> Result<PreviewCommandAction, ViewerError>;

    /// Execute the PREVIEW command action.
    /// Addresses: Requirement 3, all criteria
    pub fn execute(
        &self,
        action: PreviewCommandAction,
        context: &ExecutionContext,
    ) -> Result<CommandResult, ViewerError>;
}
```

### RefreshController — Debounced Notifications

```rust
impl RefreshController {
    /// Create a new refresh controller with the given debounce interval.
    /// Addresses: Requirement 9, criterion 2
    pub fn new(debounce_ms: u64) -> Self;

    /// Notify that the document has changed. Resets the debounce timer.
    /// After the quiet period, will trigger `on_content_changed` on the viewer.
    /// Addresses: Requirement 9, criteria 1/2
    pub fn notify_document_changed(&mut self);

    /// Notify that the file was modified externally (VFS watch event).
    /// Addresses: Requirement 9, criterion 4
    pub fn notify_external_change(&mut self);

    /// Update the debounce interval (from config hot-reload).
    /// Addresses: Requirement 9, criterion 3; Requirement 10, criterion 3
    pub fn set_debounce_ms(&mut self, debounce_ms: u64);

    /// Check whether a refresh should fire (called each frame/tick).
    /// Returns true if the debounce period has elapsed.
    pub fn should_refresh(&mut self) -> bool;
}
```

---

## 6. Error Types

```rust
/// Errors produced by the viewer framework.
/// Formatted per Error Message Standards: `[viewers] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ViewerError {
    /// Invalid ViewerKey format
    #[error("[viewers] key: invalid format '{key}' — {reason}")]
    InvalidViewerKey { key: String, reason: String },

    /// Attempted to register a duplicate viewer key
    /// Addresses: Requirement 1, criterion 6
    #[error("[viewers] register: viewer '{key}' is already registered")]
    DuplicateViewerKey { key: String },

    /// Viewer key not found in the registry
    /// Addresses: Requirement 1, criterion 8
    #[error("[viewers] lookup: viewer '{key}' is not registered")]
    ViewerNotFound { key: String },

    /// No suitable viewer found for the given resource
    /// Addresses: Requirement 6, criterion 4
    #[error("[viewers] select: no suitable viewer for resource '{uri}'")]
    NoSuitableViewer { uri: String },

    /// Viewer's on_content_changed failed or panicked
    /// Addresses: Requirement 9, criterion 5
    #[error("[viewers] refresh: viewer '{key}' failed to process content update — {reason}")]
    RefreshFailed { key: String, reason: String },

    /// Viewer read-only constraint violated (command mutation attempted)
    /// Addresses: Requirement 8, criterion 4
    #[error("[viewers] readonly: viewer '{key}' attempted document mutation via command '{command}'")]
    ViewerReadOnlyViolation { key: String, command: String },

    /// Plugin registration error (wraps PluginError)
    /// Addresses: Requirement 5, criterion 1
    #[error("[viewers] plugin: registration failed — {reason}")]
    PluginRegistrationFailed { reason: String },

    /// Content read failed via VFS
    #[error("[viewers] content: failed to read resource '{uri}' — {reason}")]
    ContentReadFailed { uri: String, reason: String },

    /// Configuration error (invalid value in [viewers] section)
    /// Addresses: Requirement 10, criterion 2
    #[error("[viewers] config: invalid value for key '{key}' — {reason}")]
    ConfigInvalid { key: String, reason: String },

    /// Command framework integration error
    #[error("[viewers] command: {0}")]
    CommandError(String),
}
```

---

## 7. Integration Points

### With `ff-plugin` (Plugin Architecture — upstream)

- **Dependency direction**: ff-viewers depends on ff-plugin for `PluginContext` extension
- **API surface**:
  - `PluginContext::register_viewer(viewer: Box<dyn FileViewer>) -> Result<(), PluginError>` — called by plugins during `initialize`
  - `PluginContext::deregister_viewer(viewer_key: &str) -> Result<(), PluginError>` — called during plugin reconfiguration
- **Lifecycle integration**:
  - During plugin `initialize`: viewer is registered in ViewerRegistry with `ViewerSource::Plugin`
  - During plugin `shutdown`: all viewers contributed by that plugin are deregistered; active ViewerPanels using those viewers are closed
  - Addresses: Requirement 5, all criteria

### With `ff-layout` (Layout and Docking — upstream)

- **Dependency direction**: ff-viewers depends on ff-layout for `DockablePanel` trait and `PanelRegistry`
- **API consumed**:
  - `DockablePanel` trait: ViewerPanel implements this trait
  - `PanelRegistry::register()`: ViewerPanel is registered during `ff-viewers` initialization
  - `DockZone::Center`: Default dock zone for the ViewerPanel
  - `DockState` enum: Used in `on_dock_state_changed` notifications
- **Integration**:
  - ViewerPanel registers itself with panel_id `"viewer"` in the PanelRegistry at startup
  - ViewerPanel supports tab-group placement, split views, and floating (Requirement 7, criteria 5/6)
  - Panel dock state is included in persona serialization (Requirement 7, criterion 7)
  - Addresses: Requirement 7, all criteria

### With `ff-command` (Command Framework — upstream)

- **Dependency direction**: ff-viewers depends on ff-command for command registration
- **Commands registered**:
  - `viewer.preview` — The main PREVIEW command (Requirement 3, criterion 1)
- **Shortcut registrations**: F4 as default shortcut for `viewer.preview` toggle
- **Integration**:
  - PREVIEW command handler receives `ExecutionContext` for accessing the active resource
  - Command dispatch rejects document-mutating commands when Viewer_Mode is active (Requirement 8, criterion 4)
  - PREVIEW does NOT produce Undo_Records (Requirement 3, criterion 9)
  - Addresses: Requirement 3, all criteria

### With `ff-vfs` (Virtual File System — upstream)

- **Dependency direction**: ff-viewers depends on ff-vfs for content access
- **API consumed**:
  - `ResourceUri`: Identifies the resource being viewed
  - `VfsProvider::read()`: Async content reads for viewer rendering
  - `VfsWatcher` events: External file modification notifications
- **Integration**:
  - Content is loaded from VFS when a viewer is activated
  - VFS watch events trigger `RefreshController::notify_external_change()`
  - `can_render()` receives the ResourceUri for URI-based heuristics
  - Addresses: Requirement 9, criterion 4

### With `ff-config` (Configuration System — upstream)

- **Dependency direction**: ff-viewers depends on ff-config for settings
- **Configuration consumed**: The `[viewers]` TOML section (see Section 8)
- **Integration**:
  - Config is read at startup to initialize `ViewerConfig`
  - Hot-reload support: config changes apply to next viewer activation (Requirement 10, criterion 3)
  - Per-viewer config (`[viewers.<key>]`) is passed to `FileViewer::configure()` (Requirement 10, criterion 4)
  - Invalid config values emit WARN and fall back to defaults (Requirement 10, criterion 2)
  - Addresses: Requirement 10, all criteria

### With `ff-desktop` (Shell Layer — downstream)

- **Dependency direction**: ff-desktop depends on ff-viewers; ff-viewers NEVER depends on ff-desktop
- **Shell responsibilities**:
  - Render the ViewerPanel via `DockablePanel::render()` in its assigned dock position
  - Display the active ViewerKey in the status bar (Requirement 3, criterion 7)
  - Display auto-detection notifications (Requirement 6, criterion 3)
  - Display stale-content indicator when refresh fails (Requirement 9, criterion 5)
  - Forward keyboard/mouse events to ViewerPanel (read-only — no edit affordances)
  - Permit clipboard copy from viewer display (Requirement 8, criterion 2)

### Dependency Direction Summary

```
ff-logging ← ff-config ← ff-viewers ← ff-desktop
ff-plugin  ← ff-viewers
ff-layout  ← ff-viewers
ff-command ← ff-viewers
ff-vfs     ← ff-viewers
```

---

## 8. Configuration

### Workbench TOML Schema (`[viewers]` section)

```toml
[viewers]
# Whether to display the auto-detection notification when a matching viewer exists.
# Type: boolean. Default: true
# Addresses: Requirement 10, criterion 1 (auto_offer)
auto_offer = true

# Where the ViewerPanel opens relative to the active editor.
# Values: "split-right", "split-bottom", "tab", "float"
# Default: "split-right"
# Addresses: Requirement 10, criterion 1 (default_position)
default_position = "split-right"

# Split ratio (viewer fraction) when default_position is a split variant.
# Range: 0.1–0.9. Default: 0.5
# Addresses: Requirement 10, criterion 1 (split_ratio)
split_ratio = 0.5

# Debounce interval in milliseconds for viewer refresh after document changes.
# Type: positive integer. Default: 300
# Addresses: Requirement 9, criterion 3; Requirement 10, criterion 1 (refresh_debounce_ms)
refresh_debounce_ms = 300
```

### Per-Viewer Configuration (`[viewers.<key>]` sections)

```toml
[viewers.asa-report]
# Example: page_break_style for ASA viewer
page_break_style = "line"

[viewers.csv-table]
# Example: delimiter override
delimiter = ","
has_header = true
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `auto_offer` | `true` | `true` + WARN | N/A (boolean) |
| `default_position` | `"split-right"` | `"split-right"` + WARN | N/A (enum) |
| `split_ratio` | `0.5` | `0.5` + WARN | Clamp to [0.1–0.9] + WARN |
| `refresh_debounce_ms` | `300` | `300` + WARN | Clamp to [50–5000] + WARN |

---

## 9. Concurrency Model

### Thread-Safety Approach

The viewer framework operates across two thread contexts: the main/GUI thread for rendering and a background task for content refresh.

| Component | Thread Context | Mechanism |
|-----------|---------------|-----------|
| **ViewerRegistry** | Any thread | `Arc<RwLock<HashMap>>` — reads are concurrent; writes (register/deregister) acquire exclusive lock |
| **ViewerPanel** | Main thread | Single-threaded; `render()` called from the GUI event loop |
| **RefreshController** | Main thread + background | Timer runs on Tokio; `on_content_changed` dispatched to background task |
| **ContentSelector** | Main thread | Invoked synchronously during resource open or PREVIEW |
| **PreviewCommand** | Command dispatch thread | Handler may read registry; mutations go through main thread channel |

### Background Refresh Strategy

- Document changes trigger `RefreshController::notify_document_changed()` on the main thread
- After the debounce period elapses, a background task:
  1. Reads the latest content from VFS (async)
  2. Calls `on_content_changed()` on the active viewer
  3. Sends the result (success or error) back to the main thread via a channel
- If `on_content_changed` takes longer than 100ms, a WARN is logged (Requirement 8, criterion 5)
- The main thread never blocks on the refresh — it renders the last known good state
- Addresses: Requirement 9, criterion 6

### Panic Safety

- `on_content_changed` is wrapped in `std::panic::catch_unwind` when called on the background task
- If it panics, the ViewerPanel displays a stale-content indicator and logs WARN
- The viewer is NOT deregistered — the user can still attempt manual refresh
- Addresses: Requirement 9, criterion 5

---

## 10. Correctness Properties

These properties are suitable for property-based testing with `proptest`. They validate invariants that must hold across all valid inputs.

### Property 1: Viewer Registration Uniqueness

**Statement**: For any sequence of viewer registrations, the ViewerRegistry contains at most one entry per ViewerKey. A registration with an existing key always returns `DuplicateViewerKey` error without modifying state.

**Validates**: Requirement 1, criterion 6

```rust
// proptest strategy: generate sequences of (viewer_key, source) registration attempts
// assertion: after all registrations, registry.list_all() has no duplicate keys
//            AND every duplicate attempt returned Err(DuplicateViewerKey)
```

### Property 2: ViewerKey Format Validation

**Statement**: For any string, `ViewerKey::new()` succeeds if and only if the string is non-empty, at most 64 characters, and contains only lowercase ASCII letters, digits, and hyphens. All other inputs are rejected with `InvalidViewerKey`.

**Validates**: Requirement 1, criterion 1

```rust
// proptest strategy: generate arbitrary strings (ASCII and non-ASCII)
// assertion: ViewerKey::new(s).is_ok() ⟺ s matches regex ^[a-z0-9-]{1,64}$
```

### Property 3: Deregistration Removes Viewer Completely

**Statement**: After deregistering a ViewerKey, the registry no longer contains that key, `is_registered()` returns false, and a new viewer with the same key can be registered successfully.

**Validates**: Requirement 1, criterion 5; Requirement 5, criterion 3

```rust
// proptest strategy: register N viewers, deregister a random subset
// assertion: for each deregistered key: !registry.is_registered(key)
//            AND re-registration with same key succeeds
```

### Property 4: Content Selector Priority Chain

**Statement**: When multiple match methods are available for a resource, the ContentSelector returns the match with the highest confidence: LanguageProfile > Extension > MimeType > ContentSniff. UserExplicit is never auto-selected.

**Validates**: Requirement 6, criteria 1/2/4

```rust
// proptest strategy: generate resource with extension, MIME, and language profile settings
//                    configure viewers to match at multiple levels
// assertion: selected match method == highest priority among available matches
```

### Property 5: PREVIEW Toggle Idempotence

**Statement**: For any resource with an available viewer, calling PREVIEW (toggle) twice in sequence returns the viewer to its original state — if inactive, it activates then deactivates; if active, it deactivates then activates. The panel's `is_active()` state is restored.

**Validates**: Requirement 3, criterion 2

```rust
// proptest strategy: generate viewer panel in random state (active/inactive)
// assertion: toggle(toggle(state)).is_active() == state.is_active()
```

### Property 6: Read-Only Enforcement — Render Receives Immutable Content

**Statement**: The `render` method's content parameter type is `&[u8]` — an immutable borrow. For any FileViewer implementation, calling `render()` cannot modify the content buffer. The content byte-vector before and after render is identical.

**Validates**: Requirement 8, criterion 1

```rust
// proptest strategy: generate arbitrary content bytes and viewer
// assertion: content_before == content_after calling render()
//            (verified by passing a shared reference; compiler enforces this)
```

### Property 7: Debounce Timer Coalescing

**Statement**: For any sequence of N document changes arriving within a window shorter than the debounce interval, exactly one `on_content_changed` call is made (with the latest content), not N calls. The refresh fires only after the quiet period elapses.

**Validates**: Requirement 9, criterion 2

```rust
// proptest strategy: generate N change events with timestamps within debounce_ms
// assertion: on_content_changed called exactly once after last event + debounce_ms
```

### Property 8: Config Defaults Applied on Invalid Input

**Statement**: For any malformed `[viewers]` configuration value (wrong type, out of range, unknown enum variant), the ViewerConfig applies the documented default for that key and emits a warning. No panic, no error propagation.

**Validates**: Requirement 10, criterion 2

```rust
// proptest strategy: generate TOML [viewers] sections with invalid values
//                    (negative debounce, split_ratio > 1.0, garbage string for position)
// assertion: parsed config has default value for each invalid key
//            AND a warning was logged
```

### Property 9: Plugin Shutdown Closes Active Viewer Panels

**Statement**: When a plugin that contributed the currently active viewer shuts down, the ViewerPanel is deactivated and the viewer is removed from the registry. After shutdown, `is_active()` is false and `is_registered(key)` is false for that viewer.

**Validates**: Requirement 1, criterion 5; Requirement 5, criteria 3/4

```rust
// proptest strategy: register a plugin viewer, activate it, then simulate plugin shutdown
// assertion: viewer_panel.is_active() == false
//            AND registry.is_registered(key) == false
```

### Property 10: Viewer List Completeness

**Statement**: The list returned by `ViewerRegistry::list_all()` contains exactly one entry for every registered viewer — no more, no less. The count of entries equals the number of successful registrations minus the number of successful deregistrations.

**Validates**: Requirement 1, criterion 7; Requirement 3, criterion 6

```rust
// proptest strategy: generate sequence of register/deregister operations
// assertion: list_all().len() == count(successful_registers) - count(successful_deregisters)
//            AND all keys in list_all() are unique
```

---

## Appendix A: External Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `egui` | 0.28+ | `FileViewer::render` and `ViewerPanel::render` trait method signature |
| `serde` | 1.0 | Serialization derives for ViewerConfig, ViewerKey |
| `toml` | 0.8 | Configuration parsing and per-viewer config value passing |
| `thiserror` | 2.0 | Error type derivation |
| `image` | 0.25 | Image decoding for the built-in image viewer |
| `csv` | 1.3 | CSV parsing for the built-in csv-table viewer |
| `proptest` | 1.0 | Property-based testing (dev-dependency only) |

## Appendix B: Built-In Viewer Summary

| Viewer_Key | Display Name | Supported Extensions | MIME Types | Description |
|-----------|-------------|---------------------|------------|-------------|
| `asa-report` | ASA Report | lst, rpt, spool, asa | text/x-asa | Renders ASA carriage control report files with page breaks and formatting |
| `hex` | Hex Dump | bin, dat, exe, dll | application/octet-stream | Displays binary content as offset + hex bytes + ASCII decode columns |
| `image` | Image Preview | png, jpg, jpeg, gif, bmp, webp | image/png, image/jpeg, image/gif, image/bmp, image/webp | Renders scaled image preview; shows placeholder on decode failure |
| `csv-table` | CSV Table | csv, tsv | text/csv, text/tab-separated-values | Renders tabular data with headers, aligned columns, row numbers, horizontal scroll |

## Appendix C: PREVIEW Command Reference

| Invocation | Action | Addresses |
|-----------|--------|-----------|
| `PREVIEW` | Toggle viewer (activate default or deactivate) | Req 3.2 |
| `PREVIEW ON` | Activate default viewer for current resource | Req 3.3 |
| `PREVIEW OFF` | Deactivate viewer, hide panel | Req 3.5 |
| `PREVIEW LIST` | Display all registered viewers | Req 3.6 |
| `PREVIEW <key>` | Activate named viewer (e.g., `PREVIEW hex`) | Req 3.4 |

Default shortcut: **F4** (toggle)

## Appendix D: Status Bar Integration

When Viewer_Mode is active, the status bar displays:

```
Viewer: <viewer-key>
```

For example: `Viewer: asa-report`

When no viewer is active, the viewer status bar segment is hidden.

Addresses: Requirement 3, criterion 7

## Appendix E: Refresh Timing Diagram

```
Time ──────────────────────────────────────────────────────▶

Editor:    [edit]  [edit]  [edit]          (quiet)
                                    ◀─300ms─▶
RefreshCtrl: reset  reset  reset         │ fire │
                                              │
Background:                                   ├─ VFS read
                                              ├─ on_content_changed()
                                              └─ send result to main thread

Main Thread:                                         └─ update ViewerPanel
```

- Each editor change resets the debounce timer
- Only after 300ms of silence does the refresh trigger
- Refresh runs on background task; main thread continues rendering
- If refresh takes >100ms, WARN is logged (Requirement 8, criterion 5)
