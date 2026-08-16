# Design Document: Idle Processing (`ff-idle-processing`)

## Overview

The `ff-idle-processing` crate is the **cooperative background work scheduler** for FileForgeWorkbench. It detects idle periods (no user input), grants bounded time slices to registered work sources in priority order, and ensures instant cancellation on any user activity. The scheduler is GUI-independent — it receives idle notifications through a trait abstraction and never references any windowing or rendering framework.

### Purpose

- Detect application idle state based on configurable inactivity thresholds
- Dispatch time-bounded work slices to registered background tasks
- Enforce priority ordering across heterogeneous work sources
- Guarantee immediate cancellation on user input (< 1ms signal latency)
- Track per-source progress and completion state
- Provide completion notifications for downstream consumers
- Operate with zero CPU overhead when no work is pending
- Support headless/test usage without a real event loop

### Position in Architecture

```
Wave 15 — Background Processing

┌──────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                    │
│   Implements IdleNotifier using request_repaint_after          │
│   Calls input_activity() on each user event                   │
│   Invokes on_idle() from idle callback                        │
├──────────────────────────────────────────────────────────────┤
│  Work Source Providers:                                        │
│    ff-syntax-highlighting (Wave 7) — idle background styling  │
│    ff-line-wrap-toggle (Wave 9) — wrap height calculation     │
│    ff-find-and-replace (Wave 8) — search index building       │
├──────────────────────────────────────────────────────────────┤
│         THIS CRATE: ff-idle-processing ← Wave 15              │
│   IdleScheduler, IdleWorkSource trait, priority dispatch,     │
│   time budget enforcement, cancellation, progress tracking    │
├──────────────────────────────────────────────────────────────┤
│  Upstream Dependencies:                                       │
│    ff-display-line-mapping (Wave 4) — set_height consumer     │
│    ff-configuration-system (Wave 2) — idle parameters         │
│    ff-logging (Wave 0) — structured diagnostics               │
├──────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                      │
└──────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (Req 12)**: Zero GUI dependencies — receives idle state through `IdleNotifier` trait, time through `std::time::Instant`
- **Cooperative Time-Slicing**: Work sources yield voluntarily; scheduler does not forcibly terminate
- **Input-First Responsiveness**: Any user event cancels idle work within 1ms via atomic flag
- **Multi-Crate Workspace**: Crate at `crates/ff-idle-processing`
- **Error Message Standards**: Errors follow `[idle-processing] operation: description` format
- **Zero Overhead Quiescence**: No timer handles or callbacks when all work is complete

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "GUI Shell"
        SHELL[ff-desktop / egui]
        INPUT[User Input Events]
        IDLE_CB[Idle Callback Mechanism]
    end

    subgraph "Work Source Providers"
        SYN[ff-syntax-highlighting<br/>IdleStylingTask]
        WRAP[ff-line-wrap-toggle<br/>WrapCalculationTask]
        FIND[ff-find-and-replace<br/>SearchIndexTask]
        PLUGIN_WS[Plugin-provided<br/>Work Sources]
    end

    subgraph "ff-idle-processing"
        SCHED[IdleScheduler]
        DISPATCH[Priority Dispatcher]
        TIMER[Time Budget Monitor]
        CANCEL[Cancellation Signal<br/>AtomicBool]
        PROGRESS[Progress Tracker]
        NOTIFY[Completion Notifier]
        NOTIFIER_TRAIT[IdleNotifier Trait]
        WS_TRAIT[IdleWorkSource Trait]
    end

    subgraph "Upstream"
        DLM[ff-display-line-mapping<br/>set_height target]
        CFG[ff-configuration-system<br/>idle parameters]
        LOG[ff-logging]
    end

    INPUT -->|input_activity| SCHED
    IDLE_CB -->|on_idle| SCHED
    SHELL -.->|implements| NOTIFIER_TRAIT

    SYN -.->|implements| WS_TRAIT
    WRAP -.->|implements| WS_TRAIT
    FIND -.->|implements| WS_TRAIT
    PLUGIN_WS -.->|implements| WS_TRAIT

    SCHED --> DISPATCH
    SCHED --> TIMER
    SCHED --> CANCEL
    SCHED --> PROGRESS
    SCHED --> NOTIFY
    SCHED --> NOTIFIER_TRAIT

    DISPATCH --> WS_TRAIT
    WRAP -->|set_height| DLM
    SCHED -->|read config| CFG
    SCHED -->|diagnostics| LOG
end
```

