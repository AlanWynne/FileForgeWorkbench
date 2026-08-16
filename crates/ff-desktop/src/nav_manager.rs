//! Navigation command bridge — wires `ff-navigation-commands` into the desktop shell.
//!
//! Handles LOCATE, SORT, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM commands by
//! delegating to the `ff-navigation-commands` crate, operating on the active
//! tab's `ViewportModel` and `CursorModel`.

use ff_navigation_commands::{
    LocateCommand, NavigationConfig, NavigationError, ScrollCommands, SortCommand,
};
use tokio::runtime::Runtime;

use crate::tab_manager::TabManager;

/// Bridges navigation commands to the active tab's viewport and cursor.
pub struct NavManager {
    config: NavigationConfig,
}

impl NavManager {
    pub fn new() -> Self {
        Self {
            config: NavigationConfig::default(),
        }
    }

    /// Execute `LOCATE <arg>` — jump to line number or label.
    ///
    /// Returns a status string (empty on success, error message on failure).
    pub fn locate(&self, arg: &str, tabs: &mut TabManager) -> String {
        let tab = tabs.active_tab_mut();
        let line_count = tab.line_count;
        match LocateCommand::parse_argument(arg) {
            Ok(line_num) => {
                match LocateCommand::locate_line(
                    &mut tab.viewport,
                    &mut tab.cursor,
                    line_num,
                    line_count,
                ) {
                    Ok(()) => String::new(),
                    Err(NavigationError::LineOutOfRange) => "Line number out of range".to_string(),
                    Err(e) => e.to_string(),
                }
            }
            Err(label) => format!("Label not found: {label}"),
        }
    }

    /// Execute `SORT [args…]` — sort visible lines of the active tab.
    pub fn sort(&self, args: &[&str], tabs: &mut TabManager, runtime: &Runtime) -> String {
        let params = match SortCommand::parse_args(args) {
            Ok(p) => p,
            Err(e) => return e.to_string(),
        };

        let tab = tabs.active_tab_mut();
        let line_count = tab.line_count as usize;

        // Collect all lines
        let mut lines: Vec<String> = runtime.block_on(async {
            let doc = tab.document.read().await;
            (1..=line_count as u64)
                .map(|ln| {
                    let start = doc.line_start(ff_document_model::LineNumber(ln - 1));
                    let end = doc.line_end(ff_document_model::LineNumber(ln - 1));
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
        });

        match SortCommand::execute(&mut lines, &params, None) {
            Ok(_record) => {
                // Write sorted lines back — rebuild entire document content
                let new_content = lines.join("\n");
                runtime.block_on(async {
                    let mut doc = tab.document.write().await;
                    let total_len = doc.length();
                    let _ = doc.delete(ff_document_model::BytePosition(0), total_len);
                    let _ = doc.insert(ff_document_model::BytePosition(0), new_content.as_bytes());
                });
                tab.is_modified = true;
                tab.line_count = runtime.block_on(async { tab.document.read().await.line_count() });
                tab.viewport.set_total_display_lines(tab.line_count);
                String::new()
            }
            Err(NavigationError::NothingToSort) => "Nothing to sort".to_string(),
            Err(e) => e.to_string(),
        }
    }

    /// Execute `UP [n]`.
    pub fn up(&self, arg: Option<u64>, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        match arg {
            Some(n) => ScrollCommands::up_lines(&mut tab.viewport, &mut tab.cursor, n),
            None => ScrollCommands::up_page(&mut tab.viewport, &mut tab.cursor, &self.config),
        }
    }

    /// Execute `DOWN [n]`.
    pub fn down(&self, arg: Option<u64>, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        match arg {
            Some(n) => ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, n),
            None => ScrollCommands::down_page(&mut tab.viewport, &mut tab.cursor, &self.config),
        }
    }

    /// Execute `LEFT [n]`.
    pub fn left(&self, arg: Option<u64>, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        match arg {
            Some(n) => ScrollCommands::left_columns(&mut tab.viewport, &tab.cursor, n),
            None => ScrollCommands::left_default(&mut tab.viewport, &tab.cursor, &self.config),
        }
    }

    /// Execute `RIGHT [n]`.
    pub fn right(&self, arg: Option<u64>, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        match arg {
            Some(n) => ScrollCommands::right_columns(&mut tab.viewport, &tab.cursor, n),
            None => ScrollCommands::right_default(&mut tab.viewport, &tab.cursor, &self.config),
        }
    }

    /// Execute `TOP`.
    pub fn top(&self, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        ScrollCommands::top(&mut tab.viewport, &mut tab.cursor);
    }

    /// Execute `BOTTOM`.
    pub fn bottom(&self, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        let line_count = tab.line_count;
        ScrollCommands::bottom(&mut tab.viewport, &mut tab.cursor, line_count);
    }
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

    /// Validates: Requirement 21.4 — LOCATE with valid line number scrolls viewport.
    #[test]
    fn locate_valid_line_returns_empty_status() {
        let (mut tabs, _rt) = make_tabs("line1\nline2\nline3\nline4\nline5\n");
        let nav = NavManager::new();
        let status = nav.locate("3", &mut tabs);
        assert!(status.is_empty(), "expected empty status, got: {status}");
        assert_eq!(tabs.active_tab().cursor.cursor_line(), 3);
    }

    /// Validates: Requirement 21.4 — LOCATE with out-of-range line returns error.
    #[test]
    fn locate_out_of_range_returns_error() {
        let (mut tabs, _rt) = make_tabs("line1\nline2\n");
        let nav = NavManager::new();
        let status = nav.locate("999", &mut tabs);
        assert!(!status.is_empty(), "expected error status");
        assert!(status.contains("out of range") || status.contains("range"));
    }

    /// Validates: Requirement 21.4 — UP scrolls viewport up.
    #[test]
    fn up_scrolls_viewport() {
        let content: String = (1..=50).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(10);
            ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, 20);
        }
        let before = tabs.active_tab().viewport.top_line();
        let nav = NavManager::new();
        nav.up(Some(5), &mut tabs);
        let after = tabs.active_tab().viewport.top_line();
        assert!(after < before, "UP should decrease top_line");
    }

    /// Validates: Requirement 21.4 — DOWN scrolls viewport down.
    #[test]
    fn down_scrolls_viewport() {
        let content: String = (1..=50).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(10);
        }
        let before = tabs.active_tab().viewport.top_line();
        let nav = NavManager::new();
        nav.down(Some(5), &mut tabs);
        let after = tabs.active_tab().viewport.top_line();
        assert!(after > before, "DOWN should increase top_line");
    }

    /// Validates: Requirement 21.4 — TOP scrolls to line 1.
    #[test]
    fn top_scrolls_to_line_1() {
        let content: String = (1..=50).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(10);
            ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, 30);
        }
        let nav = NavManager::new();
        nav.top(&mut tabs);
        assert_eq!(tabs.active_tab().viewport.top_line(), 1);
    }

    /// Validates: Requirement 21.4 — BOTTOM scrolls to last page.
    #[test]
    fn bottom_scrolls_to_last_page() {
        let content: String = (1..=50).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(10);
        }
        let nav = NavManager::new();
        nav.bottom(&mut tabs);
        let tab = tabs.active_tab();
        assert_eq!(tab.viewport.top_line(), tab.viewport.max_top_line());
    }
}
