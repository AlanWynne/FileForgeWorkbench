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

use crate::scroll_amount::ScrollAmount;

/// Direction of a no-argument scroll (CR-NR-087).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDir {
    Up,
    Down,
}

/// The concrete scroll action a no-argument `UP`/`DOWN` resolves to once the
/// active `ScrollAmount` and the viewport metrics are known (CR-NR-087,
/// navigation-commands Req 3.17-3.22). Pure/`Eq` so the mapping is unit-testable
/// without egui or a live viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAction {
    /// Scroll by `n` lines in the command's direction (PAGE/DATA/HALF/Lines).
    Lines(u64),
    /// Jump to the top of the document (`UP` + MAX).
    ToTop,
    /// Jump to the last page (`DOWN` + MAX).
    ToBottom,
    /// Position the current cursor line at the top of the viewport (CSR).
    CursorToTop,
}

/// Resolve a no-argument `UP`/`DOWN` to a concrete [`ScrollAction`] from the
/// active `ScrollAmount`, the viewport's `visible_count`, and the direction
/// (CR-NR-087, Req 3.17-3.22). PURE: no viewport mutation, no egui.
///
/// - PAGE / DATA -> `Lines(visible_count)` (one screen; DATA == page here).
/// - HALF        -> `Lines(max(1, visible_count / 2))`.
/// - Lines(n)    -> `Lines(n)`.
/// - MAX         -> `ToTop` for Up, `ToBottom` for Down.
/// - CSR         -> `CursorToTop`.
pub fn resolve_scroll_action(
    amount: &ScrollAmount,
    visible_count: u64,
    dir: ScrollDir,
) -> ScrollAction {
    match amount {
        ScrollAmount::Page | ScrollAmount::Data => ScrollAction::Lines(visible_count.max(1)),
        ScrollAmount::Half => ScrollAction::Lines((visible_count / 2).max(1)),
        ScrollAmount::Lines(n) => ScrollAction::Lines((*n).max(1)),
        ScrollAmount::Max => match dir {
            ScrollDir::Up => ScrollAction::ToTop,
            ScrollDir::Down => ScrollAction::ToBottom,
        },
        ScrollAmount::Csr => ScrollAction::CursorToTop,
    }
}

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

    /// Execute a no-argument `UP` governed by the active `ScrollAmount`
    /// (CR-NR-087, Req 3.17-3.22). Numeric `UP n` still goes through [`up`](Self::up).
    pub fn up_by_amount(&self, amount: &ScrollAmount, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        let vc = tab.viewport.visible_count();
        match resolve_scroll_action(amount, vc, ScrollDir::Up) {
            ScrollAction::Lines(n) => {
                ScrollCommands::up_lines(&mut tab.viewport, &mut tab.cursor, n)
            }
            ScrollAction::ToTop => ScrollCommands::top(&mut tab.viewport, &mut tab.cursor),
            ScrollAction::ToBottom => {
                let lc = tab.line_count;
                ScrollCommands::bottom(&mut tab.viewport, &mut tab.cursor, lc)
            }
            ScrollAction::CursorToTop => Self::scroll_cursor_to_top(tab),
        }
    }

    /// Execute a no-argument `DOWN` governed by the active `ScrollAmount`
    /// (CR-NR-087, Req 3.17-3.22). Numeric `DOWN n` still goes through [`down`](Self::down).
    pub fn down_by_amount(&self, amount: &ScrollAmount, tabs: &mut TabManager) {
        let tab = tabs.active_tab_mut();
        let vc = tab.viewport.visible_count();
        match resolve_scroll_action(amount, vc, ScrollDir::Down) {
            ScrollAction::Lines(n) => {
                ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, n)
            }
            ScrollAction::ToTop => ScrollCommands::top(&mut tab.viewport, &mut tab.cursor),
            ScrollAction::ToBottom => {
                let lc = tab.line_count;
                ScrollCommands::bottom(&mut tab.viewport, &mut tab.cursor, lc)
            }
            ScrollAction::CursorToTop => Self::scroll_cursor_to_top(tab),
        }
    }

    /// CSR: scroll so the current `cursor_line` becomes the topmost visible line
    /// (Req 3.22), reusing the existing clamped `up_lines`/`down_lines` so the
    /// viewport's clamping (Req 3.11/3.12) applies. Computes the signed delta
    /// between the current `top_line` and the target and scrolls that many lines.
    fn scroll_cursor_to_top(tab: &mut crate::tab_state::TabState) {
        let target = tab.cursor.cursor_line().max(1);
        let current_top = tab.viewport.top_line();
        if target < current_top {
            ScrollCommands::up_lines(&mut tab.viewport, &mut tab.cursor, current_top - target);
        } else if target > current_top {
            ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, target - current_top);
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

    // === CR-NR-087: no-argument UP/DOWN honour the active SCROLL amount ======

    /// Validates: navigation-commands Req 3.18/3.19/3.20 -- PAGE/DATA/HALF/Lines
    /// resolve to a line count in both directions.
    #[test]
    fn resolve_scroll_action_line_amounts() {
        use ScrollAmount::*;
        for dir in [ScrollDir::Up, ScrollDir::Down] {
            assert_eq!(
                resolve_scroll_action(&Page, 20, dir),
                ScrollAction::Lines(20)
            );
            assert_eq!(
                resolve_scroll_action(&Data, 20, dir),
                ScrollAction::Lines(20)
            );
            assert_eq!(
                resolve_scroll_action(&Half, 20, dir),
                ScrollAction::Lines(10)
            );
            // Half clamps to at least 1 line.
            assert_eq!(resolve_scroll_action(&Half, 1, dir), ScrollAction::Lines(1));
            assert_eq!(
                resolve_scroll_action(&Lines(7), 20, dir),
                ScrollAction::Lines(7)
            );
        }
    }

    /// Validates: navigation-commands Req 3.21 -- MAX resolves to ToTop for Up,
    /// ToBottom for Down.
    #[test]
    fn resolve_scroll_action_max_is_directional() {
        assert_eq!(
            resolve_scroll_action(&ScrollAmount::Max, 20, ScrollDir::Up),
            ScrollAction::ToTop
        );
        assert_eq!(
            resolve_scroll_action(&ScrollAmount::Max, 20, ScrollDir::Down),
            ScrollAction::ToBottom
        );
    }

    /// Validates: navigation-commands Req 3.22 -- CSR resolves to CursorToTop in
    /// both directions.
    #[test]
    fn resolve_scroll_action_csr_is_cursor_to_top() {
        assert_eq!(
            resolve_scroll_action(&ScrollAmount::Csr, 20, ScrollDir::Up),
            ScrollAction::CursorToTop
        );
        assert_eq!(
            resolve_scroll_action(&ScrollAmount::Csr, 20, ScrollDir::Down),
            ScrollAction::CursorToTop
        );
    }

    /// Validates: navigation-commands Req 3.21 -- SCROLL MAX then UP scrolls to
    /// the top of the document (the B046 row 7.3a case).
    #[test]
    fn up_by_amount_max_scrolls_to_top() {
        let content: String = (1..=50).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(10);
            ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, 30);
        }
        assert!(tabs.active_tab().viewport.top_line() > 1, "precondition");
        let nav = NavManager::new();
        nav.up_by_amount(&ScrollAmount::Max, &mut tabs);
        assert_eq!(tabs.active_tab().viewport.top_line(), 1);
    }

    /// Validates: navigation-commands Req 3.21 -- SCROLL MAX then DOWN scrolls to
    /// the last page.
    #[test]
    fn down_by_amount_max_scrolls_to_bottom() {
        let content: String = (1..=50).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(10);
        }
        let nav = NavManager::new();
        nav.down_by_amount(&ScrollAmount::Max, &mut tabs);
        let tab = tabs.active_tab();
        assert_eq!(tab.viewport.top_line(), tab.viewport.max_top_line());
    }

    /// Validates: navigation-commands Req 3.19 -- SCROLL HALF then DOWN advances
    /// by half a page.
    #[test]
    fn down_by_amount_half_advances_half_page() {
        let content: String = (1..=100).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(20);
        }
        let before = tabs.active_tab().viewport.top_line();
        let nav = NavManager::new();
        nav.down_by_amount(&ScrollAmount::Half, &mut tabs);
        let after = tabs.active_tab().viewport.top_line();
        assert_eq!(after - before, 10, "half of visible_count 20 == 10 lines");
    }

    /// Validates: navigation-commands Req 3.18 -- SCROLL PAGE (default) then DOWN
    /// advances by one full page (unchanged default behaviour).
    #[test]
    fn down_by_amount_page_advances_full_page() {
        let content: String = (1..=100).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(20);
        }
        let before = tabs.active_tab().viewport.top_line();
        let nav = NavManager::new();
        nav.down_by_amount(&ScrollAmount::Page, &mut tabs);
        let after = tabs.active_tab().viewport.top_line();
        assert_eq!(after - before, 20, "one page == visible_count 20 lines");
    }

    /// Validates: navigation-commands Req 3.22 -- SCROLL CSR then UP puts the
    /// cursor line at the top of the viewport.
    #[test]
    fn up_by_amount_csr_scrolls_cursor_to_top() {
        let content: String = (1..=100).map(|i| format!("line {i}\n")).collect();
        let (mut tabs, _rt) = make_tabs(&content);
        {
            let tab = tabs.active_tab_mut();
            tab.viewport.set_visible_count(20);
            // Scroll down so top_line is well past 1, then place the cursor on a
            // line ABOVE the current top so CSR must scroll up to it.
            ScrollCommands::down_lines(&mut tab.viewport, &mut tab.cursor, 40);
            tab.cursor.set_position(15, 1);
        }
        let nav = NavManager::new();
        nav.up_by_amount(&ScrollAmount::Csr, &mut tabs);
        assert_eq!(
            tabs.active_tab().viewport.top_line(),
            15,
            "CSR positions the cursor line at the top"
        );
    }
}
