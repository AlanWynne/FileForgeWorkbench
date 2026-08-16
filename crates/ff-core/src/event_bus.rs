//! # Event Bus — Async-Safe Typed Event Dispatch
//!
//! This module implements the `EventBus`, an async-safe typed event dispatch
//! and subscription system for bidirectional communication between the core
//! layer and the GUI shell.
//!
//! The Event Bus uses bounded channels internally (capacity: 10,000 events)
//! and supports non-blocking dispatch from any thread including Tokio worker
//! threads. It implements an oldest-event-drop overflow policy with WARN-level
//! logging when events are discarded.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Default capacity for the event bus bounded channel.
pub const DEFAULT_EVENT_BUS_CAPACITY: usize = 10_000;

/// Async-safe event dispatch system connecting core to shell and
/// subsystems to each other. Uses bounded broadcast channels internally.
///
/// The `EventBus` wraps a `tokio::sync::broadcast` channel to enable
/// multi-producer, multi-consumer event delivery. Events are wrapped in
/// `Arc<WorkbenchEvent>` to avoid expensive clones across subscribers.
///
/// # Delivery Guarantee
///
/// Events are delivered to all active subscribers within the same logical
/// tick/frame cycle. Because `dispatch()` writes synchronously to the
/// broadcast channel's internal buffer, every subscriber that has already
/// called `subscribe()` or `subscribe_all()` can immediately retrieve the
/// event via `try_recv()` — no deferred or queued delivery across ticks.
///
/// # Overflow Policy
///
/// When the internal buffer reaches capacity, the oldest events are
/// dropped and a cumulative counter is incremented. WARN-level logging
/// is emitted on overflow (implemented in later tasks).
pub struct EventBus {
    /// Broadcast sender for event dispatch.
    sender: broadcast::Sender<Arc<WorkbenchEvent>>,
    /// Cumulative count of dropped events due to overflow.
    /// Wrapped in `Arc` so that `EventSubscription` instances can share
    /// the counter and report lagged (dropped) events back to the bus.
    dropped_count: Arc<AtomicU64>,
    /// Channel capacity (for diagnostics).
    capacity: usize,
}

impl EventBus {
    /// Creates a new `EventBus` with the specified channel capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` — The maximum number of pending events in the broadcast
    ///   channel before oldest events are dropped.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            dropped_count: Arc::new(AtomicU64::new(0)),
            capacity,
        }
    }

    /// Creates a new `EventBus` with the default capacity (10,000 events).
    pub fn with_default_capacity() -> Self {
        Self::new(DEFAULT_EVENT_BUS_CAPACITY)
    }

    /// Returns the cumulative count of dropped events due to overflow.
    pub fn dropped_count(&self) -> u64 {
        self.dropped_count.load(Ordering::Relaxed)
    }

    /// Returns the configured channel capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Dispatch an event to all subscribers. Non-blocking.
    ///
    /// Wraps the event in `Arc` and sends through the broadcast channel.
    /// If no subscribers are currently listening, the event is silently dropped
    /// (this is normal when no GUI shell is connected).
    ///
    /// Returns the number of active receivers that received the event.
    pub fn dispatch(&self, event: WorkbenchEvent) -> usize {
        let arc_event = Arc::new(event);
        self.sender.send(arc_event).unwrap_or_default()
    }

    /// Subscribe to the event bus. Returns a receiver that will get all
    /// dispatched events.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<WorkbenchEvent>> {
        self.sender.subscribe()
    }

    /// Returns a reference to the internal broadcast sender.
    ///
    /// Used by dispatch and subscription methods.
    #[allow(dead_code)]
    pub(crate) fn sender(&self) -> &broadcast::Sender<Arc<WorkbenchEvent>> {
        &self.sender
    }

    /// Increments the dropped event counter by the given amount.
    ///
    /// Called internally when events overflow the bounded channel.
    #[allow(dead_code)]
    pub(crate) fn record_drops(&self, count: u64) {
        self.dropped_count.fetch_add(count, Ordering::Relaxed);
    }
}

// ─── Locally-Defined Event Payload Types ────────────────────────────────────

/// Opaque document identifier. Defined here to avoid layer violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(pub u64);

/// Opaque operation identifier for progress tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationId(pub u64);

/// Parameters passed to a command at dispatch time.
/// Defined locally in ff-core to avoid circular dependency with ff-command.
#[derive(Debug, Clone, Default)]
pub struct CommandParams(pub HashMap<String, ParamValue>);

/// A single parameter value within CommandParams.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    /// A string value.
    String(String),
    /// A 64-bit signed integer value.
    Integer(i64),
    /// A 64-bit floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// A nested map of string keys to parameter values.
    Map(HashMap<String, ParamValue>),
}

/// Outcome of a dispatched command, used in event payloads.
/// Simplified status type defined locally to avoid circular dependency with ff-command.
#[derive(Debug, Clone)]
pub struct CommandOutcome {
    /// Whether the command completed successfully.
    pub success: bool,
    /// Optional human-readable message describing the outcome.
    pub message: Option<String>,
}

