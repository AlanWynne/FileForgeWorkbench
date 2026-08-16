# Design Document: Command Completion (`ff-completion`)

## Overview

The `ff-completion` crate implements the **auto-complete popup system** for FileForgeWorkbench. It provides context-sensitive command name, argument, and line command completion in the primary command field and prefix area. The crate is **GUI-independent** in its core logic — candidate generation, filtering, ranking, and selection state management operate without any GUI dependency. Only the popup positioning model produces layout coordinates consumed by the shell renderer.

### Purpose

- Generate completion candidates from registered commands, VFS file paths, macro names, keywords, and line commands
- Filter and rank candidates using configurable prefix or fuzzy matching
- Manage popup selection state (highlight, scroll position, navigation)
- Compute popup anchor position and flip direction relative to the command field
- Support configurable trigger modes (manual, automatic, both)
- Allow plugins to register custom `CompletionProvider` implementations for their commands

### Position in Architecture

```
Wave 10 — Extensions and Macros

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│     Popup rendering, key event forwarding                    │
├─────────────────────────────────────────────────────────────┤
│         ff-completion (THIS CRATE — Wave 10)                 │
│   Completion engine, providers, popup model, navigation      │
├─────────────────────────────────────────────────────────────┤
│  ff-command (Wave 2) — CommandRegistry, CommandMetadata       │
│  ff-command-semantics (Wave 5) — parsed command context       │
│  ff-line-commands (Wave 5) — line command kinds               │
│  ff-config (Wave 2) — completion.* settings                  │
│  ff-vfs (Wave 3) — async directory listing                   │
│  lua-macro-engine (Wave 10) — macro name list                │
│  ff-plugin (Wave 2) — provider registration lifecycle        │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging (Wave 0)            │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 2)**: The completion engine, matching algorithms, and selection state have zero GUI dependencies. The popup model is data-only; rendering is shell-side.
- **Command-Driven (Req 4)**: The manual trigger action registers as Command_ID `"completion.trigger"` in the Shortcut_Registry.
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-completion`
- **Error Message Standards (Req 8)**: All errors follow `[completion] operation: description` format
- **Non-Blocking (Req 6)**: VFS path completion uses async I/O; provider invocations are async-capable
- **Extensible (Req 3)**: Plugin providers use the same `CompletionProvider` trait as built-in providers

### Upstream Dependencies

| Crate | Purpose |
|-------|---------|
| `ff-command` | CommandRegistry for command name listing; CommandMetadata for display enrichment; ShortcutRegistry for trigger binding |
| `ff-command-semantics` | Parsed command name for argument context determination |
| `ff-line-commands` | Line command kind definitions for prefix-area completion |
| `ff-config` | All `completion.*` namespace settings |
| `ff-vfs` | Async directory listing for file path argument completion |
| `lua-macro-engine` | Macro name enumeration for macro argument completion |
| `ff-plugin` | Provider registration/deregistration lifecycle hooks |
| `ff-logging` | Diagnostic output (WARN on provider failure, invalid config) |

### Downstream Consumers

| Consumer | Integration |
|----------|-------------|
| `ff-desktop` (GUI shell) | Reads `CompletionPopup` model to render overlay; forwards key events to `CompletionEngine` |

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Input Events"
        KE[Key Events<br/>typing, arrows, tab, enter, escape]
        MT[Manual Trigger<br/>Ctrl+Space]
        FC[Focus Change<br/>field blur/switch]
    end

    subgraph "ff-completion"
        CE[CompletionEngine<br/>orchestrates trigger, filter, accept]
        PM[ProviderManager<br/>registry of CompletionProviders]
        FM[FilterMatcher<br/>prefix or fuzzy matching]
        RK[Ranker<br/>relevance scoring + sort]
        SS[SelectionState<br/>highlight index, scroll offset]
        PP[PopupPositioner<br/>anchor + flip logic]
        CM[ConfigManager<br/>reads completion.* namespace]
        TC[TriggerController<br/>auto/manual threshold logic]
    end

    subgraph "Built-in Providers"
        CP[CommandNameProvider<br/>queries CommandRegistry]
        FP[FilePathProvider<br/>async VFS listing]
        KP[KeywordProvider<br/>scope modifiers, find keywords]
        MP[MacroNameProvider<br/>queries lua-macro-engine]
        LP[LineCommandProvider<br/>line command kinds]
    end

    subgraph "Upstream Crates"
        CR[ff-command<br/>CommandRegistry]
        CS[ff-command-semantics<br/>ParsedCommand context]
        VFS[ff-vfs<br/>async dir listing]
        LUA[lua-macro-engine<br/>macro list]
        CFG[ff-config<br/>completion.* keys]
        LC[ff-line-commands<br/>LineCommandKind]
    end

    KE --> CE
    MT --> CE
    FC --> CE
    CE --> TC
    TC --> PM
    PM --> CP
    PM --> FP
    PM --> KP
    PM --> MP
    PM --> LP
    CP --> CR
    FP --> VFS
    MP --> LUA
    LP --> LC
    PM --> FM
    FM --> RK
    RK --> SS
    SS --> PP
    CE --> CM
    CM --> CFG
    CE --> CS
