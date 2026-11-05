//! Shared ISPF Table Editor -- the generic egui RENDER surface (CR-CH-075,
//! ispf-table-editor Requirement 4.6, 5, FEAT-002).
//!
//! [`render_ispf_table`] draws the common ISPF table body ONCE for every table
//! editor Context: a column header row from the host's
//! [`super::state::ColumnSpec`] list, a FILTER row, the scrolling data rows
//! (each with an Act cell plus the host's per-column cells in visual Tab order),
//! and an ALWAYS-PRESENT trailing scratch row after the last displayed data row
//! that promotes into the backing list on first edit (Req 12.5, revising 4.6).
//! The egui-free apply pass ([`super::line_commands`]) + Sort ([`super::sort`])
//! stay the structural core; this surface only paints and collects the typed
//! intent.
//!
//! CR-CH-077 single-chrome: the surface draws NEITHER a Title heading NOR its own
//! `Command ===>` line. The shell's app-level Title_Line is the one title and
//! the shell's app-level command field is the one `Command ===>` line, which the
//! shell routes into the host's command-line handler for a table-editor Context
//! (Req 10.1, 10.2). The Act-cell in-cell Enter remains the surface's own apply
//! trigger (`RowCells::act_enter`).
//!
//! Cell painting is DELEGATED to the host via [`IspfTableRender`] (a SEPARATE,
//! egui-aware trait) so the pure [`super::state::IspfTableHost`] contract stays
//! GUI-free. A host implements both: `IspfTableHost` for structure/sort/filter,
//! `IspfTableRender` for the per-row and filter-row egui cells it owns (e.g. the
//! Alias editor's cascading CE -> Root dropdowns).

use super::arrow_nav::apply_arrow_walk;
use super::state::{IspfTableHost, TableEditorState};

/// What a host's per-row render reports back to the surface: whether Enter was
/// pressed in that row's Act cell (the ISPF in-cell apply) and the row's
/// focusable cell ids in VISUAL (Tab) order (Act first, then the host's columns).
/// The surface assembles these into the field matrix the arrow walk navigates
/// (CR-CH-078, Requirement 11.3). Returning the FRESH same-frame ids -- rather
/// than guessing a combo id -- is the B056 lesson: a guessed id does not
/// round-trip through egui focus.
#[derive(Debug, Clone, Default)]
pub struct RowCells {
    /// `true` WHEN Enter was pressed with this row's Act field focused.
    pub act_enter: bool,
    /// The row's focusable cell ids, left-to-right in visual order.
    pub cell_ids: Vec<egui::Id>,
}

/// The structural intent the render surface collected this frame (ispf-table
/// -editor Requirement 4, 5). The host's shell applies it after the owned-panel
/// swap, exactly like the Alias editor's existing `AliasEditorAction`.
///
/// CR-CH-077 single-chrome: the surface no longer owns a `Command ===>` line, so
/// the only structural intent it still collects is the in-Act-cell Enter
/// (`ApplyLineCommands`). A typed `Command ===>` VERB now arrives via the shell's
/// app-level command field, which the shell routes into the host's command-line
/// handler directly (see `run_command_line` -> `apply_alias_command_line`), not
/// as a `TableAction` -- so there is no `CommandLine` variant here any more.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TableAction {
    /// Nothing to apply this frame.
    #[default]
    None,
    /// An in-Act-cell Enter: apply the pending Act line commands only.
    ApplyLineCommands,
}

/// The egui-aware render contract a host implements ALONGSIDE
/// [`super::state::IspfTableHost`]. It owns ONLY the per-column cell painting the
/// generic surface cannot know (dropdowns, text fields, the host's stable ids);
/// the chrome (command line, headers, scroll, scratch row) is the surface's.
pub trait IspfTableRender: IspfTableHost {
    /// Render the FILTER row's interactive cells (the host's filterable columns),
    /// returning the egui [`egui::Id`] of the FIRST focusable filter control so
    /// the surface can report it as the interior focus anchor. The row's leading
    /// Act-column spacer is drawn by the surface.
    fn render_filter_cells(&mut self, ui: &mut egui::Ui) -> Option<egui::Id>;