/// Severity level for GUI notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationSeverity {
    /// Informational message — no action required.
    Info,
    /// Warning — something unexpected but non-fatal occurred.
    Warning,
    /// Error — an operation failed.
    Error,
}

/// Progress information for long-running operations.
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Human-readable label describing the operation in progress.
    pub label: String,
    /// Completion fraction in [0.0, 1.0], or `None` for indeterminate progress.
    pub fraction: Option<f32>,
    /// Whether the user can cancel this operation.
    pub cancellable: bool,
}

// ─── WorkbenchEvent Enum ────────────────────────────────────────────────────

/// All events that flow through the Event Bus. Categorized per Requirement 3.2.
///
/// Addresses: Requirement 3, criteria 1/2
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WorkbenchEvent {
    // --- Commands (user-initiated operations) ---
    /// A command was dispatched for execution.
    CommandDispatched {
        /// The unique identifier of the command being dispatched.
        command_id: String,
        /// Parameters passed to the command.
        params: CommandParams,
    },
    /// A command completed execution.
    CommandCompleted {
        /// The unique identifier of the command that completed.
        command_id: String,
        /// The outcome of the command execution.
        outcome: CommandOutcome,
    },

    // --- Notifications (informational messages to GUI) ---
    /// Informational message for the status bar or notification area.
    Notification {
        /// The notification message text.
        message: String,
        /// The severity level of the notification.
        severity: NotificationSeverity,
    },

    // --- State-change signals (model updates requiring re-render) ---
    /// A document's content changed.
    DocumentChanged {
        /// The identifier of the document that changed.
        document_id: DocumentId,
    },
    /// The active document/tab changed.
    ActiveDocumentChanged {
        /// The identifier of the newly active document, or `None` if no document is active.
        document_id: Option<DocumentId>,
    },
    /// Configuration was reloaded.
    ConfigReloaded,

    // --- Progress updates (long-running operation status) ---
    /// Progress update for an async operation.
    Progress {
        /// The identifier of the operation reporting progress.
        operation_id: OperationId,
        /// The current progress information.
        progress: ProgressInfo,
    },

    // --- Lifecycle events ---
    /// The workbench has completed startup and is ready for interaction.
    WorkbenchReady,
    /// A shutdown sequence has been initiated.
    ShutdownInitiated,
    /// A plugin was successfully hot-reloaded.
    PluginReloaded {
        /// The name of the plugin that was reloaded.
        plugin_name: String,
    },
}

// ─── Event Categories ───────────────────────────────────────────────────────

/// Event categories for subscription filtering.
///
/// Subscribers can register interest in one or more categories to receive
/// only relevant events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    /// Command dispatch and completion events.
    Command,
    /// Informational notification events.
    Notification,
    /// State-change signals requiring UI updates.
    StateChange,
    /// Progress updates for long-running operations.
    Progress,
    /// Application lifecycle events (ready, shutdown, plugin reload).
    Lifecycle,
}

// ─── WorkbenchEvent Implementation ─────────────────────────────────────────

impl WorkbenchEvent {
    /// Returns the category of this event for subscription filtering.
    pub fn category(&self) -> EventCategory {
        match self {
            Self::CommandDispatched { .. } | Self::CommandCompleted { .. } => {
                EventCategory::Command
            }
            Self::Notification { .. } => EventCategory::Notification,
            Self::DocumentChanged { .. }
            | Self::ActiveDocumentChanged { .. }
            | Self::ConfigReloaded => EventCategory::StateChange,
            Self::Progress { .. } => EventCategory::Progress,
            Self::WorkbenchReady | Self::ShutdownInitiated | Self::PluginReloaded { .. } => {
                EventCategory::Lifecycle
            }
        }
    }
}

// ─── Event Subscription ─────────────────────────────────────────────────────

/// Filter for event subscriptions.
///
/// Subscribers specify a filter to control which events they receive.
/// Events not matching the filter are silently skipped.
#[derive(Debug, Clone)]
pub enum EventFilter {
    /// Receive all events (unfiltered).
    All,
    /// Receive only events matching specific categories.
    Categories(Vec<EventCategory>),
}

/// A subscription handle for receiving filtered events from the EventBus.
///
/// Wraps a broadcast receiver with a filter so that subscribers only
/// receive events they registered interest in. Non-matching events are
/// silently skipped in the receive loop.
pub struct EventSubscription {
    receiver: broadcast::Receiver<Arc<WorkbenchEvent>>,
    filter: EventFilter,
    /// Shared dropped-event counter from the owning `EventBus`.
    /// Incremented when a `Lagged(n)` error is encountered, reflecting
    /// the oldest-event-drop overflow policy.
    dropped_counter: Arc<AtomicU64>,
}

