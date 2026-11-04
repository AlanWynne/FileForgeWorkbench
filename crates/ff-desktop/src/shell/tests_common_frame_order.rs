//! Common-frame ORDER and CONTAINMENT tests (CR-NR-115 defect fix). The earlier
//! common-frame snapshot tests only asserted each chrome element was PRESENT;
//! they missed that the real renderer appended the chrome AFTER the content and
//! left the `Command ===>` line and the status bar OUTSIDE the box. These tests
//! assert the POSITION of every element and that NOTHING sits below the closing
//! border, for a markdown_sections Context (Config) and a table Context (Alias).
//!
//! Required top-to-bottom order (workspace-framework Req 7.1-7.10,
//! screen-snapshot-scrm Req 30.1-30.11): window title bar, menu bar, tab bar,
//! workspace title (centered), `Command ===>` line, CONTENT, message line (only
//! when present), status bar, function-key line (only when pfshow on).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;

/// Render the ACTIVE Context through the SNAPSHOT text seam.
fn snapshot_text(shell: &super::WorkbenchShell) -> String {
    shell
        .snapshot_text_for_active("")
        .expect("the active Context must be capturable")
        .0
}

/// Render the ACTIVE Context through the CAPTURE seam (`active_screen_model`).
fn capture_text(shell: &super::WorkbenchShell) -> String {
    let model = shell
        .active_screen_model()
        .expect("the active Context must be capturable");
    crate::screen_snapshot::render_snapshot(&model, ff_screen_model::SnapshotFormat::PlainText)
}

fn open_and_settle(harness: &mut egui_kittest::Harness<'_, super::WorkbenchShell>, cmd: &str) {
    harness.state_mut().dispatch_command_string(cmd);
    for _ in 0..4 {
        harness.run();
    }
}

/// The text between the vertical frame glyphs of a row, trimmed.
fn inner(line: &str) -> String {
    line.trim_matches(|c: char| c == '\u{2502}' || c.is_whitespace())
        .to_string()
}

/// Index of the ONLY line satisfying `pred`; panics when absent or repeated.
fn only_index(lines: &[&str], what: &str, text: &str, pred: impl Fn(&str) -> bool) -> usize {
    let hits: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| pred(l))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "{what} must appear exactly once, found at {hits:?};\n{text}"
    );
    hits[0]
}

/// What a scenario expects to find in the framed capture.
struct Expect<'a> {
    title: &'a str,
    tab_tag: &'a str,
    content_marker: &'a str,
    message: Option<&'a str>,
    key_line: bool,
}