end
```

### Layer Placement

| Layer | Components | Role |
|-------|-----------|------|
| **Trigger Layer** | `TriggerController`, `ConfigManager` | Decides when to activate completion based on trigger mode, threshold, and manual shortcuts |
| **Provider Layer** | `ProviderManager`, all built-in providers | Generates raw candidate lists from various sources |
| **Matching Layer** | `FilterMatcher` | Applies prefix or fuzzy matching against typed text |
| **Ranking Layer** | `Ranker` | Scores and sorts filtered candidates by relevance |
| **Selection Layer** | `SelectionState` | Tracks highlighted item, scroll offset, handles navigation |
| **Positioning Layer** | `PopupPositioner` | Computes popup anchor, dimensions, and flip direction |

### Data Flow (Completion Lifecycle)

```
1. User types characters in command field (or presses Ctrl+Space)
2. TriggerController evaluates: is trigger threshold met? Is mode appropriate?
3. CompletionEngine builds CompletionContext from current field state
4. ProviderManager queries applicable providers → raw Vec<CompletionCandidate>
5. FilterMatcher applies prefix/fuzzy filter → filtered candidates
6. Ranker scores and sorts → ranked Vec<CompletionItem>
7. SelectionState initializes (highlight = 0, scroll = 0)
8. PopupPositioner computes anchor coordinates and popup dimensions
9. CompletionPopup model published for shell renderer
10. On each keystroke: re-filter from step 5 (incremental)
11. On accept (Tab/Enter): insert selected candidate, dismiss popup
12. On dismiss (Escape/focus loss/stop char): close popup, no insertion
```

---

## Module Structure

```
crates/ff-completion/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── engine.rs               # CompletionEngine — central orchestrator
│   ├── context.rs              # CompletionContext — trigger state snapshot
│   ├── candidate.rs            # CompletionCandidate — raw provider output
│   ├── item.rs                 # CompletionItem — ranked, display-ready item
│   ├── list.rs                 # CompletionList — filtered + sorted collection
│   ├── popup.rs                # CompletionPopup — position model for renderer
│   ├── selection.rs            # SelectionState — highlight, scroll, navigation
│   ├── trigger.rs              # TriggerController — activation logic
│   ├── config.rs               # CompletionConfig — typed config access
│   ├── matching/
│   │   ├── mod.rs              # Re-exports
│   │   ├── prefix.rs           # Prefix matching (case-insensitive)
│   │   ├── fuzzy.rs            # Fuzzy/subsequence matching + highlight spans
│   │   └── scorer.rs           # Match quality scoring for ranking
│   ├── ranking.rs              # Ranker — multi-signal relevance sorting
│   ├── positioning.rs          # PopupPositioner — anchor, flip, clipping
│   ├── provider/
│   │   ├── mod.rs              # CompletionProvider trait, ProviderManager
│   │   ├── command_name.rs     # CommandNameProvider (queries CommandRegistry)
│   │   ├── file_path.rs        # FilePathProvider (async VFS listing)
│   │   ├── keyword.rs          # KeywordProvider (scope modifiers, etc.)
│   │   ├── macro_name.rs       # MacroNameProvider (queries lua-macro-engine)
│   │   └── line_command.rs     # LineCommandProvider (line command kinds)
│   └── error.rs                # CompletionError enum
└── tests/
    ├── engine_tests.rs         # Engine lifecycle property tests
    ├── matching_tests.rs       # Prefix and fuzzy matching property tests
    ├── ranking_tests.rs        # Ranking order property tests
    ├── navigation_tests.rs     # Selection navigation property tests
    ├── positioning_tests.rs    # Popup positioning property tests
    ├── provider_tests.rs       # Provider integration tests
    └── config_tests.rs         # Configuration validation tests