impl EventSubscription {
    /// Await the next event matching this subscription's filter.
    ///
    /// Skips events that don't match the filter. Returns `None` when
    /// the channel is closed (all senders dropped).
    ///
    /// # Lagged receivers
    ///
    /// If this subscription falls behind and events are dropped due to
    /// channel overflow, the dropped events are silently skipped and
    /// reception continues with the next available event.
    pub async fn recv(&mut self) -> Option<Arc<WorkbenchEvent>> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    if self.matches(&event) {
                        return Some(event);
                    }
                    // Skip non-matching events
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // Events were dropped due to overflow — record the count
                    // and continue receiving from the next available event.
                    self.dropped_counter.fetch_add(n, Ordering::Relaxed);
                    ff_logging::log_warn!(
                        "[core] event-bus: {} event(s) dropped due to buffer overflow (total dropped: {})",
                        n,
                        self.dropped_counter.load(Ordering::Relaxed)
                    );
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    }

    /// Try to receive the next matching event without blocking.
    ///
    /// Returns `None` if no matching event is currently available or if
    /// the channel is closed. Silently skips non-matching events and
    /// lagged entries.
    pub fn try_recv(&mut self) -> Option<Arc<WorkbenchEvent>> {
        loop {
            match self.receiver.try_recv() {
                Ok(event) => {
                    if self.matches(&event) {
                        return Some(event);
                    }
                    // Skip non-matching, continue trying
                }
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    // Events were dropped due to overflow — record the count
                    // and continue receiving from the next available event.
                    self.dropped_counter.fetch_add(n, Ordering::Relaxed);
                    ff_logging::log_warn!(
                        "[core] event-bus: {} event(s) dropped due to buffer overflow (total dropped: {})",
                        n,
                        self.dropped_counter.load(Ordering::Relaxed)
                    );
                    continue;
                }
                Err(_) => return None,
            }
        }
    }

    /// Returns the filter this subscription was created with.
    pub fn filter(&self) -> &EventFilter {
        &self.filter
    }

    /// Check whether the given event matches this subscription's filter.
    fn matches(&self, event: &WorkbenchEvent) -> bool {
        match &self.filter {
            EventFilter::All => true,
            EventFilter::Categories(cats) => cats.contains(&event.category()),
        }
    }
}

impl EventBus {
    /// Subscribe to the event bus with a filter. Returns an `EventSubscription`
    /// that only delivers events matching the specified filter.
    ///
    /// # Arguments
    ///
    /// * `filter` — The filter controlling which events this subscription receives.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use ff_core::{EventBus, EventFilter, EventCategory};
    ///
    /// let bus = EventBus::with_default_capacity();
    /// let mut sub = bus.subscribe_filtered(EventFilter::Categories(
    ///     vec![EventCategory::Command, EventCategory::Lifecycle],
    /// ));
    /// ```
    pub fn subscribe_filtered(&self, filter: EventFilter) -> EventSubscription {
        EventSubscription {
            receiver: self.sender.subscribe(),
            filter,
            dropped_counter: Arc::clone(&self.dropped_count),
        }
    }

    /// Subscribe to all events (unfiltered). Convenience method equivalent to
    /// `subscribe_filtered(EventFilter::All)`.
    ///
    /// Used by the GUI shell to receive the full event stream.
    pub fn subscribe_all(&self) -> EventSubscription {
        self.subscribe_filtered(EventFilter::All)
    }
}

