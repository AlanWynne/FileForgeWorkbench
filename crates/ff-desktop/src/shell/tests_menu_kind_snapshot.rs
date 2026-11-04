//! Menu Kind snapshot goldens and shell-level guards (CR-NR-117, menu-workspace
//! Requirement 23). Full-text goldens for the box (plain) and the Markdown flavor
//! (fenced box + `Option | Command | Description` table) of POM, Settings, the
//! SCRM menu and a user `menus/<name>.toml`. Every Menu Kind goes through the
//! SAME provider; nothing here branches on a menu name.
//!
//! Set `FFWB_UPDATE_GOLDEN=1` to (re)write the fixtures, then review them by hand.

use super::tests_common_frame_golden::{assert_golden, fixed_stamp};
use super::tests_common_frame_menu::{assert_menu_frame_shape, capture, pom_shell};
use crate::tab_state::KindTag;

const USER_MENU_TOML: &str = r#"title = "Reports"
show_calendar = false
group_separator = "space"
group_headers = true

[[options]]
key = "1"
command = "DAILY"
description = "Daily summary report"
group = "Standard"

[[options]]
key = "22"
command = "WEEKLY REPORT"
description = "Weekly roll-up | all teams"
group = "Standard"

[[options]]
key = "A"
command = "AUDIT"
description = "Audit trail (disabled)"
group = "Admin"
enabled = false
"#;

fn markdown(shell: &super::WorkbenchShell) -> String {
    shell
        .snapshot_text_for_active("MARKDOWN")
        .expect("menu Context must be capturable")
        .0
        .trim_end_matches('\n')
        .to_string()
}

/// Re-pin what dispatching a command restamps, so the golden is deterministic.
fn settle(shell: &mut super::WorkbenchShell) {
    shell.last_action_at = fixed_stamp();
    shell.command_text = "ss".to_string();
    shell.message_line.clear();
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::MenuWorkspace);
}

fn settings_shell() -> super::WorkbenchShell {
    let mut shell = pom_shell(false, None);
    shell.dispatch_command_string("SETTINGS");
    settle(&mut shell);
    shell
}

fn scrm_shell() -> super::WorkbenchShell {
    let mut shell = pom_shell(false, None);
    shell.dispatch_command_string("SCRM");
    settle(&mut shell);
    shell
}

fn user_shell() -> (super::WorkbenchShell, tempfile::TempDir) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).expect("mkdir");
    std::fs::write(menus.join("reports.toml"), USER_MENU_TOML).expect("write menu");
    let mut shell = pom_shell(false, None);
    shell.dir_overrides.menus = Some(menus);
    shell.dispatch_command_string("REPORTS");
    settle(&mut shell);
    (shell, dir)
}

// Validates: menu-workspace Requirement 23.21 -- POM Markdown flavor golden.
#[test]
fn menu_kind_pom_markdown_matches_golden() {
    let shell = pom_shell(true, Some(super::tests_common_frame_menu::CONFIRMATION));
    assert_golden("menu_pom_clipboard_markdown.md", &markdown(&shell));
}

// Validates: menu-workspace Requirement 23.21, 23.7 -- Settings box golden (group
// rule in the box) and Markdown flavor golden (rule not in the table).
#[test]
fn menu_kind_settings_box_and_markdown_match_golden() {
    let shell = settings_shell();
    let text = capture(&shell);
    assert_menu_frame_shape(&text, false, None);
    assert!(
        text.contains("-----"),
        "group rule is shown in the box:\n{text}"
    );
    let md = markdown(&shell);
    assert_golden("menu_settings_clipboard_markdown.md", &md);
    let rows = md.lines().filter(|l| l.starts_with("| ")).count();
    assert_eq!(
        rows,
        1 + 7,
        "header plus one row per option, no group rule row"
    );
}

// Validates: menu-workspace Requirement 23.21 -- SCRM menu box and Markdown goldens.
#[test]
fn menu_kind_scrm_box_and_markdown_match_golden() {
    let shell = scrm_shell();
    let text = capture(&shell);
    assert_menu_frame_shape(&text, false, None);
    assert_golden("menu_scrm_pfshow_off.txt", &text);
    assert_golden("menu_scrm_clipboard_markdown.md", &markdown(&shell));
}

// Validates: menu-workspace Requirement 23.1, 23.21 -- a user menu goes through
// the same path (genericity): box and Markdown goldens.
#[test]
fn menu_kind_user_menu_box_and_markdown_match_golden() {
    let (shell, _dir) = user_shell();
    let text = capture(&shell);
    assert_menu_frame_shape(&text, false, None);
    assert_golden("menu_user_reports_pfshow_off.txt", &text);
    let md = markdown(&shell);
    assert_golden("menu_user_reports_clipboard_markdown.md", &md);
    assert!(md.contains("a \\| b") || md.contains("Weekly roll-up \\| all teams"));
}

// Validates: menu-workspace Requirement 23.2 -- the snapshot rows are the SAME
// displayed surfaces the live panel reads (mnemonic, command, description).
#[test]
fn menu_snapshot_rows_use_live_displayed_surfaces() {
    let shell = settings_shell();
    let rows = shell.menu_row_displays(shell.tabs.active_tab().kind.menu_workspace().unwrap());
    let keys = shell.menu_option_mnemonic_surfaces();
    let commands = shell.menu_option_command_surfaces();
    assert_eq!(rows.len(), keys.len());
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row.key, keys[i]);
        assert_eq!(row.command, commands[i]);
    }
}

// Validates: menu-workspace Requirement 23.14 -- no Menu Kind reports workspace
// status cells and the captured status band is exactly Last | Prof | Zoom.
#[test]
fn menu_kind_reports_no_workspace_status_cells() {
    let (user, _dir) = user_shell();
    for shell in [pom_shell(false, None), settings_shell(), scrm_shell(), user] {
        assert!(shell.workspace_status_cells().is_empty());
        let text = capture(&shell);
        let band = text
            .lines()
            .find(|l| l.contains("Last: 11:02"))
            .expect("band");
        let cells: Vec<&str> = band
            .trim_matches('\u{2502}')
            .split('\u{2502}')
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        assert_eq!(cells.len(), 3, "Last | Prof | Zoom only: {band}");
    }
}
