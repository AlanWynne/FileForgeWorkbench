//! Shared ISPF Table Editor -- the GENERIC, egui-free contract and state
//! (CR-CH-075, ispf-table-editor Requirement 1, 3, 4, 5).
//!
//! This module owns the pure data types the shared table editor is built on:
//!   - [`ColumnId`] / [`CompareMode`] / [`ColumnSpec`] -- the host's declared
//!     column model (header order, filterable + comparison, sortable).
//!   - [`LineCommandSet`] -- the subset of Act letters a host enables
//!     (I / D / C / M + Sort), reported to the apply pass.
//!   - [`FilterState`] -- the per-filterable-column filter inputs (display-only).
//!   - [`TableEditorState`] -- the transient `Command ===>` + filter state the
//!     host embeds in its panel state (never persisted).
//!   - [`IspfTableHost`] -- the trait a host editor implements so the pure apply
//!     pass ([`super::line_commands`]) and Sort ([`super::sort`]) can structure
//!     its backing `Vec<Row>` WITHOUT knowing the row's field semantics.
//!
//! Nothing here depends on egui; the render surface is a separate concern added
//! by FEAT-002. Keeping this core pure makes the line-command apply pass, Sort,
//! and filter fully unit-testable without a GUI.

// A few helpers in this shared core (e.g. `LineCommandSet::enabled_letters`, the
// `ContainsIgnoreCase` compare mode) are declared for the full column model but
// not yet exercised by the sole consumer (the Alias host); they are retained for
// the Menu / Kinds hosts that follow. Keep the module-level allow until a second
// host consumes them.
#![allow(dead_code)]

/// A stable column identifier declared by the host editor. The host uses its own
/// `&'static str` ids (e.g. `"surface"`, `"canonical"`); the shared component
/// treats them opaquely for header order, filter mapping, and the `SORT <column>`
/// argument match (case-insensitive).
pub type ColumnId = &'static str;

/// How a filterable column's value is compared against the filter input
/// (ispf-table-editor Requirement 3.3). The host declares one per filterable
/// column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareMode {
    /// The value must equal the filter input, case-insensitively.
    EqualsIgnoreCase,
    /// The value must contain the filter input, case-insensitively.
    ContainsIgnoreCase,
}

/// One declared column of a table editor (ispf-table-editor Requirement 1.3,
/// 3.1, 5.1). The ORDER of a host's `&[ColumnSpec]` is the authoritative header
/// order (and the snapshot column order, Requirement 7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnSpec {
    /// The stable column id (opaque to the component).
    pub id: ColumnId,
    /// The human-readable header label.
    pub header: &'static str,
    /// Whether this column is a filter input in the FILTER row.
    pub filterable: bool,
    /// How a filterable column compares; ignored when `filterable` is false.
    pub compare_mode: CompareMode,
    /// Whether this column is a valid `SORT <column>` key.
    pub sortable: bool,
}

impl ColumnSpec {
    /// A non-filterable, non-sortable display column.
    pub const fn display(id: ColumnId, header: &'static str) -> Self {
        Self {
            id,
            header,
            filterable: false,
            compare_mode: CompareMode::EqualsIgnoreCase,
            sortable: false,
        }
    }

    /// A filterable column with the given comparison mode.
    pub const fn filterable(id: ColumnId, header: &'static str, compare_mode: CompareMode) -> Self {
        Self {
            id,
            header,
            filterable: true,
            compare_mode,
            sortable: false,
        }
    }

    /// Return a copy of this spec marked sortable.
    pub const fn with_sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
}

/// The direction of a `SORT` (ispf-table-editor Requirement 5.1), ascending by
/// default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Ascending (the default).
    #[default]
    Ascending,
    /// Descending.
    Descending,
}

/// A parsed `SORT <column> [ASC|DESC]` request (ispf-table-editor Requirement
/// 5.1). `column == None` means "the host's first sortable column".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortRequest {
    /// The requested key column id (matched case-insensitively), or `None` for
    /// the first sortable column.
    pub column: Option<String>,
    /// Ascending or descending.
    pub direction: SortDirection,
}

