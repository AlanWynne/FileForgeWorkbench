//! Pure, unit-testable tree model + keyboard reducer for the Config View
//! (configuration-system Requirement 21, CR-CH-039 / B069).
//!
//! The Config View is a NODE TREE: a level of NAMESPACE GROUP nodes (expandable
//! branches, e.g. `editor`, `logging`) whose children are the KEY nodes under
//! that namespace (leaves, e.g. `editor.tab_size`). This module holds the pure
//! logic so the keyboard transitions are covered by tests independent of egui
//! rendering, mirroring the File Explorer's `explorer_view::reduce_key` pattern.
//!
//! Rendering + the per-frame egui driver live in the parent module; this file is
//! deliberately GUI-free (it takes plain data), so every transition is a pure
//! function over `(rows, cursor)`.

/// A node in the Config_Tree: either a namespace GROUP (branch) or a KEY (leaf).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigNodeId {
    /// A namespace group node, keyed by its namespace segment (e.g. `"editor"`).
    Namespace(String),
    /// A configuration key node (leaf), keyed by its full key (e.g.
    /// `"editor.tab_size"`).
    Key(String),
}

/// One visible row of the Config_Tree, in display order.
///
/// `expandable` is true only for namespace group nodes; `expanded` reflects the
/// current per-namespace collapse state (a group is expanded when NOT collapsed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRow {
    /// The node this row draws.
    pub id: ConfigNodeId,
    /// Indent depth: 0 for a namespace group, 1 for a key under it.
    pub depth: u8,
    /// True for a namespace group node (a branch that can expand/collapse).
    pub expandable: bool,
    /// True when this (expandable) node is currently expanded.
    pub expanded: bool,
}

/// A schema entry reduced to the two fields the tree needs (full key + the
/// text searched by the filter). Keeps this module free of the `ff_config`
/// `SchemaEntry` type so it is trivially testable.
#[derive(Debug, Clone)]
pub struct TreeEntry {
    /// Full dotted key path (e.g. `"editor.tab_size"`).
    pub key: String,
    /// Human-readable description (also matched by the filter).
    pub description: String,
}

/// A keyboard gesture the Config_Tree understands (Requirement 21.3-21.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigTreeKey {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Home,
    End,
}

/// The side effect a reducer step asks the caller (the render/driver) to apply.
/// Pure cursor moves mutate the cursor in place and return `None` (mirroring
/// `explorer_view::reduce_key`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigTreeEffect {
    /// No side effect (a pure cursor move, or a no-op at a boundary).
    None,
    /// Expand the named namespace group (set collapsed[ns] = false).
    Expand(String),
    /// Collapse the named namespace group (set collapsed[ns] = true).
    Collapse(String),
    /// Move egui keyboard focus to the value-editing widget of the named key.
    FocusKeyWidget(String),
}

/// The namespace segment of a full key (first dot-segment). Matches
/// `config_panel::namespace_of`.
fn namespace_of(key: &str) -> String {
    key.split('.').next().unwrap_or(key).to_string()
}

/// Whether an entry matches the (already lowercased) filter, by key OR
/// description substring. An empty filter matches everything. Matches the
/// grouping filter in `config_panel::render`.
fn entry_matches(entry: &TreeEntry, filter_lower: &str) -> bool {
    if filter_lower.is_empty() {
        return true;
    }
    entry.key.to_lowercase().contains(filter_lower)
        || entry.description.to_lowercase().contains(filter_lower)
}