## Components and Interfaces

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| **IdleScheduler** | Central coordinator: owns registered work sources, detects idle transitions, dispatches time slices, manages cancellation and completion state |
| **Priority Dispatcher** | Selects the highest-priority active work source for the current time slice; implements round-robin among same-priority sources and starvation prevention |
| **Time Budget Monitor** | Measures elapsed time during each slice; provides `time_remaining()` query to work sources; logs warnings on budget overruns |
| **Cancellation Signal** | `AtomicBool` flag set on user input; polled by work sources via `IdleWorkContext::is_cancelled()` for sub-millisecond interrupt |
| **Progress Tracker** | Stores per-source `WorkProgress` state; provides aggregate queries for UI progress display |
| **Completion Notifier** | Callback registry for per-source and global completion events; supports synchronous and queued delivery |
| **IdleNotifier Trait** | GUI-shell abstraction for requesting/cancelling idle callbacks from the event loop |
| **IdleWorkSource Trait** | Contract for background work providers: perform_work, priority, name, progress, invalidate |

### Scheduler State Machine

```
                    ┌───────────────────────┐
                    │       INACTIVE        │
                    │  (no sources / all    │
                    │   complete)           │
                    └───────────┬───────────┘
                                │ register() or invalidate_source()
                                ▼
                    ┌───────────────────────┐
                    │   WAITING_FOR_IDLE    │
                    │  (input cooldown)     │◄──── input_activity()
                    └───────────┬───────────┘
                                │ Idle_Detection_Threshold elapsed
                                ▼
                    ┌───────────────────────┐
                    │        ACTIVE         │◄──── on_idle() dispatches
                    │  (dispatching slices) │      time slices
                    └───────────┬───────────┘
                                │ all sources complete
                                ▼
                    ┌───────────────────────┐
                    │       INACTIVE        │
                    └───────────────────────┘

    From ACTIVE:
        input_activity() → cancel current slice → WAITING_FOR_IDLE
```

---

## Module Structure

```
crates/ff-idle-processing/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── scheduler.rs                # IdleScheduler struct, main state machine
│   ├── traits.rs                   # IdleWorkSource, IdleNotifier trait definitions
│   ├── context.rs                  # IdleWorkContext: time budget + cancellation
│   ├── priority.rs                 # WorkPriority type and built-in constants
│   ├── progress.rs                 # WorkProgress, WorkStatus types
│   ├── dispatcher.rs              # Priority dispatch logic, round-robin, starvation prevention
│   ├── notifier.rs                # Completion notification registry and dispatch
│   ├── config.rs                   # IdleConfig, TOML parameter resolution
│   ├── clock.rs                    # Clock trait for testable time injection
│   ├── error.rs                    # IdleProcessingError enum
│   └── test_support.rs            # ManualIdleNotifier, MockClock for testing
└── tests/
    ├── scheduler_tests.rs          # Core scheduling state machine tests
    ├── priority_tests.rs           # Priority ordering and dispatch tests
    ├── time_budget_tests.rs        # Time budget enforcement tests
    ├── cancellation_tests.rs       # Cancellation signal and interrupt tests
    ├── progress_tests.rs           # Progress tracking and completion tests
    ├── notification_tests.rs       # Completion notification delivery tests
    ├── starvation_tests.rs         # Starvation prevention tests
    ├── property_tests.rs           # Property-based tests (proptest)
    └── integration.rs              # End-to-end with mock work sources
```

---

## Data Models

### WorkPriority

```rust
/// Numeric priority for idle work sources. Lower values = higher priority.
/// Priority 0 is the highest possible; u32::MAX is the lowest.
///
/// Addresses: Requirement 4 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkPriority(pub u32);

impl WorkPriority {
    /// Syntax re-highlighting beyond viewport — highest built-in priority.
    /// Addresses: Requirement 4 AC 2
    pub const SYNTAX_HIGHLIGHT: Self = Self(10);

    /// Word-wrap height measurement.
    /// Addresses: Requirement 4 AC 2
    pub const WRAP_CALCULATION: Self = Self(20);

    /// Fold-level computation for collapsed regions.
    /// Addresses: Requirement 4 AC 2
    pub const FOLD_COMPUTATION: Self = Self(30);

    /// Search index building for find-all.
    /// Addresses: Requirement 4 AC 2
    pub const SEARCH_INDEX: Self = Self(40);

    /// Returns the raw numeric value.
    pub fn value(self) -> u32 {
        self.0
    }
}
```

### WorkStatus

```rust
/// Result returned by `IdleWorkSource::perform_work` indicating the
/// outcome of a single time slice.
///
/// Addresses: Requirement 3 AC 1, Requirement 5 AC 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkStatus {
    /// More work remains; the source should be serviced again on the next
    /// idle callback.
    MoreWork,

    /// All work is complete; the source transitions to dormant state.
    /// Addresses: Requirement 7 AC 1
    Complete,

    /// The work was interrupted by a cancellation signal. The source
    /// has saved its progress and can resume from the same position.
    /// Addresses: Requirement 5 AC 3
    Interrupted,
}
```

### WorkProgress

