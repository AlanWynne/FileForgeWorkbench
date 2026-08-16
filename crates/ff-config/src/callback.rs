//! Reload callback management.
//!
//! Manages registration, deregistration, and invocation of callbacks that
//! subsystems register to be notified when configuration keys they depend on
//! change during hot-reload.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use crate::reload::ReloadEvent;

/// A callback invoked when configuration keys change during hot-reload.
///
/// Subsystems register callbacks of this type to be notified when the
/// configuration keys they depend on are modified. The callback receives
/// a reference to the `ReloadEvent` describing what changed.
///
/// Callbacks must be `Send + Sync` to support invocation from the
/// file-watcher thread and registration from any thread.
pub type ReloadCallback = Box<dyn Fn(&ReloadEvent) + Send + Sync>;

/// Handle returned when registering a callback; used for deregistration.
///
/// Each handle wraps a unique `u64` identifier assigned at registration time.
/// Pass this handle to `remove_callback()` to deregister the associated callback.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallbackHandle(pub(crate) u64);

impl CallbackHandle {
    /// Creates a new `CallbackHandle` with the given identifier.
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying identifier.
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// A registered callback entry storing the handle, watched keys, and callback function.
///
/// The callback is stored as an `Arc` so that references can be cloned out of
/// the locked section and invoked without holding the registry lock.
struct CallbackEntry {
    /// The handle used for deregistration.
    handle: CallbackHandle,
    /// The set of key filters this callback is interested in.
    watched_keys: Vec<String>,
    /// The callback function to invoke on reload.
    ///
    /// Stored as `Arc` (converted from the `Box` provided at registration) so
    /// that invoke can clone references before releasing the lock.
    callback: Arc<dyn Fn(&ReloadEvent) + Send + Sync>,
}

/// Thread-safe registry for managing reload callbacks.
///
/// Subsystems register callbacks via [`on_reload`](CallbackRegistry::on_reload),
/// specifying which configuration keys they want to be notified about. The
/// registry assigns a unique [`CallbackHandle`] to each registration, which
/// can later be used to deregister the callback.
///
/// Thread safety is provided by an internal `Mutex` protecting the callback
/// list, and an `AtomicU64` counter for lock-free handle ID generation.
pub struct CallbackRegistry {
    /// Atomic counter for generating unique callback handle IDs.
    next_id: AtomicU64,
    /// The list of registered callbacks, protected by a mutex.
    entries: Mutex<Vec<CallbackEntry>>,
}

impl CallbackRegistry {
    /// Creates a new, empty `CallbackRegistry`.
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Register a reload callback for specific keys.
    ///
    /// The callback is invoked when any of the specified keys' effective values
    /// change during hot-reload. Returns a [`CallbackHandle`] that can be used
    /// to deregister the callback later via [`remove_callback`](CallbackRegistry::remove_callback).
    ///
    /// # Arguments
    ///
    /// * `keys` — The configuration keys to watch. The callback will be invoked
    ///   when any of these keys change.
    /// * `callback` — The function to invoke on reload events affecting the
    ///   watched keys.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let registry = CallbackRegistry::new();
    /// let handle = registry.on_reload(
    ///     &["editor.tab_size", "editor.indent_style"],
    ///     Box::new(|event| {
    ///         println!("Editor settings changed: {:?}", event.changed_keys);
    ///     }),
    /// );
    /// ```
    pub fn on_reload(&self, keys: &[&str], callback: ReloadCallback) -> CallbackHandle {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let handle = CallbackHandle::new(id);

        let entry = CallbackEntry {
            handle: handle.clone(),
            watched_keys: keys.iter().map(|k| (*k).to_string()).collect(),
            callback: Arc::from(callback),
        };

        let mut entries = self
            .entries
            .lock()
            .expect("callback registry lock poisoned");
        entries.push(entry);

        handle
    }

