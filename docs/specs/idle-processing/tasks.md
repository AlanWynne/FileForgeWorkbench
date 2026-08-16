# Implementation Plan: Idle Processing (`ff-idle-processing`)

## Overview

This task plan implements the `ff-idle-processing` crate — the cooperative idle-time background work scheduler for FileForgeWorkbench. The crate provides idle detection, priority-ordered time-slice dispatch, cooperative cancellation on user input, progress tracking, work source registration/deregistration, and completion notifications. It operates as a GUI-independent coordinator that receives idle callbacks through a trait abstraction, enabling unit testing without a running event loop.

**Crate location:** `crates/ff-idle-processing`
**Upstream dependencies:** `ff-core` (service registry), `ff-logging` (diagnostics), `ff-configuration` (configurable thresholds and budgets)
**Downstream consumers:** `syntax-highlighting` (idle styling), `line-wrap-toggle` (wrap calculation), `find-and-replace` (search index building), `display-line-mapping` (wrap height updates)

---

## Tasks

- [ ] 1. Crate scaffolding and core types
  - [ ] 1.1 Create `crates/ff-idle-processing/Cargo.toml` with dependencies (ff-core, ff-logging, ff-configuration, thiserror) and dev-dependencies (proptest, pretty_assertions, mockall)
  - [ ] 1.2 Create `crates/ff-idle-processing/src/lib.rs` with crate-level doc comment and public module declarations
  - [ ] 1.3 Implement `src/types.rs` — define `WorkPriority` newtype (u32) with Ord/PartialOrd, well-known priority constants: `PRIORITY_SYNTAX_HIGHLIGHT` = 10, `PRIORITY_WRAP_CALCULATION` = 20, `PRIORITY_FOLD_COMPUTATION` = 30, `PRIORITY_SEARCH_INDEX` = 40
  - [ ] 1.4 Implement `src/types.rs` — define `WorkStatus` enum with variants: `Continue` (more work remains), `Complete` (all work finished), `Interrupted` (cancelled by user input, progress saved)
  - [ ] 1.5 Implement `src/types.rs` — define `WorkProgress` struct with fields: `completed_units: u64`, `total_units: u64`, `is_complete: bool`
  - [ ] 1.6 Implement `src/context.rs` — define `IdleWorkContext` struct with methods: `time_remaining() → Duration`, `is_cancelled() → bool`, `elapsed() → Duration`
  - [ ] 1.7 Write unit tests for WorkPriority ordering (lower value = higher priority), WorkStatus variants, WorkProgress construction
    - Validates: Requirement 4 AC 1, AC 2

- [ ] 2. IdleWorkSource trait and IdleNotifier trait
  - [ ] 2.1 Implement `src/work_source.rs` — define `IdleWorkSource` trait with required methods: `perform_work(&mut self, context: &mut IdleWorkContext) → WorkStatus`, `priority(&self) → WorkPriority`, `name(&self) → &str`, `progress(&self) → WorkProgress`
  - [ ] 2.2 Implement optional method `invalidate(&mut self)` on `IdleWorkSource` trait with default no-op implementation that resets progress to beginning
  - [ ] 2.3 Ensure `IdleWorkSource` trait is object-safe (dyn-compatible) — verify `Box<dyn IdleWorkSource>` compiles and can be stored in heterogeneous collections
  - [ ] 2.4 Implement `src/notifier.rs` — define `IdleNotifier` trait with methods: `request_idle_callback(&self)`, `cancel_idle_callback(&self)`
  - [ ] 2.5 Implement `src/notifier.rs` — define `ManualIdleNotifier` struct implementing `IdleNotifier` for test/headless usage, with `trigger_idle()` method that directly invokes scheduler's `on_idle()`
  - [ ] 2.6 Write unit tests verifying `IdleWorkSource` is object-safe (create `Vec<Box<dyn IdleWorkSource>>`, store heterogeneous implementations)
    - Validates: Requirement 3 AC 6
  - [ ] 2.7 Write unit tests for ManualIdleNotifier request/cancel tracking
    - Validates: Requirement 9 AC 7

- [ ] 3. IdleScheduler — core structure and registration API
  - [ ] 3.1 Implement `src/scheduler.rs` — define `IdleScheduler` struct with fields: work source registry (Vec<Box<dyn IdleWorkSource>>), notifier reference, idle detection threshold, time budget, active/dormant state tracking, round-robin index, starvation counter
  - [ ] 3.2 Implement `new(notifier: Box<dyn IdleNotifier>, config: IdleConfig) → IdleScheduler` constructor
  - [ ] 3.3 Implement `register(&mut self, source: Box<dyn IdleWorkSource>)` — add work source to active set, request idle callback if currently idle or in no-op state
  - [ ] 3.4 Implement `unregister(&mut self, name: &str) → Option<Box<dyn IdleWorkSource>>` — remove by name from both active and dormant sets, return ownership, cancel idle callback if no remaining active sources
  - [ ] 3.5 Implement capacity: support at least 8 concurrent work sources without performance degradation in dispatch loop
  - [ ] 3.6 Write unit tests for register/unregister lifecycle (register adds to active set, unregister removes and returns ownership)
    - Validates: Requirement 3 AC 3, AC 4
  - [ ] 3.7 Write unit tests for registering during idle state triggers immediate service on next callback
    - Validates: Requirement 3 AC 5
  - [ ] 3.8 Write unit tests for capacity (register 8+ sources, verify dispatch loop performance)
    - Validates: Requirement 3 AC 7