/// Build the ordered list of Visible_Rows (Requirement 21.1): every namespace
/// group (that has at least one filter-matching key), in sorted namespace order;
/// and, for each EXPANDED group, its matching key nodes in sorted key order.
///
/// `collapsed` maps a namespace to true when the user has collapsed it; a
/// namespace absent from the map (or mapped to false) is expanded.
pub fn visible_rows(
    entries: &[TreeEntry],
    filter: &str,
    collapsed: &std::collections::HashMap<String, bool>,
) -> Vec<ConfigRow> {
    let filter_lower = filter.to_lowercase();

    // Group matching entries by namespace, sorted (BTreeMap) like the render.
    let mut groups: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for entry in entries {
        if !entry_matches(entry, &filter_lower) {
            continue;
        }
        groups
            .entry(namespace_of(&entry.key))
            .or_default()
            .push(entry.key.clone());
    }

    let mut rows = Vec::new();
    for (ns, mut keys) in groups {
        let expanded = !collapsed.get(&ns).copied().unwrap_or(false);
        rows.push(ConfigRow {
            id: ConfigNodeId::Namespace(ns.clone()),
            depth: 0,
            expandable: true,
            expanded,
        });
        if expanded {
            keys.sort();
            for key in keys {
                rows.push(ConfigRow {
                    id: ConfigNodeId::Key(key),
                    depth: 1,
                    expandable: false,
                    expanded: false,
                });
            }
        }
    }
    rows
}

/// Index of the row holding `cursor` within `rows`, if present.
fn cursor_index(rows: &[ConfigRow], cursor: &Option<ConfigNodeId>) -> Option<usize> {
    let c = cursor.as_ref()?;
    rows.iter().position(|r| &r.id == c)
}

/// The namespace a node belongs to: a `Key` maps to its namespace segment; a
/// `Namespace` maps to itself.
fn parent_namespace(id: &ConfigNodeId) -> String {
    match id {
        ConfigNodeId::Namespace(ns) => ns.clone(),
        ConfigNodeId::Key(key) => namespace_of(key),
    }
}

/// Apply one keyboard gesture to the Config_Tree over `rows`, moving `cursor`
/// in place for navigation and returning any [`ConfigTreeEffect`] the caller
/// must apply (expand/collapse/focus). Pure: it reads `rows` (built from the
/// current collapsed state) and never touches egui.
///
/// Transition table (Requirement 21.3-21.7), mirroring
/// `explorer_view::reduce_key`:
/// - Down/Up: cursor to next/previous row, clamped (no wrap).
/// - Right: collapsed group -> Expand; expanded group -> cursor to first child.
/// - Left: expanded group -> Collapse; key or collapsed group -> cursor to parent.
/// - Enter: group -> toggle expand/collapse; key -> FocusKeyWidget.
/// - Home/End: cursor to first/last row.
///
/// Validates: Requirement 21.3, 21.4, 21.5, 21.6, 21.7, 21.11.
pub fn reduce_config_key(
    rows: &[ConfigRow],
    cursor: &mut Option<ConfigNodeId>,
    key: ConfigTreeKey,
) -> ConfigTreeEffect {
    if rows.is_empty() {
        *cursor = None;
        return ConfigTreeEffect::None;
    }

    // If there is no cursor yet, any navigation key establishes it on the first
    // row (Requirement 21.2), then the gesture applies from there for Down.
    let idx = match cursor_index(rows, cursor) {
        Some(i) => i,
        None => {
            *cursor = Some(rows[0].id.clone());
            if matches!(key, ConfigTreeKey::Down) {
                // Down from "no cursor" lands on the first row (just set); do not
                // additionally advance.
                return ConfigTreeEffect::None;
            }
            0
        }
    };

    match key {
        ConfigTreeKey::Down => {
            let next = (idx + 1).min(rows.len() - 1);
            *cursor = Some(rows[next].id.clone());
            ConfigTreeEffect::None
        }
        ConfigTreeKey::Up => {
            let prev = idx.saturating_sub(1);
            *cursor = Some(rows[prev].id.clone());
            ConfigTreeEffect::None
        }
        ConfigTreeKey::Right => {
            let row = &rows[idx];
            if row.expandable {
                if !row.expanded {
                    // Collapsed group -> expand it.
                    return ConfigTreeEffect::Expand(parent_namespace(&row.id));
                }
                // Expanded group -> move to first child (the next row, if it is a
                // key under this namespace).
                if let Some(next) = rows.get(idx + 1) {
                    if next.depth > row.depth {
                        *cursor = Some(next.id.clone());
                    }
                }
            }
            ConfigTreeEffect::None
        }
        ConfigTreeKey::Left => {
            let row = &rows[idx];
            if row.expandable && row.expanded {
                // Expanded group -> collapse it.
                return ConfigTreeEffect::Collapse(parent_namespace(&row.id));
            }
            // Key node (or collapsed group) -> move cursor to the parent namespace
            // group row. A collapsed top-level group has no parent -> no-op.
            if let ConfigNodeId::Key(_) = row.id {
                let ns = parent_namespace(&row.id);
                let ns_id = ConfigNodeId::Namespace(ns);
                if rows.iter().any(|r| r.id == ns_id) {
                    *cursor = Some(ns_id);
                }
            }
            ConfigTreeEffect::None
        }
        ConfigTreeKey::Enter => {
            let row = &rows[idx];
            match &row.id {
                ConfigNodeId::Namespace(ns) => {
                    if row.expanded {
                        ConfigTreeEffect::Collapse(ns.clone())
                    } else {
                        ConfigTreeEffect::Expand(ns.clone())
                    }
                }
                ConfigNodeId::Key(key) => ConfigTreeEffect::FocusKeyWidget(key.clone()),
            }
        }
        ConfigTreeKey::Home => {
            *cursor = Some(rows[0].id.clone());
            ConfigTreeEffect::None
        }
        ConfigTreeKey::End => {
            *cursor = Some(rows[rows.len() - 1].id.clone());
            ConfigTreeEffect::None
        }
    }
}

