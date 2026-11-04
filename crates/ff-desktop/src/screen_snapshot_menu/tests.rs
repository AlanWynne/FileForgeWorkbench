//! Builder-level tests for the Menu Kind snapshot content (CR-NR-117).

use super::*;
use crate::menu_workspace::defaults::{
    recovery_pom_menu, recovery_scrm_menu, recovery_settings_menu,
};
use crate::menu_workspace::MenuWorkspaceState;
use crate::screen_snapshot::{
    calendar_right_column, menu_workspace_screen_model_full, render_snapshot, MenuScreenChrome,
};
use ff_screen_model::SnapshotFormat;

fn row(key: &str, command: &str, description: &str, group: Option<&str>) -> MenuRowDisplay {
    MenuRowDisplay {
        key: key.to_string(),
        command: command.to_string(),
        description: description.to_string(),
        group: group.map(str::to_string),
    }
}

fn state_for(menu: crate::menu_workspace::MenuFile) -> MenuWorkspaceState {
    let mut state = MenuWorkspaceState::load("/nonexistent/menu.toml");
    state.menu = Some(menu);
    state.load_error = None;
    state
}

fn calendar_chrome() -> MenuScreenChrome {
    MenuScreenChrome {
        right_column: calendar_right_column(2026, 11, 4, 12, 55, 308),
        rows: Vec::new(),
    }
}

// Validates: Requirement 23.2 -- no `(<command>)` bracket suffix.
#[test]
fn menu_row_has_no_bracketed_command_suffix() {
    let rows = [row("0", "Settings", "FFWB Settings", Some("Core"))];
    let lines = format_menu_rows(&rows, GroupSeparator::Space, false);
    assert_eq!(lines, vec!["0 Settings : FFWB Settings".to_string()]);
}

// Validates: Requirement 23.3 -- colon aligns to the widest key and command.
#[test]
fn menu_rows_align_colon_to_widest_key_and_command() {
    let rows = [
        row("S", "SNAPSHOT", "a", None),
        row("CS", "CAPTURE START", "b", None),
    ];
    let lines = format_menu_rows(&rows, GroupSeparator::Space, false);
    let colons: Vec<usize> = lines.iter().map(|l| l.find(" : ").unwrap()).collect();
    assert_eq!(colons[0], colons[1]);
    assert_eq!(lines[0], "S  SNAPSHOT      : a");
}

// Validates: Requirement 23.5 -- key equal to command still shows both.
#[test]
fn menu_row_with_key_equal_command_shows_both() {
    let rows = [row("X", "X", "Collapse", None)];
    assert_eq!(
        format_menu_rows(&rows, GroupSeparator::Space, false),
        vec!["X X : Collapse".to_string()]
    );
}

// Validates: Requirement 23.6 -- the calendar column is widest row + 4 cells.
#[test]
fn menu_calendar_block_starts_at_widest_row_plus_four() {
    let state = state_for(recovery_pom_menu());
    let model = menu_workspace_screen_model_full(&state, &calendar_chrome());
    let rows = rows_from_menu(state.menu.as_ref().unwrap());
    let widest = format_menu_rows(&rows, GroupSeparator::Space, false)
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap();
    let month = model
        .body_lines
        .iter()
        .find(|l| l.contains("November 2026"))
        .expect("month header");
    assert_eq!(month.find("November 2026").unwrap(), widest + 4);
}

// Validates: Requirement 23.6 -- no calendar means no right padding.
#[test]
fn menu_without_calendar_has_no_right_padding() {
    let lines = vec!["a : b".to_string(), "cc : d".to_string()];
    assert_eq!(compose_with_calendar(&lines, &[]), lines);
}

// Validates: Requirement 23.7 -- a line separator is emitted between groups only.
#[test]
fn menu_group_line_separator_emits_rule_between_groups_only() {
    let rows = [
        row("A", "One", "x", Some("Core")),
        row("B", "Two", "y", Some("Core")),
        row("R", "Three", "z", Some("Recovery")),
    ];
    let lines = format_menu_rows(&rows, GroupSeparator::Line, false);
    assert_eq!(lines.len(), 4);
    assert!(lines[2].chars().all(|c| c == '-'));
    assert_eq!(lines[2].len(), lines[0].chars().count());
}

// Validates: Requirement 23.7 -- space gives an empty row and none gives nothing.
#[test]
fn menu_group_space_and_none_separators() {
    let rows = [
        row("A", "One", "x", Some("G1")),
        row("B", "Two", "y", Some("G2")),
    ];
    let space = format_menu_rows(&rows, GroupSeparator::Space, false);
    assert_eq!(space.len(), 3);
    assert!(space[1].is_empty());
    assert_eq!(
        format_menu_rows(&rows, GroupSeparator::None, false).len(),
        2
    );
}

