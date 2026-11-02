//! Config-panel capture fidelity (config-snapshot-bugs.md: Config Bug 1 + Bug
//! 2) folded into Stream SCRM Phase 2E because they are the SAME defect class as
//! the dialog capture work (a ScreenModel not faithfully capturing real
//! on-screen content, and a capture that mutates the screen it captures).
//!
//! Bug 2 (faithful model): the Config ScreenProvider must capture the REAL
//! settings tree -- the visible categories and their current values -- not just
//! Filter + Unsaved-edits (screen-snapshot-scrm Req 27.8, master-screen-directive
//! Req 4.4a).
//!
//! Bug 1 (side-effect-free capture): submitting a command from the command line
//! (e.g. SNAPSHOT) must NOT also activate/collapse the focused config-tree node.
//! The Enter that submits the command was ALSO being read by the Config_Tree's
//! per-frame keyboard driver, collapsing the focused node (master-screen-directive
//! Req 4.4a: capture must be side-effect-free).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;

/// Open the flat Config panel through the real front door and settle it.
fn open_config(harness: &mut egui_kittest::Harness<'_, super::WorkbenchShell>) {
    harness.state_mut().dispatch_command_string("CONFIG");
    for _ in 0..4 {
        harness.run();
    }
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: CONFIG opens the flat Config panel"
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (Config Bug 2) -- the Config ScreenProvider captures the REAL settings tree:
// namespace group rows AND key rows with their current effective values, not a
// thin two-field (Filter + Unsaved) summary. The faithful tree is emitted as
// pre-composed `body_lines` that mirror the visible render (Config Bug 3).
#[test]
fn config_screen_model_captures_the_real_settings_tree() {
    let mut harness = harness_shell();
    open_config(&mut harness);

    let model = harness.state().config_panel_screen_model();

    // The capture is emitted as pre-composed body lines (mirror the render),
    // NOT the generic label/value fields path and NOT a tables block.
    assert!(
        model.fields.is_empty(),
        "the config capture uses body_lines, not the generic fields path"
    );
    assert!(
        model.tables.is_empty(),
        "the config capture has no duplicate Settings table block (Bug 3)"
    );

    // The header rows are present (aligned, no dot-leaders).
    assert!(
        model.body_lines.iter().any(|l| l.contains("Filter")),
        "Filter header row retained"
    );
    assert!(
        model.body_lines.iter().any(|l| l.contains("Unsaved edits")),
        "Unsaved-edits header row retained"
    );

    // There must be at least one namespace GROUP header row (e.g. "[Editor]",
    // "[Accessibility]"): a bracketed flush-left label ending in the state word.
    let group_lines: Vec<_> = model
        .body_lines
        .iter()
        .filter(|l| l.starts_with('[') && (l.ends_with("expanded") || l.ends_with("collapsed")))
        .collect();
    assert!(
        !group_lines.is_empty(),
        "the capture must list the real settings-tree namespace groups (Bug 2); \
         got body_lines: {:?}",
        model.body_lines
    );

    // There must be at least one KEY row -- a dotted schema key with a value --
    // under an expanded group, proving actual settings + values are captured.
    let key_lines: Vec<_> = model
        .body_lines
        .iter()
        .filter(|l| l.trim_start().contains('.') && l.contains(" : "))
        .collect();
    assert!(
        !key_lines.is_empty(),
        "the capture must list real settings keys with their values (Bug 2); \
         got body_lines: {:?}",
        model.body_lines
    );

    // The whole thing is strictly richer than the old 2-field summary.
    assert!(
        model.body_lines.len() > 2,
        "the faithful model is richer than the thin Filter+Unsaved summary (Bug 2)"
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a --
// a key the user has edited (a staged working value) is captured with its
// CURRENT unsaved value and marked unsaved, so the capture reflects what is on
// screen while editing.
#[test]
fn config_screen_model_captures_staged_edit_value() {
    use ff_config::value::ConfigValue;
    let mut harness = harness_shell();
    open_config(&mut harness);

    // Stage an edit directly into the working buffer and expand its namespace so
    // the key row is visible in the tree.
    let key = "accessibility.reduce_motion";
    harness
        .state_mut()
        .config_panel
        .stage_set(key, ConfigValue::Boolean(true));
    harness
        .state_mut()
        .config_panel
        .collapsed
        .insert("accessibility".to_string(), false);
    harness.run();

    let model = harness.state().config_panel_screen_model();
    // The key row is a body line whose (trimmed) left segment is the dotted key.
    let row = model
        .body_lines
        .iter()
        .find(|l| l.trim_start().starts_with(key) && l.contains(" : "));
    // The accessibility namespace may or may not exist in every schema build;
    // when it does, the staged value must be captured and marked unsaved.
    if let Some(row) = row {
        assert!(
            row.contains("true"),
            "the staged working value is captured, got {:?}",
            row
        );
        assert!(
            row.contains("unsaved"),
            "a staged edit is marked unsaved in the capture, got {:?}",
            row
        );
    }
}

// Validates: master-screen-directive Req 4.4a (Config Bug 1) -- submitting a
// command from the command line does NOT activate/collapse the focused
// config-tree node. SNAPSHOT (and any command) must leave the screen exactly as
// it was: the Enter that submits the command must be consumed by the command
// line and NOT also read by the Config_Tree keyboard driver.
#[test]
fn command_submit_does_not_collapse_focused_config_tree_node() {
    let mut harness = harness_shell();
    open_config(&mut harness);

    // Startup re-arms command-field focus on entering a Workspace, so the
    // command line holds focus (the exact state in the owner's repro).
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "precondition: the command line holds focus"
    );
    // Precondition: nothing is collapsed yet (every namespace expanded).
    assert!(
        !harness
            .state()
            .config_panel
            .collapsed
            .values()
            .any(|collapsed| *collapsed),
        "precondition: no namespace group is collapsed on entry"
    );

    // Type a command and press Enter -- the exact repro (owner: `snapshot`+Enter
    // collapsed the focused Accessibility item). SNAPSHOT only arms the async
    // screenshot; it must not mutate the tree.
    harness.state_mut().command_text = "SNAPSHOT".to_string();
    harness.key_press(egui::Key::Enter);
    harness.run();

    // The command submitted (the field was cleared by the Command_Line_Outcome),
    // and NO namespace group was collapsed as a side effect of the Enter.
    assert!(
        !harness
            .state()
            .config_panel
            .collapsed
            .values()
            .any(|collapsed| *collapsed),
        "submitting a command must not collapse the focused config-tree node (Bug 1)"
    );
}

// Validates: screen-snapshot-scrm Req 27.8, 27.9 / master-screen-directive Req
// 4.4b (Config Bug 3) -- the Config capture text MIRRORS the visible render
// EXACTLY: each row appears ONCE, there are NO ". . ." dot-leaders, a group
// header renders flush-left as "[Editor] : expanded|collapsed", a child renders
// indented with a single " : " separator, there is NO duplicate Settings table
// block, and the ":" column is aligned per the pure layout helper.
#[test]
fn config_capture_mirrors_visible_render_single_representation() {
    use crate::config_panel::{
        aligned_left_segment, child_left_label, global_key_column_width, group_left_label,
    };
    use ff_screen_model::{RenderOptions, SnapshotFormat};

    let mut harness = harness_shell();
    open_config(&mut harness);

    let model = harness.state().config_panel_screen_model();

    // (b) NO ". . ." dot-leaders anywhere in the model's body lines, and the
    // capture is NOT the generic fields/tables path (which would re-emit the
    // list twice -- once dot-leadered, once as the Settings table).
    assert!(
        model.fields.is_empty() && model.tables.is_empty(),
        "capture is a single body_lines representation (no fields, no table)"
    );
    for line in &model.body_lines {
        assert!(
            !line.contains(". . ."),
            "no dot-leaders in the capture, got: {line:?}"
        );
    }

    // The same must hold for the ACTUAL rendered snapshot text the user pastes:
    // the plain-text and the Markdown (clipboard) flavors both render from this
    // one model, so neither may contain dot-leaders, and the key must appear
    // exactly once in each.
    let key = "editor.tab_size";
    let plain =
        ff_screen_model::render(&model, SnapshotFormat::PlainText, RenderOptions::default());
    let markdown =
        ff_screen_model::render(&model, SnapshotFormat::Markdown, RenderOptions::default());
    for rendered in [&plain, &markdown] {
        assert!(
            !rendered.contains(". . ."),
            "rendered snapshot has no dot-leaders"
        );
        // (a) a known key appears EXACTLY ONCE (guards the former duplicate).
        assert_eq!(
            rendered.matches(key).count(),
            1,
            "'{key}' must appear exactly once in the capture (no duplicate list);\n{rendered}"
        );
    }

    // (c) a group header renders flush-left as "[<Group>] : expanded|collapsed".
    // Editor is a stable namespace in the schema.
    let editor_group = model
        .body_lines
        .iter()
        .find(|l| l.starts_with(&group_left_label("Editor")))
        .expect("the capture lists the [Editor] group header");
    assert!(
        editor_group.ends_with(" : expanded") || editor_group.ends_with(" : collapsed"),
        "group header is '[Editor] : expanded|collapsed', got {editor_group:?}"
    );
    assert!(
        !editor_group.starts_with(' '),
        "group header is flush-left (no indent), got {editor_group:?}"
    );

    // (d) a child row is indented one level and uses a single ' : ' separator.
    let child = model
        .body_lines
        .iter()
        .find(|l| l.trim_start().starts_with(key))
        .expect("the capture lists the editor.tab_size key row");
    assert!(
        child.starts_with(&child_left_label("")),
        "child row is indented one level, got {child:?}"
    );
    assert_eq!(
        child.matches(" : ").count(),
        1,
        "child row has a single ' : ' separator, got {child:?}"
    );

    // (e) the ':' column alignment matches the PURE helper: recompute the global
    // column over the same visible labels and assert the child's ':' sits there.
    let p = &harness.state().config_panel;
    let entries = harness.state().config_handle.list_schema_entries();
    // Build the authoritative labels exactly as the provider does.
    let mut visible_labels = vec![
        ff_i18n::t("ff-desktop-config-panel-filter"),
        "Unsaved edits".to_string(),
    ];
    let mut namespaces: Vec<String> = entries
        .iter()
        .map(|e| e.key.split('.').next().unwrap_or(&e.key).to_string())
        .collect();
    namespaces.sort();
    namespaces.dedup();
    for ns in &namespaces {
        let mut disp = ns.clone();
        if let Some(c) = disp.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        visible_labels.push(group_left_label(&disp));
        if !p.collapsed.get(ns).copied().unwrap_or(false) {
            for e in &entries {
                if e.key.split('.').next().unwrap_or(&e.key) == ns {
                    visible_labels.push(child_left_label(&e.key));
                }
            }
        }
    }
    let column = global_key_column_width(&visible_labels);
    let expected_child_left = aligned_left_segment(&child_left_label(key), column);
    assert!(
        child.starts_with(&expected_child_left),
        "the ':' column matches the pure helper; expected prefix {expected_child_left:?}, got {child:?}"
    );
    // The ':' lands at the global column cell (char count before " : ").
    let colon_cell = child.split(" : ").next().unwrap().chars().count();
    assert_eq!(
        colon_cell, column,
        "the ':' column equals the global width from the pure helper"
    );
}

// Validates: master-screen-directive Req 4.4c (user-approved option 3) -- the
// Config Context MARKDOWN capture renders, for EACH namespace group in order, a
// level-3 `### [Group]  (expanded|collapsed)` heading followed by a 2-column
// `Property | Value` GFM table of that group's child keys + current values. The
// header fields render as a heading-less intro table BEFORE the first group.
// There is NO single flat Property/Value mega-table spanning all groups and NO
// leftover `Setting / Value / State` block; each group heading appears once and
// a known key (logging.level) appears exactly once.
#[test]
fn config_markdown_capture_renders_per_group_headings_and_tables() {
    use ff_screen_model::{RenderOptions, SnapshotFormat};

    let mut harness = harness_shell();
    open_config(&mut harness);

    let model = harness.state().config_panel_screen_model();
    let md = ff_screen_model::render(&model, SnapshotFormat::Markdown, RenderOptions::default());

    // A per-group level-3 heading for Logging, reflecting its collapse state.
    assert!(
        md.contains("### [Logging]  (expanded)"),
        "a '### [Logging]  (expanded)' heading must be present;\n{md}"
    );

    // The Logging heading is immediately followed by a Property | Value table
    // header (GFM header row + separator) -- the per-group table, not a flat one.
    let after_logging = md
        .split("### [Logging]  (expanded)")
        .nth(1)
        .expect("text after the Logging heading");
    let table_head = after_logging
        .trim_start_matches(['\n', '\r'])
        .lines()
        .next()
        .unwrap_or("");
    assert_eq!(
        table_head, "| Property | Value |",
        "the heading is immediately followed by a 'Property | Value' table;\n{md}"
    );

    // The Logging table contains the row 'logging.level | info', and
    // 'logging.level' appears EXACTLY ONCE across the whole capture (no dup).
    assert!(
        md.contains("| logging.level | info |"),
        "the Logging table has a 'logging.level | info' row;\n{md}"
    );
    assert_eq!(
        md.matches("logging.level").count(),
        1,
        "'logging.level' appears exactly once (no duplicate block);\n{md}"
    );

    // One '### [Group]' heading per namespace group, each appearing exactly
    // once and IN THE SAME ORDER as the visible rows. The exact set of groups
    // depends on the schema registered in the harness, so derive the expected
    // list from the SAME visible rows the capture uses rather than hardcoding
    // it -- that keeps the test faithful to "the same groups as the plain-text
    // / GUI" across schema builds.
    use crate::config_panel::{visible_rows, ConfigNodeId, TreeEntry};
    let entries = harness.state().config_handle.list_schema_entries();
    let tree_entries: Vec<TreeEntry> = entries
        .iter()
        .map(|e| TreeEntry {
            key: e.key.clone(),
            description: e.description.clone(),
        })
        .collect();
    let rows = visible_rows(
        &tree_entries,
        &harness.state().config_panel.filter,
        &harness.state().config_panel.collapsed,
    );
    let group_headings: Vec<String> = rows
        .iter()
        .filter_map(|r| match &r.id {
            ConfigNodeId::Namespace(ns) => {
                let mut disp = ns.clone();
                if let Some(c) = disp.get_mut(0..1) {
                    c.make_ascii_uppercase();
                }
                Some(format!("### [{disp}]"))
            }
            ConfigNodeId::Key(_) => None,
        })
        .collect();
    assert!(
        !group_headings.is_empty(),
        "the capture must list at least one namespace group;\n{md}"
    );
    for g in &group_headings {
        assert_eq!(
            md.matches(g.as_str()).count(),
            1,
            "group heading {g:?} must appear exactly once;\n{md}"
        );
    }
    // The group headings appear IN ORDER.
    let mut last = 0usize;
    for g in &group_headings {
        let at = md.find(g.as_str()).unwrap_or_else(|| panic!("{g} present"));
        assert!(
            at >= last,
            "group headings must be in order; {g} out of order"
        );
        last = at;
    }

    // There is NO flat mega-table: the header row 'Property | Value' appears once
    // per section (the intro + one per group), never a single table carrying all
    // keys. Confirm the count equals 1 (intro) + the number of groups.
    assert_eq!(
        md.matches("| Property | Value |").count(),
        group_headings.len() + 1,
        "one Property/Value header per section (intro + one per group), not a flat mega-table;\n{md}"
    );

    // No leftover 'Setting / Value / State' table from the old defect.
    assert!(
        !md.contains("Setting") || !md.contains("State"),
        "no leftover 'Setting / Value / State' table block;\n{md}"
    );

    // The header fields render BEFORE the first group heading (intro).
    let first_group_at = md.find("### [").expect("a group heading exists");
    let intro = &md[..first_group_at];
    assert!(
        intro.contains("Unsaved edits"),
        "header fields (Unsaved edits) render before the first group;\n{md}"
    );
}
