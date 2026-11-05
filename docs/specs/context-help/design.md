# Design Document: Context-Sensitive Help (`ff-help`)

## Overview

The `ff-help` crate is the **context-sensitive help subsystem** for FileForgeWorkbench. It provides F1-triggered context detection, a dockable Help Panel model, a searchable Help Topic Registry, topic navigation with back/forward history, Markdown-based help content loading, and dynamic content generation for function key assignments.

### Purpose

- Detect the user's current context (active command, line command, mode, focused panel) and resolve the most relevant help topic
- Maintain a Help Topic Registry indexed by Topic_Key with O(1) lookup and keyword search
- Load help content from `.help.md` Markdown files at startup with hot-reload support
- Aggregate help topics from file-based content, command metadata, and plugin contributions
- Provide a Help Panel model (dockable, non-modal) with navigation stack, search, and breadcrumb
- Generate dynamic help content for function key assignments from the active key map
- Expose the `HELP` primary command for command-line help access
- Integrate with the command framework (F1 reserved shortcut) and layout system (DockablePanel)

### Position in Architecture

```
Wave 9 -- Desktop Integration

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
│   Renders HelpPanel via DockablePanel::render; Markdown -> UI  │
├──────────────────────────────────────────────────────────────┤
│         ff-help (THIS CRATE -- Wave 9)                         │
│   Context detection, topic registry, navigation, search,      │
│   content loading, HELP command, Help Panel model              │
├──────────────────────────────────────────────────────────────┤
│  ff-command (Wave 2) -- command dispatch, CommandMetadata,      │
│                         shortcut registry (F1 reserved)        │
│  ff-layout (Wave 2) -- DockablePanel trait, dock zones          │
│  ff-config (Wave 2) -- [help] config section, hot-reload        │
│  ff-plugin (Wave 2) -- plugin lifecycle (topic registration)    │
│  ff-keys (Wave 9) -- Key_Map, Shortcut_Registry (dynamic help) │
│  ff-core (Wave 2) -- EventBus, VFS file-watcher                │
├──────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging (Wave 0)             │
└──────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: All help logic (context detection, topic resolution, search, navigation) is GUI-free; the Help Panel rendering is shell-side via `DockablePanel::render`
- **Command-Driven (Req 4)**: Operations registered as commands (`help.show`, `help.search`, `help.back`, `help.forward`, `help.index`, `help.close`)
- **Keyboard Shortcut Registry (Req 10)**: F1 is reserved (hard-coded, non-overridable) -- always means Help
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-help`
- **Error Message Standards (Req 8)**: All errors follow `[help] operation: description` format
- **Plugin Architecture (Req 3)**: Plugins register/deregister help topics during lifecycle phases
- **Configuration Namespace (Req 5)**: Settings under `[help]` namespace in TOML

### Upstream Dependencies

| Crate | Usage |
|-------|-------|
| `ff-command` | `CommandRegistry` for HELP command registration; `CommandMetadata.help_text` / `help_syntax` for auto-topic creation; `ShortcutRegistry` for F1 reserved binding |
| `ff-layout` | `DockablePanel` trait implementation for Help Panel; `DockZone`, `DockState` |
| `ff-config` | `ConfigAccess` for `[help]` section keys; hot-reload subscription |
| `ff-plugin` | `PluginContext` for topic registration/deregistration during plugin lifecycle |
| `ff-keys` | `KeyMapResolver`, `GlobalKeyMap`, `ProfileKeyMap` for dynamic function key help generation |
| `ff-core` | `EventBus` for context-change events; VFS file-watcher for help content hot-reload |
| `ff-logging` | Structured diagnostics (WARN for missing topics, INFO for content reload) |

### Downstream Consumers

| Crate | Usage |
|-------|-------|
| `ff-desktop` | Renders Help Panel content (Markdown -> styled text) via `DockablePanel::render` |
| `ff-menu-statusbar` | Help_Menu items dispatch into this crate's commands |


---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Invocation Sources"
        F1[F1 Key Press<br/>reserved shortcut]
        CMD_LINE[Primary Command Line<br/>HELP, HELP CHANGE, etc.]
        MENU[Help Menu<br/>Index, Commands, Keys]
        LINK[Cross-Reference Link<br/>within help content]
        SEARCH[Help Panel Search<br/>keyword query]
    end

    subgraph "ff-help"
        CD[ContextDetector<br/>focus + command + mode -> TopicKey]
        TR[HelpTopicRegistry<br/>TopicKey -> HelpTopic, thread-safe]
        CL[ContentLoader<br/>.help.md file parser + indexer]
        HR[HotReloader<br/>VFS watcher -> re-index]
        NS[NavigationStack<br/>back/forward history]
        SE[SearchEngine<br/>keyword search + ranking]
        HP[HelpPanelModel<br/>current topic, breadcrumb, TOC]
        DG[DynamicGenerator<br/>function keys, config keys]
        CMD_H[HelpCommandHandler<br/>HELP primary command]
        CFG[HelpConfig<br/>typed config access]
    end

    subgraph "Upstream Crates"
        CMD_REG[ff-command<br/>CommandRegistry, Metadata]
        LAYOUT[ff-layout<br/>DockablePanel, DockZone]
        CONFIG[ff-config<br/>ConfigAccess]
        PLUGIN[ff-plugin<br/>PluginContext]
        KEYS[ff-keys<br/>KeyMapResolver]
        CORE[ff-core<br/>EventBus, VFS Watcher]
        LOG[ff-logging]
    end

    F1 --> CMD_REG
    CMD_LINE --> CMD_REG
    MENU --> CMD_REG
    CMD_REG --> CMD_H
    CMD_H --> CD
    CD --> TR
    TR --> HP
    LINK --> NS
    SEARCH --> SE
    SE --> TR

    CL --> TR
    HR --> CL
    CMD_REG -->|CommandMetadata.help_text| TR
    PLUGIN -->|topic registration| TR
    DG --> KEYS
    DG --> TR

    HP --> NS
    HP --> LAYOUT
    CFG --> CONFIG
    HR --> CORE
    TR --> LOG
end
```


### Layer Placement

| Layer | Role |
|-------|------|
| **Command Layer** | `HelpCommandHandler` -- translates F1, HELP primary command, menu actions into help engine calls |
| **Context Layer** | `ContextDetector` -- inspects focus, command input, prefix area, mode to resolve a `TopicKey` |
| **Registry Layer** | `HelpTopicRegistry` -- thread-safe store of all topics; aggregates file-based, command-based, and plugin topics |
| **Content Layer** | `ContentLoader`, `HotReloader`, `DynamicGenerator` -- load, parse, and generate help content |
| **Search Layer** | `SearchEngine` -- case-insensitive keyword matching with relevance ranking |
| **Presentation Layer** | `HelpPanelModel`, `NavigationStack` -- panel state, breadcrumb, TOC, navigation history |
| **Integration Layer** | `DockablePanel` implementation, config reader, plugin lifecycle hooks |

---

## Components and Interfaces

### Module Structure

```
crates/ff-help/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── topic.rs                # HelpTopic, TopicKey, TopicSource, HelpContent
│   ├── registry.rs             # HelpTopicRegistry -- thread-safe indexed store
│   ├── context.rs              # ContextDetector -- focus/command/mode -> TopicKey
│   ├── loader.rs               # ContentLoader -- .help.md file parser
│   ├── hot_reload.rs           # HotReloader -- VFS watcher integration
│   ├── search.rs               # SearchEngine -- keyword search + relevance ranking
│   ├── navigation.rs           # NavigationStack -- back/forward/history
│   ├── panel.rs                # HelpPanelModel -- panel state, breadcrumb, TOC
│   ├── dynamic.rs              # DynamicGenerator -- function keys, config key topics
│   ├── config.rs               # HelpConfig -- typed config access for [help] section
│   ├── commands/
│   │   ├── mod.rs              # Re-exports for all command handlers
│   │   ├── help_show.rs        # help.show -- F1 / HELP <topic> handler
│   │   ├── help_search.rs      # help.search -- search panel activation
│   │   ├── help_back.rs        # help.back -- navigate back
│   │   ├── help_forward.rs     # help.forward -- navigate forward
│   │   ├── help_index.rs       # help.index -- jump to Help Index
│   │   └── help_close.rs       # help.close -- close Help Panel (HELP OFF)
│   ├── error.rs                # HelpError enum
│   └── plugin_bridge.rs        # Plugin topic registration/deregistration hooks
└── tests/
    ├── context_tests.rs        # Context detection property tests
    ├── registry_tests.rs       # Topic registry property tests
    ├── loader_tests.rs         # Content loader property tests
    ├── search_tests.rs         # Search engine property tests
    ├── navigation_tests.rs     # Navigation stack property tests
    ├── panel_tests.rs          # Help panel model property tests
    ├── dynamic_tests.rs        # Dynamic generation property tests
    ├── config_tests.rs         # Config parsing property tests
    └── integration.rs          # End-to-end help scenarios
```


---

## Data Models

### TopicKey

```rust
/// A typed identifier for a help topic. Determines the lookup key in the registry.
/// Format: `"<namespace>:<identifier>"` or bare identifiers for special topics.
/// Addresses: Requirement 1 (1.2-1.7), Requirement 6 (6.1, 6.3)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicKey(String);

impl TopicKey {
    /// Create a command help topic key: `"cmd:CHANGE"`, `"cmd:FIND"`.
    pub fn command(name: &str) -> Self;

    /// Create a line command help topic key: `"line:CC"`, `"line:D"`.
    pub fn line_command(name: &str) -> Self;

    /// Create a mode help topic key: `"mode:hex"`, `"mode:edit"`.
    pub fn mode(name: &str) -> Self;

    /// Create a feature help topic key: `"feature:undo"`, `"feature:macros"`.
    pub fn feature(name: &str) -> Self;

    /// Create a config key help topic key: `"config:help_panel_position"`.
    pub fn config(name: &str) -> Self;

    /// Create a macro API function help topic key: `"api:cursor_line"`.
    pub fn api_function(name: &str) -> Self;

    /// The Help Index topic key.
    pub fn index() -> Self;

    /// The Getting Started topic key.
    pub fn getting_started() -> Self;

    /// Parse a raw string into a TopicKey.
    pub fn parse(raw: &str) -> Self;

    /// Returns the raw string value.
    pub fn as_str(&self) -> &str;

    /// Returns the namespace prefix (e.g., "cmd", "line", "mode").
    pub fn namespace(&self) -> Option<&str>;

    /// Returns the identifier portion after the colon.
    pub fn identifier(&self) -> &str;
}
```


### HelpTopic

```rust
/// A single unit of help content -- one topic per command, line command, feature, or mode.
/// Addresses: Requirement 5 (5.1-5.6), Requirement 6 (6.1-6.7)
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// The unique topic key for registry lookup.
    key: TopicKey,
    /// Human-readable title displayed at the top of the Help Panel.
    title: String,
    /// The Markdown body content of the help topic.
    content: HelpContent,
    /// Where this topic was registered from (for priority resolution).
    source: TopicSource,
    /// Optional aliases that also resolve to this topic.
    aliases: Vec<String>,
    /// Breadcrumb path segments (e.g., ["Help", "Commands", "CHANGE"]).
    breadcrumb: Vec<String>,
}

impl HelpTopic {
    pub fn new(key: TopicKey, title: String, content: HelpContent, source: TopicSource) -> Self;
    pub fn key(&self) -> &TopicKey;
    pub fn title(&self) -> &str;
    pub fn content(&self) -> &HelpContent;
    pub fn source(&self) -> TopicSource;
    pub fn aliases(&self) -> &[String];
    pub fn breadcrumb(&self) -> &[String];
    pub fn with_aliases(self, aliases: Vec<String>) -> Self;
    pub fn with_breadcrumb(self, breadcrumb: Vec<String>) -> Self;
}
```

### HelpContent

```rust
/// The body content of a help topic, stored as raw Markdown.
/// The GUI shell is responsible for rendering Markdown -> styled text.
/// Addresses: Requirement 5 (5.3)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpContent {
    /// Raw Markdown text content.
    markdown: String,
    /// Cross-reference links extracted during parsing ([link](topic_key)).
    cross_references: Vec<TopicKey>,
    /// Section headings extracted for Table of Contents generation.
    sections: Vec<SectionHeading>,
}