    /// Deregister a previously registered reload callback.
    ///
    /// Removes the callback associated with the given handle. If no callback
    /// matches the handle, the call is a no-op (no error is returned).
    ///
    /// # Arguments
    ///
    /// * `handle` — The handle returned by a prior call to
    ///   [`on_reload`](CallbackRegistry::on_reload).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let registry = CallbackRegistry::new();
    /// let handle = registry.on_reload(
    ///     &["editor.tab_size"],
    ///     Box::new(|_event| {}),
    /// );
    /// registry.remove_callback(handle);
    /// assert!(registry.is_empty());
    /// ```
    pub fn remove_callback(&self, handle: CallbackHandle) {
        let mut entries = self
            .entries
            .lock()
            .expect("callback registry lock poisoned");
        entries.retain(|entry| entry.handle != handle);
    }

    /// Returns the number of currently registered callbacks.
    pub fn len(&self) -> usize {
        let entries = self
            .entries
            .lock()
            .expect("callback registry lock poisoned");
        entries.len()
    }

    /// Returns `true` if no callbacks are registered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Deregister all callbacks.
    ///
    /// Removes every registered callback from the registry. Used during
    /// system shutdown to ensure no stale callbacks remain.
    pub fn clear_all(&self) {
        let mut entries = self
            .entries
            .lock()
            .expect("callback registry lock poisoned");
        entries.clear();
    }

    /// Invoke all callbacks whose watched keys overlap with the event's changed keys.
    ///
    /// A callback is invoked if ANY of its watched keys appears in
    /// `event.changed_keys`. Callbacks whose watched keys have no overlap
    /// with the changed keys are skipped.
    ///
    /// The lock on the internal entries list is held only while cloning
    /// references needed for invocation; callbacks themselves run without
    /// holding the lock. This prevents deadlocks if a callback re-enters
    /// the registry (e.g., to register another callback or query `len()`).
    pub fn invoke(&self, event: &ReloadEvent) {
        // Step 1: Acquire the lock and collect Arc references for matching callbacks.
        #[allow(clippy::type_complexity)]
        let to_invoke: Vec<Arc<dyn Fn(&ReloadEvent) + Send + Sync>> = {
            let entries = self
                .entries
                .lock()
                .expect("callback registry lock poisoned");
            entries
                .iter()
                .filter(|entry| {
                    entry
                        .watched_keys
                        .iter()
                        .any(|key| event.changed_keys.contains(key))
                })
                .map(|entry| Arc::clone(&entry.callback))
                .collect()
        };
        // Step 2: Lock is released here. Invoke callbacks without holding the lock.
        for callback in &to_invoke {
            callback(event);
        }
    }
}

impl Default for CallbackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::SystemTime;

    use crate::layer::ConfigLayer;
    use crate::reload::ReloadEvent;

    /// Helper to create a dummy ReloadEvent for testing.
    fn dummy_event(keys: &[&str]) -> ReloadEvent {
        ReloadEvent {
            changed_keys: keys.iter().map(|k| (*k).to_string()).collect(),
            source_layer: ConfigLayer::User,
            timestamp: SystemTime::now(),
        }
    }

    // Validates: Requirement 3.4 — on_reload registers a callback and returns a handle
    #[test]
    fn on_reload_returns_unique_handle() {
        let registry = CallbackRegistry::new();

        let handle1 = registry.on_reload(&["editor.tab_size"], Box::new(|_event| {}));
        let handle2 = registry.on_reload(&["logging.level"], Box::new(|_event| {}));

        assert_ne!(handle1.id(), handle2.id());
        assert_eq!(registry.len(), 2);
    }

    // Validates: Requirement 3.4 — each handle has a unique ID
    #[test]
    fn handles_have_monotonically_increasing_ids() {
        let registry = CallbackRegistry::new();

        let h1 = registry.on_reload(&["a"], Box::new(|_| {}));
        let h2 = registry.on_reload(&["b"], Box::new(|_| {}));
        let h3 = registry.on_reload(&["c"], Box::new(|_| {}));

        assert!(h1.id() < h2.id());
        assert!(h2.id() < h3.id());
    }

    // Validates: Requirement 3.3 — callback stores watched keys correctly
    #[test]
    fn on_reload_stores_multiple_watched_keys() {
        let registry = CallbackRegistry::new();

        let _handle = registry.on_reload(
            &["editor.tab_size", "editor.indent_style", "logging.level"],
            Box::new(|_event| {}),
        );

        assert_eq!(registry.len(), 1);
    }

