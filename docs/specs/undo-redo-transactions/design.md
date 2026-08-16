# Design Document: Undo/Redo Transactions (`ff-undo-redo`)

## 1. Overview

The `ff-undo-redo` crate implements the **full transaction system** for undo and redo in FileForgeWorkbench. It owns the undo and redo stacks, manages transaction boundaries and coalescing, tracks save-point semantics for the dirty flag, supports bulk transaction optimisations, provides tentative action support for IME composition, manages selection history for cursor restoration on undo/redo, and persists undo state for crash recovery.

This crate is the bridge between the command framework (`ff-command`) — which produces undo records — and the document model (`ff-document-model`) — which receives reversed/re-applied edit operations.

### Position in Architecture

```
Wave 4 — Core Editor

┌─────────────────────────────────────────────────────────┐
│         Application Binary (ffwb / ff-desktop)           │
├─────────────────────────────────────────────────────────┤
│  ff-core │ ff-plugin │ All editor subsystems             │
├─────────────────────────────────────────────────────────┤
│  ff-command (Command Framework — Wave 2)                 │
│    ↕ UndoManager trait                                   │
│  ff-undo-redo (this crate — Wave 4)                      │
│    ↕ EditTarget trait                                    │
│  ff-document-model (Document Model — Wave 4)             │
├─────────────────────────────────────────────────────────┤
│  ff-logging (Wave 0) │ ff-configuration (Wave 2)         │
└─────────────────────────────────────────────────────────┘
```


### Design Constraints (Cross-Cutting)

- **Command-Driven Architecture (Req 4)**: Every undoable command produces a transaction record via this crate
- **GUI Independence (Req 2)**: Zero GUI dependencies — pure data structures and logic
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-undo-redo`
- **Error Message Standards (Req 8)**: All errors follow `[undo] operation: description` format
- **Async I/O (Req 6)**: Bulk operations > 1s run async with progress; recovery file I/O is async

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Invocation Sources
        A[Command Framework<br/>ff-command]
        B[Edit Operations<br/>ff-edit-ops]
        C[Macro Engine<br/>lua scripts]
    end

    subgraph ff-undo-redo
        D[UndoManager<br/>per-document orchestrator]
        E[Transaction Builder<br/>begin/end/abort]
        F[Undo Stack<br/>bounded VecDeque]
        G[Redo Stack<br/>Vec of Transaction]
        H[Coalescing Engine<br/>merge rules + timeout]
        I[Save Point Tracker<br/>dirty flag derivation]
        J[Selection History<br/>before/after states]
        K[Scrap Stack<br/>contiguous text buffer]
        L[Bulk Transaction<br/>Rule + Index variants]
        M[Tentative Actions<br/>IME composition]
        N[Recovery Writer<br/>periodic persistence]
        O[Validation<br/>integrity checking]
    end

    subgraph Downstream
        P[Document Model<br/>ff-document-model]
        Q[Logging<br/>ff-logging]
        R[Configuration<br/>ff-configuration]
        S[File System<br/>recovery file I/O]
    end

    A -->|push UndoRecord| D
    B -->|edit operations| E
    C -->|grouped edits| E
    D --> E
    D --> F
    D --> G
    D --> H
    D --> I
    D --> J
    D --> K
    D --> L
    D --> M
    D --> N
    D --> O
    E -->|commit| F
    F -->|undo pop| G
    G -->|redo pop| F
    N --> S
    D --> Q
    R -->|config values| D
    D -->|apply/reverse ops| P
end
```


### Layer Responsibilities

| Layer | Role |
|-------|------|
| **Transaction Builder** | Manages open transactions, nesting depth, accumulates edit operations, handles abort/rollback |
| **Coalescing Engine** | Evaluates whether consecutive single-char operations should merge into the current transaction |
| **Stack Management** | Bounded undo stack (FIFO eviction), redo stack (cleared on new commit), per-document isolation |
| **Save Point Tracker** | Maintains save/detach markers, derives dirty flag, tracks modified line markers |
| **Selection History** | Captures before/after selection state per transaction, restores on undo/redo |
| **Bulk Transaction** | Optimised storage for Rule_Transaction (O(1)) and Index_Transaction (O(n)) patterns |
| **Tentative Actions** | IME composition support — uncommitted actions that can be rolled back without undo history trace |
| **Recovery Writer** | Periodic serialisation of undo state to disk for crash recovery |
| **Validation** | Integrity checking of undo history against document state |

---

## 3. Module Structure

```
crates/ff-undo-redo/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── manager.rs              # UndoManager — per-document orchestrator
│   ├── transaction.rs          # Transaction struct, TransactionBuilder
│   ├── edit_op.rs              # EditOperation enum (Insert, Delete, Replace)
│   ├── stack.rs                # UndoStack, RedoStack — bounded collections
│   ├── coalesce.rs             # Coalescing engine — merge rules, timeout
│   ├── save_point.rs           # SavePointTracker, DetachPoint, DirtyFlag
│   ├── selection.rs            # SelectionState, SelectionHistory
│   ├── bulk.rs                 # BulkTransaction, RuleTransaction, IndexTransaction
│   ├── tentative.rs            # TentativeActionManager — IME composition
│   ├── recovery.rs             # RecoveryWriter, RecoveryReader — crash recovery
│   ├── scrap.rs                # ScrapStack — contiguous text storage
│   ├── record_id.rs            # LogicalRecordId, RecordIdMap
│   ├── container.rs            # ContainerAction, UndoableState trait
│   ├── validate.rs             # History validation and integrity checks
│   ├── notify.rs               # UndoNotifier trait — state-change callbacks
│   ├── config.rs               # UndoConfig — parsed configuration values
│   ├── error.rs                # UndoError enum
│   └── undo_manager_trait.rs   # UndoManager trait impl (for ff-command)
└── tests/
    ├── stack_tests.rs          # Undo/redo stack property tests
    ├── coalesce_tests.rs       # Coalescing property tests
    ├── save_point_tests.rs     # Save point and dirty flag property tests
    ├── transaction_tests.rs    # Transaction builder property tests
    ├── bulk_tests.rs           # Bulk transaction property tests
    ├── tentative_tests.rs      # IME tentative action tests
    ├── recovery_tests.rs       # Recovery file round-trip tests
    ├── selection_tests.rs      # Selection history tests
    ├── validate_tests.rs       # Validation property tests
    └── integration.rs          # End-to-end undo/redo scenarios
```