- [ ] 4. Idle detection and state transitions
  - [ ] 4.1 Implement `input_activity(&mut self)` method — records timestamp of last user input event, cancels current idle state if active, resets idle detection timer
  - [ ] 4.2 Implement idle state transition logic — transition to idle when elapsed time since last `input_activity()` call exceeds Idle_Detection_Threshold (default 200ms, configurable)
  - [ ] 4.3 Implement `src/config.rs` — define `IdleConfig` struct with fields: `idle_detection_threshold_ms` (u32, default 200), `time_budget_ms` (u32, default 10), `lines_per_slice` (u32, default 256)
  - [ ] 4.4 Implement configuration loading from `ff-configuration` system for idle detection threshold
  - [ ] 4.5 Write unit tests for idle state transition after threshold elapses with no input
    - Validates: Requirement 1 AC 1, AC 2
  - [ ] 4.6 Write unit tests for input_activity resetting idle detection timer
    - Validates: Requirement 1 AC 1

- [ ] 5. Time-slice dispatch and priority ordering
  - [ ] 5.1 Implement `on_idle(&mut self) → bool` method — the single entry point invoked by GUI shell's idle callback; dispatches one time slice to highest-priority active work source, returns `true` if more work remains
  - [ ] 5.2 Implement priority selection in dispatch loop — always service the numerically lowest priority (highest-importance) work source with pending work first
  - [ ] 5.3 Implement round-robin dispatch for equal-priority work sources — when multiple sources share same priority, cycle through them across successive idle callbacks
  - [ ] 5.4 Implement time budget tracking within `on_idle()` — measure elapsed time, construct `IdleWorkContext` with remaining budget, signal work source to yield when budget approaches
  - [ ] 5.5 Implement one-time-slice-per-callback rule — dispatch exactly one work source per `on_idle()` invocation
  - [ ] 5.6 Implement continuous idle callback requests — continue requesting callbacks as long as any work source has pending work
  - [ ] 5.7 Implement starvation prevention — if a higher-priority source continuously invalidates without completing, service lower-priority sources at least once every 10 idle cycles
  - [ ] 5.8 Write unit tests for priority-ordered dispatch (lower priority value serviced first)
    - Validates: Requirement 4 AC 3, AC 4
  - [ ] 5.9 Write unit tests for round-robin among equal-priority sources
    - Validates: Requirement 1 AC 5
  - [ ] 5.10 Write unit tests for one time slice per callback invocation
    - Validates: Requirement 1 AC 4
  - [ ] 5.11 Write unit tests for continuous callback requests while work pending
    - Validates: Requirement 1 AC 6
  - [ ] 5.12 Write unit tests for starvation prevention (lower-priority serviced within 10 cycles)
    - Validates: Requirement 4 AC 6

- [ ] 6. Time budget enforcement
  - [ ] 6.1 Implement time budget measurement — track elapsed time during each time slice using `std::time::Instant`, provide `time_remaining()` on `IdleWorkContext`
  - [ ] 6.2 Implement budget overrun detection — when a work source exceeds Time_Budget by more than 2ms, log WARN identifying the offending source and actual elapsed time
  - [ ] 6.3 Implement cooperative enforcement — scheduler does NOT forcibly terminate overrunning work sources; enforcement is via `time_remaining()` query only
  - [ ] 6.4 Implement disabled mode — when Time_Budget is set to 0ms, no idle work is dispatched (scheduler effectively disabled)
  - [ ] 6.5 Implement scheduler overhead budget — ensure scheduler's own bookkeeping (priority selection, progress tracking) consumes less than 1ms of the time budget
  - [ ] 6.6 Write unit tests for time_remaining() accuracy within IdleWorkContext
    - Validates: Requirement 2 AC 2
  - [ ] 6.7 Write unit tests for WARN log on budget overrun (>2ms over)
    - Validates: Requirement 2 AC 3
  - [ ] 6.8 Write unit tests for cooperative-only enforcement (no forced termination)
    - Validates: Requirement 2 AC 4
  - [ ] 6.9 Write unit tests for disabled mode (budget = 0 → no dispatch)
    - Validates: Requirement 2 AC 5
  - [ ] 6.10 Write unit tests for scheduler overhead under 1ms
    - Validates: Requirement 2 AC 6

