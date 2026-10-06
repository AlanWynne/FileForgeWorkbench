//! Theme Editor Context -- shell adapter (CR-NR-098 decomposition Wave 1).
//!
//! The pure part of the Theme Editor (the [`EditableToken`] model, the
//! [`ThemeEditorAction`] enum, the [`ThemeEditorState`] UI state, and the
//! [`render`] free function) now lives in the standalone `ff-theme-editor`
//! crate, which depends only on `ff_theme` + `egui`. Editing that panel no
//! longer recompiles the whole `ff-desktop` binary crate.
//!
//! This module is the THIN shell-side adapter that stays in `ff-desktop`
//! because it wires the crate onto shell-owned infrastructure:
//!
//! - it re-exports the crate's pure types so existing `crate::theme_editor_panel::*`
//!   references keep resolving unchanged;
//! - it holds the [`crate::shell::workspace_context::WorkspaceContext`] impl,
//!   which MUST live here: `ff-desktop` owns that trait (it depends on shell
//!   types the pure crate cannot see), so the impl is legal only in this crate.
//!
//! The rich side effects (file writes, palette swap, config persist) remain in
//! `shell/commands.rs::apply_theme_editor_action`; the render stays pure.
//!
//! Validates: theme-and-appearance Requirement 20; workspace-framework
//! Requirement 1.4, 1.5, 6.1.

use eframe::egui;

// Re-export the pure Theme Editor API from the extracted crate so that every
// existing `crate::theme_editor_panel::{EditableToken, ThemeEditorAction,
// ThemeEditorState, render}` reference in the shell resolves unchanged.
pub use ff_theme_editor::{render, EditableToken, ThemeEditorAction, ThemeEditorState};

/// `WorkspaceContext` impl (CR-NR-078): render the Theme Editor, stash the
/// produced [`ThemeEditorAction`] on `pending_action` for the shell to apply via
/// `apply_theme_editor_action` (the action is rich shell-side state, not a
/// generic `ShellRequest`), and report the interior focus contract: FIRST = the
/// Theme selector combo (captured on `first_interior_id`), LAST = the final
/// colour hex field.
///
/// The trait is local to `ff-desktop`, so this impl on the extracted
/// `ThemeEditorState` type satisfies the orphan rule and keeps the single
/// focus-latch path (framework-conformance mechanism 5).
///
/// Validates: workspace-framework Requirement 1.4, 1.5, 6.1.
impl crate::shell::workspace_context::WorkspaceContext for ThemeEditorState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        _services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        self.pending_action = render(ui, self);
        let last = egui::Id::new(("theme_editor_hex", EditableToken::all().len() - 1));
        crate::shell::workspace_context::InteriorFocus {
            first: self.first_interior_id,
            last: Some(last),
        }
    }
}