---

## 4. Key Data Models and Types

### EditOperation

```rust
/// A single atomic change to the document buffer.
/// Addresses: Requirement 3.1, Requirement 17
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOperation {
    /// Insert text at a byte position.
    Insert {
        position: u64,
        /// Length of inserted text (actual bytes stored in ScrapStack)
        length: u32,
    },
    /// Delete text at a byte position.
    Delete {
        position: u64,
        /// Length of deleted text (actual bytes stored in ScrapStack)
        length: u32,
    },
    /// Replace text at a byte position (delete + insert as atomic unit).
    Replace {
        position: u64,
        old_length: u32,
        new_length: u32,
    },
}
```

### Transaction

```rust
/// A named, atomic unit of work in the undo history.
/// Contains one or more EditOperations that are applied/reversed together.
/// Addresses: Requirement 3.6, Requirement 4
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Human-readable description (e.g., "Delete line 42")
    pub name: String,
    /// UTC timestamp when the transaction was committed
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Ordered list of edit operations in this transaction
    pub operations: Vec<EditOperation>,
    /// Container actions interleaved with edit operations
    pub container_actions: Vec<(usize, Box<dyn UndoableState>)>,
    /// Selection state before the transaction
    pub selection_before: Option<SelectionState>,
    /// Selection state after the transaction
    pub selection_after: Option<SelectionState>,
    /// Whether this transaction may be coalesced with the next
    pub may_coalesce: bool,
    /// Scrap stack offset for this transaction's text data
    pub scrap_offset: u64,
    /// Total bytes in scrap stack consumed by this transaction
    pub scrap_length: u64,
}
```


### BulkTransaction

```rust
/// Optimised storage for multi-edit operations.
/// Addresses: Requirement 7
#[derive(Debug, Clone)]
pub enum BulkTransaction {
    /// O(1) memory — stores the transformation rule; re-scans on undo.
    /// Addresses: Requirement 7.3, 7.6, 7.8
    Rule(RuleTransaction),
    /// O(n) memory — stores rule + list of affected record IDs.
    /// Addresses: Requirement 7.4, 7.5, 7.7, 7.8
    Index(IndexTransaction),
}

/// A rule-based bulk transaction. Scope is deterministic and re-scannable.
#[derive(Debug, Clone)]
pub struct RuleTransaction {
    /// The transformation rule (e.g., pattern + replacement)
    pub rule: TransformRule,
    /// Scope description for re-scanning on undo
    pub scope: BulkScope,
    /// Human-readable name
    pub name: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Selection states
    pub selection_before: Option<SelectionState>,
    pub selection_after: Option<SelectionState>,
}

/// An index-based bulk transaction. Materialises affected record IDs.
#[derive(Debug, Clone)]
pub struct IndexTransaction {
    /// The transformation rule
    pub rule: TransformRule,
    /// The logical record IDs of affected records
    pub affected_records: Vec<LogicalRecordId>,
    /// Human-readable name
    pub name: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Selection states
    pub selection_before: Option<SelectionState>,
    pub selection_after: Option<SelectionState>,
}

/// The transformation applied by a bulk operation.
#[derive(Debug, Clone)]
pub struct TransformRule {
    /// Pattern to match (regex or literal)
    pub pattern: String,
    /// Replacement string
    pub replacement: String,
    /// Whether match is case-sensitive
    pub case_sensitive: bool,
    /// Additional rule metadata
    pub metadata: HashMap<String, String>,
}

/// Scope of a bulk operation — determines Rule vs Index strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulkScope {
    /// All records — deterministic, use RuleTransaction
    All,
    /// Explicit line range — deterministic
    Range { start: u64, end: u64 },
    /// CC block — deterministic from command context
    Block { start: u64, end: u64 },
    /// Visible/non-excluded — transient, use IndexTransaction
    Visible,
    /// Excluded only — transient
    Excluded,
    /// Tagged records — transient
    Tagged,
    /// Filtered by criteria — transient
    Filtered,
}
```


### SelectionState

```rust
/// The cursor/selection state at a point in time.
/// Stored with each transaction for restoration on undo/redo.
/// Addresses: Requirement 9
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionState {
    /// All active caret positions (supports multi-caret)
    pub carets: Vec<CaretPosition>,
    /// The selection type at this point
    pub selection_type: SelectionType,
}

/// A single caret position with optional anchor for selection.
#[derive(Debug, Clone, PartialEq)]
pub struct CaretPosition {
    /// Caret byte position in document
    pub position: u64,
    /// Anchor byte position (for selection range; equals position if no selection)
    pub anchor: u64,
    /// Virtual space offset beyond line end
    pub virtual_space: u32,
    /// Anchor virtual space
    pub anchor_virtual_space: u32,
}

/// Type of selection active at capture time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    Stream,
    Rectangular,
    Line,
    None,
}
```

### LogicalRecordId

