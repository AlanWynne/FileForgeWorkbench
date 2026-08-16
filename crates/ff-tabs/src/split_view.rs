//! Split editor views — same DocumentHandle in multiple Tab_Groups.
//!
//! When a document is split, both tabs share the same `Arc<RwLock<Document>>`
//! but maintain independent viewport and cursor state.