    // Validates: Requirement 3.4 — registry is thread-safe for concurrent registration
    #[test]
    fn concurrent_registration_is_safe() {
        let registry = Arc::new(CallbackRegistry::new());
        let mut handles = Vec::new();

        for i in 0..10 {
            let reg = Arc::clone(&registry);
            let handle = std::thread::spawn(move || {
                let key = format!("key.{}", i);
                reg.on_reload(&[key.as_str()], Box::new(|_| {}))
            });
            handles.push(handle);
        }

        let results: Vec<CallbackHandle> = handles
            .into_iter()
            .map(|h| h.join().expect("thread panicked"))
            .collect();

        // All handles should be unique
        let ids: std::collections::HashSet<u64> = results.iter().map(|h| h.id()).collect();
        assert_eq!(ids.len(), 10);
        assert_eq!(registry.len(), 10);
    }

    // Validates: Requirement 3.3 — empty registry has zero callbacks
    #[test]
    fn new_registry_is_empty() {
        let registry = CallbackRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    // Validates: Requirement 3.4 — remove_callback decreases callback count
    #[test]
    fn remove_callback_decreases_count() {
        let registry = CallbackRegistry::new();

        let handle1 = registry.on_reload(&["editor.tab_size"], Box::new(|_| {}));
        let _handle2 = registry.on_reload(&["logging.level"], Box::new(|_| {}));
        assert_eq!(registry.len(), 2);

        registry.remove_callback(handle1);
        assert_eq!(registry.len(), 1);
    }

    // Validates: Requirement 3.4 — removed callback is no longer invoked
    #[test]
    fn removed_callback_is_not_invoked() {
        let registry = CallbackRegistry::new();
        let invoked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let invoked_clone = Arc::clone(&invoked);

        let handle = registry.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                invoked_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        registry.remove_callback(handle);

        // After removal, iterating entries should not find the removed callback
        let entries = registry.entries.lock().unwrap();
        let event = dummy_event(&["editor.tab_size"]);
        for entry in entries.iter() {
            (entry.callback)(&event);
        }
        drop(entries);

        assert!(!invoked.load(std::sync::atomic::Ordering::SeqCst));
    }

    // Validates: Requirement 3.4 — remove_callback with non-existent handle is a no-op
    #[test]
    fn remove_callback_with_unknown_handle_is_noop() {
        let registry = CallbackRegistry::new();

        let _handle = registry.on_reload(&["editor.tab_size"], Box::new(|_| {}));
        assert_eq!(registry.len(), 1);

        // Create a handle that was never registered
        let bogus_handle = CallbackHandle::new(9999);
        registry.remove_callback(bogus_handle);

        // Count unchanged
        assert_eq!(registry.len(), 1);
    }

    // Validates: Requirement 3.4 — removing all callbacks leaves registry empty
    #[test]
    fn remove_all_callbacks_leaves_registry_empty() {
        let registry = CallbackRegistry::new();

        let h1 = registry.on_reload(&["a"], Box::new(|_| {}));
        let h2 = registry.on_reload(&["b"], Box::new(|_| {}));
        let h3 = registry.on_reload(&["c"], Box::new(|_| {}));
        assert_eq!(registry.len(), 3);

        registry.remove_callback(h1);
        registry.remove_callback(h2);
        registry.remove_callback(h3);
        assert!(registry.is_empty());
    }

    // Validates: Requirement 3.4 — callback can be invoked with a ReloadEvent
    #[test]
    fn registered_callback_is_callable() {
        let registry = CallbackRegistry::new();
        let invoked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let invoked_clone = Arc::clone(&invoked);

        let _handle = registry.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                invoked_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        // Directly invoke the callback through the entries to confirm it works
        let entries = registry.entries.lock().unwrap();
        let event = dummy_event(&["editor.tab_size"]);
        (entries[0].callback)(&event);
        drop(entries);

        assert!(invoked.load(std::sync::atomic::Ordering::SeqCst));
    }

    // ========================================================================
    // 13.5 — Selective callback invocation
    // ========================================================================

    // Validates: Requirement 3.3 — invoke calls callback when watched key overlaps with changed keys
    #[test]
    fn invoke_calls_callback_when_watched_key_overlaps() {
        let registry = CallbackRegistry::new();
        let invoked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let invoked_clone = Arc::clone(&invoked);

        let _handle = registry.on_reload(
            &["editor.tab_size", "editor.indent_style"],
            Box::new(move |_event| {
                invoked_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        let event = dummy_event(&["editor.tab_size", "logging.level"]);
        registry.invoke(&event);

        assert!(invoked.load(std::sync::atomic::Ordering::SeqCst));
    }

    // Validates: Requirement 3.3 — invoke does NOT call callback when no watched key overlaps
    #[test]
    fn invoke_skips_callback_when_no_watched_key_overlaps() {
        let registry = CallbackRegistry::new();
        let invoked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let invoked_clone = Arc::clone(&invoked);

        let _handle = registry.on_reload(
            &["theme.active"],
            Box::new(move |_event| {
                invoked_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        let event = dummy_event(&["editor.tab_size"]);
        registry.invoke(&event);

        assert!(!invoked.load(std::sync::atomic::Ordering::SeqCst));
    }

    // Validates: Requirement 3.3 — invoke selectively invokes only matching callbacks
    #[test]
    fn invoke_selectively_invokes_only_matching_callbacks() {
        let registry = CallbackRegistry::new();

        let editor_invoked = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let theme_invoked = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let logging_invoked = Arc::new(std::sync::atomic::AtomicU32::new(0));

        let editor_clone = Arc::clone(&editor_invoked);
        let theme_clone = Arc::clone(&theme_invoked);
        let logging_clone = Arc::clone(&logging_invoked);

        let _h1 = registry.on_reload(
            &["editor.tab_size", "editor.indent_style"],
            Box::new(move |_event| {
                editor_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        let _h2 = registry.on_reload(
            &["theme.active"],
            Box::new(move |_event| {
                theme_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        let _h3 = registry.on_reload(
            &["logging.level"],
            Box::new(move |_event| {
                logging_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        // Event only changes editor.tab_size and logging.level
        let event = dummy_event(&["editor.tab_size", "logging.level"]);
        registry.invoke(&event);

        // Editor callback should be invoked (editor.tab_size overlaps)
        assert_eq!(editor_invoked.load(std::sync::atomic::Ordering::SeqCst), 1);
        // Theme callback should NOT be invoked (no overlap)
        assert_eq!(theme_invoked.load(std::sync::atomic::Ordering::SeqCst), 0);
        // Logging callback should be invoked (logging.level overlaps)
        assert_eq!(logging_invoked.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // Validates: Requirement 3.3 — invoke with empty changed_keys invokes no callbacks
    #[test]
    fn invoke_with_empty_changed_keys_invokes_no_callbacks() {
        let registry = CallbackRegistry::new();
        let invoked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let invoked_clone = Arc::clone(&invoked);

        let _handle = registry.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                invoked_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        let event = dummy_event(&[]);
        registry.invoke(&event);

        assert!(!invoked.load(std::sync::atomic::Ordering::SeqCst));
    }

    // Validates: Requirement 3.3 — invoke passes the correct event to the callback
    #[test]
    fn invoke_passes_event_to_callback() {
        let registry = CallbackRegistry::new();
        let received_keys = Arc::new(Mutex::new(Vec::<String>::new()));
        let keys_clone = Arc::clone(&received_keys);

        let _handle = registry.on_reload(
            &["editor.tab_size"],
            Box::new(move |event| {
                let mut keys = keys_clone.lock().unwrap();
                keys.extend(event.changed_keys.clone());
            }),
        );

        let event = dummy_event(&["editor.tab_size", "logging.level"]);
        registry.invoke(&event);

        let keys = received_keys.lock().unwrap();
        assert_eq!(
            *keys,
            vec!["editor.tab_size".to_string(), "logging.level".to_string()]
        );
    }

    // Validates: Requirement 3.3 — callback watching multiple keys fires if any one matches
    #[test]
    fn invoke_fires_if_any_single_watched_key_matches() {
        let registry = CallbackRegistry::new();
        let count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let count_clone = Arc::clone(&count);

        let _handle = registry.on_reload(
            &[
                "editor.tab_size",
                "editor.indent_style",
                "editor.line_endings",
            ],
            Box::new(move |_event| {
                count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        // Only editor.indent_style changed — should still invoke
        let event = dummy_event(&["editor.indent_style"]);
        registry.invoke(&event);

        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // Validates: Requirement 3.3 — callback is invoked only once even if multiple watched keys match
    #[test]
    fn invoke_fires_callback_only_once_even_with_multiple_overlapping_keys() {
        let registry = CallbackRegistry::new();
        let count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let count_clone = Arc::clone(&count);

        let _handle = registry.on_reload(
            &["editor.tab_size", "editor.indent_style"],
            Box::new(move |_event| {
                count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        // Both watched keys changed — callback should still only fire once
        let event = dummy_event(&["editor.tab_size", "editor.indent_style"]);
        registry.invoke(&event);

        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // ========================================================================
    // 13.6 — Callback invocation ordering: no lock held during invocation
    // ========================================================================

    // Validates: Requirement 3.3 — callbacks run after state update, no lock held during invocation
    #[test]
    fn callback_can_reenter_registry_without_deadlock() {
        let registry = Arc::new(CallbackRegistry::new());
        let registry_clone = Arc::clone(&registry);
        let reentry_succeeded = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let reentry_clone = Arc::clone(&reentry_succeeded);

        // Register a callback that re-enters the registry by calling len()
        // and registering another callback. If the lock were held during
        // invocation this would deadlock.
        let _handle = registry.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                // Re-enter the registry: query length
                let count = registry_clone.len();
                assert!(count >= 1);

                // Re-enter the registry: register another callback
                let _new_handle = registry_clone.on_reload(&["logging.level"], Box::new(|_| {}));

                reentry_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        let event = dummy_event(&["editor.tab_size"]);
        registry.invoke(&event);

        // Verify the callback actually ran and re-entry succeeded
        assert!(reentry_succeeded.load(std::sync::atomic::Ordering::SeqCst));
        // The new callback registered during invocation should be present
        assert_eq!(registry.len(), 2);
    }

    // Validates: Requirement 3.4 — CallbackRegistry is Send + Sync (thread-safe sharing)
    #[test]
    fn callback_registry_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CallbackRegistry>();
    }

    // Validates: Requirement 3.4 — concurrent invocation from multiple threads is safe
    #[test]
    fn concurrent_invocation_is_safe() {
        let registry = Arc::new(CallbackRegistry::new());
        let count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let count_clone = Arc::clone(&count);

        let _handle = registry.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        let mut handles = Vec::new();
        for _ in 0..10 {
            let reg = Arc::clone(&registry);
            let handle = std::thread::spawn(move || {
                let event = dummy_event(&["editor.tab_size"]);
                reg.invoke(&event);
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("thread panicked");
        }

        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 10);
    }

    // Validates: Requirement 3.3 — callback can remove itself from the registry during invocation
    #[test]
    fn callback_can_remove_callback_during_invocation_without_deadlock() {
        let registry = Arc::new(CallbackRegistry::new());
        let registry_clone = Arc::clone(&registry);
        let removal_succeeded = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let removal_clone = Arc::clone(&removal_succeeded);

        // Register a "target" callback we will remove from within another callback
        let target_handle = registry.on_reload(&["logging.level"], Box::new(|_| {}));

        // Register a callback that removes the target callback during invocation
        let _handle = registry.on_reload(
            &["editor.tab_size"],
            Box::new(move |_event| {
                registry_clone.remove_callback(target_handle.clone());
                removal_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            }),
        );

        assert_eq!(registry.len(), 2);

        let event = dummy_event(&["editor.tab_size"]);
        registry.invoke(&event);

        assert!(removal_succeeded.load(std::sync::atomic::Ordering::SeqCst));
        // Target callback was removed during invocation
        assert_eq!(registry.len(), 1);
    }
}