```rust
/// Progress information for a single work source.
///
/// Addresses: Requirement 6 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkProgress {
    /// Amount of work completed (e.g., lines styled, lines measured).
    pub completed_units: u64,

    /// Total scope of work (e.g., total document lines).
    pub total_units: u64,

    /// Whether all work is finished.
    pub is_complete: bool,
}

impl WorkProgress {
    /// Create a new progress value indicating no work done.
    pub fn new(total_units: u64) -> Self {
        Self {
            completed_units: 0,
            total_units,
            is_complete: false,
        }
    }
}
```

### IdleWorkContext

```rust
/// Context provided to a work source during its time slice.
/// Exposes time budget queries and the cancellation signal.
///
/// Addresses: Requirement 2 AC 2, Requirement 5 AC 2
pub struct IdleWorkContext<'a> {
    /// Reference to the cancellation flag (set by input_activity).
    cancelled: &'a std::sync::atomic::AtomicBool,

    /// Instant when this time slice started.
    slice_start: std::time::Instant,

    /// Total time budget for this slice.
    time_budget: std::time::Duration,
}

impl<'a> IdleWorkContext<'a> {
    /// Check whether a cancellation event has occurred.
    /// Work sources MUST poll this at least once per significant work unit.
    ///
    /// Addresses: Requirement 5 AC 2
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Returns the remaining time in the current time slice.
    /// When this reaches zero, the work source should yield.
    ///
    /// Addresses: Requirement 2 AC 2
    pub fn time_remaining(&self) -> std::time::Duration {
        let elapsed = self.slice_start.elapsed();
        self.time_budget.saturating_sub(elapsed)
    }

    /// Returns true if the time budget has been exhausted.
    pub fn budget_exhausted(&self) -> bool {
        self.slice_start.elapsed() >= self.time_budget
    }
}
```

### SchedulerState

```rust
/// Internal state of the idle scheduler's state machine.
///
/// Addresses: Requirement 1, Requirement 11
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchedulerState {
    /// No active work sources or all complete. Zero overhead.
    /// Addresses: Requirement 11 AC 1, AC 2
    Inactive,

    /// Active work sources exist but waiting for idle detection threshold.
    /// Addresses: Requirement 1 AC 1
    WaitingForIdle,

    /// Currently in idle state, dispatching time slices.
    /// Addresses: Requirement 1 AC 3
    Active,
}
```

### IdleConfig

```rust
/// Configuration parameters for the idle scheduler.
/// Loaded from the configuration-system's `[idle-processing]` namespace.
///
/// Addresses: Requirement 1 AC 2, Requirement 2 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdleConfig {
    /// Duration of input inactivity before entering idle state.
    /// Default: 200ms.
    /// Addresses: Requirement 1 AC 2
    pub idle_detection_threshold: std::time::Duration,

    /// Maximum duration of a single time slice.
    /// Default: 10ms. Set to 0 to disable idle processing entirely.
    /// Addresses: Requirement 2 AC 1, AC 5
    pub time_budget: std::time::Duration,

    /// Maximum lines a work source should process per time slice.
    /// Used as guidance for work sources. Default: 256.
    /// Addresses: Requirement 8 AC 1
    pub lines_per_slice: usize,

    /// Number of idle cycles before lower-priority sources get a
    /// guaranteed time slice (starvation prevention).
    /// Default: 10.
    /// Addresses: Requirement 4 AC 6
    pub starvation_cycle_limit: u32,
}

impl Default for IdleConfig {
    fn default() -> Self {
        Self {
            idle_detection_threshold: std::time::Duration::from_millis(200),
            time_budget: std::time::Duration::from_millis(10),
            lines_per_slice: 256,
            starvation_cycle_limit: 10,
        }
    }
}
```

### SubscriptionId

```rust
/// Handle for a registered completion callback.
///
/// Addresses: Requirement 10 AC 6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);
```

---

## Public API Surface

### IdleWorkSource Trait

```rust
/// Trait that background work providers implement to participate in
/// idle-time scheduling. Object-safe for heterogeneous collections.
///
/// Addresses: Requirement 3 AC 1, AC 6
pub trait IdleWorkSource: Send + Sync {
    /// Execute a bounded unit of work within the time budget.
    /// The implementation MUST poll `context.is_cancelled()` at least once
    /// per significant unit of work and return `WorkStatus::Interrupted`
    /// if cancellation is detected.
    ///
    /// Addresses: Requirement 3 AC 1, Requirement 5 AC 2
    fn perform_work(&mut self, context: &mut IdleWorkContext) -> WorkStatus;

    /// Returns the priority level of this work source.
    /// Lower values = higher priority.
    ///
    /// Addresses: Requirement 3 AC 1
    fn priority(&self) -> WorkPriority;

    /// Returns a human-readable identifier for diagnostics and logging.
    ///
    /// Addresses: Requirement 3 AC 1
    fn name(&self) -> &str;

    /// Returns the current progress state for tracking.
    ///
    /// Addresses: Requirement 3 AC 1
    fn progress(&self) -> WorkProgress;

    /// Reset progress to the beginning. Called when previous work is stale
    /// (e.g., document edit invalidates completed styling).
    /// Default implementation does nothing.
    ///
    /// Addresses: Requirement 3 AC 2
    fn invalidate(&mut self) {}
}
```

