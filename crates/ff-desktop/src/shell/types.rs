//! # Shell Value Types
//!
//! Small value/enum types owned by the shell: the Key_Label_Bar scope and the
//! Detached_Workspace (floating tab) bookkeeping types. Moved out of `mod.rs`
//! verbatim as part of the Phase 2 task 2.2 file-size split and re-exported from
//! `mod.rs` via `pub(crate) use types::*;`, so every `super::KeyBarScope`,
//! `super::WorkspaceCommandContext`, and `super::FloatingTab` reference in
//! sibling modules and tests keeps resolving unchanged.

use eframe::egui;
use ff_keys::KeyModifier;

use super::command_line_outcome;
use super::ScrollAmount;

// === Key_Label_Bar scope (CR-CH-046) ===

/// Which modifier layer the Key_Label_Bar shows when visible (CR-CH-046).
///
/// The bar's overall mode is the pair `(key_bar_visible, key_bar_scope)`:
/// when hidden the bar is "Off" and `key_bar_scope` retains the last-shown scope
/// so `PFSHOW ON` can restore it. `Base` shows the plain F-keys, `Shift` the
/// SHIFT layer, `Ctrl` the CTRL layer, and `Alt` the ALT layer.
///
/// Validates: function-keys-and-history Requirement 12.8-12.12, Requirement 13.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum KeyBarScope {
    /// Base (unmodified) F-keys.
    #[default]
    Base,
    /// SHIFT + F-keys.
    Shift,
    /// CTRL + F-keys.
    Ctrl,
    /// ALT + F-keys.
    Alt,
}

impl KeyBarScope {
    /// The `KeyModifier` layer this scope displays.
    pub(crate) fn to_modifier(self) -> KeyModifier {
        match self {
            KeyBarScope::Base => KeyModifier::None,
            KeyBarScope::Shift => KeyModifier::Shift,
            KeyBarScope::Ctrl => KeyModifier::Ctrl,
            KeyBarScope::Alt => KeyModifier::Alt,
        }
    }

    /// The leading Scope_Segment label shown on the Key_Label_Bar.
    pub(crate) fn segment_label(self) -> &'static str {
        match self {
            KeyBarScope::Base => "Base",
            KeyBarScope::Shift => "Shift",
            KeyBarScope::Ctrl => "Ctrl",
            KeyBarScope::Alt => "Alt",
        }
    }

    /// The lowercase name used for session persistence (`"base"` etc.).
    pub(crate) fn persist_name(self) -> &'static str {
        match self {
            KeyBarScope::Base => "base",
            KeyBarScope::Shift => "shift",
            KeyBarScope::Ctrl => "ctrl",
            KeyBarScope::Alt => "alt",
        }
    }

    /// Parse a scope from a persisted/command name (case-insensitive). Returns
    /// `None` for an unrecognised value (caller decides the fallback).
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "BASE" => Some(KeyBarScope::Base),
            "SHIFT" => Some(KeyBarScope::Shift),
            "CTRL" => Some(KeyBarScope::Ctrl),
            "ALT" => Some(KeyBarScope::Alt),
            _ => None,
        }
    }
}

// === Detachable tab windows -- Validates: Requirement 18.1-18.7 ===

/// The per-window command state that makes a Detached_Workspace an INDEPENDENT
/// command context (CR-CH-036, menu-and-statusbar Req 18.10). Each detached
/// window owns one: its own `Command ===>` buffer, SCROLL buffer/amount, status
/// line, and the command-field focus + Command_Line_Outcome latches. The shell's
/// own same-named fields serve as the Primary_Window's implicit context.
///
/// The command pipeline stays bound to the shell's fields; to dispatch for a
/// detached window we temporarily swap this context into the shell (see
/// `WorkbenchShell::with_workspace_context`), run the UNCHANGED pipeline, then
/// swap the (possibly command-modified) buffers back -- so each window's command
/// line acts only on its own tab.
#[derive(Debug, Clone)]
pub(crate) struct WorkspaceCommandContext {
    pub command_text: String,
    pub scroll_field_text: String,
    pub scroll_amount: ScrollAmount,
    pub open_error: Option<String>,
    pub command_field_focus_requested: bool,
    pub pending_command_line_outcome: Option<command_line_outcome::CommandLineOutcome>,
}

impl Default for WorkspaceCommandContext {
    fn default() -> Self {
        Self {
            command_text: String::new(),
            scroll_field_text: "PAGE".to_string(),
            scroll_amount: ScrollAmount::default(),
            open_error: None,
            command_field_focus_requested: false,
            pending_command_line_outcome: None,
        }
    }
}

/// Tracks a tab that has been detached into a floating OS window.
///
/// Validates: Requirement 18.1, 18.2, 18.3, 18.10
pub(crate) struct FloatingTab {
    /// egui viewport id allocated for this floating window.
    pub viewport_id: egui::ViewportId,
    /// Stable identity of the detached tab. The live TabManager index is resolved
    /// from this each frame (`index_of_id`) so concurrent detach/redock reorderings
    /// never desync the floating window from its tab (CR-CH-035, Req 18.9).
    pub tab_id: crate::tab_state::TabId,
    /// The tab index at the moment of detach -- used to restore position on redock.
    pub origin_index: usize,
    /// This window's INDEPENDENT command context (CR-CH-036, Req 18.10): its own
    /// command line, SCROLL, status, and focus/outcome latches.
    pub cmd_ctx: WorkspaceCommandContext,
}