```rust
/// A stable identifier for a document record (line).
/// Assigned at file-open, invariant under insertions/deletions of other records.
/// Addresses: Requirement 14
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LogicalRecordId(pub u64);

/// Maps logical record IDs to current byte offsets.
/// Updated on every document modification.
/// Addresses: Requirement 14.4
pub struct RecordIdMap {
    /// Next ID to assign
    next_id: u64,
    /// Active mapping: record ID → current byte offset
    id_to_offset: HashMap<LogicalRecordId, u64>,
    /// Reverse mapping for lookup: byte offset → record ID (line starts only)
    offset_to_id: BTreeMap<u64, LogicalRecordId>,
}
```


### ScrapStack

```rust
/// Contiguous byte buffer storing all text data for undo history.
/// Minimises allocation overhead and cache misses.
/// Addresses: Requirement 17
pub struct ScrapStack {
    /// The contiguous byte buffer
    buffer: Vec<u8>,
    /// Current position pointer (advances on push, retreats on undo traversal)
    position: u64,
}
```

### UndoConfig

```rust
/// Parsed configuration values for the undo system.
/// Addresses: Requirements 1.3, 6.4, 8.2, 9.7
#[derive(Debug, Clone)]
pub struct UndoConfig {
    /// Maximum undo stack depth. Range: [0, 10000]. Default: 100.
    pub max_levels: u32,
    /// Coalesce timeout in milliseconds. Range: [100, 10000]. Default: 2000.
    pub coalesce_timeout_ms: u32,
    /// Whether selection history is enabled. Default: true.
    pub selection_history_enabled: bool,
    /// Recovery file write interval in seconds. 0 = disabled. Default: 60.
    pub recovery_interval_seconds: u32,
}
```

### CoalesceState

```rust
/// Tracks the current coalescing state for a document session.
/// Addresses: Requirement 6
#[derive(Debug)]
pub struct CoalesceState {
    /// Whether coalescing is currently active (accumulating into existing txn)
    pub active: bool,
    /// The type of the last operation (for type-change detection)
    pub last_op_type: Option<CoalesceOpType>,
    /// The end position of the last operation (for contiguity detection)
    pub last_end_position: Option<u64>,
    /// Timestamp of the last operation (for timeout detection)
    pub last_timestamp: Option<std::time::Instant>,
    /// Whether the current in-progress transaction is marked may_coalesce
    pub may_coalesce: bool,
}

/// Operation types for coalescing boundary detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoalesceOpType {
    /// Single character insert
    CharInsert,
    /// Single character delete (backspace — position moves backward)
    CharBackspace,
    /// Single character delete (delete key — position stays)
    CharDelete,
}
```


### TentativeState

```rust
/// Manages IME composition tentative actions.
/// Addresses: Requirement 12
#[derive(Debug)]
pub struct TentativeState {
    /// Whether tentative mode is active
    pub active: bool,
    /// The action index where tentative mode began
    pub tentative_point: Option<usize>,
    /// Number of tentative steps since the tentative point
    pub step_count: usize,
}
```

### SavePointState

```rust
/// Tracks the save point and detach point for dirty flag derivation.
/// Addresses: Requirement 5
#[derive(Debug)]
pub struct SavePointState {
    /// The action index corresponding to the last save (or file open).
    /// None if no save point has been set.
    pub save_point: Option<usize>,
    /// The detach point — set when the save point becomes unreachable
    /// due to redo history truncation.
    pub detach_point: Option<usize>,
    /// Current action index (undo position)
    pub current_action: usize,
}
```

---

## 5. Public API Surface

### UndoManager (per-document orchestrator)

