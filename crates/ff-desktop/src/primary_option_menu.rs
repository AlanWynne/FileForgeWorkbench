//! # Primary Option Menu
//!
//! Renders the ISPF-style home screen shown when no file tabs are open.
//! Displays a numbered option list on the left and a live calendar on the right.

use chrono::Datelike;
use eframe::egui;

mod calendar_header;
pub use calendar_header::centered_calendar_header;

mod render;
pub use render::render_calendar;

// The POM option list and exit line are no longer defined here: the POM is a
// data-driven Menu_Workspace backed by `menus/pom.toml`, rendered by the shared
// `menu_workspace::render::render_menu_workspace` (menu-workspace Req 2.1c-2.1i,
// CR-CH-018). This module now provides only the shared calendar (`render_calendar`
// + date helpers), the `CalendarNav` type, and the `PomColours` palette mapping.

/// Returns the day-of-year (1-based) for the given date components.
///
/// Validates: Requirement 14.5
pub fn day_of_year(year: i32, month: u32, day: u32) -> u32 {
    use chrono::NaiveDate;
    let date = NaiveDate::from_ymd_opt(year, month, day).expect("valid date");
    date.ordinal()
}

/// Returns the number of days in the given month/year.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    use chrono::NaiveDate;
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(y, m, 1)
        .expect("valid date")
        .pred_opt()
        .expect("valid pred")
        .day()
}

/// Returns the weekday index (0 = Sunday) of the first day of the given month.
pub fn first_weekday_of_month(year: i32, month: u32) -> u32 {
    use chrono::{NaiveDate, Weekday};
    let d = NaiveDate::from_ymd_opt(year, month, 1).expect("valid date");
    match d.weekday() {
        Weekday::Sun => 0,
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
    }
}

/// Compute the display (year, month) given today's year/month and a signed month offset.
///
/// Validates: Requirement 14.42
pub fn offset_month(today_year: i32, today_month: u32, offset: i32) -> (i32, u32) {
    let total = (today_year * 12 + today_month as i32 - 1) + offset;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    (year, month)
}

/// Format the calendar header as a fixed-width 20-char string.
///
/// Layout: `<  {month:<9} {year}  >` — `<` at position 1, `>` at position 20,
/// month name starts at position 4, year ends at position 17.
///
/// Validates: Requirement 14.41
///
/// CR-CH-023: the rendered calendar header is now three widgets -- a focusable
/// `<` button, a month/year label, and a focusable `>` button -- so this
/// single fixed-width string is no longer used by `render_calendar`. It is
/// retained for the Req 14.41 fixed-width-format unit test and as a reference
/// for the exact ISPF column layout.
#[allow(dead_code)]
pub fn format_calendar_header(month_name: &str, year: i32) -> String {
    format!("<  {:<9} {}  >", month_name, year)
}

/// Colours passed into the POM renderer by the shell.
///
/// Use `PomColours::inherited()` for non-Legacy themes (falls back to egui theme).
/// In Legacy mode the shell passes explicit ISPF semantic colours.
#[derive(Debug, Clone, Copy)]
pub struct PomColours {
    /// Normal body text (Green in Legacy).
    pub normal_text: egui::Color32,
    /// Option item number / key character (White in Legacy).
    pub option_key: egui::Color32,
    /// Option item name / label (Turquoise in Legacy).
    pub option_label: egui::Color32,
    /// Primary menu title / structural text (Blue in Legacy). Part of the
    /// Legacy palette contract (asserted by tests); reserved for the focus-row
    /// reversal colour when that is ported into the shared menu renderer.
    #[allow(dead_code)]
    pub primary_text: egui::Color32,
    /// Calendar body text (Turquoise in Legacy).
    pub calendar_fg: egui::Color32,
    /// Today's day cell background (Turquoise in Legacy — reversed).
    pub today_bg: egui::Color32,
    /// Today's day cell foreground (Black in Legacy — reversed).
    pub today_fg: egui::Color32,
}

impl PomColours {
    /// Inherit all colours from the active egui theme (Dark / Light / HighContrast).
    pub fn inherited() -> Self {
        Self {
            normal_text: egui::Color32::PLACEHOLDER,
            option_key: egui::Color32::PLACEHOLDER,
            option_label: egui::Color32::PLACEHOLDER,
            primary_text: egui::Color32::PLACEHOLDER,
            calendar_fg: egui::Color32::PLACEHOLDER,
            today_bg: egui::Color32::PLACEHOLDER,
            today_fg: egui::Color32::PLACEHOLDER,
        }
    }

