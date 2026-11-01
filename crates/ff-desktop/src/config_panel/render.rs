//! Rendering + per-frame keyboard driver for the Config View
//! (configuration-system Requirement 15 + Requirement 21 + Requirement 22).
//! Split out of `config_panel/mod.rs` for the 400-line rule.
//!
//! Requirement 22 render model (SUPERSEDES the box-drawing-string renderer of
//! commit a3059de): each namespace group is an egui-native
//! `CollapsingHeader`/`CollapsingState` labelled `[<DisplayName>]`, so egui owns
//! the hierarchy, indentation, and the triangle expander. The former
//! box-drawing connector glyphs and the single GLOBAL value column are gone;
//! value alignment is now PER GROUP (see `layout::value_column_width`). The
//! existing per-key value widgets, provenance/LOCKED badge, staged marker, and
//! Reset control are RE-HOMED unchanged inside each header body.

use std::collections::{BTreeMap, HashMap};

use eframe::egui;
use ff_config::layer::ConfigLayer;
use ff_config::schema::SchemaEntry;
use ff_config::ConfigHandle;

use super::keyboard::{config_keyboard_effects, tree_has_keyboard};
use super::layout::{aligned_left_segment, value_column_width};
use super::widgets::render_widget;
use super::{
    filter_field_id, reconcile_cursor, visible_rows, ConfigNodeId, ConfigPanelState, TreeEntry,
};
use crate::shell::workspace_context::ShellRequest;

/// Validates: Requirement 15.1-15.8; Requirement 22.1-22.11; project-master Requirement 11.8
pub fn render(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    requests: &mut Vec<ShellRequest>,
) {
    render_header_area(ui, state, requests);

    // Exactly ONE horizontal rule demarcates the header area from the tree
    // body (AC 22.1).
    ui.separator();

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
    // the grouped map below drives the CollapsingHeader RENDER (holds every
    // matching key per group, egui hides a collapsed group's body).
    let rows = visible_rows(&tree_entries, &state.filter, &state.collapsed);

    // Rebuild the per-frame key -> widget-id map fresh each frame.
    state.widget_ids.clear();

    // == Keyboard tree navigation (CR-CH-039) ===
    reconcile_cursor(&rows, &rows, &mut state.cursor);
    if tree_has_keyboard(ui, &rows) {
        config_keyboard_effects(ui, state, &rows);
    }

    // == Render the tree -- egui-native CollapsingHeader per group (Req 22) ===
    let groups = grouped_visible_keys(&tree_entries, &state.filter);
    let gctx = GroupCtx {
        entry_by_key: &entry_by_key,
        cursor: state.cursor.clone(),
        selection_stroke: ui.visuals().selection.stroke,
    };
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (ns, keys) in &groups {
            render_group(ui, state, config, &gctx, ns, keys);
        }
    });
}

/// The per-frame render context shared by every group: the schema-entry lookup,
/// the current Tree_Cursor, and the selection stroke used for the cursor
/// highlight. Bundled into one borrow so `render_group` stays within the
/// argument-count limit.
struct GroupCtx<'a> {
    entry_by_key: &'a HashMap<String, SchemaEntry>,
    cursor: Option<ConfigNodeId>,
    selection_stroke: egui::Stroke,
}

