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
Wave 9 — Desktop Integration

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
│   Renders HelpPanel via DockablePanel::render; Markdown → UI  │
├──────────────────────────────────────────────────────────────┤
│         ff-help (THIS CRATE — Wave 9)                         │
│   Context detection, topic registry, navigation, search,      │
│   content loading, HELP command, Help Panel model              │
├──────────────────────────────────────────────────────────────┤
│  ff-command (Wave 2) — command dispatch, CommandMetadata,      │
│                         shortcut registry (F1 reserved)        │
│  ff-layout (Wave 2) — DockablePanel trait, dock zones          │
│  ff-config (Wave 2) — [help] config section, hot-reload        │
│  ff-plugin (Wave 2) — plugin lifecycle (topic registration)    │
│  ff-keys (Wave 9) — Key_Map, Shortcut_Registry (dynamic help) │
│  ff-core (Wave 2) — EventBus, VFS file-watcher                │
├──────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging (Wave 0)             │
└──────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: All help logic (context detection, topic resolution, search, navigation) is GUI-free; the Help Panel rendering is shell-side via `DockablePanel::render`
- **Command-Driven (Req 4)**: Operations registered as commands (`help.show`, `help.search`, `help.back`, `help.forward`, `help.index`, `help.close`)
- **Keyboard Shortcut Registry (Req 10)**: F1 is reserved (hard-coded, non-overridable) — always means Help
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
| `ff-desktop` | Renders Help Panel content (Markdown → styled text) via `DockablePanel::render` |
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
        CD[ContextDetector<br/>focus + command + mode → TopicKey]
        TR[HelpTopicRegistry<br/>TopicKey → HelpTopic, thread-safe]
        CL[ContentLoader<br/>.help.md file parser + indexer]
        HR[HotReloader<br/>VFS watcher → re-index]
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
| **Command Layer** | `HelpCommandHandler` — translates F1, HELP primary command, menu actions into help engine calls |
| **Context Layer** | `ContextDetector` — inspects focus, command input, prefix area, mode to resolve a `TopicKey` |
| **Registry Layer** | `HelpTopicRegistry` — thread-safe store of all topics; aggregates file-based, command-based, and plugin topics |
| **Content Layer** | `ContentLoader`, `HotReloader`, `DynamicGenerator` — load, parse, and generate help content |
| **Search Layer** | `SearchEngine` — case-insensitive keyword matching with relevance ranking |
| **Presentation Layer** | `HelpPanelModel`, `NavigationStack` — panel state, breadcrumb, TOC, navigation history |
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
│   ├── registry.rs             # HelpTopicRegistry — thread-safe indexed store
│   ├── context.rs              # ContextDetector — focus/command/mode → TopicKey
│   ├── loader.rs               # ContentLoader — .help.md file parser
│   ├── hot_reload.rs           # HotReloader — VFS watcher integration
│   ├── search.rs               # SearchEngine — keyword search + relevance ranking
│   ├── navigation.rs           # NavigationStack — back/forward/history
│   ├── panel.rs                # HelpPanelModel — panel state, breadcrumb, TOC
│   ├── dynamic.rs              # DynamicGenerator — function keys, config key topics
│   ├── config.rs               # HelpConfig — typed config access for [help] section
│   ├── commands/
│   │   ├── mod.rs              # Re-exports for all command handlers
│   │   ├── help_show.rs        # help.show — F1 / HELP <topic> handler
│   │   ├── help_search.rs      # help.search — search panel activation
│   │   ├── help_back.rs        # help.back — navigate back
│   │   ├── help_forward.rs     # help.forward — navigate forward
│   │   ├── help_index.rs       # help.index — jump to Help Index
│   │   └── help_close.rs       # help.close — close Help Panel (HELP OFF)
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
/// Addresses: Requirement 1 (1.2–1.7), Requirement 6 (6.1, 6.3)
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
/// A single unit of help content — one topic per command, line command, feature, or mode.
/// Addresses: Requirement 5 (5.1–5.6), Requirement 6 (6.1–6.7)
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
/// The GUI shell is responsible for rendering Markdown → styled text.
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
/// Addresses: Requirement 2 (2.1–2.10), Requirement 3 (3.1–3.6)
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
    /// Vertical scroll position within the current topic (0.0–1.0).
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
/// Addresses: Requirement 3 (3.1–3.6)
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
/// The top-level help index content — auto-generated from the registry.
/// Lists all topic categories with navigable links.
/// Addresses: Requirement 12 (12.1–12.4)
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
/// Addresses: Requirement 4 (4.1–4.5)
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

