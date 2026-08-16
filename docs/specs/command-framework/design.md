# Design Document: Command Framework (`ff-command`)

## 1. Overview

The `ff-command` crate is the **central dispatch mechanism** for all user-facing operations in the FileForgeWorkbench platform. It implements the Command-Driven Architecture principle (cross-cutting Requirement 4) by providing:

- A global **Command Registry** for registering and discovering commands
- A single **Command Dispatch** entry point through which all state changes flow
- Rich **Command Metadata** for menus, palettes, and help systems
- Automatic **Undo/Redo Integration** with the transaction system
- A **Keyboard Shortcut Registry** with conflict detection and reserved shortcuts
- A **Scripting Bridge** for Lua macro command invocation
- A **Command History** log for retrieval and audit

### Position in Architecture

```
Wave 2 — Platform Architecture

┌─────────────────────────────────────────────────────────┐
│                    Application Binary (ffwb)              │
│              (ff-desktop / GUI shell)                     │
├─────────────────────────────────────────────────────────┤
│  ff-core │ ff-plugin │ ff-workflow │ lua-macro-engine    │
│  All editor subsystems │ All plugins                     │
├─────────────────────────────────────────────────────────┤
│               ff-command (this crate)                     │
│        Command registry, dispatch, shortcuts             │
├─────────────────────────────────────────────────────────┤
│               ff-logging (Wave 0 — diagnostics)          │
└─────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **Command-Driven Architecture (Req 4)**: ALL state-changing user operations route through `execute_command`
- **GUI Independence (Req 2)**: Zero GUI dependencies — no egui, no windowing imports
- **Plugin Architecture (Req 3)**: Plugins register commands via `PluginContext`
- **Keyboard Shortcut Registry (Req 10)**: Reserved shortcuts cannot be overridden; conflict detection at registration
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-command`
- **Error Message Standards (Req 8)**: All errors follow `[command] operation: description` format
- **Async I/O (Req 6)**: Supports async command execution via futures

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Invocation Sources
        A[Keyboard Shortcut]
        B[Menu / Toolbar]
        C[Command Palette]
        D[Lua Macro Script]
        E[Plugin Code]
        F[Command Line]
    end

    subgraph ff-command
        G[Shortcut Registry<br/>chord → Command_ID]
        H[Command Registry<br/>ID → CommandEntry]
        I[Command Dispatch<br/>execute_command]
        J[Context Builder<br/>ExecutionContext]
        K[Undo/Redo Bridge<br/>undo stack push]
        L[Command History<br/>bounded log]
        M[Scripting Bridge<br/>Lua ↔ Params/Result]
    end

    subgraph Downstream
        N[Undo/Redo Stack<br/>ff-undo-redo]
        O[Logging<br/>ff-logging]
        P[Application State]
    end

    A --> G
    G --> I
    B --> I
    C --> I
    D --> M
    M --> I
    E --> I
    F --> I
    I --> J
    I --> H
    I --> K
    I --> L
    K --> N
    I --> O
    I --> P
end
```

### Layer Placement

| Layer | Role |
|-------|------|
| **Invocation Layer** | Shortcut registry, scripting bridge, UI bindings — translates user intent into `execute_command` calls |
| **Dispatch Layer** | Validates command existence, evaluates enabled predicate, constructs `ExecutionContext`, routes to handler |
| **Execution Layer** | Command handler executes, produces `CommandResult` with optional `UndoRecord` |
| **Integration Layer** | Pushes undo records, logs history entries, emits diagnostics via `ff-logging` |

---

## 3. Module Structure

```
crates/ff-command/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API re-exports, crate docs
│   ├── id.rs               # CommandId newtype, validation, parsing
│   ├── params.rs           # CommandParams typed key-value map
│   ├── context.rs          # ExecutionContext construction and accessors
│   ├── result.rs           # CommandResult enum, UndoRecord trait object
│   ├── metadata.rs         # CommandMetadata struct, predicates
│   ├── handler.rs          # CommandHandler trait (sync + async variants)
│   ├── registry.rs         # CommandRegistry — concurrent map of CommandEntry
│   ├── dispatch.rs         # CommandDispatch — execute_command entry point
│   ├── undo_bridge.rs      # Undo/Redo integration, stack push/pop logic
│   ├── shortcut/
│   │   ├── mod.rs          # Re-exports for shortcut module
│   │   ├── chord.rs        # KeyChord, modifier keys, key codes
│   │   ├── sequence.rs     # Multi-key sequence, pending state, timeout
│   │   ├── registry.rs     # ShortcutRegistry — chord → CommandId mapping
│   │   ├── reserved.rs     # Reserved shortcut definitions (Req 10.1)
│   │   └── conflict.rs     # Conflict detection logic
│   ├── scripting.rs        # ScriptingBridge — Lua ↔ CommandParams/Result conversion
│   ├── history.rs          # CommandHistory — bounded, persistent log
│   ├── error.rs            # CommandError enum
│   └── builtin.rs          # Built-in commands: edit.undo, edit.redo
└── tests/
    ├── registry_tests.rs       # Registry property tests
    ├── dispatch_tests.rs       # Dispatch property tests
    ├── shortcut_tests.rs       # Shortcut conflict property tests
    ├── history_tests.rs        # History FIFO property tests
    ├── params_tests.rs         # Params conversion property tests
    └── integration.rs          # End-to-end command registration and execution
