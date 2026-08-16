# Requirements Document

## Introduction

This feature specifies the **Idle Processing** subsystem for FileForgeWorkbench (`ff-idle-processing` crate). The idle-processing scheduler is a **GUI-independent background work coordinator** that grants time slices to registered work sources when no user input is active. It enables computationally intensive operations — syntax re-highlighting beyond the viewport, word-wrap height calculation, fold-level computation, and search index building — to proceed incrementally without blocking user interactions.

The scheduler operates on a cooperative time-slicing model: registered work sources receive bounded time budgets during idle periods and must yield control within their allotted slice. Any user input (keystroke, mouse event, scroll) immediately cancels the current idle work and returns control to the event loop for responsive handling.

This design adapts Scintilla's idle-work mechanism (`IdleWork`, `SetIdle`, `QueueIdleWork`, `IdleStyle`) from its monolithic editor architecture into a generalised, trait-based idle scheduler suitable for the FileForgeWorkbench multi-crate platform. Where Scintilla hard-codes idle tasks (wrap and style), this subsystem provides a registration API for arbitrary work sources with priority ordering.

**Source references:**
- **[SCI-IDLE]** = Scintilla `Editor::IdleWork`, `SetIdle`, `QueueIdleWork`, `StartIdleStyling`, `IdleStyle` — idle-time wrap and styling coordination via platform timer/idle callbacks
- **[WB]** = Workbench Platform Architecture Brief — GUI-independent core, cooperative background processing, responsive UI guarantee

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `syntax-highlighting` | **Work Source** | Registers as an idle work source for background styling of unstyled regions beyond the viewport (idle-time styling). Highest-priority idle work source. |
| `display-line-mapping` | **Consumer** | Receives wrap height updates computed by the wrap calculation work source during idle time. |
| `line-wrap-toggle` | **Work Source** | When wrap mode is active, registers a wrap-height calculation work source that incrementally computes display heights for document lines not yet measured. |
| `find-and-replace` | **Work Source** | Registers a search index building work source that pre-computes match positions for highlight-all-matches mode during idle time. |
| `document-model` | **Dependency** | Provides document content and edit notifications that invalidate idle work progress and trigger re-registration of work sources. |
| `configuration-system` | **Dependency** | Provides configurable parameters: idle detection threshold, time budget per slice, lines-per-slice limits. |
| `platform-core` | **Integration** | The idle scheduler integrates with the platform's event loop abstraction, receiving idle callbacks when the event loop has no pending user events. |

## Glossary

- **Idle_Period**: A span of time during which no user input events (keystrokes, mouse clicks, mouse movement, scroll events) have been received. The idle period begins after the Idle_Detection_Threshold elapses with no input. [SCI-IDLE]
- **Idle_Detection_Threshold**: The configurable duration of input inactivity required before the scheduler considers the application idle and begins granting time slices to work sources. Default: 200 milliseconds. [WB]
- **Time_Slice**: A bounded execution window granted to a single work source during an idle period. The work source must yield control before the Time_Budget expires. [SCI-IDLE, WB]
- **Time_Budget**: The maximum duration of a single Time_Slice. Configurable, default 10 milliseconds. Work sources that exceed this budget risk introducing perceptible input latency. [WB]
- **Work_Source**: A trait implementor that performs incremental background computation during idle time. Each work source has a priority, progress state, and a method to perform a bounded unit of work. [SCI-IDLE, WB]
- **Work_Source_Priority**: An integer value determining the execution order of registered work sources. Lower values indicate higher priority. When multiple work sources have pending work, higher-priority sources are serviced first. [WB]
- **Idle_Scheduler**: The central coordinator that detects idle periods, manages registered work sources, and dispatches time slices according to priority ordering. [SCI-IDLE, WB]
- **Work_Progress**: A value indicating how much work a source has completed relative to its total scope (e.g., "styled to position 5000 of 50000"). Used for progress tracking and completion detection. [SCI-IDLE]
- **Work_Invalidation**: The process of resetting a work source's progress when an event (document edit, configuration change) makes previously completed work stale. [SCI-IDLE]
- **Cancellation_Event**: Any user input event that immediately interrupts the current idle time slice, returning control to the event loop. [SCI-IDLE]
- **Work_Completion**: The state when a work source has no remaining work to perform. A completed work source is deregistered (or suspended) until the next invalidation event re-activates it. [SCI-IDLE]
- **Idle_Callback**: The mechanism by which the GUI event loop notifies the idle scheduler that the application is idle. Implementation varies by platform: `WM_TIMER` on Windows, `g_idle_add` on GTK, `egui::Context::request_repaint_after` in egui. [SCI-IDLE]

