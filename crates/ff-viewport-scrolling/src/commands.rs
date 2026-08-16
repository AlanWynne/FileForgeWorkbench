//! Scroll command definitions and handler.
//!
//! Defines the set of scroll commands for registration with the command
//! framework. Scroll commands are navigation-only and NOT recorded on
//! the undo stack.

/// Scroll commands registered with the command framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollCommand {
    /// Scroll viewport up by one line.
    ScrollLineUp,
    /// Scroll viewport down by one line.
    ScrollLineDown,
    /// Scroll viewport up by one page.
    ScrollPageUp,
    /// Scroll viewport down by one page.
    ScrollPageDown,
    /// Scroll viewport to a specific line.
    ScrollToLine(u64),
    /// Scroll viewport to the top.
    ScrollToTop,
    /// Scroll viewport to the bottom.
    ScrollToBottom,
    /// Set horizontal scroll offset.
    ScrollHorizontal(u64),
}

impl ScrollCommand {
    /// Command identifier string for the command framework.
    pub fn id(&self) -> &'static str {
        match self {
            Self::ScrollLineUp => "viewport.scrollLineUp",
            Self::ScrollLineDown => "viewport.scrollLineDown",
            Self::ScrollPageUp => "viewport.scrollPageUp",
            Self::ScrollPageDown => "viewport.scrollPageDown",
            Self::ScrollToLine(_) => "viewport.scrollToLine",
            Self::ScrollToTop => "viewport.scrollToTop",
            Self::ScrollToBottom => "viewport.scrollToBottom",
            Self::ScrollHorizontal(_) => "viewport.scrollHorizontal",
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ScrollLineUp => "Scroll Line Up",
            Self::ScrollLineDown => "Scroll Line Down",
            Self::ScrollPageUp => "Scroll Page Up",
            Self::ScrollPageDown => "Scroll Page Down",
            Self::ScrollToLine(_) => "Scroll To Line",
            Self::ScrollToTop => "Scroll To Top",
            Self::ScrollToBottom => "Scroll To Bottom",
            Self::ScrollHorizontal(_) => "Scroll Horizontal",
        }
    }

    /// Category for grouping in command palette.
    pub fn category(&self) -> &'static str {
        "Navigation"
    }

    /// Whether this command should be recorded on the undo stack.
    /// Scroll commands are navigation-only — always false.
    pub fn is_undoable(&self) -> bool {
        false
    }
}

use crate::caret_policy::CaretPolicyEngine;
use crate::cursor::CursorModel;
use crate::viewport::ViewportModel;

/// Execute a scroll command against the viewport model.
///
/// Returns true if the command was handled, false if the command
/// could not be executed (e.g., invalid target line).
pub fn execute_scroll_command(
    command: &ScrollCommand,
    viewport: &mut ViewportModel,
    cursor: &mut CursorModel,
    _policy: &CaretPolicyEngine,
) -> bool {
    match command {
        ScrollCommand::ScrollLineUp => {
            viewport.scroll_line_up(cursor);
            true
        }
        ScrollCommand::ScrollLineDown => {
            viewport.scroll_line_down(cursor);
            true
        }
        ScrollCommand::ScrollPageUp => {
            viewport.scroll_page_up(cursor);
            true
        }
        ScrollCommand::ScrollPageDown => {
            viewport.scroll_page_down(cursor);
            true
        }
        ScrollCommand::ScrollToLine(line) => {
            if *line == 0 {
                return false;
            }
            viewport.scroll_to_line(*line, cursor);
            true
        }
        ScrollCommand::ScrollToTop => {
            viewport.scroll_to_top(cursor);
            true
        }
        ScrollCommand::ScrollToBottom => {
            viewport.scroll_to_bottom(cursor);
            true
        }
        ScrollCommand::ScrollHorizontal(offset) => {
            viewport.set_horizontal_offset(*offset, cursor);
            true
        }
    }
}
