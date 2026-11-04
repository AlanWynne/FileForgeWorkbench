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

use ff_screen_model::{RenderOptions, ScreenModel, SnapshotFormat};

use crate::menu_workspace::MenuWorkspaceState;

/// The one live Workbench window title (the brand token). This is the SINGLE
/// source of the title text: `main.rs` references it for the OS window title and
/// the common-frame chrome helper (`shell::common_frame`) references it for the
/// in-frame window title bar, so the two can never drift (CR-NR-115, Req 30.2).
/// It is a brand name and is deliberately NOT translated (documentation-charset:
/// ASCII only).
pub const WINDOW_TITLE: &str = "FileForge Workbench";

/// The full-screen chrome surrounding a Menu_Workspace's option list, built by
/// the shell from its live logical state (NOT from egui) and folded into the
/// snapshot so a POM/menu capture matches the LIVE ISPF screen (screen-snapshot
/// -scrm Requirement 3.8-3.13).
///
/// The menu snapshot is a TWO-COLUMN ISPF body: the option rows on the LEFT and
/// the right-hand block (calendar + Time + Day of Year) on the RIGHT of the same
/// lines, the `Command ===>` line at the top, and the PF-key footer at the
/// bottom.
///
/// The former window-title / menu-bar / tab-header / title-line "header chrome"
/// fields were REMOVED: the owner's authoritative ISPF mock has NONE of that
/// block -- just the title in the frame border, the command line, then the
/// options. The title is already carried in `ScreenModel::title` (rendered in
/// the top border), so it is never repeated in the body (Bug 2, owner
/// correction 2026-11-01; supersedes the Phase 2A Req 3.9 header-chrome model
/// FOR MENUS).
///
/// Every entry is plain, already-rendered selectable text. The PF-key footer is
/// carried in `key_bar_lines`, which the shell leaves EMPTY when the
/// Key_Label_Bar is hidden (pfshow off), so the capture reflects the live pfshow
/// state (Requirement 3.10).
#[derive(Debug, Default, Clone)]
pub struct MenuScreenChrome {
    /// The RIGHT-column block, top-to-bottom: the calendar lines (month/year
    /// header, weekday header, week rows) then the `Time` and `Day of Year`
    /// lines. Built by the shell from the SAME calendar/time/day-of-year logic
    /// the live POM uses (screen-snapshot-scrm Req 3.9). Empty when the menu
    /// hides the calendar (`show_calendar = false`), in which case the body is
    /// a single left column.
    pub right_column: Vec<String>,
    /// The PF-key footer line(s), present ONLY when the Key_Label_Bar is shown
    /// (pfshow on). Empty when pfshow is off (Requirement 3.10).
    pub key_bar_lines: Vec<String>,
}

/// The gap, in spaces, between the padded left option column and the right-hand
/// calendar column in the two-column menu body.
const MENU_COLUMN_GAP: usize = 3;

/// The total character width of the `Command ===>` input field (the underscore
/// run after any typed text), matching the on-screen POM command field.
const COMMAND_INPUT_WIDTH: usize = 119;

/// The `Command ===>` field value for a non-menu Context: the typed text followed
/// by the underscore input area, padded to the SAME width the POM/menu command
/// field uses (screen-snapshot-scrm Req 29.4, 30.6). Always non-empty, so the
/// common frame always shows its Command line.
pub(crate) fn command_field_value(typed: &str) -> String {
    let typed = typed.trim_end();
    let underscores = COMMAND_INPUT_WIDTH
        .saturating_sub(typed.chars().count())
        .max(4);
    format!("{typed}{}", "_".repeat(underscores))
}

/// Format one menu option row as the LIVE ISPF screen shows it:
/// `{key} {shortname} : {description} ({command})`, e.g.
/// `0 Settings : FFWB Settings and Client Parameters (Settings)`.
///
/// The short name is the option's `command` label (there is no separate
/// short-name field on `MenuOption`; the command IS the on-screen mnemonic, as
/// the owner's mock shows -- `0 Settings`, `S SCRM`, `X X`). When an option has
/// no command the description stands alone with no `(..)` suffix.
///
/// Validates: screen-snapshot-scrm Req 3.11, 3.12.
fn option_row_text(key: &str, command: &str, description: &str) -> String {
    let command = command.trim();
    if command.is_empty() {
        format!("{key} {description}")
    } else {
        format!("{key} {command} : {description} ({command})")
    }
}