```rust
/// The primary public type — one instance per open document.
/// Encapsulates all undo/redo state for a single document session.
/// Addresses: Requirement 18.2
pub struct DocumentUndoManager { /* ... */ }

impl DocumentUndoManager {
    /// Create a new undo manager with the given configuration.
    pub fn new(config: UndoConfig) -> Self;

    // --- Transaction API (Requirement 3) ---

    /// Begin an explicit transaction group. Nested calls increment depth.
    /// Addresses: Requirement 3.2, 3.3
    pub fn begin_transaction(&mut self, name: &str);

    /// End an explicit transaction group. Only outermost call commits.
    /// Addresses: Requirement 3.2, 3.3
    pub fn end_transaction(&mut self);

    /// Abort the current transaction, rolling back all operations.
    /// Addresses: Requirement 3.4
    pub fn abort_transaction(&mut self);

    /// Returns the current transaction nesting depth (0 = no active txn).
    /// Addresses: Requirement 3.7
    pub fn transaction_depth(&self) -> usize;

    // --- Edit Recording ---

    /// Record an insert operation. Text data is pushed to ScrapStack.
    /// Addresses: Requirement 1.2, 17.3
    pub fn record_insert(&mut self, position: u64, text: &[u8]);

    /// Record a delete operation. Deleted text is pushed to ScrapStack.
    /// Addresses: Requirement 1.2, 17.3
    pub fn record_delete(&mut self, position: u64, text: &[u8]);

    /// Record a replace operation.
    pub fn record_replace(
        &mut self,
        position: u64,
        old_text: &[u8],
        new_text: &[u8],
    );

    /// Record a container action (plugin/extension state).
    /// Addresses: Requirement 13
    pub fn record_container_action(
        &mut self,
        action: Box<dyn UndoableState>,
    );

    // --- Undo/Redo Execution (Requirement 4) ---

    /// Execute a single undo operation.
    /// Returns the reversed transaction for display/logging, or None if empty.
    /// Addresses: Requirement 4.1, 4.2
    pub fn undo(&mut self) -> Result<Option<&Transaction>, UndoError>;

    /// Execute N successive undo operations.
    /// Returns the count actually undone.
    /// Addresses: Requirement 4.6
    pub fn undo_n(&mut self, count: usize) -> Result<usize, UndoError>;

    /// Execute a single redo operation.
    /// Addresses: Requirement 4.4, 4.5
    pub fn redo(&mut self) -> Result<Option<&Transaction>, UndoError>;

    /// Execute N successive redo operations.
    /// Addresses: Requirement 4.7
    pub fn redo_n(&mut self, count: usize) -> Result<usize, UndoError>;

    /// Check whether undo is available.
    pub fn can_undo(&self) -> bool;

    /// Check whether redo is available.
    pub fn can_redo(&self) -> bool;

    /// Get the description of the next undo transaction (for UI display).
    pub fn undo_description(&self) -> Option<&str>;

    /// Get the description of the next redo transaction (for UI display).
    pub fn redo_description(&self) -> Option<&str>;

    // --- Save Point (Requirement 5) ---

    /// Mark the current undo position as the save point.
    /// Addresses: Requirement 5.2
    pub fn set_save_point(&mut self);

    /// Returns true if the current position is at the save point.
    /// Addresses: Requirement 5.6
    pub fn is_at_save_point(&self) -> bool;

    /// Returns true if the current position is before the save point.
    /// Addresses: Requirement 5.6
    pub fn before_save_point(&self) -> bool;

    /// Returns true if the current position is after the save point.
    /// Addresses: Requirement 5.6
    pub fn after_save_point(&self) -> bool;

    /// Returns true if the save point is unreachable (detached).
    /// Addresses: Requirement 5.6
    pub fn after_detach_point(&self) -> bool;

    /// Returns the current dirty flag state.
    /// Addresses: Requirement 5.3
    pub fn is_dirty(&self) -> bool;

    // --- Coalescing (Requirement 6) ---

    /// Notify the undo manager that the coalesce timeout has elapsed.
    /// Closes the current coalescing window.
    /// Addresses: Requirement 6.4
    pub fn coalesce_timeout_expired(&mut self);

    /// Force a coalescing boundary (e.g., cursor moved, operation type changed).
    /// Addresses: Requirement 6.3
    pub fn break_coalesce(&mut self);

    // --- Tentative Actions / IME (Requirement 12) ---

    /// Enter tentative mode for IME composition.
    /// Addresses: Requirement 12.2
    pub fn tentative_start(&mut self);

    /// Commit tentative actions — they become permanent history.
    /// Addresses: Requirement 12.3
    pub fn tentative_commit(&mut self);

    /// Roll back all tentative actions, restoring pre-composition state.
    /// Addresses: Requirement 12.4
    pub fn tentative_rollback(&mut self);

    /// Query whether tentative mode is active.
    /// Addresses: Requirement 12.5
    pub fn tentative_active(&self) -> bool;

    /// Number of actions since the tentative point.
    /// Addresses: Requirement 12.5
    pub fn tentative_steps(&self) -> Option<usize>;

    // --- Bulk Transactions (Requirement 7) ---

    /// Begin a bulk transaction with the given rule and scope.
    /// Automatically selects Rule or Index strategy based on scope.
    /// Addresses: Requirement 7.2, 7.3, 7.4
    pub fn begin_bulk_transaction(
        &mut self,
        name: &str,
        rule: TransformRule,
        scope: BulkScope,
    );

    /// Record an affected logical record ID for Index transactions.
    /// Addresses: Requirement 7.5
    pub fn record_bulk_affected(&mut self, record_id: LogicalRecordId);

    /// Commit the bulk transaction.
    pub fn end_bulk_transaction(&mut self);

    /// Cancel an in-progress bulk transaction, rolling back.
    /// Addresses: Requirement 7.10
    pub fn abort_bulk_transaction(&mut self);

    // --- Selection History (Requirement 9) ---

    /// Set the current selection state (called before/after transaction).
    /// Addresses: Requirement 9.1, 9.2
    pub fn set_selection_state(&mut self, state: SelectionState);

    // --- Recovery (Requirement 8) ---

    /// Serialize current undo state for recovery file.
    /// Addresses: Requirement 8.1, 8.7
    pub fn serialize_for_recovery(&self) -> Result<Vec<u8>, UndoError>;

    /// Restore undo state from recovery data.
    /// Validates integrity before accepting.
    /// Addresses: Requirement 8.5, 16.4
    pub fn restore_from_recovery(
        data: &[u8],
        document_length: u64,
        config: UndoConfig,
    ) -> Result<Self, UndoError>;

    // --- History Management ---

    /// Clear all undo/redo history and reset state.
    /// Addresses: Requirement 1.7
    pub fn delete_history(&mut self);

    /// Validate internal consistency against current document size.
    /// Addresses: Requirement 16.1, 16.2
    pub fn validate(&self, document_length: u64) -> bool;

    /// Get the current undo stack depth.
    pub fn undo_depth(&self) -> usize;

    /// Get the current redo stack depth.
    pub fn redo_depth(&self) -> usize;

    /// Get the configured max levels.
    pub fn max_levels(&self) -> u32;

    /// Check if undo is disabled (max_levels == 0).
    /// Addresses: Requirement 1.5
    pub fn is_undo_disabled(&self) -> bool;

    // --- Notifications ---

    /// Register a notification listener.
    /// Addresses: Requirement 18.3
    pub fn add_listener(&mut self, listener: Box<dyn UndoNotifier>);

    /// Remove a notification listener.
    pub fn remove_listener(&mut self, id: ListenerId);
}
```


### UndoManager Trait Implementation (for ff-command)