### IdleNotifier Trait

```rust
/// Abstraction over the GUI event loop's idle callback mechanism.
/// The GUI shell implements this trait to integrate with the scheduler.
///
/// Addresses: Requirement 9 AC 1, AC 2
pub trait IdleNotifier: Send + Sync {
    /// Request the event loop to invoke `IdleScheduler::on_idle()` when
    /// the application becomes idle.
    ///
    /// Addresses: Requirement 9 AC 1
    fn request_idle_callback(&self);

    /// Cancel a previously requested idle callback.
    ///
    /// Addresses: Requirement 9 AC 1
    fn cancel_idle_callback(&self);
}
```

### Clock Trait (Test Support)

```rust
/// Abstraction over time measurement for testability.
/// Production code uses `SystemClock`; tests use `MockClock`.
///
/// Addresses: Requirement 12 AC 2
pub trait Clock: Send + Sync {
    /// Returns the current instant.
    fn now(&self) -> std::time::Instant;
}

/// Production clock using `std::time::Instant::now()`.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}
```

### IdleScheduler

```rust
/// Central idle-time work coordinator. Owns registered work sources,
/// manages the scheduling state machine, and dispatches time slices.
///
/// Addresses: Requirements 1–12
pub struct IdleScheduler {
    // Internal state (private)
}

impl IdleScheduler {
    /// Create a new scheduler with the given configuration, notifier, and clock.
    ///
    /// Addresses: Requirement 9 AC 2, Requirement 12 AC 2
    pub fn new(
        config: IdleConfig,
        notifier: Box<dyn IdleNotifier>,
        clock: Box<dyn Clock>,
    ) -> Self;

    // --- Work Source Registration ---

    /// Register a work source. Immediately enables it for time-slice dispatch.
    /// If the scheduler is already idle, begins servicing on next callback.
    ///
    /// Addresses: Requirement 3 AC 3, AC 5
    pub fn register(&mut self, source: Box<dyn IdleWorkSource>);

    /// Unregister a work source by name, returning ownership to the caller.
    /// Fully removes from both active and dormant sets.
    ///
    /// Addresses: Requirement 3 AC 4, Requirement 7 AC 6
    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn IdleWorkSource>>;

    // --- Input Activity ---

    /// Notify the scheduler that user input has occurred.
    /// Sets the cancellation flag and resets the idle detection timer.
    ///
    /// Addresses: Requirement 5 AC 1, Requirement 12 AC 3
    pub fn input_activity(&mut self);

    // --- Idle Callback Entry Point ---

    /// Single entry point invoked by the GUI shell's idle callback.
    /// Dispatches one time-slice to the highest-priority active source.
    /// Returns `true` if more idle work remains, `false` if all complete.
    ///
    /// Addresses: Requirement 9 AC 6
    pub fn on_idle(&mut self) -> bool;

    // --- Progress Queries ---

    /// Returns progress for a named work source.
    ///
    /// Addresses: Requirement 6 AC 2
    pub fn progress(&self, name: &str) -> Option<WorkProgress>;

    /// Returns progress for all registered work sources.
    ///
    /// Addresses: Requirement 6 AC 3
    pub fn all_progress(&self) -> Vec<(String, WorkProgress)>;

    /// Returns true when all registered work sources are complete.
    ///
    /// Addresses: Requirement 6 AC 5
    pub fn is_all_complete(&self) -> bool;

    // --- Invalidation ---

    /// Externally invalidate a specific work source by name.
    /// Resets its progress and reactivates it for dispatch.
    ///
    /// Addresses: Requirement 7 AC 4
    pub fn invalidate_source(&mut self, name: &str);

    /// Invalidate all registered work sources simultaneously.
    ///
    /// Addresses: Requirement 7 AC 5
    pub fn invalidate_all(&mut self);

    // --- Completion Notifications ---

    /// Subscribe to completion events for a specific work source.
    /// Returns a subscription handle for later removal.
    ///
    /// Addresses: Requirement 10 AC 5
    pub fn subscribe_completion(
        &mut self,
        source_name: &str,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> SubscriptionId;

    /// Remove a previously registered completion callback.
    ///
    /// Addresses: Requirement 10 AC 6
    pub fn unsubscribe_completion(
        &mut self,
        source_name: &str,
        subscription_id: SubscriptionId,
    );

    /// Subscribe to the global "all work complete" event.
    ///
    /// Addresses: Requirement 10 AC 3
    pub fn subscribe_all_completed(
        &mut self,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> SubscriptionId;

    // --- Configuration ---

    /// Update the scheduler configuration at runtime.
    /// Takes effect on the next idle cycle.
    pub fn update_config(&mut self, config: IdleConfig);

    /// Get the current configuration.
    pub fn config(&self) -> &IdleConfig;
}
```

