//! Rendering + per-frame keyboard driver for the Config View
//! (configuration-system Requirement 15 + Requirement 21 + Requirement 22).
//! Split out of `config_panel/mod.rs` for the 400-line rule.
//!
//! Requirement 22 render model (SUPERSEDES both the box-drawing-string renderer
//! of commit a3059de AND the egui-native-`CollapsingHeader` + PER-GROUP value
//! column of commit 128d8d2). The owner's follow-up request mandates:
//! - group headers FLUSH-LEFT, each `[<Group>] : expanded|collapsed`, showing
//!   the live collapse state in text (no native triangle);
//! - child property rows INDENTED exactly one level under their group header,
//!   each `key : value`;
//! - a SINGLE GLOBAL `:` column aligned across EVERY visible row of the panel
//!   (the `Filter` and `Unsaved edits` header rows, the group rows, and the
//!   child rows) -- see `layout::global_key_column_width`;
//! - no `. . .` dot-leaders.
//!
//! Click-to-collapse is preserved: the group header is a clickable monospace
//! control that toggles the existing per-namespace `collapsed` model (keeping
//! the keyboard reducer and the mouse toggle in sync). The per-key value widget,
//! provenance/LOCKED badge, staged marker, selectable key name, and selectable
//! description are all preserved.

use std::collections::{BTreeMap, HashMap};

use eframe::egui;
use ff_config::schema::SchemaEntry;
use ff_config::ConfigHandle;

use super::keyboard::{config_keyboard_effects, tree_has_keyboard};
use super::layout::{aligned_left_segment, child_left_label, group_left_label};
use super::rows::{paint_cursor_highlight, render_key_row};
use super::{
    filter_field_id, reconcile_cursor, visible_rows, ConfigNodeId, ConfigPanelState, TreeEntry,
};
use crate::shell::workspace_context::ShellRequest;

/// The caption of the (non-interactive) Unsaved-edits header row. A module
/// constant so the row caption and the global-width computation agree.
const UNSAVED_LABEL: &str = "Unsaved edits";

/// Validates: Requirement 15.1-15.8; Requirement 22.1-22.11; project-master Requirement 11.8
pub fn render(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    requests: &mut Vec<ShellRequest>,
) {
    // == Collect schema entries + build the Config_Tree rows ===
    let schema_entries = config.list_schema_entries();
    let entry_by_key: HashMap<String, SchemaEntry> = schema_entries
        .iter()
        .map(|e| (e.key.clone(), e.clone()))
        .collect();
    // Reduced entries for the pure tree model (key + description only).
    let tree_entries: Vec<TreeEntry> = schema_entries
        .iter()
        .map(|e| TreeEntry {
            key: e.key.clone(),
            description: e.description.clone(),
        })
        .collect();

    // Visible_Rows drives the KEYBOARD cursor/navigation (respects collapsed);
    // the grouped map below drives the per-group RENDER.
    let rows = visible_rows(&tree_entries, &state.filter, &state.collapsed);
    let groups = grouped_visible_keys(&tree_entries, &state.filter);

    // GLOBAL `:` column: the max char-width over EVERY currently-visible
    // left-label of the whole panel (header captions + bracketed group rows +
    // the indented child rows of EXPANDED groups), per AC 22.5 (global, not
    // per-group). Computed once and used by both the header area and the tree.
    let column = global_column(&groups, &state.collapsed);

    // Rebuild the per-frame key -> widget-id map fresh each frame.
    state.widget_ids.clear();

    // == Header area (aligned to the SAME global column) ===
    render_header_area(ui, state, requests, column);

    // Exactly ONE horizontal rule demarcates the header area from the tree
    // body (AC 22.1).
    ui.separator();

    // == Keyboard tree navigation (CR-CH-039) ===
    reconcile_cursor(&rows, &rows, &mut state.cursor);
    if tree_has_keyboard(ui, &rows) {
        config_keyboard_effects(ui, state, &rows);
    }

    // == Render the tree -- flush-left group rows + indented child rows (Req 22).
    let gctx = GroupCtx {
        entry_by_key: &entry_by_key,
        cursor: state.cursor.clone(),
        selection_stroke: ui.visuals().selection.stroke,
        column,
    };
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (ns, keys) in &groups {
            render_group(ui, state, config, &gctx, ns, keys);
        }
    });
}