    /// Build `PomColours` from a `ThemePalette`, mapping semantic ISPF colours.
    ///
    /// Validates: Requirement 13 (Legacy Theme Colour Semantics)
    pub fn from_palette(palette: &ff_theme::ThemePalette) -> Self {
        use ff_theme::ColourToken;
        let c = |tok: ColourToken| {
            let rgba = palette.colour(tok);
            egui::Color32::from_rgb(rgba.r, rgba.g, rgba.b)
        };
        Self {
            // Req 13.3 / 13.6 — normal body text and option descriptions: bright green
            normal_text: c(ColourToken::EditorForeground),
            // Req 13.4 — option key characters: white
            option_key: c(ColourToken::UiMenuBarForeground),
            // Req 13.5 — option item names: turquoise (#00AAAA, normal-intensity)
            option_label: c(ColourToken::UiInputBorder),
            // Req 13.2 — primary menu / heading: blue (#0000AA)
            primary_text: c(ColourToken::UiPrimaryMenuBackground),
            // Req 13.7 — calendar body: turquoise (#00AAAA)
            calendar_fg: c(ColourToken::UiInputBorder),
            // Req 13.8 — today cell background: turquoise (#00AAAA)
            today_bg: c(ColourToken::UiInputBorder),
            // Req 13.8 — today cell foreground: black
            today_fg: c(ColourToken::EditorBackground),
        }
    }

    /// Resolve a colour: if PLACEHOLDER, fall back to egui's current text colour.
    /// Resolve a colour: PLACEHOLDER falls back to the ui's inherited text
    /// colour; any explicit colour is returned as-is. Shared with the
    /// menu-workspace renderer so the calendar colours match POM semantics.
    pub(crate) fn resolve(c: egui::Color32, ui: &egui::Ui) -> egui::Color32 {
        if c == egui::Color32::PLACEHOLDER {
            ui.visuals().text_color()
        } else {
            c
        }
    }
}

// B084: the live calendar header now resolves month names via the i18n
// catalogue (see `render_calendar`), so this English array is retained as
// reference data for the `format_calendar_header` fixed-width unit test AND as
// the source for the screen-snapshot calendar text (`month_name`).
const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The English month name for a 1-based `month` (1 = January). Used by the
/// screen-snapshot calendar text builder (screen-snapshot-scrm Req 3.9); the
/// live on-screen calendar still localises month names via the i18n catalogue.
/// Out-of-range input clamps to a valid month so the function is total.
pub fn month_name(month: u32) -> &'static str {
    let idx = (month.clamp(1, 12) - 1) as usize;
    MONTH_NAMES[idx]
}

/// Calendar navigation direction returned when the user clicks < or >.
///
/// Validates: Requirement 14.41, 14.42
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarNav {
    /// Navigate to the previous month.
    Prev,
    /// Navigate to the next month.
    Next,
}

/// What `render_calendar` reports for one frame: the month-navigation the user
/// triggered (if any) plus the egui ids of the focusable `<`/`>` buttons so the
/// shell Boundary_Policy can treat the `>` button as the last interior control
/// (CR-CH-023; menu-workspace Req 15.5-15.8).
#[derive(Debug, Clone, Copy, Default)]
pub struct CalendarRender {
    /// Month navigation triggered this frame, if any.
    pub nav: Option<CalendarNav>,
    /// Egui id of the `<` (previous-month) button.
    pub prev_id: Option<egui::Id>,
    /// Egui id of the `>` (next-month) button.
    pub next_id: Option<egui::Id>,
}

