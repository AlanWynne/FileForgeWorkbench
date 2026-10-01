//! Per-tab EXCLUDE/SHOW/RESET engine store.
//!
//! Each open tab gets its own `ExclusionEngine` backed by a `ContractionState`
//! sized to the tab's current line count. The engine is re-created whenever the
//! stored line count differs from the engine's line count (e.g. after edits).
//!
//! This crate is PURE: it has no dependency on the shell runtime, Tokio, or the
//! document model. The caller identifies a tab by a plain `u64` id, supplies the
//! current `line_count`, and supplies the line content LAZILY via a closure that
//! is invoked ONLY when the engine must be (re)built. The shell owns the glue
//! that snapshots lines from its `TabState` through its async runtime and feeds
//! it in; see the `ff-desktop` `exclude_manager` adapter.

use std::collections::HashMap;

use ff_display_line_mapping::ContractionState;
use ff_exclude_show_filter::{
    DocumentAccess, ExcludeArgs, ExcludeScope, ExclusionBlock, ExclusionEngine, ResetVariant,
    ShowArgs,
};

// === TabDocAdapter ==================================================

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

// === ExcludeManager =================================================

/// Per-tab exclusion engine storage.
///
/// Keyed by a plain `u64` tab id. Each entry is an `ExclusionEngine` whose
/// `ContractionState` is sized to the tab's line count at the time the engine
/// was created.
pub struct ExcludeManager {
    engines: HashMap<u64, ExclusionEngine<ContractionState, TabDocAdapter>>,
}

impl Default for ExcludeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExcludeManager {
    /// Create an empty manager with no per-tab engines.
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    /// Return the exclusion blocks for the given tab (for viewport rendering).
    pub fn exclusion_blocks(&self, tab_id: u64) -> Vec<ExclusionBlock> {
        self.engines
            .get(&tab_id)
            .map(|e| e.exclusion_blocks())
            .unwrap_or_default()
    }

    /// True if the given document line (1-based) is excluded in the given tab.
    pub fn is_excluded(&self, tab_id: u64, doc_line_1based: u64) -> bool {
        self.engines
            .get(&tab_id)
            .map(|e| e.is_excluded(doc_line_1based.saturating_sub(1) as usize))
            .unwrap_or(false)
    }

    // === EXCLUDE ====================================================

