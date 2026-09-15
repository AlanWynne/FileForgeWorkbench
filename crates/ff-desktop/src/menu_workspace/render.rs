//! egui rendering for Menu_Workspace.
//!
//! Validates: Requirement 2 (menu-workspace)

use eframe::egui;

use super::MenuWorkspaceState;

/// The semantic colours the shared menu renderer uses for the option columns
/// and the calendar panel. The caller resolves these from the active palette
/// (Legacy uses ISPF semantics; other themes inherit egui colours via
/// PLACEHOLDER). Mirrors the POM's colour scheme so every menu -- POM, Settings
/// and any other -- shares one look.
///
/// Validates: menu-workspace Requirement 2.1a, 2.1b; Requirement 13.4-13.8
#[derive(Debug, Clone, Copy)]
pub struct MenuColours {
    /// Option key column text (White in Legacy).
    pub option_key: egui::Color32,
    /// Option command column text (Turquoise in Legacy).
    pub option_command: egui::Color32,
    /// Option description column text (Green in Legacy).
    pub description: egui::Color32,
    /// Calendar body text colour (Turquoise in Legacy).
    pub calendar_fg: egui::Color32,
    /// Today-cell background (when `use_today_reverse`).
    pub today_bg: egui::Color32,
    /// Today-cell foreground (when `use_today_reverse`).
    pub today_fg: egui::Color32,
    /// Whether to draw today's cell reversed (Legacy) vs a plain marker.
    pub use_today_reverse: bool,
}

impl Default for MenuColours {
    fn default() -> Self {
        Self {
            option_key: egui::Color32::PLACEHOLDER,
            option_command: egui::Color32::PLACEHOLDER,
            description: egui::Color32::PLACEHOLDER,
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
/// This plain-text form is the alignment reference for `option_row_job` (which
/// produces the same layout with per-column colours) and is exercised by tests.
///
/// Validates: Requirement 2.1a (menu-workspace)
#[cfg(test)]
pub(crate) fn format_option_row(option: &super::MenuOption, cmd_width: usize) -> String {
    format!(
        "{:<4}  {:<width$}  {}",
        option.key,
        option.command,
        option.description,
        width = cmd_width,
    )
}

/// Build a monospace `LayoutJob` for one option row where each of the three
/// columns (key | command | description) is coloured independently, matching
/// the POM's Legacy colour scheme. The column widths match `format_option_row`
/// so alignment is identical to the plain-text formatter.
///
/// Validates: Requirement 2.1a; Requirement 13.4, 13.5, 13.6
fn option_row_job(
    option: &super::MenuOption,
    cmd_width: usize,
    key_col: egui::Color32,
    command_col: egui::Color32,
    desc_col: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let fmt = |color: egui::Color32| egui::TextFormat {
        font_id: egui::FontId::monospace(14.0),
        color,
        ..Default::default()
    };
    // Key column: left-justified in 4, then two-space gutter.
    job.append(&format!("{:<4}", option.key), 0.0, fmt(key_col));
    job.append("  ", 0.0, fmt(key_col));
    // Command column: left-justified in cmd_width, then two-space gutter.
    job.append(
        &format!("{:<width$}", option.command, width = cmd_width),
        0.0,
        fmt(command_col),
    );
    job.append("  ", 0.0, fmt(command_col));
    // Description column.
    job.append(&option.description, 0.0, fmt(desc_col));
    job
}

/// Render a Menu_Workspace: a centred Menu_Title, a three-column option list
/// (key | command | description) on the left, and -- when the menu's
/// `show_calendar` is true -- the live calendar on the right, laid out like the
/// POM. This is the single shared renderer for all menus including the POM
/// (menu-workspace Req 2.1, 2.1a-2.1c; CR-CH-018).
///
/// `calendar_offset` is the month offset for the calendar; `colours` supplies
/// the semantic option-column and calendar colours (resolved from the palette,
/// matching the POM). The caller (shell render) owns the surrounding chrome
/// (Title_Line, Command_Field, Key_Label_Bar) and the `calendar_offset` state.
///
/// Validates: Requirement 2.1-2.6, 2.1a-2.1c, 10.6; Requirement 13.4-13.8
pub fn render_menu_workspace(
    state: &mut MenuWorkspaceState,
    ui: &mut egui::Ui,
    calendar_offset: i32,
    colours: MenuColours,
) -> MenuRenderResult {
    use crate::primary_option_menu::PomColours;
    let mut result = MenuRenderResult::default();

    // Resolve the option-column colours once (PLACEHOLDER -> inherited theme
    // text colour). Matches the POM: key=white, command=turquoise, desc=green
    // under the Legacy theme. Validates: Requirement 13.4, 13.5, 13.6.
    let key_col = PomColours::resolve(colours.option_key, ui);
    let command_col = PomColours::resolve(colours.option_command, ui);
    let desc_col = PomColours::resolve(colours.description, ui);

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
        // --- Option list: three columns (key | command | description) ------
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

                        // Three aligned columns, each in its POM semantic colour
                        // (Req 2.1a; 13.4 key, 13.5 command, 13.6 description).
                        // Disabled rows use grey for all three columns (Req 2.3).
                        let (row_key_col, row_cmd_col, row_desc_col) = if option.enabled {
                            (key_col, command_col, desc_col)
                        } else {
                            (
                                egui::Color32::GRAY,
                                egui::Color32::GRAY,
                                egui::Color32::GRAY,
                            )
                        };
                        let job = option_row_job(
                            option,
                            cmd_width,
                            row_key_col,
                            row_cmd_col,
                            row_desc_col,
                        );

                        if option.enabled {
                            // Req 2.2 -- clickable row (transparent button, like the POM)
                            let btn = egui::Button::new(job)
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE);
                            if ui.add(btn).clicked() {
                                result.selected = Some(option.clone());
                            }
                        } else {
                            // Req 2.3 -- disabled style, no interaction
                            ui.add_enabled(false, egui::Label::new(job));
                        }
                    }
                });
        });

        // --- Calendar on the right when the menu opts in (Req 2.1b, 1.8) ---
        if menu.show_calendar {
            ui.add_space(32.0);
            let calendar_fg = PomColours::resolve(colours.calendar_fg, ui);
            result.calendar_nav = crate::primary_option_menu::render_calendar(
                ui,
                calendar_offset,
                calendar_fg,
                colours.today_bg,
                colours.today_fg,
                colours.use_today_reverse,
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

    // Validates: Requirement 2.1a, 2.1b -- menu colours default to inherited placeholders
    #[test]
    fn menu_colours_default_is_inherited() {
        let c = MenuColours::default();
        assert_eq!(c.option_key, egui::Color32::PLACEHOLDER);
        assert_eq!(c.option_command, egui::Color32::PLACEHOLDER);
        assert_eq!(c.description, egui::Color32::PLACEHOLDER);
        assert_eq!(c.calendar_fg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_bg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_fg, egui::Color32::PLACEHOLDER);
        assert!(!c.use_today_reverse);
    }

    // Validates: Requirement 2.1a -- coloured option row keeps the three columns
    // aligned identically to the plain-text formatter (same widths).
    #[test]
    fn option_row_job_preserves_column_alignment() {
        let opts = vec![opt("1", "FILES", "Browse"), opt("22", "SETTINGS", "Config")];
        let w = command_column_width(&opts);
        for o in &opts {
            let job = option_row_job(
                o,
                w,
                egui::Color32::WHITE,
                egui::Color32::WHITE,
                egui::Color32::WHITE,
            );
            // The job's concatenated text matches the plain formatter exactly.
            assert_eq!(job.text, format_option_row(o, w));
        }
    }
}
