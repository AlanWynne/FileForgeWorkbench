//! Shared ISPF Table Editor component (CR-CH-075, `docs/specs/ispf-table-editor/`).
//!
//! A reusable, egui-free CORE for the Alias / Menu / Kinds editor Contexts: the
//! generic column model + host contract ([`state`]), the deterministic
//! line-command apply pass over a generic `Vec<Row>` ([`line_commands`]), the
//! stable Sort ([`sort`]), and the generic egui RENDER surface ([`render`]) the
//! host editors delegate their presentation to.
//!
//! The pure core REUSES the GUI-free `ff-line-commands` engine for line-command
//! parsing and classification (no new parser) and introduces NO new dispatcher,
//! navigation stack, save path, or on-disk format -- it operates purely on the
//! host's in-memory working-copy row list.

pub(crate) mod arrow_nav;
pub(crate) mod block_apply;
pub(crate) mod line_commands;
pub(crate) mod render;
pub(crate) mod sort;
pub(crate) mod state;

#[allow(unused_imports)]
pub(crate) use arrow_nav::{apply_arrow_walk, arrow_target, ArrowDir};

#[allow(unused_imports)]
pub(crate) use line_commands::{apply_line_commands, ApplyResult};
#[allow(unused_imports)]
pub(crate) use render::{
    render_ispf_table, InteriorAnchors, IspfTableRender, RowCells, TableAction, TableRenderParams,
};
#[allow(unused_imports)]
pub(crate) use sort::{apply_sort, parse_sort_command, SortResult};
#[allow(unused_imports)]
pub(crate) use state::{
    ColumnId, ColumnSpec, CompareMode, FilterState, IspfTableHost, LineCommandSet, SortDirection,
    SortRequest, TableEditorState,
};
