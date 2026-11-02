//! Pure render for the Alias Editor Context, now DELEGATED to the shared ISPF
//! Table Editor surface (command-configurator Req 5.13-5.24, Req 7; CR-CH-068 +
//! CR-CH-075 FEAT-002).
//!
//! The chrome (title, `Command ===>` line, headers, FILTER row frame, scroll
//! area, empty-store scratch row) is drawn ONCE by
//! [`crate::shell::ispf_table_editor::render::render_ispf_table`]; the Alias
//! editor supplies the host contract (columns, line-command set, sort key,
//! filter) and the per-cell egui painting (the cascading CE -> Root dropdowns,
//! the text fields) via its [`IspfTableHost`] / [`IspfTableRender`] impls in
//! `host.rs`. The render stays PURE: it returns an [`AliasEditorAction`] and the
//! shell applies the structural side effects. Per-field edits bind directly to
//! the mutable working rows (B054 pattern).

use super::filter::displayed_indices;
use super::state::{AliasEditorAction, AliasEditorState};
use crate::shell::ispf_table_editor::render::{render_ispf_table, TableAction, TableRenderParams};
use crate::shell::ispf_table_editor::state::TableEditorState;

/// `WorkspaceContext` impl (CR-NR-078): render the Alias Editor, stash the
/// produced [`AliasEditorAction`] on `pending_action`, and report interior focus
/// (FIRST = the FILTER row's filter CE dropdown captured fresh each frame; LAST =
/// the same control, anchoring the reverse boundary).
///
/// Validates: command-configurator Requirement 5.13, 5.23.
impl crate::shell::workspace_context::WorkspaceContext for AliasEditorState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        _services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        self.pending_action = render(ui, self);
        crate::shell::workspace_context::InteriorFocus {
            first: self.first_interior_id,
            last: self.last_interior_id,
        }
    }
}

/// Render the Alias Editor Context through the shared surface, returning the
/// action to apply. Tab focus order is egui-native and set by the shared
/// surface's control creation order (`Command ===>` entry; then filter CE ->
/// filter Root -> each data row Act -> CE -> Root -> ALIAS -> Parameters ->
/// Description), with the FILTER CE dropdown reported as the first interior stop
/// (Req 5.21, 5.23).
///
/// Validates: command-configurator Requirement 5.13-5.23, 7.2, 7.3
pub fn render(ui: &mut egui::Ui, state: &mut AliasEditorState) -> AliasEditorAction {
    // Reset the reported focus anchors each frame; set from the surface below.
    state.first_interior_id = None;
    state.last_interior_id = None;

    // Pre-filter to the BACKING indices to display (CE + Root; backing indices,
    // never display positions -- the index hazard stays the host's concern).
    let displayed = displayed_indices(&state.rows, state.filter_ce, &state.filter_root);

    // Move the rows out so the shared surface can borrow the host (`state`)
    // mutably for cell rendering. CR-CH-077 single-chrome: the surface no longer
    // owns a `Command ===>` line, so the transient `TableEditorState` carries no
    // command text -- the shell's app-level command field is the one command
    // line and is routed into the Alias command-line handler by the shell.
    let mut editor = TableEditorState::default();
    let mut rows = std::mem::take(&mut state.rows);
    let error = state.error.clone();

    let (table_action, anchors) = render_ispf_table(
        ui,
        state,
        &mut editor,
        &mut rows,
        &displayed,
        TableRenderParams {
            error: error.as_deref(),
            blank_scratch: super::state::AliasRowEdit::default,
        },
    );

    // Put the owned pieces back and record the reported focus anchors.
    state.rows = rows;
    state.first_interior_id = anchors.first;
    state.last_interior_id = anchors.last;

    // CR-CH-077 single-chrome: the surface only reports the in-Act-cell Enter;
    // a typed `Command ===>` verb arrives via the shell's command field, routed
    // straight to `apply_alias_command_line`, never through this action.
    match table_action {
        TableAction::None => AliasEditorAction::None,
        TableAction::ApplyLineCommands => AliasEditorAction::ApplyLineCommands,
    }
}