- [ ] 7. Cancellation on user input
  - [ ] 7.1 Implement cancellation signalling — when `input_activity()` is called during an active time slice, set atomic cancellation flag visible to the active work source via `IdleWorkContext::is_cancelled()`
  - [ ] 7.2 Implement low-latency cancellation — use `AtomicBool` or equivalent mechanism ensuring cancellation signal is visible to work source within less than 1ms (no message queue)
  - [ ] 7.3 Implement post-cancellation cooldown — after cancellation, do not dispatch further time slices until Idle_Detection_Threshold elapses again with no new input
  - [ ] 7.4 Implement interrupted work source handling — when work source returns `WorkStatus::Interrupted`, preserve its priority and progress position for resumption on next idle period
  - [ ] 7.5 Write unit tests for cancellation flag visibility during active slice
    - Validates: Requirement 5 AC 1, AC 2
  - [ ] 7.6 Write unit tests for cancellation latency (atomic flag, sub-1ms visibility)
    - Validates: Requirement 5 AC 5
  - [ ] 7.7 Write unit tests for post-cancellation cooldown (no dispatch until threshold re-elapses)
    - Validates: Requirement 5 AC 4
  - [ ] 7.8 Write unit tests for interrupted source resumption without penalty
    - Validates: Requirement 5 AC 3, AC 6

- [ ] 8. Progress tracking
  - [ ] 8.1 Implement `progress(&self, name: &str) → Option<WorkProgress>` on IdleScheduler — returns current progress of a named work source, or None if not registered
  - [ ] 8.2 Implement `all_progress(&self) → Vec<(String, WorkProgress)>` on IdleScheduler — returns progress for all registered sources
  - [ ] 8.3 Implement `is_all_complete(&self) → bool` on IdleScheduler — returns true only when all registered sources report is_complete = true
  - [ ] 8.4 Implement progress update on WorkStatus::Complete — set source's is_complete = true and exclude from dispatch rotation
  - [ ] 8.5 Implement progress reset on invalidation — set completed_units = 0, is_complete = false, re-include in dispatch rotation
  - [ ] 8.6 Write unit tests for progress() query by name
    - Validates: Requirement 6 AC 2
  - [ ] 8.7 Write unit tests for all_progress() returning all sources
    - Validates: Requirement 6 AC 3
  - [ ] 8.8 Write unit tests for is_all_complete() when all done vs some pending
    - Validates: Requirement 6 AC 5
  - [ ] 8.9 Write unit tests for progress update on completion and reset on invalidation
    - Validates: Requirement 6 AC 4, AC 6

- [ ] 9. Work completion, dormancy, and invalidation
  - [ ] 9.1 Implement dormancy — when work source returns WorkStatus::Complete, mark as dormant and exclude from active dispatch set
  - [ ] 9.2 Implement no-op state — when all sources are dormant, stop requesting idle callbacks (zero CPU overhead)
  - [ ] 9.3 Implement reactivation on invalidation — when dormant source is invalidated, return to active dispatch set, resume requesting idle callbacks
  - [ ] 9.4 Implement `invalidate_source(&mut self, name: &str)` — externally invalidate a specific work source, reset progress, reactivate
  - [ ] 9.5 Implement `invalidate_all(&mut self)` — invalidate all registered work sources simultaneously (full document reload scenario)
  - [ ] 9.6 Implement full removal on unregister — source removed from both active and dormant sets, no further time slices or invalidation signals
  - [ ] 9.7 Implement `SchedulerIdle` notification emission — when last active source completes and scheduler enters no-op state
  - [ ] 9.8 Write unit tests for dormancy on completion (excluded from dispatch)
    - Validates: Requirement 7 AC 1
  - [ ] 9.9 Write unit tests for no-op state (no idle callbacks requested when all dormant)
    - Validates: Requirement 7 AC 2; Requirement 11 AC 2
  - [ ] 9.10 Write unit tests for reactivation on invalidation (dormant → active, callbacks resume)
    - Validates: Requirement 7 AC 3
  - [ ] 9.11 Write unit tests for invalidate_source and invalidate_all
    - Validates: Requirement 7 AC 4, AC 5
  - [ ] 9.12 Write unit tests for full removal on unregister
    - Validates: Requirement 7 AC 6
  - [ ] 9.13 Write unit tests for SchedulerIdle notification on no-op transition
    - Validates: Requirement 7 AC 7

