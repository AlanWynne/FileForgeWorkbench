//! The Alias Editor as a host of the shared ISPF Table Editor component
//! (CR-CH-075, command-configurator Req 7, FEAT-002).
//!
//! `AliasEditorState` implements BOTH shared contracts:
//!   - [`IspfTableHost`] (egui-free): the column model, the enabled line-command
//!     set (I/D/C + SORT, Req 7.4), blank/clone rows for the apply pass, the Act
//!     accessors, the display filter, and the per-column sort key.
//!   - [`IspfTableRender`] (egui-aware): the stable command-line id, the FILTER
//!     row's cascading CE -> Root cells, and the per-row Act / CE / Root / Alias /
//!     Parameters / Description cells, preserving the exact egui ids the
//!     full-shell focus tests depend on.
//!
//! The shared sort key is `user_aliases::alias_sort_key` (uppercased surface asc,
//! canonical tie-break) so a manual `SORT` and the auto-sort-on-save order
//! identically (Req 7.5a).

use super::rows::{render_data_row, render_root_combo};
use super::state::{AliasEditorState, AliasRowEdit};
use crate::shell::environment_registry::EnvironmentRegistry;
use crate::shell::ispf_table_editor::render::IspfTableRender;
use crate::shell::ispf_table_editor::state::{
    ColumnId, ColumnSpec, CompareMode, FilterState, IspfTableHost, LineCommandSet,
};
use crate::shell::user_aliases::{alias_sort_key, AliasEnvironment};

/// The alias column ids (opaque to the shared component; used for header order,
/// the sort-key column match, and the snapshot column order, Req 7.2).
pub(super) const COL_ACT: ColumnId = "act";
pub(super) const COL_CE: ColumnId = "ce";
pub(super) const COL_COMMAND: ColumnId = "canonical";
pub(super) const COL_ALIAS: ColumnId = "surface";
pub(super) const COL_PARAMETERS: ColumnId = "command_template";
pub(super) const COL_DESCRIPTION: ColumnId = "description";

/// The ALIAS (surface) column id -- the default sort key column for a bare
/// `SORT` command (Req 7.5a), exposed so the shell's SORT handler can default to
/// it instead of the generic first-sortable fallback (which would pick Command).
pub fn alias_surface_column() -> ColumnId {
    COL_ALIAS
}

/// The alias column model in Req 7.2 order: Act, CE, Command (canonical), Alias
/// (surface), Parameters (template), Description. Alias + Command are sortable;
/// CE + Command are the filterable columns (filter CE + Root, Req 5.15 / 7.8).
const ALIAS_COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec::display(COL_ACT, "Act"),
    ColumnSpec::filterable(COL_CE, "CE", CompareMode::EqualsIgnoreCase),
    ColumnSpec::filterable(COL_COMMAND, "Command", CompareMode::EqualsIgnoreCase).with_sortable(),
    ColumnSpec::display(COL_ALIAS, "Alias").with_sortable(),
    ColumnSpec::display(COL_PARAMETERS, "Parameters"),
    ColumnSpec::display(COL_DESCRIPTION, "Description"),
];

impl IspfTableHost for AliasEditorState {
    type Row = AliasRowEdit;

    fn columns(&self) -> &[ColumnSpec] {
        &ALIAS_COLUMNS
    }

    fn line_command_set(&self) -> LineCommandSet {
        LineCommandSet::alias()
    }

    fn blank_row(&self) -> AliasRowEdit {
        AliasRowEdit::default()
    }

    fn clone_row(&self, row: &AliasRowEdit) -> AliasRowEdit {
        let mut clone = row.clone();
        clone.act.clear();
        clone
    }

    fn act_of<'a>(&self, row: &'a AliasRowEdit) -> &'a str {
        &row.act
    }

    fn clear_act(&self, row: &mut AliasRowEdit) {
        row.act.clear();
    }

    fn filter_matches(&self, row: &AliasRowEdit, filter: &FilterState) -> bool {
        // CE + Root only (never Alias / Parameters / Description), ANDed. Mirrors
        // `filter::displayed_indices` so the two filter views agree.
        let ce_ok = match filter.get(COL_CE) {
            Some(v) => row.environment.label().eq_ignore_ascii_case(v),
            None => true,
        };
        let root_ok = match filter.get(COL_COMMAND) {
            Some(v) => row.canonical.trim().eq_ignore_ascii_case(v.trim()),
            None => true,
        };
        ce_ok && root_ok
    }

    fn sort_key(&self, row: &AliasRowEdit, column: ColumnId) -> String {
        // The canonical alias ordering keys on the uppercased SURFACE with the
        // uppercased canonical as the tie-break (Req 7.5). Sorting by the Command
        // column orders on canonical first; both reuse `alias_sort_key` so the
        // ordering is one source of truth.
        let (surface, canonical) = alias_sort_key(&row.surface, &row.canonical);
        match column {
            COL_COMMAND => format!("{canonical}\u{0}{surface}"),
            // COL_ALIAS (the default) and any other sortable column key on
            // surface-then-canonical.
            _ => format!("{surface}\u{0}{canonical}"),
        }
    }
}

impl IspfTableRender for AliasEditorState {
    fn command_line_id(&self) -> egui::Id {
        egui::Id::new("alias_editor_command_line")
    }

    fn render_filter_cells(&mut self, ui: &mut egui::Ui) -> Option<egui::Id> {
        // Filter CE dropdown -- the FIRST interior control (Req 5.23 / 7.8). Its
        // id is captured FRESH each frame from the combo response.
        let shown = match self.filter_ce {
            Some(env) => env.label(),
            None => "--",
        };
        let mut picked: Option<Option<AliasEnvironment>> = None;
        let combo = egui::ComboBox::from_id_salt("alias_filter_ce")
            .selected_text(shown)
            .width(84.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(self.filter_ce.is_none(), "--")
                    .clicked()
                {
                    picked = Some(None);
                }
                for choice in AliasEnvironment::ALL {
                    if ui
                        .selectable_label(self.filter_ce == Some(choice), choice.label())
                        .clicked()
                    {
                        picked = Some(Some(choice));
                    }
                }
            });
        let first = combo.response.id;
        if let Some(sel) = picked {
            self.filter_ce = sel;
            let env_name = sel.unwrap_or_default().env_name();
            if !self.filter_root.is_empty()
                && !EnvironmentRegistry::contains_root(env_name, &self.filter_root)
            {
                self.filter_root.clear();
            }
        }

        // Filter Root dropdown constrained to the chosen CE's roots (Req 5.15).
        let root_env = self.filter_ce.unwrap_or_default();
        render_root_combo(ui, "alias_filter_root", root_env, &mut self.filter_root);

        Some(first)
    }

    fn render_row_cells(&self, ui: &mut egui::Ui, backing: usize, row: &mut AliasRowEdit) {
        render_data_row(ui, backing, row);
    }
}