```

---

## 4. Key Data Models and Types

### CommandId

```rust
/// A validated command identifier. Non-empty UTF-8 string containing only
/// lowercase ASCII letters, digits, dots, and underscores.
/// Dot serves as namespace separator (e.g., "file.save", "edit.undo").
/// Addresses: Requirement 1, criterion 1
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(String);

impl CommandId {
    /// Attempts to create a CommandId from a string, validating format.
    /// Returns None if the string is empty or contains invalid characters.
    pub fn new(id: impl Into<String>) -> Option<Self>;

    /// Returns the category prefix (everything before the last dot).
    /// E.g., "file.save" → "file", "plugin.git.commit" → "plugin.git"
    pub fn category(&self) -> &str;

    /// Returns the full ID as a string slice.
    pub fn as_str(&self) -> &str;

    /// Returns true if this ID starts with the given prefix.
    pub fn has_prefix(&self, prefix: &str) -> bool;
}
```

### CommandParams

```rust
/// A typed key-value map of parameters passed to a command at execution time.
/// Supports string, integer, float, boolean, and nested map value types.
/// Addresses: Requirement 2, criterion 8
#[derive(Debug, Clone, Default)]
pub struct CommandParams {
    inner: HashMap<String, ParamValue>,
}

/// A single parameter value within CommandParams.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Map(HashMap<String, ParamValue>),
}

impl CommandParams {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<ParamValue>);
    pub fn get(&self, key: &str) -> Option<&ParamValue>;
    pub fn get_string(&self, key: &str) -> Option<&str>;
    pub fn get_integer(&self, key: &str) -> Option<i64>;
    pub fn get_float(&self, key: &str) -> Option<f64>;
    pub fn get_bool(&self, key: &str) -> Option<bool>;
    pub fn is_empty(&self) -> bool;
}
```

### ExecutionContext

```rust
/// The ambient state available to a command during execution.
/// Constructed by the dispatch layer before invoking the command handler.
/// Addresses: Requirement 2, criterion 3
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The URI of the currently active document (if any)
    pub active_document: Option<String>,
    /// Current cursor position (line, column) — 0-indexed
    pub cursor_position: Option<(usize, usize)>,
    /// Current selection range, if any: (start_line, start_col, end_line, end_col)
    pub selection: Option<(usize, usize, usize, usize)>,
    /// The identifier of the currently focused panel
    pub active_panel: Option<String>,
}
```

### CommandResult

```rust
/// The outcome of a command execution.
/// Addresses: Requirement 2, criteria 1/2/6; Requirement 4, criteria 1/2/4
#[derive(Debug)]
pub enum CommandResult {
    /// Command executed successfully with no return value.
    Ok,
    /// Command executed successfully with an undo record.
    OkUndoable {
        undo_record: Box<dyn UndoRecord>,
    },
    /// Command executed successfully with a return value (for scripting bridge).
    OkValue(ParamValue),
    /// Command executed successfully with both a return value and an undo record.
    OkValueUndoable {
        value: ParamValue,
        undo_record: Box<dyn UndoRecord>,
    },
    /// Command execution failed.
    Err(CommandError),
}

impl CommandResult {
    pub fn is_ok(&self) -> bool;
    pub fn is_err(&self) -> bool;
    pub fn undo_record(self) -> Option<Box<dyn UndoRecord>>;
    pub fn value(&self) -> Option<&ParamValue>;
}
```

### UndoRecord

```rust
/// An opaque token that encapsulates information needed to reverse a command's effect.
/// Implemented by each undoable command. The undo/redo system stores these.
/// Addresses: Requirement 4, criteria 1/2/5/6
pub trait UndoRecord: Send + Sync + std::fmt::Debug {
    /// Apply this record to reverse the original command's effect.
    fn undo(&self, ctx: &ExecutionContext) -> Result<(), CommandError>;

    /// Re-apply the original command's effect (for redo).
    fn redo(&self, ctx: &ExecutionContext) -> Result<(), CommandError>;

    /// Human-readable description for undo/redo history display.
    fn description(&self) -> &str;

    /// The command ID that produced this record.
    fn command_id(&self) -> &CommandId;
}
```

### CommandMetadata

```rust
/// Descriptive information attached to a registered command.
/// Addresses: Requirement 3, all criteria
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Human-readable display name (localizable)
    pub display_name: String,
    /// One-sentence description of what the command does
    pub description: String,
    /// Category derived from Command_ID prefix (e.g., "file", "edit")
    pub category: String,
    /// Optional default keyboard shortcut binding
    pub default_shortcut: Option<ShortcutBinding>,
    /// Optional icon asset reference string
    pub icon: Option<String>,
}
```

### CommandHandler

```rust
/// The execution trait for a command. Implementors define the command's behaviour.
/// Addresses: Requirement 2, criterion 1; Requirement 4, criterion 1
pub trait CommandHandler: Send + Sync {
    /// Whether this command is undoable. Checked at registration time.
    /// Addresses: Requirement 4, criterion 1
    fn is_undoable(&self) -> bool;

    /// Evaluates whether the command can currently execute given the context.
    /// Must complete within 1ms and produce no side effects.
    /// Addresses: Requirement 3, criterion 4/7
    fn is_enabled(&self, ctx: &ExecutionContext) -> bool { true }