- [ ] 10. Completion notification system
  - [ ] 10.1 Implement `src/notifications.rs` — define `SubscriptionId` type and notification event types: `WorkSourceCompleted { name: String }`, `AllWorkCompleted`
  - [ ] 10.2 Implement `subscribe_completion(&mut self, source_name: &str, callback: Box<dyn Fn()>) → SubscriptionId` — register callback for specific source completion
  - [ ] 10.3 Implement `unsubscribe_completion(&mut self, source_name: &str, subscription_id: SubscriptionId)` — remove callback
  - [ ] 10.4 Implement synchronous callback dispatch — invoke completion callbacks on same thread when source completes
  - [ ] 10.5 Implement notification queue for cross-thread consumers — queue notifications for consumers that poll on next frame
  - [ ] 10.6 Implement spurious notification prevention — when source is invalidated after completion (reactivated), do not emit completion notification until it completes new work cycle
  - [ ] 10.7 Implement AllWorkCompleted notification — emit when scheduler transitions to no-op state
  - [ ] 10.8 Write unit tests for subscribe/unsubscribe lifecycle
    - Validates: Requirement 10 AC 5, AC 6
  - [ ] 10.9 Write unit tests for WorkSourceCompleted notification on completion
    - Validates: Requirement 10 AC 2
  - [ ] 10.10 Write unit tests for AllWorkCompleted notification on no-op transition
    - Validates: Requirement 10 AC 3
  - [ ] 10.11 Write unit tests for synchronous callback and notification queue mechanisms
    - Validates: Requirement 10 AC 4
  - [ ] 10.12 Write unit tests for spurious notification prevention after invalidation
    - Validates: Requirement 10 AC 7

- [ ] 11. No-op state and resource efficiency
  - [ ] 11.1 Implement zero-overhead quiescent state — when no work sources are registered, no idle callbacks requested, no timer handles or pending callbacks allocated
  - [ ] 11.2 Implement immediate callback request on new registration or invalidation from no-op state
  - [ ] 11.3 Implement single-cycle transition — active to no-op occurs within one idle callback cycle after last source reports completion (no trailing callbacks)
  - [ ] 11.4 Implement minimal memory footprint in no-op state — only registration table and per-source progress records retained
  - [ ] 11.5 Write unit tests for zero overhead when no sources registered (no callbacks requested)
    - Validates: Requirement 11 AC 1
  - [ ] 11.6 Write unit tests for immediate activation from no-op on register/invalidation
    - Validates: Requirement 11 AC 3
  - [ ] 11.7 Write unit tests for single-cycle transition to no-op (no trailing callbacks)
    - Validates: Requirement 11 AC 5

- [ ] 12. GUI-independent architecture enforcement
  - [ ] 12.1 Implement GUI-independent time source — use `std::time::Instant` for all timing, with injectable clock trait (`trait Clock { fn now(&self) -> Instant; }`) for deterministic testing
  - [ ] 12.2 Implement `input_activity()` as the sole input state interface — no direct platform input API usage, GUI shell calls this method on each input event
  - [ ] 12.3 Implement public API as struct with well-defined methods — no message passing or GUI event dispatch, direct programmatic control
  - [ ] 12.4 Ensure IdleWorkSource trait has no GUI types in signature — operates on abstract document positions, line numbers, byte offsets only
  - [ ] 12.5 Write unit tests exercising full scheduling logic with mock clock and synthetic input signals (no GUI)
    - Validates: Requirement 12 AC 4
  - [ ] 12.6 Write unit test verifying zero GUI framework dependencies in crate source (no egui, winit, platform-specific APIs)
    - Validates: Requirement 12 AC 1
  - [ ] 12.7 Write unit tests for injectable clock (mock Instant for deterministic time-budget tests)
    - Validates: Requirement 12 AC 2

- [ ] 13. Event loop integration (IdleNotifier)
  - [ ] 13.1 Implement IdleNotifier request/cancel contract — scheduler calls `request_idle_callback()` when active sources exist and not in cancellation cooldown
  - [ ] 13.2 Implement cancel_idle_callback on no-op transition — scheduler calls `cancel_idle_callback()` when entering no-op state to prevent unnecessary event loop overhead
  - [ ] 13.3 Implement `on_idle()` return value semantics — returns bool indicating whether more idle work remains (GUI shell decides whether to re-request)
  - [ ] 13.4 Ensure no direct GUI framework dependency — scheduler receives idle notifications through IdleNotifier trait only
  - [ ] 13.5 Write unit tests for request_idle_callback called when active sources and not in cooldown
    - Validates: Requirement 9 AC 4
  - [ ] 13.6 Write unit tests for cancel_idle_callback on no-op state
    - Validates: Requirement 9 AC 5
  - [ ] 13.7 Write unit tests for on_idle() return value (true when more work, false when all done)
    - Validates: Requirement 9 AC 6
  - [ ] 13.8 Write unit tests for GUI-independence (no framework references in scheduler)
    - Validates: Requirement 9 AC 2