impl HelpContent {
    pub fn new(markdown: String) -> Self;
    pub fn markdown(&self) -> &str;
    pub fn cross_references(&self) -> &[TopicKey];
    pub fn sections(&self) -> &[SectionHeading];
    /// Parse Markdown to extract cross-references and section headings.
    pub fn parse(markdown: String) -> Self;
}

/// A section heading extracted from Markdown content for TOC display.
/// Addresses: Requirement 3 (3.4)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionHeading {
    /// Heading level (1 = `#`, 2 = `##`, etc.).
    pub level: u8,
    /// Heading text.
    pub text: String,
    /// Byte offset in the Markdown content (for scroll-to-section).
    pub offset: usize,
}
```


### TopicSource

```rust
/// Identifies where a help topic was registered from.
/// Used for priority resolution (runtime > file-based).
/// Addresses: Requirement 6 (6.4, 6.5, 6.6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TopicSource {
    /// Loaded from a `.help.md` file on disk.
    FileBased,
    /// Auto-generated from CommandMetadata.help_text at command registration.
    CommandMetadata,
    /// Contributed by a plugin during its `initialize` lifecycle phase.
    Plugin,
    /// Dynamically generated at display time (function keys, config index).
    Dynamic,
}
```

### HelpPanelState

```rust
/// The current state of the Help Panel, driving rendering decisions.
/// Addresses: Requirement 2 (2.1-2.10), Requirement 3 (3.1-3.6)
#[derive(Debug, Clone)]
pub struct HelpPanelState {
    /// Whether the Help Panel is currently visible.
    is_open: bool,
    /// The currently displayed topic (None if panel is closed).
    current_topic: Option<TopicKey>,
    /// The rendered topic content for display.
    current_content: Option<HelpTopic>,
    /// Navigation history stack.
    navigation: NavigationStack,
    /// Active search query (empty if not searching).
    search_query: String,
    /// Search results (empty if no active search).
    search_results: Vec<SearchResult>,
    /// Vertical scroll position within the current topic (0.0-1.0).
    scroll_position: f32,
    /// Whether the TOC sidebar is expanded.
    toc_expanded: bool,
}

impl HelpPanelState {
    pub fn new() -> Self;
    pub fn is_open(&self) -> bool;
    pub fn current_topic(&self) -> Option<&TopicKey>;
    pub fn current_content(&self) -> Option<&HelpTopic>;
    pub fn navigation(&self) -> &NavigationStack;
    pub fn search_query(&self) -> &str;
    pub fn search_results(&self) -> &[SearchResult];
    pub fn scroll_position(&self) -> f32;
    pub fn toc_expanded(&self) -> bool;
}
```


### NavigationStack

```rust
/// A back/forward navigation history for the Help Panel.
/// Each topic visit pushes onto the stack. Back/forward traverse without removing entries.
/// Addresses: Requirement 3 (3.1-3.6)
#[derive(Debug, Clone)]
pub struct NavigationStack {
    /// Ordered history of visited topics.
    entries: Vec<TopicKey>,
    /// Current position index within entries (0-based).
    position: usize,
}

impl NavigationStack {
    /// Create a new empty navigation stack.
    pub fn new() -> Self;

    /// Push a new topic onto the stack, discarding any forward history.
    pub fn push(&mut self, key: TopicKey);

    /// Navigate back. Returns the previous TopicKey, or None if at start.
    pub fn back(&mut self) -> Option<&TopicKey>;

    /// Navigate forward. Returns the next TopicKey, or None if at end.
    pub fn forward(&mut self) -> Option<&TopicKey>;

    /// Returns the current topic key.
    pub fn current(&self) -> Option<&TopicKey>;

    /// Whether back navigation is possible.
    pub fn can_go_back(&self) -> bool;

    /// Whether forward navigation is possible.
    pub fn can_go_forward(&self) -> bool;

    /// Clear the entire stack (called when Help Panel is closed and reopened).
    pub fn clear(&mut self);

    /// Returns the total number of entries in the stack.
    pub fn len(&self) -> usize;

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool;
}
```

### HelpIndex

```rust
/// The top-level help index content -- auto-generated from the registry.
/// Lists all topic categories with navigable links.
/// Addresses: Requirement 12 (12.1-12.4)
#[derive(Debug, Clone)]
pub struct HelpIndex {
    /// Getting started topic link.
    pub getting_started: TopicKey,
    /// Alphabetical list of primary command topics with one-line descriptions.
    pub commands: Vec<IndexEntry>,
    /// Compact reference table of line command topics.
    pub line_commands: Vec<IndexEntry>,
    /// Editor mode topics.
    pub modes: Vec<IndexEntry>,
    /// Feature topics.
    pub features: Vec<IndexEntry>,
    /// Configuration overview link.
    pub configuration: TopicKey,
    /// Function keys topic link (dynamically generated).
    pub function_keys: TopicKey,
    /// Macro API entry point link.
    pub macro_api: TopicKey,
    /// Application name and version string.
    pub app_version: String,
}

/// A single entry in the Help Index category listing.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Topic key for navigation.
    pub key: TopicKey,
    /// Display title.
    pub title: String,
    /// One-line description.
    pub description: String,
}
```


### SearchResult

```rust
/// A single result from a help topic search.
/// Addresses: Requirement 4 (4.1-4.5)
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matching topic's key.
    pub key: TopicKey,
    /// The matching topic's title.
    pub title: String,
    /// A brief excerpt showing the matching context (highlighted substring).
    pub excerpt: String,
    /// Relevance score for ranking (higher = more relevant).
    pub relevance: u32,
    /// Where the match was found.
    pub match_location: MatchLocation,
}

/// Where within a topic the search match was found -- used for ranking.
/// Addresses: Requirement 4 (4.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchLocation {
    /// Exact match on the topic title (highest relevance).
    Title,
    /// Match in a section heading.
    Heading,
    /// Match in the topic body text (lowest relevance).
    Body,
    /// Match on a TopicKey alias.
    Alias,
}
```

### ContextState

```rust
/// Snapshot of the current editor context used by ContextDetector to resolve a TopicKey.
/// Provided by the shell layer at the moment F1 is pressed or HELP is invoked.
/// Addresses: Requirement 1 (1.1-1.9)
#[derive(Debug, Clone)]
pub struct ContextState {
    /// Current content of the command input field (trimmed).
    pub command_input: String,
    /// Whether the command input field currently has keyboard focus.
    pub command_input_focused: bool,
    /// Content of the focused prefix area cell (if any).
    pub prefix_area_content: Option<String>,
    /// Whether a prefix area cell currently has keyboard focus.
    pub prefix_area_focused: bool,
    /// The currently active editor mode.
    pub active_mode: Option<EditorMode>,
    /// The currently focused panel ID (if any non-editor panel has focus).
    pub focused_panel_id: Option<String>,
    /// Whether the Help Panel is currently open.
    pub help_panel_open: bool,
    /// The currently displayed topic in the Help Panel (for toggle detection).
    pub current_help_topic: Option<TopicKey>,
}

/// Editor mode identifiers for context detection.
/// Addresses: Requirement 1 (1.5, 1.9), Requirement 11 (11.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Browse,
    Edit,
    View,
    Hex,
    Preview,
    GridBrowse,
    GridEdit,
}
```


### HelpConfig

```rust
/// Typed configuration for the help subsystem, loaded from [help] TOML section.
/// Addresses: Requirement 16 (16.1-16.3)
#[derive(Debug, Clone, PartialEq)]
pub struct HelpConfig {
    /// Custom path to help content directory. None uses default search locations.
    pub directory: Option<String>,
    /// Help Panel width as a fraction of window width (0.2-0.5, default 0.35).
    pub panel_width_ratio: f32,
    /// Default dock zone for the Help Panel ("right", "left", "bottom").
    pub panel_position: HelpPanelPosition,
    /// Whether to highlight search matches in help content.
    pub search_highlight: bool,
}

impl Default for HelpConfig {
    fn default() -> Self {
        Self {
            directory: None,
            panel_width_ratio: 0.35,
            panel_position: HelpPanelPosition::Right,
            search_highlight: true,
        }
    }
}

/// The configured dock zone for the Help Panel.
/// Addresses: Requirement 16 (16.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpPanelPosition {
    Right,
    Left,
    Bottom,
}
```

---

## Public API Surface

### ContextDetector

```rust
/// Inspects the current editor state and resolves the most relevant TopicKey.
/// Implements best-effort context detection: command input > prefix area > mode > fallback to index.
/// Addresses: Requirement 1 (1.1-1.9)
pub struct ContextDetector;

impl ContextDetector {
    /// Resolve the most relevant TopicKey from the given context state.
    ///
    /// Priority order:
    /// 1. Command input field focused + contains recognisable command -> `cmd:<NAME>`
    /// 2. Command input field focused + empty -> `index`
    /// 3. Prefix area focused + contains line command -> `line:<CMD>`
    /// 4. Active special mode (Hex, Preview, Grid_*) -> `mode:<MODE>`
    /// 5. Fallback -> `index`
    pub fn resolve(state: &ContextState) -> TopicKey;

    /// Determine if F1 should toggle the Help Panel closed (same topic redisplay).
    /// Returns true if the panel is open and the resolved topic matches current display.
    pub fn should_toggle_close(state: &ContextState, resolved: &TopicKey) -> bool;
}
```


### HelpTopicRegistry

```rust
/// Thread-safe store of all help topics, indexed by TopicKey.
/// Supports O(1) lookup, keyword search, and runtime registration/deregistration.
/// Addresses: Requirement 6 (6.1-6.7)
pub struct HelpTopicRegistry {
    /// TopicKey -> HelpTopic mapping (RwLock for concurrent read access).
    topics: Arc<RwLock<HashMap<TopicKey, HelpTopic>>>,
    /// Alias -> TopicKey mapping for alternative lookups.
    aliases: Arc<RwLock<HashMap<String, TopicKey>>>,
    /// Plugin ID -> set of TopicKeys contributed by that plugin (for cleanup).
    plugin_topics: Arc<RwLock<HashMap<String, Vec<TopicKey>>>>,
}

impl HelpTopicRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self;

    /// Look up a topic by key. Returns None if not found.
    pub fn get(&self, key: &TopicKey) -> Option<HelpTopic>;

    /// Look up a topic by alias string. Returns None if alias not registered.
    pub fn get_by_alias(&self, alias: &str) -> Option<HelpTopic>;

    /// Register a topic from file-based content.
    /// Does NOT overwrite existing runtime-registered topics (lower priority).
    pub fn register_file_topic(&self, topic: HelpTopic);

    /// Register a topic from CommandMetadata (auto-created at command registration).
    /// Overwrites file-based topics for the same key.
    pub fn register_command_topic(&self, topic: HelpTopic);

    /// Register a topic from a plugin. Associates with plugin_id for cleanup.
    /// Overwrites file-based topics for the same key.
    pub fn register_plugin_topic(&self, plugin_id: &str, topic: HelpTopic);

    /// Remove all topics contributed by the given plugin.
    pub fn deregister_plugin(&self, plugin_id: &str);

    /// Bulk-register topics from a content load (startup or hot-reload).
    pub fn load_file_topics(&self, topics: Vec<HelpTopic>);

    /// Check if a topic exists for the given key.
    pub fn contains(&self, key: &TopicKey) -> bool;

    /// Return all registered TopicKeys (for index generation).
    pub fn all_keys(&self) -> Vec<TopicKey>;

    /// Return all topics in a given namespace (e.g., "cmd", "line", "mode").
    pub fn topics_in_namespace(&self, namespace: &str) -> Vec<HelpTopic>;

    /// Total number of registered topics.
    pub fn len(&self) -> usize;

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool;
}
```

### ContentLoader

```rust
/// Parses `.help.md` files and produces HelpTopic instances for registry population.
/// Addresses: Requirement 5 (5.1-5.6)
pub struct ContentLoader;

impl ContentLoader {
    /// Load all `.help.md` files from the given directory path.
    /// Returns parsed topics and any parse warnings.
    pub fn load_directory(path: &Path) -> Result<ContentLoadResult, HelpError>;

    /// Parse a single `.help.md` file into one or more HelpTopics.
    /// Topic delimiters: `<!-- TOPIC: topic_key -->` followed by `<!-- TITLE: title -->`.
    pub fn parse_file(path: &Path, content: &str) -> Result<Vec<HelpTopic>, HelpError>;

    /// Resolve the help content directory using the search order:
    /// 1. Custom path from config (`help_directory`)
    /// 2. Directory containing the workbench binary
    /// 3. User data directory
    pub fn resolve_directory(config: &HelpConfig) -> Option<PathBuf>;
}

