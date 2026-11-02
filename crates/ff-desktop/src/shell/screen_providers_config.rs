//! # Config Panel ScreenProvider -- faithful settings-tree capture
//!
//! Split out of `shell/screen_providers.rs` for the 400-line rule (behaviour
//! unchanged). This holds the Config Panel's `ScreenModel` builder, which is a
//! FAITHFUL capture of the on-screen settings tree (config-snapshot-bugs.md
//! Config Bug 2, master-screen-directive Req 4.4a, screen-snapshot-scrm Req
//! 27.8): the active filter/namespace scope, the dirty flag, AND the visible
//! tree rows -- each namespace group and, for an expanded group, its keys with
//! their current effective values.
//!
//! The capture MIRRORS the live egui render EXACTLY (config-snapshot-bugs.md
//! Config Bug 3): it emits pre-composed `body_lines` built from the SAME pure
//! layout helpers the render uses (`config_panel::layout`), so the captured text
//! is a single representation of the visible tree -- each row ONCE, flush-left
//! `[Group] : expanded|collapsed`, each child indented one level as `key : value`,
//! a single GLOBAL aligned `:` column, NO `. . .` dot-leaders, and NO duplicate
//! `Settings` / `Setting / Value / State` table block. Built from the panel's
//! live state + the config handle, never from egui widgets (Req 2.2).
//!
//! Validates: screen-snapshot-scrm Requirement 27.8, 27.9; master-screen-directive
//! Requirement 4.4a, 4.4b.

use ff_screen_model::ScreenModel;

use super::screen_providers::active_display_title;
use super::WorkbenchShell;
use crate::config_panel::{
    aligned_left_segment, child_left_label, global_key_column_width, group_left_label,
    visible_rows, ConfigNodeId, TreeEntry,
};

/// The caption of the (non-interactive) Unsaved-edits header row. Mirrors the
/// render's `UNSAVED_LABEL` so the capture header text equals the on-screen row.
const UNSAVED_LABEL: &str = "Unsaved edits";