- [ ] 14. Built-in work source category contracts
  - [ ] 14.1 Document and define the contract for syntax-highlighting idle work source — incremental styling from Styling_Position, bounded lines per slice (default 256), respects time budget via `time_remaining()` / `is_cancelled()`
  - [ ] 14.2 Document and define the contract for wrap-calculation idle work source — incremental wrap height computation, updates display-line-mapping via `set_height`, saves progress position across slices
  - [ ] 14.3 Document and define the contract for search-index idle work source — pre-computes match positions for highlight-all, rebuilds entirely on invalidation
  - [ ] 14.4 Implement invalidation contracts — syntax highlighting invalidates to edited line start, wrap calculation invalidates to edited line, search index invalidates entirely
  - [ ] 14.5 Write unit tests for mock syntax-highlighting source respecting time budget (yields when budget exhausted)
    - Validates: Requirement 8 AC 5
  - [ ] 14.6 Write unit tests for mock work sources saving/restoring progress across slices
    - Validates: Requirement 8 AC 6
  - [ ] 14.7 Write unit tests for invalidation to correct position on document edit
    - Validates: Requirement 8 AC 7

- [ ] 15. Property-based tests
  - [ ] 15.1 Write property test: priority ordering invariant (Property 1) — for any set of N work sources with distinct priorities, the scheduler always dispatches the lowest-priority-value source first
    - Validates: Requirement 4 AC 3
  - [ ] 15.2 Write property test: round-robin fairness (Property 2) — for N sources with identical priority, after N idle cycles each source has been serviced exactly once
    - Validates: Requirement 1 AC 5
  - [ ] 15.3 Write property test: time budget bound (Property 3) — for any work source execution, the time_remaining() value provided at context creation equals time_budget minus elapsed scheduler overhead, and elapsed never exceeds budget + 2ms without a WARN log
    - Validates: Requirement 2 AC 2, AC 3
  - [ ] 15.4 Write property test: cancellation atomicity (Property 4) — for any cancellation event during a time slice, is_cancelled() becomes true within less than 1ms and the work source can observe it
    - Validates: Requirement 5 AC 5
  - [ ] 15.5 Write property test: idle detection threshold (Property 5) — for any sequence of input events, the scheduler transitions to idle only after Idle_Detection_Threshold ms of silence, never before
    - Validates: Requirement 1 AC 1, AC 2
  - [ ] 15.6 Write property test: starvation prevention (Property 6) — given a high-priority source that is invalidated on every completion, lower-priority sources are still serviced at least once every 10 idle cycles
    - Validates: Requirement 4 AC 6
  - [ ] 15.7 Write property test: no-op transition completeness (Property 7) — after all sources report Complete, the scheduler enters no-op within exactly one additional idle callback (no trailing callbacks)
    - Validates: Requirement 11 AC 5; Requirement 7 AC 2
  - [ ] 15.8 Write property test: progress monotonicity (Property 8) — for any work source across successive time slices, completed_units is non-decreasing unless invalidated
    - Validates: Requirement 6 AC 1, AC 4
  - [ ] 15.9 Write property test: notification correctness (Property 9) — a completion notification is emitted if and only if a source transitions from non-complete to complete, and never emitted after invalidation until re-completion
    - Validates: Requirement 10 AC 2, AC 7
  - [ ] 15.10 Write property test: dormancy exclusion (Property 10) — dormant (complete) sources never receive time slices until invalidated
    - Validates: Requirement 7 AC 1; Requirement 11 AC 2