/// Reconcile the cursor against a freshly built row list (Requirement 21.10):
/// keep it if still visible; otherwise move it to the nearest remaining row at
/// or after its previous index; otherwise clear it when no rows remain. The
/// caller passes the PREVIOUS row list (before the filter change) so "nearest"
/// can be computed from the old index.
///
/// Validates: Requirement 21.10.
pub fn reconcile_cursor(
    new_rows: &[ConfigRow],
    prev_rows: &[ConfigRow],
    cursor: &mut Option<ConfigNodeId>,
) {
    if cursor.is_none() {
        return;
    }
    // Still visible? keep it.
    if cursor_index(new_rows, cursor).is_some() {
        return;
    }
    if new_rows.is_empty() {
        *cursor = None;
        return;
    }
    // Move to the nearest remaining row: clamp the old index into the new list.
    let old_idx = cursor_index(prev_rows, cursor).unwrap_or(0);
    let new_idx = old_idx.min(new_rows.len() - 1);
    *cursor = Some(new_rows[new_idx].id.clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn entries() -> Vec<TreeEntry> {
        vec![
            TreeEntry {
                key: "editor.tab_size".into(),
                description: "Tab size in spaces".into(),
            },
            TreeEntry {
                key: "editor.word_wrap".into(),
                description: "Wrap long lines".into(),
            },
            TreeEntry {
                key: "logging.level".into(),
                description: "Log level".into(),
            },
            TreeEntry {
                key: "theme.active".into(),
                description: "Active theme name".into(),
            },
        ]
    }

    // Validates: Requirement 21.1 -- rows are namespaces sorted, then keys sorted
    // within each expanded group; a collapsed group hides its keys.
    #[test]
    fn visible_rows_orders_namespaces_then_keys_and_honours_collapse() {
        let e = entries();
        let collapsed = HashMap::new();
        let rows = visible_rows(&e, "", &collapsed);
        let ids: Vec<&ConfigNodeId> = rows.iter().map(|r| &r.id).collect();
        assert_eq!(
            ids,
            vec![
                &ConfigNodeId::Namespace("editor".into()),
                &ConfigNodeId::Key("editor.tab_size".into()),
                &ConfigNodeId::Key("editor.word_wrap".into()),
                &ConfigNodeId::Namespace("logging".into()),
                &ConfigNodeId::Key("logging.level".into()),
                &ConfigNodeId::Namespace("theme".into()),
                &ConfigNodeId::Key("theme.active".into()),
            ]
        );

        // Collapse "editor": its keys disappear, the group stays.
        let mut collapsed = HashMap::new();
        collapsed.insert("editor".to_string(), true);
        let rows = visible_rows(&e, "", &collapsed);
        assert!(rows
            .iter()
            .any(|r| r.id == ConfigNodeId::Namespace("editor".into()) && !r.expanded));
        assert!(!rows
            .iter()
            .any(|r| r.id == ConfigNodeId::Key("editor.tab_size".into())));
    }

    // Validates: Requirement 21.1 -- the filter (key OR description substring)
    // limits which keys, and therefore which groups, are visible.
    #[test]
    fn visible_rows_applies_filter_by_key_or_description() {
        let e = entries();
        let collapsed = HashMap::new();
        // "wrap" matches editor.word_wrap by key AND "Wrap long lines" by desc.
        let rows = visible_rows(&e, "wrap", &collapsed);
        assert!(rows
            .iter()
            .any(|r| r.id == ConfigNodeId::Key("editor.word_wrap".into())));
        assert!(!rows
            .iter()
            .any(|r| r.id == ConfigNodeId::Namespace("logging".into())));
    }

    // Validates: Requirement 21.2/21.3 -- Down with no cursor lands on the first
    // row; further Down/Up move and clamp at the ends (no wrap).
    #[test]
    fn down_up_move_and_clamp_no_wrap() {
        let e = entries();
        let collapsed = HashMap::new();
        let rows = visible_rows(&e, "", &collapsed);
        let mut cursor = None;

        // First Down establishes the cursor on row 0.
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Down),
            ConfigTreeEffect::None
        );
        assert_eq!(cursor, Some(ConfigNodeId::Namespace("editor".into())));

        // Up at the top clamps (stays on row 0).
        reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Up);
        assert_eq!(cursor, Some(ConfigNodeId::Namespace("editor".into())));

        // Down walks to the next row.
        reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Down);
        assert_eq!(cursor, Some(ConfigNodeId::Key("editor.tab_size".into())));

        // End jumps to the last row; Down there clamps.
        reduce_config_key(&rows, &mut cursor, ConfigTreeKey::End);
        assert_eq!(cursor, Some(ConfigNodeId::Key("theme.active".into())));
        reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Down);
        assert_eq!(cursor, Some(ConfigNodeId::Key("theme.active".into())));
    }

    // Validates: Requirement 21.7 -- Home/End jump to first/last visible row.
    #[test]
    fn home_and_end_jump_to_first_and_last() {
        let e = entries();
        let collapsed = HashMap::new();
        let rows = visible_rows(&e, "", &collapsed);
        let mut cursor = Some(ConfigNodeId::Key("logging.level".into()));

        reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Home);
        assert_eq!(cursor, Some(ConfigNodeId::Namespace("editor".into())));

        reduce_config_key(&rows, &mut cursor, ConfigTreeKey::End);
        assert_eq!(cursor, Some(ConfigNodeId::Key("theme.active".into())));
    }

    // Validates: Requirement 21.4 -- Right on a collapsed group expands it;
    // Right on an expanded group moves the cursor to its first child key.
    #[test]
    fn right_expands_collapsed_group_then_steps_to_first_child() {
        let e = entries();
        let mut collapsed = HashMap::new();
        collapsed.insert("editor".to_string(), true);
        let rows = visible_rows(&e, "", &collapsed);
        let mut cursor = Some(ConfigNodeId::Namespace("editor".into()));

        // Collapsed -> Expand effect (cursor unchanged).
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Right),
            ConfigTreeEffect::Expand("editor".into())
        );
        assert_eq!(cursor, Some(ConfigNodeId::Namespace("editor".into())));

        // Now expanded (rebuild rows): Right steps to the first child key.
        let collapsed = HashMap::new();
        let rows = visible_rows(&e, "", &collapsed);
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Right),
            ConfigTreeEffect::None
        );
        assert_eq!(cursor, Some(ConfigNodeId::Key("editor.tab_size".into())));
    }

    // Validates: Requirement 21.5 -- Left on an expanded group collapses it;
    // Left on a key moves the cursor to its parent namespace group.
    #[test]
    fn left_collapses_expanded_group_and_key_goes_to_parent() {
        let e = entries();
        let collapsed = HashMap::new();
        let rows = visible_rows(&e, "", &collapsed);

        // On an expanded group, Left -> Collapse effect.
        let mut cursor = Some(ConfigNodeId::Namespace("editor".into()));
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Left),
            ConfigTreeEffect::Collapse("editor".into())
        );

        // On a key, Left -> cursor moves to the parent namespace.
        let mut cursor = Some(ConfigNodeId::Key("editor.word_wrap".into()));
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Left),
            ConfigTreeEffect::None
        );
        assert_eq!(cursor, Some(ConfigNodeId::Namespace("editor".into())));
    }

    // Validates: Requirement 21.6 -- Enter toggles a group; Enter on a key asks
    // to focus that key's value widget.
    #[test]
    fn enter_toggles_group_and_focuses_key_widget() {
        let e = entries();
        let collapsed = HashMap::new();
        let rows = visible_rows(&e, "", &collapsed);

        // Expanded group -> Collapse.
        let mut cursor = Some(ConfigNodeId::Namespace("editor".into()));
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Enter),
            ConfigTreeEffect::Collapse("editor".into())
        );

        // Collapsed group -> Expand.
        let mut collapsed = HashMap::new();
        collapsed.insert("logging".to_string(), true);
        let rows = visible_rows(&e, "", &collapsed);
        let mut cursor = Some(ConfigNodeId::Namespace("logging".into()));
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Enter),
            ConfigTreeEffect::Expand("logging".into())
        );

        // Key -> FocusKeyWidget.
        let rows = visible_rows(&e, "", &HashMap::new());
        let mut cursor = Some(ConfigNodeId::Key("theme.active".into()));
        assert_eq!(
            reduce_config_key(&rows, &mut cursor, ConfigTreeKey::Enter),
            ConfigTreeEffect::FocusKeyWidget("theme.active".into())
        );
    }

    // Validates: Requirement 21.10 -- when the filter change hides the cursor
    // node, the cursor moves to the nearest remaining row; when nothing remains
    // it clears; when still visible it is kept.
    #[test]
    fn reconcile_cursor_keeps_moves_or_clears() {
        let e = entries();
        let prev = visible_rows(&e, "", &HashMap::new());

        // Cursor on a row that survives the filter -> kept.
        let mut cursor = Some(ConfigNodeId::Key("editor.word_wrap".into()));
        let new_rows = visible_rows(&e, "wrap", &HashMap::new());
        reconcile_cursor(&new_rows, &prev, &mut cursor);
        assert_eq!(cursor, Some(ConfigNodeId::Key("editor.word_wrap".into())));

        // Cursor on a row the filter hides -> moves to nearest remaining.
        let mut cursor = Some(ConfigNodeId::Key("theme.active".into()));
        reconcile_cursor(&new_rows, &prev, &mut cursor);
        assert!(new_rows.iter().any(|r| Some(&r.id) == cursor.as_ref()));

        // Filter matches nothing -> cursor cleared.
        let empty = visible_rows(&e, "zzz-no-match", &HashMap::new());
        let mut cursor = Some(ConfigNodeId::Key("theme.active".into()));
        reconcile_cursor(&empty, &prev, &mut cursor);
        assert_eq!(cursor, None);
    }
}
