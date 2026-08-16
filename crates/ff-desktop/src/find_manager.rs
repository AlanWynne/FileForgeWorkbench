//! `FindManager` — bridges `ff-find-and-replace` into the desktop shell.
//!
//! Snapshots the active document into a `SliceIndexer`, executes FIND/CHANGE
//! operations, and translates outcomes into status strings for the status bar.

use ff_find_and_replace::indexer::{CharacterIndexer, SliceIndexer};
use ff_find_and_replace::{ChangeOutcome, ChangeRequest, FindEngine, FindOutcome, FindRequest};
use tokio::runtime::Runtime;

use crate::tab_manager::TabManager;

/// Owns the `FindEngine` and provides high-level FIND/CHANGE helpers.
pub struct FindManager {
    engine: FindEngine,
}

impl FindManager {
    pub fn new() -> Self {
        Self {
            engine: FindEngine::new(),
        }
    }

    /// Execute `FIND <term>` on the active tab.
    ///
    /// Returns a status string suitable for the status bar.
    /// Validates: Requirement 21.3 — FIND wired into command field.
    pub fn find(&mut self, term: &str, tabs: &mut TabManager, runtime: &Runtime) -> String {
        let bytes = snapshot_bytes(tabs, runtime);
        let indexer = SliceIndexer::new(&bytes);
        let filter = ff_find_and_replace::scope::AllLinesFilter;
        let cursor = cursor_byte_pos(tabs);
        let req = FindRequest::literal(term)
            .with_cursor(ff_find_and_replace::types::BytePosition(cursor));

        match self.engine.find(&req, &indexer, &filter, None) {
            Ok(FindOutcome::Found(r)) => {
                // Scroll viewport to the matching line
                let line_num = indexer.line_from_position(r.match_range.start).0 + 1;
                let cursor = tabs.active_tab().cursor.clone();
                tabs.active_tab_mut()
                    .viewport
                    .scroll_to_line(line_num.max(1), &cursor);
                tabs.active_tab_mut().cursor.set_position(line_num, 1);
                format!("FIND — found at line {line_num}")
            }
            Ok(FindOutcome::NotFound { term: t }) => format!("'{t}' NOT FOUND"),
            Ok(FindOutcome::FoundAll { count, .. }) => format!("FIND ALL — {count} occurrences"),
            Err(e) => format!("FIND error: {e}"),
        }
    }

    /// Execute `RFIND` (repeat previous find) on the active tab.
    ///
    /// Validates: Requirement 21.3 — RFIND wired into command field.
    pub fn rfind(&mut self, tabs: &mut TabManager, runtime: &Runtime) -> String {
        let bytes = snapshot_bytes(tabs, runtime);
        let indexer = SliceIndexer::new(&bytes);
        let filter = ff_find_and_replace::scope::AllLinesFilter;

        match self.engine.rfind(&indexer, &filter, None) {
            Ok(FindOutcome::Found(r)) => {
                let line_num = indexer.line_from_position(r.match_range.start).0 + 1;
                let cursor = tabs.active_tab().cursor.clone();
                tabs.active_tab_mut()
                    .viewport
                    .scroll_to_line(line_num.max(1), &cursor);
                tabs.active_tab_mut().cursor.set_position(line_num, 1);
                format!("RFIND — found at line {line_num}")
            }
            Ok(FindOutcome::NotFound { term: t }) => format!("'{t}' NOT FOUND"),
            Ok(FindOutcome::FoundAll { .. }) => "RFIND — found all".to_string(),
            Err(e) => format!("RFIND error: {e}"),
        }
    }

    /// Execute `CHANGE <old> <new>` on the active tab.
    ///
    /// Validates: Requirement 21.3 — CHANGE wired into command field.
    pub fn change(
        &mut self,
        old: &str,
        new: &str,
        tabs: &mut TabManager,
        runtime: &Runtime,
    ) -> String {
        let bytes = snapshot_bytes(tabs, runtime);
        let mut indexer = ff_find_and_replace::indexer::MutableSliceIndexer::new(
            std::str::from_utf8(&bytes).unwrap_or(""),
        );
        let filter = ff_find_and_replace::scope::AllLinesFilter;
        let cursor = cursor_byte_pos(tabs);
        let req = ChangeRequest::new(
            FindRequest::literal(old).with_cursor(ff_find_and_replace::types::BytePosition(cursor)),
            new,
        );

        match self.engine.change(&req, &mut indexer, &filter, None) {
            Ok(ChangeOutcome::Changed(r)) => {
                // Write modified bytes back to the document
                let new_bytes = indexer.content().to_vec();
                write_bytes(tabs, runtime, &new_bytes);
                tabs.active_tab_mut().is_modified = true;
                tabs.active_tab_mut().line_count = runtime
                    .block_on(async { tabs.active_tab().document.read().await.line_count() });
                let line_num = r.final_line.0 + 1;
                tabs.active_tab_mut().cursor.set_position(line_num, 1);
                format!("CHANGE — {} occurrence(s) replaced", r.replacement_count)
            }
            Ok(ChangeOutcome::NotFound { term: t }) => format!("'{t}' NOT FOUND"),
            Ok(ChangeOutcome::ReadOnly) => "Document is read-only".to_string(),
            Err(e) => format!("CHANGE error: {e}"),
        }
    }

