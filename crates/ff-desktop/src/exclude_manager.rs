//! Exclude/Show/Reset adapter -- wires `ff-exclude-manager` into the desktop
//! shell.
//!
//! The pure per-tab exclusion engine store moved into the `ff-exclude-manager`
//! crate (CR-NR-098 decomposition Wave 6, Task 21). That crate is keyed on a
//! plain `u64` tab id and takes a lazily-supplied line snapshot, so it has no
//! dependency on the shell runtime, Tokio, or the document model.
//!
//! This module is the thin shell-side adapter. It re-exports `ExcludeManager`
//! so every existing `crate::exclude_manager::ExcludeManager` path keeps
//! resolving, and it owns `snapshot_lines`, which reads the active `TabState`'s
//! document through the Tokio runtime to produce the `Vec<String>` the crate's
//! command methods consume. The snapshot closure is still invoked only when the
//! engine must be (re)built, so EXCLUDE/SHOW/RESET behaviour is unchanged.

use tokio::runtime::Runtime;

pub use ff_exclude_manager::ExcludeManager;

/// Snapshot all document lines as `Vec<String>` for the exclusion engine.
pub fn snapshot_lines(tab: &crate::tab_state::TabState, runtime: &Runtime) -> Vec<String> {
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
    use ff_exclude_show_filter::ResetVariant;
    use tokio::runtime::Runtime;

    fn make_tabs(content: &str) -> (TabManager, Runtime) {
        let rt = Runtime::new().expect("runtime");
        let tabs = TabManager::new(&rt, content);
        (tabs, rt)
    }

    /// Validates: Requirement 21.5 -- EXCLUDE ALL hides all lines.
    #[test]
    fn exclude_all_reports_lines_excluded() {
        let (tabs, rt) = make_tabs("alpha\nbeta\ngamma\n");
        let mut mgr = ExcludeManager::new();
        let tab = tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        let msg = mgr.exclude_all(tab_id, line_count, || snapshot_lines(tab, &rt));
        assert!(
            msg.contains("excluded") || msg.contains("line"),
            "expected exclusion message, got: {msg}"
        );
        assert!(mgr.is_excluded(tab_id, 1));
    }

    /// Validates: Requirement 21.5 -- EXCLUDE 'text' hides matching lines only.
    #[test]
    fn exclude_text_hides_matching_lines() {
        let (tabs, rt) = make_tabs("hello world\nfoo bar\nhello again\n");
        let mut mgr = ExcludeManager::new();
        let tab = tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        let msg = mgr.exclude_text("hello", tab_id, line_count, || snapshot_lines(tab, &rt));
        assert!(msg.contains("2") || msg.contains("line"), "got: {msg}");
        assert!(mgr.is_excluded(tab_id, 1), "line 1 should be excluded");
        assert!(!mgr.is_excluded(tab_id, 2), "line 2 should be visible");
        assert!(mgr.is_excluded(tab_id, 3), "line 3 should be excluded");
    }

    /// Validates: Requirement 21.5 -- SHOW ALL restores all lines.
    #[test]
    fn show_all_restores_all_lines() {
        let (tabs, rt) = make_tabs("alpha\nbeta\ngamma\n");
        let mut mgr = ExcludeManager::new();
        let tab = tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        mgr.exclude_all(tab_id, line_count, || snapshot_lines(tab, &rt));
        let msg = mgr.show_all(tab_id, line_count, || snapshot_lines(tab, &rt));
        assert!(msg.contains("shown") || msg.contains("line"), "got: {msg}");
        assert!(!mgr.is_excluded(tab_id, 1));
    }

    /// Validates: Requirement 21.5 -- RESET EXCLUDED clears all exclusion state.
    #[test]
    fn reset_excluded_clears_exclusion_state() {
        let (tabs, rt) = make_tabs("alpha\nbeta\ngamma\n");
        let mut mgr = ExcludeManager::new();
        let tab = tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        mgr.exclude_all(tab_id, line_count, || snapshot_lines(tab, &rt));
        let msg = mgr.reset(ResetVariant::Excluded, tab_id, line_count, || {
            snapshot_lines(tab, &rt)
        });
        assert!(
            msg.contains("RESET") || msg.contains("restored"),
            "got: {msg}"
        );
        assert!(!mgr.is_excluded(tab_id, 1));
    }

    /// Validates: Requirement 21.5 -- exclusion_blocks returns correct block count.
    #[test]
    fn exclusion_blocks_returns_correct_count() {
        let (tabs, rt) = make_tabs("a\nb\nc\nd\ne\n");
        let mut mgr = ExcludeManager::new();
        let tab = tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        // Exclude lines 1 and 3 (1-based) -> two separate blocks
        mgr.exclude_text("a", tab_id, line_count, || snapshot_lines(tab, &rt));
        mgr.exclude_text("c", tab_id, line_count, || snapshot_lines(tab, &rt));
        let blocks = mgr.exclusion_blocks(tab_id);
        assert_eq!(blocks.len(), 2);
    }
}
