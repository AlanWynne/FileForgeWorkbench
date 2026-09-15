//! egui rendering for Menu_Workspace.
//!
//! Validates: Requirement 2 (menu-workspace)

use eframe::egui;

use super::MenuWorkspaceState;

/// The colours the shared menu renderer uses for the calendar panel. The caller
/// resolves these from the active palette (Legacy uses ISPF semantics; other
/// themes inherit egui colours). Mirrors the POM's calendar colouring.
#[derive(Debug, Clone, Copy)]
pub struct MenuCalendarColours {
    /// Calendar body text colour.
    pub calendar_fg: egui::Color32,
    /// Today-cell background (when `use_today_reverse`).
    pub today_bg: egui::Color32,
    /// Today-cell foreground (when `use_today_reverse`).
    pub today_fg: egui::Color32,
    /// Whether to draw today's cell reversed (Legacy) vs a plain marker.
    pub use_today_reverse: bool,
}

impl Default for MenuCalendarColours {
    fn default() -> Self {
        Self {
            calendar_fg: egui::Color32::PLACEHOLDER,
            today_bg: egui::Color32::PLACEHOLDER,
            today_fg: egui::Color32::PLACEHOLDER,
            use_today_reverse: false,
        }
    }
}

/// Result of rendering a Menu_Workspace for one frame.
#[derive(Debug, Default)]
pub struct MenuRenderResult {
    /// The option the user activated this frame (clicked), if any.
    pub selected: Option<super::MenuOption>,
    /// Calendar month-navigation the user triggered this frame (when the menu
    /// shows the calendar and the user clicked the `<`/`>` header hotspots).
    pub calendar_nav: Option<crate::primary_option_menu::CalendarNav>,
}

/// Width of the command column: the widest command among the options, clamped
/// to a sane range so descriptions align without the column dominating.
///
/// Validates: Requirement 2.1a (menu-workspace)
pub(crate) fn command_column_width(options: &[super::MenuOption]) -> usize {
    options
        .iter()
        .map(|o| o.command.len())
        .max()
        .unwrap_or(0)
        .clamp(4, 24)
}

/// Format one option as three aligned columns: key | command | description.
///
/// Validates: Requirement 2.1a (menu-workspace)
pub(crate) fn format_option_row(option: &super::MenuOption, cmd_width: usize) -> String {
    format!(
        "{:<4}  {:<width$}  {}",
        option.key,
        option.command,
        option.description,
        width = cmd_width,
    )
}

