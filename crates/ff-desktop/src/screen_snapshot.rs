//! Screen snapshot wiring for the desktop shell (CR-NR-098, Wave 1).
//!
//! This module bridges the shell's live Context state to the GUI-independent
//! [`ff_screen_model`] capture engine. It builds a logical [`ScreenModel`] from a
//! Context's state (so capture is copy/paste-able TEXT, never a raster) and
//! renders it to the requested [`SnapshotFormat`].
//!
//! Wave 1 provides the FIRST `ScreenProvider`: the Home Context (POM), whose
//! content is a data-driven Menu_Workspace. Later waves add providers for other
//! Contexts and the collection/replay wiring.
//!
//! Validates: screen-snapshot-scrm Requirement 2.1, 2.2, 3.1, 3.8, 4.1-4.6.

use ff_screen_model::{Field, RenderOptions, ScreenModel, SnapshotFormat};

use crate::menu_workspace::MenuWorkspaceState;

/// The full-screen chrome surrounding a Menu_Workspace's option list, built by
/// the shell from its live logical state (NOT from egui) and folded into the
/// snapshot so a POM/menu capture is the FULL screen a user sees, not a trimmed
/// option list (screen-snapshot-scrm Requirement 3.9-3.13).
///
/// Every field is plain, already-rendered selectable text. The PF-key footer is
/// carried in `key_bar_lines`, which the shell leaves EMPTY when the
/// Key_Label_Bar is hidden (pfshow off), so the capture reflects the live pfshow
/// state (Requirement 3.10).
#[derive(Debug, Default, Clone)]
pub struct MenuScreenChrome {
    /// The window chrome / OS title-bar text (Requirement 3.9).
    pub window_title: String,
    /// The menu-bar entries, in display order (Requirement 3.9).
    pub menu_bar: Vec<String>,
    /// The tab header tag, e.g. `[POM]` (Requirement 3.9).
    pub tab_header: String,
    /// The title line as the user sees it -- the descriptive display title
    /// (Requirement 3.9). May duplicate the model title; kept explicit so the
    /// capture records the on-screen title line.
    pub title_line: String,
    /// The calendar lines, when the menu shows a calendar (Requirement 3.9).
    /// Empty when the menu hides the calendar (`show_calendar = false`).
    pub calendar_lines: Vec<String>,
    /// The PF-key footer line(s), present ONLY when the Key_Label_Bar is shown
    /// (pfshow on). Empty when pfshow is off (Requirement 3.10).
    pub key_bar_lines: Vec<String>,
}

/// Build a logical [`ScreenModel`] from a Menu_Workspace's loaded state (the POM
/// and every other menu Context share this shape). Each menu option becomes a
/// [`Field`] whose label is the option key and whose value is
/// "description (command)". A load error is surfaced as a message so the capture
/// still reflects what the user sees.
///
/// `command_line` is the current `Command ===>` field content, threaded through
/// so the snapshot matches the live screen (Requirement 3.8).
///
/// Validates: Requirement 2.1, 3.1, 3.8.
pub fn menu_workspace_screen_model(state: &MenuWorkspaceState, command_line: &str) -> ScreenModel {
    let title = state.menu_title().unwrap_or_else(|| "Menu".to_string());
    let mut model = ScreenModel::new(title);
    model.command_line = command_line.to_string();

    match &state.menu {
        Some(menu) => {
            for opt in &menu.options {
                // label = key; value = "description (command)". Both are visible,
                // copy/paste-able text on the screen.
                let value = if opt.command.trim().is_empty() {
                    opt.description.clone()
                } else {
                    format!("{} ({})", opt.description, opt.command)
                };
                model = model.with_field(Field::new(opt.key.clone(), value));
            }
        }
        None => {
            let msg = state
                .load_error
                .clone()
                .unwrap_or_else(|| "Menu not loaded".to_string());
            model.messages.push(msg);
        }
    }
    model
}

/// Build the FULL-screen logical model for a Menu_Workspace: the option list
/// (from [`menu_workspace_screen_model`]) plus the surrounding chrome the user
/// sees -- window title bar, menu bar, tab header, title line (as header chrome),
/// and the calendar and PF-key footer (as footer chrome). The chrome is supplied
/// by the shell from its logical state (NOT read from egui), so the capture is a
/// faithful full screen (screen-snapshot-scrm Requirement 3.9-3.13).
///
/// Validates: Requirement 3.9, 3.10, 3.11, 3.12, 3.13.
pub fn menu_workspace_screen_model_full(
    state: &MenuWorkspaceState,
    command_line: &str,
    chrome: &MenuScreenChrome,
) -> ScreenModel {
    let mut model = menu_workspace_screen_model(state, command_line);

    // Header chrome, top-to-bottom: window title bar, menu bar, tab header,
    // title line (Requirement 3.9).
    if !chrome.window_title.is_empty() {
        model = model.with_header_line(chrome.window_title.clone());
    }
    if !chrome.menu_bar.is_empty() {
        model = model.with_header_line(chrome.menu_bar.join("   "));
    }
    if !chrome.tab_header.is_empty() {
        model = model.with_header_line(chrome.tab_header.clone());
    }
    if !chrome.title_line.is_empty() {
        model = model.with_header_line(chrome.title_line.clone());
    }

    // Footer chrome: the calendar (Requirement 3.9), then the PF-key footer
    // which is present only when pfshow is on (Requirement 3.10).
    for line in &chrome.calendar_lines {
        model = model.with_footer_line(line.clone());
    }
    for line in &chrome.key_bar_lines {
        model = model.with_footer_line(line.clone());
    }
    model
}

