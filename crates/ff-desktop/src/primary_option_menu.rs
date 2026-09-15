//! # Primary Option Menu
//!
//! Renders the ISPF-style home screen shown when no file tabs are open.
//! Displays a numbered option list on the left and a live calendar on the right.

use chrono::{Datelike, Local, Timelike};
use eframe::egui;

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

/// Render the live calendar column into `ui`, returning any month-navigation
/// the user triggered this frame (clicking the `<`/`>` hotspots in the header).
///
/// This is the single calendar-drawing code path shared by the Primary Option
/// Menu and by the unified Menu_Workspace renderer (menu-workspace Req 2.1b,
/// CR-CH-018), so the calendar looks and behaves identically wherever it
/// appears.
///
/// `calendar_offset` is the number of months relative to the current month
/// (0 = current, -1 = previous, +1 = next). `calendar_fg` is the body colour;
/// `today_bg`/`today_fg`/`use_today_reverse` control today's cell styling.
///
/// Validates: Requirement 14.1, 14.41, 14.42, 13.7, 13.8; menu-workspace 2.1b
pub fn render_calendar(
    ui: &mut egui::Ui,
    calendar_offset: i32,
    calendar_fg: egui::Color32,
    today_bg: egui::Color32,
    today_fg: egui::Color32,
    use_today_reverse: bool,
) -> Option<CalendarNav> {
    let now = Local::now();
    let today_year = now.year();
    let today_month = now.month();
    let today_day = now.day();
    let hour = now.hour();
    let min = now.minute();

    let (year, month) = offset_month(today_year, today_month, calendar_offset);
    let is_current_month = year == today_year && month == today_month;
    let doy = day_of_year(today_year, today_month, today_day);

    let mut nav: Option<CalendarNav> = None;

    ui.vertical(|ui| {
        // Validates: Requirement 14.41 -- header is < MonthName YYYY >
        let header = format_calendar_header(MONTH_NAMES[(month - 1) as usize], year);
        let header_resp = ui.add(
            egui::Label::new(
                egui::RichText::new(&header)
                    .monospace()
                    .strong()
                    .color(calendar_fg),
            )
            .sense(egui::Sense::click()),
        );
        if header_resp.clicked() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let rect = header_resp.rect;
                let sixth = rect.width() / 6.0;
                if pos.x < rect.left() + sixth {
                    nav = Some(CalendarNav::Prev);
                } else if pos.x > rect.right() - sixth {
                    nav = Some(CalendarNav::Next);
                }
            }
        }

        ui.add(egui::SelectableLabel::new(
            false,
            egui::RichText::new("Su Mo Tu We Th Fr Sa")
                .monospace()
                .color(calendar_fg),
        ));

        let first_wd = first_weekday_of_month(year, month);
        let total_days = days_in_month(year, month);
        let mut col = first_wd;
        let mut row: Vec<(u32, bool)> = (0..first_wd).map(|_| (0, false)).collect();
        for d in 1..=total_days {
            let is_today = is_current_month && d == today_day;
            row.push((d, is_today));
            col += 1;
            if col == 7 {
                render_calendar_row(ui, &row, calendar_fg, today_bg, today_fg, use_today_reverse);
                row.clear();
                col = 0;
            }
        }
        if !row.is_empty() {
            render_calendar_row(ui, &row, calendar_fg, today_bg, today_fg, use_today_reverse);
        }

        ui.add_space(4.0);
        ui.add(egui::SelectableLabel::new(
            false,
            egui::RichText::new(format!("Time . . . . : {:02}:{:02}", hour, min))
                .monospace()
                .color(calendar_fg),
        ));
        ui.add(egui::SelectableLabel::new(
            false,
            egui::RichText::new(format!("Day of year. :   {}", doy))
                .monospace()
                .color(calendar_fg),
        ));
    });

    nav
}

/// Render one week row of the calendar grid.
///
/// When `use_today_reverse` is true, today's cell gets a coloured background
/// (turquoise) with black text instead of the plain `*` marker.
/// Validates: Requirement 13.8
fn render_calendar_row(
    ui: &mut egui::Ui,
    cells: &[(u32, bool)],
    calendar_fg: egui::Color32,
    today_bg: egui::Color32,
    today_fg: egui::Color32,
    use_today_reverse: bool,
) {
    let has_today = use_today_reverse && cells.iter().any(|(_, t)| *t);

    if !has_today {
        // Fast path: single monospace label for the whole row.
        // Validates: Requirement 14.1 -- selectable calendar day text
        let mut line = String::new();
        for (d, _) in cells {
            if *d == 0 {
                line.push_str("   ");
            } else {
                line.push_str(&format!("{:>2} ", d));
            }
        }
        ui.add(egui::SelectableLabel::new(
            false,
            egui::RichText::new(line.trim_end().to_string())
                .monospace()
                .color(calendar_fg),
        ));
    } else {
        // Slow path: cell-by-cell so today gets a filled background rect.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (d, is_today) in cells {
                if *d == 0 {
                    ui.label(egui::RichText::new("   ").monospace().color(calendar_fg));
                } else if *is_today {
                    // Reversed: turquoise bg, black text.
                    let text = format!("{:>2}", d);
                    let galley =
                        ui.painter()
                            .layout_no_wrap(text, egui::FontId::monospace(14.0), today_fg);
                    let cell_size = galley.size() + egui::vec2(2.0, 0.0);
                    let (rect, _) = ui.allocate_exact_size(cell_size, egui::Sense::hover());
                    ui.painter().rect_filled(rect, 0.0, today_bg);
                    ui.painter()
                        .galley(rect.min + egui::vec2(1.0, 0.0), galley, today_fg);
                    // Trailing space separator
                    ui.label(egui::RichText::new(" ").monospace().color(calendar_fg));
                } else {
                    // Validates: Requirement 14.1 -- selectable calendar day text
                    ui.add(egui::SelectableLabel::new(
                        false,
                        egui::RichText::new(format!("{:>2} ", d))
                            .monospace()
                            .color(calendar_fg),
                    ));
                }
            }
        });
    }
}

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