/// Where within a topic the search match was found — used for ranking.
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
/// Addresses: Requirement 1 (1.1–1.9)
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
/// Addresses: Requirement 16 (16.1–16.3)
#[derive(Debug, Clone, PartialEq)]
pub struct HelpConfig {
    /// Custom path to help content directory. None uses default search locations.
    pub directory: Option<String>,
    /// Help Panel width as a fraction of window width (0.2–0.5, default 0.35).
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
/// Addresses: Requirement 1 (1.1–1.9)
pub struct ContextDetector;

impl ContextDetector {
    /// Resolve the most relevant TopicKey from the given context state.
    ///
    /// Priority order:
    /// 1. Command input field focused + contains recognisable command → `cmd:<NAME>`
    /// 2. Command input field focused + empty → `index`
    /// 3. Prefix area focused + contains line command → `line:<CMD>`
    /// 4. Active special mode (Hex, Preview, Grid_*) → `mode:<MODE>`
    /// 5. Fallback → `index`
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
/// Addresses: Requirement 6 (6.1–6.7)
pub struct HelpTopicRegistry {
    /// TopicKey → HelpTopic mapping (RwLock for concurrent read access).
    topics: Arc<RwLock<HashMap<TopicKey, HelpTopic>>>,
    /// Alias → TopicKey mapping for alternative lookups.
    aliases: Arc<RwLock<HashMap<String, TopicKey>>>,
    /// Plugin ID → set of TopicKeys contributed by that plugin (for cleanup).
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
/// Addresses: Requirement 5 (5.1–5.6)
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
/// Addresses: Requirement 4 (4.1–4.5)
pub struct SearchEngine;

impl SearchEngine {
    /// Search all topics in the registry for the given query.
    /// Minimum query length is 2 characters; returns empty for shorter queries.
    /// Results are ranked: title matches > heading matches > body matches.
    pub fn search(registry: &HelpTopicRegistry, query: &str) -> Vec<SearchResult>;

    /// Case-insensitive substring match against a single topic.
    /// Returns the best MatchLocation or None if no match.
    fn match_topic(topic: &HelpTopic, query: &str) -> Option<(MatchLocation, String)>;

    /// Extract a context excerpt around the first match position (±40 chars).
    fn extract_excerpt(text: &str, query: &str) -> String;
}
```

### DynamicGenerator

```rust
/// Generates help topics dynamically at display time (not stored in registry).
/// Addresses: Requirement 15 (15.1–15.4)
pub struct DynamicGenerator;

impl DynamicGenerator {
    /// Generate the function keys help topic from the active key map.
    /// Produces a Markdown table: Key | Command | Label.
    pub fn generate_function_keys(
        key_map: &dyn KeyMapAccess,
        shortcut_registry: &dyn ShortcutRegistryAccess,
    ) -> HelpTopic;

    /// Generate the Help Index topic from the current registry state.
    /// Addresses: Requirement 12 (12.1–12.4)
    pub fn generate_index(
        registry: &HelpTopicRegistry,
        app_version: &str,
    ) -> HelpTopic;
}
```

### HelpPanelModel

```rust
/// The core model for the Help Panel — manages display state, navigation, search.
/// The GUI shell reads this model to render the panel content.
/// Addresses: Requirement 2 (2.1–2.10), Requirement 3 (3.1–3.6)
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
/// Addresses: Requirement 13 (13.1–13.10)
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
    /// - No args → show Help Index
    /// - `OFF` → close Help Panel
    /// - `LINECOMMANDS` → show `"line:index"` topic
    /// - `KEYS` → show dynamically generated function key topic
    /// - `MACRO` or `API` → show `"feature:macros"` topic
    /// - `CONFIG` or `CONFIGURATION` → show `"feature:configuration"` topic
    /// - `<name>` → try `"cmd:<NAME>"`, then `"feature:<name>"`, then unrecognised message
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
    #[error("[help] lookup: topic not found — {key}")]
    TopicNotFound { key: String },