---

## Requirements

### Requirement 1: Idle Detection and Scheduling

**User Story:** As the workbench platform, I need an idle-time scheduler that detects when no user input is active and grants execution time to background work, so that computationally expensive operations proceed without blocking user interactions.

**Source:** [SCI-IDLE] `SetIdle`, platform timer/idle integration; [WB] responsive UI guarantee.

#### Acceptance Criteria

1. THE Idle_Scheduler SHALL transition to idle state when no user input events (keystrokes, mouse clicks, mouse movement, scroll wheel events, touch events) have been received for at least the Idle_Detection_Threshold duration.
2. THE Idle_Detection_Threshold SHALL be configurable via the configuration-system, with a default value of 200 milliseconds.
3. WHEN the Idle_Scheduler transitions to idle state, IT SHALL begin dispatching Time_Slices to registered work sources in priority order until all work is complete or a Cancellation_Event occurs.
4. THE Idle_Scheduler SHALL dispatch one Time_Slice per idle callback invocation, servicing the highest-priority work source that has pending work.
5. IF multiple work sources share the same priority level, THE Idle_Scheduler SHALL service them in round-robin order across successive idle callbacks.
6. THE Idle_Scheduler SHALL continue requesting idle callbacks from the event loop as long as any registered work source has pending work.
7. WHEN all registered work sources report completion, THE Idle_Scheduler SHALL stop requesting idle callbacks (no-op state) until the next work source registration or invalidation event.

---

### Requirement 2: Time Budget Enforcement

**User Story:** As a user, I want background processing to never cause perceptible UI lag, so that the editor remains instantly responsive to my keystrokes even while background work is in progress.

**Source:** [SCI-IDLE] bounded idle styling (lines per idle call); [WB] 60fps responsiveness target.

#### Acceptance Criteria

1. THE Time_Budget per idle slice SHALL be configurable via the configuration-system, with a default value of 10 milliseconds.
2. THE Idle_Scheduler SHALL measure elapsed time during each Time_Slice and signal the active work source to yield when the Time_Budget is approaching (providing a time-remaining query).
3. WHEN a work source exceeds the Time_Budget by more than 2 milliseconds, THE Idle_Scheduler SHALL log a WARN-level message identifying the offending work source and the actual elapsed time.
4. THE Idle_Scheduler SHALL NOT forcibly terminate a work source that exceeds the Time_Budget — enforcement is cooperative. The work source is responsible for checking the time-remaining signal and yielding.
5. WHEN the Time_Budget is set to 0 milliseconds, THE Idle_Scheduler SHALL be effectively disabled (no idle work is dispatched), allowing users to disable background processing entirely.
6. THE Time_Budget SHALL apply to the total scheduler overhead plus the work source execution time combined — the scheduler's own bookkeeping (priority selection, progress tracking) SHALL consume less than 1 millisecond of the budget.

---

### Requirement 3: Work Source Registration API

**User Story:** As a subsystem developer (syntax highlighting, wrap calculation, search indexing), I want a trait-based API to register my background work with the idle scheduler, so that I can perform incremental computation during idle time without implementing my own idle detection or event loop integration.