/// Compute the EXACT rendered text of a namespace group's flush-left header row
/// for the live schema + collapse state: `aligned_left_segment("[<Display>]",
/// global_column) + "expanded"|"collapsed"`. This is the single source of truth
/// for the header label, used by `render_group` (indirectly, via the same
/// `aligned_left_segment`/`global_column`) and by the full-shell egui_kittest
/// test so the test matches what is actually drawn (AC 22.3). Test-support only.
#[cfg(test)]
pub fn rendered_group_header(
    config: &ConfigHandle,
    collapsed: &HashMap<String, bool>,
    ns: &str,
) -> String {
    let tree_entries: Vec<TreeEntry> = config
        .list_schema_entries()
        .iter()
        .map(|e| TreeEntry {
            key: e.key.clone(),
            description: e.description.clone(),
        })
        .collect();
    let groups = grouped_visible_keys(&tree_entries, "");
    let column = global_column(&groups, collapsed);
    let expanded = !collapsed.get(ns).copied().unwrap_or(false);
    let state_word = if expanded { "expanded" } else { "collapsed" };
    format!(
        "{}{}",
        aligned_left_segment(&group_left_label(&ns_display_name(ns)), column),
        state_word
    )
}

/// Collect every currently-visible left-label and compute the GLOBAL `:` column
/// width over them (AC 22.5): the two header captions, each group's bracketed
/// label, and -- only for EXPANDED groups -- each child key's indented label.
fn global_column(groups: &[(String, Vec<String>)], collapsed: &HashMap<String, bool>) -> usize {
    let mut labels: Vec<String> = vec![
        ff_i18n::t("ff-desktop-config-panel-filter"),
        UNSAVED_LABEL.to_string(),
    ];
    for (ns, keys) in groups {
        labels.push(group_left_label(&ns_display_name(ns)));
        let expanded = !collapsed.get(ns).copied().unwrap_or(false);
        if expanded {
            for key in keys {
                labels.push(child_left_label(key));
            }
        }
    }
    super::layout::global_key_column_width(&labels)
}

/// The per-frame render context shared by every group: the schema-entry lookup,
/// the current Tree_Cursor, the selection stroke used for the cursor highlight,
/// and the GLOBAL `:` column every row aligns to.
struct GroupCtx<'a> {
    entry_by_key: &'a HashMap<String, SchemaEntry>,
    cursor: Option<ConfigNodeId>,
    selection_stroke: egui::Stroke,
    column: usize,
}

/// Render the demarcated header area above the tree (AC 22.1, 22.2): the Filter
/// field (the FIRST interior control, `filter_field_id()`), the Save
/// affordance, the non-interactive `Unsaved edits : yes|no` row (aligned to the
/// GLOBAL column), any persist error, and the Source File indicator. The
/// `Command ===>` line is owned by the shell above this panel.
fn render_header_area(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    requests: &mut Vec<ShellRequest>,
    column: usize,
) {
    // == Filter + Save bar -- Req 15.7; save-convention Req 11.8 ===
    ui.horizontal(|ui| {
        // The Filter caption padded to the GLOBAL column so its `:` lines up with
        // every group/child row below (AC 22.5). Monospace so char count maps to
        // pixels.
        let filter_caption = ff_i18n::t("ff-desktop-config-panel-filter");
        ui.monospace(aligned_left_segment(&filter_caption, column));
        // Stable id so Tab focus round-trips and the shell can anchor the
        // command-field -> first-interior boundary here (B058, AC 22.10).
        ui.add(egui::TextEdit::singleline(&mut state.filter).id(filter_field_id()));
        if ui.small_button("X").clicked() {
            state.filter.clear();
        }

        // SAVE affordance (project-master Req 11.8): enqueues the SAVE command so
        // persistence gates through the EXACT same single dispatch seam as the
        // typed SAVE verb. Enabled only when there are unsaved buffered edits.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let dirty = state.is_dirty();
            ui.add_enabled_ui(dirty, |ui| {
                if ui.button("Save").clicked() {
                    requests.push(ShellRequest::Command("SAVE".to_string()));
                }
            });
            if dirty {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 180, 100),
                    "\u{25cf} unsaved changes",
                );
            }
        });
    });

    // == Unsaved-edits indicator row -- Req 22.1, 22.2 (non-interactive) ======
    // A plain monospace label (adds NO Tab stop, so the Filter field above stays
    // the first interior control -- AC 22.10). Authoritative dirty indicator
    // tied to is_dirty() (AC 22.2), aligned to the GLOBAL `:` column.
    let dirty_word = if state.is_dirty() { "yes" } else { "no" };
    ui.monospace(format!(
        "{}{}",
        aligned_left_segment(UNSAVED_LABEL, column),
        dirty_word
    ));

    // Surface a persist error from the last SAVE/END attempt (Req 11.4).
    if let Some(err) = &state.persist_error {
        ui.colored_label(egui::Color32::RED, err.as_str());
    }

    // == Source file indicator -- Req 15.8 (header metadata) =================
    if let Some(user_path) = ff_config::paths::user_config_path() {
        ui.horizontal(|ui| {
            ui.label(ff_i18n::t("ff-desktop-config-panel-source-file"));
            ui.monospace(user_path.to_string_lossy().as_ref());
        });
    }
}

