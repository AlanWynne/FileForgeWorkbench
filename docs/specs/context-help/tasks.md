# Implementation Plan: Context-Sensitive Help System (`ff-help`)

## Overview

This plan covers the complete implementation of the `ff-help` crate — the context-sensitive help system for FileForgeWorkbench. The crate provides F1-triggered context detection, the dockable Help Panel model, a searchable Help Topic Registry populated from Markdown `.help.md` files and runtime command/plugin registrations, back/forward navigation, cross-reference linking, dynamic content generation (function key display), and the HELP primary command integration.

This is a **Wave 9 (Desktop Integration)** sub-project. It depends on `ff-command` (command framework — CommandMetadata, Shortcut_Registry, reserved shortcuts), `ff-layout` (layout-and-docking — DockablePanel trait, dock zones), `ff-plugin` (plugin-architecture — lifecycle hooks, topic registration/deregistration), `ff-config` (configuration-system — TOML `[help]` section, hot-reload), `ff-keys` (function-keys-and-history — Key_Map for dynamic help content), and `ff-logging` (logging subsystem).

---

## Tasks

- [ ] 1. Crate scaffolding and core types
  - [ ] 1.1 Create `crates/ff-help/Cargo.toml` with dependencies (ff-command, ff-layout, ff-plugin, ff-config, ff-keys, ff-logging, thiserror, serde, pulldown-cmark, proptest dev-dep)
  - [ ] 1.2 Create `crates/ff-help/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `topic.rs`, `topic_key.rs`, `registry.rs`, `content_loader.rs`, `content_parser.rs`, `context_detector.rs`, `help_panel.rs`, `navigation.rs`, `search.rs`, `plugin_help.rs`, `dynamic_content.rs`, `config.rs`, `commands.rs`, `error.rs`
  - [ ] 1.4 Add `ff-help` to workspace `Cargo.toml` members list
  - [ ] 1.5 Define `HelpError` enum with variants: TopicNotFound, ContentFileNotFound, ContentParseError, ContentDirectoryMissing, RegistryLockPoisoned, InvalidTopicKey, SearchQueryTooShort, NavigationStackEmpty, PluginTopicConflict, ConfigInvalid, HotReloadFailed
  - [ ] 1.6 Implement `Display` and `thiserror::Error` derives with descriptive messages for all error variants
  - Covers: Structural foundation for all requirements

- [ ] 2. Help topic model and Topic Key
  - [ ] 2.1 Define `TopicKey` newtype wrapping String with constructor, `Display`, `FromStr`, `Eq`, `Hash`, `Clone`, `Serialize`, `Deserialize`
  - [ ] 2.2 Implement `TopicKey` parsing with prefix validation — accept `"cmd:<name>"`, `"line:<name>"`, `"mode:<name>"`, `"feature:<name>"`, `"config:<key>"`, `"api:<name>"`, `"index"`, `"getting_started"`, `"line:index"`
  - [ ] 2.3 Implement `TopicKey::category() -> TopicCategory` — extract the prefix as an enum (Command, LineCommand, Mode, Feature, Config, Api, Index, GettingStarted)
  - [ ] 2.4 Define `TopicCategory` enum with variants matching all valid key prefixes
  - [ ] 2.5 Define `HelpTopic` struct with fields: key (TopicKey), title (String), body (String), syntax (Option<String>), aliases (Vec<String>), see_also (Vec<TopicKey>), source (TopicSource)
  - [ ] 2.6 Define `TopicSource` enum with variants: FileBased { file_path: PathBuf }, CommandRegistry { command_id: String }, Plugin { plugin_id: String }
  - [ ] 2.7 Implement `HelpTopic::cross_references() -> &[TopicKey]` — extract cross-reference links from body content
  - [ ] 2.8 Write unit tests for TopicKey parsing (valid prefixes, invalid format), category extraction, HelpTopic construction
  - Covers: Requirement 5 (AC 5.2), Requirement 6 (AC 6.1), Requirement 7 (AC 7.1)

- [ ] 3. Help content loading and parsing
  - [ ] 3.1 Implement `ContentLoader` struct with fields: search_paths (Vec<PathBuf>), loaded_files (HashMap<PathBuf, Vec<TopicKey>>)
  - [ ] 3.2 Implement `ContentLoader::resolve_help_directory() -> Option<PathBuf>` — search in order: (a) directory containing workbench binary, (b) User_Data_Dir, (c) custom path from `help_directory` config key
  - [ ] 3.3 Implement `ContentLoader::discover_files(dir: &Path) -> Vec<PathBuf>` — find all `.help.md` files recursively in the help directory
  - [ ] 3.4 Implement `ContentParser` struct for parsing `.help.md` file format
  - [ ] 3.5 Implement topic delimiter parsing — detect `<!-- TOPIC: topic_key -->` followed by `<!-- TITLE: Human Title -->` separating multiple topics within a single file
  - [ ] 3.6 Implement YAML front-matter parsing as alternative topic delimiter — extract `topic_key` and `title` fields from front-matter blocks
  - [ ] 3.7 Implement Markdown body parsing — extract section headings, bullet lists, inline code, fenced code blocks, bold text, and cross-reference links `[text](topic_key)`
  - [ ] 3.8 Implement `ContentLoader::load_all() -> Result<Vec<HelpTopic>, HelpError>` — load and parse all discovered files, index by TopicKey
  - [ ] 3.9 Implement missing directory handling — when help directory not found or contains no `.help.md` files, produce a built-in minimal help page explaining expected file locations
  - [ ] 3.10 Implement hot-reload detection — subscribe to VFS file-watcher events for the help directory, reload affected topics on file modification without restart
  - [ ] 3.11 Write unit tests for: directory resolution order, file discovery, delimiter parsing (HTML comment and YAML), Markdown element extraction, missing directory graceful handling, multi-topic file parsing
  - Covers: Requirement 5 (AC 5.1–5.7)

- [ ] 4. Context resolution engine
  - [ ] 4.1 Define `ContextDetector` struct with methods to inspect current editor state and resolve the most relevant TopicKey
  - [ ] 4.2 Define `EditorContext` struct capturing current state: focused_panel (PanelId), command_line_text (String), command_line_has_focus (bool), prefix_area_text (Option<String>), prefix_area_has_focus (bool), active_mode (EditorMode), active_line_command (Option<String>)
  - [ ] 4.3 Implement `ContextDetector::resolve(ctx: &EditorContext) -> TopicKey` — apply resolution priority rules to determine the best topic
  - [ ] 4.4 Implement command input resolution — when command field has focus and contains a recognisable command name (first whitespace-delimited token), resolve to `"cmd:<COMMAND_NAME>"`
  - [ ] 4.5 Implement empty command field resolution — when command field has focus and is empty or whitespace-only, resolve to `"index"`
  - [ ] 4.6 Implement prefix area resolution — when prefix area cell has focus and contains a recognisable line command, resolve to `"line:<COMMAND>"`
  - [ ] 4.7 Implement mode-based resolution — when no specific context is available, resolve to `"mode:<active_mode>"` for special modes (hex, preview, grid_edit, grid_browse)
  - [ ] 4.8 Implement fallback resolution — when no specific context can be determined, resolve to `"index"` (Help_Index)
  - [ ] 4.9 Implement resolution priority order: (1) focused command field with command name, (2) focused prefix area with line command, (3) active special mode, (4) Help_Index fallback
  - [ ] 4.10 Write unit tests for: command field with command, empty command field, prefix area with line command, mode-only context, no-context fallback, resolution priority when multiple contexts exist
  - Covers: Requirement 1 (AC 1.1–1.5, 1.7, 1.9)

- [ ] 5. Help Panel model
  - [ ] 5.1 Define `HelpPanelModel` struct with fields: current_topic (Option<HelpTopic>), is_open (bool), breadcrumb (Vec<BreadcrumbEntry>), scroll_offset (usize), toc_entries (Vec<TocEntry>), toc_visible (bool)
  - [ ] 5.2 Define `BreadcrumbEntry` struct with fields: label (String), topic_key (TopicKey)
  - [ ] 5.3 Define `TocEntry` struct with fields: heading (String), level (u8), anchor (String)
  - [ ] 5.4 Implement `HelpPanelModel::open(topic: HelpTopic)` — set current topic, compute breadcrumb path, extract TOC, set is_open to true
  - [ ] 5.5 Implement `HelpPanelModel::close()` — clear state, set is_open to false
  - [ ] 5.6 Implement breadcrumb computation — derive path from topic category hierarchy (e.g., `Help > Commands > CHANGE`)
  - [ ] 5.7 Implement TOC extraction — parse headings from topic body Markdown to build section outline
  - [ ] 5.8 Implement `HelpPanelModel::scroll_up()` / `scroll_down()` — adjust scroll offset within content bounds
  - [ ] 5.9 Implement toggle behaviour — when F1 resolves to the same topic currently displayed, close the panel
  - [ ] 5.10 Implement narrow-width detection — when panel width below 200px, set a flag for the UI to display resize suggestion
  - [ ] 5.11 Implement DockablePanel trait — provide panel_id, title, default_zone (Right), preferred_width_ratio (from config, default 0.35)
  - [ ] 5.12 Write unit tests for: open/close state transitions, breadcrumb derivation, TOC extraction from headings, toggle behaviour, scroll bounds clamping, narrow-width detection
  - Covers: Requirement 2 (AC 2.1–2.10)

- [ ] 6. Navigation — back, forward, and index
  - [ ] 6.1 Define `NavigationStack` struct with fields: history (Vec<TopicKey>), pointer (usize)
  - [ ] 6.2 Implement `NavigationStack::push(key: TopicKey)` — add topic to stack, truncate forward history if navigating from a non-head position
  - [ ] 6.3 Implement `NavigationStack::back() -> Option<TopicKey>` — move pointer backward, return previous topic; return None if at beginning
  - [ ] 6.4 Implement `NavigationStack::forward() -> Option<TopicKey>` — move pointer forward, return next topic; return None if at head
  - [ ] 6.5 Implement `NavigationStack::can_go_back() -> bool` and `can_go_forward() -> bool` — for UI button enable/disable state
  - [ ] 6.6 Implement `NavigationStack::current() -> Option<&TopicKey>` — return the topic at the current pointer position
  - [ ] 6.7 Implement `NavigationStack::clear()` — reset stack when Help Panel is closed and reopened (fresh session per Req 3.6)
  - [ ] 6.8 Implement `NavigationStack::go_to_index()` — navigate to Help_Index topic regardless of current position, push onto stack
  - [ ] 6.9 Implement cross-reference link handling — when user activates a `[text](topic_key)` link, push linked topic onto navigation stack
  - [ ] 6.10 Write unit tests for: push/back/forward sequences, truncation on branch, clear on reopen, go_to_index from mid-stack, can_go_back/forward boundary conditions
  - Covers: Requirement 3 (AC 3.1–3.6)

- [ ] 7. Search and filter
  - [ ] 7.1 Define `HelpSearch` struct with fields: index (SearchIndex), min_query_length (usize, default 2)
  - [ ] 7.2 Define `SearchIndex` struct — inverted index mapping keywords to TopicKeys for efficient lookup
  - [ ] 7.3 Implement `SearchIndex::build(topics: &[HelpTopic])` — build keyword index from topic titles, body text, and TopicKey aliases
  - [ ] 7.4 Implement `HelpSearch::query(text: &str) -> Vec<SearchResult>` — case-insensitive substring matching across titles, body, and aliases; minimum 2-character query
  - [ ] 7.5 Define `SearchResult` struct with fields: topic_key (TopicKey), title (String), excerpt (String), relevance_score (u32)
  - [ ] 7.6 Implement relevance ranking — exact title match (score 100), keyword in heading (score 50), keyword in body (score 10); sort results by descending score
  - [ ] 7.7 Implement excerpt generation — extract the sentence or line containing the first keyword match for display in results list
  - [ ] 7.8 Implement no-results handling — when query produces zero results, return empty Vec (UI displays "No help topics found" message)
  - [ ] 7.9 Implement incremental index update — when topics are added/removed (hot-reload, plugin changes), update the SearchIndex without full rebuild
  - [ ] 7.10 Write unit tests for: query below minimum length rejected, exact title match ranked first, case-insensitive matching, no-results for unmatched query, excerpt extraction, relevance ordering, incremental update after topic add/remove
  - Covers: Requirement 4 (AC 4.1–4.5)

- [ ] 8. Help Topic Registry
  - [ ] 8.1 Define `HelpTopicRegistry` struct with fields: topics (HashMap<TopicKey, HelpTopic>), lock (RwLock for thread-safety)
  - [ ] 8.2 Implement `HelpTopicRegistry::new()` — create empty registry
  - [ ] 8.3 Implement `HelpTopicRegistry::register(topic: HelpTopic)` — insert topic by key; if key exists, apply priority rules (runtime > file-based)
  - [ ] 8.4 Implement `HelpTopicRegistry::unregister(key: &TopicKey)` — remove topic by key
  - [ ] 8.5 Implement `HelpTopicRegistry::get(key: &TopicKey) -> Option<HelpTopic>` — O(1) lookup by TopicKey
  - [ ] 8.6 Implement `HelpTopicRegistry::contains(key: &TopicKey) -> bool` — existence check
  - [ ] 8.7 Implement priority resolution — runtime-registered help (CommandRegistry, plugins) preferred over file-based content for same TopicKey
  - [ ] 8.8 Implement `HelpTopicRegistry::register_from_command_metadata(cmd_id: &str, help_text: &str, help_syntax: &str)` — create and register topic from CommandMetadata fields
  - [ ] 8.9 Implement fallback on empty help_text — when command registered with empty help_text, fall back to file-based content for `"cmd:<command_id>"`
  - [ ] 8.10 Implement `HelpTopicRegistry::all_topics() -> Vec<&HelpTopic>` — iterate all registered topics for search index building
  - [ ] 8.11 Implement `HelpTopicRegistry::topics_by_category(cat: TopicCategory) -> Vec<&HelpTopic>` — filtered iteration for Help Index category display
  - [ ] 8.12 Implement thread-safety — wrap internal HashMap with `RwLock<HashMap<TopicKey, HelpTopic>>` for concurrent read/write access
  - [ ] 8.13 Write unit tests for: register/unregister, O(1) lookup, priority override (runtime > file), fallback on empty help_text, category filtering, thread-safe concurrent access
  - Covers: Requirement 6 (AC 6.1–6.7)

- [ ] 9. Plugin-contributed help registration
  - [ ] 9.1 Define `PluginHelpProvider` trait with methods: `register_topics(&self, registry: &mut HelpTopicRegistry)`, `plugin_id(&self) -> &str`
  - [ ] 9.2 Implement plugin lifecycle integration — register plugin-contributed topics during plugin `initialize` phase
  - [ ] 9.3 Implement plugin unload cleanup — remove all topics contributed by a specific plugin_id during plugin `shutdown` phase
  - [ ] 9.4 Implement `HelpTopicRegistry::register_plugin_topic(plugin_id: &str, topic: HelpTopic)` — register with TopicSource::Plugin tracking
  - [ ] 9.5 Implement `HelpTopicRegistry::unregister_plugin_topics(plugin_id: &str)` — bulk remove all topics from a specific plugin
  - [ ] 9.6 Implement conflict resolution — when plugin topic conflicts with existing file-based topic, plugin wins (runtime priority)
  - [ ] 9.7 Write unit tests for: plugin topic registration, unload removes all plugin topics, conflict resolution with file-based topics, multiple plugins contributing non-overlapping topics
  - Covers: Requirement 6 (AC 6.1, 6.4, 6.6)

- [ ] 10. Dynamic content generation
  - [ ] 10.1 Define `DynamicContentGenerator` trait with method `generate(registry: &HelpTopicRegistry) -> HelpTopic`
  - [ ] 10.2 Implement `FunctionKeyHelpGenerator` — generate the `"feature:function_keys"` topic dynamically from active Shortcut_Registry and Key_Map
  - [ ] 10.3 Implement function key table generation — produce Markdown table with columns: Key, Command, Label for all assigned keys F1–F24
  - [ ] 10.4 Implement profile-aware generation — when Profile_Key_Map is active, show profile key map and note which profile is active
  - [ ] 10.5 Implement empty key map handling — when no keys assigned, display configuration guidance message
  - [ ] 10.6 Implement `HelpIndexGenerator` — generate the `"index"` topic dynamically from all registered topics, organised by category
  - [ ] 10.7 Implement Help_Index category sections — Getting Started, Primary Commands (alphabetical), Line Commands (compact table), Modes, Features, Configuration, Function Keys, Macro API
  - [ ] 10.8 Implement Help_Index footer — display workbench application name and version at bottom
  - [ ] 10.9 Write unit tests for: function key table generation with various key maps, profile-active annotation, empty key map message, index category organisation, alphabetical command listing
  - Covers: Requirement 12 (AC 12.1–12.4), Requirement 15 (AC 15.1–15.4)

- [ ] 11. Configuration
  - [ ] 11.1 Define `HelpConfig` struct with fields: directory (Option<PathBuf>), panel_width_ratio (f32, default 0.35), panel_position (DockPosition, default Right), search_highlight (bool, default true)
  - [ ] 11.2 Implement `Default` for `HelpConfig` — panel_width_ratio=0.35, panel_position=Right, search_highlight=true, directory=None
  - [ ] 11.3 Define `DockPosition` enum with variants: Right, Left, Bottom
  - [ ] 11.4 Implement configuration key registration for `[help]` TOML section: `directory`, `panel_width_ratio`, `panel_position`, `search_highlight`
  - [ ] 11.5 Implement validation for `panel_width_ratio` — reject values outside 0.2–0.5 range, emit WARN log, apply default 0.35
  - [ ] 11.6 Implement validation for `panel_position` — reject unrecognised values, emit WARN log, apply default "right"
  - [ ] 11.7 Implement hot-reload listener — subscribe to configuration-system change events for `[help]` section keys, apply new values without restart
  - [ ] 11.8 Implement hot-reload effect propagation — notify HelpPanelModel of width/position changes, notify ContentLoader of directory changes
  - [ ] 11.9 Write unit tests for: default values, valid config parsing, panel_width_ratio validation (out-of-range), panel_position validation (invalid string), hot-reload updates applied
  - Covers: Requirement 16 (AC 16.1–16.3)

- [ ] 12. Command registration — HELP command and F1 activation
  - [ ] 12.1 Register HELP as a primary command in the command framework with command_id "HELP", aliases: none
  - [ ] 12.2 Implement HELP command handler — parse arguments, resolve topic, open Help Panel
  - [ ] 12.3 Implement `HELP` (no arguments) — open Help_Panel displaying Help_Index
  - [ ] 12.4 Implement `HELP <command_name>` — resolve to `"cmd:<COMMAND_NAME>"` and display that topic
  - [ ] 12.5 Implement `HELP LINECOMMANDS` — display line command summary topic `"line:index"`
  - [ ] 12.6 Implement `HELP MACRO` / `HELP API` — display macro API reference topic `"feature:macros"`
  - [ ] 12.7 Implement `HELP KEYS` — display dynamically generated function keys topic `"feature:function_keys"`
  - [ ] 12.8 Implement `HELP CONFIG` / `HELP CONFIGURATION` — display configuration overview topic `"feature:configuration"`
  - [ ] 12.9 Implement `HELP OFF` — close the Help Panel if open
  - [ ] 12.10 Implement unrecognised topic handling — display Help_Index with message "No help available for: <topic>"
  - [ ] 12.11 Implement F1 key binding registration — register F1 as reserved shortcut (non-overridable) in Shortcut_Registry
  - [ ] 12.12 Implement F1 handler — invoke ContextDetector, resolve topic, open/toggle Help Panel
  - [ ] 12.13 Implement toggle behaviour — if Help Panel open and F1 resolves to same topic, close panel; if different topic, navigate to new topic
  - [ ] 12.14 Implement history exclusion — HELP command and F1 presses not added to command history, not recorded as undoable transactions
  - [ ] 12.15 Implement mode validity — HELP command valid in Browse, Edit, View, Hex, Preview, and all FileForge special modes
  - [ ] 12.16 Write unit tests for: HELP no-args opens index, HELP CHANGE opens cmd:CHANGE, HELP LINECOMMANDS opens line:index, HELP OFF closes panel, unrecognised topic shows index with message, F1 toggle behaviour, history exclusion, reserved shortcut non-overridable
  - Covers: Requirement 1 (AC 1.1, 1.6, 1.8, 1.10), Requirement 13 (AC 13.1–13.10)

- [ ] 13. Help Menu integration
  - [ ] 13.1 Define `HelpMenuModel` struct providing menu item definitions for the Help menu bar entry
  - [ ] 13.2 Implement menu items: Help Index, Command Reference, Line Command Reference, Key Bindings, separator, About FileForgeWorkbench
  - [ ] 13.3 Implement "Help Index" action — open Help Panel with Help_Index topic
  - [ ] 13.4 Implement "Command Reference" action — open Help Panel displaying primary commands category
  - [ ] 13.5 Implement "Line Command Reference" action — open Help Panel displaying line command summary
  - [ ] 13.6 Implement "Key Bindings" action — open Help Panel with function key/shortcut reference
  - [ ] 13.7 Implement "About FileForgeWorkbench" action — produce AboutInfo struct with application name, version, build date, Rust compiler version, license
  - [ ] 13.8 Define `AboutInfo` struct with fields: app_name, version, build_date, rust_version, license
  - [ ] 13.9 Write unit tests for: menu item list completeness, each action dispatches correct topic, AboutInfo population
  - Covers: Requirement 14 (AC 14.1–14.6)

- [ ] 14. Property-based tests
  - [ ] 14.1 Write PBT: Context resolution determinism property
  - [ ] 14.2 Write PBT: Topic Registry priority resolution invariant
  - [ ] 14.3 Write PBT: Navigation stack back/forward consistency property
  - [ ] 14.4 Write PBT: Search relevance ranking monotonicity property
  - [ ] 14.5 Write PBT: Content parser round-trip fidelity property
  - [ ] 14.6 Write PBT: TopicKey parsing totality property
  - [ ] 14.7 Write PBT: Help Panel toggle idempotency property
  - [ ] 14.8 Write PBT: Plugin registration/unregistration symmetry property
  - [ ] 14.9 Write PBT: Configuration validation boundary property
  - [ ] 14.10 Write PBT: Search index incremental update equivalence property
  - Covers: All requirements (property-based validation)

- [ ] 15. Integration tests
  - [ ] 15.1 Write integration test: F1 with command in command field resolves and displays correct topic end-to-end
  - [ ] 15.2 Write integration test: F1 with empty command field opens Help Index
  - [ ] 15.3 Write integration test: F1 with line command in prefix area displays line command help
  - [ ] 15.4 Write integration test: HELP CHANGE command opens Help Panel with cmd:CHANGE topic
  - [ ] 15.5 Write integration test: HELP OFF closes an open Help Panel
  - [ ] 15.6 Write integration test: navigation back/forward across multiple topic visits
  - [ ] 15.7 Write integration test: search query returns ranked results and navigation to selected result
  - [ ] 15.8 Write integration test: plugin registers topics during initialize, topics available via F1, topics removed on plugin shutdown
  - [ ] 15.9 Write integration test: hot-reload of .help.md file updates topic content without restart
  - [ ] 15.10 Write integration test: configuration hot-reload changes panel position and width ratio
  - [ ] 15.11 Write integration test: command registered with help_text creates topic accessible via HELP command
  - [ ] 15.12 Write integration test: Help Panel toggle — F1 same topic closes, F1 different topic navigates
  - Covers: Cross-requirement interaction validation

---

## Property-Based Test Definitions

### Property 1: Context Resolution Determinism

**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.7**

- **Statement:** For any given EditorContext state, the ContextDetector always resolves to the same TopicKey. The resolution is a pure function of the input context with no hidden state or non-determinism.
- **Strategy:** Generate:
  - EditorContext: random combination of command_line_text (empty, whitespace, valid command name, partial text), prefix_area_text (None, valid line command, invalid text), active_mode (Browse, Edit, View, Hex, Preview, Grid_Browse, Grid_Edit), focus flags (command_line_has_focus, prefix_area_has_focus, neither)
- **Invariant:** `resolve(ctx) == resolve(ctx)` for all generated contexts. Additionally, the resolved TopicKey always matches the highest-priority applicable rule: command field focus with command > prefix area focus with line command > special mode > index fallback.

### Property 2: Topic Registry Priority Resolution Invariant

**Validates: Requirements 6.4, 6.5**

- **Statement:** When both a file-based topic and a runtime-registered topic (from CommandRegistry or plugin) exist for the same TopicKey, the registry always returns the runtime-registered version. When the runtime registration is removed, the file-based content becomes visible again.
- **Strategy:** Generate:
  - TopicKey: random valid key from all prefix categories
  - File-based topic: random body content (10–500 chars)
  - Runtime topic: different random body content (10–500 chars)
  - Operation sequence: register file-based, then register runtime, then optionally unregister runtime
- **Invariant:** (1) After both registered: `registry.get(key).source == Runtime`. (2) After runtime unregistered: `registry.get(key).source == FileBased` and body matches original file-based content. (3) Runtime topic is never lost — it is always preferred while registered.

### Property 3: Navigation Stack Back/Forward Consistency

**Validates: Requirements 3.1, 3.2, 3.3**

- **Statement:** For any sequence of push, back, and forward operations on the NavigationStack, the stack maintains these invariants: (1) back() followed by forward() returns to the same topic, (2) push after back truncates forward history, (3) the current topic always reflects the pointer position.
- **Strategy:** Generate:
  - Topic sequence: 5–50 random TopicKeys
  - Operation sequence: 20–200 interleaved push/back/forward operations
- **Invariant:** After every operation: (1) `can_go_back()` iff pointer > 0. (2) `can_go_forward()` iff pointer < history.len() - 1. (3) If `back()` returns Some(k), then immediate `forward()` returns the topic we just left. (4) After `push(new)` when pointer is not at head, forward history is discarded.

### Property 4: Search Relevance Ranking Monotonicity

**Validates: Requirements 4.2, 4.4**

- **Statement:** For any search query, if a topic has an exact title match it always ranks higher than a topic with only a body match. Topics with heading matches always rank between title matches and body-only matches. The ranking is a total order — no two results with different match types have inverted relative positions.
- **Strategy:** Generate:
  - Query string: random 2–20 character substring
  - Topic set: 10–100 topics, some with query in title, some in headings, some in body only, some with no match
  - Execute search
- **Invariant:** For all pairs (a, b) in results: if a.match_type is Title and b.match_type is Body, then a.relevance_score > b.relevance_score. If a.match_type is Heading and b.match_type is Body, then a.relevance_score > b.relevance_score. Non-matching topics never appear in results.

### Property 5: Content Parser Round-Trip Fidelity

**Validates: Requirements 5.2, 5.3, 5.4**

- **Statement:** For any valid `.help.md` content containing N topic delimiter blocks, parsing always produces exactly N HelpTopic objects, each with the correct TopicKey and title as specified in the delimiter, and with body content containing all Markdown elements from between delimiters.
- **Strategy:** Generate:
  - Topic count: 1–10 per file
  - Per topic: random topic_key (valid format), random title (1–100 chars), random Markdown body (headings, lists, code blocks, bold, links — 1–50 elements)
  - Serialise to `.help.md` format using `<!-- TOPIC: key -->` / `<!-- TITLE: title -->` delimiters
  - Parse back
- **Invariant:** `parse(serialize(topics)).len() == topics.len()`. For each parsed topic: `parsed[i].key == topics[i].key` and `parsed[i].title == topics[i].title`. Body content round-trips without loss of semantic structure.

### Property 6: TopicKey Parsing Totality

**Validates: Requirements 5.2, 6.1**

- **Statement:** TopicKey::from_str accepts all strings matching the pattern `"<valid_prefix>:<non_empty_name>"` plus the special keys `"index"` and `"getting_started"`. All other strings are rejected with InvalidTopicKey error. No valid input panics; no invalid input is silently accepted.
- **Strategy:** Generate:
  - Valid inputs: random selection from valid prefixes (cmd, line, mode, feature, config, api) + ":" + random non-empty alphanumeric string; also "index" and "getting_started"
  - Invalid inputs: empty string, missing colon, unknown prefix + ":" + name, valid prefix with empty name, strings with whitespace/special characters in prefix position
- **Invariant:** All valid inputs produce Ok(TopicKey). All invalid inputs produce Err(InvalidTopicKey). No panic occurs for any input.

### Property 7: Help Panel Toggle Idempotency

**Validates: Requirements 1.6, 2.4**

- **Statement:** When the Help Panel is open showing topic T, pressing F1 with context resolving to T closes the panel. Pressing F1 again (same context) reopens it showing T. This toggle cycle is idempotent: N consecutive F1 presses with same context alternate between open and closed states perfectly.
- **Strategy:** Generate:
  - Initial topic: random valid TopicKey
  - F1 press count: 2–20 consecutive presses with same resolved context
- **Invariant:** After press k: panel is open if k is odd, closed if k is even (1-indexed). The displayed topic when open is always T. No state corruption accumulates across toggles.

### Property 8: Plugin Registration/Unregistration Symmetry

**Validates: Requirements 6.1, 6.6**

- **Statement:** For any plugin that registers N topics during initialize, unregistering that plugin removes exactly those N topics and no others. The registry state after register-then-unregister is identical to the state before registration (for keys that had no prior file-based content) or reverts to the file-based version (for keys that did).
- **Strategy:** Generate:
  - Pre-existing file-based topics: 5–30 random topics
  - Plugin topics: 3–15 random topics, some with keys overlapping file-based topics, some unique
  - Sequence: register plugin topics, then unregister plugin
- **Invariant:** After unregister: (1) topics with keys unique to the plugin are completely removed (`registry.contains(key) == false`). (2) Topics with keys that had file-based counterparts revert to the file-based version (`registry.get(key).source == FileBased`). (3) All other topics are unchanged.

### Property 9: Configuration Validation Boundary

**Validates: Requirements 16.1, 16.2**

- **Statement:** For any `panel_width_ratio` value, values in [0.2, 0.5] are accepted as-is; values outside this range are rejected and the default (0.35) is applied. For `panel_position`, only "right", "left", "bottom" are accepted; all other strings result in default "right". No configuration value causes a panic or undefined behaviour.
- **Strategy:** Generate:
  - panel_width_ratio: f32 values in range [-1.0, 2.0] (including exact boundaries 0.2 and 0.5)
  - panel_position: random strings including valid values, empty, random unicode, numeric strings
- **Invariant:** (1) For ratio in [0.2, 0.5]: applied value == input. For ratio outside: applied value == 0.35 and warning logged. (2) For position in {"right", "left", "bottom"}: applied value == input. For others: applied value == "right" and warning logged.

### Property 10: Search Index Incremental Update Equivalence

**Validates: Requirements 4.1, 5.7**

- **Statement:** An incrementally-updated search index (via topic add/remove after initial build) produces identical query results to a freshly-built index from the same final topic set. Hot-reloading or plugin changes never cause the search index to diverge from a full rebuild.
- **Strategy:** Generate:
  - Initial topic set: 10–50 random topics
  - Mutations: 5–20 random add/remove operations
  - Query set: 5–15 random search queries
- **Invariant:** For each query: `incremental_index.query(q) == fresh_index.query(q)` where fresh_index is built from scratch on the final topic set. Result sets are identical in content and ordering.

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Help Topic Model", "tasks": ["2"], "dependsOn": [0] },
    { "id": 2, "label": "Content Loading and Parsing", "tasks": ["3"], "dependsOn": [1] },
    { "id": 3, "label": "Context Resolution Engine", "tasks": ["4"], "dependsOn": [1] },
    { "id": 4, "label": "Help Panel Model", "tasks": ["5"], "dependsOn": [1] },
    { "id": 5, "label": "Navigation", "tasks": ["6"], "dependsOn": [4] },
    { "id": 6, "label": "Search and Filter", "tasks": ["7"], "dependsOn": [1, 2] },
    { "id": 7, "label": "Help Topic Registry", "tasks": ["8"], "dependsOn": [1, 2] },
    { "id": 8, "label": "Plugin Help Registration", "tasks": ["9"], "dependsOn": [7] },
    { "id": 9, "label": "Dynamic Content Generation", "tasks": ["10"], "dependsOn": [7] },
    { "id": 10, "label": "Configuration", "tasks": ["11"], "dependsOn": [0, 4] },
    { "id": 11, "label": "Command Registration", "tasks": ["12"], "dependsOn": [3, 4, 5, 7] },
    { "id": 12, "label": "Help Menu Integration", "tasks": ["13"], "dependsOn": [11] },
    { "id": 13, "label": "Property-Based Tests", "tasks": ["14"], "dependsOn": [2, 3, 4, 5, 6, 7, 8, 9, 10, 11] },
    { "id": 14, "label": "Integration Tests", "tasks": ["15"], "dependsOn": [11, 12, 13] }
  ]
}
```