/// Result of loading help content from a directory.
#[derive(Debug)]
pub struct ContentLoadResult {
    /// Successfully parsed topics.
    pub topics: Vec<HelpTopic>,
    /// Files that could not be parsed (path + error message).
    pub warnings: Vec<(PathBuf, String)>,
    /// Total number of files scanned.
    pub files_scanned: usize,
}
```


### SearchEngine

```rust
/// Keyword search across all loaded help topics with relevance ranking.
/// Addresses: Requirement 4 (4.1-4.5)
pub struct SearchEngine;

impl SearchEngine {
    /// Search all topics in the registry for the given query.
    /// Minimum query length is 2 characters; returns empty for shorter queries.
    /// Results are ranked: title matches > heading matches > body matches.
    pub fn search(registry: &HelpTopicRegistry, query: &str) -> Vec<SearchResult>;

    /// Case-insensitive substring match against a single topic.
    /// Returns the best MatchLocation or None if no match.
    fn match_topic(topic: &HelpTopic, query: &str) -> Option<(MatchLocation, String)>;

    /// Extract a context excerpt around the first match position (+/-40 chars).
    fn extract_excerpt(text: &str, query: &str) -> String;
}
```

### DynamicGenerator

```rust
/// Generates help topics dynamically at display time (not stored in registry).
/// Addresses: Requirement 15 (15.1-15.4)
pub struct DynamicGenerator;

impl DynamicGenerator {
    /// Generate the function keys help topic from the active key map.
    /// Produces a Markdown table: Key | Command | Label.
    pub fn generate_function_keys(
        key_map: &dyn KeyMapAccess,
        shortcut_registry: &dyn ShortcutRegistryAccess,
    ) -> HelpTopic;

    /// Generate the Help Index topic from the current registry state.
    /// Addresses: Requirement 12 (12.1-12.4)
    pub fn generate_index(
        registry: &HelpTopicRegistry,
        app_version: &str,
    ) -> HelpTopic;
}
```

### HelpPanelModel

```rust
/// The core model for the Help Panel -- manages display state, navigation, search.
/// The GUI shell reads this model to render the panel content.
/// Addresses: Requirement 2 (2.1-2.10), Requirement 3 (3.1-3.6)
pub struct HelpPanelModel {
    state: HelpPanelState,
    registry: Arc<HelpTopicRegistry>,
    config: HelpConfig,
}

impl HelpPanelModel {
    pub fn new(registry: Arc<HelpTopicRegistry>, config: HelpConfig) -> Self;

    /// Open the panel and display the given topic.
    pub fn show_topic(&mut self, key: &TopicKey) -> Result<(), HelpError>;

    /// Close the Help Panel and clear navigation history.
    pub fn close(&mut self);

    /// Toggle: if open with same topic, close; otherwise show new topic.
    pub fn toggle(&mut self, key: &TopicKey) -> Result<(), HelpError>;

    /// Navigate back in history.
    pub fn navigate_back(&mut self) -> Result<(), HelpError>;

    /// Navigate forward in history.
    pub fn navigate_forward(&mut self) -> Result<(), HelpError>;

    /// Navigate to the Help Index.
    pub fn navigate_to_index(&mut self) -> Result<(), HelpError>;

    /// Follow a cross-reference link to another topic.
    pub fn follow_link(&mut self, key: &TopicKey) -> Result<(), HelpError>;

    /// Execute a search query. Updates search_results in state.
    pub fn search(&mut self, query: &str);

    /// Clear active search and return to current topic display.
    pub fn clear_search(&mut self);

    /// Get the current panel state for rendering.
    pub fn state(&self) -> &HelpPanelState;

    /// Update configuration (e.g., after hot-reload).
    pub fn update_config(&mut self, config: HelpConfig);

    /// Whether the panel is currently open.
    pub fn is_open(&self) -> bool;
}
```


### HelpCommandHandler

```rust
/// Command handler for the HELP primary command.
/// Routes `HELP`, `HELP <topic>`, `HELP LINECOMMANDS`, `HELP KEYS`, `HELP OFF`, etc.
/// Addresses: Requirement 13 (13.1-13.10)
pub struct HelpCommandHandler {
    panel: Arc<Mutex<HelpPanelModel>>,
    context_detector: ContextDetector,
}

impl CommandHandler for HelpCommandHandler {
    fn is_undoable(&self) -> bool { false }

    fn execute(&self, ctx: &ExecutionContext, params: &CommandParams) -> CommandResult;
}

impl HelpCommandHandler {
    /// Parse the HELP command arguments and dispatch to the appropriate action.
    ///
    /// Routing rules:
    /// - No args -> show Help Index
    /// - `OFF` -> close Help Panel
    /// - `LINECOMMANDS` -> show `"line:index"` topic
    /// - `KEYS` -> show dynamically generated function key topic
    /// - `MACRO` or `API` -> show `"feature:macros"` topic
    /// - `CONFIG` or `CONFIGURATION` -> show `"feature:configuration"` topic
    /// - `<name>` -> try `"cmd:<NAME>"`, then `"feature:<name>"`, then unrecognised message
    fn resolve_help_argument(&self, args: &str) -> HelpAction;
}

/// The resolved action for a HELP command invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpAction {
    /// Show a specific topic.
    ShowTopic(TopicKey),
    /// Close the Help Panel.
    Close,
    /// Show the Help Index with an "unrecognised topic" message.
    UnrecognisedTopic(String),
}
```

### Plugin Bridge

```rust
/// Interface for plugins to register and deregister help topics.
/// Exposed via PluginContext during plugin lifecycle.
/// Addresses: Requirement 6 (6.1, 6.6)
pub struct HelpPluginBridge {
    registry: Arc<HelpTopicRegistry>,
}

impl HelpPluginBridge {
    pub fn new(registry: Arc<HelpTopicRegistry>) -> Self;

    /// Register a help topic from a plugin.
    /// Called during the plugin `initialize` lifecycle phase.
    pub fn register_topic(
        &self,
        plugin_id: &str,
        key: TopicKey,
        title: String,
        markdown_content: String,
    );

    /// Remove all help topics contributed by this plugin.
    /// Called during the plugin `shutdown` lifecycle phase.
    pub fn deregister_all(&self, plugin_id: &str);
}
```


---

## Error Handling

```rust
/// All errors produced by the help subsystem.
/// Follows the `[help] operation: description` format standard.
/// Addresses: Requirement 5 (5.5, 5.6), Requirement 16 (16.2)
#[derive(Debug, thiserror::Error)]
pub enum HelpError {
    /// Requested topic key does not exist in the registry.
    #[error("[help] lookup: topic not found -- {key}")]
    TopicNotFound { key: String },

    /// Help content directory not found at any search location.
    #[error("[help] content: help directory not found (searched: {searched_paths})")]
    DirectoryNotFound { searched_paths: String },

    /// No `.help.md` files found in the help directory.
    #[error("[help] content: no help files found in {directory}")]
    NoHelpFiles { directory: String },

    /// Failed to read a `.help.md` file.
    #[error("[help] content: failed to read {path} -- {source}")]
    FileReadError { path: String, source: std::io::Error },

    /// Failed to parse a `.help.md` file (invalid topic delimiter format).
    #[error("[help] parse: invalid topic format in {path} at line {line} -- {reason}")]
    ParseError { path: String, line: usize, reason: String },

    /// Navigation stack is empty -- cannot go back.
    #[error("[help] navigation: no previous topic in history")]
    NoPreviousTopic,

    /// Navigation stack has no forward entry.
    #[error("[help] navigation: no next topic in history")]
    NoNextTopic,

    /// Configuration value is invalid (fallback applied, logged as warning).
    #[error("[help] config: invalid value for {key}, using default -- {reason}")]
    InvalidConfig { key: String, reason: String },

    /// Help Panel is not open (close/navigate operation on closed panel).
    #[error("[help] panel: Help Panel is not open")]
    PanelNotOpen,

    /// Hot-reload file watcher registration failed.
    #[error("[help] reload: failed to watch {path} -- {reason}")]
    WatcherError { path: String, reason: String },

    /// Plugin topic registration failed (duplicate key from same plugin).
    #[error("[help] plugin: duplicate topic key {key} from plugin {plugin_id}")]
    DuplicatePluginTopic { plugin_id: String, key: String },
}
```

---

## Integration Points

### Command Framework (`ff-command`)

| Command ID | Default Shortcut | Handler | Description |
|-----------|-----------------|---------|-------------|
| `help.show` | F1 (reserved) | `commands::help_show` | Context-sensitive help -- resolves topic from current state |
| `help.search` | *(none)* | `commands::help_search` | Activate search mode in Help Panel |
| `help.back` | Alt+Left | `commands::help_back` | Navigate to previous topic |
| `help.forward` | Alt+Right | `commands::help_forward` | Navigate to next topic |
| `help.index` | *(none)* | `commands::help_index` | Navigate to Help Index |
| `help.close` | Escape | `commands::help_close` | Close Help Panel (also via HELP OFF) |
| `help.command` | *(primary command)* | `HelpCommandHandler` | HELP primary command dispatcher |

Integration notes:
- F1 is registered as a **reserved shortcut** via `ShortcutRegistry::register_reserved()` -- cannot be overridden by plugins or user key maps
- `help.show` and `help.command` do NOT produce undo records (`is_undoable() -> false`)
- `help.show` and `help.command` are NOT added to command history (Requirement 1.10, 13.10)
- The system reads `CommandMetadata.help_text` and `help_syntax` fields from all registered commands to auto-populate the topic registry (Requirement 6.2, 6.3)


### Layout and Docking (`ff-layout`)

The Help Panel implements `DockablePanel`:

```rust
impl DockablePanel for HelpPanel {
    fn panel_id(&self) -> &str { "help_panel" }

    fn default_dock_zone(&self) -> DockZone {
        // Converted from HelpConfig.panel_position at construction
        DockZone::Right
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        // Shell-side: reads HelpPanelState from HelpPanelModel
        // Renders Markdown content, breadcrumb, TOC, search field, nav controls
    }

    fn title(&self) -> &str { "Help" }

    fn on_dock_state_changed(&mut self, state: DockState) {
        // Adjust rendering for narrow-width warning (Requirement 2.9)
    }

    fn minimum_size(&self) -> Option<(f32, f32)> {
        Some((200.0, 100.0))
    }
}
```

Integration notes:
- Help Panel defaults to right dock zone, width ratio from config (Requirement 2.2)
- Panel width is resizable via standard dock zone divider (Requirement 2.3)
- When docked width < 200px, displays narrow-width advisory message (Requirement 2.9)
- Panel participates in tab groups and floating windows per layout system rules

### Configuration System (`ff-config`)

The help system reads from the `[help]` TOML configuration section:

| Key | Type | Default | Effect |
|-----|------|---------|--------|
| `help.directory` | `String` | *(none)* | Custom path to help content directory |
| `help.panel_width_ratio` | `f32` | `0.35` | Help Panel width as fraction of window (0.2-0.5) |
| `help.panel_position` | `String` | `"right"` | Default dock zone: `"right"`, `"left"`, `"bottom"` |
| `help.search_highlight` | `bool` | `true` | Highlight search matches in content |

Integration notes:
- Invalid values trigger `HelpError::InvalidConfig` warning log and fallback to defaults (Requirement 16.2)
- Help system subscribes to hot-reload events for `help.*` keys (Requirement 16.3)
- On config change, `HelpPanelModel::update_config()` applies new settings without restart

### Plugin Architecture (`ff-plugin`)

| Lifecycle Phase | Action |
|----------------|--------|
| `initialize` | Plugin calls `HelpPluginBridge::register_topic()` to add help topics for its commands |
| `shutdown` | Help system calls `HelpTopicRegistry::deregister_plugin(plugin_id)` to remove all topics contributed by the unloading plugin |

Integration notes:
- Plugin-registered topics have higher priority than file-based content (Requirement 6.4)
- Plugin deregistration is automatic -- the help system listens for plugin shutdown events via `EventBus`
- `HelpPluginBridge` is exposed to plugins via `PluginContext::help()` accessor

### Function Keys and History (`ff-keys`)

The help system uses a read-only accessor trait to query the active key map:

```rust
/// Trait for reading key map state -- implemented by ff-keys, consumed by ff-help.
/// Decouples ff-help from ff-keys implementation details.
pub trait KeyMapAccess: Send + Sync {
    /// Returns all assigned function key bindings (F1-F24).
    fn function_key_bindings(&self) -> Vec<FunctionKeyBinding>;
    /// Returns the name of the active profile (if any).
    fn active_profile_name(&self) -> Option<String>;
}