    /// Evaluates whether the command should appear in menus and palettes.
    /// Must complete within 1ms and produce no side effects.
    /// Addresses: Requirement 3, criterion 5/7
    fn is_visible(&self, ctx: &ExecutionContext) -> bool { true }

    /// Execute the command synchronously.
    /// Addresses: Requirement 2, criterion 4 (sync path)
    fn execute(&self, ctx: &ExecutionContext, params: &CommandParams) -> CommandResult;
}

/// Async variant for commands that perform I/O or long-running operations.
/// Addresses: Requirement 2, criterion 4 (async path)
#[async_trait::async_trait]
pub trait AsyncCommandHandler: Send + Sync {
    fn is_undoable(&self) -> bool;
    fn is_enabled(&self, ctx: &ExecutionContext) -> bool { true }
    fn is_visible(&self, ctx: &ExecutionContext) -> bool { true }

    /// Execute the command asynchronously.
    async fn execute(&self, ctx: &ExecutionContext, params: &CommandParams) -> CommandResult;
}
```

### CommandEntry (internal)

```rust
/// A registered command: combines ID, metadata, and handler.
/// Stored in the CommandRegistry.
pub(crate) struct CommandEntry {
    pub id: CommandId,
    pub metadata: CommandMetadata,
    pub handler: CommandHandlerKind,
}

/// Supports both sync and async handlers.
pub(crate) enum CommandHandlerKind {
    Sync(Box<dyn CommandHandler>),
    Async(Box<dyn AsyncCommandHandler>),
}
```

### KeyChord and ShortcutBinding

```rust
/// Modifier keys for keyboard shortcuts.
/// Addresses: Requirement 5, criterion 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,  // Win/Cmd key
}

/// A single keyboard chord: modifiers + primary key.
/// Addresses: Requirement 5, criterion 1
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub modifiers: Modifiers,
    pub key: KeyCode,
}

/// A shortcut binding — either a single chord or a multi-key sequence.
/// Addresses: Requirement 5, criteria 1/2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShortcutBinding {
    /// Single chord (e.g., Ctrl+S)
    Single(KeyChord),
    /// Multi-key sequence (e.g., Ctrl+K, Ctrl+C)
    Sequence(KeyChord, KeyChord),
}

/// Platform-independent key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    // Special
    Tab, Space, Enter, Escape, Backspace, Delete,
    Home, End, PageUp, PageDown,
    Up, Down, Left, Right,
    Plus, Minus,
    // Punctuation
    Comma, Period, Semicolon, Slash, Backslash,
    LeftBracket, RightBracket, Grave, Equals,
}
```

### HistoryEntry

```rust
/// A single entry in the command history log.
/// Addresses: Requirement 7, criterion 1
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    /// The command that was executed
    pub command_id: String,
    /// UTC timestamp with millisecond precision
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Parameters that were passed to the command
    pub params: HashMap<String, serde_json::Value>,
}
```

---

## 5. Public API Surface

### Command Registry

```rust
/// The global, thread-safe command registry.
/// Addresses: Requirement 1, all criteria
pub struct CommandRegistry { /* ... */ }

impl CommandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self;

    /// Register a synchronous command with its metadata and handler.
    /// Returns Err if a command with the same ID already exists.
    /// Addresses: Requirement 1, criteria 2/3
    pub fn register(
        &self,
        id: CommandId,
        metadata: CommandMetadata,
        handler: Box<dyn CommandHandler>,
    ) -> Result<(), CommandError>;

    /// Register an asynchronous command.
    /// Returns Err if a command with the same ID already exists.
    pub fn register_async(
        &self,
        id: CommandId,
        metadata: CommandMetadata,
        handler: Box<dyn AsyncCommandHandler>,
    ) -> Result<(), CommandError>;

    /// Deregister a command by ID. Returns true if removed, false if not found.
    /// Addresses: Requirement 1, criterion 7
    pub fn deregister(&self, id: &CommandId) -> bool;

    /// Look up a command by ID. Returns None if not found.
    /// Addresses: Requirement 1, criterion 5
    pub fn get(&self, id: &CommandId) -> Option<CommandRef<'_>>;
```

### CommandRef

```rust
/// Guard type for accessing a registered command. Holds a read lock on the registry.
/// Derefs to the inner CommandEntry's metadata and handler.
/// Addresses: Requirement 1, criterion 5
pub struct CommandRef<'a> {
    guard: RwLockReadGuard<'a, HashMap<CommandId, CommandEntry>>,
    id: CommandId,
}

impl<'a> CommandRef<'a> {
    /// Access the command's metadata.
    pub fn metadata(&self) -> &CommandMetadata;

    /// Check if the command is enabled in the given context.
    pub fn is_enabled(&self, ctx: &ExecutionContext) -> bool;

    /// Check if the command is visible in the given context.
    pub fn is_visible(&self, ctx: &ExecutionContext) -> bool;
}
```

### CommandRegistry API (continued)

```rust
impl CommandRegistry {
    /// Query metadata for a command by ID without executing.
    /// Addresses: Requirement 3, criterion 6
    pub fn metadata(&self, id: &CommandId) -> Option<CommandMetadata>;

    /// List all registered command IDs.
    /// Addresses: Requirement 1, criterion 6
    pub fn list_all(&self) -> Vec<CommandId>;

    /// List commands whose ID starts with the given category prefix.
    /// Addresses: Requirement 1, criterion 6
    pub fn list_by_category(&self, prefix: &str) -> Vec<CommandId>;