/// The subset of line commands a host editor enables (ispf-table-editor
/// Requirement 4, 5; command-configurator Req 7.4). An Act letter whose command
/// is not enabled is cleared and ignored by the apply pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCommandSet {
    /// `I` -- insert a blank row after.
    pub insert: bool,
    /// `D` -- delete the row.
    pub delete: bool,
    /// `R` -- repeat (clone the row after it); a single-row Copy alias
    /// (command-configurator Req 7.6).
    pub repeat: bool,
    /// `C` (+ `A`/`B`) -- copy the row to a target.
    pub copy: bool,
    /// `M` (+ `A`/`B`) -- move the row to a target.
    pub move_: bool,
    /// `SORT <column>` -- reorder the backing list by a sortable column.
    pub sort: bool,
    /// Whether the DOUBLED block forms (`RR` / `DD` / `CC` / `MM`) are enabled
    /// (ispf-table-editor Requirement 9.5, CR-CH-076). Blocks are OPT-IN per host:
    /// a host that leaves this `false` continues to clear and ignore a doubled
    /// marker with a non-blocking notice (Requirement 4.5). `MM` additionally
    /// depends on `move_`, so a host with `blocks = true, move_ = false` (the
    /// Alias set) enables RR / DD / CC but not MM (Requirement 7.12).
    pub blocks: bool,
}

impl LineCommandSet {
    /// The Alias Editor set: I / D / R / C / SORT; Move is NOT enabled because an
    /// alias set is not an inherently ordered list (command-configurator Req 7.4).
    /// `R` is retained as a clone-after alias of a single-row Copy (Req 7.6).
    pub const fn alias() -> Self {
        Self {
            insert: true,
            delete: true,
            repeat: true,
            copy: true,
            move_: false,
            sort: true,
            blocks: true,
        }
    }

    /// The enabled single-letter Act commands, for a header hint or a notice.
    /// Returns the uppercase letters in a stable order (I, D, R, C, M, then S for
    /// SORT when any surface advertises it).
    pub fn enabled_letters(&self) -> Vec<char> {
        let mut out = Vec::new();
        if self.insert {
            out.push('I');
        }
        if self.delete {
            out.push('D');
        }
        if self.repeat {
            out.push('R');
        }
        if self.copy {
            out.push('C');
        }
        if self.move_ {
            out.push('M');
        }
        if self.sort {
            out.push('S');
        }
        out
    }
}

/// The per-filterable-column filter inputs (ispf-table-editor Requirement 3).
/// Each entry is `(column_id, input)`; an absent or empty input means "no filter
/// on this column". Display-only -- never mutates the backing row list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterState {
    inputs: Vec<(ColumnId, String)>,
}

impl FilterState {
    /// An empty filter (shows all rows).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set (or clear, when `value` trims to empty) the filter input for `column`.
    pub fn set(&mut self, column: ColumnId, value: &str) {
        let trimmed = value.trim();
        self.inputs.retain(|(c, _)| *c != column);
        if !trimmed.is_empty() {
            self.inputs.push((column, trimmed.to_string()));
        }
    }

    /// The active (non-empty) filter input for `column`, if any.
    pub fn get(&self, column: ColumnId) -> Option<&str> {
        self.inputs
            .iter()
            .find(|(c, _)| *c == column)
            .map(|(_, v)| v.as_str())
    }

    /// Clear every filter input.
    pub fn clear(&mut self) {
        self.inputs.clear();
    }

    /// Whether no filter input is set (shows all rows).
    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty()
    }
}

/// The transient shared state a host editor embeds in its panel state
/// (ispf-table-editor design "The shared `TableEditorState`"). None of it is
/// persisted; the Act letters live on the host's Row_Model, not here.
#[derive(Debug, Clone, Default)]
pub struct TableEditorState {
    /// The `Command ===>` primary command field text.
    pub command_line: String,
    /// The FILTER row's per-column inputs (display-only).
    pub filter: FilterState,
}

impl TableEditorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }
}

/// The contract a host editor implements so the shared, egui-free apply pass and
/// Sort can structure its backing `Vec<Row>` without knowing the row's field
/// semantics (ispf-table-editor design "The generic contract"). `Row` is the
/// host's working-copy row type.
pub trait IspfTableHost {
    /// The host's working-copy row type.
    type Row;

    /// The declared columns in header (and snapshot) order.
    fn columns(&self) -> &[ColumnSpec];

    /// The enabled line-command set (which Act letters + Sort are live).
    fn line_command_set(&self) -> LineCommandSet;