/// A single function key binding entry for help display.
#[derive(Debug, Clone)]
pub struct FunctionKeyBinding {
    pub key: String,         // e.g., "F3"
    pub command_id: String,  // e.g., "file.close"
    pub label: String,       // e.g., "Close"
}
```

Integration notes:
- Dynamic generation happens at display time, not at registry load (always current)
- If no key map is configured, displays a "how to configure" message (Requirement 15.4)
- Profile key map display indicates which profile is active (Requirement 15.3)


---

## Correctness Properties

The following properties are designed for verification with the `proptest` crate. Each property maps to one or more acceptance criteria from `requirements.md`.

### Property 1: Context Detection Always Resolves a Valid TopicKey

**Statement:** For any valid `ContextState`, `ContextDetector::resolve()` always returns a `TopicKey` -- never panics, never returns an empty or malformed key. If no specific context is detected, it falls back to the Help Index key.

**Validates: Requirements 1.1, 1.3, 1.5, 1.7, 1.9**

**Strategy:** Generate arbitrary `ContextState` values with varied combinations of command input, prefix area, modes, and focus states. Assert that the returned TopicKey is non-empty and follows the `<namespace>:<id>` format or is the known index key.

---

### Property 2: Context Priority Order -- Command Input Dominates

**Statement:** When `command_input_focused` is true and `command_input` contains a non-empty recognisable command token, the resolved TopicKey always has namespace `"cmd"`, regardless of prefix area content, active mode, or focused panel.

**Validates: Requirements 1.2, 1.3**

**Strategy:** Generate `ContextState` with `command_input_focused = true` and `command_input` containing a valid command name (from a known set). Vary all other fields randomly. Assert the result has namespace `"cmd"`.

---

### Property 3: Navigation Stack Back/Forward Consistency

**Statement:** For any sequence of `push`, `back`, and `forward` operations on a `NavigationStack`, the following invariants hold:
- `back()` followed by `forward()` returns to the same topic
- After `push`, `can_go_back()` is true (if stack had at least one prior entry)
- After `push`, `can_go_forward()` is false (forward history is discarded)
- `current()` always returns the topic at the current position

**Validates: Requirements 3.1, 3.2, 3.3, 3.6**

**Strategy:** Generate a sequence of navigation operations (push random TopicKeys, back, forward) as a command list. Execute the sequence and verify invariants at each step.

---

### Property 4: Search Results Relevance Ordering

**Statement:** For any search query and topic set, search results are always sorted by descending relevance score. Title matches always appear before heading matches, which appear before body matches.

**Validates: Requirements 4.1, 4.2, 4.4**

**Strategy:** Generate a registry with topics having titles, headings, and body text containing various substrings. Execute searches and verify the `relevance` field is monotonically non-increasing and `MatchLocation` ordering is respected.

---

### Property 5: Topic Registry Priority Resolution

**Statement:** When both a file-based topic and a runtime-registered topic (command metadata or plugin) exist for the same TopicKey, `registry.get()` always returns the runtime-registered topic. When a plugin is deregistered, file-based fallback is restored.

**Validates: Requirements 6.4, 6.5, 6.6**

**Strategy:** Register file-based topics, then runtime topics for overlapping keys. Assert get() returns the runtime version. Deregister the runtime source. Assert get() returns the file-based version.

---

### Property 6: Content Loader Topic Delimiter Parsing

**Statement:** For any well-formed `.help.md` content containing N topic delimiters (`<!-- TOPIC: key -->` / `<!-- TITLE: title -->`), `ContentLoader::parse_file()` produces exactly N `HelpTopic` instances, each with the correct key and title.

**Validates: Requirements 5.2, 5.4**

**Strategy:** Generate `.help.md` content with 1-10 randomly generated topic blocks using valid delimiter syntax. Parse and verify the count, keys, and titles match.

---

### Property 7: Help Panel Toggle Behaviour

**Statement:** If the Help Panel is open displaying topic T, and `toggle(T)` is called, the panel closes. If `toggle(U)` is called where U != T, the panel navigates to U and remains open.

**Validates: Requirements 1.6, 2.4**

**Strategy:** Create a `HelpPanelModel` with a populated registry. Open a random topic. Call toggle with the same key -- assert closed. Open again. Call toggle with a different key -- assert open and displaying the new key.

---

### Property 8: Help Config Validation and Fallback

**Statement:** For any raw configuration values, `HelpConfig` construction with invalid values (panel_width_ratio outside 0.2-0.5, unrecognised panel_position) applies defaults and logs a warning, never panics or produces an unusable config.

**Validates: Requirements 16.1, 16.2**

**Strategy:** Generate arbitrary f32 values for width ratio (including NaN, infinity, negatives, values > 1.0) and arbitrary strings for position. Assert the resulting config always has width_ratio in [0.2, 0.5] and position is one of the three valid values.

---

### Property 9: HELP Command Routing Correctness

**Statement:** For any input string to the HELP command, `resolve_help_argument()` returns a deterministic, well-defined `HelpAction`. Empty maps to `ShowTopic(index)`, `"OFF"` maps to `Close`, `"LINECOMMANDS"` maps to `ShowTopic("line:index")`, `"KEYS"` maps to `ShowTopic("feature:function_keys")`, a known command name maps to `ShowTopic("cmd:<NAME>")`, and an unknown name maps to `UnrecognisedTopic(name)`.

**Validates: Requirements 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8**

**Strategy:** Generate the known dispatch keywords plus arbitrary strings. Assert routing matches the specification rules exactly.

---

### Property 10: Search Minimum Query Length Enforcement

**Statement:** For any query string with length < 2, `SearchEngine::search()` returns an empty result set. For any query with length >= 2, the result set contains only topics whose title, headings, body, or aliases contain the query as a case-insensitive substring.

**Validates: Requirements 4.1, 4.2, 4.5**

**Strategy:** Generate queries of length 0-1 and assert empty results. Generate queries of length 2+ with a known topic set and verify all results contain the query substring (case-insensitive) in at least one searchable field.

---

### Property 11: Navigation Stack Clear on Reopen

**Statement:** After `close()` followed by `show_topic(key)`, the navigation stack contains exactly one entry (the newly shown topic) and `can_go_back()` is false.

**Validates: Requirements 3.6**

**Strategy:** Build a navigation stack with multiple entries. Call close. Call show_topic with a new key. Assert stack length is 1 and back is unavailable.

---

---

## External Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 1.0 | Error derive macros |
| `proptest` | 1.0 | Property-based testing (dev-dependency) |
| `egui` | 0.28+ | `DockablePanel::render` trait method signature only (shell-side) |
| `serde` | 1.0 | Deserialization of HelpConfig from TOML |

---

## Testing Strategy

- **Unit tests**: `#[cfg(test)] mod tests` block at the bottom of each source module, testing individual functions in isolation
- **Property-based tests**: `proptest` crate with minimum 256 cases per property; 11 properties defined covering all major subsystem behaviours
- **Integration tests**: `tests/integration.rs` for end-to-end scenarios (F1 press -> context detection -> registry lookup -> panel state update)
- **Test doubles**: `MockKeyMapAccess` for function key generation; `MockConfigAccess` for config tests; in-memory `HelpTopicRegistry` with pre-loaded topics for all non-I/O tests
- **File I/O tests**: Use `tempfile::TempDir` with synthetic `.help.md` files for ContentLoader and HotReloader tests
- **No GUI testing in this crate**: All panel logic is testable via `HelpPanelModel` state assertions; actual Markdown rendering is the shell's responsibility

---

## Design Decisions and Rationale

1. **Markdown as help content format** -- Chosen over plain text for richer formatting (code blocks, links, headings) while remaining easy to author and version-control. The GUI shell handles Markdown->styled rendering; the core crate stores raw Markdown.

2. **Topic delimiter syntax** (`<!-- TOPIC: key -->`) -- Uses HTML comments rather than YAML front-matter to allow multiple topics per file without complex parsing. Valid Markdown that renderers ignore.

3. **Registry priority (runtime > file-based)** -- Ensures dynamically registered commands always have up-to-date help without requiring file updates. File-based content serves as fallback documentation.

4. **Dynamic generation for function keys** -- Generated at display time rather than cached because key maps can change at runtime (profile switches, hot-reload). Avoids stale content.

5. **GUI-free panel model** -- `HelpPanelModel` contains all logic; the shell's `DockablePanel::render` merely reads state and draws. This allows unit testing all help behaviour without a GUI framework.

6. **Thread-safe registry with RwLock** -- Allows concurrent reads from multiple threads (rendering, search) while serializing writes (plugin registration, hot-reload). Matches the workbench concurrency model.

7. **Navigation stack clears on close** -- Per ISPF convention, each F1 press starts a fresh help session. Users do not accumulate unbounded history across multiple help invocations.

---

## CR-NR-097 Delta: Shipped Content Set + F1 Display Pipeline (Requirements 17, 18)

This delta records the design decisions for authoring the shipped help content and
for the shell-side pipeline that loads it and displays a resolved topic. It does
NOT contradict the earlier design; it makes concrete the two pieces that were left
implicit (the actual content files, and how the shell displays a resolved topic),
and it re-phrases the panel-integration decision onto the current framework.

### D1. Help content is a shipped directory of `.help.md` files (no change to Requirement 5)

The storage format is unchanged and already specified: a `help/` DIRECTORY of
Markdown `.help.md` files (NOT a single monolithic file, NOT an HTML tree), each
file holding one or more topics delimited by `<!-- TOPIC: <key> -->` /
`<!-- TITLE: ... -->` (HTML-comment delimiter; YAML front-matter is the accepted
alternative). Topics are indexed by `Topic_Key`; cross-references are Markdown
links whose target is a `Topic_Key` (`[FIND](cmd:FIND)`); the egui shell renders
Markdown to styled text. This delta only COMMITS the authored files.

Authoring layout (grouped by category for maintainability; the loader is
directory-recursive so grouping is a convenience, not a contract):

```
help/
  index.help.md              # getting_started + any hand-authored index prose
                             #   (the "index" topic itself is generated -- D3)
  commands/
    find.help.md             # cmd:FIND, cmd:RFIND (aliases share a file)
    change.help.md           # cmd:CHANGE, cmd:RCHANGE
    navigation.help.md       # cmd:UP/DOWN/TOP/BOTTOM/LOCATE
    session.help.md          # cmd:SAVE/CANCEL/END/UNDO/REDO
    display.help.md          # cmd:HEX/EXCLUDE/SHOW/RESET
    menu-options.help.md     # cmd:POM, cmd:SETTINGS, cmd:FILES, cmd:HELP, cmd:KEYS
  line-commands/
    line-index.help.md       # line:index summary table
    line-basic.help.md       # line:D/I/R/C/CC/M/MM/A/B/X/... per Req 8.3
  modes/
    modes.help.md            # mode:browse/edit/view/hex/preview/grid_browse/grid_edit
  features/
    undo.help.md             # feature:undo
    macros.help.md           # feature:macros (also HELP MACRO / HELP API)
    function-keys.help.md     # feature:function_keys prose (TABLE is generated -- D3)
    command-history.help.md   # feature:command_history
    tabs.help.md             # feature:tabs
    docking.help.md          # feature:docking
    configuration.help.md     # feature:configuration
```

Content authoring is verified structurally, not by prose: a test asserts every
`Topic_Key` promised by Requirement 17 is present after load, and that no
cross-reference in shipped content is dangling (Req 17.8).

