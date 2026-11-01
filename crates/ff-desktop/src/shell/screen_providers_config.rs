//! # Config Panel ScreenProvider -- faithful settings-tree capture
//!
//! Split out of `shell/screen_providers.rs` for the 400-line rule (behaviour
//! unchanged). This holds the Config Panel's `ScreenModel` builder, which is a
//! FAITHFUL capture of the on-screen settings tree (config-snapshot-bugs.md
//! Config Bug 2, master-screen-directive Req 4.4a, screen-snapshot-scrm Req
//! 27.8): the active filter/namespace scope, the dirty flag, AND the visible
//! tree rows -- each namespace group and, for an expanded group, its keys with
//! their current effective values. Built from the panel's live state + the
//! config handle, never from egui widgets (Req 2.2).
//!
//! Validates: screen-snapshot-scrm Requirement 27.8; master-screen-directive
//! Requirement 4.4a.

use ff_screen_model::{Field, ScreenModel};

use super::screen_providers::active_display_title;
use super::WorkbenchShell;

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
    /// model is a FAITHFUL capture of the on-screen settings tree (Config Bug 2,
    /// master-screen-directive Req 4.4a, screen-snapshot-scrm Req 27.8): the
    /// active filter/namespace scope, the dirty flag, AND the VISIBLE tree rows
    /// -- each namespace group and, for an expanded group, its keys with their
    /// current EFFECTIVE values (the staged working value if the user has edited
    /// it, else the persisted value), respecting the live filter and
    /// expand/collapse state. A key with a staged edit is marked so the capture
    /// shows what the user is editing. Built from the panel's live state + the
    /// config handle, never from egui widgets (Req 2.2). ANSI colour for
    /// changed/error keys is a later enhancement.
    pub(super) fn config_panel_screen_model(&self) -> ScreenModel {
        use crate::config_panel::{visible_rows, ConfigNodeId, TreeEntry};
        let p = &self.config_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        if let Some(ns) = &p.namespace_filter {
            model = model.with_field(Field::new("Namespace", ns.clone()));
        }
        model = model.with_field(Field::new("Filter", p.filter.clone()));
        model = model.with_field(Field::new(
            "Unsaved edits",
            if p.is_dirty() { "yes" } else { "no" }.to_string(),
        ));

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
        for row in &rows {
            match &row.id {
                ConfigNodeId::Namespace(ns) => {
                    // A group header row: "editor" collapsed/expanded, with the
                    // same display-cased name the panel shows.
                    let state = if row.expanded {
                        "expanded"
                    } else {
                        "collapsed"
                    };
                    model = model.with_field(Field::new(
                        format!("[{}]", ns_display_name(ns)),
                        state.to_string(),
                    ));
                }
                ConfigNodeId::Key(key) => {
                    // A key row under an expanded group: its CURRENT effective
                    // value (staged working value if edited, else persisted),
                    // with a trailing marker when a staged edit is pending.
                    let value = self.config_key_display_value(key);
                    let value = if p.working.contains_key(key) {
                        format!("{value} (unsaved)")
                    } else {
                        value
                    };
                    model = model.with_field(Field::new(key.clone(), value));
                }
            }
        }
        if let Some(err) = &p.persist_error {
            model.messages.push(err.clone());
        }
        model
    }

    /// The CURRENT displayed value of a config key as a plain string: the staged
    /// working value when the user has edited it (so the capture shows the
    /// unsaved content), otherwise the persisted effective value, falling back
    /// to the schema default. Used only by [`Self::config_panel_screen_model`].
    fn config_key_display_value(&self, key: &str) -> String {
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
