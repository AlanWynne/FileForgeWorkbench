//! egui rendering for the shared POM / Menu_Workspace calendar.
//!
//! Split out of `primary_option_menu.rs` (behaviour-preserving refactor, no
//! observable change) to keep each source file under the 400-line limit
//! (`.kiro/steering/rust-standards.md`). This module owns `render_calendar`
//! and its private week-row helper; the date math, palette, and header
//! formatting remain in the parent module and are imported here.

use chrono::{Datelike, Local, Timelike};
use eframe::egui;

use super::{
    centered_calendar_header, day_of_year, days_in_month, first_weekday_of_month, offset_month,
    CalendarNav, CalendarRender,
};

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
) -> CalendarRender {
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
    let mut prev_id: Option<egui::Id> = None;
    let mut next_id: Option<egui::Id> = None;

    ui.vertical(|ui| {
        // Validates: Requirement 14.41; menu-workspace Req 15.7/15.8 --
        // header is `<  MonthName YYYY  >` with `<` and `>` as REAL focusable
        // buttons (CR-CH-023 replaces the former single click-region label, so
        // the buttons participate in egui-native Tab traversal and respond to
        // Enter/Space as well as mouse clicks).
        // B084: resolve the month name through the i18n catalogue (keys
        // ff-desktop-pom-month-01 .. -12) instead of the former hardcoded
        // English MONTH_NAMES index, so the header localises like the weekday /
        // time / day-of-year lines (localization Req 4.1). The two-space gap
        // before the year is preserved byte-for-byte.
        let month_key = format!("ff-desktop-pom-month-{:02}", month);
        let month_name = ff_i18n::t(&month_key);
        // menu-workspace Req 15.12 (CR-CH-063): center the month+year in a
        // constant-width monospace field so the `<` and `>` buttons below land
        // in the SAME columns for every month. The field width is derived from
        // the active locale's month names (B084), so this re-widens with locale.
        let month_label = centered_calendar_header(&month_name, year);
        ui.horizontal(|ui| {
            let prev = ui.add(
                egui::Button::new(
                    egui::RichText::new("<")
                        .monospace()
                        .strong()
                        .color(calendar_fg),
                )
                .frame(false),
            );
            prev_id = Some(prev.id);
            if prev.clicked() {
                nav = Some(CalendarNav::Prev);
            }
            ui.label(
                egui::RichText::new(&month_label)
                    .monospace()
                    .strong()
                    .color(calendar_fg),
            );
            let next = ui.add(
                egui::Button::new(
                    egui::RichText::new(">")
                        .monospace()
                        .strong()
                        .color(calendar_fg),
                )
                .frame(false),
            );
            next_id = Some(next.id);
            if next.clicked() {
                nav = Some(CalendarNav::Next);
            }
        });

        // Day-of-week header: non-interactive (not a Tab stop).
        ui.label(
            egui::RichText::new(ff_i18n::t("ff-desktop-pom-weekday-header"))
                .monospace()
                .color(calendar_fg),
        );

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
        // The time / day-of-year VALUES are pre-formatted here and passed as TEXT
        // placeables so ":02"/spacing render byte-identically; only the dotted-
        // leader label prose localises.
        ui.label(
            egui::RichText::new(ff_i18n::t_args(
                "ff-desktop-pom-time",
                &ff_i18n::MessageArgs::new().set("value", format!("{:02}:{:02}", hour, min)),
            ))
            .monospace()
            .color(calendar_fg),
        );
        ui.label(
            egui::RichText::new(ff_i18n::t_args(
                "ff-desktop-pom-day-of-year",
                &ff_i18n::MessageArgs::new().set("value", doy.to_string()),
            ))
            .monospace()
            .color(calendar_fg),
        );
    });

    CalendarRender {
        nav,
        prev_id,
        next_id,
    }
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
        // Non-interactive: day-grid rows must NOT be Tab focus stops
        // (menu-workspace Req 15.10; B056 -- SelectableLabel is focusable and was
        // adding a focus stop per week row between the calendar and the menu bar).
        ui.label(
            egui::RichText::new(line.trim_end().to_string())
                .monospace()
                .color(calendar_fg),
        );
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
                    // Non-interactive day cell (Req 15.10; B056: not a Tab stop).
                    ui.label(
                        egui::RichText::new(format!("{:>2} ", d))
                            .monospace()
                            .color(calendar_fg),
                    );
                }
            }
        });
    }
}