**Help content is rendered product content, not project documentation (amended
CR-NR-103 / CR-NR-109).** The shipped `.help.md` files under the WHOLE `help/`
tree -- the English base in the `help/` root AND every per-locale
`help/<locale>/` subdirectory -- are RENDERED PRODUCT CONTENT for the planned
dedicated help viewer (a future feature that will display the help markdown
properly rendered and allow topic navigation). They are therefore a DISTINCT
ARTIFACT CLASS from project documentation (docs/**) and from Rust source, and the
project ASCII documentation character-set rule DOES NOT apply to any file under
`help/`. Shipped help files MAY contain non-ASCII UTF-8 and SHALL be valid UTF-8
(Req 17.9). This is the rationale for exempting the whole tree, English base
included; the earlier narrower framing (English base stays ASCII) is discarded.

### D2. The shell owns ONE registry, loaded once at startup

Today `shell/commands.rs` does `let registry = HelpTopicRegistry::new();` inside
the `HELP` arm on every invocation -- an empty registry that can never contain a
topic, which is exactly why `resolve_with_fallback` returns
`"Help not yet available..."`. The fix (Req 18.1):

- The shell holds a single `help_registry: Arc<HelpTopicRegistry>` field,
  populated once during startup by loading the resolved `help/` directory
  (`ContentLoader::resolve_directory` per Req 5.1 -> `load_directory`), then
  augmented with command-metadata topics and (later) plugin topics.
- The `HELP`/F1 handler borrows that shared registry instead of newing one.
- `ContentLoader` already exists; the only new `ff-help` surface is a thin
  convenience entry (e.g. `HelpService::load_default(config) -> Arc<HelpTopicRegistry>`)
  if one is not already present -- wiring, not new subsystems.

### D3. Dynamic topics resolve at display time

`index` and `feature:function_keys` are NOT authored as files; they are produced by
the existing `DynamicGenerator` (`generate_index`, `generate_function_keys`) at
display time (Req 18.3), so the index always reflects the loaded set and the
function-key table always reflects the live Key_Map. The display path checks: is
the resolved key one of the dynamic keys? generate it. Else look it up in the
registry. Else fall back to the generated index with an "unrecognised topic"
message (Req 18.4).

### D4. Panel integration RE-PHRASED: `WorkspaceContext`, not `DockablePanel` (framework conformance)

The "Layout and Docking" integration section above (and design Decision 5) predates
the `WorkspaceContext` framework (CR-NR-078) and the workspace tab-order conformance
rule (CR-CH-023). Per `framework-conformance` and `workspace-conformance`, the Help
panel MUST be a Workspace Context:

- Implement `WorkspaceContext` for the Help Context; its
  `render(&mut self, ui, &mut ShellServices) -> InteriorFocus` returns the focus
  contract, dispatched by the shell through `render_workspace_context` (the single
  focus-latch path). Do NOT hand-write a per-arm focus ring, and do NOT implement
  `DockablePanel` directly for focus purposes.
- The FIRST interior focus stop is the Help_Search field, which carries a stable
  `egui::Id` (e.g. `egui::Id::new("help_search_field")`); because the panel content
  (topic body, nav controls) is variable, the search field MAY serve as both the
  first and last interior stop, with egui-native Tab walking the controls between
  (the pattern the conformance rule documents).
- The `HelpPanelModel` remains the GUI-free state holder; the Context's `render`
  reads it and draws, exactly as Decision 5 intended -- only the focus/dispatch
  contract changes to the framework's.

This is a re-phrasing of pre-framework wording onto the current framework (recorded
in `change-log.md` CR-NR-097), not a framework change: no new dispatch path, no
second navigation stack, no new persistence format.

### D5. Single dispatch path for HELP and F1 (framework conformance)

HELP is a command resolved through `resolve_target` and dispatched via
`target_dispatch.rs` (a `Function` target); F1 is the reserved shortcut that invokes
the same command; Help menu items invoke the HELP command rather than calling help
internals (Req 18.6). The existing `if upper == "HELP"` inline arm in `commands.rs`
is replaced by dispatch through the standard path as part of this wiring (closing a
pre-framework intercept, consistent with framework-conformance Principle 1). HELP
and F1 stay out of command history and undo (Req 18.7), preserving Req 1.10 / 13.10.

### D6. Testing delta

- Content-set test (Req 17): load the shipped `help/`; assert every promised
  `Topic_Key` is present; assert zero dangling cross-references. The former
  "assert ASCII" over the shipped set HAS BEEN REMOVED/RELAXED for the whole
  `help/` tree (Req 17.9 amended, CR-NR-103 / CR-NR-109): the ASCII sub-assertion
  is superseded, and the test is now `shipped_content_files_are_valid_utf8`,
  walking the whole tree (English base included) and asserting only valid UTF-8.
  This whole-tree UTF-8 test has LANDED (the former `shipped_content_files_are_ascii`
  no longer exists).
- Pipeline test (Req 18.1-18.4): a shell-level test that the registry is loaded
  once and reused, that a resolved file-based topic displays, that `index` /
  `feature:function_keys` generate-and-display, and that an unknown key shows the
  index-with-message rather than only an `open_error`.
- Full-shell first-Tab focus test (Req 18.5, MANDATORY per workspace-conformance):
  `build_eframe` harness, open the Help Context, assert `first_interior_id` is the
  Help_Search field and the FIRST Tab from the command field lands on it (no
  phantom stop). Model on `full_shell_config_first_tab_focuses_filter_field`.

### D7. Missing-topic diagnostics (Requirement 19)

The single choke point for a help miss already exists:
`ContextDetector::resolve_with_fallback` returns
`Err("Help not yet available for <label> [topic-key: <key>]")` precisely when the
resolved key is absent from the registry. The diagnostics attach at that seam and
at the shell display path (which also knows the dynamic keys), so a miss is defined
once, not scattered.

- **WARN log (Req 19.1).** On a miss that is not a dynamic key, emit one WARN via
  `ff-logging` naming the `Topic_Key` and label. This is the trace the owner asked
  for -- it shows which topics F1/HELP actually reach in real use.
- **Session tally (Req 19.2, 19.6).** A shell-owned `MissingTopicTally`
  (`HashMap<TopicKey, u32>`), in-memory and session-scoped (NOT persisted). Each
  miss increments the count for its key. This lives on the shell alongside the
  `Arc<HelpTopicRegistry>` (D2), not in a file, so runtime behaviour never mutates
  the repo.
- **`HELP MISSING` report (Req 19.3, 19.4).** A HELP subcommand (routed through the
  same single dispatch path as the rest of HELP, per D5) that renders the tally in
  the Help Context. Each row is classified EXPECTED (key is in the Requirement 17
  promised set -> authoring backlog, already tracked by CR-NR-097) or UNEXPECTED
  (key no requirement promised -> a real coverage gap, a triage candidate). The
  "promised set" is the same set the content-set test (D6) checks, so the two share
  one source of truth for which keys are expected.
- **No auto-filing (Req 19.5).** The report only DISPLAYS candidates; it never
  writes to `bugs.md` / `change-log.md` / specs. Escalating an UNEXPECTED miss is a
  human triage decision. Rationale: an EXPECTED miss is not a new defect or
  requirement -- it is a not-yet-done item already owned by CR-NR-097, so
  auto-filing would flood the tracking docs with duplicates of one known item
  (exactly the noise `workflow.md` triage avoids). Only UNEXPECTED misses carry new
  information, and those are surfaced for a person to triage deliberately.

Testing (Req 19): unit-test the tally (distinct-key counting, increment on repeat);
unit-test the EXPECTED/UNEXPECTED classification against a known promised set; a
shell test that a miss increments the tally and emits a WARN and does NOT touch any
docs file; a `HELP MISSING` display test.

---

## Markdown Help Rendering (CR-NR-100)

This delta EVALUATES rendering the Help Context body with Markdown formatting
instead of raw monospace text (Requirement 20). It is a PROPOSAL: it lays out the
rendering-target options and RECOMMENDS one, but the final choice is an open design
question for the owner to confirm (criterion 20.7). It does NOT contradict the
earlier design; it concretises the display seam that D4 (Help Context render) and
Requirement 18.2 established.

### M1. Current text-rendering approach (what exists today)

Help content is already Markdown and already carried RAW through the engine:

- `ff-help` parses `.help.md` topics (`content_parser.rs`), and `HelpPanelModel`
  stores the body UNRENDERED -- `HelpTopic.body()` returns the raw Markdown string
  (design Decisions 1 and 5: "the core crate stores raw Markdown"; "content is data,
  not code").
- The shell-side Help Context (`crates/ff-desktop/src/help_context.rs`,
  `HelpContextPanel::render`) draws the body with a single
  `ui.label(egui::RichText::new(topic.body()).monospace())` inside a vertical
  `ScrollArea`. The in-source comment there states the monospace label is a
  placeholder and "a richer Markdown renderer is a later enhancement." THIS is the
  exact seam Requirement 20 targets.
- The HELP/F1 pipeline (`crates/ff-desktop/src/shell/help.rs`) loads one shell-owned
  `Arc<HelpTopicRegistry>` at startup, resolves a topic, and opens the Help Context
  (D2-D5). HELP routes through the single command-dispatch path. None of that
  pipeline changes for Req 20; only the body-drawing call inside the Help Context
  `render` is replaced by a Markdown-aware rendering, with a monospace fallback.

### M2. The rendering-target options (with tradeoffs)

The available Markdown machinery (CR-CH-049) is `ff-md-viewer::render_to_html`
(pulldown-cmark -> an HTML FRAGMENT string, with tables/footnotes/strikethrough/
task-lists/smart-punctuation enabled) and `ff-mdx-plugin::MdxFileViewer` (the
ff-viewers `FileViewer` for md/markdown, rendering via `render_to_html`).

CRITICAL TARGET GAP: egui does NOT render HTML natively. An HTML fragment from
`render_to_html` therefore cannot be dropped into the Help Context `ui` directly;
it needs an HTML-capable surface. This gap drives the option analysis.

- **Option A -- Direct egui Markdown widgets.** Render the raw Markdown STRAIGHT to
  egui widgets with an egui-native Markdown widget (e.g. an `egui_commonmark`-style
  renderer), inside the Help Context `render`. No HTML intermediary, so the
  HTML-vs-egui gap does not arise. Fits the existing Help Context render path (just
  replaces the one `ui.label(...monospace())` call). Does NOT hard-depend on the
  Task 21 viewer-to-shell seam for the rendering itself. Tradeoff: introduces a new
  rendering dependency (an egui Markdown widget crate) and its feature set must be
  checked against Req 20.2 (tables, task lists, etc.); link activation must be
  wired to Help navigation (Req 20.3). This keeps help rendering a Help-Context
  concern rather than a viewer concern.

- **Option B -- `render_to_html` + an HTML view.** Reuse `ff-md-viewer::render_to_html`
  and display the HTML fragment in an HTML/webview surface. Tradeoff: egui has no
  native HTML renderer, so this needs a webview or an HTML-rendering widget -- a
  heavy dependency (an embedded browser or a substantial HTML layout engine),
  disproportionate to a help panel, and awkward to drive headlessly in
  `egui_kittest`. Likely OUT OF SCOPE for the Help Context. Recorded for
  completeness.

- **Option C -- Reuse the ff-viewers ViewerPanel / `MdxFileViewer` (post-Task 21).**
  Once the viewer-to-shell seam exists (custom-file-viewers Req 14 / Task 21),
  surface help bodies through the SAME PREVIEW path the Markdown viewer uses, so
  help and file preview share one Markdown rendering. Tradeoff: INHERITS the Task 21
  / Req 14 dependency (the seam does not exist yet -- `ff-desktop` has no ff-viewers
  / ff-mdx-plugin dependency and no PREVIEW wiring), AND inherits the same
  HTML-fragment-vs-egui target gap unless the ViewerPanel itself resolves it. It is
  the most framework-consistent LONGER-TERM convergence, but cannot land before
  Task 21 and the ff-desktop decoupling.

### M3. Dependencies and sequencing

Regardless of option, implementation is ORDERED AFTER two things (criterion 20.8):

- the ff-desktop DECOUPLING work -- the DECOMP waves (CR-NR-098 Req 19.3/19.4 in
  `docs/project-management/project-master/tasks.md`); the owner wants decoupling
  finished before wiring new rendering into `ff-desktop`; and
- Task 21 / custom-file-viewers Requirement 14 -- the viewer-to-shell seam. Option C
  depends on it hard; Options A/B do not strictly need it for rendering but should
  not pre-empt the decoupling.

So: specify + task-list now (this gate), implement later.

### M4. Recommendation (for owner confirmation -- NOT finalised)

RECOMMEND Option A (direct egui Markdown widgets) for the Help Context body:

- It avoids the HTML-fragment-vs-egui target gap entirely (no webview).
- It does NOT hard-depend on the Task 21 viewer-to-shell seam, so once the
  decoupling is done the Help render can proceed on its own schedule.
- It is a localised change to the one `help_context.rs` draw call, preserving the
  rest of the pipeline (D2-D5) and the WorkspaceContext/InteriorFocus contract
  (Req 20.6).
- Option C remains the longer-term convergence target IF/WHEN Task 21 lands and the
  owner wants help and file-preview to share one renderer; the design should keep
  the rendering behind a small internal seam so a later switch from A to C is a
  localised change.

RATIONALE FOR THE LOWER-COUPLING CHOICE (owner-endorsed, CR-NR-100): Option A is
preferred specifically because it DECOUPLES help markdown rendering from the Task 21
viewer-to-shell seam. It needs only the one localised `help_context.rs` draw-call
change, carries NO hard Task 21 dependency, and keeps a clean path to switch to
Option C (ff-viewers ViewerPanel reuse) later if/when the viewer framework is wired
into the shell. That lower-coupling choice matches the owner's priority of finishing
the ff-desktop decoupling BEFORE wiring new subsystems into `ff-desktop`. Option A is
therefore the recommended-pending-owner-confirmation design direction; Option C is
recorded as the future-consolidation path, not a prerequisite.

Internal cross-reference links (`[text](topic_key)`) activate Help navigation
(Req 20.3) via the existing `HelpPanelModel::follow_link` / navigation stack; the
egui Markdown widget's link callback is wired to that, NOT to a web open. External
`http(s)` links default to display-only (not treated as Help topics); whether to
open them in the OS browser is deferred to the implementation decision and recorded
when Option A is confirmed. On any rendering error, fall back to the current
monospace `ui.label` of the raw body (Req 20.4).

### M5. Framework conformance -- do NOT add a parallel path

Per `framework-conformance`: HELP and F1 continue to route through the single
command-dispatch path (`resolve_target` / `dispatch_command_target`); the Help
Context remains a `WorkspaceContext` dispatched via `render_workspace_context`
(no second focus mechanism). Markdown rendering is a CHANGE TO HOW THE BODY IS
DRAWN inside that existing Context, not a new dispatch seam and not a new renderer
pipeline parallel to ff-viewers. If Option C is later chosen, help bodies go through
the ff-viewers PREVIEW integration (Req 14), reusing the framework's one viewer
path -- again not a bespoke parallel renderer.

### M6. Testing delta

- Unit test (where applicable): if a content-format detection or Markdown-to-egui
  mapping helper is introduced, unit-test it (e.g. a heading line maps to a heading
  element, a fenced block maps to a code element, an internal link maps to a Help
  navigation action, a malformed body falls back to monospace).
- Rendered-panel `egui_kittest` test (MANDATORY per testing.md GUI rule): a
  full-shell / Help-Context harness test asserting the Help Context shows FORMATTED
  output -- e.g. a `# Heading` line is rendered as a heading element (or otherwise
  NOT as the literal `#` character in a monospace block), proving Req 20.1/20.2 by
  behaviour, not by pixels. The existing full-shell first-Tab focus test
  (`full_shell_help_first_tab_focuses_search_field`) continues to pass, proving
  Req 20.6 (the focus contract survives the rendering change).
- Link-navigation tests (Req 20.9-20.14): unit tests for `classify_help_link`
  (external vs bare Topic_Key vs file vs cross-file-anchor vs same-file-anchor),
  `heading_slug`, and `resolve_fragment` (Topic_Key first, slug fallback, missing
  -> top-of-target); plus `egui_kittest` behaviour tests -- clicking a SAME-FILE
  anchor (`[x](#cmd:CHANGE)`) navigates within the current content and pushes the
  Help_Navigation_Stack, and clicking a CROSS-FILE anchor (`[x](other.help.md#key)`)
  loads the target then positions at the topic. These are harness-tested, not
  manual.
- MANUAL is reserved only for pixel-exact Markdown appearance (colour/spacing),
  which asserts the model/structure instead; link-activation and heading-rendering
  behaviour are harness-tested, not manual.

### M7. Link-target resolution (CR-NR-100 refinement -- Req 20.9-20.14)

Help links fall into FIVE target classes, resolved by the egui Markdown widget's
link callback (the renderer walks pulldown-cmark events and surfaces each link's
destination string; the Help Context owns the click -> navigate mapping, NOT a
web open). This is the exact seam that makes Option A (direct egui rendering)
sufficient: the renderer hands the destination to the Help Context, which
classifies and dispatches it onto the EXISTING `HelpPanelModel::follow_link` /
Help_Navigation_Stack. No new dispatch path, no second navigation stack.

EXTENSION DIRECTION (SETTLED, owner-confirmed): HTML-anchor-style navigation
(same-file `#anchor` and cross-file `other.md#anchor`) is delivered by extending
RESOLUTION semantics ONLY, NOT parse syntax. `[text](#anchor)`,
`[text](other.help.md)`, and `[text](other.help.md#anchor)` are ALREADY standard,
portable Markdown link forms, so `.help.md` files stay standard Markdown that any
other tool renders as ordinary links; only OUR resolver assigns navigation meaning
to the fragment (primarily a Topic_Key, matching the `<!-- TOPIC: topic_key -->`
markers of Req 5.2, with a heading-slug fallback). A custom-parse-syntax option
(a bespoke anchor marker) was considered and REJECTED for portability. No new
markup is introduced.

Classification of a link destination string `dest`:

1. **External** -- `dest` begins with `http://` or `https://` (Req 20.12). NOT a
   topic/file/anchor. Default policy: display-only / open in the OS browser per
   the policy recorded when Option A is confirmed (M4). Never routed to
   `follow_link`.
2. **Bare Topic_Key** -- `dest` has no leading `#`, no `.help.md` / `.md` filename
   component, and no scheme (e.g. `cmd:CHANGE`, `index`, `line:CC`) (Req 20.12 /
   20.3, UNCHANGED). Parsed with `TopicKey::parse` and passed to
   `HelpPanelModel::follow_link`.
3. **File link (no fragment)** -- `dest` ends in `.help.md` or `.md` with no
   `#fragment` (e.g. `setup.help.md`) (Req 20.9). Resolves to the file's PRIMARY
   topic -- the FIRST topic defined in that file by delimiter order (Requirement
   5.2). Because the registry is keyed by `Topic_Key`, not filename, resolution
   maps the filename to its topics via a filename -> topic-keys index built at
   load time (`ContentLoader` already records which `Topic_Key`s each file
   contributed -- `loaded_files: HashMap<PathBuf, Vec<TopicKey>>` in `tasks.md`
   Task 3.1); the first entry is the primary topic. The resolved `Topic_Key` goes
   through `follow_link`.
4. **Cross-file anchor** -- `dest` is `<file>.help.md#<fragment>` or
   `<file>.md#<fragment>` (e.g. `setup.help.md#config:theme`) (Req 20.10). The
   file part selects the content file; the fragment is resolved WITHIN that file
   as a Topic_Key FIRST (against the file's contributed topic-key set), falling
   back to a heading slug (class 6 below). Load/open the file THEN position at the
   resolved topic/heading, via `follow_link`.
5. **Same-file anchor** -- `dest` is `#<fragment>` with no filename and no scheme
   (e.g. `#cmd:CHANGE`) (Req 20.11). The fragment is resolved WITHIN the CURRENT
   topic's source file as a Topic_Key FIRST, falling back to a heading slug.
   Positions within the current content and pushes `follow_link`.

6. **Heading-slug fallback (Req 20.13)** -- for classes 4 and 5, a fragment that
   is NOT a known Topic_Key is matched against a GitHub-style auto-generated slug
   of the Markdown headings in the target content (lowercase, spaces -> hyphens,
   punctuation stripped), e.g. `#my-section` matches `## My Section`. The
   Topic_Key form is the PRIMARY help convention; the slug is the fallback.

Resolution helpers (pure, unit-testable without a harness; first red tests for
Task 22):
- `classify_help_link(dest: &str) -> HelpLinkTarget` (enum: `External(url)`,
  `TopicKey(key)`, `File { file, fragment: None }`, `File { file, fragment:
  Some(frag) }`, `SameFileAnchor(frag)`).
- `heading_slug(heading: &str) -> String` (GitHub-style slug) and a
  `resolve_fragment(fragment, file_topic_keys, headings) -> FragmentResolution`
  that tries Topic_Key then slug.

The filename -> primary/first-topic, filename+fragment -> topic, and
fragment -> slug resolution reuse the existing load-time file/topic index plus the
parsed heading list the content loader already extracts; no new registry
structure is required.

Unresolvable target (Req 20.14): a missing ANCHOR within an otherwise-resolved
target positions at the TOP of that target (its primary topic); a missing TOPIC
or FILE shows the existing "Help topic not found" affordance (Req 5.5) with a link
back to the Help_Index -- the same miss path Req 18.4 / 19 already use -- rather
than failing the display or navigating away.

Framework note: ALL classes route Help navigation through the ONE existing seam
(`follow_link` / Help_Navigation_Stack). The only new code is the pure classifier,
the slug helper, and the file/anchor -> topic lookup over the already-recorded
load-time index. This stays Option A (no viewer dependency, no Task 21 dependency
for the help-internal navigation) and preserves the WorkspaceContext/InteriorFocus
contract (Req 20.6).

---

## CR-CH-051 Delta: About dialog -- author credit + richer metadata (Requirement 14.6-14.8)

This delta expands the `AboutInfo` content model behind the "About FileForgeWorkbench"
Help_Menu action. It does NOT change the dispatch mechanism (the About action remains
a Help_Menu item invoking the HELP/menu command path, D5); it only enriches the DATA
the modal displays, per revised criterion 14.6 and new criteria 14.7-14.8.

### A1. AboutInfo field set (revised)

`AboutInfo` is a GUI-free data struct built once at display time from COMPILE-TIME
sources (no runtime I/O, nothing user-editable). Fields:

```rust
/// Metadata shown in the About FileForgeWorkbench dialog.
/// Addresses: context-help Requirement 14.6, 14.7, 14.8 (CR-CH-051)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutInfo {
    /// Application display name (e.g. "FileForgeWorkbench").
    pub app_name: String,
    /// Semantic version, from CARGO_PKG_VERSION.
    pub version: String,
    /// Build date, from a build-time env var (build script / vergen).
    pub build_date: String,
    /// Rust compiler version used for the build (build script).
    pub rust_version: String,
    /// Author / credit line, from CARGO_PKG_AUTHORS (Req 14.7).
    pub authors: String,
    /// Copyright notice (Req 14.7). Derived from authors + year, or a
    /// dedicated copyright metadata value if one is defined.
    pub copyright: String,
    /// Licence SPDX identifier or full name, from CARGO_PKG_LICENSE (Req 14.8).
    pub license: String,
    /// Optional reference to the full licence text (path/URL) (Req 14.8).
    pub license_reference: Option<String>,
    /// Optional project homepage or repository URL, from CARGO_PKG_HOMEPAGE
    /// or CARGO_PKG_REPOSITORY (Req 14.8).
    pub homepage: Option<String>,
}
```

### A2. Compile-time sources (no runtime I/O)

- `app_name`, `version`, `authors`, `license` come from Cargo package metadata via
  `env!("CARGO_PKG_NAME")` / `env!("CARGO_PKG_VERSION")` / `env!("CARGO_PKG_AUTHORS")`
  / `env!("CARGO_PKG_LICENSE")`. The crate's `Cargo.toml` SHALL carry `authors`,
  `license` (SPDX), and `homepage`/`repository` so these are populated; where the
  workspace sets them via `[workspace.package]`, the binary crate inherits them.
- `homepage`/`license_reference` come from `env!("CARGO_PKG_HOMEPAGE")` /
  `env!("CARGO_PKG_REPOSITORY")` (whichever is defined).
- `build_date` and `rust_version` come from build-time environment captured by a
  build script (the existing/added `vergen`-style build metadata already noted for
  the pre-existing version/build-date fields).
- `copyright` is composed at build time from the author(s) and the build year
  (e.g. `"(C) <year> <authors>"`), or taken from a dedicated metadata value if the
  project defines one. ASCII `(C)` is used, not a non-ASCII copyright glyph, per the
  documentation character-set rule.
- Per criterion 14.8, a field whose source metadata is ABSENT at build time
  (e.g. no homepage set) is modelled as `Option` and OMITTED from the rendered
  dialog rather than shown empty; the present fields still render.

### A3. Rendering and framework conformance

- The About modal reads an `AboutInfo` and renders the fields as labelled rows; the
  licence row shows the SPDX id/name plus the `license_reference` link when present,
  and the homepage row shows the URL when present. This is a modal dialog (as 14.6
  already specifies), not a Workspace Context, so the `WorkspaceContext`/
  `InteriorFocus` contract does not apply to it; it follows the existing modal-dialog
  conventions of the shell.
- The About action stays a Help_Menu item invoking the single command path (D5);
  no new dispatch path is introduced.

### A4. Testing delta

- Unit tests (pure): `AboutInfo` population from the compile-time values yields all
  mandatory fields non-empty (app_name, version, build_date, rust_version, authors,
  copyright, license); an absent optional source (homepage/license_reference) yields
  `None` and is therefore omittable (Req 14.8). Assert ASCII-only content (no
  non-ASCII copyright glyph).
- GUI behaviour (`egui_kittest`, per testing.md): opening the About dialog shows the
  author/credit line, the copyright notice, and the licence; a build with an absent
  optional field does not render an empty row for it. Pixel-exact appearance is the
  only MANUAL aspect; the presence/absence of each field is asserted on the model/
  rendered structure, not pixels.

---

# CR-NR-119 -- Root-Command Help Model (design)

This section designs the root-command help model: every home-grown FFCMD root
command is defined with a structured Markdown help file that is the SINGLE SOURCE
OF TRUTH for its one-line description and its full help, reachable by both
`HELP <cmd>` (ISPF) and `<cmd> --help` (POSIX). It covers context-help
Requirements 21-23, command-semantics Requirement 15, command-configurator
Requirement 9, and command-framework Requirement 3.8-3.9.

## Overview

The investigation (`.agents/tasks/alias-seed-set/root-command-descriptions.md`)
established that FFWB has NO single source of truth for a root command's
description: descriptions are scattered across the `CommandRegistry` (4 commands),
the `ff-file-ops` `file.*` table, optional user `CommandDefinition`s, and partial
`cmd:<NAME>` help topics, while the FFCMD root verbs carry none. The help
subsystem (`ff-help`), by contrast, ALREADY has the right shape: `.help.md` files
parsed by `ContentParser` into `HelpTopic`s keyed `cmd:<NAME>`, with per-command
sections Syntax / Description / Modifiers / Examples / See Also (context-help
Req 7.2), loaded once into a shell-owned `HelpTopicRegistry` and displayed via
`open_help`. The design therefore UNIFIES description sourcing onto the help file
and builds the two reach-paths (`HELP <cmd>`, `--help`) on the EXISTING single
dispatch front door. Nothing new is invented: the retrofit authors `cmd:<ROOT>`
topics for the FFCMD roots that lack them and adds (a) a pure one-line extraction
function, (b) a registry lookup, and (c) a prelude-level `--help` short-circuit.

## Central design decision -- the help-file structure and where the one-liner comes from

Three options were considered for making the one-line description deterministically
extractable:

- **(a) YAML frontmatter with an explicit `description:` field.** A dedicated field
  is unambiguous, but it adds a second parse path to `ContentParser` (which today
  recognises `<!-- TOPIC: -->` / `<!-- TITLE: -->` delimiters, not YAML), introduces
  a field that can DRIFT from the prose Description section, and changes the shape
  of every existing `.help.md`.
- **(b) A fixed first-line / first-paragraph-after-the-`# <CMD>`-heading
  convention.** No new field, but the shipped topics do NOT lead with an `# <CMD>`
  heading -- they lead with `## Syntax` then `## Description` (see
  `help/commands/session.help.md`). Adopting (b) would require reworking existing
  topics and is fragile against authors who reorder sections.
- **(c) Formalise the EXISTING `cmd:<NAME>` structure and extract the first
  paragraph of the `## Description` section.** The shipped topics already have a
  `## Description` section whose first paragraph is a natural one-liner ("Writes
  the current buffer to its backing file ...", "Closes the current Context ..."). No
  new markup, no second store, no change to the parser or renderer; the files stay
  standard Markdown.

**CHOSEN: Option (c).** It reuses what already ships, adds no parse syntax (so
`.help.md` files remain portable Markdown), keeps ONE description per command (the
prose), and matches context-help Req 7.2. Owner confirmation is requested on this
choice (it is the one genuinely load-bearing decision). The extraction rule
(context-help Req 21.3): take the `## Description` section body, take its first
non-empty paragraph, strip inline Markdown (``code``, `**bold**`, `[text](key)` ->
`text`) to plain text, collapse internal whitespace to single spaces, and trim.
Validate non-empty and `<= 200` chars (Req 21.4); a missing/empty Description is a
FAILURE surfaced by the enforcement check, never silently defaulted.

### Extraction API (ff-help)

Add to `ff-help` a pure function and a registry method:

```
// ff-help/src/one_liner.rs (new, < 100 lines incl. tests split out)
/// Extract the one-line description from a topic body per context-help Req 21.3.
/// Pure: depends only on the body string. Returns None if no non-empty
/// Description-section first paragraph exists.
pub fn extract_one_line_description(body: &str) -> Option<String>;
```

```
// ff-help/src/registry.rs (method on HelpTopicRegistry)
/// context-help Req 22.2: the single short-description lookup.
pub fn short_description(&self, key: &TopicKey) -> Option<String> {
    self.get(key).and_then(|t| extract_one_line_description(t.body()))
}
```

`extract_one_line_description` scans the body line-by-line for the `## Description`
heading (case-insensitive, any heading level that reads "Description"), collects
lines until the next `##`/`#` heading or EOF, and returns the first blank-line-
delimited paragraph cleaned as above. Locale-awareness (Req 21.6) is automatic: the
registry is already built per active locale (the shell loads per-locale help dirs
then the English base), so `short_description` reads whichever `cmd:<ROOT>` topic
is loaded for the active locale, falling back to English when only that exists.

## The `--help` dispatch hook (command-semantics Req 15)

The single front door is `WorkbenchShell::dispatch_command_string` (shell/dispatch.rs):
`=` strip -> `rewrite_verb_to_root` -> `run_command_prelude` -> `resolve_target` ->
`run_command_ladder`. The `--help` convention is a prelude-level short-circuit
placed AFTER `rewrite_verb_to_root` (so an alias is already canonicalised: `SS` ->
`SNAPSHOT`) and BEFORE `resolve_target`/the ladder (so the command never executes).

Concretely, add one guarded branch near the top of `run_command_prelude` (or as the
first check `dispatch_command_string` makes after the rewrite), reusing the existing
`split_verb_arg` helper:

```
let (verb, arg) = split_verb_arg(&raw);   // single-token verb (Req 15.9)
if first_token_is_help_flag(arg) {         // --help | -h | /?  (first arg token)
    self.open_root_help(verb);             // enforced-root-aware help open
    return true;                            // short-circuit: do not execute
}
```

`open_root_help(verb)` opens the Help Context on the command's topic through the
ONE help-display seam. For an enforced FFCMD root (membership tested against
`builtin_ffcmd_roots()`) it resolves `cmd:<VERB-UPPERCASED>` DIRECTLY, bypassing
`resolve_help_argument`'s keyword special-cases so `KEYS --help` -> `cmd:KEYS` and
`CONFIG --help` -> `cmd:CONFIG` (context-help Req 22.8) rather than the
`feature:` topics; for any other verb it falls back to the existing `open_help`
behaviour. The SAME `open_root_help` entry is used by the `HELP <ROOT>` path so
there is exactly one resolution of a root's help (Req 22.1 / 22.8). Thus
`EXIT --help`, `HELP EXIT`, and (after rewrite) `SS --help` / `HELP SS` all reach
`cmd:EXIT` / `cmd:SNAPSHOT`, and `KEYS --help` / `HELP KEYS` reach `cmd:KEYS` --
no second dispatcher, no bespoke scattered intercepts (framework-conformance
mechanism 1). Precedence is settled: `--help` SHORT-CIRCUITS before the command
runs (Req 15.2). The flag is recognised only as the FIRST argument token of a
single-token verb (Req 15.7, 15.9). Not recorded in history/undo (Req 15.8), the
same as HELP.

Recognised spellings: `--help`, `-h`, `/?` (letter forms case-insensitive). These
live in one small `first_token_is_help_flag` helper so the set is defined once.

### Known limitation -- two-word roots and the `--help` token split

`split_verb_arg` (`crates/ff-desktop/src/shell/helpers.rs`) splits on the FIRST
whitespace only, returning `(head, rest)`. For the two enforced TWO-WORD roots
`FILE CATALOGS` and `RESET BARE` this breaks the simple short-circuit above:

- `FILE CATALOGS --help` -> verb = `FILE`, arg = `CATALOGS --help`. The FIRST arg
  token is `CATALOGS`, not a help flag, so `first_token_is_help_flag(arg)` is
  FALSE: the short-circuit does not fire and the command would execute.
- Even if it fired, `open_help("FILE")` would resolve `cmd:FILE`, not
  `cmd:FILE CATALOGS`.

CHOSEN FIX (recommended option (a)): the `--help` convention applies to
SINGLE-TOKEN verbs in THIS version; `--help` for the two-word roots
`FILE CATALOGS` and `RESET BARE` is explicitly DEFERRED and recorded as a known
limitation (command-semantics Req 15.9). The ISPF reach-path is UNAFFECTED --
`HELP FILE CATALOGS` and `HELP RESET BARE` still work, because
`resolve_help_argument` uppercases and preserves the embedded space, resolving
`cmd:FILE CATALOGS` / `cmd:RESET BARE` (the enforcement at Req 23.2 asserts those
topics exist, and decision point 5 covers the `TopicKey` space). So both two-word
roots still have a reachable single source of truth via HELP; only their POSIX
`--help` spelling waits for a later version. The rejected alternative (option (b)
-- detect a help flag as the TRAILING token and derive the root from the leading
tokens for the known two-word roots) was declined for this version: it
complicates the otherwise one-line short-circuit and the two-word roots are a
closed, tiny set already covered by HELP. If `--help` for two-word roots is
wanted later, option (b) is the additive follow-up (no framework change).

## HELP <cmd> routing for FFCMD roots (context-help Req 22.1)

For MOST roots `HELP <cmd>` already resolves `cmd:<NAME>` via `open_help` and the
only work is CONTENT: author the missing `cmd:<ROOT>` topics so the resolution
lands on real content instead of the "No help available" miss path.

TWO enforced roots are an EXCEPTION and DO require a routing change. The existing
argument resolver `resolve_help_argument` (`crates/ff-help/src/commands.rs`)
hard-special-cases several surface strings BEFORE the `cmd:<NAME>` fall-through,
two of which are enforced FFCMD roots:

- `KEYS` -> `TopicKey::feature("function_keys")`
- `CONFIG` / `CONFIGURATION` -> `TopicKey::feature("configuration")`

(The other special-cased strings -- `OFF`, `INDEX`, `LINECOMMANDS`, `MACRO`,
`API` -- are NOT FFCMD roots, so they do not collide.) Because both the `HELP`
path and the `--help` path funnel through `resolve_help_argument`, today
`HELP KEYS` / `KEYS --help` resolve `feature:function_keys` and
`HELP CONFIG` / `CONFIG --help` resolve `feature:configuration` -- NEVER
`cmd:KEYS` / `cmd:CONFIG`. That directly contradicts the model's invariant that
every enforced root is reached, by both conventions, at its own `cmd:<ROOT>`
topic (Req 22.1, Req 15.4), and the enforcement (Req 23.2) that requires those
`cmd:` topics to be the reachable single source of truth.

CHOSEN FIX (recommended option (a)): FFCMD-root `cmd:<ROOT>` resolution takes
PRECEDENCE over the `feature:` keyword special-cases for `KEYS` and `CONFIG`. The
root-help reach-path (the `--help` short-circuit, and `HELP <ROOT>` for an
enforced root) resolves `cmd:<ROOT>` DIRECTLY -- `open_help` for an enforced root
constructs `TopicKey::command(<ROOT>)` rather than deferring to
`resolve_help_argument`'s keyword table. Concretely: the enumeration
`builtin_ffcmd_roots()` is the authority on which surface strings are enforced
roots; when the HELP/`--help` argument (after uppercasing) is in that set, bypass
the keyword table and go straight to `cmd:<ROOT>`. The rejected alternative
(option (b) -- leave `KEYS`/`CONFIG` pointing at their `feature:` topics and key
their short-description lookup to the feature topic) was declined because it
breaks the "one `cmd:` topic per root reached identically by both conventions"
invariant and complicates the Req 22.2 lookup with a per-root special case.

Note the `feature:function_keys` and `feature:configuration` topics are NOT
removed -- they remain reachable via the Help Index and the menus that link them
(context-help Req 15, Req 13.5/13.6); only the HELP/`--help` resolution of the
ENFORCED ROOT verbs `KEYS` and `CONFIG` is repointed to `cmd:KEYS` / `cmd:CONFIG`.
The retrofit therefore authors `cmd:KEYS` (a topic already ships under
`menu-options.help.md`, so KEYS needs only the resolution repoint) and `cmd:CONFIG`
(new) with conforming `## Description` sections, and those two topics MAY link to
their fuller `feature:` counterparts via See Also.

## Enforcement mechanism (context-help Req 23)

A test in `ff-desktop` (the crate that owns both the FFCMD root sets and the
`HelpTopicRegistry` wiring) enumerates `EnvironmentRegistry::root_commands("FFCMD")`
(= `builtin_ffcmd_roots()`), loads the shipped `help/` tree the same way
construct.rs does, and asserts for each root: a `cmd:<ROOT>` topic is present AND
`short_description` returns a non-empty string within limits. The test FAILS if any
enforced root lacks a conforming topic -- so adding a new FFCMD root without its
help file breaks the suite (Req 23.2/23.3). FFEDIT roots are excluded now
(Req 23.4). Extensions are never asserted (Req 23.5); the guideline is steering
only (`.kiro/steering/command-help-guideline.md`).

A small roadmap nuance: a handful of enforced roots are navigation/lifecycle verbs
(END, RETURN, CLOSE, DOCK, SPLIT, ...) that today have no `cmd:` topic. The retrofit
authors a `cmd:<ROOT>` topic for EVERY enforced root, not only EXIT/SNAPSHOT/
CAPTURE/RETRIEVE; see the retrofit list below. The enforcement test is added
AFTER the topics are authored so the suite goes green in the same change (TDD: the
test is written first and is RED until the topics exist).

## FFCMD root enumeration and retrofit list

The enforced set is `builtin_ffcmd_roots()` = `BUILTIN_FFCMD_LADDER_VERB_ROOTS` +
`BUILTIN_WORKSPACE_VERB_ROOTS` + the code-only built-in menu option roots. From
`command_config/mod.rs`:

- Ladder roots: EDIT, VIEW, BROWSE, COMPARE, CREATE, REPLACE, EXIT, QUIT, LOGOFF,
  START, MENU, CLOSE, DOCK, END, RETURN, DETACH, SPLIT, UNSPLIT, FOCUS, SWAP,
  WORKSPACE, PFSHOW, COMMAND, SNAPSHOT, CAPTURE, RETRIEVE, SUBMIT, TIME, STATUS,
  SAVE, CANCEL, REFRESH.
- Workspace-verb roots: FILE CATALOGS, CONFIG, KEYS, THEME, FILES, CATALOGS,
  COMMANDS, LOG, PLUGINS, MACROS, GSEARCH, SEARCH, KINDS, MENUS, ALIASES.
- Plus the built-in menu option roots unioned by `builtin_ffcmd_roots()` (e.g.
  HELP, X, RESET BARE, SETTINGS).

Coverage today (investigation + `help/commands/*.help.md`): topics EXIST for SAVE,
CANCEL, END, UNDO, REDO, HELP, KEYS, POM, SETTINGS, FILES and the navigation verbs
UP/DOWN/TOP/BOTTOM/LOCATE. Topics are MISSING for, among others: EXIT, SNAPSHOT,
CAPTURE (START/STOP/REPLAY), RETRIEVE, START, MENU, CLOSE, DOCK, RETURN, DETACH,
SPLIT, UNSPLIT, FOCUS, SWAP, WORKSPACE, PFSHOW, COMMAND, SUBMIT, TIME, STATUS,
REFRESH, EDIT, VIEW, BROWSE, COMPARE, CREATE, REPLACE, CONFIG, THEME, CATALOGS,
COMMANDS, LOG, PLUGINS, MACROS, GSEARCH, SEARCH, KINDS, MENUS, ALIASES. The retrofit
authors a `cmd:<ROOT>` topic (minimum: a non-empty `## Description`) for every
MISSING enforced root, grouped into a few new `help/commands/*.help.md` files
(e.g. `session-lifecycle.help.md` for EXIT/START/MENU/CLOSE/RETURN, `scrm.help.md`
for SNAPSHOT/CAPTURE/RETRIEVE, `workspace-verbs.help.md` for CONFIG/THEME/... ).
Two-word roots (FILE CATALOGS, RESET BARE) key as `cmd:FILE CATALOGS` /
`cmd:RESET BARE`; confirm `TopicKey` accepts a space (it stores the raw suffix) or
normalise to the resolved root the enumeration returns. These two roots are
reachable by `HELP <ROOT>` but their POSIX `--help` spelling is a deferred known
limitation (command-semantics Req 15.9; see "Known limitation -- two-word roots"
above).

## Unification of today's scattered description sources

- The 4 registered `CommandMetadata.description`s (file.open/exit, menu.open,
  config.open) and the `ff-file-ops` `file.*` table remain for the dotted
  Command_Ids they key, but the ROOT-verb description (what the alias editor /
  palette / status line show for a typed verb) now comes from the ONE
  `short_description(cmd:<ROOT>)` lookup. Over time the registry/table descriptions
  can be made to defer to the same lookup, but that is not required by this change
  (keeps blast radius small; command-framework Req 3.8 only mandates the DERIVED
  source for FFCMD roots).
- The alias Description column (command-configurator Req 7/9) and the `X -> EXIT`
  seed (Req 8) stop needing a literal: an empty row inherits `cmd:<ROOT>` at display
  time (Req 9.1/9.5). This directly resolves the owner's "reset-bare alias has no
  description" observation WITHOUT a throwaway literal on the seed row.

## Error handling (concrete, per operation)

- Missing `## Description` / empty first paragraph: `extract_one_line_description`
  returns `None`. Recoverable at the call site: the enforcement test treats it as
  a FAILURE (Req 21.4/23.2); the alias editor / palette treat it as "no inherited
  description" and show empty (Req 9.4/22.6). No panic, no log spam at display time
  (the enforcement test is the authoring-time signal).
- One-liner over 200 chars: extraction returns `None` (treated as a failure by the
  enforcement test, so the author shortens the first paragraph). Not a runtime
  error.
- `cmd:<ROOT>` topic absent (help dir missing, extension command, or an
  un-retrofitted root): `short_description` returns `None` and `open_help` shows
  the existing "No help available for: <cmd>" index-with-message (context-help
  Req 13.7). `--help` and `HELP` both degrade to this; neither errors as
  "unrecognised command" (Req 15.5/22.6). Recoverable, no fatal path.
- `--help` on a verb that does not resolve to any root: the rewrite leaves it
  unchanged, `open_help(verb)` resolves `cmd:<verb>`, misses, and shows the
  no-help affordance -- identical to typing `HELP <verb>` for an unknown verb.
- Extensions: the framework NEVER rejects an extension command for a missing help
  file (Req 23.5). Graceful degradation is the only behaviour; no registration-time
  validation of extension help.

## Input validation

- Help flag tokens: validated against the fixed set {`--help`, `-h`, `/?`} on the
  FIRST argument token only; letter forms case-insensitive; `/?` exact. Anything
  else is a normal argument and the command runs (Req 15.7).
- Topic key for a root: uppercased canonical root from the front door's verb token;
  spaces preserved for two-word roots. No user-supplied key reaches the extractor
  except through the resolved root.
- Extracted description: validated non-empty and length <= 200 before it is treated
  as valid (Req 21.4).

## Invariant ownership

- "One description per command" is owned by the help file (the prose Description)
  and enforced by the `ff-help` extractor + the `ff-desktop` enforcement test. No
  consumer stores its own copy (Req 22.2/22.3/9.3).
- "`--help` never executes the command" is owned by the front-door prelude
  short-circuit (the single place the branch lives), not by each command handler
  (Req 15.2/15.6).
- "Enforced for FFCMD, guideline for extensions" is owned by the enforcement test's
  SCOPE (it enumerates FFCMD roots only) and by steering (the extension guideline
  file), not by any code path that could block an extension (Req 23.4/23.5).

## Testability

- Unit (pure, `ff-help`): `extract_one_line_description` over fixture bodies --
  happy path (first Description paragraph), inline-markup stripping, missing
  Description -> None, over-length -> None, multi-paragraph (takes first only),
  locale body. `short_description` over a registry with/without the topic.
- Unit (`ff-desktop`): the `--help` short-circuit -- `EXIT --help` opens `cmd:EXIT`
  and does NOT fire the EXIT side effect (assert the close flag is NOT set);
  `SS --help` opens `cmd:SNAPSHOT` (alias canonicalised first); `-h` and `/?`
  spellings; a help flag as a non-first token does NOT short-circuit; a missing
  topic degrades to the no-help affordance.
- Unit (`ff-desktop`): the alias-description inheritance -- an empty-description row
  for `X -> EXIT` DISPLAYS the `cmd:EXIT` one-liner; a non-empty row keeps the user
  string; a row whose root has no topic shows empty.
- Unit (`ff-help`/`ff-desktop`): the enforced-root precedence -- `KEYS --help` /
  `HELP KEYS` resolve `cmd:KEYS` (not `feature:function_keys`), `CONFIG --help` /
  `HELP CONFIG` resolve `cmd:CONFIG` (not `feature:configuration`), while the
  `feature:*` topics remain reachable via the Index (Req 22.8).
- Unit (`ff-desktop`): the two-word-root limitation -- `FILE CATALOGS --help` and
  `RESET BARE --help` do NOT short-circuit (Req 15.9), while `HELP FILE CATALOGS`
  / `HELP RESET BARE` DO resolve their `cmd:` topics.
- Enforcement test (`ff-desktop`): every `root_commands("FFCMD")` entry has a
  conforming `cmd:<ROOT>` topic with a valid one-liner (RED until the retrofit
  topics are authored, GREEN after).
- No GUI-harness test is strictly required for the extraction/dispatch logic; the
  existing help-display egui_kittest coverage (Req 18.5) is unaffected.

## Framework verdict

NO framework change. The `--help` convention is a prelude-level short-circuit on
the EXISTING single front door (`dispatch_command_string`), reusing the EXISTING
`open_help` seam -- no new dispatcher, no new navigation stack, no new persistence
format (framework-conformance mechanisms 1 and 5 respected). The help-file
structure reuses the EXISTING `ff-help` topic format and registry. The enforcement
lives in a test, not in a new registration gate. Extensions are explicitly NOT
blocked. This design BUILDS ON the single dispatch front door and the context-help
subsystem.

## Review responses (revision 1 -- review.json verdict CHANGES_REQUESTED)

Both MEDIUM findings were verified against the live tree and ADDRESSED:

- **Finding 1 -- "No routing change needed" is false for KEYS and CONFIG.**
  Confirmed in `crates/ff-help/src/commands.rs`: `resolve_help_argument`
  special-cases `KEYS` -> `feature:function_keys` and `CONFIG`/`CONFIGURATION` ->
  `feature:configuration` before the `cmd:<NAME>` fall-through, and both are
  enforced FFCMD roots. FIX (recommended option (a)): the unqualified "no routing
  change is needed" claim is DELETED; the "HELP <cmd> routing" section now
  documents the divergence and specifies that enforced-root `cmd:<ROOT>`
  resolution takes PRECEDENCE over the `feature:` special-cases (the reach-path
  resolves `cmd:KEYS` / `cmd:CONFIG` directly). Added criterion context-help
  Req 22.8; updated the `--help` pseudocode to an enforced-root-aware
  `open_root_help`; added task 25.4 and the 26.2 note.
- **Finding 2 -- the `--help` short-circuit cannot serve two-word roots.**
  Confirmed in `crates/ff-desktop/src/shell/helpers.rs`: `split_verb_arg` splits
  on the first whitespace only, so `FILE CATALOGS --help` yields verb=`FILE`,
  arg=`CATALOGS --help` and the first arg token is not a help flag. FIX
  (recommended option (a)): the `--help` convention is scoped to single-token
  verbs this version; `FILE CATALOGS` / `RESET BARE` POSIX `--help` is explicitly
  DEFERRED as a known limitation, both still reachable via `HELP <ROOT>`. Added
  the "Known limitation -- two-word roots" subsection, criterion command-semantics
  Req 15.9, tasks 25.2/25.5, and the coverage mapping update.