```rust
/// Implements the UndoManager trait defined by ff-command.
/// This is the bridge between the command framework and the per-document undo stacks.
/// Addresses: Requirement 11.5, 15, 18.2
///
/// The trait is defined in ff-command as:
///   pub trait UndoManager: Send + Sync {
///       fn push_undo(&self, record: Box<dyn UndoRecord>);
///       fn pop_undo(&self) -> Option<Box<dyn UndoRecord>>;
///       fn push_redo(&self, record: Box<dyn UndoRecord>);
///       fn pop_redo(&self) -> Option<Box<dyn UndoRecord>>;
///       fn clear_redo(&self);
///   }
///
/// ff-undo-redo provides `WorkbenchUndoManager` which routes operations
/// to the active document's `DocumentUndoManager` based on the current
/// ExecutionContext.
pub struct WorkbenchUndoManager { /* ... */ }

impl WorkbenchUndoManager {
    /// Create with a document registry for routing undo to the active document.
    pub fn new() -> Self;

    /// Register a document's undo manager.
    /// Addresses: Requirement 11.1
    pub fn register_document(
        &self,
        document_id: &str,
        manager: Arc<Mutex<DocumentUndoManager>>,
    );

    /// Unregister a document (on close).
    /// Addresses: Requirement 11.3
    pub fn unregister_document(&self, document_id: &str);

    /// Set the currently active document ID (for routing).
    /// Addresses: Requirement 11.2
    pub fn set_active_document(&self, document_id: &str);
}
```

### UndoNotifier Trait

```rust
/// Notification trait for undo state changes.
/// Implemented by the GUI shell or other consumers.
/// Addresses: Requirement 18.3
pub trait UndoNotifier: Send + Sync {
    /// Called when the dirty flag changes.
    fn dirty_flag_changed(&self, is_dirty: bool);
    /// Called when undo availability changes.
    fn undo_available_changed(&self, available: bool);
    /// Called when redo availability changes.
    fn redo_available_changed(&self, available: bool);
    /// Called when a transaction is committed.
    fn transaction_committed(&self, name: &str);
    /// Called when a transaction is undone.
    fn transaction_undone(&self, name: &str);
    /// Called when a transaction is redone.
    fn transaction_redone(&self, name: &str);
}
```


### UndoableState Trait (Container Actions)

```rust
/// Trait for plugin/extension state that participates in undo.
/// Addresses: Requirement 13.2
pub trait UndoableState: Send + Sync + std::fmt::Debug {
    /// Reverse this state change.
    fn undo(&self);
    /// Re-apply this state change.
    fn redo(&self);
    /// Human-readable description for diagnostics.
    fn description(&self) -> &str;
    /// Whether this container action may coalesce with adjacent actions.
    /// Addresses: Requirement 13.3
    fn may_coalesce(&self) -> bool { false }
}
```

### RecordIdMap API

```rust
impl RecordIdMap {
    /// Create a new map, assigning IDs to initial line count.
    /// Addresses: Requirement 14.1
    pub fn new(initial_line_count: u64) -> Self;

    /// Assign a new ID for an inserted record.
    /// Addresses: Requirement 14.2
    pub fn assign_id(&mut self) -> LogicalRecordId;

    /// Retire an ID (record deleted). Never reused.
    /// Addresses: Requirement 14.3
    pub fn retire_id(&mut self, id: LogicalRecordId);

    /// Look up current byte offset for a record ID.
    /// Addresses: Requirement 14.4
    pub fn offset_for(&self, id: LogicalRecordId) -> Option<u64>;

    /// Update offsets after a document modification.
    /// Addresses: Requirement 14.4
    pub fn update_offsets(&mut self, position: u64, delta: i64);

    /// Serialize for recovery file.
    /// Addresses: Requirement 14.7
    pub fn serialize(&self) -> Vec<u8>;

    /// Deserialize from recovery data.
    pub fn deserialize(data: &[u8]) -> Result<Self, UndoError>;
}
```

---

## 6. Error Types

```rust
/// Errors produced by the undo-redo-transactions crate.
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UndoError {
    /// Undo stack is empty — nothing to undo.
    /// Addresses: Requirement 4.2
    #[error("[undo] undo: nothing to undo")]
    NothingToUndo,

    /// Redo stack is empty — nothing to redo.
    /// Addresses: Requirement 2.5
    #[error("[undo] redo: nothing to redo")]
    NothingToRedo,

    /// Undo is disabled (max_levels == 0).
    /// Addresses: Requirement 1.5
    #[error("[undo] operation: undo is disabled (max_levels=0)")]
    UndoDisabled,

    /// Operation not available in current mode (Browse/View).
    /// Addresses: Requirement 4.8
    #[error("[undo] {operation}: not available in {mode} mode")]
    NotAvailableInMode { operation: String, mode: String },

    /// Transaction rollback failed.
    /// Addresses: Requirement 3.4
    #[error("[undo] rollback: failed to reverse operation at position {position}")]
    RollbackFailed { position: u64 },

    /// No active transaction to end or abort.
    #[error("[undo] end_transaction: no transaction in progress")]
    NoActiveTransaction,

    /// Recovery file I/O error.
    /// Addresses: Requirement 8
    #[error("[undo] recovery: {operation} failed — {source}")]
    RecoveryIo {
        operation: String,
        source: std::io::Error,
    },

    /// Recovery file is corrupted or incompatible.
    /// Addresses: Requirement 16.3
    #[error("[undo] recovery: file is corrupted or incompatible")]
    RecoveryCorrupted,

    /// History validation failed.
    /// Addresses: Requirement 16.1
    #[error("[undo] validate: history inconsistent with document length {expected}, computed {actual}")]
    ValidationFailed { expected: u64, actual: u64 },

    /// Tentative mode error (e.g., commit without active tentative).
    /// Addresses: Requirement 12
    #[error("[undo] tentative: {description}")]
    TentativeError { description: String },

    /// Bulk transaction error.
    /// Addresses: Requirement 7
    #[error("[undo] bulk: {description}")]
    BulkError { description: String },

    /// Configuration value out of range (applied default).
    /// Addresses: Requirement 1.6
    #[error("[undo] config: {key} value {value} out of range, using default {default}")]
    ConfigOutOfRange {
        key: String,
        value: String,
        default: String,
    },
}
```