// ─── Unit Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // Validates: Requirement 1.2 — interface compiles independently of any GUI crate
    // Validates: Requirement 3.2 — event categories

    // ─── WorkbenchEvent variant construction ────────────────────────────────

    #[test]
    fn workbench_event_command_dispatched_can_be_constructed() {
        let event = WorkbenchEvent::CommandDispatched {
            command_id: "file.open".to_string(),
            params: CommandParams::default(),
        };
        assert!(matches!(event, WorkbenchEvent::CommandDispatched { .. }));
    }

    #[test]
    fn workbench_event_command_completed_can_be_constructed() {
        let event = WorkbenchEvent::CommandCompleted {
            command_id: "file.save".to_string(),
            outcome: CommandOutcome {
                success: true,
                message: Some("Saved successfully".to_string()),
            },
        };
        assert!(matches!(event, WorkbenchEvent::CommandCompleted { .. }));
    }

    #[test]
    fn workbench_event_notification_can_be_constructed() {
        let event = WorkbenchEvent::Notification {
            message: "Hello".to_string(),
            severity: NotificationSeverity::Info,
        };
        assert!(matches!(event, WorkbenchEvent::Notification { .. }));
    }

    #[test]
    fn workbench_event_document_changed_can_be_constructed() {
        let event = WorkbenchEvent::DocumentChanged {
            document_id: DocumentId(42),
        };
        assert!(matches!(event, WorkbenchEvent::DocumentChanged { .. }));
    }

    #[test]
    fn workbench_event_active_document_changed_can_be_constructed() {
        let event = WorkbenchEvent::ActiveDocumentChanged {
            document_id: Some(DocumentId(1)),
        };
        assert!(matches!(
            event,
            WorkbenchEvent::ActiveDocumentChanged { .. }
        ));
    }

    #[test]
    fn workbench_event_config_reloaded_can_be_constructed() {
        let event = WorkbenchEvent::ConfigReloaded;
        assert!(matches!(event, WorkbenchEvent::ConfigReloaded));
    }

    #[test]
    fn workbench_event_progress_can_be_constructed() {
        let event = WorkbenchEvent::Progress {
            operation_id: OperationId(99),
            progress: ProgressInfo {
                label: "Loading...".to_string(),
                fraction: Some(0.5),
                cancellable: true,
            },
        };
        assert!(matches!(event, WorkbenchEvent::Progress { .. }));
    }

    #[test]
    fn workbench_event_workbench_ready_can_be_constructed() {
        let event = WorkbenchEvent::WorkbenchReady;
        assert!(matches!(event, WorkbenchEvent::WorkbenchReady));
    }

    #[test]
    fn workbench_event_shutdown_initiated_can_be_constructed() {
        let event = WorkbenchEvent::ShutdownInitiated;
        assert!(matches!(event, WorkbenchEvent::ShutdownInitiated));
    }

    #[test]
    fn workbench_event_plugin_reloaded_can_be_constructed() {
        let event = WorkbenchEvent::PluginReloaded {
            plugin_name: "syntax-highlight".to_string(),
        };
        assert!(matches!(event, WorkbenchEvent::PluginReloaded { .. }));
    }

    // ─── WorkbenchEvent::category() correctness ─────────────────────────────

    #[test]
    fn category_returns_command_for_command_dispatched() {
        let event = WorkbenchEvent::CommandDispatched {
            command_id: "test".to_string(),
            params: CommandParams::default(),
        };
        assert_eq!(event.category(), EventCategory::Command);
    }

    #[test]
    fn category_returns_command_for_command_completed() {
        let event = WorkbenchEvent::CommandCompleted {
            command_id: "test".to_string(),
            outcome: CommandOutcome {
                success: false,
                message: None,
            },
        };
        assert_eq!(event.category(), EventCategory::Command);
    }

    #[test]
    fn category_returns_notification_for_notification() {
        let event = WorkbenchEvent::Notification {
            message: "msg".to_string(),
            severity: NotificationSeverity::Warning,
        };
        assert_eq!(event.category(), EventCategory::Notification);
    }

    #[test]
    fn category_returns_state_change_for_document_changed() {
        let event = WorkbenchEvent::DocumentChanged {
            document_id: DocumentId(1),
        };
        assert_eq!(event.category(), EventCategory::StateChange);
    }

    #[test]
    fn category_returns_state_change_for_active_document_changed() {
        let event = WorkbenchEvent::ActiveDocumentChanged { document_id: None };
        assert_eq!(event.category(), EventCategory::StateChange);
    }

    #[test]
    fn category_returns_state_change_for_config_reloaded() {
        let event = WorkbenchEvent::ConfigReloaded;
        assert_eq!(event.category(), EventCategory::StateChange);
    }

    #[test]
    fn category_returns_progress_for_progress() {
        let event = WorkbenchEvent::Progress {
            operation_id: OperationId(1),
            progress: ProgressInfo {
                label: "op".to_string(),
                fraction: None,
                cancellable: false,
            },
        };
        assert_eq!(event.category(), EventCategory::Progress);
    }

    #[test]
    fn category_returns_lifecycle_for_workbench_ready() {
        let event = WorkbenchEvent::WorkbenchReady;
        assert_eq!(event.category(), EventCategory::Lifecycle);
    }

    #[test]
    fn category_returns_lifecycle_for_shutdown_initiated() {
        let event = WorkbenchEvent::ShutdownInitiated;
        assert_eq!(event.category(), EventCategory::Lifecycle);
    }

    #[test]
    fn category_returns_lifecycle_for_plugin_reloaded() {
        let event = WorkbenchEvent::PluginReloaded {
            plugin_name: "p".to_string(),
        };
        assert_eq!(event.category(), EventCategory::Lifecycle);
    }

    // ─── EventCategory has all expected variants ────────────────────────────

    #[test]
    fn event_category_has_all_expected_variants() {
        let categories = [
            EventCategory::Command,
            EventCategory::Notification,
            EventCategory::StateChange,
            EventCategory::Progress,
            EventCategory::Lifecycle,
        ];
        // All variants are distinct
        let as_set: HashSet<EventCategory> = categories.iter().copied().collect();
        assert_eq!(as_set.len(), 5);
    }

    // ─── DocumentId and OperationId types ───────────────────────────────────

    #[test]
    fn document_id_construction_and_equality() {
        let id1 = DocumentId(100);
        let id2 = DocumentId(100);
        let id3 = DocumentId(200);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn document_id_hashing_works() {
        let mut set = HashSet::new();
        set.insert(DocumentId(1));
        set.insert(DocumentId(2));
        set.insert(DocumentId(1)); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn operation_id_construction_and_equality() {
        let id1 = OperationId(50);
        let id2 = OperationId(50);
        let id3 = OperationId(51);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn operation_id_hashing_works() {
        let mut set = HashSet::new();
        set.insert(OperationId(10));
        set.insert(OperationId(20));
        set.insert(OperationId(10)); // duplicate
        assert_eq!(set.len(), 2);
    }

    // ─── CommandParams ──────────────────────────────────────────────────────

    #[test]
    fn command_params_default_is_empty() {
        let params = CommandParams::default();
        assert!(params.0.is_empty());
    }

    #[test]
    fn command_params_can_be_populated() {
        let mut params = CommandParams::default();
        params.0.insert(
            "file".to_string(),
            ParamValue::String("/tmp/test.txt".to_string()),
        );
        params.0.insert("line".to_string(), ParamValue::Integer(42));
        assert_eq!(params.0.len(), 2);
        assert_eq!(
            params.0.get("file"),
            Some(&ParamValue::String("/tmp/test.txt".to_string()))
        );
    }

    // ─── ParamValue variants ────────────────────────────────────────────────

    #[test]
    fn param_value_string_variant_works() {
        let val = ParamValue::String("hello".to_string());
        assert_eq!(val, ParamValue::String("hello".to_string()));
    }

    #[test]
    fn param_value_integer_variant_works() {
        let val = ParamValue::Integer(-10);
        assert_eq!(val, ParamValue::Integer(-10));
    }

    #[test]
    fn param_value_float_variant_works() {
        let val = ParamValue::Float(3.14);
        assert_eq!(val, ParamValue::Float(3.14));
    }

    #[test]
    fn param_value_boolean_variant_works() {
        let val = ParamValue::Boolean(true);
        assert_eq!(val, ParamValue::Boolean(true));
        assert_ne!(val, ParamValue::Boolean(false));
    }

    #[test]
    fn param_value_map_variant_works() {
        let mut inner = HashMap::new();
        inner.insert("nested_key".to_string(), ParamValue::Integer(99));
        let val = ParamValue::Map(inner.clone());
        assert_eq!(val, ParamValue::Map(inner));
    }

    // ─── NotificationSeverity variants are distinct ─────────────────────────

    #[test]
    fn notification_severity_variants_are_distinct() {
        assert_ne!(NotificationSeverity::Info, NotificationSeverity::Warning);
        assert_ne!(NotificationSeverity::Warning, NotificationSeverity::Error);
        assert_ne!(NotificationSeverity::Info, NotificationSeverity::Error);
    }

    // ─── ProgressInfo fields ────────────────────────────────────────────────

    #[test]
    fn progress_info_determinate_progress() {
        let info = ProgressInfo {
            label: "Indexing files".to_string(),
            fraction: Some(0.75),
            cancellable: true,
        };
        assert_eq!(info.label, "Indexing files");
        assert_eq!(info.fraction, Some(0.75));
        assert!(info.cancellable);
    }

    #[test]
    fn progress_info_indeterminate_progress() {
        let info = ProgressInfo {
            label: "Connecting...".to_string(),
            fraction: None,
            cancellable: false,
        };
        assert_eq!(info.label, "Connecting...");
        assert_eq!(info.fraction, None);
        assert!(!info.cancellable);
    }

    // ─── EventBus struct construction and capacity ──────────────────────────

    // Validates: Requirement 3.1 — EventBus uses bounded async-capable channel

    #[test]
    fn event_bus_with_default_capacity_has_10000_capacity() {
        let bus = EventBus::with_default_capacity();
        assert_eq!(bus.capacity(), DEFAULT_EVENT_BUS_CAPACITY);
        assert_eq!(bus.capacity(), 10_000);
    }

    #[test]
    fn event_bus_new_with_custom_capacity() {
        let bus = EventBus::new(500);
        assert_eq!(bus.capacity(), 500);
    }

    #[test]
    fn event_bus_dropped_count_starts_at_zero() {
        let bus = EventBus::with_default_capacity();
        assert_eq!(bus.dropped_count(), 0);
    }

    #[test]
    fn event_bus_record_drops_increments_counter() {
        let bus = EventBus::with_default_capacity();
        bus.record_drops(5);
        assert_eq!(bus.dropped_count(), 5);
        bus.record_drops(3);
        assert_eq!(bus.dropped_count(), 8);
    }

    #[test]
    fn event_bus_sender_can_create_receiver() {
        let bus = EventBus::with_default_capacity();
        // Verify we can subscribe (create a receiver) from the sender
        let _receiver = bus.sender().subscribe();
    }

    #[test]
    fn event_bus_broadcast_channel_delivers_events() {
        let bus = EventBus::with_default_capacity();
        let mut receiver = bus.sender().subscribe();

        let event = Arc::new(WorkbenchEvent::WorkbenchReady);
        bus.sender().send(event).expect("send should succeed");

        let received = receiver.try_recv().expect("should receive event");
        assert!(matches!(*received, WorkbenchEvent::WorkbenchReady));
    }

    #[test]
    fn default_event_bus_capacity_constant_is_10000() {
        assert_eq!(DEFAULT_EVENT_BUS_CAPACITY, 10_000);
    }

    // ─── EventBus::dispatch and subscribe ───────────────────────────────────

    // Validates: Requirement 3.1 — bidirectional event flow

    #[test]
    fn dispatch_returns_zero_when_no_subscribers() {
        let bus = EventBus::with_default_capacity();
        let count = bus.dispatch(WorkbenchEvent::WorkbenchReady);
        assert_eq!(count, 0);
    }

    #[test]
    fn dispatch_returns_subscriber_count() {
        let bus = EventBus::with_default_capacity();
        let _rx1 = bus.subscribe();
        let _rx2 = bus.subscribe();
        let count = bus.dispatch(WorkbenchEvent::WorkbenchReady);
        assert_eq!(count, 2);
    }

    #[test]
    fn subscribe_receives_dispatched_event() {
        let bus = EventBus::with_default_capacity();
        let mut rx = bus.subscribe();
        bus.dispatch(WorkbenchEvent::ConfigReloaded);
        let received = rx.try_recv().expect("should receive the dispatched event");
        assert!(matches!(*received, WorkbenchEvent::ConfigReloaded));
    }

    #[test]
    fn bidirectional_event_flow_core_to_gui_and_gui_to_core() {
        // Validates: Requirement 3.1 — bidirectional event flow
        // The same EventBus supports events flowing in both directions:
        // - Core→GUI: state-change events (e.g., DocumentChanged)
        // - GUI→Core: input events (e.g., CommandDispatched)
        let bus = EventBus::with_default_capacity();

        // Simulate a "GUI subscriber" that listens for core state-change events
        let mut gui_rx = bus.subscribe();
        // Simulate a "Core subscriber" that listens for GUI input events
        let mut core_rx = bus.subscribe();

        // Core dispatches a state-change event (Core→GUI direction)
        bus.dispatch(WorkbenchEvent::DocumentChanged {
            document_id: DocumentId(1),
        });

        // Both subscribers receive it (broadcast)
        let gui_event = gui_rx.try_recv().expect("GUI should receive core event");
        assert!(matches!(
            *gui_event,
            WorkbenchEvent::DocumentChanged {
                document_id: DocumentId(1)
            }
        ));
        let core_event = core_rx
            .try_recv()
            .expect("Core should also receive the event");
        assert!(matches!(
            *core_event,
            WorkbenchEvent::DocumentChanged {
                document_id: DocumentId(1)
            }
        ));

        // GUI dispatches a command event (GUI→Core direction)
        bus.dispatch(WorkbenchEvent::CommandDispatched {
            command_id: "file.open".to_string(),
            params: CommandParams::default(),
        });

        // Both subscribers receive the command event
        let gui_event = gui_rx.try_recv().expect("GUI should receive command event");
        assert!(matches!(
            *gui_event,
            WorkbenchEvent::CommandDispatched { .. }
        ));
        let core_event = core_rx
            .try_recv()
            .expect("Core should receive command event");
        assert!(matches!(
            *core_event,
            WorkbenchEvent::CommandDispatched { .. }
        ));
    }

    #[test]
    fn dispatch_silently_drops_event_when_all_receivers_dropped() {
        let bus = EventBus::with_default_capacity();
        // Create and immediately drop a receiver
        let rx = bus.subscribe();
        drop(rx);
        // Dispatch should not panic and should return 0
        let count = bus.dispatch(WorkbenchEvent::ShutdownInitiated);
        assert_eq!(count, 0);
    }

    // ─── Non-blocking dispatch from any thread (Task 6.3) ───────────────────

    #[test]
    fn event_bus_is_send_and_sync() {
        // Validates: Requirement 3.3 — EventBus can be shared across threads
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<EventBus>();
    }

    #[test]
    fn dispatch_from_multiple_threads_is_non_blocking() {
        // Validates: Requirement 3.3 — non-blocking dispatch from any thread
        use std::sync::Arc;
        use std::thread;

        let bus = Arc::new(EventBus::with_default_capacity());
        let mut rx = bus.subscribe();

        let handles: Vec<_> = (0..5)
            .map(|i| {
                let bus = Arc::clone(&bus);
                thread::spawn(move || {
                    bus.dispatch(WorkbenchEvent::Notification {
                        message: format!("from thread {}", i),
                        severity: NotificationSeverity::Info,
                    });
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // All 5 events should have been received
        let mut count = 0;
        while rx.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 5);
    }

    // ─── EventFilter and EventSubscription (Task 7.1) ───────────────────────

    // Validates: Requirement 3.4 — event subscription with interest registration

    #[test]
    fn subscribe_all_receives_every_event() {
        let bus = EventBus::with_default_capacity();
        let mut sub = bus.subscribe_all();

        bus.dispatch(WorkbenchEvent::WorkbenchReady);
        bus.dispatch(WorkbenchEvent::ConfigReloaded);
        bus.dispatch(WorkbenchEvent::Notification {
            message: "test".to_string(),
            severity: NotificationSeverity::Info,
        });

        assert!(sub.try_recv().is_some());
        assert!(sub.try_recv().is_some());
        assert!(sub.try_recv().is_some());
        assert!(sub.try_recv().is_none());
    }

    #[test]
    fn all_subscribers_receive_event_in_same_tick() {
        // Validates: Requirement 3.5 — delivery within same tick/frame cycle
        let bus = EventBus::with_default_capacity();
        let mut sub1 = bus.subscribe_all();
        let mut sub2 = bus.subscribe_all();
        let mut sub3 = bus.subscribe_all();

        bus.dispatch(WorkbenchEvent::WorkbenchReady);

        // All subscribers should have the event available immediately
        assert!(sub1.try_recv().is_some());
        assert!(sub2.try_recv().is_some());
        assert!(sub3.try_recv().is_some());
    }

    #[test]
    fn subscribe_filtered_receives_only_matching_categories() {
        let bus = EventBus::with_default_capacity();
        let mut sub = bus.subscribe_filtered(EventFilter::Categories(vec![EventCategory::Command]));

        // Dispatch events of different categories
        bus.dispatch(WorkbenchEvent::WorkbenchReady); // Lifecycle — should be skipped
        bus.dispatch(WorkbenchEvent::CommandDispatched {
            command_id: "test.cmd".to_string(),
            params: CommandParams::default(),
        }); // Command — should be received
        bus.dispatch(WorkbenchEvent::ConfigReloaded); // StateChange — should be skipped

        let received = sub.try_recv();
        assert!(received.is_some());
        assert!(matches!(
            *received.unwrap(),
            WorkbenchEvent::CommandDispatched { .. }
        ));

        // No more matching events
        assert!(sub.try_recv().is_none());
    }

    #[test]
    fn subscribe_filtered_with_multiple_categories() {
        let bus = EventBus::with_default_capacity();
        let mut sub = bus.subscribe_filtered(EventFilter::Categories(vec![
            EventCategory::Lifecycle,
            EventCategory::Notification,
        ]));

        bus.dispatch(WorkbenchEvent::ConfigReloaded); // StateChange — skipped
        bus.dispatch(WorkbenchEvent::WorkbenchReady); // Lifecycle — received
        bus.dispatch(WorkbenchEvent::Notification {
            message: "hi".to_string(),
            severity: NotificationSeverity::Warning,
        }); // Notification — received
        bus.dispatch(WorkbenchEvent::Progress {
            operation_id: OperationId(1),
            progress: ProgressInfo {
                label: "op".to_string(),
                fraction: None,
                cancellable: false,
            },
        }); // Progress — skipped

        let first = sub.try_recv().unwrap();
        assert!(matches!(*first, WorkbenchEvent::WorkbenchReady));

        let second = sub.try_recv().unwrap();
        assert!(matches!(*second, WorkbenchEvent::Notification { .. }));

        assert!(sub.try_recv().is_none());
    }

    #[test]
    fn subscribe_filtered_with_empty_categories_receives_nothing() {
        let bus = EventBus::with_default_capacity();
        let mut sub = bus.subscribe_filtered(EventFilter::Categories(vec![]));

        bus.dispatch(WorkbenchEvent::WorkbenchReady);
        bus.dispatch(WorkbenchEvent::ConfigReloaded);

        assert!(sub.try_recv().is_none());
    }

    #[test]
    fn event_subscription_filter_accessor_returns_correct_filter() {
        let bus = EventBus::with_default_capacity();
        let sub = bus.subscribe_all();
        assert!(matches!(sub.filter(), EventFilter::All));

        let sub2 = bus.subscribe_filtered(EventFilter::Categories(vec![EventCategory::Progress]));
        match sub2.filter() {
            EventFilter::Categories(cats) => {
                assert_eq!(cats.len(), 1);
                assert_eq!(cats[0], EventCategory::Progress);
            }
            _ => panic!("Expected Categories filter"),
        }
    }

    #[test]
    fn subscribe_filtered_also_counted_in_dispatch_return_value() {
        let bus = EventBus::with_default_capacity();
        let _sub1 = bus.subscribe_all();
        let _sub2 = bus.subscribe_filtered(EventFilter::Categories(vec![EventCategory::Command]));

        // Both subscriptions create broadcast receivers, so dispatch counts both
        let count = bus.dispatch(WorkbenchEvent::WorkbenchReady);
        assert_eq!(count, 2);
    }

    #[test]
    fn multiple_filtered_subscriptions_operate_independently() {
        let bus = EventBus::with_default_capacity();
        let mut cmd_sub =
            bus.subscribe_filtered(EventFilter::Categories(vec![EventCategory::Command]));
        let mut lifecycle_sub =
            bus.subscribe_filtered(EventFilter::Categories(vec![EventCategory::Lifecycle]));

        bus.dispatch(WorkbenchEvent::CommandDispatched {
            command_id: "cmd1".to_string(),
            params: CommandParams::default(),
        });
        bus.dispatch(WorkbenchEvent::WorkbenchReady);

        // cmd_sub should only get the command event
        let cmd_event = cmd_sub.try_recv().unwrap();
        assert!(matches!(
            *cmd_event,
            WorkbenchEvent::CommandDispatched { .. }
        ));
        assert!(cmd_sub.try_recv().is_none());

        // lifecycle_sub should only get the lifecycle event
        let lc_event = lifecycle_sub.try_recv().unwrap();
        assert!(matches!(*lc_event, WorkbenchEvent::WorkbenchReady));
        assert!(lifecycle_sub.try_recv().is_none());
    }

    // ─── Overflow and oldest-event-drop policy (Task 8.2) ───────────────────

    // Validates: Requirement 3.7 — oldest-event-drop policy when buffer is full

    #[test]
    fn overflow_drops_oldest_events_and_increments_dropped_count() {
        // Create a small-capacity EventBus to trigger overflow easily.
        let capacity = 4;
        let bus = EventBus::new(capacity);

        // Subscribe before dispatching so the receiver holds all events.
        let mut sub = bus.subscribe_all();

        // Fill the buffer beyond capacity. The broadcast channel holds
        // `capacity` events; dispatching more causes the oldest to be dropped
        // from slow receivers.
        for i in 0..(capacity + 3) {
            bus.dispatch(WorkbenchEvent::Notification {
                message: format!("event-{}", i),
                severity: NotificationSeverity::Info,
            });
        }

        // The subscriber is now lagged. Calling try_recv will encounter a
        // Lagged error, which should update the shared dropped_count.
        let mut received_count = 0;
        while sub.try_recv().is_some() {
            received_count += 1;
        }

        // The subscriber could not receive all events — some were dropped.
        // The dropped_count on the bus should reflect the overflow.
        let dropped = bus.dropped_count();
        assert!(
            dropped > 0,
            "Expected dropped_count > 0 after overflow, got {}",
            dropped,
        );

        // Total dispatched = capacity + 3 = 7
        // received + dropped should equal total dispatched
        assert_eq!(
            received_count + dropped as usize,
            capacity + 3,
            "received ({}) + dropped ({}) should equal total dispatched ({})",
            received_count,
            dropped,
            capacity + 3,
        );
    }

    #[test]
    fn overflow_tracked_via_subscription_recv_async() {
        // Validates: Requirement 3.7 — dropped events reported through async recv
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let capacity = 4;
            let bus = EventBus::new(capacity);
            let mut sub = bus.subscribe_all();

            // Overflow the buffer
            for i in 0..(capacity + 5) {
                bus.dispatch(WorkbenchEvent::Notification {
                    message: format!("async-event-{}", i),
                    severity: NotificationSeverity::Info,
                });
            }

            // The receiver is lagged. The first call to recv() will encounter
            // Lagged(n) and record the drops, then return the next available event.
            // We drain using try_recv after the first async recv to avoid blocking.
            let first = sub.recv().await;
            assert!(
                first.is_some(),
                "Should receive at least one event after lag"
            );

            let mut received_count = 1;
            while sub.try_recv().is_some() {
                received_count += 1;
            }

            let dropped = bus.dropped_count();
            assert!(
                dropped > 0,
                "Expected dropped_count > 0 after async overflow, got {}",
                dropped,
            );
            assert_eq!(
                received_count + dropped as usize,
                capacity + 5,
                "received ({}) + dropped ({}) should equal total dispatched ({})",
                received_count,
                dropped,
                capacity + 5,
            );
        });
    }

    #[test]
    fn multiple_subscribers_each_report_their_own_lag_to_shared_counter() {
        // Validates: Requirement 3.7 — multiple lagged subscribers accumulate into the same counter
        let capacity = 4;
        let bus = EventBus::new(capacity);

        let mut sub1 = bus.subscribe_all();
        let mut sub2 = bus.subscribe_all();

        // Overflow the buffer — both subscribers will be lagged
        for i in 0..(capacity + 3) {
            bus.dispatch(WorkbenchEvent::Notification {
                message: format!("multi-{}", i),
                severity: NotificationSeverity::Info,
            });
        }

        // Drain both subscribers
        while sub1.try_recv().is_some() {}
        while sub2.try_recv().is_some() {}

        // Each subscriber reports its own lag to the shared counter.
        // Since both are lagged by the same amount, dropped_count should
        // reflect drops from both.
        let dropped = bus.dropped_count();
        assert!(
            dropped > 0,
            "Expected dropped_count > 0 with two lagged subscribers, got {}",
            dropped,
        );
    }
}