```

---

## Data Models

### CompletionContext

```rust
/// The state snapshot at the moment completion is triggered or re-evaluated.
/// Passed to providers so they can generate contextually-relevant candidates.
/// Addresses: Requirements 1, 2, 7
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Which input field triggered the completion.
    pub field: CompletionField,
    /// The full text content of the field.
    pub field_text: String,
    /// The cursor position within the field (0-indexed character offset).
    pub cursor_offset: usize,
    /// The prefix being completed — substring from anchor to cursor.
    pub prefix: String,
    /// The anchor offset — start of the prefix within the field.
    pub anchor_offset: usize,
    /// The parsed command name, if in argument position (None if completing command name).
    pub command_name: Option<String>,
    /// The argument index being completed (0 = first arg after command name).
    pub argument_index: Option<usize>,
    /// The Command_ID of the resolved command (if known).
    pub command_id: Option<String>,
}

/// Identifies which input field is being completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionField {
    /// The primary command field at the top of the editor panel.
    PrimaryCommand,
    /// The prefix area (line command input) on a specific line.
    PrefixArea,
}
```

### CompletionCandidate

```rust
/// A raw completion candidate produced by a CompletionProvider.
/// Contains all information needed for filtering, ranking, and display.
/// Addresses: Requirements 1.3, 2, 7.2
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    /// The display label shown in the popup.
    pub label: String,
    /// The text to insert when this candidate is accepted.
    pub insert_text: String,
    /// The category or source of this candidate (for grouping/icons).
    pub kind: CompletionKind,
    /// Optional detail text shown alongside the label (e.g., command category).
    pub detail: Option<String>,
    /// Optional longer description for tooltip display.
    pub description: Option<String>,
    /// Provider-assigned relevance weight (higher = more relevant). Default 0.
    pub base_relevance: i32,
}

/// The kind/category of a completion candidate — used for icon display and grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompletionKind {
    /// A registered command name.
    Command,
    /// A file or directory path from VFS.
    FilePath,
    /// A directory entry (shown with trailing separator).
    Directory,
    /// A command keyword or modifier (e.g., CHARS, PREFIX, VISIBLE).
    Keyword,
    /// A registered Lua macro name.
    Macro,
    /// A line command kind (e.g., CC, DD, M5).
    LineCommand,
    /// A plugin-provided custom candidate.
    Plugin,
}
```

### CompletionItem

```rust
/// A display-ready completion item after filtering and ranking.
/// Includes match highlight information for the popup renderer.
/// Addresses: Requirements 6.3, 1.4
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The original candidate data.
    pub candidate: CompletionCandidate,
    /// Computed relevance score (higher is better). Used for sort order.
    pub score: f64,
    /// Character positions in the label that matched the typed prefix.
    /// Used by the renderer to highlight matched characters.
    pub match_positions: Vec<usize>,
}
```

### CompletionList

```rust
/// The complete set of filtered, ranked completion items for the current context.
/// Serves as the data model consumed by SelectionState and PopupPositioner.
/// Addresses: Requirements 1.6, 1.7, 4
#[derive(Debug, Clone)]
pub struct CompletionList {
    /// The filtered and ranked items, in display order (highest relevance first).
    items: Vec<CompletionItem>,
    /// Whether items were truncated to fit max display limit.
    is_truncated: bool,
}

impl CompletionList {
    /// Creates a new list from ranked items.
    pub fn new(items: Vec<CompletionItem>) -> Self;

