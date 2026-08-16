//! Document watcher trait and notification system.
//!
//! Provides the `DocumentWatcher` trait for receiving document change
//! notifications, and `WatcherHandle` for managing registrations.

use crate::types::BytePosition;

/// Trait for receiving document change notifications.
/// Implementations must be non-blocking; expensive work should be deferred.
pub trait DocumentWatcher: Send + Sync {
    /// Called when a modification is attempted on a read-only document.
    fn notify_modify_attempt(&self) {}

    /// Called after text is inserted.
    fn notify_insert(&self, _position: BytePosition, _text: &[u8], _lines_added: u64) {}

    /// Called after text is deleted.
    fn notify_delete(&self, _position: BytePosition, _length: u64, _lines_removed: u64) {}

    /// Called when the document reaches or leaves its save point.
    fn notify_save_point(&self, _at_save_point: bool) {}

    /// Called before the document is deallocated.
    fn notify_deleted(&self) {}

    /// Called when syntax styling needs to be extended to a position.
    fn notify_style_needed(&self, _end_position: BytePosition) {}

    /// Unique identifier for deduplication purposes.
    fn watcher_id(&self) -> u64;
}

/// Handle returned by `add_watcher` for later removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WatcherHandle(pub(crate) u64);

impl WatcherHandle {
    /// Returns the internal ID.
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Registry managing document watchers.
#[derive(Default)]
pub(crate) struct WatcherRegistry {
    watchers: Vec<Box<dyn DocumentWatcher>>,
    next_handle: u64,
}

impl WatcherRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
            next_handle: 1,
        }
    }

    /// Add a watcher, returning a handle. Returns error if duplicate.
    pub fn add(&mut self, watcher: Box<dyn DocumentWatcher>) -> Result<WatcherHandle, ()> {
        let id = watcher.watcher_id();
        // Check for duplicates
        if self.watchers.iter().any(|w| w.watcher_id() == id) {
            return Err(());
        }
        let handle = WatcherHandle(id);
        self.watchers.push(watcher);
        Ok(handle)
    }

    /// Remove a watcher by handle.
    pub fn remove(&mut self, handle: WatcherHandle) -> bool {
        let len_before = self.watchers.len();
        self.watchers.retain(|w| w.watcher_id() != handle.0);
        self.watchers.len() < len_before
    }

    /// Notify all watchers of an insert.
    pub fn notify_insert(&self, position: BytePosition, text: &[u8], lines_added: u64) {
        for watcher in &self.watchers {
            watcher.notify_insert(position, text, lines_added);
        }
    }

    /// Notify all watchers of a delete.
    pub fn notify_delete(&self, position: BytePosition, length: u64, lines_removed: u64) {
        for watcher in &self.watchers {
            watcher.notify_delete(position, length, lines_removed);
        }
    }

    /// Notify all watchers of a modify attempt on read-only document.
    pub fn notify_modify_attempt(&self) {
        for watcher in &self.watchers {
            watcher.notify_modify_attempt();
        }
    }

    /// Notify all watchers of save point change.
    pub fn notify_save_point(&self, at_save_point: bool) {
        for watcher in &self.watchers {
            watcher.notify_save_point(at_save_point);
        }
    }

    /// Notify all watchers of document deletion.
    pub fn notify_deleted(&self) {
        for watcher in &self.watchers {
            watcher.notify_deleted();
        }
    }

    /// Number of registered watchers.
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.watchers.len()
    }
}

impl std::fmt::Debug for WatcherRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WatcherRegistry")
            .field("watcher_count", &self.watchers.len())
            .field("next_handle", &self.next_handle)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    struct TestWatcher {
        id: u64,
        insert_count: Arc<AtomicU64>,
        delete_count: Arc<AtomicU64>,
    }

    impl TestWatcher {
        fn new(id: u64) -> (Self, Arc<AtomicU64>, Arc<AtomicU64>) {
            let insert_count = Arc::new(AtomicU64::new(0));
            let delete_count = Arc::new(AtomicU64::new(0));
            (
                Self {
                    id,
                    insert_count: insert_count.clone(),
                    delete_count: delete_count.clone(),
                },
                insert_count,
                delete_count,
            )
        }
    }

    impl DocumentWatcher for TestWatcher {
        fn notify_insert(&self, _position: BytePosition, _text: &[u8], _lines_added: u64) {
            self.insert_count.fetch_add(1, Ordering::SeqCst);
        }

        fn notify_delete(&self, _position: BytePosition, _length: u64, _lines_removed: u64) {
            self.delete_count.fetch_add(1, Ordering::SeqCst);
        }

        fn watcher_id(&self) -> u64 {
            self.id
        }
    }

    #[test]
    fn add_and_remove_watcher() {
        let mut registry = WatcherRegistry::new();
        let (watcher, _, _) = TestWatcher::new(1);
        let handle = registry.add(Box::new(watcher)).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.remove(handle));
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn duplicate_watcher_rejected() {
        let mut registry = WatcherRegistry::new();
        let (w1, _, _) = TestWatcher::new(1);
        let (w2, _, _) = TestWatcher::new(1);
        registry.add(Box::new(w1)).unwrap();
        assert!(registry.add(Box::new(w2)).is_err());
    }

    #[test]
    fn notify_insert_reaches_all_watchers() {
        let mut registry = WatcherRegistry::new();
        let (w1, ic1, _) = TestWatcher::new(1);
        let (w2, ic2, _) = TestWatcher::new(2);
        registry.add(Box::new(w1)).unwrap();
        registry.add(Box::new(w2)).unwrap();

        registry.notify_insert(BytePosition(0), b"hello", 0);
        assert_eq!(ic1.load(Ordering::SeqCst), 1);
        assert_eq!(ic2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn removed_watcher_not_notified() {
        let mut registry = WatcherRegistry::new();
        let (w1, ic1, _) = TestWatcher::new(1);
        let handle = registry.add(Box::new(w1)).unwrap();
        registry.remove(handle);

        registry.notify_insert(BytePosition(0), b"hello", 0);
        assert_eq!(ic1.load(Ordering::SeqCst), 0);
    }
}
