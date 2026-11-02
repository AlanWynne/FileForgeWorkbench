//! GROUP A capture-fidelity: editor Context snapshots are FAITHFUL -- real
//! field VALUES (A1: Theme hex), and rendered as proper TABLES (A2:
//! Theme/Alias/Menu/Keys/Kinds/Config populate `model.tables` with the editor's
//! real columns + real current values). These are provider-only changes that
//! reuse `ff_screen_model::Table`; the renderers already emit tables.
//!
//! Validates: screen-snapshot-scrm Requirement 27.8; master-screen-directive
//! Requirement 4.4a.

#![allow(unused_imports)]
use super::tests_common::*;
use crate::shell::user_aliases::AliasEnvironment;
use crate::tab_state::KindTag;

// === A1 -- Theme snapshot carries the REAL hex value per token ==============

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (A1) -- the Theme Editor provider populates each token's REAL hex colour value
// (e.g. `#...`) read from the working palette, not a blank string (the former
// `Field::new(label, String::new())` bug, findings 1b).
#[test]
fn theme_screen_model_has_real_hex_values_not_blank() {
    let mut shell = make_shell();
    shell.dispatch_command_string("THEME");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ThemeEditor,
        "precondition: THEME opens the Theme Editor"
    );

    let model = shell.theme_editor_screen_model();

    // Every token field must carry a non-empty hex VALUE starting with '#'.
    let value_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|f| !f.label.is_empty())
        .collect();
    assert!(
        !value_fields.is_empty(),
        "the Theme capture must list the editable tokens"
    );
    assert!(
        value_fields.iter().all(|f| f.value.starts_with('#')),
        "every token value must be a real hex colour (A1), none blank; got {:?}",
        value_fields
            .iter()
            .map(|f| (&f.label, &f.value))
            .collect::<Vec<_>>()
    );
}