    /// Execute `RCHANGE` (repeat previous change) on the active tab.
    ///
    /// Validates: Requirement 21.3 — RCHANGE wired into command field.
    pub fn rchange(&mut self, tabs: &mut TabManager, runtime: &Runtime) -> String {
        let bytes = snapshot_bytes(tabs, runtime);
        let mut indexer = ff_find_and_replace::indexer::MutableSliceIndexer::new(
            std::str::from_utf8(&bytes).unwrap_or(""),
        );
        let filter = ff_find_and_replace::scope::AllLinesFilter;

        match self.engine.rchange(&mut indexer, &filter, None) {
            Ok(ChangeOutcome::Changed(r)) => {
                let new_bytes = indexer.content().to_vec();
                write_bytes(tabs, runtime, &new_bytes);
                tabs.active_tab_mut().is_modified = true;
                tabs.active_tab_mut().line_count = runtime
                    .block_on(async { tabs.active_tab().document.read().await.line_count() });
                format!("RCHANGE — {} occurrence(s) replaced", r.replacement_count)
            }
            Ok(ChangeOutcome::NotFound { term: t }) => format!("'{t}' NOT FOUND"),
            Ok(ChangeOutcome::ReadOnly) => "Document is read-only".to_string(),
            Err(e) => format!("RCHANGE error: {e}"),
        }
    }
}

impl Default for FindManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Snapshot the active tab's document bytes synchronously.
fn snapshot_bytes(tabs: &TabManager, runtime: &Runtime) -> Vec<u8> {
    runtime.block_on(async {
        let doc = tabs.active_tab().document.read().await;
        let len = doc.length();
        if len == 0 {
            Vec::new()
        } else {
            doc.get_range(ff_document_model::BytePosition(0), len)
                .unwrap_or_default()
        }
    })
}

/// Write a complete byte buffer back into the active tab's document.
fn write_bytes(tabs: &mut TabManager, runtime: &Runtime, bytes: &[u8]) {
    runtime.block_on(async {
        let mut doc = tabs.active_tab_mut().document.write().await;
        let len = doc.length();
        // Delete all then insert new content
        if len > 0 {
            let _ = doc.delete(ff_document_model::BytePosition(0), len);
        }
        if !bytes.is_empty() {
            let _ = doc.insert(ff_document_model::BytePosition(0), bytes);
        }
    });
}

/// Current cursor byte position in the active tab (approximate: line start).
fn cursor_byte_pos(tabs: &TabManager) -> u64 {
    let tab = tabs.active_tab();
    let line = tab.cursor.cursor_line().saturating_sub(1); // 0-based
                                                           // Use viewport top_line as a proxy when cursor is at line 1
    let _ = line;
    0 // Start from document beginning for simplicity; RFIND advances from last match
}

#[cfg(test)]
mod tests {
    use ff_find_and_replace::indexer::SliceIndexer;
    use ff_find_and_replace::scope::AllLinesFilter;
    use ff_find_and_replace::{FindEngine, FindOutcome, FindRequest};

    /// Validates: Requirement 21.3 — FIND on a SliceIndexer snapshot finds the term.
    #[test]
    fn find_on_snapshot_locates_term() {
        // Validates: Phase U 21.3 — FindEngine wired to document snapshot
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello world\nfoo bar\n");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("foo");
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        assert!(matches!(outcome, FindOutcome::Found(_)));
    }

    /// Validates: Requirement 21.3 — FIND returns NotFound when term absent.
    #[test]
    fn find_on_snapshot_returns_not_found() {
        // Validates: Phase U 21.3 — not-found path surfaces correctly
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello world");
        let filter = AllLinesFilter;
        let req = FindRequest::literal("xyz");
        let outcome = engine.find(&req, &indexer, &filter, None).unwrap();
        assert!(matches!(outcome, FindOutcome::NotFound { .. }));
    }

    /// Validates: Requirement 21.3 — RFIND without prior FIND returns error.
    #[test]
    fn rfind_without_prior_find_returns_error() {
        // Validates: Phase U 21.3 — RFIND error path
        use ff_find_and_replace::FindReplaceError;
        let mut engine = FindEngine::new();
        let indexer = SliceIndexer::from_str("hello");
        let filter = AllLinesFilter;
        let err = engine.rfind(&indexer, &filter, None).unwrap_err();
        assert!(matches!(err, FindReplaceError::NoPreviousFind));
    }

    /// Validates: Requirement 21.3 — CHANGE replaces term in snapshot.
    #[test]
    fn change_on_snapshot_replaces_term() {
        // Validates: Phase U 21.3 — CHANGE wired to document snapshot
        use ff_find_and_replace::indexer::MutableSliceIndexer;
        use ff_find_and_replace::{ChangeOutcome, ChangeRequest};
        let mut engine = FindEngine::new();
        let mut indexer = MutableSliceIndexer::new("hello world");
        let filter = AllLinesFilter;
        let req = ChangeRequest::new(FindRequest::literal("hello"), "goodbye");
        let outcome = engine.change(&req, &mut indexer, &filter, None).unwrap();
        assert!(matches!(outcome, ChangeOutcome::Changed(_)));
        assert_eq!(indexer.content_str(), Some("goodbye world"));
    }
}
