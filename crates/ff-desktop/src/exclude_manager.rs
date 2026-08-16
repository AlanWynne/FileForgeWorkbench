//! Exclude/Show/Reset command bridge — wires `ff-exclude-show-filter` into the
//! desktop shell.
//!
//! Each open tab gets its own `ExclusionEngine` backed by a `ContractionState`
//! sized to the tab's current line count.  The engine is re-created whenever
//! the active tab changes and the stored line count differs from the engine's
//! line count (e.g. after edits).
//!
//! The `TabDocAdapter` provides synchronous line-content access by holding a
//! pre-snapshotted `Vec<String>` of the document lines, built once per command
//! invocation via the Tokio runtime.

use std::collections::HashMap;

use ff_display_line_mapping::ContractionState;
use ff_exclude_show_filter::{
    DocumentAccess, ExcludeArgs, ExcludeScope, ExclusionBlock, ExclusionEngine, ResetVariant,
    ShowArgs,
};
use tokio::runtime::Runtime;

use crate::tab_manager::TabManager;
use crate::tab_state::TabId;

// ── TabDocAdapter ────────────────────────────────────────────────────────────

/// Synchronous document-content adapter backed by a pre-built line snapshot.
struct TabDocAdapter {
    lines: Vec<String>,
}

impl TabDocAdapter {
    fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl DocumentAccess for TabDocAdapter {
    fn line_content(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn is_tagged(&self, _line: usize) -> bool {
        false
    }
}

// ── ExcludeManager ───────────────────────────────────────────────────────────

/// Per-tab exclusion engine storage.
///
/// Keyed by `TabId`.  Each entry is an `ExclusionEngine` whose `ContractionState`
/// is sized to the tab's line count at the time the engine was created.
pub struct ExcludeManager {
    engines: HashMap<TabId, ExclusionEngine<ContractionState, TabDocAdapter>>,
}

impl ExcludeManager {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    /// Return the exclusion blocks for the active tab (for viewport rendering).
    #[allow(dead_code)]
    pub fn exclusion_blocks(&self, tab_id: TabId) -> Vec<ExclusionBlock> {
        self.engines
            .get(&tab_id)
            .map(|e| e.exclusion_blocks())
            .unwrap_or_default()
    }

    /// True if the given document line (1-based) is excluded in the active tab.
    #[allow(dead_code)]
    pub fn is_excluded(&self, tab_id: TabId, doc_line_1based: u64) -> bool {
        self.engines
            .get(&tab_id)
            .map(|e| e.is_excluded(doc_line_1based.saturating_sub(1) as usize))
            .unwrap_or(false)
    }

    // ── EXCLUDE ──────────────────────────────────────────────────────────

    /// Execute `EXCLUDE ALL`.
    pub fn exclude_all(&mut self, tabs: &mut TabManager, runtime: &Runtime) -> String {
        let engine = self.engine_for(tabs, runtime);
        match engine.execute_exclude(&ExcludeArgs::All) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    /// Execute `EXCLUDE 'text'` (visible lines, case-insensitive).
    pub fn exclude_text(&mut self, text: &str, tabs: &mut TabManager, runtime: &Runtime) -> String {
        let engine = self.engine_for(tabs, runtime);
        let args = ExcludeArgs::Text {
            pattern: text.to_string(),
            scope: ExcludeScope::Visible,
        };
        match engine.execute_exclude(&args) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    /// Execute `EXCLUDE 'text' ALL` (all lines regardless of visibility).
    pub fn exclude_text_all(
        &mut self,
        text: &str,
        tabs: &mut TabManager,
        runtime: &Runtime,
    ) -> String {
        let engine = self.engine_for(tabs, runtime);
        let args = ExcludeArgs::Text {
            pattern: text.to_string(),
            scope: ExcludeScope::All,
        };
        match engine.execute_exclude(&args) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    // ── SHOW ─────────────────────────────────────────────────────────────

    /// Execute `SHOW ALL`.
    pub fn show_all(&mut self, tabs: &mut TabManager, runtime: &Runtime) -> String {
        let engine = self.engine_for(tabs, runtime);
        match engine.execute_show(&ShowArgs::All) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    /// Execute `SHOW 'text'` (reveal excluded lines containing text).
    pub fn show_text(&mut self, text: &str, tabs: &mut TabManager, runtime: &Runtime) -> String {
        let engine = self.engine_for(tabs, runtime);
        match engine.execute_show(&ShowArgs::Text {
            pattern: text.to_string(),
        }) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    // ── RESET ────────────────────────────────────────────────────────────

    /// Execute `RESET` / `RESET EXCLUDED` / `RESET ALL`.
    pub fn reset(
        &mut self,
        variant: ResetVariant,
        tabs: &mut TabManager,
        runtime: &Runtime,
    ) -> String {
        let engine = self.engine_for(tabs, runtime);
        engine.execute_reset(variant).message
    }

    // ── Internal ─────────────────────────────────────────────────────────

    /// Get or create the engine for the active tab, rebuilding if line count changed.
    fn engine_for<'a>(
        &'a mut self,
        tabs: &mut TabManager,
        runtime: &Runtime,
    ) -> &'a mut ExclusionEngine<ContractionState, TabDocAdapter> {
        let tab = tabs.active_tab_mut();
        let tab_id = tab.id;
        let line_count = tab.line_count as usize;

        // Rebuild if missing or stale (line count changed after edits)
        let needs_rebuild = self
            .engines
            .get(&tab_id)
            .map(|e| e.line_count() != line_count)
            .unwrap_or(true);

        if needs_rebuild {
            let lines = snapshot_lines(tab, runtime);
            let mapping = ContractionState::new(line_count);
            let doc = TabDocAdapter::new(lines);
            self.engines
                .insert(tab_id, ExclusionEngine::new(mapping, doc));
        }

        self.engines.get_mut(&tab_id).expect("just inserted")
    }
}

/// Snapshot all document lines as `Vec<String>` for the adapter.
fn snapshot_lines(tab: &crate::tab_state::TabState, runtime: &Runtime) -> Vec<String> {
    use ff_document_model::LineNumber;
    let count = tab.line_count as usize;
    runtime.block_on(async {
        let doc = tab.document.read().await;
        (1..=count as u64)
            .map(|ln| {
                let start = doc.line_start(LineNumber(ln - 1));
                let end = doc.line_end(LineNumber(ln - 1));
                let len = end.0.saturating_sub(start.0);
                if len == 0 {
                    String::new()
                } else {
                    doc.get_range(start, len)
                        .map(|b| String::from_utf8_lossy(&b).into_owned())
                        .unwrap_or_default()
                }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab_manager::TabManager;
    use tokio::runtime::Runtime;

    fn make_tabs(content: &str) -> (TabManager, Runtime) {
        let rt = Runtime::new().expect("runtime");
        let tabs = TabManager::new(&rt, content);
        (tabs, rt)
    }

    /// Validates: Requirement 21.5 — EXCLUDE ALL hides all lines.
    #[test]
    fn exclude_all_reports_lines_excluded() {
        let (mut tabs, rt) = make_tabs("alpha\nbeta\ngamma\n");
        let mut mgr = ExcludeManager::new();
        let msg = mgr.exclude_all(&mut tabs, &rt);
        assert!(
            msg.contains("excluded") || msg.contains("line"),
            "expected exclusion message, got: {msg}"
        );
        let tab_id = tabs.active_tab().id;
        assert!(mgr.engines[&tab_id].has_excluded_lines());
    }

    /// Validates: Requirement 21.5 — EXCLUDE 'text' hides matching lines only.
    #[test]
    fn exclude_text_hides_matching_lines() {
        let (mut tabs, rt) = make_tabs("hello world\nfoo bar\nhello again\n");
        let mut mgr = ExcludeManager::new();
        let msg = mgr.exclude_text("hello", &mut tabs, &rt);
        assert!(msg.contains("2") || msg.contains("line"), "got: {msg}");
        let tab_id = tabs.active_tab().id;
        let engine = &mgr.engines[&tab_id];
        assert!(engine.is_excluded(0), "line 0 should be excluded");
        assert!(!engine.is_excluded(1), "line 1 should be visible");
        assert!(engine.is_excluded(2), "line 2 should be excluded");
    }

    /// Validates: Requirement 21.5 — SHOW ALL restores all lines.
    #[test]
    fn show_all_restores_all_lines() {
        let (mut tabs, rt) = make_tabs("alpha\nbeta\ngamma\n");
        let mut mgr = ExcludeManager::new();
        mgr.exclude_all(&mut tabs, &rt);
        let msg = mgr.show_all(&mut tabs, &rt);
        assert!(msg.contains("shown") || msg.contains("line"), "got: {msg}");
        let tab_id = tabs.active_tab().id;
        assert!(!mgr.engines[&tab_id].has_excluded_lines());
    }

    /// Validates: Requirement 21.5 — RESET EXCLUDED clears all exclusion state.
    #[test]
    fn reset_excluded_clears_exclusion_state() {
        let (mut tabs, rt) = make_tabs("alpha\nbeta\ngamma\n");
        let mut mgr = ExcludeManager::new();
        mgr.exclude_all(&mut tabs, &rt);
        let msg = mgr.reset(ResetVariant::Excluded, &mut tabs, &rt);
        assert!(
            msg.contains("RESET") || msg.contains("restored"),
            "got: {msg}"
        );
        let tab_id = tabs.active_tab().id;
        assert!(!mgr.engines[&tab_id].has_excluded_lines());
    }

    /// Validates: Requirement 21.5 — exclusion_blocks returns correct block count.
    #[test]
    fn exclusion_blocks_returns_correct_count() {
        let (mut tabs, rt) = make_tabs("a\nb\nc\nd\ne\n");
        let mut mgr = ExcludeManager::new();
        // Exclude lines 0 and 2 (0-based) → two separate blocks
        {
            let engine = mgr.engine_for(&mut tabs, &rt);
            engine.exclude_line(0);
            engine.exclude_line(2);
        }
        let tab_id = tabs.active_tab().id;
        let blocks = mgr.exclusion_blocks(tab_id);
        assert_eq!(blocks.len(), 2);
    }
}