### Test Support Types

```rust
/// A manual idle notifier for headless/test mode.
/// Allows tests to trigger on_idle() directly without a real event loop.
///
/// Addresses: Requirement 9 AC 7
pub struct ManualIdleNotifier {
    /// Tracks whether an idle callback has been requested.
    pub idle_requested: std::cell::Cell<bool>,
}

impl IdleNotifier for ManualIdleNotifier {
    fn request_idle_callback(&self) {
        self.idle_requested.set(true);
    }

    fn cancel_idle_callback(&self) {
        self.idle_requested.set(false);
    }
}

/// A mock clock for deterministic time control in tests.
///
/// Addresses: Requirement 12 AC 4
pub struct MockClock {
    /// The current mock time, advanced manually by tests.
    pub now: std::cell::Cell<std::time::Instant>,
}

impl Clock for MockClock {
    fn now(&self) -> std::time::Instant {
        self.now.get()
    }
}

impl MockClock {
    /// Advance mock time by the given duration.
    pub fn advance(&self, duration: std::time::Duration) {
        self.now.set(self.now.get() + duration);
    }
}
```

---

## Error Handling

```rust
/// Errors originating from the ff-idle-processing crate.
/// Formatted per project standards: `[idle-processing] operation: description`
///
/// Addresses: Cross-cutting error message standards
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IdleProcessingError {
    /// Attempted to register a work source with a name that already exists.
    #[error("[idle-processing] register: work source '{name}' already registered")]
    DuplicateWorkSource { name: String },

    /// Attempted to unregister a work source that does not exist.
    #[error("[idle-processing] unregister: work source '{name}' not found")]
    WorkSourceNotFound { name: String },

    /// Invalid configuration value.
    #[error("[idle-processing] config: {field} value {value} is invalid — {reason}")]
    InvalidConfig {
        field: String,
        value: String,
        reason: String,
    },

    /// Subscription not found for removal.
    #[error("[idle-processing] unsubscribe: subscription {id} not found for source '{source_name}'")]
    SubscriptionNotFound { source_name: String, id: u64 },
}
```

---

## Integration Points

### With `ff-syntax-highlighting` (Wave 7 — work source provider)

- **Dependency direction**: ff-syntax-highlighting depends on ff-idle-processing (implements `IdleWorkSource`)
- **Integration pattern**: The `IdleStylingTask` in ff-syntax-highlighting implements `IdleWorkSource`. When a document is opened and the lexer is bound, the highlighting engine registers this task with the idle scheduler at `WorkPriority::SYNTAX_HIGHLIGHT` (10). The task calls `HighlightEngine::idle_style_increment()` during each time slice, styling up to `lines_per_slice` lines beyond the viewport's styled position.
- **Invalidation**: On document edit, the syntax highlighting engine calls `scheduler.invalidate_source("syntax-highlight")` to reset the idle styling position to the edit point.
- **Addresses**: Requirement 8 AC 1, AC 5, AC 6, AC 7

### With `ff-display-line-mapping` (Wave 4 — downstream consumer)

- **Dependency direction**: ff-idle-processing does NOT directly depend on ff-display-line-mapping. The wrap calculation work source (provided by ff-line-wrap-toggle) holds a reference to the `DisplayLineMapping` trait and calls `set_height()` as results become available.
- **Integration pattern**: The wrap work source computes display heights during idle time and writes results via `set_height(doc_line, height)`, updating the mapping incrementally.
- **Addresses**: Requirement 8 AC 2

### With `ff-line-wrap-toggle` (Wave 9 — work source provider)

- **Dependency direction**: ff-line-wrap-toggle depends on ff-idle-processing (implements `IdleWorkSource`)
- **Integration pattern**: When wrap mode is toggled on, ff-line-wrap-toggle registers a `WrapCalculationTask` at `WorkPriority::WRAP_CALCULATION` (20). This task incrementally measures line display heights using content width and viewport width. When wrap mode is toggled off, it unregisters the task.
- **Invalidation**: On document edit or viewport width change, the wrap task is invalidated to re-measure from the affected line.
- **Addresses**: Requirement 8 AC 2, AC 5, AC 6, AC 7

### With `ff-find-and-replace` (Wave 8 — work source provider)

- **Dependency direction**: ff-find-and-replace depends on ff-idle-processing (implements `IdleWorkSource`)
- **Integration pattern**: When highlight-all-matches mode is activated, ff-find-and-replace registers a `SearchIndexTask` at `WorkPriority::SEARCH_INDEX` (40). This task pre-computes match positions for the entire document during idle time. When the search term changes, the source is invalidated for full rebuild.
- **Addresses**: Requirement 8 AC 4, AC 7