    /// Help content directory not found at any search location.
    #[error("[help] content: help directory not found (searched: {searched_paths})")]
    DirectoryNotFound { searched_paths: String },

    /// No `.help.md` files found in the help directory.
    #[error("[help] content: no help files found in {directory}")]
    NoHelpFiles { directory: String },

    /// Failed to read a `.help.md` file.
    #[error("[help] content: failed to read {path} — {source}")]
    FileReadError { path: String, source: std::io::Error },

    /// Failed to parse a `.help.md` file (invalid topic delimiter format).
    #[error("[help] parse: invalid topic format in {path} at line {line} — {reason}")]
    ParseError { path: String, line: usize, reason: String },

    /// Navigation stack is empty — cannot go back.
    #[error("[help] navigation: no previous topic in history")]
    NoPreviousTopic,

    /// Navigation stack has no forward entry.
    #[error("[help] navigation: no next topic in history")]
    NoNextTopic,

    /// Configuration value is invalid (fallback applied, logged as warning).
    #[error("[help] config: invalid value for {key}, using default — {reason}")]
    InvalidConfig { key: String, reason: String },

    /// Help Panel is not open (close/navigate operation on closed panel).
    #[error("[help] panel: Help Panel is not open")]
    PanelNotOpen,

    /// Hot-reload file watcher registration failed.
    #[error("[help] reload: failed to watch {path} — {reason}")]
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
| `help.show` | F1 (reserved) | `commands::help_show` | Context-sensitive help — resolves topic from current state |
| `help.search` | *(none)* | `commands::help_search` | Activate search mode in Help Panel |
| `help.back` | Alt+Left | `commands::help_back` | Navigate to previous topic |
| `help.forward` | Alt+Right | `commands::help_forward` | Navigate to next topic |
| `help.index` | *(none)* | `commands::help_index` | Navigate to Help Index |
| `help.close` | Escape | `commands::help_close` | Close Help Panel (also via HELP OFF) |
| `help.command` | *(primary command)* | `HelpCommandHandler` | HELP primary command dispatcher |

Integration notes:
- F1 is registered as a **reserved shortcut** via `ShortcutRegistry::register_reserved()` — cannot be overridden by plugins or user key maps
- `help.show` and `help.command` do NOT produce undo records (`is_undoable() → false`)
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
| `help.panel_width_ratio` | `f32` | `0.35` | Help Panel width as fraction of window (0.2–0.5) |
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
- Plugin deregistration is automatic — the help system listens for plugin shutdown events via `EventBus`
- `HelpPluginBridge` is exposed to plugins via `PluginContext::help()` accessor

### Function Keys and History (`ff-keys`)

The help system uses a read-only accessor trait to query the active key map:

```rust
/// Trait for reading key map state — implemented by ff-keys, consumed by ff-help.
/// Decouples ff-help from ff-keys implementation details.
pub trait KeyMapAccess: Send + Sync {
    /// Returns all assigned function key bindings (F1–F24).
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

**Statement:** For any valid `ContextState`, `ContextDetector::resolve()` always returns a `TopicKey` — never panics, never returns an empty or malformed key. If no specific context is detected, it falls back to the Help Index key.

**Validates: Requirements 1.1, 1.3, 1.5, 1.7, 1.9**

**Strategy:** Generate arbitrary `ContextState` values with varied combinations of command input, prefix area, modes, and focus states. Assert that the returned TopicKey is non-empty and follows the `<namespace>:<id>` format or is the known index key.

---

### Property 2: Context Priority Order — Command Input Dominates

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

**Strategy:** Generate `.help.md` content with 1–10 randomly generated topic blocks using valid delimiter syntax. Parse and verify the count, keys, and titles match.

---

### Property 7: Help Panel Toggle Behaviour

**Statement:** If the Help Panel is open displaying topic T, and `toggle(T)` is called, the panel closes. If `toggle(U)` is called where U ≠ T, the panel navigates to U and remains open.

**Validates: Requirements 1.6, 2.4**

**Strategy:** Create a `HelpPanelModel` with a populated registry. Open a random topic. Call toggle with the same key — assert closed. Open again. Call toggle with a different key — assert open and displaying the new key.

---

### Property 8: Help Config Validation and Fallback

**Statement:** For any raw configuration values, `HelpConfig` construction with invalid values (panel_width_ratio outside 0.2–0.5, unrecognised panel_position) applies defaults and logs a warning, never panics or produces an unusable config.

**Validates: Requirements 16.1, 16.2**

**Strategy:** Generate arbitrary f32 values for width ratio (including NaN, infinity, negatives, values > 1.0) and arbitrary strings for position. Assert the resulting config always has width_ratio in [0.2, 0.5] and position is one of the three valid values.

---

### Property 9: HELP Command Routing Correctness

**Statement:** For any input string to the HELP command, `resolve_help_argument()` returns a deterministic, well-defined `HelpAction`. Empty maps to `ShowTopic(index)`, `"OFF"` maps to `Close`, `"LINECOMMANDS"` maps to `ShowTopic("line:index")`, `"KEYS"` maps to `ShowTopic("feature:function_keys")`, a known command name maps to `ShowTopic("cmd:<NAME>")`, and an unknown name maps to `UnrecognisedTopic(name)`.

**Validates: Requirements 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8**

**Strategy:** Generate the known dispatch keywords plus arbitrary strings. Assert routing matches the specification rules exactly.

---

### Property 10: Search Minimum Query Length Enforcement

**Statement:** For any query string with length < 2, `SearchEngine::search()` returns an empty result set. For any query with length ≥ 2, the result set contains only topics whose title, headings, body, or aliases contain the query as a case-insensitive substring.

**Validates: Requirements 4.1, 4.2, 4.5**

**Strategy:** Generate queries of length 0–1 and assert empty results. Generate queries of length 2+ with a known topic set and verify all results contain the query substring (case-insensitive) in at least one searchable field.

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
- **Integration tests**: `tests/integration.rs` for end-to-end scenarios (F1 press → context detection → registry lookup → panel state update)
- **Test doubles**: `MockKeyMapAccess` for function key generation; `MockConfigAccess` for config tests; in-memory `HelpTopicRegistry` with pre-loaded topics for all non-I/O tests
- **File I/O tests**: Use `tempfile::TempDir` with synthetic `.help.md` files for ContentLoader and HotReloader tests
- **No GUI testing in this crate**: All panel logic is testable via `HelpPanelModel` state assertions; actual Markdown rendering is the shell's responsibility

---

## Design Decisions and Rationale

1. **Markdown as help content format** — Chosen over plain text for richer formatting (code blocks, links, headings) while remaining easy to author and version-control. The GUI shell handles Markdown→styled rendering; the core crate stores raw Markdown.

2. **Topic delimiter syntax** (`<!-- TOPIC: key -->`) — Uses HTML comments rather than YAML front-matter to allow multiple topics per file without complex parsing. Valid Markdown that renderers ignore.

3. **Registry priority (runtime > file-based)** — Ensures dynamically registered commands always have up-to-date help without requiring file updates. File-based content serves as fallback documentation.

4. **Dynamic generation for function keys** — Generated at display time rather than cached because key maps can change at runtime (profile switches, hot-reload). Avoids stale content.

5. **GUI-free panel model** — `HelpPanelModel` contains all logic; the shell's `DockablePanel::render` merely reads state and draws. This allows unit testing all help behaviour without a GUI framework.

6. **Thread-safe registry with RwLock** — Allows concurrent reads from multiple threads (rendering, search) while serializing writes (plugin registration, hot-reload). Matches the workbench concurrency model.

7. **Navigation stack clears on close** — Per ISPF convention, each F1 press starts a fresh help session. Users do not accumulate unbounded history across multiple help invocations.
