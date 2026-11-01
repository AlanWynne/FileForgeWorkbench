//! Shared ISPF Table Editor -- the generic egui RENDER surface (CR-CH-075,
//! ispf-table-editor Requirement 4.6, 5, FEAT-002).
//!
//! [`render_ispf_table`] draws the common ISPF chrome ONCE for every table
//! editor Context: the `Command ===>` primary command line (the entry control),
//! a column header row from the host's [`super::state::ColumnSpec`] list, a
//! FILTER row, the scrolling data rows (each with an Act cell plus the host's
//! per-column cells in visual Tab order), and the always-present scratch row
//! when the backing list is empty (Req 4.6). The egui-free apply pass
//! ([`super::line_commands`]) + Sort ([`super::sort`]) stay the structural core;
//! this surface only paints and collects the typed intent.
//!
//! Cell painting is DELEGATED to the host via [`IspfTableRender`] (a SEPARATE,
//! egui-aware trait) so the pure [`super::state::IspfTableHost`] contract stays
//! GUI-free. A host implements both: `IspfTableHost` for structure/sort/filter,
//! `IspfTableRender` for the per-row and filter-row egui cells it owns (e.g. the
//! Alias editor's cascading CE -> Root dropdowns).

use super::state::{IspfTableHost, TableEditorState};

/// The structural intent the render surface collected this frame (ispf-table
/// -editor Requirement 4, 5). The host's shell applies it after the owned-panel
/// swap, exactly like the Alias editor's existing `AliasEditorAction`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TableAction {
    /// Nothing to apply this frame.
    #[default]
    None,
    /// A bare `Command ===>` Enter: apply the pending Act line commands only.
    ApplyLineCommands,
    /// A `Command ===>` verb was entered (carries the typed text). The shell
    /// applies pending line commands first, then interprets the verb.
    CommandLine(String),
}

/// The egui-aware render contract a host implements ALONGSIDE
/// [`super::state::IspfTableHost`]. It owns ONLY the per-column cell painting the
/// generic surface cannot know (dropdowns, text fields, the host's stable ids);
/// the chrome (command line, headers, scroll, scratch row) is the surface's.
pub trait IspfTableRender: IspfTableHost {
    /// The stable egui id of the `Command ===>` primary command field. Each host
    /// keeps its existing id so focus tests are unaffected.
    fn command_line_id(&self) -> egui::Id;

    /// Render the FILTER row's interactive cells (the host's filterable columns),
    /// returning the egui [`egui::Id`] of the FIRST focusable filter control so
    /// the surface can report it as the interior focus anchor. The row's leading
    /// Act-column spacer is drawn by the surface.
    fn render_filter_cells(&mut self, ui: &mut egui::Ui) -> Option<egui::Id>;

    /// Render one data row's Act cell + per-column cells for the row at BACKING
    /// index `backing`, binding edits directly to `row`. Returns `true` WHEN
    /// Enter was pressed with that row's Act field focused, so the surface can
    /// apply the pending line-command batch from the Act cell itself (the ISPF
    /// in-cell Enter, command-configurator Req 7.6).
    fn render_row_cells(&self, ui: &mut egui::Ui, backing: usize, row: &mut Self::Row) -> bool;
}

/// The reported interior focus anchors a host sets from the surface's result.
#[derive(Debug, Clone, Copy, Default)]
pub struct InteriorAnchors {
    /// First focusable interior control (the first filter cell).
    pub first: Option<egui::Id>,
    /// Last focusable interior control (kept equal to `first`, matching the
    /// Alias editor's prior behaviour where the filter CE anchors both ends).
    pub last: Option<egui::Id>,
}

/// The non-core inputs the surface needs besides `host` / `state` / `rows` /
/// `displayed`: the centred ISPF heading, the optional inline error, and the
/// blank-row factory the empty-store scratch row is built from.
pub struct TableRenderParams<'a, F> {
    /// The centred ISPF title line.
    pub title: &'a str,
    /// The optional inline error shown under the command line.
    pub error: Option<&'a str>,
    /// A factory for a fresh blank row (the empty-store scratch row + its
    /// unchanged baseline).
    pub blank_scratch: F,
}

