//! The Alias Editor as a host of the shared ISPF Table Editor component
//! (CR-CH-075, command-configurator Req 7, FEAT-002).
//!
//! `AliasEditorState` implements BOTH shared contracts:
//!   - [`IspfTableHost`] (egui-free): the column model, the enabled line-command
//!     set (I/D/C + SORT, Req 7.4), blank/clone rows for the apply pass, the Act
//!     accessors, the display filter, and the per-column sort key.
//!   - [`IspfTableRender`] (egui-aware): the FILTER row's cascading CE -> Root
//!     cells and the per-row Act / CE / Root / Alias / Parameters / Description
//!     cells, preserving the exact egui ids the full-shell focus tests depend on.
//!     CR-CH-077 single-chrome: the host no longer owns a `Command ===>` line --
//!     the shell's app-level command field is the one command line.
//!
//! The shared sort key is `user_aliases::alias_sort_key` (uppercased surface asc,
//! canonical tie-break) so a manual `SORT` and the auto-sort-on-save order
//! identically (Req 7.5a).

use super::rows::{render_data_row, render_root_combo};
use super::state::{AliasEditorState, AliasRowEdit};
use crate::shell::environment_registry::EnvironmentRegistry;
use crate::shell::ispf_table_editor::render::IspfTableRender;
use crate::shell::ispf_table_editor::snapshot::sanitise_cell;
use crate::shell::ispf_table_editor::state::{
    CellKind, ColumnId, ColumnSpec, CompareMode, FilterState, IspfTableHost, LineCommandSet,
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
///
/// CR-NR-116 (command-configurator Req 7.14-7.28, ispf-table-editor Req 12.1):
/// each column also carries additive SNAPSHOT metadata (cell kind + field
/// width) so a captured Alias screen looks like the live grid. CE and Command
/// are COMBO cells (`[value v]`); the rest are TEXT. Widths account for the
/// `[ v]` wrapper (+4) on a combo plus the longest value.
pub(crate) const ALIAS_COLUMNS: [ColumnSpec; 6] = [
    ColumnSpec::display(COL_ACT, "Act").with_snapshot(CellKind::Text, 4),
    ColumnSpec::filterable(COL_CE, "CE", CompareMode::EqualsIgnoreCase)
        .with_snapshot(CellKind::Combo, 10),
    ColumnSpec::filterable(COL_COMMAND, "Command", CompareMode::EqualsIgnoreCase)
        .with_sortable()
        .with_snapshot(CellKind::Combo, 20),
    ColumnSpec::display(COL_ALIAS, "Alias")
        .with_sortable()
        .with_snapshot(CellKind::Text, 16),
    ColumnSpec::display(COL_PARAMETERS, "Parameters").with_snapshot(CellKind::Text, 28),
    ColumnSpec::display(COL_DESCRIPTION, "Description").with_snapshot(CellKind::Text, 32),
];

/// The six RAW per-column snapshot values for `row`, in [`ALIAS_COLUMNS`] order
/// (Act, CE, Command, Alias, Parameters, Description), for the shared
/// `snapshot_table` / `raw_table` builders (command-configurator Req 7.15,
/// ispf-table-editor Req 12.3).
///
/// These are the row's REAL field values, never the live widget HINT TEXT: an
/// empty Parameters field is `""` (the renderer draws the `_` run / `[ v]`
/// placeholder), not `(&1)`; an empty Description is `""`, not `note`; an empty
/// Alias is `""`, not `name`. A typed `&1` IS a real value and appears. The CE
/// is the canonical UPPERCASE `env_name()` (FFCMD / FFEDIT), matching the
/// uppercased live combo (CR-NR-116). Control characters are flattened to a
/// space (via [`sanitise_cell`]) so a cell with a tab / newline, or a
/// Description containing ` | `, stays one discrete value.
pub(crate) fn alias_snapshot_cells(row: &AliasRowEdit) -> Vec<String> {
    vec![
        sanitise_cell(&row.act),
        row.environment.env_name().to_string(),
        sanitise_cell(&row.canonical),
        sanitise_cell(&row.surface),
        sanitise_cell(&row.command_template),
        sanitise_cell(&row.description),
    ]
}

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
    fn render_filter_cells(&mut self, ui: &mut egui::Ui) -> Option<egui::Id> {
        // Filter CE dropdown -- the FIRST interior control (Req 5.23 / 7.8). Its
        // id is captured FRESH each frame from the combo response. CR-NR-116: the
        // shown text is the canonical UPPERCASE `env_name()` (FFCMD / FFEDIT); the
        // match below keeps `eq_ignore_ascii_case`, so only the display changed.
        let shown = match self.filter_ce {
            Some(env) => env.env_name(),
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
                        .selectable_label(self.filter_ce == Some(choice), choice.env_name())
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

    fn render_row_cells(
        &self,
        ui: &mut egui::Ui,
        backing: usize,
        row: &mut AliasRowEdit,
    ) -> crate::shell::ispf_table_editor::render::RowCells {
        // CR-NR-119 (Req 9.1): the DISPLAY-TIME inherited one-liner for this
        // row's canonical root, used as the Description cell HINT when the row's
        // own Description is empty. Keyed by the UPPERCASED canonical; empty when
        // the root has no cmd:<ROOT> topic (Req 9.4).
        let inherited = self
            .inherited_descriptions
            .get(&row.canonical.trim().to_uppercase())
            .map(String::as_str)
            .unwrap_or_default();
        render_data_row(ui, backing, row, inherited)
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    // Validates: command-configurator Requirement 7.14, ispf-table-editor
    // Requirement 12.1 -- ALIAS_COLUMNS declares the six columns in Req 7.2 order
    // with the correct cell kinds (CE + Command are combos, the rest text) and
    // the declared snapshot widths.
    #[test]
    fn alias_columns_declare_six_columns_with_cell_kinds_and_widths() {
        assert_eq!(ALIAS_COLUMNS.len(), 6, "six alias columns");

        let kinds_widths: Vec<(ColumnId, CellKind, usize)> = ALIAS_COLUMNS
            .iter()
            .map(|c| (c.id, c.cell, c.snapshot_width))
            .collect();
        assert_eq!(
            kinds_widths,
            vec![
                (COL_ACT, CellKind::Text, 4),
                (COL_CE, CellKind::Combo, 10),
                (COL_COMMAND, CellKind::Combo, 20),
                (COL_ALIAS, CellKind::Text, 16),
                (COL_PARAMETERS, CellKind::Text, 28),
                (COL_DESCRIPTION, CellKind::Text, 32),
            ],
            "column order, cell kind and width per Req 7.14"
        );
    }

    // Validates: command-configurator Requirement 7.15 -- alias_snapshot_cells
    // returns the six RAW values with the CE uppercased via env_name(), a typed
    // &1 preserved, and empty fields left empty (never the live hint text).
    #[test]
    fn alias_snapshot_cells_returns_six_raw_values_ce_uppercase() {
        let row = AliasRowEdit {
            act: String::new(),
            surface: "SS".to_string(),
            canonical: "SNAPSHOT".to_string(),
            environment: AliasEnvironment::Ffcmd,
            command_template: String::new(),
            description: String::new(),
        };
        let cells = alias_snapshot_cells(&row);
        assert_eq!(
            cells,
            vec![
                "".to_string(),         // Act (empty, never the '_' hint)
                "FFCMD".to_string(),    // CE via env_name() UPPERCASE
                "SNAPSHOT".to_string(), // Command (canonical)
                "SS".to_string(),       // Alias (surface)
                "".to_string(),         // Parameters (empty, never the '(&1)' hint)
                "".to_string(),         // Description (empty, never the 'note' hint)
            ]
        );
    }

    // Validates: command-configurator Requirement 7.15 -- a typed &1 parameter
    // IS a real value and appears; the FFEDIT environment uppercases to FFEDIT.
    #[test]
    fn alias_snapshot_cells_preserves_typed_param_and_ffedit() {
        let row = AliasRowEdit {
            act: "I".to_string(),
            surface: "ECOB".to_string(),
            canonical: "EDIT".to_string(),
            environment: AliasEnvironment::Ffedit,
            command_template: "&1".to_string(),
            description: "Edit a COBOL member".to_string(),
        };
        let cells = alias_snapshot_cells(&row);
        assert_eq!(cells[1], "FFEDIT", "CE uppercased");
        assert_eq!(cells[4], "&1", "a typed &1 is a real value and appears");
        assert_eq!(cells[0], "I");
        assert_eq!(cells[5], "Edit a COBOL member");
    }

    // Validates: command-configurator Requirement 7.15, ispf-table-editor
    // Requirement 12.2 -- control characters in a cell are flattened to a space,
    // and a Description containing ' | ' stays a SINGLE discrete value.
    #[test]
    fn alias_snapshot_cells_sanitises_and_keeps_pipe_in_one_cell() {
        let row = AliasRowEdit {
            command_template: "a\tb\nc".to_string(),
            description: "left | right".to_string(),
            ..Default::default()
        };
        let cells = alias_snapshot_cells(&row);
        assert_eq!(cells[4], "a b c", "control chars become a single space");
        assert_eq!(
            cells[5], "left | right",
            "an in-cell ' | ' stays one discrete value"
        );
    }
}