- [ ] 16. Integration tests
  - [ ] 16.1 Write integration test: full idle lifecycle — register sources, simulate input silence past threshold, verify on_idle dispatches to highest priority, verify completion and no-op transition
    - Validates: Requirement 1 AC 1, AC 3, AC 4, AC 6, AC 7; Requirement 7 AC 2
  - [ ] 16.2 Write integration test: cancellation mid-slice — start idle processing, inject input event during work source execution, verify is_cancelled() true, verify WorkStatus::Interrupted returned, verify cooldown before next dispatch
    - Validates: Requirement 5 AC 1, AC 2, AC 3, AC 4
  - [ ] 16.3 Write integration test: priority ordering with multiple sources — register 4 sources at different priorities, verify dispatch order matches priority values (lowest first)
    - Validates: Requirement 4 AC 3, AC 4; Requirement 1 AC 4
  - [ ] 16.4 Write integration test: invalidation and reactivation — complete a source, invalidate it, verify it re-enters dispatch rotation and is serviced on next idle
    - Validates: Requirement 7 AC 3, AC 4; Requirement 6 AC 6
  - [ ] 16.5 Write integration test: starvation prevention under continuous invalidation — high-priority source invalidates itself on each completion, verify low-priority sources still serviced within 10 cycles
    - Validates: Requirement 4 AC 6
  - [ ] 16.6 Write integration test: disabled mode (budget = 0) — set time budget to 0, verify no work dispatched even with active sources and idle state
    - Validates: Requirement 2 AC 5
  - [ ] 16.7 Write integration test: notification system end-to-end — subscribe to completion events, run sources to completion, verify callbacks invoked, verify AllWorkCompleted emitted
    - Validates: Requirement 10 AC 2, AC 3, AC 4, AC 5
  - [ ] 16.8 Write integration test: ManualIdleNotifier headless operation — use ManualIdleNotifier to drive scheduler without event loop, verify full scheduling cycle works
    - Validates: Requirement 9 AC 7; Requirement 12 AC 4
  - [ ] 16.9 Write integration test: multiple sources with same priority round-robin — register 3 sources at priority 20, verify each gets serviced in rotation across callbacks
    - Validates: Requirement 1 AC 5
  - [ ] 16.10 Write integration test: scheduler overhead measurement — verify scheduler's own bookkeeping (priority scan, state transitions) takes less than 1ms per on_idle call
    - Validates: Requirement 2 AC 6

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Idle Detection and Scheduling | AC 1 (idle state after threshold with no input) | 4.2, 4.5, 4.6, 15.5, 16.1 |
| Req 1: Idle Detection and Scheduling | AC 2 (threshold configurable, default 200ms) | 4.3, 4.4, 4.5, 15.5 |
| Req 1: Idle Detection and Scheduling | AC 3 (dispatch time slices in priority order) | 5.1, 5.2, 16.1 |
| Req 1: Idle Detection and Scheduling | AC 4 (one time slice per idle callback) | 5.5, 5.10, 16.1, 16.3 |
| Req 1: Idle Detection and Scheduling | AC 5 (round-robin for equal priority) | 5.3, 5.9, 15.2, 16.9 |
| Req 1: Idle Detection and Scheduling | AC 6 (continue requesting callbacks while work pending) | 5.6, 5.11, 16.1 |
| Req 1: Idle Detection and Scheduling | AC 7 (stop callbacks when all complete) | 9.2, 9.9, 16.1 |
| Req 2: Time Budget Enforcement | AC 1 (budget configurable, default 10ms) | 4.3, 6.1 |
| Req 2: Time Budget Enforcement | AC 2 (measure elapsed, signal yield via time_remaining) | 6.1, 6.6, 15.3 |
| Req 2: Time Budget Enforcement | AC 3 (WARN log when overrun > 2ms) | 6.2, 6.7, 15.3 |
| Req 2: Time Budget Enforcement | AC 4 (cooperative only, no forced termination) | 6.3, 6.8 |
| Req 2: Time Budget Enforcement | AC 5 (budget = 0 disables scheduler) | 6.4, 6.9, 16.6 |
| Req 2: Time Budget Enforcement | AC 6 (scheduler overhead < 1ms) | 6.5, 6.10, 16.10 |
| Req 3: Work Source Registration API | AC 1 (IdleWorkSource trait with required methods) | 2.1 |
| Req 3: Work Source Registration API | AC 2 (optional invalidate() method) | 2.2 |
| Req 3: Work Source Registration API | AC 3 (register method adds to active set) | 3.3, 3.6 |
| Req 3: Work Source Registration API | AC 4 (unregister removes and returns ownership) | 3.4, 3.6 |
| Req 3: Work Source Registration API | AC 5 (register during idle starts servicing immediately) | 3.3, 3.7 |
| Req 3: Work Source Registration API | AC 6 (trait is object-safe / dyn-compatible) | 2.3, 2.6 |
| Req 3: Work Source Registration API | AC 7 (support 8+ concurrent sources) | 3.5, 3.8 |
| Req 4: Priority Ordering | AC 1 (WorkPriority: lower value = higher priority) | 1.3, 1.7 |
| Req 4: Priority Ordering | AC 2 (well-known priority constants) | 1.3, 1.7 |
| Req 4: Priority Ordering | AC 3 (dispatch lowest-value first) | 5.2, 5.8, 15.1, 16.3 |
| Req 4: Priority Ordering | AC 4 (service lower priority when higher complete) | 5.2, 5.8, 16.3 |
| Req 4: Priority Ordering | AC 5 (public constants usable by plugins) | 1.3 |
| Req 4: Priority Ordering | AC 6 (starvation prevention: every 10 cycles) | 5.7, 5.12, 15.6, 16.5 |
| Req 5: Cancellation on User Input | AC 1 (input event cancels current slice) | 7.1, 7.5, 16.2 |
| Req 5: Cancellation on User Input | AC 2 (is_cancelled() method on context) | 7.1, 7.5, 16.2 |
| Req 5: Cancellation on User Input | AC 3 (WorkStatus::Interrupted, progress saved) | 7.4, 7.8, 16.2 |
| Req 5: Cancellation on User Input | AC 4 (no dispatch until threshold re-elapses) | 7.3, 7.7, 16.2 |
| Req 5: Cancellation on User Input | AC 5 (latency < 1ms, atomic flag) | 7.2, 7.6, 15.4 |
| Req 5: Cancellation on User Input | AC 6 (no penalty on interrupted source) | 7.4, 7.8 |
| Req 6: Progress Tracking | AC 1 (WorkProgress: completed_units, total_units, is_complete) | 1.5, 15.8 |
| Req 6: Progress Tracking | AC 2 (progress(name) returns Option<WorkProgress>) | 8.1, 8.6 |
| Req 6: Progress Tracking | AC 3 (all_progress() returns Vec) | 8.2, 8.7 |
| Req 6: Progress Tracking | AC 4 (on Complete, set is_complete = true, exclude from dispatch) | 8.4, 8.9, 15.8 |
| Req 6: Progress Tracking | AC 5 (is_all_complete() method) | 8.3, 8.8 |
| Req 6: Progress Tracking | AC 6 (invalidation resets progress) | 8.5, 8.9, 16.4 |
| Req 7: Work Completion and Deregistration | AC 1 (complete → dormant, excluded from dispatch) | 9.1, 9.8, 15.10 |
| Req 7: Work Completion and Deregistration | AC 2 (all dormant → stop callbacks, no-op) | 9.2, 9.9, 15.7, 16.1 |
| Req 7: Work Completion and Deregistration | AC 3 (invalidation reactivates dormant source) | 9.3, 9.10, 16.4 |
| Req 7: Work Completion and Deregistration | AC 4 (invalidate_source by name) | 9.4, 9.11 |
| Req 7: Work Completion and Deregistration | AC 5 (invalidate_all for full reload) | 9.5, 9.11 |
| Req 7: Work Completion and Deregistration | AC 6 (unregister fully removes from all sets) | 9.6, 9.12 |
| Req 7: Work Completion and Deregistration | AC 7 (SchedulerIdle notification on no-op) | 9.7, 9.13 |
| Req 8: Built-In Work Source Categories | AC 1 (syntax-highlighting: 256 lines/slice) | 14.1 |
| Req 8: Built-In Work Source Categories | AC 2 (wrap-calculation: set_height updates) | 14.2 |
| Req 8: Built-In Work Source Categories | AC 3 (fold-level computation) | 14.1 |
| Req 8: Built-In Work Source Categories | AC 4 (search-index building) | 14.3 |
| Req 8: Built-In Work Source Categories | AC 5 (respects time budget per line/batch) | 14.5 |
| Req 8: Built-In Work Source Categories | AC 6 (save progress across slices) | 14.6 |
| Req 8: Built-In Work Source Categories | AC 7 (invalidation to correct position on edit) | 14.4, 14.7 |
| Req 9: Integration with GUI Event Loop | AC 1 (IdleNotifier trait: request/cancel) | 2.4 |
| Req 9: Integration with GUI Event Loop | AC 2 (no direct GUI framework dependency) | 13.4, 13.8 |
| Req 9: Integration with GUI Event Loop | AC 3 (GUI shell implements IdleNotifier) | 2.4 |
| Req 9: Integration with GUI Event Loop | AC 4 (request_idle_callback when active and not in cooldown) | 13.1, 13.5 |
| Req 9: Integration with GUI Event Loop | AC 5 (cancel_idle_callback on no-op) | 13.2, 13.6 |
| Req 9: Integration with GUI Event Loop | AC 6 (on_idle returns bool for more work) | 5.1, 13.3, 13.7 |
| Req 9: Integration with GUI Event Loop | AC 7 (ManualIdleNotifier for headless/test) | 2.5, 2.7, 16.8 |
| Req 10: Work Completion Notification | AC 1 (subscribe by source name) | 10.2 |
| Req 10: Work Completion Notification | AC 2 (WorkSourceCompleted on Complete) | 10.1, 10.9, 15.9, 16.7 |
| Req 10: Work Completion Notification | AC 3 (AllWorkCompleted on no-op) | 10.7, 10.10, 16.7 |
| Req 10: Work Completion Notification | AC 4 (sync callbacks + notification queue) | 10.4, 10.5, 10.11, 16.7 |
| Req 10: Work Completion Notification | AC 5 (subscribe_completion method) | 10.2, 10.8, 16.7 |
| Req 10: Work Completion Notification | AC 6 (unsubscribe_completion method) | 10.3, 10.8 |
| Req 10: Work Completion Notification | AC 7 (no spurious notifications after invalidation) | 10.6, 10.12, 15.9 |
| Req 11: No-Op When All Work Complete | AC 1 (no callbacks when no sources registered) | 11.1, 11.5 |
| Req 11: No-Op When All Work Complete | AC 2 (no callbacks when all dormant) | 9.2, 9.9, 15.10 |
| Req 11: No-Op When All Work Complete | AC 3 (immediate activation from no-op on register/invalidate) | 11.2, 11.6 |
| Req 11: No-Op When All Work Complete | AC 4 (minimal memory footprint in no-op) | 11.4 |
| Req 11: No-Op When All Work Complete | AC 5 (single-cycle transition to no-op) | 11.3, 11.7, 15.7 |
| Req 12: GUI-Independent Architecture | AC 1 (zero GUI framework deps) | 12.6 |
| Req 12: GUI-Independent Architecture | AC 2 (std::time::Instant or injected clock) | 12.1, 12.7 |
| Req 12: GUI-Independent Architecture | AC 3 (input_activity method, no platform input API) | 12.2 |
| Req 12: GUI-Independent Architecture | AC 4 (fully testable without GUI) | 12.5, 16.8 |
| Req 12: GUI-Independent Architecture | AC 5 (struct with methods, no message passing) | 12.3 |
| Req 12: GUI-Independent Architecture | AC 6 (no GUI types in IdleWorkSource signature) | 12.4 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | Priority ordering invariant: for any set of N work sources with distinct priorities, the scheduler always dispatches the lowest-priority-value source first | 15.1 | Req 4.3 |
| P2 | Round-robin fairness: for N sources with identical priority, after N idle cycles each source has been serviced exactly once | 15.2 | Req 1.5 |
| P3 | Time budget bound: time_remaining() at context creation equals time_budget minus elapsed scheduler overhead; overrun > 2ms always produces WARN log | 15.3 | Req 2.2, Req 2.3 |
| P4 | Cancellation atomicity: for any cancellation event during a time slice, is_cancelled() becomes true within < 1ms | 15.4 | Req 5.5 |
| P5 | Idle detection threshold: scheduler transitions to idle only after Idle_Detection_Threshold ms of silence, never before | 15.5 | Req 1.1, Req 1.2 |
| P6 | Starvation prevention: given a high-priority source that invalidates on every completion, lower-priority sources are serviced at least once every 10 cycles | 15.6 | Req 4.6 |
| P7 | No-op transition completeness: after all sources report Complete, scheduler enters no-op within exactly one additional idle callback | 15.7 | Req 11.5, Req 7.2 |
| P8 | Progress monotonicity: for any work source across successive slices, completed_units is non-decreasing unless invalidated | 15.8 | Req 6.1, Req 6.4 |
| P9 | Notification correctness: completion notification emitted iff source transitions non-complete → complete; never emitted after invalidation until re-completion | 15.9 | Req 10.2, Req 10.7 |
| P10 | Dormancy exclusion: dormant (complete) sources never receive time slices until invalidated | 15.10 | Req 7.1, Req 11.2 |