/// Render the shared ISPF table surface for `host` over its backing `rows`,
/// returning the collected [`TableAction`] and the interior focus anchors. The
/// caller (a `WorkspaceContext::render`) stashes the action and reports the
/// anchors as its `InteriorFocus`.
///
/// `rows` is the host's working-copy list (the surface renders an always-present
/// scratch row when it is empty and promotes the scratch into `rows` the moment
/// it is edited -- Req 4.6). `displayed` is the pre-filtered list of BACKING
/// indices to show (the host computes it from its filter so the index hazard is
/// the host's concern). `params` carries the title / error / scratch factory.
///
/// Validates: ispf-table-editor Requirement 4.6, 5.1
pub fn render_ispf_table<H, FScratch>(
    ui: &mut egui::Ui,
    host: &mut H,
    state: &mut TableEditorState,
    rows: &mut Vec<H::Row>,
    displayed: &[usize],
    params: TableRenderParams<'_, FScratch>,
) -> (TableAction, InteriorAnchors)
where
    H: IspfTableRender,
    FScratch: Fn() -> H::Row,
    H::Row: PartialEq,
{
    let TableRenderParams {
        title,
        error,
        blank_scratch,
    } = params;
    let mut action = TableAction::None;
    let mut anchors = InteriorAnchors::default();

    // --- Title (ISPF) ----------------------------------------------------
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(title).strong());
    });

    // --- Command ===> primary command line -------------------------------
    let cmd_id = host.command_line_id();
    ui.horizontal(|ui| {
        ui.label("Command ===>");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.command_line)
                .id(cmd_id)
                .desired_width(ui.available_width().max(200.0)),
        );
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let text = std::mem::take(&mut state.command_line);
            action = if text.trim().is_empty() {
                TableAction::ApplyLineCommands
            } else {
                TableAction::CommandLine(text)
            };
        }
    });

    if let Some(err) = error {
        ui.colored_label(egui::Color32::from_rgb(0xD2, 0x0F, 0x39), err);
    }

    // --- Column headings (from the host's declared columns) --------------
    ui.horizontal(|ui| {
        for col in host.columns() {
            heading(ui, col.header, header_width(col.header));
        }
    });

    // --- FILTER row (host-owned filterable cells) ------------------------
    ui.horizontal(|ui| {
        // Spacer under the Act heading (the FILTER row has no Act field).
        ui.add_space(ACT_WIDTH + ui.spacing().item_spacing.x);
        anchors.first = host.render_filter_cells(ui);
        anchors.last = anchors.first;
    });
    ui.separator();

    // --- Data rows inside a scroll area with a right-hand scrollbar -------
    // `act_enter` is set WHEN Enter is pressed inside a row's Act cell (the ISPF
    // in-cell apply): the pending line-command batch must run from there too, not
    // only from the `Command ===>` line (command-configurator Req 7.6).
    let rows_height = ui.available_height().max(64.0);
    let act_enter = egui::ScrollArea::vertical()
        .id_salt("ispf_table_rows")
        .auto_shrink([false, false])
        .max_height(rows_height)
        .show(ui, |ui| {
            let mut act_enter = false;
            if rows.is_empty() {
                // EMPTY-STORE affordance (Req 4.6): always render one empty
                // insertable scratch row at backing index 0 so the first entry
                // can be created; promote it into `rows` on the first edit. An
                // Enter in the scratch row's Act cell applies the batch AFTER the
                // scratch has been promoted into `rows`.
                let mut scratch = blank_scratch();
                let default = blank_scratch();
                act_enter |= host.render_row_cells(ui, 0, &mut scratch);
                if scratch != default {
                    rows.push(scratch);
                }
            } else {
                for &backing in displayed {
                    if let Some(row) = rows.get_mut(backing) {
                        act_enter |= host.render_row_cells(ui, backing, row);
                    }
                }
            }
            act_enter
        })
        .inner;

    // A command-line verb/Enter takes precedence (it already applies line
    // commands first); otherwise an Act-cell Enter applies the pending batch.
    if act_enter && matches!(action, TableAction::None) {
        action = TableAction::ApplyLineCommands;
    }

    (action, anchors)
}

/// The fixed Act-column width shared by the header spacer and the data rows.
pub const ACT_WIDTH: f32 = 28.0;

/// Render a strong, fixed-width column heading.
fn heading(ui: &mut egui::Ui, text: &str, width: f32) {
    ui.add_sized(
        [width, ui.spacing().interact_size.y],
        egui::Label::new(egui::RichText::new(text).strong()),
    );
}

/// A per-header display width mirroring the Alias editor's prior fixed widths so
/// the layout is unchanged (Act 28, CE 84, Command/Root 96, Alias 120,
/// Parameters 260, Description 180). An unknown header falls back to a sensible
/// default.
fn header_width(header: &str) -> f32 {
    match header {
        "Act" => ACT_WIDTH,
        "CE" => 84.0,
        "Command" | "Root" => 96.0,
        "Alias" => 120.0,
        "Parameters" => 260.0,
        "Description" => 180.0,
        _ => 120.0,
    }
}