---

## Notes

- This is a Wave 9 (Desktop Integration) crate depending on `ff-command` (Wave 2), `ff-layout` (Wave 2), `ff-plugin` (Wave 2), `ff-config` (Wave 2), `ff-keys` (Wave 9), and `ff-logging` (Wave 0)
- The Help Panel is a data model only in this crate — actual rendering lives in the GUI shell (FFW-ARCH-001 principle: GUI-independent platform-core)
- The Help_Topic_Registry uses `RwLock<HashMap<TopicKey, HelpTopic>>` for thread-safe concurrent access without external lock acquisition by callers
- Context detection is best-effort — the system never fails to produce a TopicKey (worst case: Help_Index fallback)
- F1 is registered as a reserved shortcut per `command-framework` Requirement 5.3 — it cannot be overridden by user key maps, plugins, or language profiles
- The HELP command and F1 presses are excluded from command history and undo transactions — they are pure read-only navigational actions
- Help content uses `.help.md` extension (not plain `.md`) to distinguish from project documentation files in the same directories
- Topic delimiter format `<!-- TOPIC: key -->` is chosen for easy grep-ability and compatibility with standard Markdown renderers (renders as HTML comment)
- The priority model (runtime > file-based) ensures dynamically registered commands always show up-to-date help even if `.help.md` files are stale
- Hot-reload of `.help.md` files uses VFS file-watcher integration — the help system does not poll; it reacts to change events
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The SearchIndex is a simple inverted index — no full-text search engine dependency; complexity is bounded by the total help corpus size (expected < 500 topics)
- The Help Menu is a data model defining menu items — actual menu rendering is in the `menu-and-statusbar` GUI shell layer
- The About dialog content (version, build date, Rust compiler version) is populated from compile-time environment variables (`env!("CARGO_PKG_VERSION")`, `env!("VERGEN_BUILD_DATE")`, etc.)
- Help content authoring for primary commands (Req 7), line commands (Req 8), macro API (Req 9), configuration keys (Req 10), and modes/features (Req 11) is a content task — the `.help.md` files are data artifacts separate from the crate source code; those tasks are tracked in a content-authoring plan, not here

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: F1 Key — Context-Sensitive Help | AC 1.1 | Tasks 4, 12 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.2 | Task 4 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.3 | Task 4 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.4 | Task 4 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.5 | Task 4 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.6 | Tasks 5, 12 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.7 | Task 4 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.8 | Task 12 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.9 | Tasks 4, 12 |
| Req 1: F1 Key — Context-Sensitive Help | AC 1.10 | Task 12 |
| Req 2: Help Panel — Dockable Display | AC 2.1 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.2 | Tasks 5, 11 |
| Req 2: Help Panel — Dockable Display | AC 2.3 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.4 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.5 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.6 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.7 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.8 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.9 | Task 5 |
| Req 2: Help Panel — Dockable Display | AC 2.10 | Task 5 |
| Req 3: Help Navigation | AC 3.1 | Task 6 |
| Req 3: Help Navigation | AC 3.2 | Task 6 |
| Req 3: Help Navigation | AC 3.3 | Task 6 |
| Req 3: Help Navigation | AC 3.4 | Task 5 |
| Req 3: Help Navigation | AC 3.5 | Task 6 |
| Req 3: Help Navigation | AC 3.6 | Task 6 |
| Req 4: Help Search | AC 4.1 | Task 7 |
| Req 4: Help Search | AC 4.2 | Task 7 |
| Req 4: Help Search | AC 4.3 | Task 7 |
| Req 4: Help Search | AC 4.4 | Task 7 |
| Req 4: Help Search | AC 4.5 | Task 7 |
| Req 5: Help Content Format | AC 5.1 | Task 3 |
| Req 5: Help Content Format | AC 5.2 | Tasks 2, 3 |
| Req 5: Help Content Format | AC 5.3 | Task 3 |
| Req 5: Help Content Format | AC 5.4 | Task 3 |
| Req 5: Help Content Format | AC 5.5 | Task 8 |
| Req 5: Help Content Format | AC 5.6 | Task 3 |
| Req 5: Help Content Format | AC 5.7 | Task 3 |
| Req 6: Help Topic Registry | AC 6.1 | Tasks 8, 9 |
| Req 6: Help Topic Registry | AC 6.2 | Task 8 |
| Req 6: Help Topic Registry | AC 6.3 | Task 8 |
| Req 6: Help Topic Registry | AC 6.4 | Task 8 |
| Req 6: Help Topic Registry | AC 6.5 | Task 8 |
| Req 6: Help Topic Registry | AC 6.6 | Task 9 |
| Req 6: Help Topic Registry | AC 6.7 | Task 8 |
| Req 7: Help Content — Primary Commands | AC 7.1 | Task 8 |
| Req 7: Help Content — Primary Commands | AC 7.2 | Content authoring (external) |
| Req 7: Help Content — Primary Commands | AC 7.3 | Content authoring (external) |
| Req 7: Help Content — Primary Commands | AC 7.4 | Content authoring (external) |
| Req 8: Help Content — Line Commands | AC 8.1 | Task 8 |
| Req 8: Help Content — Line Commands | AC 8.2 | Content authoring (external) |
| Req 8: Help Content — Line Commands | AC 8.3 | Content authoring (external) |
| Req 8: Help Content — Line Commands | AC 8.4 | Tasks 10, 12 |
| Req 9: Help Content — Macro API | AC 9.1 | Content authoring (external) |
| Req 9: Help Content — Macro API | AC 9.2 | Content authoring (external) |
| Req 9: Help Content — Macro API | AC 9.3 | Task 9 |
| Req 9: Help Content — Macro API | AC 9.4 | Task 9 |
| Req 10: Help Content — Configuration Keys | AC 10.1 | Content authoring (external) |
| Req 10: Help Content — Configuration Keys | AC 10.2 | Task 8 |
| Req 10: Help Content — Configuration Keys | AC 10.3 | Content authoring (external) |
| Req 10: Help Content — Configuration Keys | AC 10.4 | Task 12 |
| Req 11: Help Content — Modes and Features | AC 11.1 | Content authoring (external) |
| Req 11: Help Content — Modes and Features | AC 11.2 | Content authoring (external) |
| Req 11: Help Content — Modes and Features | AC 11.3 | Content authoring (external) |
| Req 11: Help Content — Modes and Features | AC 11.4 | Content authoring (external) |
| Req 11: Help Content — Modes and Features | AC 11.5 | Content authoring (external) |
| Req 12: Help Index | AC 12.1 | Tasks 10, 12 |
| Req 12: Help Index | AC 12.2 | Task 10 |
| Req 12: Help Index | AC 12.3 | Task 10 |
| Req 12: Help Index | AC 12.4 | Task 10 |
| Req 13: HELP Primary Command | AC 13.1 | Task 12 |
| Req 13: HELP Primary Command | AC 13.2 | Task 12 |
| Req 13: HELP Primary Command | AC 13.3 | Task 12 |
| Req 13: HELP Primary Command | AC 13.4 | Task 12 |
| Req 13: HELP Primary Command | AC 13.5 | Task 12 |
| Req 13: HELP Primary Command | AC 13.6 | Task 12 |
| Req 13: HELP Primary Command | AC 13.7 | Task 12 |
| Req 13: HELP Primary Command | AC 13.8 | Task 12 |
| Req 13: HELP Primary Command | AC 13.9 | Task 12 |
| Req 13: HELP Primary Command | AC 13.10 | Task 12 |
| Req 14: Help Menu Integration | AC 14.1 | Task 13 |
| Req 14: Help Menu Integration | AC 14.2 | Task 13 |
| Req 14: Help Menu Integration | AC 14.3 | Task 13 |
| Req 14: Help Menu Integration | AC 14.4 | Task 13 |
| Req 14: Help Menu Integration | AC 14.5 | Task 13 |
| Req 14: Help Menu Integration | AC 14.6 | Task 13 |
| Req 15: Dynamic Help — Function Keys | AC 15.1 | Task 10 |
| Req 15: Dynamic Help — Function Keys | AC 15.2 | Task 10 |
| Req 15: Dynamic Help — Function Keys | AC 15.3 | Task 10 |
| Req 15: Dynamic Help — Function Keys | AC 15.4 | Task 10 |
| Req 16: Help System Configuration | AC 16.1 | Task 11 |
| Req 16: Help System Configuration | AC 16.2 | Task 11 |
| Req 16: Help System Configuration | AC 16.3 | Task 11 |