// === A2 -- each tabular editor populates model.tables =======================

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (A2) -- the Theme provider populates a [Setting, Value] table, one row per
// token, with the real hex value.
#[test]
fn theme_screen_model_populates_a_setting_value_table() {
    let mut shell = make_shell();
    shell.dispatch_command_string("THEME");
    let model = shell.theme_editor_screen_model();
    let table = model
        .tables
        .first()
        .expect("Theme capture populates a table (A2)");
    assert_eq!(
        table.headers,
        vec!["Setting".to_string(), "Value".to_string()]
    );
    assert!(!table.rows.is_empty(), "the Theme table has token rows");
    // Every value cell is a real hex colour.
    assert!(
        table
            .rows
            .iter()
            .all(|r| r.len() == 2 && r[1].starts_with('#')),
        "every Theme table row carries a real hex value, got {:?}",
        table.rows
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (A2) -- the Alias provider populates a table matching the alias editor's real
// columns [Act, CE, Command, Alias, Parameters] and includes a real saved alias
// row's actual command/alias text.
#[test]
fn alias_screen_model_populates_table_with_real_saved_row() {
    let mut shell = make_shell();
    let _dir = tempfile::TempDir::new().expect("tempdir");
    shell.dir_overrides.user_aliases = Some(_dir.path().join("aliases"));
    shell.dispatch_command_string("ALIASES");
    // A real saved alias the owner would have created: surface GG -> root GSEARCH.
    shell.alias_editor_panel.rows = vec![alias_row("GG", "GSEARCH", AliasEnvironment::Ffcmd, "")];

    let model = shell.alias_editor_screen_model();
    let table = model
        .tables
        .first()
        .expect("Alias capture populates a table (A2)");
    assert_eq!(
        table.headers,
        vec![
            "Act".to_string(),
            "CE".to_string(),
            "Command".to_string(),
            "Alias".to_string(),
            "Parameters".to_string(),
            "Description".to_string(),
        ]
    );
    // The saved row's real Alias + Command text is present.
    let row = table
        .rows
        .iter()
        .find(|r| r.contains(&"GG".to_string()))
        .expect("the saved GG alias row is captured");
    assert!(
        row.contains(&"GSEARCH".to_string()),
        "the row carries the real root command, got {row:?}"
    );
    assert!(
        row.contains(&"ffcmd".to_string()),
        "the row carries the real CE, got {row:?}"
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (A2) -- the Menus provider populates a table with the real MenuOption columns
// [Option, Command, Description, Group, On, Bar] with the real current menu's
// values.
#[test]
fn menu_screen_model_populates_table_with_real_option_columns() {
    let (shell, _dir) = make_shell_with_menus_editor();
    let model = shell.menus_editor_screen_model();
    let table = model
        .tables
        .first()
        .expect("Menus capture populates a table (A2)");
    assert_eq!(
        table.headers,
        vec![
            "Option".to_string(),
            "Command".to_string(),
            "Description".to_string(),
            "Group".to_string(),
            "On".to_string(),
            "Bar".to_string(),
        ]
    );
    assert!(
        !table.rows.is_empty(),
        "the POM working menu has real option rows"
    );
    // Every row has one cell per header column.
    assert!(
        table.rows.iter().all(|r| r.len() == 6),
        "every menu row has six column cells, got {:?}",
        table.rows
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (A2) -- the Keys provider populates a table of the real per-key assignments
// (one row per F-key, a column per modifier layer) with real values.
#[test]
fn keys_screen_model_populates_table_with_real_assignments() {
    let mut shell = make_shell();
    shell.dispatch_command_string("KEYS");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "precondition: KEYS opens the Keys Editor"
    );
    let model = shell.keys_editor_screen_model();
    let table = model
        .tables
        .first()
        .expect("Keys capture populates a table (A2)");
    assert_eq!(
        table.headers,
        vec![
            "Key".to_string(),
            "Base".to_string(),
            "Shift".to_string(),
            "Ctrl".to_string(),
            "Alt".to_string(),
        ]
    );
    assert_eq!(table.rows.len(), 12, "one row per F1-F12");
    // F1 Base is HELP in the default map -- a real value, not blank.
    let f1 = table
        .rows
        .iter()
        .find(|r| r.first().map(|c| c == "F1").unwrap_or(false))
        .expect("an F1 row exists");
    assert_eq!(f1[1], "HELP", "F1 Base carries the real HELP binding");
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (A2) -- the Kinds provider populates a table of the real kind rows with real
// values (name / modelled-on / title).
#[test]
fn kinds_screen_model_populates_table_with_real_kind_rows() {
    let mut shell = make_shell();
    shell.dispatch_command_string("KINDS");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KindsEditor,
        "precondition: KINDS opens the Kinds Editor"
    );
    let model = shell.kinds_editor_screen_model();
    let table = model
        .tables
        .first()
        .expect("Kinds capture populates a table (A2)");
    assert_eq!(
        table.headers,
        vec![
            "Kind".to_string(),
            "Modelled on".to_string(),
            "Title".to_string(),
            "Menu bar".to_string(),
            "Key list".to_string(),
        ]
    );
    assert!(
        !table.rows.is_empty(),
        "the Kinds table lists the real kind rows"
    );
    // The built-in "editor" kind is present with its real default title.
    let editor = table
        .rows
        .iter()
        .find(|r| r.first().map(|c| c == "editor").unwrap_or(false))
        .expect("the built-in editor kind row is captured");
    assert_eq!(
        editor[2], "[EDITOR]",
        "the editor kind carries its real default title"
    );
}

// Validates: screen-snapshot-scrm Req 27.8 / master-screen-directive Req 4.4a
// (Config Bug 3) -- the Config capture is a SINGLE representation that mirrors
// the visible render: the real settings rows are emitted as pre-composed
// body_lines, NOT as a second Settings/Setting-Value-State table (which
// duplicated the list in the pasted text). The provider must therefore emit NO
// table at all.
//
// Validates: screen-snapshot-scrm Req 27.8, 27.9; master-screen-directive Req
// 4.4b.
#[test]
fn config_screen_model_emits_single_body_line_tree_not_duplicate_table() {
    let mut shell = make_shell();
    shell.dispatch_command_string("CONFIG");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "precondition: CONFIG opens the Config panel"
    );
    let model = shell.config_panel_screen_model();
    // NO duplicate Settings table block (Bug 3): the former second list is gone.
    assert!(
        model.tables.is_empty(),
        "the Config capture must NOT emit a duplicate settings table, got {:?}",
        model.tables
    );
    // The real settings rows live in body_lines as a single representation, with
    // at least one bracketed namespace group header flush-left.
    assert!(
        model.body_lines.iter().any(|l| {
            l.starts_with('[') && (l.ends_with("expanded") || l.ends_with("collapsed"))
        }),
        "the real namespace group rows are captured as body_lines, got {:?}",
        model.body_lines
    );
}

/// Build a plain alias row for the tests (mirrors the alias editor's row shape).
fn alias_row(
    surface: &str,
    canonical: &str,
    environment: AliasEnvironment,
    template: &str,
) -> crate::alias_editor_panel::AliasRowEdit {
    crate::alias_editor_panel::AliasRowEdit {
        act: String::new(),
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment,
        command_template: template.to_string(),
        description: String::new(),
    }
}