---

## 7. Integration Points

### With `ff-command` (upstream — Wave 2)

- `ff-undo-redo` implements the `UndoManager` trait defined by `ff-command` (see §5 WorkbenchUndoManager)
- The command framework pushes `UndoRecord` trait objects after undoable command execution
- Built-in `edit.undo` / `edit.redo` commands in `ff-command` call `pop_undo()` / `pop_redo()` on the UndoManager
- The command framework clears the redo stack when a new undoable command executes
- `ff-undo-redo` does NOT depend on `ff-command` at crate level — it implements a trait defined there, connected at runtime by `ff-core`

### With `ff-document-model` (peer — Wave 4)

- `ff-undo-redo` does NOT depend on `ff-document-model` directly (per Requirement 18.5)
- Instead, it accepts edit operations via a trait interface (`EditTarget`) that the document model implements
- When undo/redo executes, the manager calls `EditTarget::apply_insert()`, `EditTarget::apply_delete()`, `EditTarget::apply_replace()` to reverse/re-apply operations
- The document model calls `record_insert()`, `record_delete()`, `record_replace()` on the undo manager during mutations

```rust
/// Trait that the document model implements for receiving reversed/re-applied edits.
/// Addresses: Requirement 18.5
pub trait EditTarget: Send + Sync {
    fn apply_insert(&mut self, position: u64, text: &[u8]) -> Result<(), UndoError>;
    fn apply_delete(&mut self, position: u64, length: u64) -> Result<(), UndoError>;
    fn apply_replace(
        &mut self,
        position: u64,
        old_length: u64,
        new_text: &[u8],
    ) -> Result<(), UndoError>;
}
```

### With `ff-logging` (upstream — Wave 0)

- `ff-undo-redo` uses `ff-logging` for:
  - WARN when `max_levels` config is negative (applying default) — Requirement 1.6
  - WARN when an orphaned transaction is force-closed — Requirement 3.5
  - WARN when history validation fails (clearing history) — Requirement 16.3
  - WARN when recovery file load fails — Requirement 8
  - INFO for transaction commit/undo/redo in debug builds

### With `ff-configuration` (upstream — Wave 2)

- Configuration values are passed to `DocumentUndoManager` at construction via `UndoConfig`
- The crate does NOT directly read configuration files — `ff-core` mediates
- Relevant keys:
  - `editor.undo.max_levels` (Requirement 1.3)
  - `editor.undo.coalesce_timeout_ms` (Requirement 6.4)
  - `editor.undo.selection_history` (Requirement 9.7)
  - `editor.recovery.interval_seconds` (Requirement 8.2)

### With `edit-operations` (peer — Wave 4)

- The `edit-operations` crate defines what constitutes a transaction boundary (Requirement 3.1)
- Each editing command (character insert, delete, paste, etc.) calls `begin_transaction()` / `end_transaction()` or relies on coalescing for single-char edits
- Multi-caret operations wrap all caret edits in a single transaction group

### With `file-operations` (downstream — Wave 8)

- File save triggers `set_save_point()` and recovery file deletion (Requirements 5.2, 8.3)
- File open checks for recovery file existence and offers restore/discard (Requirement 8.4)

### Dependency Direction

```
ff-logging ← ff-undo-redo → (implements trait from) ff-command
                           → (accepts trait impl from) ff-document-model
                           ← ff-core (wires everything together)
                           ← edit-operations (records edits)
                           ← file-operations (save point, recovery)
```

`ff-undo-redo` depends only on: `ff-logging`, `chrono`, `serde`, `thiserror`, standard library. All other crates are connected via traits at runtime.


---

## 8. Configuration

All configuration is provided through `ff-core` at initialization time. The crate does not directly read configuration files.

### Relevant Configuration Keys

```toml
[editor.undo]
# Maximum number of transactions on the undo stack per document.
# Range: 0–10000. 0 disables undo entirely.
# Values < 0 → apply default (100), emit WARN.
# Default: 100
# Addresses: Requirement 1.3, 1.5, 1.6
max_levels = 100

# Timeout for coalescing consecutive character edits into one transaction.
# Range: 100–10000 (milliseconds).
# Default: 2000
# Addresses: Requirement 6.4
coalesce_timeout_ms = 2000

# Whether undo/redo restores cursor/selection position.
# Values: "enabled", "disabled"
# Default: "enabled"
# Addresses: Requirement 9.7
selection_history = "enabled"

[editor.recovery]
# Interval between recovery file writes (seconds). 0 = disabled.
# Default: 60
# Addresses: Requirement 8.2
interval_seconds = 60
```

---

## 9. Concurrency Model

### Thread-Safety Approach

| Component | Mechanism | Rationale |
|-----------|-----------|-----------|
| `DocumentUndoManager` | `Arc<Mutex<DocumentUndoManager>>` | Single writer at a time (edit + undo are sequential within a document) |
| `WorkbenchUndoManager` | `RwLock<HashMap<String, Arc<Mutex<DocumentUndoManager>>>>` | Multiple documents; read-heavy (routing), rare writes (open/close) |
| `RecoveryWriter` | Separate async task with channel | Non-blocking periodic writes; document edits are not blocked by I/O |
| `ScrapStack` | Owned by `DocumentUndoManager` (inside Mutex) | No independent locking needed |
| `RecordIdMap` | Owned by `DocumentUndoManager` (inside Mutex) | Updated synchronously with edits |
| `UndoNotifier` callbacks | Invoked outside lock scope | Prevents deadlock; listeners must not call back into undo manager |