**Source:** [SCI-IDLE] `QueueIdleWork(WorkItems, upTo)`; [WB] trait-based extensibility.

#### Acceptance Criteria

1. THE `ff-idle-processing` crate SHALL define an `IdleWorkSource` trait with the following required methods:
   - `perform_work(context: &mut IdleWorkContext) → WorkStatus` — executes a bounded unit of work within the time budget, returning whether more work remains.
   - `priority() → WorkPriority` — returns the priority level of this work source.
   - `name() → &str` — returns a human-readable identifier for diagnostics and logging.
   - `progress() → WorkProgress` — returns the current progress state for tracking.
2. THE `IdleWorkSource` trait SHALL define an optional method `invalidate()` that resets the work source's progress to the beginning, called when the scheduler detects that previous work is stale.
3. THE Idle_Scheduler SHALL provide a `register(source: Box<dyn IdleWorkSource>)` method that adds a work source to the active set, immediately enabling it to receive time slices during the next idle period.
4. THE Idle_Scheduler SHALL provide an `unregister(name: &str) → Option<Box<dyn IdleWorkSource>>` method that removes a work source by name, returning ownership to the caller.
5. WHEN a work source is registered while the application is already idle, THE Idle_Scheduler SHALL begin servicing it on the next idle callback without waiting for a new idle-detection cycle.
6. THE `IdleWorkSource` trait SHALL be object-safe (`dyn`-compatible) to support heterogeneous collections of work sources within the scheduler.
7. THE Idle_Scheduler SHALL support registering multiple work sources simultaneously (at least 8 concurrent work sources) without performance degradation in the dispatch loop.

---

### Requirement 4: Priority Ordering of Work Sources