    /// Returns the total number of registered commands.
    pub fn count(&self) -> usize;
}
```

### Command Dispatch

```rust
/// The single entry point for executing commands.
/// Addresses: Requirement 2, all criteria; Requirement 4, criteria 2–7
pub struct CommandDispatch { /* ... */ }

impl CommandDispatch {
    /// Create a new dispatch instance connected to the given registry.
    pub fn new(
        registry: Arc<CommandRegistry>,
        history: Arc<CommandHistory>,
    ) -> Self;

    /// Execute a command synchronously by ID with parameters.
    /// If the target command is registered with `AsyncCommandHandler`, this returns
    /// `CommandResult::Err` with a descriptive error. Callers that may invoke async
    /// commands should use `execute_command_async` instead.
    /// Addresses: Requirement 2, criteria 1/2/3/5/6/7
    pub fn execute_command(
        &self,
        id: &str,
        params: CommandParams,
    ) -> CommandResult;

    /// Execute a command asynchronously. This is the primary dispatch path for
    /// code running inside the Tokio runtime. Handles both sync and async handlers:
    /// sync handlers are called directly, async handlers are awaited.
    /// Addresses: Requirement 2, criterion 4
    pub async fn execute_command_async(
        &self,
        id: &str,
        params: CommandParams,
    ) -> CommandResult;

    /// Set the context provider — called by platform-core at startup.
    /// The provider is invoked before each command execution to build
    /// the current ExecutionContext.
    pub fn set_context_provider(
        &self,
        provider: Box<dyn ContextProvider>,
    );

    /// Set the undo stack manager for undo/redo integration.
    /// Called by platform-core after undo system initialization.
    pub fn set_undo_manager(
        &self,
        manager: Box<dyn UndoManager>,
    );
}

/// Trait for providing the current execution context.
/// Implemented by platform-core to inject application state.
pub trait ContextProvider: Send + Sync {
    fn current_context(&self) -> ExecutionContext;
}

/// Trait for managing undo/redo stacks.
/// Implemented by the undo-redo-transactions crate.
pub trait UndoManager: Send + Sync {
    fn push_undo(&self, record: Box<dyn UndoRecord>);
    fn pop_undo(&self) -> Option<Box<dyn UndoRecord>>;
    fn push_redo(&self, record: Box<dyn UndoRecord>);
    fn pop_redo(&self) -> Option<Box<dyn UndoRecord>>;
    fn clear_redo(&self);
}
```

### Shortcut Registry

```rust
/// The keyboard shortcut registry. Manages chord → CommandId mappings.
/// Addresses: Requirement 5, all criteria; Cross-cutting Requirement 10
pub struct ShortcutRegistry { /* ... */ }

impl ShortcutRegistry {
    /// Create a new registry pre-populated with reserved shortcuts.
    /// Addresses: Requirement 5, criterion 3
    pub fn new() -> Self;

    /// Register a shortcut binding for a command.
    /// Returns Err if the binding conflicts with an existing or reserved shortcut.
    /// Addresses: Requirement 5, criteria 4/5
    pub fn register(
        &self,
        binding: ShortcutBinding,
        command_id: CommandId,
    ) -> Result<(), CommandError>;

    /// Deregister a shortcut binding. Returns true if removed.
    pub fn deregister(&self, binding: &ShortcutBinding) -> bool;

    /// Resolve a single chord. Returns:
    /// - Some(CommandId) if the chord matches a single-chord binding
    /// - None if no match (check if it starts a multi-key sequence via `is_prefix`)
    pub fn resolve_chord(&self, chord: &KeyChord) -> Option<CommandId>;

    /// Resolve a two-chord sequence.
    /// Addresses: Requirement 5, criterion 2
    pub fn resolve_sequence(
        &self,
        first: &KeyChord,
        second: &KeyChord,
    ) -> Option<CommandId>;

    /// Returns true if the chord is the first part of any multi-key sequence.
    /// Used to enter pending state and wait for the second chord.
    pub fn is_prefix(&self, chord: &KeyChord) -> bool;

    /// Check if a binding is reserved (cannot be overridden).
    /// Addresses: Requirement 5, criterion 5
    pub fn is_reserved(&self, binding: &ShortcutBinding) -> bool;

    /// Load user-configurable shortcut overrides from TOML key map.
    /// Addresses: Requirement 5, criterion 6
    pub fn load_user_overrides(&self, keymap: &toml::Value) -> Vec<CommandError>;

    /// List all current bindings (for UI display, help system).
    pub fn list_all(&self) -> Vec<(ShortcutBinding, CommandId)>;

    /// Get the binding for a specific command, if any.
    pub fn binding_for(&self, command_id: &CommandId) -> Option<ShortcutBinding>;
}
```

### Scripting Bridge

```rust
/// The interface through which the Lua macro engine invokes commands.
/// Addresses: Requirement 6, all criteria
pub struct ScriptingBridge { /* ... */ }

impl ScriptingBridge {
    /// Create a new bridge connected to the command dispatch.
    pub fn new(dispatch: Arc<CommandDispatch>) -> Self;

    /// Execute a command from a Lua script.
    /// Converts Lua table → CommandParams, dispatches, converts result → Lua value.
    /// Addresses: Requirement 6, criteria 1/2/3/5
    pub fn execute(
        &self,
        command_id: &str,
        lua_params: LuaParams,
    ) -> Result<LuaValue, ScriptingError>;