/// Assert order + containment + single-chrome for one rendered capture.
fn assert_frame_layout(text: &str, e: &Expect<'_>) {
    let lines: Vec<&str> = text.lines().collect();

    // Containment: exactly one top border (first line) and exactly one bottom
    // border, and the bottom border is the LAST non-blank line (nothing below).
    assert!(
        lines[0].starts_with('\u{250C}'),
        "frame opens on line 0;\n{text}"
    );
    let bottoms: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with('\u{2514}'))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(bottoms.len(), 1, "exactly one closing border;\n{text}");
    let last_non_blank = lines.iter().rposition(|l| !l.trim().is_empty()).unwrap();
    assert_eq!(
        bottoms[0], last_non_blank,
        "no non-blank line may follow the closing border;\n{text}"
    );
    // Every non-blank line lies INSIDE the frame (starts with a frame glyph).
    for (i, l) in lines.iter().enumerate() {
        if l.trim().is_empty() {
            continue;
        }
        let first = l.chars().next().unwrap();
        assert!(
            "\u{250C}\u{2502}\u{251C}\u{2514}".contains(first),
            "line {i} is outside the frame: {l:?};\n{text}"
        );
    }

    // Each chrome element appears exactly once, at a strictly increasing index.
    let title_bar = only_index(&lines, "window title bar", text, |l| {
        l.contains(crate::screen_snapshot::WINDOW_TITLE)
    });
    let menu_bar = only_index(&lines, "menu bar", text, |l| {
        // The menu items are separate cells divided by the vertical glyph.
        l.contains("Settings") && l.contains("Help") && l.contains("SCRM")
    });
    let tab_bar = only_index(&lines, "tab bar", text, |l| l.contains(e.tab_tag));
    let ws_title = only_index(&lines, "workspace title", text, |l| inner(l) == e.title);
    let command = only_index(&lines, "Command line", text, |l| l.contains("Command ===>"));
    // The content marker may repeat (a field row AND its grid cell), so take the
    // FIRST occurrence: that is the first content line.
    let content = lines
        .iter()
        .position(|l| l.contains(e.content_marker))
        .unwrap_or_else(|| panic!("content marker {:?} missing;\n{text}", e.content_marker));
    let status = only_index(&lines, "status bar", text, |l| l.contains("Last:"));
    assert!(title_bar < menu_bar, "title bar < menu bar;\n{text}");
    assert!(menu_bar < tab_bar, "menu bar < tab bar;\n{text}");
    assert!(tab_bar < ws_title, "tab bar < workspace title;\n{text}");
    assert!(
        ws_title < command,
        "workspace title < Command line;\n{text}"
    );
    assert!(
        command < content,
        "Command line < first content line;\n{text}"
    );

    let mut floor = content;
    if let Some(msg) = e.message {
        let m = only_index(&lines, "message line", text, |l| l.contains(msg));
        assert!(m > floor, "message line is below the content;\n{text}");
        assert!(m < status, "message line is above the status bar;\n{text}");
        floor = m;
    } else {
        assert!(
            !text.contains("info one"),
            "no message line when none;\n{text}"
        );
    }
    assert!(status > floor, "status bar below content/message;\n{text}");
    assert!(status < bottoms[0], "status bar inside the frame;\n{text}");
    assert!(!text.contains("RUNNING"), "no RUNNING segment;\n{text}");

    let key_line = lines.iter().position(|l| l.contains("F1 Help"));
    if e.key_line {
        let k = key_line.unwrap_or_else(|| panic!("pfshow ON needs a key line;\n{text}"));
        assert!(k > status, "key line is below the status bar;\n{text}");
        assert!(k < bottoms[0], "key line inside the frame;\n{text}");
    } else {
        assert!(key_line.is_none(), "pfshow OFF omits the key line;\n{text}");
    }

    // The workspace title is CENTERED (left/right padding differ by <= 1) and is
    // not ALSO a box caption: the first row after the top border is the window
    // title bar, not the workspace title.
    let row: Vec<char> = lines[ws_title].chars().collect();
    let body: String = row[1..row.len() - 1].iter().collect();
    let left = body.len() - body.trim_start().len();
    let right = body.len() - body.trim_end().len();
    assert!(
        left.abs_diff(right) <= 1 && left > 1,
        "workspace title is centered (left {left}, right {right});\n{text}"
    );
    assert_eq!(
        title_bar, 1,
        "first row under the top border is the title bar;\n{text}"
    );
}

fn config_shell_with_state(
    message: Option<&str>,
    pfshow: bool,
) -> egui_kittest::Harness<'static, super::WorkbenchShell> {
    let mut harness = harness_shell();
    open_and_settle(&mut harness, "CONFIG");
    harness.state_mut().key_bar_visible = pfshow;
    if let Some(m) = message {
        harness.state_mut().message_line = vec![m.to_string()];
    }
    harness.run();
    harness
}

fn alias_shell_with_state(
    message: Option<&str>,
    pfshow: bool,
) -> egui_kittest::Harness<'static, super::WorkbenchShell> {
    let mut harness = harness_shell();
    open_and_settle(&mut harness, "CONFIG");
    open_and_settle(&mut harness, "ALIASES");
    assert_eq!(
        harness.state().tabs.active_tab().kind.tag(),
        KindTag::AliasEditor
    );
    harness.state_mut().alias_editor_panel.rows = vec![alias_row("SS", "SNAPSHOT")];
    harness.state_mut().key_bar_visible = pfshow;
    if let Some(m) = message {
        harness.state_mut().message_line = vec![m.to_string()];
    }
    harness.run();
    harness
}