/// Build the compact text calendar lines for `year`/`month`, marking `today`
/// (1-based day-of-month, or 0 for none) with a leading `*`. The layout is a
/// month/year header, a weekday header, and one line per week -- the logical
/// content of the POM calendar, derived from the date (NOT egui).
///
/// Each day occupies a THREE-char cell so a leading `*` marker never eats a
/// digit: a plain day is right-aligned in three (`  3`, ` 25`) and today keeps
/// both digits with the marker in the spare column (`* 3`, `*25`). The weekday
/// header is widened to match so every column stays aligned.
///
/// Validates: screen-snapshot-scrm Requirement 3.9-3.13.
pub fn calendar_lines(year: i32, month: u32, today: u32) -> Vec<String> {
    use crate::primary_option_menu::{days_in_month, first_weekday_of_month, month_name};
    let mut lines = Vec::new();
    lines.push(format!("{} {}", month_name(month), year));
    lines.push(" Su  Mo  Tu  We  Th  Fr  Sa".to_string());

    let first = first_weekday_of_month(year, month) as usize;
    let total = days_in_month(year, month);
    let mut cells: Vec<String> = Vec::new();
    for _ in 0..first {
        cells.push("   ".to_string());
    }
    for day in 1..=total {
        if day == today {
            // Mark today (ASCII only). The `*` sits in the spare column so the
            // full day number is always preserved and the grid stays aligned.
            cells.push(format!("*{day:>2}"));
        } else {
            cells.push(format!("{day:>3}"));
        }
    }
    for week in cells.chunks(7) {
        lines.push(week.join(" "));
    }
    lines
}

/// Build the PF-key footer line for the active key map at the given scope,
/// matching the on-screen Key_Label_Bar: a leading scope segment then each
/// assigned `Fn label` pair. The caller passes this ONLY when the Key_Label_Bar
/// is shown (pfshow on); when hidden the caller omits it entirely so the capture
/// reflects the pfshow state (Requirement 3.10).
///
/// Validates: screen-snapshot-scrm Requirement 3.9, 3.10.
pub fn key_bar_line(
    key_map: &ff_keys::KeyMap,
    scope_segment: &str,
    modifier: ff_keys::KeyModifier,
) -> String {
    use ff_keys::KeyLabelBarModel;
    let row = KeyLabelBarModel::row_for_modifier(key_map, modifier);
    let mut parts: Vec<String> = vec![scope_segment.to_string()];
    for slot in row.slots() {
        if let Some(label) = &slot.label {
            parts.push(format!("{} {}", slot.key.display_name(), label));
        }
    }
    parts.join(" | ")
}