    /// List all registered commands with metadata.
    /// Returns data suitable for conversion to a Lua table.
    /// Addresses: Requirement 6, criterion 6
    pub fn list_commands(&self) -> Vec<ScriptingCommandInfo>;
}

/// Lua-compatible parameter representation (converted from Lua tables).
#[derive(Debug, Clone)]
pub enum LuaParams {
    None,
    Table(HashMap<String, LuaValue>),
}

/// Lua-compatible value representation.
#[derive(Debug, Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Table(HashMap<String, LuaValue>),
}

/// Command info for scripting discovery.
#[derive(Debug, Clone)]
pub struct ScriptingCommandInfo {
    pub id: String,
    pub display_name: String,
    pub category: String,
    pub description: String,
}
```

### Command History

```rust
/// A bounded, persistent log of recently executed commands.
/// Addresses: Requirement 7, all criteria
pub struct CommandHistory { /* ... */ }

impl CommandHistory {
    /// Create a new history with the specified maximum depth.
    /// Addresses: Requirement 7, criterion 2
    pub fn new(max_depth: usize) -> Self;

    /// Create from configuration — reads `commands.history_depth` setting.
    /// Clamps values outside [10, 10000] and logs a WARN.
    /// Addresses: Requirement 7, criterion 3
    pub fn from_config(depth_value: Option<i64>) -> Self;

    /// Record a successfully executed command.
    /// Addresses: Requirement 7, criteria 1/4
    pub fn record(
        &self,
        command_id: &CommandId,
        params: &CommandParams,
    );

    /// Load persisted history from disk.
    /// Returns empty history on failure and logs a WARN.
    /// Addresses: Requirement 7, criteria 5/6
    pub fn load(path: &Path) -> Self;

    /// Persist current history to disk.
    /// Addresses: Requirement 7, criterion 5
    pub fn save(&self, path: &Path) -> Result<(), CommandError>;

    /// Retrieve the last N entries.
    /// Addresses: Requirement 7, criterion 8
    pub fn last_n(&self, n: usize) -> Vec<HistoryEntry>;

    /// Retrieve entries matching a command ID prefix.
    /// Addresses: Requirement 7, criterion 8
    pub fn by_prefix(&self, prefix: &str) -> Vec<HistoryEntry>;

    /// Retrieve entries within a time range.
    /// Addresses: Requirement 7, criterion 8
    pub fn by_time_range(
        &self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Vec<HistoryEntry>;

    /// Returns the current number of entries.
    pub fn len(&self) -> usize;

    /// Returns the configured maximum depth.
    pub fn max_depth(&self) -> usize;
}
```

---

## 6. Error Types

```rust
/// Errors produced by the command framework.
/// Addresses: Cross-cutting Requirement 8 (error format: "[command] operation: description")
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CommandError {
    /// Command ID is not registered in the registry.
    /// Addresses: Requirement 2, criterion 2
    #[error("[command] dispatch: command '{id}' is not registered")]
    NotFound { id: String },

    /// Command is currently disabled (enabled predicate returned false).
    /// Addresses: Requirement 2, criterion 5
    #[error("[command] dispatch: command '{id}' is not currently available")]
    Disabled { id: String },

    /// Duplicate command registration attempt.
    /// Addresses: Requirement 1, criterion 2
    #[error("[command] register: command '{id}' is already registered")]
    DuplicateId { id: String },

    /// Invalid command ID format.
    /// Addresses: Requirement 1, criterion 1
    #[error("[command] register: invalid command ID '{id}' — {reason}")]
    InvalidId { id: String, reason: String },

    /// Shortcut binding conflicts with an existing binding.
    /// Addresses: Requirement 5, criterion 4
    #[error("[command] shortcut: binding '{binding}' conflicts with existing command '{existing_id}'")]
    ShortcutConflict {
        binding: String,
        new_id: String,
        existing_id: String,
    },

    /// Shortcut binding conflicts with a reserved shortcut.
    /// Addresses: Requirement 5, criterion 5
    #[error("[command] shortcut: binding '{binding}' is reserved and cannot be overridden")]
    ShortcutReserved { binding: String },

    /// Command handler returned an execution error.
    /// Addresses: Requirement 2, criterion 6
    #[error("[command] execute '{id}': {description}")]
    ExecutionFailed { id: String, description: String },

    /// Undo operation failed.
    /// Addresses: Requirement 4, criterion 5
    #[error("[command] undo '{id}': {description}")]
    UndoFailed { id: String, description: String },

    /// Redo operation failed.
    /// Addresses: Requirement 4, criterion 6
    #[error("[command] redo '{id}': {description}")]
    RedoFailed { id: String, description: String },

    /// History persistence I/O error.
    /// Addresses: Requirement 7, criteria 5/6
    #[error("[command] history: {operation} failed — {source}")]
    HistoryIo {
        operation: String,
        source: std::io::Error,
    },

    /// Scripting bridge conversion error.
    /// Addresses: Requirement 6, criterion 5
    #[error("[command] scripting: {description}")]
    ScriptingError { description: String },
}

/// Error type for the scripting bridge (converted to Lua errors).
#[derive(Debug, thiserror::Error)]
pub enum ScriptingError {
    #[error("command '{id}' not found")]
    CommandNotFound { id: String },