    /// Returns the number of items in the list.
    pub fn len(&self) -> usize;

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool;

    /// Returns a reference to the item at the given index.
    pub fn get(&self, index: usize) -> Option<&CompletionItem>;

    /// Returns a slice of all items.
    pub fn items(&self) -> &[CompletionItem];

    /// Returns items within the visible window for rendering.
    pub fn visible_window(&self, scroll_offset: usize, max_visible: usize) -> &[CompletionItem];
}
```

### CompletionPopup

```rust
/// The data model describing the popup's position, dimensions, and content
/// for the GUI shell renderer. This struct is GUI-independent — it contains
/// only coordinates and data. The shell reads it each frame to paint the overlay.
/// Addresses: Requirement 3
#[derive(Debug, Clone)]
pub struct CompletionPopup {
    /// Whether the popup is currently visible.
    pub visible: bool,
    /// The anchor point (top-left of the popup relative to window origin).
    pub anchor: PopupAnchor,
    /// The computed width of the popup in logical pixels.
    pub width: f32,
    /// The computed height of the popup in logical pixels.
    pub height: f32,
    /// Whether the popup is positioned above the command field (flipped).
    pub flipped: bool,
    /// The current completion list being displayed.
    pub list: CompletionList,
    /// The index of the currently highlighted item within the list.
    pub selected_index: usize,
    /// The scroll offset (first visible item index).
    pub scroll_offset: usize,
    /// Maximum number of visible items (from config).
    pub max_visible_items: usize,
}

/// The anchor coordinates for popup placement.
/// Addresses: Requirement 3.1
#[derive(Debug, Clone, Copy)]
pub struct PopupAnchor {
    /// X coordinate — horizontal position at the start of the prefix in the command field.
    pub x: f32,
    /// Y coordinate — vertical position (bottom edge of command field for below,
    /// top edge for above).
    pub y: f32,
}
```

### CompletionProvider

```rust
/// The trait that completion candidate sources implement.
/// Both built-in providers and plugin-registered providers use this trait.
/// Addresses: Requirement 10
#[async_trait::async_trait]
pub trait CompletionProvider: Send + Sync {
    /// Returns a stable identifier for this provider (for logging and deregistration).
    fn id(&self) -> &str;

    /// Returns true if this provider can produce candidates for the given context.
    /// Called before `provide_candidates` as a fast filter.
    fn is_applicable(&self, context: &CompletionContext) -> bool;

    /// Generates completion candidates for the given context.
    /// May be async (e.g., VFS directory listing). Implementations should
    /// respect cancellation if the context changes mid-flight.
    async fn provide_candidates(
        &self,
        context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError>;
}
```

### CompletionConfig

```rust
/// Typed representation of all `completion.*` configuration keys.
/// Read from ff-config at engine initialization and on hot-reload.
/// Addresses: Requirement 9
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    /// Trigger mode: Manual, Automatic, or Both.
    pub trigger_mode: TriggerMode,
    /// Character count threshold for automatic triggering (1–10).
    pub auto_trigger_chars: u8,
    /// Matching algorithm: Prefix or Fuzzy.
    pub matching_mode: MatchingMode,
    /// Whether matching is case-sensitive.
    pub case_sensitive: bool,
    /// Maximum visible candidates in popup (3–50).
    pub popup_max_items: u8,
    /// Maximum popup width in logical pixels (100–1000).
    pub popup_max_width: u16,
    /// Whether to auto-hide when zero candidates match.
    pub auto_hide: bool,
    /// Whether to dismiss when cursor retreats past anchor.
    pub cancel_at_start_pos: bool,
    /// Whether to auto-accept when only one candidate matches.
    pub choose_single: bool,
    /// Whether arrow navigation wraps around list edges.
    pub wrap_navigation: bool,
    /// Characters that dismiss the popup when typed.
    pub stop_chars: Vec<char>,
    /// Characters that accept the selection when typed.
    pub fill_up_chars: Vec<char>,
    /// Whether prefix-area completion is enabled.
    pub line_command_completion: bool,
    /// Whether accepting a candidate removes trailing text up to next word boundary.
    pub drop_rest_of_word: bool,
}