/// Render a [`ScreenModel`] to the requested format's text, ready for the
/// clipboard. Thin wrapper over [`ff_screen_model::render`] with default options
/// (Unicode box drawing).
///
/// Validates: Requirement 4.1-4.6.
pub fn render_snapshot(model: &ScreenModel, format: SnapshotFormat) -> String {
    ff_screen_model::render(model, format, RenderOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_menu(toml: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("tempfile");
        f.write_all(toml.as_bytes()).expect("write");
        f
    }

    fn pom_like() -> MenuWorkspaceState {
        let toml = r#"
title = "Primary Option Menu"
[[options]]
key = "1"
command = "FILES"
description = "File Explorer"
[[options]]
key = "0"
command = "SETTINGS"
description = "Settings"
"#;
        let f = write_menu(toml);
        MenuWorkspaceState::load(f.path())
    }

    // Validates: Requirement 2.1, 3.1 -- model carries the menu title + one
    // field per option with visible key/description/command text.
    #[test]
    fn menu_model_has_title_and_option_fields() {
        let state = pom_like();
        let model = menu_workspace_screen_model(&state, "");
        assert_eq!(model.title, "Primary Option Menu");
        assert_eq!(model.fields.len(), 2);
        assert_eq!(model.fields[0].label, "1");
        assert!(model.fields[0].value.contains("File Explorer"));
        assert!(model.fields[0].value.contains("FILES"));
    }

    // Validates: Requirement 3.8 -- the command line is threaded into the model.
    #[test]
    fn command_line_is_captured() {
        let state = pom_like();
        let model = menu_workspace_screen_model(&state, "SNAPSHOT");
        assert_eq!(model.command_line, "SNAPSHOT");
    }

    // Validates: Requirement 2.1 -- a load error surfaces as a message.
    #[test]
    fn load_error_surfaces_as_message() {
        let state = MenuWorkspaceState::load("/nonexistent/menu.toml");
        let model = menu_workspace_screen_model(&state, "");
        assert!(!model.messages.is_empty());
    }

    // Validates: Requirement 1.1, 4.2 -- rendered snapshot is selectable text
    // containing the option descriptions.
    #[test]
    fn render_snapshot_contains_option_text() {
        let state = pom_like();
        let model = menu_workspace_screen_model(&state, "");
        let text = render_snapshot(&model, SnapshotFormat::PlainText);
        assert!(text.contains("Primary Option Menu"));
        assert!(text.contains("File Explorer"));
        assert!(text.contains("Settings"));
    }

    fn sample_chrome(with_keys: bool) -> MenuScreenChrome {
        MenuScreenChrome {
            window_title: "FileForge Workbench".to_string(),
            menu_bar: vec!["Settings".to_string(), "Help".to_string()],
            tab_header: "[POM]".to_string(),
            title_line: "Primary Option Menu".to_string(),
            calendar_lines: vec!["October 2026".to_string(), "Su Mo Tu We".to_string()],
            key_bar_lines: if with_keys {
                vec!["Base | F1 Help | F3 Exit".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    // Validates: Requirement 3.9, 3.12, 3.13 -- the full model carries the
    // window title bar, menu bar, tab header, title line as header chrome, and
    // keeps the option rows with their command values.
    #[test]
    fn full_model_includes_chrome_and_option_rows() {
        let state = pom_like();
        let model = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));
        // Option rows (with command values) are preserved.
        assert_eq!(model.fields.len(), 2);
        assert!(model.fields[0].value.contains("FILES"));
        // Header chrome: title bar, menu bar, tab header, title line.
        let header = model.header_lines.join("\n");
        assert!(header.contains("FileForge Workbench"));
        assert!(header.contains("Settings"));
        assert!(header.contains("Help"));
        assert!(header.contains("[POM]"));
        assert!(header.contains("Primary Option Menu"));
    }

    // Validates: Requirement 3.9 -- the calendar appears in the rendered
    // snapshot when the menu shows it.
    #[test]
    fn full_snapshot_text_contains_calendar() {
        let state = pom_like();
        let model = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));
        let text = render_snapshot(&model, SnapshotFormat::PlainText);
        assert!(text.contains("October 2026"), "calendar month must appear");
    }

    // Validates: screen-snapshot-scrm Req 3.9-3.13 -- a DOUBLE-DIGIT today
    // keeps its full day number in the capture (regression: the old marker
    // truncated `*25` to `*2`), today is marked, and every column aligns.
    #[test]
    fn calendar_double_digit_today_keeps_full_day_and_aligns() {
        // October 2026, today = the 25th.
        let lines = calendar_lines(2026, 10, 25);
        let text = lines.join("\n");
        // The full day number is preserved and marked.
        assert!(
            text.contains("*25"),
            "double-digit today must keep both digits and the marker:\n{text}"
        );
        // The marker never produced a corrupted `*2` standalone cell.
        for line in &lines[2..] {
            for cell in line.split(' ').filter(|c| !c.is_empty()) {
                assert_ne!(cell, "*2", "today marker must not truncate the day number");
            }
        }
        // Every grid row is column-aligned: header + each week line share width.
        let header = &lines[1];
        for week in &lines[2..] {
            assert_eq!(
                week.chars().count(),
                header.chars().count(),
                "week row must align with the weekday header:\n{text}"
            );
        }
    }

    // Validates: screen-snapshot-scrm Req 3.9-3.13 -- a SINGLE-DIGIT today still
    // renders its correct number, is marked, and the grid stays aligned.
    #[test]
    fn calendar_single_digit_today_renders_correctly_and_aligns() {
        // October 2026, today = the 3rd.
        let lines = calendar_lines(2026, 10, 3);
        let text = lines.join("\n");
        assert!(
            text.contains("* 3"),
            "single-digit today must be marked with its number:\n{text}"
        );
        // Day 3 must never appear corrupted (e.g. truncated to just `*`).
        let header = &lines[1];
        for week in &lines[2..] {
            assert_eq!(
                week.chars().count(),
                header.chars().count(),
                "week row must align with the weekday header:\n{text}"
            );
        }
    }

    // Validates: Requirement 3.10 -- the PF-key footer line is present when
    // pfshow is on and ABSENT when pfshow is off.
    #[test]
    fn full_snapshot_pfkey_footer_follows_pfshow_state() {
        let state = pom_like();

        let on = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));
        let on_text = render_snapshot(&on, SnapshotFormat::PlainText);
        assert!(
            on_text.contains("F1 Help"),
            "pfshow ON must include the PF-key footer"
        );

        let off = menu_workspace_screen_model_full(&state, "", &sample_chrome(false));
        let off_text = render_snapshot(&off, SnapshotFormat::PlainText);
        assert!(
            !off_text.contains("F1 Help"),
            "pfshow OFF must omit the PF-key footer"
        );
    }
}