### With `ff-configuration-system` (Wave 2 — dependency)

- **Dependency direction**: ff-idle-processing depends on ff-configuration-system
- **API consumed**: Reads `[idle-processing]` TOML namespace for configurable parameters
- **Hot-reload**: On configuration change notification, the scheduler calls `update_config()` with the new values. Changes take effect on the next idle cycle.
- **Addresses**: Requirement 1 AC 2, Requirement 2 AC 1

### With `ff-logging` (Foundation — dependency)

- **Dependency direction**: ff-idle-processing depends on ff-logging
- **API consumed**: `log_info!`, `log_warn!`, `log_debug!` macros
- **Usage patterns**:
  - INFO: Scheduler state transitions (inactive → active, active → inactive)
  - WARN: Time budget overruns (Requirement 2 AC 3)
  - DEBUG: Per-slice dispatch details, work source registration/unregistration
- **Log prefix**: `[idle-processing]`
- **Addresses**: Requirement 2 AC 3

### With `ff-platform-core` (Wave 1 — architectural peer)

- **Dependency direction**: ff-idle-processing integrates with platform-core's event loop abstraction
- **Integration**: The `IdleNotifier` trait bridges the scheduler to whatever event loop the platform uses. The scheduler itself has no event loop dependency.
- **Addresses**: Requirement 9 AC 2, Requirement 12 AC 1

### Dependency Direction Summary

```
ff-logging                ← ff-idle-processing
ff-configuration-system   ← ff-idle-processing
ff-idle-processing        ← ff-syntax-highlighting (implements IdleWorkSource)
ff-idle-processing        ← ff-line-wrap-toggle (implements IdleWorkSource)
ff-idle-processing        ← ff-find-and-replace (implements IdleWorkSource)
ff-idle-processing        ← ff-desktop (implements IdleNotifier)
```

---

## Configuration

The `ff-idle-processing` crate owns the `[idle-processing]` namespace in the workbench TOML configuration file.

### TOML Schema

```toml
[idle-processing]
# Duration of input inactivity (ms) before entering idle state.
# Range: 50–5000. Default: 200.
idle_detection_threshold_ms = 200

# Maximum duration (ms) of a single time slice granted to a work source.
# Range: 0–100. Default: 10. Set to 0 to disable idle processing.
time_budget_ms = 10

# Maximum lines a work source should process per time slice (guidance).
# Range: 16–4096. Default: 256.
lines_per_slice = 256

# Number of idle cycles before lower-priority sources get a guaranteed slice.
# Range: 1–100. Default: 10.
starvation_cycle_limit = 10
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `idle_detection_threshold_ms` | Default 200 | Default + WARN log | Clamp to [50–5000] + WARN |
| `time_budget_ms` | Default 10 | Default + WARN log | Clamp to [0–100] + WARN |
| `lines_per_slice` | Default 256 | Default + WARN log | Clamp to [16–4096] + WARN |
| `starvation_cycle_limit` | Default 10 | Default + WARN log | Clamp to [1–100] + WARN |

---

## Design Decisions

### Decision 1: Cooperative vs. Preemptive Time-Slicing

**Chosen: Cooperative (voluntary yield)**

Rationale:
1. **Single-threaded simplicity**: Work sources run on the main thread within idle callbacks — no need for thread synchronization on mutable document state
2. **Scintilla precedent**: Scintilla's idle styling uses cooperative yielding; proven model for editor idle work
3. **No unsafe required**: Preemptive interruption would require unsafe mechanisms (thread cancellation, signal handling)
4. **Predictable state**: Work sources always reach a consistent save point before yielding, simplifying resumption logic

Trade-offs accepted:
- A misbehaving work source can hold the slice longer than budgeted — mitigated by WARN logging (Requirement 2 AC 3) and the expectation that all built-in sources are well-behaved
- No forced fairness — mitigated by starvation prevention (Requirement 4 AC 6)

### Decision 2: AtomicBool for Cancellation Signal

**Chosen: `std::sync::atomic::AtomicBool` with Acquire/Release ordering**

Rationale:
1. **Sub-millisecond latency** (Requirement 5 AC 5): Atomic loads are nanosecond-level, far below the 1ms requirement
2. **No allocation**: Zero-cost signal mechanism — a single boolean in the scheduler struct
3. **Cross-function visibility**: The flag is set by `input_activity()` and polled by work sources via `IdleWorkContext::is_cancelled()` without needing to pass mutable references
4. **No mutex contention**: Unlike a channel or mutex, atomic reads never block

Trade-offs:
- Only communicates a single boolean (cancel/no-cancel) — sufficient for this use case since we only need "stop now" semantics

### Decision 3: Priority Dispatch with Starvation Prevention

**Chosen: Strict priority with periodic lower-priority grants**

Rationale:
1. **User-visible work first**: Syntax highlighting near the viewport is more impactful than background search indexing — strict priority reflects user perception
2. **Starvation prevention**: Every N cycles (configurable, default 10), the dispatcher bypasses the highest-priority source and services the next-in-line, ensuring all sources eventually make progress
3. **Round-robin within same priority**: If multiple sources share a priority level, they alternate on successive idle callbacks for fairness

Alternative considered: weighted fair queuing — rejected as over-complex for the typical 2–4 work sources in a single editor session.

### Decision 4: Single-Threaded Scheduler (No Background Thread)

**Chosen: Scheduler runs on the main/UI thread within idle callbacks**

Rationale:
1. **Simplest correctness model**: No data races on document content; work sources can safely read document state without locks
2. **Matches Scintilla model**: Scintilla performs all idle work on the UI thread via platform idle callbacks
3. **Sufficient for workloads**: At 10ms per slice, background work progresses at ~100 slices/second during idle — fast enough for styling and wrap calculation
4. **No thread join/cleanup complexity**: The scheduler lifecycle is tied to the application — no orphan thread concerns

Trade-offs:
- Cannot utilize multiple CPU cores for idle work — acceptable since the work is I/O-free and CPU-bound in small increments
- If a platform has genuine background thread support, a future `ThreadedIdleNotifier` could dispatch `on_idle()` on a worker thread (the trait abstraction supports this)

### Decision 5: Object-Safe IdleWorkSource Trait

**Chosen: `dyn IdleWorkSource` in `Box<dyn IdleWorkSource>`**

Rationale:
1. **Heterogeneous collection**: The scheduler must hold work sources of different concrete types (styling, wrap, search) in a single Vec
2. **Dynamic registration**: Work sources are registered at runtime — trait objects are the natural Rust pattern
3. **Plugin support**: Third-party plugins can implement `IdleWorkSource` without the scheduler knowing their concrete types

Constraints this imposes:
- No generic methods on the trait (object safety requirement)
- `perform_work` takes `&mut self` (compatible with object safety)
- No associated types on the trait

---

## Correctness Properties

The following properties are suitable for property-based testing with the `proptest` crate. Each property is universal — it must hold for all valid inputs.

### Property 1: Priority Ordering Invariant

**Statement:** The scheduler always dispatches the highest-priority active (non-dormant, non-complete) work source first, except during starvation prevention cycles.

```
∀ IdleScheduler S, on each on_idle() call where starvation_counter < starvation_cycle_limit:
    let dispatched = source that received the time slice
    ∀ other_source in active_sources where other_source ≠ dispatched:
        dispatched.priority() <= other_source.priority()