/// Trigger activation mode.
/// Addresses: Requirement 9.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerMode {
    /// Only activate on explicit Ctrl+Space.
    Manual,
    /// Activate after typing N characters (auto_trigger_chars threshold).
    Automatic,
    /// Both automatic and manual triggers are active.
    Both,
}

/// Matching algorithm selection.
/// Addresses: Requirements 6.1, 6.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchingMode {
    /// Strict prefix match (candidate must start with typed text).
    Prefix,
    /// Subsequence match (all typed characters appear in order).
    Fuzzy,
}
```

### NavigationAction

```rust
/// Actions the shell forwards to the CompletionEngine when the popup is visible.
/// Addresses: Requirement 4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    /// Move selection down by one item.
    Down,
    /// Move selection up by one item.
    Up,
    /// Move selection down by one page (max_visible_items count).
    PageDown,
    /// Move selection up by one page.
    PageUp,
    /// Accept the current selection (Tab behaviour — insert and dismiss).
    AcceptTab,
    /// Accept and execute (Enter behaviour — insert, dismiss, and submit if at end).
    AcceptEnter,
    /// Dismiss without accepting.
    Dismiss,
}

/// The result of processing a navigation action or text change.
/// Tells the shell what happened so it can update the command field.
#[derive(Debug, Clone)]
pub enum CompletionAction {
    /// The popup state was updated (re-render needed).
    PopupUpdated,
    /// A candidate was accepted — the shell should perform this text insertion.
    Accept {
        /// Text to insert, replacing the prefix at [anchor_offset..cursor_offset].
        insert_text: String,
        /// Whether to append a trailing space after insertion.
        trailing_space: bool,
        /// Whether to submit the command (Enter with no further args expected).
        submit: bool,
    },
    /// The popup was dismissed (hide it).
    Dismissed,
    /// No change — the event was not consumed by completion.
    NotConsumed,
}
```

---

## Components and Interfaces

### CompletionEngine

```rust
/// The central orchestrator for the completion subsystem.
/// Manages the lifecycle: trigger → provide → filter → rank → navigate → accept/dismiss.
/// Addresses: All requirements
pub struct CompletionEngine { /* ... */ }

impl CompletionEngine {
    /// Create a new engine with the given configuration and provider manager.
    pub fn new(config: CompletionConfig, provider_manager: ProviderManager) -> Self;

    /// Notify the engine that text changed in the active field.
    /// The engine evaluates whether to trigger, re-filter, or dismiss.
    /// Returns the resulting action for the shell to handle.
    /// Addresses: Requirements 1.6, 5.3, 5.4, 9.3
    pub async fn on_text_changed(
        &mut self,
        field: CompletionField,
        text: &str,
        cursor_offset: usize,
    ) -> CompletionAction;

    /// Notify the engine that the user explicitly triggered completion (Ctrl+Space).
    /// Addresses: Requirement 9.2
    pub async fn on_manual_trigger(
        &mut self,
        field: CompletionField,
        text: &str,
        cursor_offset: usize,
    ) -> CompletionAction;

    /// Process a navigation action while the popup is visible.
    /// Addresses: Requirement 4
    pub fn on_navigation(&mut self, action: NavigationAction) -> CompletionAction;

    /// Notify the engine that focus left the command field.
    /// Addresses: Requirement 5.2
    pub fn on_focus_lost(&mut self) -> CompletionAction;

    /// Notify the engine that the command was submitted (Enter to execute).
    /// Addresses: Requirement 5.5
    pub fn on_command_submit(&mut self) -> CompletionAction;

    /// Returns the current popup model for rendering.
    /// The shell calls this each frame to paint the overlay.
    pub fn popup(&self) -> &CompletionPopup;

    /// Returns true if the popup is currently visible.
    pub fn is_active(&self) -> bool;

    /// Update configuration (called on hot-reload notification).
    /// Addresses: Requirement 9.6
    pub fn update_config(&mut self, config: CompletionConfig);

    /// Set the viewport dimensions for popup positioning calculations.
    /// Called on window resize.
    /// Addresses: Requirement 3.8
    pub fn set_viewport(&mut self, viewport: ViewportRect);