### Recovery File I/O

The recovery writer runs as an async task:
1. A timer fires every `recovery_interval_seconds`
2. If the dirty flag is true, the writer acquires the document mutex briefly to serialize state
3. The serialized data is written to disk asynchronously (outside the lock)
4. On success, the writer records the timestamp of the last successful recovery write

This ensures document editing is never blocked by disk I/O.

### Lock Ordering (extends ff-command ordering)

1. `WorkbenchUndoManager` routing lock (read)
2. `DocumentUndoManager` mutex (write)
3. `UndoNotifier` callbacks (no lock held)

No operation acquires these in reverse order.


---

## 10. Correctness Properties

These properties are suitable for property-based testing with `proptest`. They validate invariants that must hold across all valid inputs.

### Property 1: Undo/Redo Stack Depth Invariant

**Statement**: For any sequence of commit, undo, and redo operations on a `DocumentUndoManager` with `max_levels = M`, the undo stack depth is always ≤ M, and the redo stack depth is always ≤ M. After N commits (N > M), the undo stack contains exactly M transactions (the most recent M), and the oldest transactions have been evicted.

**Validates**: Requirement 1.3, 1.4

```rust
// proptest strategy: generate max_levels in [1, 200], generate sequence of commit operations
// assertion: undo_depth() <= max_levels at all times; after N > M commits, depth == M
```

### Property 2: Undo-Redo Symmetry

**Statement**: For any document with initial content and any sequence of edit operations committed as transactions, performing undo() followed by redo() returns the document to the same state as before the undo. Conversely, redo() after undo() produces a byte-identical document to the original post-edit state.

**Validates**: Requirement 4.1, 4.4, 4.9

```rust
// proptest strategy: generate initial content, generate sequence of edits, commit
// assertion: state_after_edits == state_after_undo_then_redo (byte-identical)
```

### Property 3: Redo Stack Cleared on New Commit

**Statement**: For any state where the redo stack is non-empty (achieved by performing undo operations), committing a new transaction clears the redo stack entirely. After the commit, `can_redo()` returns false and `redo_depth()` returns 0.

**Validates**: Requirement 2.2

```rust
// proptest strategy: generate commits, perform K undos (K > 0), then commit new
// assertion: redo_depth() == 0 after the new commit
```

### Property 4: Save Point Dirty Flag Derivation

**Statement**: For any sequence of commits, undos, and redos: `is_dirty()` returns false if and only if the current action position equals the save point position and the detach point is not set. Setting the save point at any position makes `is_dirty()` false at that position. Undoing back to the save point makes `is_dirty()` false again.

**Validates**: Requirement 5.1, 5.3, 5.4

```rust
// proptest strategy: generate sequence of (commit, undo, redo, set_save_point)
// assertion: is_dirty() == (current_position != save_point || detach_point.is_some())
```

### Property 5: Detach Point Semantics

**Statement**: When a new commit occurs while the redo stack is non-empty and the save point is located in the redo portion being discarded, the detach point is set. Once detached, `is_dirty()` always returns true regardless of any subsequent undo/redo operations (the saved state is unreachable).

**Validates**: Requirement 5.5

```rust
// proptest strategy: commit N, save, undo K, commit new → detach
// assertion: after_detach_point() == true ∧ is_dirty() == true for all subsequent positions
```


### Property 6: Coalescing Contiguity Rule

**Statement**: For any sequence of single-character inserts where each insert position equals the previous insert's position + 1 (contiguous typing), and no boundary event occurs within the coalesce timeout, all characters are merged into a single transaction. The undo stack depth increases by exactly 1 for the entire typing sequence.

**Validates**: Requirement 6.1, 6.7

```rust
// proptest strategy: generate N contiguous single-char inserts with no timeout or boundary
// assertion: undo_depth() increases by 1 total; undoing once reverses all N characters
```

### Property 7: Coalescing Boundary Events

**Statement**: For any of the defined boundary events (cursor movement, operation type change, explicit begin_transaction, timeout, save, non-character edit), coalescing is broken: the next edit starts a new transaction even if it would otherwise be contiguous with the previous edit.

**Validates**: Requirement 6.3

```rust
// proptest strategy: generate typing sequence, inject boundary event at random point
// assertion: undo_depth() == 2 (one for pre-boundary, one for post-boundary)
```

### Property 8: Transaction Nesting Depth Tracking

**Statement**: For any sequence of `begin_transaction()` and `end_transaction()` calls, `transaction_depth()` equals the count of begin calls minus end calls (clamped at 0). Only when depth returns to 0 is the transaction committed to the undo stack. Intermediate begins/ends do not create additional stack entries.

**Validates**: Requirement 3.3, 3.7

```rust
// proptest strategy: generate interleaved begin/end calls with edits
// assertion: depth == begins - ends (clamped ≥ 0); commits only when depth reaches 0
```

### Property 9: Bulk Transaction Memory Efficiency

**Statement**: For a `RuleTransaction` affecting N records, the memory cost is O(1) — it does not grow with N. For an `IndexTransaction` affecting N records, the memory cost is O(N) (proportional to the number of affected record IDs stored).

**Validates**: Requirement 7.8

```rust
// proptest strategy: generate bulk operations with varying N
// assertion: RuleTransaction size is constant; IndexTransaction size grows linearly with N
```

### Property 10: Selection History Restoration

**Statement**: For any transaction committed with selection history enabled, the before-state is restored on undo and the after-state is restored on redo. When selection history is disabled, undo/redo still modifies document content correctly but does not alter the current selection state.