    /// A fresh blank row for an `I` (Insert).
    fn blank_row(&self) -> Self::Row;

    /// A deep copy of `row` for a `C` (Copy). The copy carries the field values;
    /// the apply pass clears the copy's Act cell.
    fn clone_row(&self, row: &Self::Row) -> Self::Row;

    /// The transient Act cell text of `row` (the typed line-command letter).
    fn act_of<'a>(&self, row: &'a Self::Row) -> &'a str;

    /// Clear `row`'s transient Act cell.
    fn clear_act(&self, row: &mut Self::Row);

    /// Whether `row` matches the active filter (every set input ANDed). Used for
    /// display only; the apply pass keys off backing indices.
    fn filter_matches(&self, row: &Self::Row, filter: &FilterState) -> bool;

    /// The sort key STRING for `row` on the given sortable `column`. Only called
    /// for a column whose `ColumnSpec.sortable` is true.
    fn sort_key(&self, row: &Self::Row, column: ColumnId) -> String;
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: ispf-table-editor Requirement 1.3, 3.1, 5.1 -- a ColumnSpec
    // round-trips its filterable + compare mode + sortable flags.
    #[test]
    fn column_spec_carries_filter_and_sort_flags() {
        let display = ColumnSpec::display("act", "Act");
        assert!(!display.filterable);
        assert!(!display.sortable);

        let ce = ColumnSpec::filterable("ce", "CE", CompareMode::EqualsIgnoreCase);
        assert!(ce.filterable);
        assert_eq!(ce.compare_mode, CompareMode::EqualsIgnoreCase);
        assert!(!ce.sortable);

        let surface = ColumnSpec::filterable("surface", "Alias", CompareMode::ContainsIgnoreCase)
            .with_sortable();
        assert!(surface.filterable);
        assert_eq!(surface.compare_mode, CompareMode::ContainsIgnoreCase);
        assert!(surface.sortable, "with_sortable marks the column sortable");
    }

    // Validates: command-configurator Requirement 7.4, 7.6 -- the Alias
    // line-command set reports I / D / R / C enabled (R is the clone-after Copy
    // alias) and Move disabled, with Sort enabled and advertised as `S`.
    #[test]
    fn alias_line_command_set_reports_enabled_letters() {
        let set = LineCommandSet::alias();
        assert_eq!(set.enabled_letters(), vec!['I', 'D', 'R', 'C', 'S']);
        assert!(set.repeat, "Repeat (R) is enabled for aliases");
        assert!(!set.move_, "Move is not enabled for aliases");
        assert!(set.sort, "Sort is enabled for aliases");
    }

    // Validates: ispf-table-editor Requirement 9.1, 9.5 -- the Alias set opts IN
    // to block line commands (blocks == true) while keeping Move disabled (so MM
    // stays off); a default/non-alias set does NOT enable blocks (blocks ==
    // false), so a host must opt in explicitly.
    #[test]
    fn block_opt_in_flag_is_per_host() {
        let alias = LineCommandSet::alias();
        assert!(alias.blocks, "the Alias set opts in to block line commands");
        assert!(
            !alias.move_,
            "Move stays disabled on the Alias set so MM is off"
        );

        let non_alias = LineCommandSet {
            insert: true,
            delete: true,
            repeat: false,
            copy: false,
            move_: false,
            sort: false,
            blocks: false,
        };
        assert!(
            !non_alias.blocks,
            "a host that does not opt in has blocks == false"
        );
    }

    // Validates: ispf-table-editor Requirement 3 -- FilterState sets, reads, and
    // clears inputs; an empty value clears that column's filter.
    #[test]
    fn filter_state_set_get_clear() {
        let mut filter = FilterState::new();
        assert!(filter.is_empty());
        filter.set("ce", "ffcmd");
        assert_eq!(filter.get("ce"), Some("ffcmd"));
        assert!(!filter.is_empty());
        // Setting an empty value clears that column.
        filter.set("ce", "   ");
        assert_eq!(filter.get("ce"), None);
        assert!(filter.is_empty());
        // Two columns, then clear-all.
        filter.set("ce", "ffedit");
        filter.set("root", "FIND");
        assert_eq!(filter.get("root"), Some("FIND"));
        filter.clear();
        assert!(filter.is_empty());
    }
}
