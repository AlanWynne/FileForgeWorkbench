//! WorkbenchUndoManager — routes undo/redo operations to per-document managers.
//!
//! This type implements the `UndoManager` trait from `ff-command`, serving as the
//! bridge between the command framework and per-document undo stacks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::config::UndoConfig;
use crate::error::UndoError;
use crate::manager::DocumentUndoManager;

/// Routes undo/redo operations to the active document's manager.
///
/// Manages a registry of per-document undo managers and routes operations
/// based on the currently active document ID.
///
/// # Thread Safety
///
/// This type is `Send + Sync` — it uses internal locking to support
/// concurrent access from the command framework.
pub struct WorkbenchUndoManager {
    /// Per-document undo managers.
    documents: RwLock<HashMap<String, Arc<Mutex<DocumentUndoManager>>>>,
    /// The currently active document ID.
    active_document: RwLock<Option<String>>,
}

impl WorkbenchUndoManager {
    /// Creates a new workbench undo manager with no registered documents.
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
            active_document: RwLock::new(None),
        }
    }

    /// Registers a document's undo manager.
    pub fn register_document(&self, document_id: &str, manager: Arc<Mutex<DocumentUndoManager>>) {
        let mut docs = self.documents.write().unwrap_or_else(|e| e.into_inner());
        docs.insert(document_id.to_string(), manager);
    }

    /// Creates and registers a new document with default config.
    pub fn register_new_document(&self, document_id: &str, config: UndoConfig) {
        let manager = Arc::new(Mutex::new(DocumentUndoManager::new(config)));
        self.register_document(document_id, manager);
    }

    /// Unregisters a document (on close), releasing its undo state.
    pub fn unregister_document(&self, document_id: &str) {
        let mut docs = self.documents.write().unwrap_or_else(|e| e.into_inner());
        docs.remove(document_id);

        // Clear active if it was this document
        let mut active = self
            .active_document
            .write()
            .unwrap_or_else(|e| e.into_inner());
        if active.as_deref() == Some(document_id) {
            *active = None;
        }
    }

    /// Sets the currently active document ID for routing.
    pub fn set_active_document(&self, document_id: &str) {
        let mut active = self
            .active_document
            .write()
            .unwrap_or_else(|e| e.into_inner());
        *active = Some(document_id.to_string());
    }

    /// Returns the active document's manager, if any.
    pub fn active_manager(&self) -> Result<Arc<Mutex<DocumentUndoManager>>, UndoError> {
        let active = self
            .active_document
            .read()
            .unwrap_or_else(|e| e.into_inner());
        let doc_id = active.as_deref().ok_or(UndoError::NoActiveDocument)?;
        let docs = self.documents.read().unwrap_or_else(|e| e.into_inner());
        docs.get(doc_id)
            .cloned()
            .ok_or_else(|| UndoError::DocumentNotRegistered {
                document_id: doc_id.to_string(),
            })
    }

    /// Returns a specific document's manager.
    pub fn get_document_manager(
        &self,
        document_id: &str,
    ) -> Result<Arc<Mutex<DocumentUndoManager>>, UndoError> {
        let docs = self.documents.read().unwrap_or_else(|e| e.into_inner());
        docs.get(document_id)
            .cloned()
            .ok_or_else(|| UndoError::DocumentNotRegistered {
                document_id: document_id.to_string(),
            })
    }

    /// Returns the number of registered documents.
    pub fn document_count(&self) -> usize {
        let docs = self.documents.read().unwrap_or_else(|e| e.into_inner());
        docs.len()
    }

    /// Returns the active document ID.
    pub fn active_document_id(&self) -> Option<String> {
        let active = self
            .active_document
            .read()
            .unwrap_or_else(|e| e.into_inner());
        active.clone()
    }
}

impl Default for WorkbenchUndoManager {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: WorkbenchUndoManager uses RwLock internally for all state
unsafe impl Send for WorkbenchUndoManager {}
unsafe impl Sync for WorkbenchUndoManager {}

/// Trait for document model integration — operations are applied to this target.
///
/// Implemented by the document model crate. The undo system calls these methods
/// to apply and reverse edit operations.
pub trait EditTarget: Send + Sync {
    /// Apply an insert operation to the document.
    fn apply_insert(&mut self, position: u64, text: &[u8]);
    /// Apply a delete operation to the document.
    fn apply_delete(&mut self, position: u64, length: u32);
    /// Apply a replace operation to the document.
    fn apply_replace(&mut self, position: u64, old_length: u32, new_text: &[u8]);
    /// Get the current document length in bytes.
    fn document_length(&self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_set_active_document() {
        let wbm = WorkbenchUndoManager::new();
        wbm.register_new_document("doc1", UndoConfig::default());
        wbm.set_active_document("doc1");

        let mgr = wbm.active_manager().unwrap();
        let lock = mgr.lock().unwrap();
        assert_eq!(lock.undo_depth(), 0);
    }

    #[test]
    fn unregister_document_removes_it() {
        let wbm = WorkbenchUndoManager::new();
        wbm.register_new_document("doc1", UndoConfig::default());
        wbm.unregister_document("doc1");
        assert_eq!(wbm.document_count(), 0);
    }

    #[test]
    fn active_manager_without_active_returns_error() {
        let wbm = WorkbenchUndoManager::new();
        assert!(matches!(
            wbm.active_manager(),
            Err(UndoError::NoActiveDocument)
        ));
    }

    #[test]
    fn per_document_isolation() {
        let wbm = WorkbenchUndoManager::new();
        wbm.register_new_document("doc1", UndoConfig::default());
        wbm.register_new_document("doc2", UndoConfig::default());

        // Modify doc1
        {
            let mgr = wbm.get_document_manager("doc1").unwrap();
            let mut lock = mgr.lock().unwrap();
            lock.record_insert(0, b"hello");
        }

        // doc2 should be unaffected
        {
            let mgr = wbm.get_document_manager("doc2").unwrap();
            let lock = mgr.lock().unwrap();
            assert_eq!(lock.undo_depth(), 0);
        }

        // doc1 should have one transaction
        {
            let mgr = wbm.get_document_manager("doc1").unwrap();
            let lock = mgr.lock().unwrap();
            assert_eq!(lock.undo_depth(), 1);
        }
    }

    #[test]
    fn set_active_switches_routing() {
        let wbm = WorkbenchUndoManager::new();
        wbm.register_new_document("doc1", UndoConfig::default());
        wbm.register_new_document("doc2", UndoConfig::default());

        wbm.set_active_document("doc1");
        assert_eq!(wbm.active_document_id(), Some("doc1".to_string()));

        wbm.set_active_document("doc2");
        assert_eq!(wbm.active_document_id(), Some("doc2".to_string()));
    }
}