    /// Render one data row's Act cell + per-column cells for the row at BACKING
    /// index `backing`, binding edits directly to `row`. Returns a [`RowCells`]
    /// carrying whether Enter was pressed with that row's Act field focused (so
    /// the surface can apply the pending line-command batch from the Act cell
    /// itself -- the ISPF in-cell Enter, command-configurator Req 7.6) AND the
    /// row's focusable cell ids in visual order, captured FRESH this frame, so
    /// the surface can assemble the arrow-navigation field matrix (Req 11.3).
    fn render_row_cells(&self, ui: &mut egui::Ui, backing: usize, row: &mut Self::Row) -> RowCells;
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
/// `displayed`: the optional inline error and the blank-row factory the
/// empty-store scratch row is built from.
///
/// CR-CH-077 single-chrome: there is NO `title` field -- the surface no longer
/// draws a centred heading; the shell's app-level Title_Line is the one title.
pub struct TableRenderParams<'a, F> {
    /// The optional inline error shown above the header.
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
/// TRAILING scratch row after the displayed rows and promotes the scratch into
/// `rows` the moment it is edited -- Req 12.5, revising 4.6). `displayed` is the
/// pre-filtered list of BACKING
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
        error,
        blank_scratch,
    } = params;
    let mut action = TableAction::None;
    let mut anchors = InteriorAnchors::default();

    // CR-CH-077 single-chrome: the surface draws NEITHER a centred Title heading
    // NOR its own `Command ===>` line. The shell's app-level Title_Line (the
    // Context / Kind descriptive title) is the ONE title, and the shell's
    // app-level command field is the ONE `Command ===>` line, which the shell
    // routes into the host's command-line handler for a table-editor Context
    // (see `run_command_line`). This REMOVES the former duplicate chrome the host
    // drew on top of the shell's; it adds no new chrome mechanism and changes no
    // dispatch / navigation / save seam (Requirement 10.1, 10.2, 10.4). The same
    // single-chrome applies to every future host (Menu / Kinds) automatically.
    let _ = state;

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
    // The arrow-navigation field matrix (row-major, visual Tab order) assembled
    // from the FRESH same-frame cell ids each row reports (Req 11.3).
    let mut field_matrix: Vec<Vec<egui::Id>> = Vec::new();
    let act_enter = egui::ScrollArea::vertical()
        .id_salt("ispf_table_rows")
        .auto_shrink([false, false])
        .max_height(rows_height)
        .show(ui, |ui| {
            let mut act_enter = false;
            // Render the displayed data rows (the empty-list case renders none,
            // behaving exactly as before -- only the scratch row below shows).
            for &backing in displayed {
                if let Some(row) = rows.get_mut(backing) {
                    let cells = host.render_row_cells(ui, backing, row);
                    act_enter |= cells.act_enter;
                    field_matrix.push(cells.cell_ids);
                }
            }

            // ALWAYS-PRESENT TRAILING SCRATCH ROW (Req 12.5, revising Req 4.6):
            // after the last displayed data row, render one empty insertable
            // scratch row built from `blank_scratch()` so the user can always add
            // a row (the first row when the list is empty, a next row otherwise).
            // Its backing index is `rows.len()` -- the slot it will occupy once
            // promoted -- which is 0 for the empty-list case, keeping that
            // behaviour identical. Promote it into `rows` on the first edit; a
            // fresh scratch row then appears next frame. An untouched scratch row
            // is never pushed, so it is never persisted. Its cell ids are pushed
            // LAST into `field_matrix`, so it is the last focus stop; the filter
            // row stays `anchors.first` (first_interior_id is unchanged).
            let scratch_index = rows.len();
            let mut scratch = blank_scratch();
            let default = blank_scratch();
            let cells = host.render_row_cells(ui, scratch_index, &mut scratch);
            act_enter |= cells.act_enter;
            field_matrix.push(cells.cell_ids);
            if scratch != default {
                rows.push(scratch);
            }
            act_enter
        })
        .inner;

    // --- Arrow-key field-matrix navigation (CR-CH-078, Req 11) -----------
    // The component OWNS the arrow walk: CONSUME any arrow press BEFORE egui
    // traverses (the render_menu_bar / render_chrome B056-B059 lesson) and move
    // focus to the clamped adjacent cell. `apply_arrow_walk` SELF-GUARDS: it only
    // acts (and only consumes the arrows) when a MATRIX cell holds focus, so when
    // the shell's app-level command field holds focus the CR-NR-096 Up/Down
    // history gesture is preserved (Req 11.5). The surface no longer owns a
    // command line (CR-CH-077), so there is no in-surface id to exclude here.
    let focused = ui.ctx().memory(|m| m.focused());
    apply_arrow_walk(ui, &field_matrix, focused);

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

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ispf_table_editor::state::{
        ColumnSpec, CompareMode, FilterState, LineCommandSet,
    };
    use egui_kittest::Harness;

    /// A minimal working-copy row: a value plus a transient Act cell.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    struct TestRow {
        value: String,
        act: String,
    }

    /// A minimal host implementing both the pure `IspfTableHost` and the
    /// egui-aware `IspfTableRender`. `inject_at` simulates a user edit landing in
    /// the row at that BACKING index: when the surface renders that row the host
    /// writes a non-default value into it, so the surface's promote-on-edit path
    /// (`scratch != default`) runs exactly as a real keystroke would drive it.
    struct TestHost {
        columns: Vec<ColumnSpec>,
        inject_at: Option<usize>,
    }

    impl TestHost {
        fn new() -> Self {
            Self {
                columns: vec![
                    ColumnSpec::display("act", "Act"),
                    ColumnSpec::filterable("value", "Value", CompareMode::ContainsIgnoreCase),
                ],
                inject_at: None,
            }
        }
    }

    impl IspfTableHost for TestHost {
        type Row = TestRow;
        fn columns(&self) -> &[ColumnSpec] {
            &self.columns
        }
        fn line_command_set(&self) -> LineCommandSet {
            LineCommandSet::alias()
        }
        fn blank_row(&self) -> TestRow {
            TestRow::default()
        }
        fn clone_row(&self, row: &TestRow) -> TestRow {
            row.clone()
        }
        fn act_of<'a>(&self, row: &'a TestRow) -> &'a str {
            &row.act
        }
        fn clear_act(&self, row: &mut TestRow) {
            row.act.clear();
        }
        fn filter_matches(&self, row: &TestRow, filter: &FilterState) -> bool {
            match filter.get("value") {
                Some(v) => row.value.to_uppercase().contains(&v.to_uppercase()),
                None => true,
            }
        }
        fn sort_key(&self, row: &TestRow, _column: super::super::state::ColumnId) -> String {
            row.value.to_uppercase()
        }
    }

    impl IspfTableRender for TestHost {
        fn render_filter_cells(&mut self, ui: &mut egui::Ui) -> Option<egui::Id> {
            let id = egui::Id::new("test_filter_value");
            let mut buf = String::new();
            let resp = ui.add(egui::TextEdit::singleline(&mut buf).id(id));
            Some(resp.id)
        }

        fn render_row_cells(
            &self,
            ui: &mut egui::Ui,
            backing: usize,
            row: &mut Self::Row,
        ) -> RowCells {
            // Simulate a user edit landing in the row at `inject_at` (used to
            // drive the scratch-row promotion deterministically through the real
            // surface code path).
            if self.inject_at == Some(backing) {
                row.value = "typed".to_string();
            }
            let act_id = egui::Id::new(("test_act", backing));
            let val_id = egui::Id::new(("test_val", backing));
            let act_resp = ui.add(egui::TextEdit::singleline(&mut row.act).id(act_id));
            ui.add(egui::TextEdit::singleline(&mut row.value).id(val_id));
            RowCells {
                act_enter: false,
                cell_ids: vec![act_resp.id, val_id],
            }
        }
    }

    fn run_once(host: &mut TestHost, rows: &mut Vec<TestRow>) -> InteriorAnchors {
        use std::cell::Cell;
        use std::rc::Rc;

        let displayed: Vec<usize> = (0..rows.len()).collect();
        // The anchors reported by the surface, captured through a shared Cell so
        // the closure's borrow ends when the harness is dropped (before we read).
        let captured: Rc<Cell<InteriorAnchors>> = Rc::new(Cell::new(InteriorAnchors::default()));
        let sink = Rc::clone(&captured);
        let mut state = TableEditorState::default();
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .build(|ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let (_action, anchors) = render_ispf_table(
                        ui,
                        host,
                        &mut state,
                        rows,
                        &displayed,
                        TableRenderParams {
                            error: None,
                            blank_scratch: TestRow::default,
                        },
                    );
                    sink.set(anchors);
                });
            });
        harness.run();
        drop(harness);
        captured.get()
    }

    // Validates: ispf-table-editor Requirement 12.5 -- a NON-EMPTY list renders a
    // trailing scratch row after the last data row; typing in it promotes it into
    // the backing list and a fresh scratch row appears, while an untouched
    // scratch row is never persisted.
    #[test]
    fn non_empty_list_has_trailing_scratch_that_promotes_on_edit() {
        let mut host = TestHost::new();
        let mut rows = vec![TestRow {
            value: "existing".to_string(),
            act: String::new(),
        }];

        // Frame 1: no edit -- the untouched trailing scratch row is NOT persisted.
        let _ = run_once(&mut host, &mut rows);
        assert_eq!(
            rows.len(),
            1,
            "an untouched trailing scratch row is never persisted"
        );

        // Frame 2: simulate a user edit in the scratch row (backing index 1, the
        // slot after the one existing row). It must be promoted into `rows`.
        host.inject_at = Some(1);
        let _ = run_once(&mut host, &mut rows);
        assert_eq!(rows.len(), 2, "editing the scratch row promotes it");
        assert_eq!(rows[1].value, "typed", "the promoted row carries the edit");

        // Frame 3: no edit again -- the fresh scratch row (now at index 2) is not
        // persisted, so the count holds at 2.
        host.inject_at = None;
        let _ = run_once(&mut host, &mut rows);
        assert_eq!(
            rows.len(),
            2,
            "a fresh untouched scratch row appears but is not persisted"
        );
    }

    // Validates: ispf-table-editor Requirement 12.5, 7.1 -- the trailing scratch
    // row does NOT change the reported first interior focus anchor: it stays the
    // FILTER row's first control.
    #[test]
    fn trailing_scratch_row_does_not_change_first_interior_id() {
        let mut host = TestHost::new();
        let mut rows = vec![TestRow {
            value: "existing".to_string(),
            act: String::new(),
        }];
        let anchors = run_once(&mut host, &mut rows);
        assert_eq!(
            anchors.first,
            Some(egui::Id::new("test_filter_value")),
            "first_interior_id stays the filter row's first control"
        );
    }

    // Validates: ispf-table-editor Requirement 12.5 -- when the list is EMPTY the
    // behaviour is identical to the old Req 4.6: a single scratch row shows and
    // an untouched one is not persisted.
    #[test]
    fn empty_list_behaviour_is_unchanged() {
        let mut host = TestHost::new();
        let mut rows: Vec<TestRow> = Vec::new();
        let _ = run_once(&mut host, &mut rows);
        assert!(
            rows.is_empty(),
            "an untouched empty-store scratch is not kept"
        );

        host.inject_at = Some(0);
        let _ = run_once(&mut host, &mut rows);
        assert_eq!(rows.len(), 1, "editing the empty-store scratch promotes it");
        assert_eq!(rows[0].value, "typed");
    }
}