/// Compose the TWO-COLUMN menu body as exact verbatim lines: the option rows on
/// the LEFT (each padded to a shared column width) and the right-hand block
/// (calendar + Time + Day of Year) on the RIGHT of the same lines. Rows beyond
/// the shorter column are padded so both columns stay aligned and never overlap.
///
/// `left` is the already-formatted option rows; `right` is the right-column
/// block. The returned lines carry no command line and no footer -- the caller
/// frames them with the command line on top and the PF footer at the bottom.
///
/// Validates: screen-snapshot-scrm Req 3.11, 3.13.
fn compose_two_column_body(left: &[String], right: &[String]) -> Vec<String> {
    if right.is_empty() {
        // Calendar-less menu (e.g. Settings): a single left column.
        return left.to_vec();
    }
    let left_width = left.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let rows = left.len().max(right.len());
    let mut out = Vec::with_capacity(rows);
    for i in 0..rows {
        let l = left.get(i).map(String::as_str).unwrap_or("");
        let pad = left_width.saturating_sub(l.chars().count()) + MENU_COLUMN_GAP;
        match right.get(i) {
            Some(r) if !r.is_empty() => {
                out.push(format!("{l}{}{r}", " ".repeat(pad)));
            }
            // No right-column content on this row: keep only the left text
            // (trailing spaces trimmed by not padding a bare-left row).
            _ => out.push(l.to_string()),
        }
    }
    out
}