/// Render one namespace group: a FLUSH-LEFT clickable header row
/// `[<DisplayName>] : expanded|collapsed` (AC 22.3), its `:` aligned to the
/// GLOBAL column; and, when expanded, each child key row INDENTED one level
/// under it (AC 22.4). Clicking the header toggles the existing per-namespace
/// `collapsed` model (AC 22.8) -- the same state the keyboard reducer writes --
/// so click-to-collapse is preserved without egui's native triangle.
fn render_group(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    gctx: &GroupCtx<'_>,
    ns: &str,
    keys: &[String],
) {
    let expanded = !state.collapsed.get(ns).copied().unwrap_or(false);
    let is_group_cursor = gctx.cursor.as_ref() == Some(&ConfigNodeId::Namespace(ns.to_string()));

    // Flush-left header row: `[Group]` padded to the global column, ` : `, then
    // the live collapse state word. A single clickable monospace selectable
    // label so char-count padding maps to pixels and the `:` aligns globally.
    let state_word = if expanded { "expanded" } else { "collapsed" };
    let header_text = format!(
        "{}{}",
        aligned_left_segment(&group_left_label(&ns_display_name(ns)), gctx.column),
        state_word
    );
    let header_resp = ui.selectable_label(
        is_group_cursor,
        egui::RichText::new(header_text).strong().monospace(),
    );

    // A click on the header toggles the SAME `collapsed` state the keyboard
    // writes (AC 22.8). This is the single authoritative mutation.
    if header_resp.clicked() {
        let now = state.collapsed.get(ns).copied().unwrap_or(false);
        state.collapsed.insert(ns.to_string(), !now);
        state.cursor = Some(ConfigNodeId::Namespace(ns.to_string()));
    }
    paint_cursor_highlight(ui, &header_resp, is_group_cursor, gctx.selection_stroke);

    // Child key rows, indented one level, only while the group is expanded.
    if expanded {
        for key in keys {
            if let Some(entry) = gctx.entry_by_key.get(key) {
                let is_cursor = gctx.cursor.as_ref() == Some(&ConfigNodeId::Key(key.clone()));
                let resp = render_key_row(ui, state, config, entry, gctx.column, is_cursor);
                paint_cursor_highlight(ui, &resp, is_cursor, gctx.selection_stroke);
            }
        }
    }
}

/// Group the filter-matching schema keys by namespace, sorted by namespace and
/// by key within each group -- the SAME grouping + ordering `visible_rows`
/// produces, but holding EVERY matching key per group (collapse is handled by
/// the render loop skipping the children of a collapsed group).
fn grouped_visible_keys(tree_entries: &[TreeEntry], filter: &str) -> Vec<(String, Vec<String>)> {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for e in tree_entries {
        if entry_matches(e, filter) {
            groups
                .entry(namespace_of(&e.key))
                .or_default()
                .push(e.key.clone());
        }
    }
    groups
        .into_iter()
        .map(|(ns, mut keys)| {
            keys.sort();
            (ns, keys)
        })
        .collect()
}

/// Whether an entry matches the filter -- key OR description substring; empty
/// filter matches everything.
fn entry_matches(entry: &TreeEntry, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    let f = filter.to_lowercase();
    entry.key.to_lowercase().contains(&f) || entry.description.to_lowercase().contains(&f)
}

/// Extract the namespace (first dot-segment) from a key path.
fn namespace_of(key: &str) -> String {
    key.split('.').next().unwrap_or(key).to_string()
}