    /// Set the command field rectangle for popup anchor computation.
    pub fn set_field_rect(&mut self, rect: FieldRect);
}
```

### ProviderManager

```rust
/// Manages the registry of CompletionProviders (built-in and plugin-contributed).
/// Addresses: Requirement 10
pub struct ProviderManager { /* ... */ }

impl ProviderManager {
    /// Create a new manager with the built-in providers registered.
    pub fn new(
        command_registry: Arc<CommandRegistry>,
        // Other upstream references passed during construction
    ) -> Self;

    /// Register a plugin-provided CompletionProvider.
    /// Addresses: Requirement 10.2
    pub fn register_provider(&self, provider: Box<dyn CompletionProvider>);

    /// Deregister all providers associated with a given plugin ID.
    /// Addresses: Requirement 10.3
    pub fn deregister_plugin_providers(&self, plugin_id: &str);

    /// Query all applicable providers for the given context and merge results.
    /// Provider failures are logged and skipped (Requirement 10.5).
    /// Addresses: Requirements 10.4, 2.7
    pub async fn provide_candidates(
        &self,
        context: &CompletionContext,
    ) -> Vec<CompletionCandidate>;
}
```

### FilterMatcher

```rust
/// Applies the configured matching algorithm to filter candidates.
/// Addresses: Requirements 1.2, 6.1, 6.2, 6.6
pub struct FilterMatcher { /* ... */ }

impl FilterMatcher {
    /// Create a matcher with the given mode and case sensitivity.
    pub fn new(mode: MatchingMode, case_sensitive: bool) -> Self;

    /// Filter candidates against the typed prefix.
    /// Returns items that match, with match positions populated.
    pub fn filter(
        &self,
        candidates: &[CompletionCandidate],
        prefix: &str,
    ) -> Vec<CompletionItem>;

    /// Update matching configuration (on hot-reload).
    pub fn set_mode(&mut self, mode: MatchingMode, case_sensitive: bool);
}
```

### Ranker

```rust
/// Scores and sorts filtered items by multi-signal relevance.
/// Addresses: Requirements 1.4, 6.4
pub struct Ranker { /* ... */ }

impl Ranker {
    /// Create a new ranker.
    pub fn new() -> Self;

    /// Rank the given items in-place by computed score (descending).
    /// Scoring signals: exact prefix bonus, match contiguity (fuzzy),
    /// shorter name preference, frequency weight from command history.
    pub fn rank(&self, items: &mut Vec<CompletionItem>, context: &CompletionContext);
}
```

### PopupPositioner

```rust
/// Computes popup position, dimensions, and flip direction.
/// Addresses: Requirement 3
pub struct PopupPositioner { /* ... */ }

impl PopupPositioner {
    /// Create a positioner with the given viewport and field geometry.
    pub fn new(viewport: ViewportRect, field_rect: FieldRect) -> Self;

    /// Compute the popup anchor and dimensions for the given list and config.
    /// Addresses: Requirements 3.1–3.8
    pub fn compute(
        &self,
        anchor_x: f32,
        item_count: usize,
        longest_label_width: f32,
        config: &CompletionConfig,
    ) -> PopupGeometry;

    /// Update the viewport (on window resize).
    pub fn set_viewport(&mut self, viewport: ViewportRect);

    /// Update the field rectangle (on layout change).
    pub fn set_field_rect(&mut self, rect: FieldRect);
}

/// Computed geometry for popup placement.
#[derive(Debug, Clone, Copy)]
pub struct PopupGeometry {
    pub anchor: PopupAnchor,
    pub width: f32,
    pub height: f32,
    pub flipped: bool,
    pub max_visible_items: usize,
}

/// The application window viewport rectangle.
#[derive(Debug, Clone, Copy)]
pub struct ViewportRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// The command field rectangle (for positioning relative to the field).
#[derive(Debug, Clone, Copy)]
pub struct FieldRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

---

## Error Handling