/// Human-readable display name for a namespace segment (first letter upper),
/// matching the Config panel's `ns_display_name` so the capture header text
/// equals the on-screen group label (Config Bug 2).
fn ns_display_name(ns: &str) -> String {
    let mut s = ns.to_string();
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

/// Compute the GLOBAL `:` column for the capture over EVERY currently-visible
/// left-label -- the two header captions, each bracketed group label, and the
/// indented child label of every expanded group -- EXACTLY as
/// `config_panel::render::global_column` does, so the captured monospace text
/// aligns the `:` on every row like the GUI. Returns the (i18n) Filter caption
/// alongside the column so the caller reuses the same string it measured.
fn config_capture_global_column(rows: &[crate::config_panel::ConfigRow]) -> (String, usize) {
    let filter_caption = ff_i18n::t("ff-desktop-config-panel-filter");
    let mut labels: Vec<String> = vec![filter_caption.clone(), UNSAVED_LABEL.to_string()];
    for row in rows {
        match &row.id {
            ConfigNodeId::Namespace(ns) => labels.push(group_left_label(&ns_display_name(ns))),
            ConfigNodeId::Key(key) => labels.push(child_left_label(key)),
        }
    }
    (filter_caption, global_key_column_width(&labels))
}

/// Render a [`ff_config::value::ConfigValue`] as a plain display string for a
/// faithful settings-tree capture (Config Bug 2). Scalars print their natural
/// text; an array lists its elements; a nested table shows its key count so the
/// capture stays single-line per row.
fn config_value_to_string(value: &ff_config::value::ConfigValue) -> String {
    use ff_config::value::ConfigValue;
    match value {
        ConfigValue::String(s) => s.clone(),
        ConfigValue::Integer(i) => i.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::Boolean(b) => b.to_string(),
        ConfigValue::Array(items) => {
            let parts: Vec<String> = items.iter().map(config_value_to_string).collect();
            format!("[{}]", parts.join(", "))
        }
        ConfigValue::Table(t) => format!("{{{} keys}}", t.len()),
        // `ConfigValue` is #[non_exhaustive]: a future variant renders via its
        // Debug form so the capture never silently drops a value.
        other => format!("{other:?}"),
    }
}

impl WorkbenchShell {
    /// Config Panel (Settings / CONFIG) -- the flat config-key browser. The
    /// model MIRRORS the live egui render EXACTLY (Config Bug 3,
    /// master-screen-directive Req 4.4b): it is emitted as pre-composed
    /// `body_lines` built from the SAME pure `config_panel::layout` helpers the
    /// render uses, so the captured text is a SINGLE representation of the
    /// visible settings tree -- the Filter and Unsaved-edits header rows, each
    /// namespace group ONCE as `[Group] : expanded|collapsed` flush-left, and,
    /// for an expanded group, each key ONCE indented one level as `key : value`
    /// with its current EFFECTIVE value (the staged working value if the user
    /// has edited it, marked `(unsaved)`, else the persisted value). All rows
    /// share one GLOBAL aligned `:` column; there are NO `. . .` dot-leaders and
    /// NO duplicate `Settings` table block. Built from the panel's live state +
    /// the config handle, never from egui widgets (Req 2.2).
    pub(super) fn config_panel_screen_model(&self) -> ScreenModel {
        let p = &self.config_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();

        // The REAL settings tree the user sees: reuse the same pure row builder
        // the render + keyboard driver use, so the capture matches the screen
        // exactly (same ordering, same filter, same collapse state).
        let schema_entries = self.config_handle.list_schema_entries();
        let tree_entries: Vec<TreeEntry> = schema_entries
            .iter()
            .map(|e| TreeEntry {
                key: e.key.clone(),
                description: e.description.clone(),
            })
            .collect();
        let rows = visible_rows(&tree_entries, &p.filter, &p.collapsed);
        let (filter_caption, column) = config_capture_global_column(&rows);

        // Header rows, aligned to the same GLOBAL column (no dot-leaders).
        if let Some(ns) = &p.namespace_filter {
            model.body_lines.push(format!(
                "{}{}",
                aligned_left_segment("Namespace", column),
                ns
            ));
        }
        model.body_lines.push(format!(
            "{}{}",
            aligned_left_segment(&filter_caption, column),
            p.filter
        ));
        model.body_lines.push(format!(
            "{}{}",
            aligned_left_segment(UNSAVED_LABEL, column),
            if p.is_dirty() { "yes" } else { "no" }
        ));

        // The visible tree, ONE line per row: flush-left group headers and
        // one-level-indented child rows, each via the SAME aligned segment the
        // render emits.
        for row in &rows {
            model
                .body_lines
                .push(self.config_capture_row_line(row, column));
        }

        if let Some(err) = &p.persist_error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Compose the EXACT captured text line for one visible Config tree row,
    /// aligned to the GLOBAL `:` `column`: a namespace group renders flush-left
    /// as `[Group] : expanded|collapsed`; a key renders indented one level as
    /// `key : value`, the value being its current effective value (a staged edit
    /// marked `(unsaved)`). Mirrors `config_panel::render::render_group`.
    fn config_capture_row_line(
        &self,
        row: &crate::config_panel::ConfigRow,
        column: usize,
    ) -> String {
        match &row.id {
            ConfigNodeId::Namespace(ns) => {
                let state = if row.expanded {
                    "expanded"
                } else {
                    "collapsed"
                };
                format!(
                    "{}{}",
                    aligned_left_segment(&group_left_label(&ns_display_name(ns)), column),
                    state
                )
            }
            ConfigNodeId::Key(key) => {
                let value = self.config_key_display_value(key);
                let value = if self.config_panel.working.contains_key(key) {
                    format!("{value} (unsaved)")
                } else {
                    value
                };
                format!(
                    "{}{}",
                    aligned_left_segment(&child_left_label(key), column),
                    value
                )
            }
        }
    }

    /// The CURRENT displayed value of a config key as a plain string: the staged
    /// working value when the user has edited it (so the capture shows the
    /// unsaved content), otherwise the persisted effective value, falling back
    /// to the schema default. Used by [`Self::config_panel_screen_model`] and
    /// the Config settings-table builder (`screen_providers_tables`).
    pub(super) fn config_key_display_value(&self, key: &str) -> String {
        use crate::config_panel::WorkingEdit;
        // A staged Set shows the edited value; a staged Reset shows the value
        // that would take effect (the persisted/lower-layer value, resolved
        // below exactly as the live panel does).
        if let Some(WorkingEdit::Set(v)) = self.config_panel.working.get(key) {
            return config_value_to_string(v);
        }
        match self.config_handle.get_with_provenance(key) {
            Ok(ev) => config_value_to_string(&ev.value),
            Err(_) => "(default)".to_string(),
        }
    }
}