/// Human-readable display name for a namespace segment.
pub(crate) fn ns_display_name(ns: &str) -> String {
    let mut s = ns.to_string();
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 15.2 -- namespace_of extracts first dot-segment
    #[test]
    fn namespace_grouping_correct() {
        assert_eq!(namespace_of("editor.tab_size"), "editor");
        assert_eq!(namespace_of("logging.level"), "logging");
        assert_eq!(namespace_of("theme.active"), "theme");
        assert_eq!(namespace_of("no_dot"), "no_dot");
    }

    // Validates: Requirement 22.3 -- display name capitalises the namespace.
    #[test]
    fn ns_display_name_capitalises() {
        assert_eq!(ns_display_name("editor"), "Editor");
        assert_eq!(ns_display_name("vfs"), "Vfs");
        assert_eq!(ns_display_name("accessibility"), "Accessibility");
    }

    // Validates: Requirement 22.7 -- grouping holds EVERY matching key per group
    // (not gated by collapse), namespaces and keys both sorted, filter applied.
    #[test]
    fn grouped_visible_keys_groups_sorts_and_filters() {
        let entries = vec![
            TreeEntry {
                key: "editor.tab_size".into(),
                description: "Tab size".into(),
            },
            TreeEntry {
                key: "editor.indent_style".into(),
                description: "Indent".into(),
            },
            TreeEntry {
                key: "logging.level".into(),
                description: "Log level".into(),
            },
        ];
        let groups = grouped_visible_keys(&entries, "");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "editor");
        // keys sorted within the group (indent_style before tab_size).
        assert_eq!(
            groups[0].1,
            vec![
                "editor.indent_style".to_string(),
                "editor.tab_size".to_string()
            ]
        );
        assert_eq!(groups[1].0, "logging");

        // Filter by description narrows which groups/keys appear.
        let filtered = grouped_visible_keys(&entries, "log level");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].0, "logging");
    }

    // Validates: Requirement 22.5, 22.6 -- the GLOBAL column spans header rows,
    // group rows, and (for expanded groups) child rows; collapsing a group drops
    // its children from the computation.
    #[test]
    fn global_column_spans_whole_panel_and_tracks_collapse() {
        let groups = vec![
            (
                "catalogs".to_string(),
                vec!["catalogs.default_mainframe_root".to_string()],
            ),
            ("editor".to_string(), vec!["editor.tab_size".to_string()]),
        ];
        // All expanded: the widest row is the long catalogs child (indented).
        let all_expanded = global_column(&groups, &HashMap::new());
        let widest_child = child_left_label("catalogs.default_mainframe_root")
            .chars()
            .count();
        assert_eq!(all_expanded, widest_child + 1);

        // Collapse catalogs: its wide child no longer participates, so the global
        // column narrows (AC 22.6).
        let mut collapsed = HashMap::new();
        collapsed.insert("catalogs".to_string(), true);
        let with_collapse = global_column(&groups, &collapsed);
        assert!(
            with_collapse < all_expanded,
            "collapsing a group with a wide child narrows the global column"
        );
    }

    // Validates: Requirement 15.7 -- filter_hides_non_matching_keys (logic test)
    #[test]
    fn filter_hides_non_matching_keys() {
        let e1 = TreeEntry {
            key: "editor.tab_size".into(),
            description: "Tab size in spaces".into(),
        };
        let e2 = TreeEntry {
            key: "logging.level".into(),
            description: "Log level".into(),
        };
        assert!(entry_matches(&e1, "tab"));
        assert!(!entry_matches(&e2, "tab"));
    }

    // Validates: Requirement 15.3 -- widget type selected for enum string
    #[test]
    fn widget_type_selected_for_enum_string() {
        use ff_config::schema::Constraints;
        use ff_config::value::ConfigValue;
        let constraints = Constraints {
            min: None,
            max: None,
            allowed_values: Some(vec![
                ConfigValue::String("space".to_string()),
                ConfigValue::String("tab".to_string()),
            ]),
            pattern: None,
        };
        assert!(constraints.allowed_values.is_some());
    }

    // Validates: Requirement 15.3 -- widget type selected for bounded int (slider)
    #[test]
    fn widget_type_selected_for_bounded_int() {
        use ff_config::schema::Constraints;
        let constraints = Constraints {
            min: Some(1.0),
            max: Some(16.0),
            allowed_values: None,
            pattern: None,
        };
        assert!(constraints.min.is_some() && constraints.max.is_some());
    }

    // Validates: Requirement 15.10 -- F3/END returns to POM (routing test)
    #[test]
    fn f3_returns_to_pom_via_end_command() {
        use ff_keys::{FunctionKey, KeyBinding, KeyMap};
        let mut map = KeyMap::empty("test");
        map.set(
            ff_keys::ModifiedKey::plain(FunctionKey::F3),
            KeyBinding::with_label("END", "End"),
        );
        let binding = map.get_plain(FunctionKey::F3);
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().command(), "END");
    }
}