```rust
/// Errors produced by the completion subsystem.
/// Addresses: Cross-cutting Requirement 8 (error format: [completion] operation: description)
#[derive(Debug, thiserror::Error)]
pub enum CompletionError {
    /// A provider failed to produce candidates.
    #[error("[completion] provider '{provider_id}': {reason}")]
    ProviderFailed {
        provider_id: String,
        reason: String,
    },

    /// VFS directory listing failed during file path completion.
    #[error("[completion] vfs_listing '{path}': {reason}")]
    VfsListingFailed {
        path: String,
        reason: String,
    },

    /// Configuration value is invalid (wrong type, out of range).
    #[error("[completion] config '{key}': invalid value '{value}', using default '{default}'")]
    InvalidConfig {
        key: String,
        value: String,
        default: String,
    },

    /// Provider registration failed (duplicate ID).
    #[error("[completion] register_provider: provider '{provider_id}' already registered")]
    DuplicateProvider {
        provider_id: String,
    },

    /// Internal error — should not occur in normal operation.
    #[error("[completion] internal: {0}")]
    Internal(String),
}
```

---

## Integration Points

### Integration with `ff-command` (Command Framework)

| Integration | Detail |
|-------------|--------|
| **CommandRegistry::list_all()** | `CommandNameProvider` calls this to get the full set of registered command IDs for name completion |
| **CommandRegistry::metadata(id)** | Used to enrich `CompletionCandidate` with display_name, category, and description from `CommandMetadata` |
| **ShortcutRegistry** | The manual trigger shortcut (default Ctrl+Space) is registered as Command_ID `"completion.trigger"` |
| **CommandHistory** | The `Ranker` optionally queries command history for frequency weighting (recently-used commands rank higher) |

### Integration with `ff-command-semantics` (Command Semantics)

| Integration | Detail |
|-------------|--------|
| **Parsed command name** | When the cursor is in argument position, the engine consults the parsed command name to determine which providers are applicable |
| **Argument schema** | If a command publishes argument metadata (expected types per position), the engine uses it to select the correct provider (file path, keyword, etc.) |
| **Command normalizer** | Abbreviation resolution from command-semantics helps map typed abbreviations to canonical command names for provider lookup |

### Integration with `ff-config` (Configuration System)

| Integration | Detail |
|-------------|--------|
| **Namespace `completion.*`** | All 14 configuration keys (Requirement 9.1) are read from this namespace |
| **Hot-reload callback** | The engine registers a reload callback with ff-config; on notification, it calls `update_config()` to apply new settings |
| **Validation + fallback** | Invalid config values produce a WARN log and fall back to defaults (Requirement 9.5) |

### Integration with `ff-vfs` (Virtual File System)

| Integration | Detail |
|-------------|--------|
| **Async directory listing** | `FilePathProvider` calls the VFS async API to list entries matching a path prefix |
| **Resource URI support** | Supports both bare paths and `vfs://provider/path` URIs (Requirement 2.3) |
| **Non-blocking** | All VFS calls are awaited; never blocks the UI thread |

### Integration with `lua-macro-engine`

| Integration | Detail |
|-------------|--------|
| **Macro name list** | `MacroNameProvider` queries the macro engine for all registered macro names |
| **Cache refresh** | On macro add/remove/reload notifications, the provider refreshes its cached list (Requirement 8.3) |

### Integration with `ff-line-commands`

| Integration | Detail |
|-------------|--------|
| **Line command kinds** | `LineCommandProvider` reads the set of valid line command kinds (C, CC, M, MM, D, DD, etc.) and their descriptions |
| **Prefix-area context** | When `CompletionField::PrefixArea`, only the `LineCommandProvider` is applicable |

### Integration with `ff-plugin` (Plugin Architecture)

| Integration | Detail |
|-------------|--------|
| **Provider registration** | Plugins register `CompletionProvider` instances during their `initialize` lifecycle phase via `ProviderManager::register_provider()` |
| **Provider deregistration** | When a plugin is unloaded, `ProviderManager::deregister_plugin_providers(plugin_id)` removes all its providers |

---

## Correctness Properties

These properties define invariants suitable for property-based testing with `proptest`.

### Property 1: Prefix Match Correctness

**Statement:** For any non-empty prefix `p` and any candidate label `l`, the prefix matcher returns `l` as a match if and only if `l` (case-folded when case_sensitive=false) starts with `p` (case-folded).