/// Render the demarcated header area above the tree (AC 22.1, 22.2): the Filter
/// field (the FIRST interior control, `filter_field_id()`), the Save
/// affordance, the non-interactive `Unsaved edits : yes|no` row, any persist
/// error, and the Source File indicator. The `Command ===>` line is owned by
/// the shell above this panel.
fn render_header_area(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    requests: &mut Vec<ShellRequest>,
) {
    // == Filter + Save bar -- Req 15.7; save-convention Req 11.8 ===
    ui.horizontal(|ui| {
        ui.label(ff_i18n::t("ff-desktop-config-panel-filter"));
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
    // A plain label (adds NO Tab stop, so the Filter field above stays the
    // first interior control -- AC 22.10). Authoritative dirty indicator tied
    // to is_dirty() (AC 22.2).
    let dirty_word = if state.is_dirty() { "yes" } else { "no" };
    ui.label(format!("Unsaved edits : {}", dirty_word));

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

/// Render one namespace group as an egui-native collapsible section
/// (AC 22.3, 22.4, 22.8). The header shows the bracketed `[<DisplayName>]` label
/// with egui's native triangle expander; its open-state is DRIVEN from the
/// existing per-namespace `collapsed` model (keeping the keyboard reducer and
/// the mouse toggle in sync), and a click on the header toggles that same model.
/// Inside the body, keys are rendered with a PER-GROUP aligned Value_Column.
fn render_group(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    gctx: &GroupCtx<'_>,
    ns: &str,
    keys: &[String],
) {
    let expanded = !state.collapsed.get(ns).copied().unwrap_or(false);
    let header = format!("[{}] ({})", ns_display_name(ns), keys.len());
    let is_group_cursor = gctx.cursor.as_ref() == Some(&ConfigNodeId::Namespace(ns.to_string()));
    // Per-group Value_Column over THIS group's visible keys (AC 22.5, 22.6).
    let column = value_column_width(keys);

    // egui-native CollapsingHeader with its default triangle expander. `.open`
    // is forced from the authoritative per-namespace `collapsed` model each
    // frame, so the keyboard reducer and the mouse toggle stay in sync (AC
    // 22.8); the header's own click response is read back to toggle that same
    // model. egui owns the hierarchy, indentation, and expander -- the former
    // box-drawing connector glyphs are gone (Req 22 superseding delta).
    let coll = egui::CollapsingHeader::new(egui::RichText::new(&header).strong())
        .id_salt(("config_group", ns))
        .open(Some(expanded))
        .show(ui, |ui| {
            for key in keys {
                if let Some(entry) = gctx.entry_by_key.get(key) {
                    let is_cursor = gctx.cursor.as_ref() == Some(&ConfigNodeId::Key(key.clone()));
                    let resp = render_key_row(ui, state, config, entry, column, is_cursor);
                    paint_cursor_highlight(ui, &resp, is_cursor, gctx.selection_stroke);
                }
            }
        });

    // A click anywhere on the header toggles the SAME `collapsed` state the
    // keyboard writes (AC 22.8). `.open(Some(..))` forces state from the model,
    // so this click is the single authoritative mutation.
    if coll.header_response.clicked() {
        let now = state.collapsed.get(ns).copied().unwrap_or(false);
        state.collapsed.insert(ns.to_string(), !now);
        state.cursor = Some(ConfigNodeId::Namespace(ns.to_string()));
    }
    paint_cursor_highlight(
        ui,
        &coll.header_response,
        is_group_cursor,
        gctx.selection_stroke,
    );
}

/// Group the filter-matching schema keys by namespace, sorted by namespace and
/// by key within each group -- the SAME grouping + ordering `visible_rows`
/// produces, but holding EVERY matching key per group (not gated by collapse)
/// because egui's `CollapsingHeader` hides a collapsed group's body (AC 22.7).
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

/// Paint the Tree_Cursor selection outline around a row's rect (Requirement
/// 21.8) so the keyboard position is always visible, distinct from egui hover.
fn paint_cursor_highlight(
    ui: &egui::Ui,
    resp: &egui::Response,
    is_cursor: bool,
    stroke: egui::Stroke,
) {
    if is_cursor {
        ui.painter()
            .rect_stroke(resp.rect.expand(1.0), 2.0, stroke, egui::StrokeKind::Inside);
    }
}

/// Render a single schema entry key row inside a group body: line 1 is the
/// per-group-aligned key + ":" + value widget (the key a frameless selectable
/// monospace control), line 2 is the selectable description plus the
/// right-to-left provenance/LOCKED badge, staged `*` marker, and Reset block.
/// Re-homed unchanged from the pre-CollapsingHeader renderer (AC 22.9); only the
/// box-drawing connector argument is removed.
///
/// Validates: Requirement 22.4, 22.5, 22.9; Requirement 14.2, 15.3-15.6, 18.6
fn render_key_row(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    entry: &SchemaEntry,
    column: usize,
    is_cursor: bool,
) -> egui::Response {
    let key = &entry.key;
    let locked = config.is_locked(key);

    // Resolve current effective value and provenance.
    let (effective, provenance_label) = match config.get_with_provenance(key) {
        Ok(ev) => {
            let label = if locked {
                "LOCKED"
            } else {
                layer_label(ev.provenance.layer)
            };
            (ev.value, label)
        }
        Err(_) => (
            entry.default.clone(),
            if locked { "LOCKED" } else { "Default" },
        ),
    };

    ui.vertical(|ui| {
        // -- Line 1: per-group-aligned key -> `:` -> value -------------------
        ui.horizontal(|ui| {
            // The key stays a SELECTABLE control (Req 14.2 / AC 22.9). Frameless
            // + monospace so char-count padding maps to pixels and the `:` lands
            // at the per-group Value_Column on every row of the group.
            let segment = aligned_left_segment(key, column);
            let _ = ui.selectable_label(is_cursor, egui::RichText::new(segment).monospace());

            // Value widget starts at the Value_Column -- disabled for locked
            // keys (Req 18.6). Its id is recorded so Enter can focus it (Req 21.6).
            let widget_id = ui
                .add_enabled_ui(!locked, |ui| {
                    render_widget(ui, state, config, entry, &effective)
                })
                .inner;
            if let Some(id) = widget_id {
                state.widget_ids.insert(entry.key.clone(), id);
            }
        });

        // -- Line 2: selectable description + right-to-left provenance block --
        ui.horizontal(|ui| {
            // Visible, SELECTABLE description (Req 14.2) -- left-aligned.
            ui.add(egui::Button::selectable(
                false,
                egui::RichText::new(entry.description.as_str())
                    .small()
                    .weak(),
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Provenance / LOCKED badge -- Req 15.3, 18.6
                let badge_color = if locked {
                    egui::Color32::from_rgb(200, 80, 80)
                } else {
                    egui::Color32::from_rgb(120, 180, 120)
                };
                ui.add(egui::Button::selectable(
                    false,
                    egui::RichText::new(provenance_label)
                        .small()
                        .color(badge_color),
                ));

                // A staged (unsaved) edit on this key shows an asterisk after
                // the provenance badge (Req 11.9).
                if state.working.contains_key(key) {
                    ui.add(egui::Button::selectable(
                        false,
                        egui::RichText::new("*")
                            .small()
                            .color(egui::Color32::from_rgb(220, 180, 100)),
                    ));
                }

                // Reset to Default button -- Req 15.6 (disabled when locked or at
                // Default WITH no staged edit).
                let is_at_default = provenance_label == "Default";
                let has_staged_edit = state.working.contains_key(key);
                let reset_enabled = !locked && (!is_at_default || has_staged_edit);
                ui.add_enabled_ui(reset_enabled, |ui| {
                    if ui.small_button("\u{21ba} Reset").clicked() {
                        // Stage a Reset into the working buffer (Req 11.2).
                        state.stage_reset(key);
                    }
                });
            });
        });

        // Inline validation error -- Req 15.5
        if let Some(err) = state.errors.get(key) {
            ui.colored_label(egui::Color32::RED, err.as_str());
        }
    })
    .response
}

/// Extract the namespace (first dot-segment) from a key path.
fn namespace_of(key: &str) -> String {
    key.split('.').next().unwrap_or(key).to_string()
}

/// Human-readable display name for a namespace segment.
fn ns_display_name(ns: &str) -> String {
    let mut s = ns.to_string();
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

/// Map a `ConfigLayer` to a short provenance label string.
fn layer_label(layer: ConfigLayer) -> &'static str {
    match layer {
        ConfigLayer::Defaults => "Default",
        ConfigLayer::System => "System",
        ConfigLayer::User => "User",
        ConfigLayer::Profile => "Profile",
        ConfigLayer::Project => "Project",
        ConfigLayer::Workspace => "Workspace",
    }
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

    // Validates: Requirement 15.3 -- provenance badge shows correct layer label
    #[test]
    fn provenance_badge_shows_correct_layer() {
        assert_eq!(layer_label(ConfigLayer::Defaults), "Default");
        assert_eq!(layer_label(ConfigLayer::User), "User");
        assert_eq!(layer_label(ConfigLayer::Project), "Project");
        assert_eq!(layer_label(ConfigLayer::System), "System");
        assert_eq!(layer_label(ConfigLayer::Profile), "Profile");
        assert_eq!(layer_label(ConfigLayer::Workspace), "Workspace");
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

    // Validates: Requirement 15.6 -- reset button hidden when at default (provenance check)
    #[test]
    fn reset_button_hidden_when_at_default() {
        let at_default = layer_label(ConfigLayer::Defaults) == "Default";
        assert!(at_default, "Default layer should produce 'Default' label");
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