// Validates: Requirement 23.7 -- group headers precede the first option of a group.
#[test]
fn menu_group_headers_precede_first_option_of_group() {
    let rows = [
        row("A", "One", "x", Some("G1")),
        row("B", "Two", "y", Some("G1")),
        row("C", "Three", "z", Some("G2")),
    ];
    let lines = format_menu_rows(&rows, GroupSeparator::None, true);
    assert_eq!(lines[0], "G1");
    assert_eq!(lines[3], "G2");
    assert_eq!(lines.len(), 5);
}

// Validates: Requirement 23.10, 23.11 -- one Option/Command/Description section.
#[test]
fn menu_model_has_single_option_command_description_section() {
    let state = state_for(recovery_pom_menu());
    let model = menu_workspace_screen_model_full(&state, &calendar_chrome());
    assert_eq!(model.markdown_sections.len(), 1);
    let section = &model.markdown_sections[0];
    assert!(section.heading.is_none());
    assert_eq!(
        section.table.headers,
        vec!["Option", "Command", "Description"]
    );
    assert_eq!(section.table.rows.len(), 4);
    assert_eq!(
        section.table.rows[0],
        vec!["0", "Settings", "FFWB Settings and Client Parameters"]
    );
}

// Validates: Requirement 23.12 -- the table has no calendar and no group rows.
#[test]
fn menu_table_excludes_calendar_and_group_rows() {
    let state = state_for(recovery_settings_menu());
    let model = menu_workspace_screen_model_full(&state, &calendar_chrome());
    let table = &model.markdown_sections[0].table;
    assert_eq!(table.rows.len(), 7);
    let flat = table.rows.concat().join(" ");
    assert!(!flat.contains("November") && !flat.contains("Time") && !flat.contains("---"));
}

// Validates: Requirement 23.13 -- single representation: no fields, no tables.
#[test]
fn menu_model_fields_and_tables_are_empty() {
    let state = state_for(recovery_scrm_menu());
    let model = menu_workspace_screen_model_full(&state, &calendar_chrome());
    assert!(model.fields.is_empty());
    assert!(model.tables.is_empty());
}

// Validates: Requirement 23.10 -- a pipe in a description is escaped by the renderer.
#[test]
fn menu_table_escapes_pipe_in_description() {
    let rows = [row("A", "One", "a | b", None)];
    let mut model = ff_screen_model::ScreenModel::new("T");
    model
        .markdown_sections
        .push(menu_table_section(&rows).unwrap());
    let md = render_snapshot(&model, SnapshotFormat::Markdown);
    assert!(md.contains("| A | One | a \\| b |"), "{md}");
}

// Validates: Requirement 23.9 -- load error and empty menu emit no table.
#[test]
fn menu_load_error_and_empty_menu_emit_no_table() {
    let broken = MenuWorkspaceState::load("/nonexistent/menu.toml");
    let model = menu_workspace_screen_model_full(&broken, &calendar_chrome());
    assert!(model.markdown_sections.is_empty());
    assert!(menu_table_section(&[]).is_none());
}

// Validates: Requirement 23.10, 23.12 -- Markdown has box then table, HTML has
// the Option header.
#[test]
fn menu_markdown_has_box_then_table_and_html_has_option_header() {
    let state = state_for(recovery_pom_menu());
    let model = menu_workspace_screen_model_full(&state, &calendar_chrome());
    let md = render_snapshot(&model, SnapshotFormat::Markdown);
    let fence_end = md.rfind("```").expect("fenced box");
    let table = md
        .find("| Option | Command | Description |")
        .expect("table");
    assert!(fence_end < table, "box must precede the table:\n{md}");
    let html = ff_screen_model::render_tables_html(&model);
    assert!(html.contains("<th>Option</th>"), "{html}");
}

// Validates: Requirement 23.1 -- the content area is the aligned rows only (no
// bracket suffix, no Command row) for a plain-text box.
#[test]
fn menu_plain_text_box_has_aligned_rows_without_suffix() {
    let state = state_for(recovery_pom_menu());
    let model = menu_workspace_screen_model_full(&state, &MenuScreenChrome::default());
    let text = render_snapshot(&model, SnapshotFormat::PlainText);
    assert!(
        text.contains("S SCRM     : Screen Capture and Recording"),
        "{text}"
    );
    assert!(!text.contains("(SCRM)"), "{text}");
}

// Validates: Requirement 23.2 -- shell-supplied rows (the displayed surfaces)
// drive both the box and the table Command cell.
#[test]
fn menu_command_cell_is_displayed_surface() {
    let state = state_for(recovery_pom_menu());
    let chrome = MenuScreenChrome {
        right_column: Vec::new(),
        rows: vec![
            row("0", "Instellings", "Desc", Some("Core")),
            row("3", "Hulp", "Desc2", Some("Core")),
            row("S", "SCRM", "Desc3", Some("Core")),
            row("X", "X", "Desc4", Some("Core")),
        ],
    };
    let model = menu_workspace_screen_model_full(&state, &chrome);
    assert_eq!(model.markdown_sections[0].table.rows[0][1], "Instellings");
    assert!(model.body_lines[0].starts_with("0 Instellings"));
}