```

**Validates: Requirements 4.3, 4.4**

### Property 2: Cancellation Halts Dispatch

**Statement:** After `input_activity()` is called, no further time slices are dispatched until the idle detection threshold elapses again.

```
∀ IdleScheduler S in Active state:
    S.input_activity();
    // No on_idle() dispatch produces WorkStatus results until
    // clock advances by at least idle_detection_threshold
    ∀ t < idle_detection_threshold:
        S.on_idle() dispatches nothing (returns false or skips dispatch)
```

**Validates: Requirements 5.4, 1.1**

### Property 3: Completion Transitions to Inactive

**Statement:** When all registered work sources return `WorkStatus::Complete`, the scheduler transitions to Inactive state and stops requesting idle callbacks.

```
∀ IdleScheduler S:
    if ∀ source in S.registered_sources: source.progress().is_complete:
        S.state == Inactive
        ∧ S.notifier.idle_requested == false
```

**Validates: Requirements 7.2, 11.2, 11.5**

### Property 4: Invalidation Reactivates Source

**Statement:** Invalidating a dormant (complete) work source resets its progress and returns it to the active dispatch set.

```
∀ IdleScheduler S, ∀ source_name where S.progress(source_name).is_complete:
    S.invalidate_source(source_name);
    S.progress(source_name).completed_units == 0
    ∧ S.progress(source_name).is_complete == false
    ∧ S.state ≠ Inactive  // scheduler re-entered active/waiting state
```

**Validates: Requirements 6.6, 7.3**

### Property 5: Register Activates Idle Callbacks

**Statement:** Registering a work source with pending work causes the scheduler to request an idle callback (transition from Inactive to WaitingForIdle).

```
∀ IdleScheduler S in Inactive state, ∀ source where ¬source.progress().is_complete:
    S.register(source);
    S.notifier.idle_requested == true
    ∧ S.state == WaitingForIdle
```

**Validates: Requirements 3.5, 11.3**

### Property 6: Time Budget Bounds Slice Duration

**Statement:** The elapsed time of any `perform_work` invocation is reported accurately, and overruns are logged. The scheduler's own overhead consumes less than 1ms.

```
∀ IdleScheduler S, on each on_idle() call:
    let slice_start = clock.now();
    // ... dispatch happens ...
    let scheduler_overhead = (time of source selection + progress update);
    scheduler_overhead < Duration::from_millis(1)