**Validates: Requirements 1.2, 6.2, 6.6**

### Property 2: Fuzzy Match Subsequence Invariant

**Statement:** For any non-empty query `q` and candidate label `l`, the fuzzy matcher returns `l` as a match if and only if all characters of `q` appear in `l` in the same order (case-folded when case_sensitive=false). The `match_positions` vector has length equal to `q.len()` and positions are strictly increasing.

**Validates: Requirements 6.1, 6.3**

### Property 3: Navigation Wrapping Invariant

**Statement:** Given a list of N items (N > 0) with wrap_navigation=true: performing N consecutive Down actions from index 0 returns the selection to index 0. Performing one Up action from index 0 moves to index N-1.

**Validates: Requirements 4.1, 4.2**

### Property 4: Navigation Non-Wrapping Clamping

**Statement:** Given a list of N items (N > 0) with wrap_navigation=false: performing Down at index N-1 remains at N-1. Performing Up at index 0 remains at 0.

**Validates: Requirements 4.1, 4.2**

### Property 5: Popup Positioning Never Exceeds Viewport

**Statement:** For any viewport dimensions (width > 0, height > 0), any field position within the viewport, and any non-empty candidate list: the computed popup geometry (anchor + width, anchor + height) never extends beyond the viewport boundaries.

**Validates: Requirements 3.2, 3.3, 3.4**

### Property 6: Popup Does Not Overlap Command Field

**Statement:** For any computed popup geometry and field rectangle: the popup rectangle does not intersect the field rectangle.

**Validates: Requirements 3.5**

### Property 7: Accept Replaces Only Prefix

**Statement:** When a candidate is accepted, the insertion replaces exactly the text from `anchor_offset` to `cursor_offset`. Text before `anchor_offset` and text after `cursor_offset` is preserved unchanged.

**Validates: Requirements 4.10, 1.5**

### Property 8: Dynamic Filtering Monotonicity

**Statement:** Given a set of candidates and prefix `p`, extending `p` by one character to `p'` yields a result set that is a subset of (or equal to) the result set for `p`. (Adding characters can only narrow the matches.)

**Validates: Requirements 1.6**

### Property 9: Empty Match List Causes Auto-Hide

**Statement:** When `auto_hide=true` and filtering produces zero matching candidates, `CompletionPopup.visible` becomes false after the filter operation.

**Validates: Requirements 1.7, 5.4**

### Property 10: Config Validation Falls Back to Defaults

**Statement:** For every configuration key with a defined valid range: if a value outside that range is loaded, the engine uses the documented default value and the error is a `CompletionError::InvalidConfig` with the correct key, value, and default.

**Validates: Requirements 9.5, 6.5**

### Property 11: Ranking Exact Prefix Over Fuzzy

**Statement:** When matching_mode=fuzzy, candidates whose match starts at position 0 (prefix-like) always rank higher than candidates whose match starts at a later position, given equal match length and base_relevance.

**Validates: Requirements 6.4**

### Property 12: Provider Failure Isolation

**Statement:** If one registered provider returns an error (or panics), the engine still returns candidates from all other functioning providers. The total candidate count is at least the sum of successful providers' outputs.

**Validates: Requirements 10.5**

### Property 13: Single-Candidate Auto-Accept

**Statement:** When `choose_single=true` and filtering produces exactly one matching candidate, the engine returns `CompletionAction::Accept` with that candidate's insert_text, without making the popup visible.

**Validates: Requirements 4.9**

## Testing Strategy

- **Crate:** `proptest` (already in workspace `[dev-dependencies]`)
- **Minimum iterations:** 256 per property
- **Generators:** Custom strategies for `CompletionContext`, `CompletionCandidate` lists, prefix strings, viewport dimensions, and field rectangles
- **Regression files:** Committed alongside tests in `tests/` directory
- **Unit tests:** Co-located in each module's `#[cfg(test)] mod tests` block
- **Integration tests:** `tests/` directory exercising full engine lifecycle (trigger → filter → navigate → accept)
- **Coverage target:** Every acceptance criterion from requirements.md has at least one automated test