/// Build the FULL-screen logical model for a Menu_Workspace, matching the LIVE
/// ISPF screen (screen-snapshot-scrm Req 3.8-3.13): the title in the frame
/// border (from [`ScreenModel::title`]), then as verbatim body lines -- the
/// `Command ===>` line, the TWO-COLUMN body (option rows on the LEFT, the
/// calendar + Time + Day of Year on the RIGHT), blank spacer rows, and the
/// PF-key footer at the bottom (present only when pfshow is on).
///
/// There is NO superfluous header block (window title / menu-bar echo / `[POM]`
/// tag / duplicated title) -- the owner's authoritative mock has none, and the
/// title already appears in the border (Bug 2, owner correction 2026-11-01).
///
/// The body is composed here as exact lines rather than through the generic
/// label/value `fields` path, because the two-column ISPF layout is this
/// Context's own concern (wiring-standard: logical-model-first, not egui).
///
/// Validates: screen-snapshot-scrm Req 3.8, 3.9, 3.10, 3.11, 3.12, 3.13.
pub fn menu_workspace_screen_model_full(
    state: &MenuWorkspaceState,
    command_line: &str,
    chrome: &MenuScreenChrome,
) -> ScreenModel {
    let title = state.menu_title().unwrap_or_else(|| "Menu".to_string());
    let mut model = ScreenModel::new(title);
    // The command line is now rendered INSIDE the frame as the first body row;
    // keep ScreenModel::command_line empty so the generic renderer does not ALSO
    // echo it outside the frame.
    model.command_line = String::new();

    // Command ===> row (Bug 1): the first body row, mirroring the live POM's
    // command field. The typed value is followed by an underscore input area so
    // the row shows the on-screen `Command ===> ____...` field.
    let typed = command_line.trim_end();
    let underscores = COMMAND_INPUT_WIDTH.saturating_sub(typed.chars().count());
    model = model.with_body_line(format!("Command ===> {typed}{}", "_".repeat(underscores)));

    match &state.menu {
        Some(menu) => {
            let left: Vec<String> = menu
                .options
                .iter()
                .map(|o| option_row_text(&o.key, &o.command, &o.description))
                .collect();
            for line in compose_two_column_body(&left, &chrome.right_column) {
                model = model.with_body_line(line);
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

    // Two blank spacer rows then the PF-key footer at the bottom of the frame
    // (present only when pfshow is on, Req 3.10).
    if !chrome.key_bar_lines.is_empty() {
        model = model.with_body_line(String::new());
        model = model.with_body_line(String::new());
        for line in &chrome.key_bar_lines {
            model = model.with_body_line(line.clone());
        }
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

/// Build the RIGHT-column block for the POM/menu snapshot: the calendar (from
/// [`calendar_lines`]) followed by the `Time` and `Day of Year` lines, exactly
/// as the LIVE POM shows them. The caller computes `year`/`month`/`today`,
/// `hour`/`minute` and `day_of_year` from the SAME clock/date source the live
/// screen uses (`chrono::Local::now()`), so the snapshot equals the live screen
/// (screen-snapshot-scrm Req 3.9, 3.13). Passing the components keeps this
/// deterministic and stubbable in tests.
///
/// The `Time` / `Day of Year` labels are padded to a fixed width so the colons
/// align, matching the live POM block.
///
/// Validates: screen-snapshot-scrm Req 3.9, 3.13.
pub fn calendar_right_column(
    year: i32,
    month: u32,
    today: u32,
    hour: u32,
    minute: u32,
    day_of_year: u32,
) -> Vec<String> {
    let mut lines = calendar_lines(year, month, today);
    lines.push(format!("{:<17} : {:02}:{:02}", "Time", hour, minute));
    lines.push(format!("{:<17} : {:>3}", "Day of Year", day_of_year));
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

    // Validates: Requirement 2.1 -- a load error surfaces as a message in the
    // full model so the capture reflects what the user sees.
    #[test]
    fn load_error_surfaces_as_message() {
        let state = MenuWorkspaceState::load("/nonexistent/menu.toml");
        let model = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));
        assert!(!model.messages.is_empty());
    }

    // A fixed, deterministic right-column block (calendar + Time + Day of Year)
    // so Time / Day-of-Year / today never depend on the real clock in tests.
    fn sample_right_column() -> Vec<String> {
        // October 2026, today = the 25th, 12:58, day-of-year 298.
        calendar_right_column(2026, 10, 25, 12, 58, 298)
    }

    fn sample_chrome(with_keys: bool) -> MenuScreenChrome {
        MenuScreenChrome {
            right_column: sample_right_column(),
            key_bar_lines: if with_keys {
                vec!["Base | F1 Help | F3 Exit".to_string()]
            } else {
                Vec::new()
            },
        }
    }

    // Validates: screen-snapshot-scrm Req 3.8, 3.11, 3.12 -- OWNER CORRECTION
    // 2026-11-01 (Bug 1/2/3): the full model's body is the TWO-COLUMN ISPF
    // layout. The FIRST body row is the `Command ===>` line; there is NO
    // superfluous header block (no standalone window-title / menu-bar echo /
    // `[POM]` tag / duplicated title line); option rows are
    // `{key} {shortname} : {desc} ({cmd})`. (Supersedes the removed Phase 2A
    // `full_model_includes_chrome_and_option_rows` header-chrome assertion.)
    #[test]
    fn full_model_is_two_column_body_without_header_chrome() {
        let state = pom_like();
        let model = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));

        // Title lives in the border (model.title), never repeated in the body.
        assert_eq!(model.title, "Primary Option Menu");
        // No generic header chrome, no generic field path for the menu now.
        assert!(model.header_lines.is_empty(), "no header-chrome block");
        assert!(model.footer_lines.is_empty(), "no footer-chrome block");
        assert!(
            model.fields.is_empty(),
            "menu body is body_lines, not fields"
        );
        // command_line is folded into the body, not echoed outside the frame.
        assert!(model.command_line.is_empty());

        // First body row is the command line.
        assert!(
            model.body_lines[0].starts_with("Command ===> "),
            "first body row must be the command line: {:?}",
            model.body_lines[0]
        );
        // Option rows use `{key} {shortname} : {desc} ({cmd})`.
        let body = model.body_lines.join("\n");
        assert!(
            body.contains("1 FILES : File Explorer (FILES)"),
            "option row format must be key shortname : desc (cmd):\n{body}"
        );
        // No superfluous header block leaked into the body.
        assert!(!body.contains("[POM]"), "no tab tag in body:\n{body}");
        let title_line_count = model
            .body_lines
            .iter()
            .filter(|l| l.trim() == "Primary Option Menu")
            .count();
        assert_eq!(title_line_count, 0, "title must not be duplicated in body");
    }

    // Validates: screen-snapshot-scrm Req 3.13 -- the calendar + Time + Day of
    // Year appear to the RIGHT on the SAME lines as the option rows (two-column
    // body), and the Time / Day-of-Year block follows below the calendar.
    #[test]
    fn full_snapshot_is_two_column_options_left_calendar_right() {
        let state = pom_like();
        let model = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));
        let text = render_snapshot(&model, SnapshotFormat::PlainText);
        assert!(text.contains("October 2026"), "calendar month must appear");
        assert!(text.contains("Time"), "Time line must appear");
        assert!(text.contains("Day of Year"), "Day of Year line must appear");
        // A SINGLE line carries BOTH an option (left) and calendar (right).
        let combined = model
            .body_lines
            .iter()
            .any(|l| l.contains("File Explorer (FILES)") && l.contains("October 2026"));
        assert!(
            combined,
            "an option row and the calendar must share a line (two columns):\n{text}"
        );
    }

    // Validates: screen-snapshot-scrm Req 3.13 -- the two columns do not
    // overlap and the RIGHT column has a single, consistent origin on every
    // line that carries calendar content. (The calendar's own weekday header
    // carries a leading space inside that column, matching the live POM, so the
    // test compares the right-column ORIGIN, not the first glyph.)
    #[test]
    fn two_column_calendar_right_edge_is_aligned() {
        let state = pom_like();
        let model = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));

        // The longest option row drives the shared left-column width; the right
        // column therefore begins at left_width + MENU_COLUMN_GAP on every row.
        let left: Vec<String> = state
            .menu
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|o| option_row_text(&o.key, &o.command, &o.description))
            .collect();
        let left_width = left.iter().map(|l| l.chars().count()).max().unwrap();
        let right_origin = left_width + MENU_COLUMN_GAP;

        // Row 1 of the body is the first option row carrying calendar line 0
        // ("October 2026", no leading space) -- it must begin exactly at the
        // right-column origin.
        let month_line = model
            .body_lines
            .iter()
            .find(|l| l.contains("October 2026"))
            .expect("a line with the month header");
        assert_eq!(
            month_line.find("October 2026").unwrap(),
            right_origin,
            "calendar month header must begin at the right-column origin; no overlap"
        );
        // And the option text on that line stays within the left column (ends
        // before the right-column origin), so the two columns never collide.
        let opt_end = month_line.find("(FILES)").map(|i| i + "(FILES)".len());
        if let Some(end) = opt_end {
            assert!(
                end <= right_origin,
                "left option text must not run into the right column"
            );
        }
    }

    // Validates: screen-snapshot-scrm Req 3.12 -- option row short name and the
    // (command) suffix for the authoritative POM options (owner mock).
    #[test]
    fn option_row_matches_owner_mock_format() {
        assert_eq!(
            option_row_text("0", "Settings", "FFWB Settings and Client Parameters"),
            "0 Settings : FFWB Settings and Client Parameters (Settings)"
        );
        assert_eq!(
            option_row_text("S", "SCRM", "SCRM : Screen Capture and Recording"),
            "S SCRM : SCRM : Screen Capture and Recording (SCRM)"
        );
        assert_eq!(
            option_row_text("X", "X", "Collapse to the visual root"),
            "X X : Collapse to the visual root (X)"
        );
    }

    // Validates: screen-snapshot-scrm Req 3.9, 3.13 -- the right column carries
    // the Time (HH:MM) and 1-based Day of Year, deterministically, below the
    // calendar.
    #[test]
    fn right_column_has_time_and_day_of_year() {
        let col = calendar_right_column(2026, 11, 1, 12, 58, 305);
        let text = col.join("\n");
        assert!(text.contains("November 2026"));
        assert!(
            col.iter()
                .any(|l| l.contains("Time") && l.contains("12:58")),
            "Time line must be HH:MM:\n{text}"
        );
        assert!(
            col.iter()
                .any(|l| l.contains("Day of Year") && l.contains("305")),
            "Day of Year must be the 1-based ordinal:\n{text}"
        );
    }

    // Validates: screen-snapshot-scrm Req 3.13 -- a calendar-LESS menu (right
    // column empty, e.g. Settings) renders a single left column with no
    // overlap artefacts.
    #[test]
    fn calendar_less_menu_renders_single_column() {
        let state = pom_like();
        let chrome = MenuScreenChrome {
            right_column: Vec::new(),
            key_bar_lines: Vec::new(),
        };
        let model = menu_workspace_screen_model_full(&state, "", &chrome);
        let body = model.body_lines.join("\n");
        assert!(body.contains("1 FILES : File Explorer (FILES)"));
        assert!(!body.contains("October 2026"));
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

    // Validates: screen-snapshot-scrm Req 3.8 (Bug 1) -- the rendered menu
    // snapshot contains a `Command ===>` row as the FIRST body row (directly
    // under the title border), with an underscore input area, and the typed
    // value when present.
    #[test]
    fn rendered_menu_has_command_line_as_first_body_row() {
        let state = pom_like();

        // Empty command line: the row is still present with the input area.
        let model = menu_workspace_screen_model_full(&state, "", &sample_chrome(true));
        let text = render_snapshot(&model, SnapshotFormat::PlainText);
        let rows: Vec<&str> = text.lines().collect();
        // rows: [top border, title, separator, FIRST body row, ...]
        assert!(
            rows[3].contains("Command ===> "),
            "first body row must be `Command ===>`:\n{text}"
        );
        assert!(rows[3].contains('_'), "command field shows an input area");

        // A typed value is shown before the underscores.
        let typed = menu_workspace_screen_model_full(&state, "SNAPSHOT", &sample_chrome(true));
        let typed_text = render_snapshot(&typed, SnapshotFormat::PlainText);
        assert!(typed_text.contains("Command ===> SNAPSHOT"));
    }
}