fn alias_row(surface: &str, canonical: &str) -> crate::alias_editor_panel::AliasRowEdit {
    use crate::shell::user_aliases::AliasEnvironment;
    crate::alias_editor_panel::AliasRowEdit {
        act: String::new(),
        surface: surface.to_string(),
        canonical: canonical.to_string(),
        environment: AliasEnvironment::Ffcmd,
        command_template: String::new(),
        description: String::new(),
    }
}

// Validates: screen-snapshot-scrm Requirement 30.1-30.11 / workspace-framework
// Requirement 7.1-7.10 -- a markdown_sections Context (Config) snapshot lays the
// frame out top-to-bottom (title bar, menu bar, tab bar, centered title,
// Command line, content, status bar, key line), all inside ONE box.
#[test]
fn config_snapshot_orders_chrome_and_keeps_everything_inside_the_frame() {
    let harness = config_shell_with_state(None, true);
    let text = snapshot_text(harness.state());
    assert_frame_layout(
        &text,
        &Expect {
            title: "Configuration",
            tab_tag: "[CONFIG]",
            content_marker: "Filter:",
            message: None,
            key_line: true,
        },
    );
}

// Validates: screen-snapshot-scrm Requirement 30.1-30.11 / workspace-framework
// Requirement 7.1-7.10 -- a TABLE Context (Alias) is laid out in the SAME order
// with its box-drawing grid INSIDE the frame (general scope, Req 30.15).
#[test]
fn alias_snapshot_orders_chrome_and_keeps_grid_inside_the_frame() {
    let harness = alias_shell_with_state(None, true);
    let text = snapshot_text(harness.state());
    assert_frame_layout(
        &text,
        &Expect {
            title: "Alias Editor",
            tab_tag: "[ALIASES]",
            content_marker: "SNAPSHOT",
            message: None,
            key_line: true,
        },
    );
    assert!(
        text.contains('\u{253C}') || text.contains('\u{252C}'),
        "the grid is still box-drawn;\n{text}"
    );
}

// Validates: screen-snapshot-scrm Requirement 30.8, 30.9, 30.11 -- a present
// message line sits between the content and the status bar, and pfshow OFF
// removes the key line (Config).
#[test]
fn config_message_line_sits_between_content_and_status_and_pfshow_off_drops_key_line() {
    let harness = config_shell_with_state(Some("info one"), false);
    let text = snapshot_text(harness.state());
    assert_frame_layout(
        &text,
        &Expect {
            title: "Configuration",
            tab_tag: "[CONFIG]",
            content_marker: "Filter:",
            message: Some("info one"),
            key_line: false,
        },
    );
}

// Validates: screen-snapshot-scrm Requirement 30.8, 30.11, 30.15 -- the same
// message-line / key-line placement holds for the table Context (Alias).
#[test]
fn alias_message_line_sits_between_grid_and_status() {
    let harness = alias_shell_with_state(Some("info one"), true);
    let text = snapshot_text(harness.state());
    assert_frame_layout(
        &text,
        &Expect {
            title: "Alias Editor",
            tab_tag: "[ALIASES]",
            content_marker: "SNAPSHOT",
            message: Some("info one"),
            key_line: true,
        },
    );
}

// Validates: screen-snapshot-scrm Requirement 30.16 -- the CAPTURE seam and the
// SNAPSHOT seam share ONE path: byte-for-byte identical framed text for both a
// markdown_sections Context and a table Context.
#[test]
fn snapshot_and_capture_seams_are_byte_identical_for_config_and_alias() {
    let config = config_shell_with_state(None, true);
    assert_eq!(
        snapshot_text(config.state()),
        capture_text(config.state()),
        "Config: SNAPSHOT == CAPTURE"
    );
    let alias = alias_shell_with_state(None, true);
    assert_eq!(
        snapshot_text(alias.state()),
        capture_text(alias.state()),
        "Alias: SNAPSHOT == CAPTURE"
    );
}