    #[error("command '{id}' failed: {description}")]
    ExecutionFailed { id: String, description: String },

    #[error("parameter conversion failed: {description}")]
    ParamConversion { description: String },
}
```

---

## 7. Integration Points

### With `ff-logging` (upstream — Wave 0)

- `ff-command` uses `ff-logging` macros (`log_warn!`, `log_error!`, `log_info!`) for:
  - WARN when a command execution fails (Requirement 2, criterion 6)
  - WARN when history persistence fails at startup (Requirement 7, criterion 6)
  - WARN when `commands.history_depth` is clamped (Requirement 7, criterion 3)
  - INFO for command dispatch audit trail (debug builds)
- `ff-logging` is the only workspace crate dependency of `ff-command`

### With `ff-core` (platform-core — same wave, consumer)

- `ff-core` owns the `CommandRegistry` and `CommandDispatch` instances, creating them during startup
- `ff-core` implements `ContextProvider` to supply `ExecutionContext` from current application state
- `ff-core` registers built-in commands (edit.undo, edit.redo, file.save, etc.) during initialization
- `ff-core` calls `CommandHistory::load()` at startup and `CommandHistory::save()` at shutdown
- `ff-core` connects the shortcut registry to the GUI shell's key event stream

### With `ff-plugin` (plugin-architecture — same wave, consumer)

- `ff-plugin` provides `CommandRegistry` access through `PluginContext` so plugins can register commands during their `initialize` lifecycle phase (Requirement 1, criterion 3)
- `ff-plugin` calls `CommandRegistry::deregister()` during plugin `shutdown` to clean up plugin commands (Requirement 1, criterion 7)
- Plugins access `ShortcutRegistry` to register keyboard bindings (Requirement 5, criterion 8)

### With `undo-redo-transactions` (downstream — Wave 4)

- The `undo-redo-transactions` crate implements the `UndoManager` trait defined by `ff-command`
- `ff-command` pushes `UndoRecord` trait objects to the undo manager after undoable command execution (Requirement 4, criterion 2)
- The built-in `edit.undo` and `edit.redo` commands pop records from the undo manager (Requirement 4, criteria 5/6)
- Redo stack is cleared when a new undoable command executes (Requirement 4, criterion 7)

### With `lua-macro-engine` (downstream — Wave 10)

- `lua-macro-engine` uses `ScriptingBridge` to expose commands to Lua scripts
- The bridge converts Lua tables ↔ `CommandParams` and `CommandResult` ↔ Lua values (Requirement 6, criteria 2/3)
- `ScriptingBridge::list_commands()` provides runtime command discovery to scripts (Requirement 6, criterion 6)

### With `configuration-system` (same wave, provider)

- `configuration-system` provides the `commands.history_depth` setting (Requirement 7, criterion 2)
- `configuration-system` provides the key map TOML file for user shortcut overrides (Requirement 5, criterion 6)
- `ff-command` does NOT depend on `configuration-system` at the crate level — config values are passed in during initialization by `ff-core`

### With GUI Shell (`ff-desktop`)

- The GUI shell captures keyboard events and routes them to `ShortcutRegistry::resolve_chord()`
- On multi-key sequence prefix detection, the shell enters pending state with a 2-second timeout (Requirement 5, criterion 2)
- Menu items and command palette entries query `CommandRegistry::metadata()` and `CommandHandler::is_visible()` / `is_enabled()`

### Dependency Direction

```
ff-logging ← ff-command ← ff-core ← ff-plugin
                        ← ff-desktop (GUI shell)
                        ← lua-macro-engine
                        ← undo-redo-transactions (implements UndoManager)
                        ← all editor subsystems
```

`ff-command` depends only on `ff-logging`. All other crates consume `ff-command`.

---

## 8. Configuration

All configuration consumed by `ff-command` is provided through `ff-core` at initialization time. The crate does not directly read configuration files.

### Relevant Configuration Keys

```toml
[commands]
# Maximum number of history entries retained.
# Range: 10–10000. Values outside range are clamped with WARN.
# Default: 500
# Addresses: Requirement 7, criteria 2/3
history_depth = 500

# Path to the history persistence file.
# Default: platform data directory / "command_history.json"
history_file = "command_history.json"
```

```toml
[keymap]
# User shortcut overrides. Each key is a shortcut chord description,
# each value is a Command_ID.
# Addresses: Requirement 5, criterion 6
"Ctrl+B" = "edit.toggle_bold"
"Ctrl+K Ctrl+C" = "edit.comment_line"
"F5" = "debug.run"
```

### Reserved Shortcuts (Hardcoded)

The following bindings are populated into `ShortcutRegistry` at construction and reject override attempts:

| Binding | Command_ID | Source |
|---------|-----------|--------|
| F1 | `help.show` | Cross-cutting Req 10.1 |
| Ctrl+Plus | `view.zoom_in` | Cross-cutting Req 10.1 |
| Ctrl+Minus | `view.zoom_out` | Cross-cutting Req 10.1 |
| Ctrl+0 | `view.zoom_reset` | Cross-cutting Req 10.1 |
| Ctrl+Z | `edit.undo` | Cross-cutting Req 10.1 |
| Ctrl+Y | `edit.redo` | Cross-cutting Req 10.1 |
| Ctrl+Shift+Z | `edit.redo` | Cross-cutting Req 10.1 |
| Ctrl+C | `edit.copy` | Cross-cutting Req 10.1 |
| Ctrl+X | `edit.cut` | Cross-cutting Req 10.1 |
| Ctrl+V | `edit.paste` | Cross-cutting Req 10.1 |
| Ctrl+A | `edit.select_all` | Cross-cutting Req 10.1 |
| Ctrl+S | `file.save` | Cross-cutting Req 10.1 |
| Ctrl+F | `find.focus` | Cross-cutting Req 10.1 |
| Ctrl+H | `find.change` | Cross-cutting Req 10.1 |
| Ctrl+G | `navigate.goto_line` | Cross-cutting Req 10.1 |
| Ctrl+Tab | `tab.next` | Cross-cutting Req 10.1 |
| Ctrl+Shift+Tab | `tab.previous` | Cross-cutting Req 10.1 |
| Ctrl+W | `tab.close` | Cross-cutting Req 10.1 |
| Ctrl+N | `tab.new` | Cross-cutting Req 10.1 |
| Ctrl+Shift+D | `layout.dock_toggle` | Cross-cutting Req 10.1 |
| Ctrl+Shift+T | `layout.tab_undock` | Cross-cutting Req 10.1 |

---

## 9. Concurrency Model

### Thread-Safety Approach

| Component | Mechanism | Rationale |
|-----------|-----------|-----------|
| CommandRegistry | `RwLock<HashMap<CommandId, CommandEntry>>` | Many readers (lookups), rare writers (registration at startup/plugin load) |
| ShortcutRegistry | `RwLock<HashMap<ShortcutBinding, CommandId>>` | Same access pattern as command registry |
| CommandHistory | `Mutex<VecDeque<HistoryEntry>>` | Write-heavy (every command records), short critical sections |
| CommandDispatch | Immutable references to registry + history (via `Arc`) | Dispatch itself is stateless; delegates to thread-safe components |
| ExecutionContext | Constructed per-call, owned by caller | No sharing — fresh context per dispatch |
| UndoManager trait | Implementor guarantees thread safety (`Send + Sync` bound) | Allows undo stack to use its own locking strategy |

### Multi-Key Sequence Pending State

The shortcut pending state (waiting for second chord after first chord of a multi-key sequence) is managed by the GUI shell, NOT by `ff-command`. The shell:

1. Receives a key chord
2. Calls `ShortcutRegistry::resolve_chord()` — if `Some`, dispatches immediately
3. If `None`, calls `ShortcutRegistry::is_prefix()` — if `true`, enters pending state
4. In pending state, waits up to 2 seconds for the next chord (Requirement 5, criterion 2)
5. On second chord: calls `resolve_sequence(first, second)` — dispatches if `Some`, cancels if `None`
6. On timeout: reverts to no pending state, no command executed

This design keeps `ff-command` free of timer/async runtime dependencies for the shortcut system.

### Async Command Execution

- Async commands (`AsyncCommandHandler`) are awaited by the caller (GUI shell or scripting bridge)
- The dispatch layer does NOT spawn tasks — it returns a future that the caller drives
- This avoids coupling to a specific async runtime and allows the caller to manage cancellation
- For the scripting bridge, async commands are block-on'd within the Lua execution context

### Lock Ordering

To prevent deadlocks, locks are always acquired in this order:

1. `CommandRegistry` (read)
2. `ShortcutRegistry` (read)
3. `CommandHistory` (write)
4. `UndoManager` (write)

No operation ever acquires locks in reverse order.

---

## 10. Correctness Properties

These properties are suitable for property-based testing with `proptest`. They validate invariants that must hold across all valid inputs.

### Property 1: CommandId Validation Round-Trip

**Statement**: For any string that passes `CommandId::new()` validation (returns `Some`), the resulting ID's `as_str()` is equal to the original string. For any string containing invalid characters (uppercase, spaces, special characters beyond dot/underscore), `CommandId::new()` returns `None`.

**Validates**: Requirement 1, criterion 1

```rust
// proptest strategy: generate arbitrary strings, partition into valid/invalid
// assertion: valid IDs round-trip; invalid IDs are rejected
```

### Property 2: Registry Duplicate Rejection

**Statement**: For any sequence of `(CommandId, handler)` registration attempts, if the same `CommandId` appears more than once, only the first registration succeeds and subsequent attempts return `Err(DuplicateId)`. The registry contains exactly one entry per unique ID.

**Validates**: Requirement 1, criterion 2

```rust
// proptest strategy: generate Vec<CommandId> with possible duplicates
// assertion: first registration of each ID succeeds, duplicates fail, count == unique IDs
```

### Property 3: Shortcut Conflict Detection

**Statement**: For any set of `(ShortcutBinding, CommandId)` registration attempts, if two different `CommandId`s attempt to bind the same `ShortcutBinding`, the second registration returns `Err(ShortcutConflict)`. No two commands can share the same binding.

**Validates**: Requirement 5, criterion 4

```rust
// proptest strategy: generate Vec<(ShortcutBinding, CommandId)> with possible binding collisions
// assertion: at most one command per binding; conflicts detected and rejected
```

### Property 4: Reserved Shortcut Immutability

**Statement**: For any `ShortcutBinding` that is in the reserved set, any registration attempt (regardless of the `CommandId`) returns `Err(ShortcutReserved)`. Reserved shortcuts cannot be overridden by user configuration or plugin registration.

**Validates**: Requirement 5, criteria 3/5; Cross-cutting Requirement 10

```rust
// proptest strategy: generate arbitrary CommandIds and attempt to register reserved bindings
// assertion: all attempts fail with ShortcutReserved
```

### Property 5: History FIFO Eviction

**Statement**: For any `max_depth` in [10, 10000] and any sequence of N command recordings where N > max_depth, the history always contains exactly `max_depth` entries, and those entries are the most recent `max_depth` commands in chronological order.

**Validates**: Requirement 7, criteria 2/4

```rust
// proptest strategy: generate max_depth in [10, 100], generate N > max_depth recordings
// assertion: history.len() == max_depth, entries are the last max_depth recordings in order
```

### Property 6: History Depth Clamping

**Statement**: For any integer value for `commands.history_depth`, the effective depth is always within [10, 10000]. Values < 10 are clamped to 10, values > 10000 are clamped to 10000, and values within range are unchanged.

**Validates**: Requirement 7, criterion 3

```rust
// proptest strategy: generate i64 values in full range
// assertion: effective_depth ∈ [10, 10000] ∧ (input ∈ [10, 10000] → output == input)
```

### Property 7: Dispatch Routes All Sources Identically

**Statement**: For any registered command and any valid `CommandParams`, executing the command through `execute_command` produces the same `CommandResult` regardless of the invocation source (direct call, shortcut resolution, scripting bridge). The command handler is invoked exactly once with the same context and params.

**Validates**: Requirement 2, criteria 1/7; Requirement 6, criterion 2

```rust
// proptest strategy: generate command + params, invoke via multiple paths
// assertion: handler called exactly once per dispatch; results equivalent
```

### Property 8: Undo/Redo Stack Consistency

**Statement**: For any sequence of undoable command executions and undo/redo operations, the undo stack depth equals the number of undoable commands executed minus the number of undo operations (clamped at 0), and executing a new undoable command after an undo clears the redo stack.

**Validates**: Requirement 4, criteria 2/5/6/7

```rust
// proptest strategy: generate sequence of (execute, undo, redo) operations
// assertion: stack depths are consistent; redo stack cleared on new command after undo
```

### Property 9: Disabled Command Rejection

**Statement**: For any command whose `is_enabled()` predicate returns `false` given the current `ExecutionContext`, `execute_command` returns `Err(Disabled)` without invoking the command handler and without modifying the undo stack or command history.

**Validates**: Requirement 2, criterion 5; Requirement 4, criterion 4

```rust
// proptest strategy: generate commands with varying enabled predicates and contexts
// assertion: disabled commands produce Err(Disabled), no side effects
```

### Property 10: Category Prefix Query Completeness

**Statement**: For any set of registered commands, `list_by_category(prefix)` returns exactly those commands whose `CommandId` starts with `prefix + "."`. No command matching the prefix is omitted, and no command not matching the prefix is included.

**Validates**: Requirement 1, criterion 6

```rust
// proptest strategy: generate set of CommandIds, pick random prefix
// assertion: result set == { id | id.starts_with(prefix + ".") }
```

---

## Appendix A: External Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `chrono` | 0.4 | UTC timestamps for history entries |
| `serde` | 1.0 | Serialization for history persistence |
| `serde_json` | 1.0 | JSON format for history file |
| `thiserror` | 2.0 | Error type derivation |
| `toml` | 0.8 | Parsing user key map overrides (behind `config` feature) |
| `async-trait` | 0.1 | Async trait support for `AsyncCommandHandler` |
| `proptest` | 1.0 | Property-based testing (dev-dependency only) |

## Appendix B: Built-In Commands

The following commands are registered by `ff-command` itself (in `builtin.rs`):

| Command_ID | Display Name | Undoable | Description |
|-----------|-------------|----------|-------------|
| `edit.undo` | Undo | No | Pops and applies the top undo record |
| `edit.redo` | Redo | No | Pops and applies the top redo record |

All other commands (file.save, edit.copy, etc.) are registered by `ff-core` or plugins during initialization. The command framework only provides the built-in undo/redo commands because they directly interact with the undo manager.

## Appendix C: Command ID Naming Convention

Command IDs follow a dot-separated namespace convention:

```
<category>.<action>          — e.g., "file.save", "edit.copy"
<category>.<sub>.<action>    — e.g., "plugin.git.commit", "view.panel.toggle"
```

Rules:
- Only lowercase ASCII letters `[a-z]`, digits `[0-9]`, dots `.`, and underscores `_`
- Must not be empty
- Must not start or end with a dot
- Must not contain consecutive dots
- Dot is the namespace separator; category prefix is derived from first segment

## Appendix D: Multi-Key Sequence Timeout Behaviour

When the GUI shell detects a first chord that is a prefix of a multi-key sequence:

1. Shell displays a visual indicator (e.g., status bar shows "Ctrl+K ..." awaiting second key)
2. A 2-second timer starts
3. If a valid second chord arrives → sequence resolves → command dispatched
4. If an invalid second chord arrives → pending state cancelled, chord processed normally
5. If timeout expires → pending state cancelled, no command executed
6. Escape key always cancels pending state immediately

The 2-second timeout value is defined as a constant in `ff-command::shortcut::sequence::SEQUENCE_TIMEOUT_MS = 2000`.
