//! DocumentHandle type alias and constructor helpers.
//!
//! Provides shared ownership of a Document via `Arc<RwLock<Document>>`.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::document::Document;

/// Shared ownership handle for a Document. Enables multi-view and
/// multi-thread access with interior mutability via RwLock.
pub type DocumentHandle = Arc<RwLock<Document>>;

/// Create a new DocumentHandle wrapping an empty document.
pub fn new_document() -> DocumentHandle {
    Arc::new(RwLock::new(Document::new()))
}

/// Create a new DocumentHandle wrapping a document with pre-allocated capacity.
pub fn new_document_with_capacity(capacity: u64) -> DocumentHandle {
    Arc::new(RwLock::new(Document::with_capacity(capacity)))
}

/// Wrap an existing document in a handle.
pub fn wrap_document(doc: Document) -> DocumentHandle {
    Arc::new(RwLock::new(doc))
}

// Compile-time assertion: DocumentHandle is Send + Sync
#[allow(dead_code)]
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DocumentHandle>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_clone_shares_document() {
        let handle = new_document();
        let handle2 = handle.clone();

        {
            let mut doc = handle.write().await;
            doc.insert(crate::types::BytePosition(0), b"hello").unwrap();
        }

        {
            let doc = handle2.read().await;
            assert_eq!(doc.length(), 5);
        }
    }

    #[tokio::test]
    async fn handle_concurrent_reads() {
        let handle = new_document();
        {
            let mut doc = handle.write().await;
            doc.insert(crate::types::BytePosition(0), b"test data")
                .unwrap();
        }

        let h1 = handle.clone();
        let h2 = handle.clone();

        let (r1, r2) = tokio::join!(async move { h1.read().await.length() }, async move {
            h2.read().await.length()
        },);

        assert_eq!(r1, 9);
        assert_eq!(r2, 9);
    }

    #[test]
    fn document_handle_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DocumentHandle>();
    }
}