---

## Notes

- Tasks 1 and 2 form the foundation (types + traits) and must be completed before any other task
- Task 3 (scheduler struct + registration) depends on tasks 1–2
- Tasks 4 (idle detection) and 5 (dispatch) depend on task 3 — they implement the scheduler's core loop
- Task 6 (time budget) and task 7 (cancellation) depend on tasks 4–5 as they augment the dispatch loop
- Tasks 8 (progress) and 9 (dormancy/invalidation) depend on task 5 for dispatch integration
- Task 10 (notifications) depends on task 9 for completion/invalidation events
- Tasks 11 (no-op efficiency) and 12 (GUI-independence) are cross-cutting and depend on tasks 4–9
- Task 13 (event loop integration) depends on tasks 2, 5, and 11 for the IdleNotifier contract
- Task 14 (built-in source contracts) depends on tasks 1–7 for the complete trait and scheduler API
- Property tests (task 15) depend on all implementation tasks they validate
- Integration tests (task 16) depend on all preceding implementation tasks
- All tests use the ManualIdleNotifier and injectable clock for deterministic behaviour
- The `proptest` crate is used for all property-based tests with a minimum of 100 iterations
- Mock work sources for testing should implement `IdleWorkSource` with configurable behaviour (completion after N units, priority, simulated work duration)
- The scheduler is single-threaded by design (called from the GUI event loop thread); cancellation uses AtomicBool for the sole cross-thread signal from input handling

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Crate scaffold, core types, and traits", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7"] },
    { "id": 1, "label": "Scheduler structure and registration API", "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8"], "dependsOn": [0] },
    { "id": 2, "label": "Idle detection, dispatch, and configuration", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9", "5.10", "5.11", "5.12"], "dependsOn": [1] },
    { "id": 3, "label": "Time budget enforcement and cancellation", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8"], "dependsOn": [2] },
    { "id": 4, "label": "Progress tracking, dormancy, and invalidation", "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "8.9", "9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9", "9.10", "9.11", "9.12", "9.13"], "dependsOn": [3] },
    { "id": 5, "label": "Notifications, no-op state, and GUI-independence", "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9", "10.10", "10.11", "10.12", "11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7"], "dependsOn": [4] },
    { "id": 6, "label": "Event loop integration and built-in source contracts", "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8", "14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7"], "dependsOn": [5] },
    { "id": 7, "label": "Property-based tests", "tasks": ["15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "15.7", "15.8", "15.9", "15.10"], "dependsOn": [6] },
    { "id": 8, "label": "Integration tests", "tasks": ["16.1", "16.2", "16.3", "16.4", "16.5", "16.6", "16.7", "16.8", "16.9", "16.10"], "dependsOn": [7] }
  ]
}
```