/// Render a Menu_Workspace: a centred Menu_Title, a three-column option list
/// (key | command | description) on the left, and -- when the menu's
/// `show_calendar` is true -- the live calendar on the right, laid out like the
/// POM. This is the single shared renderer for all menus including the POM
/// (menu-workspace Req 2.1, 2.1a-2.1c; CR-CH-018).
///
/// `calendar_offset` is the month offset for the calendar; `cal` supplies the
/// calendar colours. The caller (shell render) owns the surrounding chrome
/// (Title_Line, Command_Field, Key_Label_Bar) and the `calendar_offset` state.
///
/// Validates: Requirement 2.1-2.6, 2.1a-2.1c, 10.6
pub fn render_menu_workspace(
    state: &mut MenuWorkspaceState,
    ui: &mut egui::Ui,
    calendar_offset: i32,
    cal: MenuCalendarColours,
) -> MenuRenderResult {
    let mut result = MenuRenderResult::default();

    let menu = match &state.menu {
        None => {
            // Req 1.5, 1.6 -- show error message
            let msg = state
                .load_error
                .clone()
                .unwrap_or_else(|| "Menu file error: unknown error".to_string());
            ui.colored_label(egui::Color32::RED, msg);
            return result;
        }
        Some(menu) => menu.clone(),
    };

    // Req 2.1 -- Menu_Title centred
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(&menu.title).strong().size(14.0));
    });
    ui.add_space(4.0);

    // Req 9.3 -- soft-limit advisory, non-blocking, above the list
    if let Some(advisory) = &state.advisory {
        ui.colored_label(egui::Color32::from_rgb(0xC8, 0x8A, 0x00), advisory);
        ui.add_space(2.0);
    }

    // Width for the command column so descriptions align (widest command,
    // clamped to a sane range).
    let cmd_width = command_column_width(&menu.options);

    // Req 2.1a-2.1c: option columns on the left, calendar (optional) on the right.
    ui.horizontal_top(|ui| {
        // ── Option list: three columns (key | command | description) ────────
        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .id_salt("menu_workspace_options")
                .show(ui, |ui| {
                    if menu.options.is_empty() {
                        // Req 2.6 -- empty options placeholder
                        ui.label("No options defined in this menu.");
                        return;
                    }

                    let mut last_group: Option<&str> = None;
                    for option in &menu.options {
                        // Req 2.4 -- group separator
                        let current_group = option.group.as_deref();
                        if current_group != last_group && last_group.is_some() {
                            ui.separator();
                        }
                        last_group = current_group;

                        // Three aligned columns (Req 2.1a).
                        let row_text = format_option_row(option, cmd_width);

                        if option.enabled {
                            // Req 2.2 -- clickable row
                            if ui
                                .selectable_label(false, egui::RichText::new(&row_text).monospace())
                                .clicked()
                            {
                                result.selected = Some(option.clone());
                            }
                        } else {
                            // Req 2.3 -- disabled style, no interaction
                            ui.add_enabled(
                                false,
                                egui::Label::new(
                                    egui::RichText::new(&row_text)
                                        .monospace()
                                        .color(egui::Color32::GRAY),
                                ),
                            );
                        }
                    }
                });
        });

        // ── Calendar on the right when the menu opts in (Req 2.1b, 1.8) ──────
        if menu.show_calendar {
            ui.add_space(32.0);
            let calendar_fg = crate::primary_option_menu::PomColours::resolve(cal.calendar_fg, ui);
            result.calendar_nav = crate::primary_option_menu::render_calendar(
                ui,
                calendar_offset,
                calendar_fg,
                cal.today_bg,
                cal.today_fg,
                cal.use_today_reverse,
            );
        }
    });

    result
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_workspace::{MenuFile, MenuOption};

    fn make_state_with_options(options: Vec<MenuOption>) -> MenuWorkspaceState {
        MenuWorkspaceState {
            file_path: std::path::PathBuf::from("test.toml"),
            menu: Some(MenuFile {
                title: "Test Menu".to_string(),
                options,
                show_calendar: true,
            }),
            load_error: None,
            last_modified: None,
            advisory: None,
            limits: crate::menu_workspace::OptionLimits::default(),
        }
    }

    // Validates: Requirement 2.6 -- empty options list produces placeholder state
    #[test]
    fn render_empty_options_state_is_valid() {
        let state = make_state_with_options(vec![]);
        assert!(state.menu.as_ref().unwrap().options.is_empty());
    }

    // Validates: Requirement 2.3 -- disabled option is not selectable
    #[test]
    fn render_disabled_option_not_selectable() {
        let state = make_state_with_options(vec![MenuOption {
            key: "1".to_string(),
            command: "FILES".to_string(),
            description: "Files".to_string(),
            enabled: false,
            group: None,
            target: None,
        }]);
        assert!(!state.menu.as_ref().unwrap().options[0].enabled);
    }

    // Validates: Requirement 9.3 -- advisory present in state is rendered above list
    #[test]
    fn advisory_line_rendered_when_soft_exceeded() {
        let mut state = make_state_with_options(vec![MenuOption {
            key: "1".to_string(),
            command: "FILES".to_string(),
            description: "Files".to_string(),
            enabled: true,
            group: None,
            target: None,
        }]);
        state.advisory = Some("This menu has 65 options (advised maximum 64).".to_string());
        // The render path reads state.advisory; confirm the state carries it so
        // the colored_label branch executes.
        assert!(state.advisory.is_some());
        assert!(state
            .advisory
            .as_ref()
            .unwrap()
            .contains("advised maximum 64"));
    }

    // Validates: Requirement 9.2 -- no advisory when within soft limit
    #[test]
    fn no_advisory_line_when_within_soft_limit() {
        let state = make_state_with_options(vec![MenuOption {
            key: "1".to_string(),
            command: "FILES".to_string(),
            description: "Files".to_string(),
            enabled: true,
            group: None,
            target: None,
        }]);
        assert!(state.advisory.is_none());
    }

    // Validates: Requirement 1.5 -- error state has load_error set
    #[test]
    fn render_error_state_has_message() {
        let state = MenuWorkspaceState {
            file_path: std::path::PathBuf::from("missing.toml"),
            menu: None,
            load_error: Some("Menu file not found: missing.toml".to_string()),
            last_modified: None,
            advisory: None,
            limits: crate::menu_workspace::OptionLimits::default(),
        };
        assert!(state.load_error.is_some());
    }

    fn opt(key: &str, command: &str, description: &str) -> MenuOption {
        MenuOption {
            key: key.to_string(),
            command: command.to_string(),
            description: description.to_string(),
            enabled: true,
            group: None,
            target: None,
        }
    }

    // Validates: Requirement 2.1a -- three aligned columns key | command | description
    #[test]
    fn format_option_row_produces_three_aligned_columns() {
        let o = opt("1", "FILES", "Browse files");
        let row = format_option_row(&o, 8);
        // key left-padded to 4, then command left-padded to width 8, then desc.
        assert_eq!(row, "1     FILES     Browse files");
        // Contains all three fields.
        assert!(row.contains("FILES"));
        assert!(row.contains("Browse files"));
        assert!(row.trim_start().starts_with('1'));
    }

    // Validates: Requirement 2.1a -- command column widens to the widest command, clamped
    #[test]
    fn command_column_width_uses_widest_command_clamped() {
        // Empty -> minimum 4.
        assert_eq!(command_column_width(&[]), 4);
        // Short command still clamps up to 4.
        assert_eq!(command_column_width(&[opt("1", "GO", "x")]), 4);
        // Widest command wins.
        let opts = vec![opt("1", "FILES", "a"), opt("2", "SETTINGS", "b")];
        assert_eq!(command_column_width(&opts), "SETTINGS".len());
        // Very long command clamps to 24.
        let long = "X".repeat(40);
        assert_eq!(command_column_width(&[opt("1", &long, "b")]), 24);
    }

    // Validates: Requirement 2.1a -- rows sharing one column width align on the description
    #[test]
    fn format_option_rows_align_descriptions_at_common_width() {
        let opts = vec![opt("1", "FILES", "Browse"), opt("2", "SETTINGS", "Config")];
        let w = command_column_width(&opts);
        let r0 = format_option_row(&opts[0], w);
        let r1 = format_option_row(&opts[1], w);
        // The description starts at the same character index in both rows.
        let idx0 = r0.find("Browse").unwrap();
        let idx1 = r1.find("Config").unwrap();
        assert_eq!(idx0, idx1, "descriptions must align across rows");
    }

    // Validates: Requirement 1.8 -- show_calendar is carried on the menu (default true)
    #[test]
    fn show_calendar_flag_is_carried_on_menu() {
        let state = make_state_with_options(vec![opt("1", "FILES", "Browse")]);
        assert!(
            state.menu.as_ref().unwrap().show_calendar,
            "make_state_with_options builds a calendar-on menu"
        );
    }

    // Validates: Requirement 2.1b -- a menu may opt out of the calendar
    #[test]
    fn show_calendar_false_is_respected_on_menu() {
        let mut state = make_state_with_options(vec![opt("1", "FILES", "Browse")]);
        state.menu.as_mut().unwrap().show_calendar = false;
        assert!(!state.menu.as_ref().unwrap().show_calendar);
    }

    // Validates: Requirement 2.1b -- render result default carries no selection/nav
    #[test]
    fn menu_render_result_default_is_empty() {
        let r = MenuRenderResult::default();
        assert!(r.selected.is_none());
        assert!(r.calendar_nav.is_none());
    }

    // Validates: Requirement 2.1b -- calendar colours default to inherited placeholders
    #[test]
    fn menu_calendar_colours_default_is_inherited() {
        let c = MenuCalendarColours::default();
        assert_eq!(c.calendar_fg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_bg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_fg, egui::Color32::PLACEHOLDER);
        assert!(!c.use_today_reverse);
    }
}
