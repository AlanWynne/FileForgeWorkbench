//! Duplicate detection — ResourceUri deduplication across Tab_Groups.
//!
//! Prevents opening the same resource in multiple tabs (unless split view
//! is explicitly requested). Normalises URIs before comparison.