```

**Validates: Requirements 2.6**

### Property 7: Round-Robin Among Same Priority

**Statement:** When multiple active work sources share the same priority level, they are serviced in round-robin order across successive idle callbacks.

```
∀ IdleScheduler S, ∀ sources A, B where A.priority() == B.priority()
    ∧ A has pending work ∧ B has pending work:
    over 2*N consecutive on_idle() calls (excluding starvation cycles),
    |count(A dispatched) - count(B dispatched)| <= 1
```

**Validates: Requirements 1.5**

### Property 8: Starvation Prevention Guarantee

**Statement:** No active work source goes more than `starvation_cycle_limit` consecutive idle cycles without receiving at least one time slice.

```
∀ IdleScheduler S, ∀ active source X:
    let cycles_since_last_dispatch(X) =
        count of on_idle() calls since X last received a time slice;
    cycles_since_last_dispatch(X) <= S.config.starvation_cycle_limit
```

**Validates: Requirements 4.6**

### Property 9: Unregister Removes Completely

**Statement:** After `unregister(name)`, the source does not receive further time slices, invalidation signals, or appear in progress queries.

```
∀ IdleScheduler S, ∀ name:
    S.unregister(name);
    S.progress(name) == None
    ∧ name never appears in subsequent on_idle() dispatch targets
    ∧ invalidate_source(name) has no effect
```

**Validates: Requirements 3.4, 7.6**

### Property 10: Zero Overhead When Inactive

**Statement:** In Inactive state, the scheduler does not request idle callbacks, does not allocate timer handles, and `on_idle()` is a no-op returning `false`.

```
∀ IdleScheduler S in Inactive state:
    S.on_idle() == false  // no dispatch, immediate return
    ∧ S.notifier.idle_requested == false
```

**Validates: Requirements 11.1, 11.2, 11.4**

### Property 11: Completion Notification Correctness

**Statement:** A completion callback for source X fires exactly once when X transitions from active to complete, and does not fire again until X is invalidated and completes a new work cycle.

```
∀ IdleScheduler S, ∀ source X with subscriber:
    on X returning WorkStatus::Complete:
        subscriber callback invoked exactly once
    on X being invalidated:
        no callback invoked
    on X returning WorkStatus::Complete again (after invalidation):
        subscriber callback invoked exactly once more
```

**Validates: Requirements 10.2, 10.7**

### Property 12: Input Activity Resets Idle Timer

**Statement:** Each call to `input_activity()` resets the idle detection timer. The scheduler does not transition to Active until `idle_detection_threshold` elapses with NO further input.

```
∀ IdleScheduler S, ∀ sequence of input_activity() calls at times t1, t2, ..., tn:
    S transitions to Active only when clock.now() - tn >= idle_detection_threshold
```

**Validates: Requirements 1.1, 5.4**

---

## Testing Strategy

### Unit Test Coverage

| Test File | Covers |
|-----------|--------|
| `scheduler_tests.rs` | State machine transitions, idle detection, `on_idle()` dispatch cycle |
| `priority_tests.rs` | Priority ordering, lower-priority exclusion when higher has work |
| `time_budget_tests.rs` | Budget measurement, overrun WARN logging, budget=0 disables |
| `cancellation_tests.rs` | AtomicBool signal, `is_cancelled()` visibility, interrupt → resume |
| `progress_tests.rs` | `WorkProgress` updates, `is_all_complete()`, progress queries |
| `notification_tests.rs` | Per-source callbacks, global callbacks, no spurious notifications |
| `starvation_tests.rs` | Starvation prevention after N cycles, lower-priority grant |
| `property_tests.rs` | All 12 correctness properties via `proptest` |
| `integration.rs` | Full cycle with multiple mock work sources, config hot-reload |

### Property Test Strategy (proptest)

The property tests use the following generated inputs:
- **Work source configurations**: random priorities (0–100), random completion points
- **Input event sequences**: random timing of `input_activity()` calls relative to idle thresholds
- **Multi-source scenarios**: 2–8 sources with varying priorities and work durations
- **Configuration variants**: threshold from 50–5000ms, budget from 0–100ms

Each property test runs a minimum of 256 cases via `proptest!` configuration.

---

## Cargo.toml Dependencies

```toml
[package]
name = "ff-idle-processing"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "2"
# ff-logging = { path = "../ff-logging" }
# ff-configuration-system = { path = "../ff-configuration-system" }

[dev-dependencies]
proptest = "1"
pretty_assertions = "1"
```

Note: Internal crate dependencies (`ff-logging`, `ff-configuration-system`) are commented out until those crates exist in the workspace. The scheduler is designed to compile and test independently using the trait abstractions.