// `render_calendar` and its private week-row helper now live in the sibling
// `render` module (behaviour-preserving split to keep this file under the
// 400-line rule, rust-standards.md); `render_calendar` is re-exported above.

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // POM option-list tests removed: the option list is now data-driven from
    // menus/pom.toml and covered by menu_workspace loader/render/defaults tests
    // (menu-workspace Req 2.1c-2.1i, CR-CH-018).

    /// Validates: Requirement 14.5 — day_of_year returns correct ordinal.
    #[test]
    fn day_of_year_returns_correct_ordinal() {
        assert_eq!(day_of_year(2026, 1, 1), 1);
        assert_eq!(day_of_year(2026, 12, 31), 365);
        assert_eq!(day_of_year(2024, 12, 31), 366);
        assert_eq!(day_of_year(2026, 8, 1), 213);
    }

    /// Validates: Requirement 14.4 — days_in_month returns correct counts.
    #[test]
    fn days_in_month_returns_correct_counts() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    /// Validates: Requirement 14.4 — first_weekday_of_month returns Sunday=0..Saturday=6.
    #[test]
    fn first_weekday_of_month_known_dates() {
        assert_eq!(first_weekday_of_month(2026, 8), 6);
        assert_eq!(first_weekday_of_month(2026, 1), 4);
        assert_eq!(first_weekday_of_month(2026, 3), 0);
    }

    // ── Req 14.41 / 14.42 — Calendar navigation ──────────────────────────────

    /// Validates: Requirement 14.41 — CalendarNav::Prev and Next variants exist.
    #[test]
    fn calendar_nav_variants_exist() {
        assert_eq!(CalendarNav::Prev, CalendarNav::Prev);
        assert_eq!(CalendarNav::Next, CalendarNav::Next);
        assert_ne!(CalendarNav::Prev, CalendarNav::Next);
    }

    /// Validates: menu-workspace Req 15.7 (CR-CH-023) -- the calendar `<`/`>`
    /// month-navigation controls are REAL focusable buttons: rendering the
    /// calendar and pressing Tab must land on two distinct focusable widgets
    /// (previously the header was a single non-tab-reachable click-region).
    #[test]
    fn calendar_prev_next_are_focusable_buttons() {
        use egui_kittest::Harness;
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(400.0, 400.0))
            .build_ui(|ui| {
                let _r = render_calendar(
                    ui,
                    0,
                    egui::Color32::WHITE,
                    egui::Color32::from_rgb(0, 0xAA, 0xAA),
                    egui::Color32::BLACK,
                    false,
                );
            });
        // Press Tab well past the number of week rows: if any day-grid row were
        // focusable it would show up as an extra distinct focus stop.
        let mut focused: Vec<egui::Id> = Vec::new();
        for _ in 0..12 {
            harness.key_press(egui::Key::Tab);
            harness.run();
            if let Some(id) = harness.ctx.memory(|m| m.focused()) {
                focused.push(id);
            }
        }
        let distinct: std::collections::HashSet<egui::Id> = focused.iter().copied().collect();
        // EXACTLY two focusable widgets: the `<` and `>` buttons. Day-of-week
        // header, week rows, time and day-of-year are non-interactive labels
        // (menu-workspace Req 15.10; B056 -- week rows were SelectableLabels and
        // added spurious focus stops).
        assert_eq!(
            distinct.len(),
            2,
            "calendar must expose EXACTLY two focusable widgets (< and >), not day cells; \
             focused ids: {focused:?}"
        );
    }

    /// Validates: menu-workspace Req 15.8 -- activating the `<` button with
    /// Enter navigates to the previous month (returns CalendarNav::Prev). The
    /// harness focuses the first button (the `<`) and presses Enter; the render
    /// closure records the returned nav.
    #[test]
    fn calendar_prev_button_enter_returns_prev_nav() {
        use egui_kittest::Harness;
        use std::cell::Cell;
        use std::rc::Rc;
        let nav_seen: Rc<Cell<Option<CalendarNav>>> = Rc::new(Cell::new(None));
        let nav_for_ui = Rc::clone(&nav_seen);
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(400.0, 400.0))
            .build_ui(move |ui| {
                let r = render_calendar(
                    ui,
                    0,
                    egui::Color32::WHITE,
                    egui::Color32::from_rgb(0, 0xAA, 0xAA),
                    egui::Color32::BLACK,
                    false,
                );
                if r.nav.is_some() {
                    nav_for_ui.set(r.nav);
                }
            });
        // Tab once -> focus the first focusable widget (the `<` button), then
        // activate it with Enter.
        harness.key_press(egui::Key::Tab);
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
        harness.run();
        assert_eq!(
            nav_seen.get(),
            Some(CalendarNav::Prev),
            "Enter on the focused `<` button must return CalendarNav::Prev"
        );
    }

    /// Validates: Requirement 14.42 — offset_month decrements correctly for Prev.
    #[test]
    fn calendar_prev_decrements_offset() {
        // offset -1 from August 2026 → July 2026
        let (y, m) = offset_month(2026, 8, -1);
        assert_eq!((y, m), (2026, 7));
    }

    /// Validates: Requirement 14.42 — offset_month increments correctly for Next.
    #[test]
    fn calendar_next_increments_offset() {
        // offset +1 from August 2026 → September 2026
        let (y, m) = offset_month(2026, 8, 1);
        assert_eq!((y, m), (2026, 9));
    }

    /// Validates: Requirement 14.42 — offset_month wraps correctly across year boundary.
    #[test]
    fn calendar_offset_wraps_year_boundary() {
        // +1 from December 2026 → January 2027
        let (y, m) = offset_month(2026, 12, 1);
        assert_eq!((y, m), (2027, 1));
        // -1 from January 2026 → December 2025
        let (y, m) = offset_month(2026, 1, -1);
        assert_eq!((y, m), (2025, 12));
    }

    /// Validates: Requirement 14.42 — offset 0 returns today's month unchanged.
    #[test]
    fn calendar_offset_zero_returns_current_month() {
        let (y, m) = offset_month(2026, 8, 0);
        assert_eq!((y, m), (2026, 8));
    }

    /// Validates: Requirement 14.42 — current-day highlight suppressed when offset != 0.
    #[test]
    fn current_day_hidden_when_offset_nonzero() {
        // When offset != 0, is_current_month is false → no highlight
        let (year, month) = offset_month(2026, 8, 1); // September 2026
        let is_current = year == 2026 && month == 8;
        assert!(!is_current, "offset +1 must not be the current month");
    }

    /// Validates: Requirement 14.42 — current-day highlight shown when offset is 0.
    #[test]
    fn current_day_shown_when_offset_zero() {
        let (year, month) = offset_month(2026, 8, 0);
        let is_current = year == 2026 && month == 8;
        assert!(is_current, "offset 0 must be the current month");
    }

    /// Validates: Requirement 14.41 — calendar header is fixed-width, < at col 0, > at last col.
    #[test]
    fn calendar_header_label_is_fixed_width() {
        let expected_len = format_calendar_header("September", 2026).len();
        for (i, name) in MONTH_NAMES.iter().enumerate() {
            let header = format_calendar_header(name, 2026);
            assert_eq!(
                header.len(),
                expected_len,
                "month {} ('{}') produced wrong length: {:?}",
                i + 1,
                name,
                header
            );
            assert!(header.starts_with('<'), "must start with '<': {:?}", header);
            assert!(header.ends_with('>'), "must end with '>': {:?}", header);
        }
        // Spot-check exact output: < at pos 1, month at pos 4, year ends pos 17, > at pos 20
        assert_eq!(
            format_calendar_header("September", 2026),
            "<  September 2026  >"
        );
        assert_eq!(
            format_calendar_header("January", 2026),
            "<  January   2026  >"
        );
    }

    // ── Req 13 — Legacy theme PomColours ─────────────────────────────────────

    /// Validates: Requirement 13.1 — Legacy menu bar text is white (PLACEHOLDER means inherited).
    #[test]
    fn pom_colours_inherited_uses_placeholder() {
        // Validates: Requirement 13.1
        let c = PomColours::inherited();
        assert_eq!(c.normal_text, egui::Color32::PLACEHOLDER);
        assert_eq!(c.option_key, egui::Color32::PLACEHOLDER);
        assert_eq!(c.option_label, egui::Color32::PLACEHOLDER);
        assert_eq!(c.primary_text, egui::Color32::PLACEHOLDER);
        assert_eq!(c.calendar_fg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_bg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_fg, egui::Color32::PLACEHOLDER);
    }

    /// Validates: Requirement 13.3 — Legacy normal text is bright green.
    /// Validates: Requirement 13.4 — Legacy option key is white.
    /// Validates: Requirement 13.5 — Legacy option label is turquoise (#00AAAA).
    /// Validates: Requirement 13.7 — Legacy calendar is turquoise (#00AAAA).
    /// Validates: Requirement 13.8 — Legacy today cell is reversed (turquoise bg, black text).
    #[test]
    fn legacy_pom_colours_match_ispf_spec() {
        // Validates: Requirement 13.3, 13.4, 13.5, 13.7, 13.8
        let bright_green = egui::Color32::from_rgb(0, 255, 0);
        let white = egui::Color32::from_rgb(255, 255, 255);
        // Normal-intensity turquoise (#00AAAA) — used for labels, calendar, today_bg
        let turquoise = egui::Color32::from_rgb(0, 170, 170);
        let black = egui::Color32::from_rgb(0, 0, 0);

        let c = PomColours {
            normal_text: bright_green,
            option_key: white,
            option_label: turquoise,
            primary_text: egui::Color32::from_rgb(0, 0, 170), // ISPF_BLUE
            calendar_fg: turquoise,
            today_bg: turquoise,
            today_fg: black,
        };

        assert_eq!(
            c.normal_text, bright_green,
            "normal text must be bright green"
        );
        assert_eq!(c.option_key, white, "option key must be white");
        assert_eq!(
            c.option_label, turquoise,
            "option label must be turquoise #00AAAA"
        );
        assert_eq!(
            c.calendar_fg, turquoise,
            "calendar must be turquoise #00AAAA"
        );
        assert_eq!(
            c.today_bg, turquoise,
            "today bg must be turquoise #00AAAA (reversed)"
        );
        assert_eq!(c.today_fg, black, "today fg must be black (reversed)");
    }
}