    /// Execute `EXCLUDE ALL`.
    pub fn exclude_all(
        &mut self,
        tab_id: u64,
        line_count: usize,
        lines: impl FnOnce() -> Vec<String>,
    ) -> String {
        let engine = self.engine_for(tab_id, line_count, lines);
        match engine.execute_exclude(&ExcludeArgs::All) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    /// Execute `EXCLUDE 'text'` (visible lines, case-insensitive).
    pub fn exclude_text(
        &mut self,
        text: &str,
        tab_id: u64,
        line_count: usize,
        lines: impl FnOnce() -> Vec<String>,
    ) -> String {
        let engine = self.engine_for(tab_id, line_count, lines);
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
        tab_id: u64,
        line_count: usize,
        lines: impl FnOnce() -> Vec<String>,
    ) -> String {
        let engine = self.engine_for(tab_id, line_count, lines);
        let args = ExcludeArgs::Text {
            pattern: text.to_string(),
            scope: ExcludeScope::All,
        };
        match engine.execute_exclude(&args) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    // === SHOW =======================================================

    /// Execute `SHOW ALL`.
    pub fn show_all(
        &mut self,
        tab_id: u64,
        line_count: usize,
        lines: impl FnOnce() -> Vec<String>,
    ) -> String {
        let engine = self.engine_for(tab_id, line_count, lines);
        match engine.execute_show(&ShowArgs::All) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    /// Execute `SHOW 'text'` (reveal excluded lines containing text).
    pub fn show_text(
        &mut self,
        text: &str,
        tab_id: u64,
        line_count: usize,
        lines: impl FnOnce() -> Vec<String>,
    ) -> String {
        let engine = self.engine_for(tab_id, line_count, lines);
        match engine.execute_show(&ShowArgs::Text {
            pattern: text.to_string(),
        }) {
            Ok(r) => r.message,
            Err(e) => e.to_string(),
        }
    }

    // === RESET ======================================================

    /// Execute `RESET` / `RESET EXCLUDED` / `RESET ALL`.
    pub fn reset(
        &mut self,
        variant: ResetVariant,
        tab_id: u64,
        line_count: usize,
        lines: impl FnOnce() -> Vec<String>,
    ) -> String {
        let engine = self.engine_for(tab_id, line_count, lines);
        engine.execute_reset(variant).message
    }

    // === Internal ===================================================

    /// Get or create the engine for the given tab, rebuilding if the line count
    /// changed. The `lines` closure is invoked ONLY when a (re)build is needed,
    /// preserving the shell's snapshot-on-rebuild laziness exactly.
    fn engine_for(
        &mut self,
        tab_id: u64,
        line_count: usize,
        lines: impl FnOnce() -> Vec<String>,
    ) -> &mut ExclusionEngine<ContractionState, TabDocAdapter> {
        // Rebuild if missing or stale (line count changed after edits)
        let needs_rebuild = self
            .engines
            .get(&tab_id)
            .map(|e| e.line_count() != line_count)
            .unwrap_or(true);

        if needs_rebuild {
            let mapping = ContractionState::new(line_count);
            let doc = TabDocAdapter::new(lines());
            self.engines
                .insert(tab_id, ExclusionEngine::new(mapping, doc));
        }

        self.engines.get_mut(&tab_id).expect("just inserted")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: Requirement 21.5 -- EXCLUDE ALL hides all lines.
    #[test]
    fn exclude_all_reports_lines_excluded() {
        let mut mgr = ExcludeManager::new();
        let msg = mgr.exclude_all(1, 3, || vec!["alpha".into(), "beta".into(), "gamma".into()]);
        assert!(
            msg.contains("excluded") || msg.contains("line"),
            "expected exclusion message, got: {msg}"
        );
        assert!(mgr.engines[&1].has_excluded_lines());
    }

    /// Validates: Requirement 21.5 -- EXCLUDE 'text' hides matching lines only.
    #[test]
    fn exclude_text_hides_matching_lines() {
        let mut mgr = ExcludeManager::new();
        let msg = mgr.exclude_text("hello", 1, 3, || {
            vec!["hello world".into(), "foo bar".into(), "hello again".into()]
        });
        assert!(msg.contains("2") || msg.contains("line"), "got: {msg}");
        let engine = &mgr.engines[&1];
        assert!(engine.is_excluded(0), "line 0 should be excluded");
        assert!(!engine.is_excluded(1), "line 1 should be visible");
        assert!(engine.is_excluded(2), "line 2 should be excluded");
    }

    /// Validates: Requirement 21.5 -- SHOW ALL restores all lines.
    #[test]
    fn show_all_restores_all_lines() {
        let mut mgr = ExcludeManager::new();
        let content = || vec!["alpha".into(), "beta".into(), "gamma".into()];
        mgr.exclude_all(1, 3, content);
        let msg = mgr.show_all(1, 3, || vec!["alpha".into(), "beta".into(), "gamma".into()]);
        assert!(msg.contains("shown") || msg.contains("line"), "got: {msg}");
        assert!(!mgr.engines[&1].has_excluded_lines());
    }

    /// Validates: Requirement 21.5 -- RESET EXCLUDED clears all exclusion state.
    #[test]
    fn reset_excluded_clears_exclusion_state() {
        let mut mgr = ExcludeManager::new();
        mgr.exclude_all(1, 3, || vec!["alpha".into(), "beta".into(), "gamma".into()]);
        let msg = mgr.reset(ResetVariant::Excluded, 1, 3, || {
            vec!["alpha".into(), "beta".into(), "gamma".into()]
        });
        assert!(
            msg.contains("RESET") || msg.contains("restored"),
            "got: {msg}"
        );
        assert!(!mgr.engines[&1].has_excluded_lines());
    }

    /// Validates: Requirement 21.5 -- exclusion_blocks returns correct block count.
    #[test]
    fn exclusion_blocks_returns_correct_count() {
        let mut mgr = ExcludeManager::new();
        // Exclude lines 0 and 2 (0-based) -> two separate blocks
        {
            let engine = mgr.engine_for(1, 5, || {
                vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()]
            });
            engine.exclude_line(0);
            engine.exclude_line(2);
        }
        let blocks = mgr.exclusion_blocks(1);
        assert_eq!(blocks.len(), 2);
    }
}