**Validates**: Requirement 9.1, 9.3, 9.4, 9.7, 9.8

```rust
// proptest strategy: generate transactions with before/after selection states
// assertion: after undo, selection == before_state; after redo, selection == after_state
// assertion (disabled): undo/redo does not modify selection
```

### Property 11: Tentative Action Isolation

**Statement**: For any tentative action sequence (tentative_start, edits, tentative_rollback), the document and undo stack are returned to their pre-tentative state. No trace of the tentative actions remains in the undo history. Conversely, tentative_commit makes the actions permanent and discards any redo history beyond them.

**Validates**: Requirement 12.1, 12.3, 12.4

```rust
// proptest strategy: generate pre-tentative edits, enter tentative, add edits, rollback
// assertion: document state == pre-tentative state; undo history unchanged
```


### Property 12: Recovery Round-Trip Integrity

**Statement**: For any valid `DocumentUndoManager` state (with undo stack, redo stack, save point, scrap data, and record ID map), serializing to recovery format and deserializing produces an equivalent state. The restored manager's `validate()` passes against the same document length, and undo/redo operations produce the same results as the original.

**Validates**: Requirement 8.5, 8.7, 16.4

```rust
// proptest strategy: generate undo manager state, serialize, deserialize, validate
// assertion: validate() == true; undo/redo produce same outcomes as original
```

### Property 13: Validation Detects Inconsistency

**Statement**: For any undo history where the cumulative size delta (sum of inserts minus deletes) does not match the actual document length change, `validate()` returns false. For any valid history, `validate()` returns true.

**Validates**: Requirement 16.1, 16.2

```rust
// proptest strategy: generate valid histories (validate == true), mutate to create inconsistency
// assertion: valid histories pass; corrupted histories fail
```

### Property 14: Per-Document Isolation

**Statement**: For any two documents registered with the `WorkbenchUndoManager`, operations on one document's undo stack (commit, undo, redo, save point) have no effect on the other document's undo stack, redo stack, dirty flag, or save point.

**Validates**: Requirement 11.1

```rust
// proptest strategy: generate operations on doc A and doc B interleaved
// assertion: doc_a.undo_depth() is independent of operations on doc B and vice versa
```

### Property 15: Logical Record ID Stability

**Statement**: For any sequence of record insertions and deletions, each assigned `LogicalRecordId` is unique within the session and is never reused. Retired IDs do not appear in subsequent `assign_id()` results. The `offset_for()` method correctly tracks position changes caused by insertions/deletions of other records.

**Validates**: Requirement 14.1, 14.2, 14.3, 14.4

```rust
// proptest strategy: generate sequence of inserts/deletes with ID assignments
// assertion: all assigned IDs are unique; retired IDs never reappear; offsets correct
```

---

## Appendix A: External Crate Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `chrono` | 0.4 | UTC timestamps for transactions |
| `serde` | 1.0 | Serialization for recovery files |
| `serde_json` | 1.0 | JSON format for recovery files (alternative: bincode) |
| `thiserror` | 2.0 | Error type derivation |
| `proptest` | 1.0 | Property-based testing (dev-dependency only) |

Note: `bincode` may be used instead of or alongside `serde_json` for compact binary recovery files (Requirement 8.7 allows "compact binary or JSON format").

## Appendix B: Recovery File Format

The recovery file (`.filename.recovery`) contains:

```
┌──────────────────────────────────────┐
│ Header (magic bytes + version)       │  8 bytes
├──────────────────────────────────────┤
│ Document metadata                    │  variable
│   - original file path               │
│   - original file size               │
│   - session ID                       │
├──────────────────────────────────────┤
│ Scrap Stack (all edit text data)     │  variable
├──────────────────────────────────────┤
│ Undo Stack (serialized transactions) │  variable
├──────────────────────────────────────┤
│ Redo Stack (serialized transactions) │  variable
├──────────────────────────────────────┤
│ Save Point position                  │  8 bytes
├──────────────────────────────────────┤
│ Detach Point position                │  8 bytes
├──────────────────────────────────────┤
│ Logical Record ID Map                │  variable
├──────────────────────────────────────┤
│ Checksum (CRC32)                     │  4 bytes
└──────────────────────────────────────┘
```

The CRC32 checksum covers all preceding sections. On restore, the checksum is validated before accepting the recovery data.

## Appendix C: Coalescing State Machine

```
┌──────────┐   char insert at pos+1   ┌──────────────┐
│  IDLE    │ ─────────────────────────→│  COALESCING  │
│(no txn)  │                           │ (accumulating)│
└──────────┘                           └──────────────┘
     ↑                                       │
     │  boundary event or timeout            │ char at pos+1
     │←─────────────────────────────────────←│ (extend txn)
     │                                       │
     │  commit (close coalesce window)       │
     │←─────────────────────────────────────←│
```

Transitions that break coalescing:
- Cursor movement (any non-typing position change)
- Operation type change (insert → delete or vice versa)
- Explicit `begin_transaction()` call
- Pause > `coalesce_timeout_ms`
- Document save (`set_save_point()`)
- Non-character operation (paste, cut, line command)
- Position at save point or tentative point

## Appendix D: Undo-Disabled Mode

When `max_levels == 0`:
- `record_insert()`, `record_delete()`, `record_replace()` are no-ops
- `begin_transaction()` / `end_transaction()` are no-ops
- `undo()` returns `Err(UndoDisabled)`
- `redo()` returns `Err(UndoDisabled)`
- `is_dirty()` is tracked by a simple boolean (modified since save) rather than save-point distance
- Recovery files are NOT written (no undo state to recover)
- The status message for UNDO/REDO indicates undo is disabled