**User Story:** As the workbench architect, I want idle work to be processed in a defined priority order, so that the most user-visible improvements (syntax highlighting the viewport's surroundings) happen before less visible work (search index building for a background find).

**Source:** [SCI-IDLE] Scintilla processes wrap before idle styling; [WB] prioritised background work.

#### Acceptance Criteria

1. THE Idle_Scheduler SHALL define a `WorkPriority` type as a numeric value where lower values indicate higher priority (priority 0 is highest).
2. THE Idle_Scheduler SHALL define well-known priority constants for built-in work source categories:
   - `PRIORITY_SYNTAX_HIGHLIGHT` = 10 (syntax re-highlighting beyond viewport)
   - `PRIORITY_WRAP_CALCULATION` = 20 (word-wrap height measurement)
   - `PRIORITY_FOLD_COMPUTATION` = 30 (fold-level computation for collapsed regions)
   - `PRIORITY_SEARCH_INDEX` = 40 (search index building for find-all)
3. WHEN multiple work sources have pending work, THE Idle_Scheduler SHALL always service the source with the numerically lowest (highest-priority) priority value first.
4. WHEN all higher-priority work sources are complete or have no pending work, THE Idle_Scheduler SHALL proceed to service lower-priority work sources.
5. THE priority constants SHALL be public and usable by plugin-provided work sources that need to position themselves relative to built-in priorities (e.g., a plugin using priority 25 runs after syntax highlighting but before fold computation).
6. THE Idle_Scheduler SHALL NOT starve lower-priority work sources indefinitely: if a higher-priority source continuously invalidates without completing, the scheduler SHALL service lower-priority sources at least once every 10 idle cycles (starvation prevention).

---

### Requirement 5: Cancellation on User Input

**User Story:** As a user, I want any keystroke or mouse event to immediately interrupt background processing, so that my input is handled with zero additional latency from idle work.

**Source:** [SCI-IDLE] Idle work interrupted by any user action; `SetIdle(false)` on input; [WB] input-first responsiveness.

#### Acceptance Criteria

1. WHEN any user input event occurs (keystroke, mouse click, mouse movement, scroll wheel, touch, window resize), THE Idle_Scheduler SHALL immediately cancel the current Time_Slice by signalling the active work source to yield.
2. THE cancellation signal SHALL be delivered via the `IdleWorkContext::is_cancelled()` method that work sources poll during their `perform_work` execution. Work sources SHALL check this method at least once per significant unit of work (e.g., per line processed).
3. WHEN a work source detects cancellation via `is_cancelled()`, IT SHALL save its current progress position and return `WorkStatus::Interrupted` from `perform_work`, enabling resumption from the same position on the next idle period.
4. AFTER a cancellation event, THE Idle_Scheduler SHALL NOT dispatch any further Time_Slices until the Idle_Detection_Threshold has elapsed again with no new input, re-entering idle state.
5. THE latency between a user input event arriving and the cancellation signal being visible to the work source SHALL be less than 1 millisecond (the signal must be an atomic flag or equivalent low-latency mechanism, not a message queue).
6. WHEN a work source is interrupted by cancellation, THE Idle_Scheduler SHALL NOT penalise it or change its priority — it resumes normally on the next idle period.

---

### Requirement 6: Progress Tracking

**User Story:** As the workbench platform, I want to track progress of each idle work source, so that consumers can query completion status, display progress indicators, and detect when background work finishes.

**Source:** [SCI-IDLE] `GetEndStyled` progress tracking, `needIdleStyling` completion flag; [WB] observable progress.

#### Acceptance Criteria

1. THE `WorkProgress` type SHALL contain: `completed_units` (amount of work done), `total_units` (total work scope), and `is_complete` (boolean indicating all work is finished).
2. THE Idle_Scheduler SHALL provide a `progress(name: &str) → Option<WorkProgress>` method that returns the current progress of a named work source, or `None` if no such source is registered.
3. THE Idle_Scheduler SHALL provide a `all_progress() → Vec<(String, WorkProgress)>` method that returns progress information for all registered work sources, enabling a unified progress display.
4. WHEN a work source's `perform_work` method returns `WorkStatus::Complete`, THE Idle_Scheduler SHALL update the source's progress to `is_complete = true` and exclude it from future time-slice dispatch until it is invalidated.
5. THE Idle_Scheduler SHALL provide a `is_all_complete() → bool` method that returns `true` only when all registered work sources report `is_complete = true`, indicating no pending idle work exists.
6. WHEN a work source is invalidated (via `invalidate()`), THE Idle_Scheduler SHALL reset its progress to `completed_units = 0, is_complete = false` and re-include it in the dispatch rotation.

---

### Requirement 7: Work Completion and Deregistration

**User Story:** As a subsystem developer, I want completed work sources to automatically become dormant until their work is invalidated again, so that the idle scheduler does not waste cycles polling sources with nothing to do.

**Source:** [SCI-IDLE] `needIdleStyling = false` stops idle callbacks; `SetIdle(false)` when all done; [WB] efficient resource usage.

#### Acceptance Criteria

1. WHEN a work source returns `WorkStatus::Complete` from `perform_work`, THE Idle_Scheduler SHALL mark it as dormant and exclude it from the active dispatch set.
2. WHEN all registered work sources are dormant (complete), THE Idle_Scheduler SHALL stop requesting idle callbacks from the event loop, entering a no-op state with zero CPU overhead.
3. WHEN a dormant work source is invalidated (e.g., due to a document edit), THE Idle_Scheduler SHALL reactivate it, return it to the dispatch set, and resume requesting idle callbacks.
4. THE Idle_Scheduler SHALL provide a `invalidate_source(name: &str)` method that externally invalidates a specific work source, resetting its progress and reactivating it.
5. THE Idle_Scheduler SHALL provide a `invalidate_all()` method that invalidates all registered work sources simultaneously (used after operations that affect the entire document, such as encoding change or full reload).
6. WHEN a work source is unregistered via `unregister()`, IT SHALL be fully removed from both the active and dormant sets — it will not receive further time slices or invalidation signals.
7. WHEN the last active work source completes and the scheduler enters no-op state, IT SHALL emit a `SchedulerIdle` notification enabling consumers to know that all background processing is finished.

---

### Requirement 8: Registered Work Sources — Built-In Categories

**User Story:** As the workbench integrator, I need well-defined idle work sources for core background tasks (syntax highlighting, wrap calculation, fold computation, search indexing), so that these subsystems leverage idle processing without each implementing their own idle detection.

**Source:** [SCI-IDLE] Idle wrapping + idle styling as built-in idle tasks; [WB] platform-provided background processing for core features.

#### Acceptance Criteria

1. THE `syntax-highlighting` crate SHALL register an idle work source that incrementally styles unstyled document regions beyond the current viewport, starting from the current Styling_Position and advancing forward by a bounded number of lines per time slice (configurable, default 256 lines).
2. THE `line-wrap-toggle` crate SHALL register an idle work source (when wrap mode is active) that incrementally computes Wrap_Height for document lines not yet measured, updating `display-line-mapping` via `set_height` calls as results become available.
3. THE `syntax-highlighting` crate MAY register a fold-level computation idle work source that computes fold levels for regions beyond the viewport, enabling fold indicators to appear progressively as the user scrolls.
4. THE `find-and-replace` crate MAY register an idle work source for search index building when highlight-all-matches mode is active, pre-computing match positions for the entire document during idle time rather than blocking on the initial search command.
5. EACH built-in work source SHALL respect the time budget by checking `IdleWorkContext::time_remaining()` or `is_cancelled()` after processing each line or small batch of lines, yielding immediately when the budget is exhausted.
6. EACH built-in work source SHALL save its progress position (e.g., last-styled byte offset, last-measured line number) across time slices, enabling seamless resumption without restarting from the beginning.
7. WHEN a document edit occurs, THE relevant work sources SHALL be invalidated to a position no later than the edit point: syntax highlighting invalidates to the edited line's start, wrap calculation invalidates to the edited line, search index invalidates entirely (full rebuild).

---

### Requirement 9: Integration with GUI Event Loop

**User Story:** As the workbench GUI shell (egui-based), I need the idle scheduler to integrate with my event loop so that idle callbacks are dispatched at the correct times without requiring a dedicated background thread for simple idle work.

**Source:** [SCI-IDLE] Platform-specific idle integration (WM_TIMER on Win32, g_idle_add on GTK); [WB] GUI-independent core with pluggable event loop integration.

#### Acceptance Criteria

1. THE Idle_Scheduler SHALL define an `IdleNotifier` trait that the GUI shell implements to provide event-loop integration:
   - `request_idle_callback()` — requests the event loop to invoke the scheduler's `on_idle()` method when the application becomes idle.
   - `cancel_idle_callback()` — cancels a previously requested idle callback.
2. THE Idle_Scheduler SHALL NOT directly depend on any GUI framework (no egui, no winit, no GTK references) — it receives idle notifications through the `IdleNotifier` trait abstraction.
3. THE GUI shell SHALL implement `IdleNotifier` using the platform's idle mechanism: `egui::Context::request_repaint_after(Duration)` for egui, `WM_TIMER` with low priority for Win32, `g_idle_add` for GTK.
4. WHEN the Idle_Scheduler has active work sources and is not currently in a cancellation cooldown, IT SHALL call `request_idle_callback()` on the `IdleNotifier` to ensure it will be invoked.
5. WHEN the Idle_Scheduler enters no-op state (all work complete), IT SHALL call `cancel_idle_callback()` to prevent unnecessary event loop overhead.
6. THE `on_idle()` method SHALL be the single entry point invoked by the GUI shell's idle callback; it SHALL perform one time-slice dispatch cycle and return a boolean indicating whether more idle work remains (enabling the GUI shell to decide whether to re-request).
7. THE Idle_Scheduler SHALL be usable in headless/test mode by providing a `ManualIdleNotifier` implementation that allows tests to trigger `on_idle()` directly without a real event loop.

---

### Requirement 10: Work Completion Notification

**User Story:** As a consumer of background work results (minimap, scrollbar decoration, search results panel), I want to be notified when idle work completes for a given source, so that I can update my display with the newly available data without polling.

**Source:** [SCI-IDLE] Notification when idle styling reaches end; [WB] observer-based notification.

#### Acceptance Criteria

1. THE Idle_Scheduler SHALL support a notification mechanism where consumers can subscribe to completion events for specific work sources by name.
2. WHEN a work source returns `WorkStatus::Complete`, THE Idle_Scheduler SHALL emit a `WorkSourceCompleted { name: String }` notification to all subscribers registered for that source.
3. WHEN the scheduler transitions to no-op state (all sources complete), IT SHALL emit a `AllWorkCompleted` notification to subscribers of the global completion event.
4. THE notification mechanism SHALL support both synchronous callbacks (for same-thread consumers) and a notification queue (for cross-thread consumers that poll on the next frame).
5. THE Idle_Scheduler SHALL provide a `subscribe_completion(source_name: &str, callback: Box<dyn Fn()>)` method for registering completion callbacks.
6. THE Idle_Scheduler SHALL provide an `unsubscribe_completion(source_name: &str, subscription_id: SubscriptionId)` method for removing previously registered callbacks.
7. WHEN a work source is invalidated after completion (reactivated), subscribers SHALL NOT receive spurious completion notifications until the source completes its new work cycle.

---

### Requirement 11: No-Op When All Work Complete

**User Story:** As the workbench platform, I want the idle scheduler to have zero overhead when no background work is pending, so that battery life and system resources are not wasted when the editor is at rest.

**Source:** [SCI-IDLE] `SetIdle(false)` stops idle timer when no work needed; [WB] efficient resource usage.

#### Acceptance Criteria

1. WHEN no work sources are registered, THE Idle_Scheduler SHALL NOT request idle callbacks from the event loop — zero CPU overhead in the quiescent state.
2. WHEN all registered work sources are dormant (complete), THE Idle_Scheduler SHALL NOT request idle callbacks — equivalent to no sources registered from a resource perspective.
3. WHEN the Idle_Scheduler is in no-op state and a new work source is registered (or an existing source is invalidated), IT SHALL immediately request an idle callback to begin processing after the Idle_Detection_Threshold.
4. IN no-op state, THE Idle_Scheduler's memory footprint SHALL be limited to the registration table and per-source progress records — no timer handles, no pending callbacks, no allocated work buffers.
5. THE transition from active state to no-op state SHALL occur within one idle callback cycle after the last source reports completion — no unnecessary trailing callbacks.

---

### Requirement 12: GUI-Independent Architecture

**User Story:** As a workbench platform developer, I want the idle-processing crate to operate without any GUI framework dependency, so that it can be unit-tested, used in headless mode, and remain decoupled from the specific GUI shell implementation.

**Source:** [WB] GUI-independent core architecture; separation of concerns.

#### Acceptance Criteria

1. THE `ff-idle-processing` crate SHALL have zero dependencies on any GUI framework (no egui, no winit, no platform-specific windowing APIs).
2. THE Idle_Scheduler SHALL receive time information through `std::time::Instant` (or an injected clock trait for testing) rather than through GUI framework timers.
3. THE Idle_Scheduler SHALL receive user input state through an `input_activity()` method called by the GUI shell on each input event, rather than directly listening to platform input APIs.
4. THE crate SHALL be fully testable without a running GUI: unit tests SHALL exercise scheduling logic, priority dispatch, time budget enforcement, and cancellation using a mock clock and synthetic input signals.
5. THE Idle_Scheduler SHALL expose its public API through a struct with well-defined methods (not through message passing or GUI event dispatch), enabling direct programmatic control from any consumer.
6. THE `IdleWorkSource` trait SHALL not reference any GUI types in its signature — it operates on abstract document positions, line numbers, and byte offsets only.
