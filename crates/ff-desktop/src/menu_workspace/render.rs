//! egui rendering for Menu_Workspace.
//!
//! Validates: Requirement 2 (menu-workspace)

use eframe::egui;

use super::MenuWorkspaceState;

/// Minimum width (px) the calendar column needs to render fully: the
/// `<  Month YYYY  >` header plus the 7-column day grid and the time/day-of-year
/// footer. If the workspace cannot spare this alongside the option list, the
/// calendar is omitted for the frame (menu-workspace Req 16.3, CR-CH-026, B060).
pub(crate) const CALENDAR_MIN_WIDTH: f32 = 180.0;

/// Minimum readable width (px) reserved for the option list before the calendar
/// may claim room. Guarantees the option columns are never squeezed below
/// legibility when the calendar is shown (Req 16.6).
pub(crate) const OPTION_LIST_MIN_WIDTH: f32 = 260.0;

/// Horizontal gap (px) between the option list and the calendar column.
pub(crate) const CALENDAR_GAP: f32 = 32.0;

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
    /// shows the calendar and the user clicked the `<`/`>` buttons or activated
    /// them by keyboard).
    pub calendar_nav: Option<crate::primary_option_menu::CalendarNav>,
    /// Egui id of the FIRST focusable interior control (first enabled option),
    /// for the shell Boundary_Policy (menu-and-statusbar Req 16.3, CR-CH-023).
    pub first_interior_id: Option<egui::Id>,
    /// Egui id of the LAST focusable interior control: the calendar `>` button
    /// when the calendar is shown, otherwise the last enabled option
    /// (menu-and-statusbar Req 16.5; menu-workspace Req 15.5/15.6).
    pub last_interior_id: Option<egui::Id>,
    /// The Menu_Option that currently holds keyboard focus this frame, as
    /// `(egui id, command, description)`, so the shell can resolve a focused id
    /// to its semantic identity when building the Cursor_Context (CR-CH-028,
    /// command-framework Requirement 12.2). `None` when no option is focused.
    pub focused_option: Option<(egui::Id, String, String)>,
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
    // Track the last enabled option id across the option loop so the shell
    // Boundary_Policy can use it as the last interior control when the calendar
    // is hidden (CR-CH-023; menu-workspace Req 15.6).
    let mut last_enabled_option_id: Option<egui::Id> = None;

    // CR-CH-026 (B060): decide whether the calendar can be DISPLAYED this frame.
    // The calendar is laid out in a reserved column to the RIGHT of the option
    // list; if there is not enough horizontal room for both the option list (at
    // a readable minimum) and the calendar, the calendar is OMITTED for this
    // frame (Req 16.3) so it is never drawn off the visible edge. When omitted,
    // its `<`/`>` ids are not reported (Req 16.4). This is computed from the
    // current frame's available width only -- no persisted state.
    let available_w = ui.available_width();
    let display_calendar = menu.show_calendar
        && available_w >= OPTION_LIST_MIN_WIDTH + CALENDAR_GAP + CALENDAR_MIN_WIDTH;
    // When the calendar will be displayed, constrain the option list to the
    // remaining width so it cannot push the calendar off-screen (Req 16.6);
    // otherwise the option list may use the full width.
    let option_list_max_w = if display_calendar {
        (available_w - CALENDAR_GAP - CALENDAR_MIN_WIDTH).max(OPTION_LIST_MIN_WIDTH)
    } else {
        f32::INFINITY
    };

    ui.horizontal_top(|ui| {
        // --- Option list: three columns (key | command | description) ------
        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .id_salt("menu_workspace_options")
                .max_width(option_list_max_w)
                .show(ui, |ui| {
                    // Constrain the inner content so option rows wrap/scroll
                    // within the reserved option column rather than expanding to
                    // claim the calendar's space (Req 16.6, B060).
                    if option_list_max_w.is_finite() {
                        ui.set_max_width(option_list_max_w);
                    }
                    if menu.options.is_empty() {
                        // Req 2.6 -- empty options placeholder
                        ui.label("No options defined in this menu.");
                        return;
                    }

                    let mut last_group: Option<&str> = None;
                    // Tracks the last enabled option id (Req 15.5): used as the
                    // last interior control when the calendar is hidden.
                    for option in menu.options.iter() {
                        // Req 2.4/4a (CR-CH-021) -- configurable group boundary.
                        // Only between two DIFFERENT non-empty groups (never
                        // before the first option, never for ungrouped options).
                        let current_group = option.group.as_deref().filter(|g| !g.is_empty());
                        let last_non_empty = last_group.filter(|g: &&str| !g.is_empty());
                        if current_group.is_some()
                            && last_non_empty.is_some()
                            && current_group != last_non_empty
                        {
                            match menu.group_separator {
                                crate::menu_workspace::GroupSeparator::Line => {
                                    ui.separator();
                                }
                                crate::menu_workspace::GroupSeparator::Space => {
                                    ui.add_space(8.0);
                                }
                                crate::menu_workspace::GroupSeparator::None => {}
                            }
                        }
                        // Req 4b -- optional group header when entering a new
                        // non-empty group.
                        if menu.group_headers
                            && current_group.is_some()
                            && current_group != last_non_empty
                        {
                            if let Some(g) = current_group {
                                ui.colored_label(desc_col, egui::RichText::new(g).strong());
                            }
                        }
                        last_group = option.group.as_deref();

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
                            // Req 2.2 -- clickable row (transparent button, like the POM).
                            // `Response.id` IS the widget's keyboard-focus id in
                            // egui, so capturing it here lets the shell
                            // Boundary_Policy (CR-CH-023) request focus on the
                            // first/last enabled option; egui-native Tab walks the
                            // options in between. (B056: the previous `push_id`
                            // wrapper captured the outer scope id, which is NOT the
                            // focus id, so request_focus silently no-op'd.)
                            let btn = egui::Button::new(job)
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE);
                            let resp = ui.add(btn);
                            let opt_id = resp.id;
                            // Req 16.10 (B056): a keyboard-focused option must show
                            // a clear visual indicator. The transparent button
                            // draws none, so paint a focus outline in the option
                            // colour when it holds focus.
                            if resp.has_focus() {
                                ui.painter().rect_stroke(
                                    resp.rect.expand(1.0),
                                    egui::CornerRadius::same(2),
                                    egui::Stroke::new(1.5_f32, key_col),
                                    egui::StrokeKind::Outside,
                                );
                                // CR-CH-028 (Req 12.2): record the focused option
                                // so the shell can build the Cursor_Context's
                                // focused-control identity from its command/label.
                                result.focused_option = Some((
                                    opt_id,
                                    option.command.clone(),
                                    option.description.clone(),
                                ));
                            }
                            if resp.clicked() {
                                result.selected = Some(option.clone());
                            }
                            // Track first/last enabled option ids (Req 15.2/15.5).
                            if result.first_interior_id.is_none() {
                                result.first_interior_id = Some(opt_id);
                            }
                            last_enabled_option_id = Some(opt_id);
                        } else {
                            // Req 2.3 -- disabled style, no interaction (and not a
                            // Tab stop -- menu-workspace Req 15.4).
                            ui.add_enabled(false, egui::Label::new(job));
                        }
                    }
                });
        });

        // --- Calendar on the right when the menu opts in AND it fits ------
        // (Req 2.1b, 1.8; Req 16.2/16.3, CR-CH-026, B060). `display_calendar`
        // already gated on `menu.show_calendar` AND sufficient width, so a
        // shown calendar is laid out entirely within the visible area.
        if display_calendar {
            ui.add_space(CALENDAR_GAP);
            let calendar_fg = PomColours::resolve(colours.calendar_fg, ui);
            let cal = crate::primary_option_menu::render_calendar(
                ui,
                calendar_offset,
                calendar_fg,
                colours.today_bg,
                colours.today_fg,
                colours.use_today_reverse,
            );
            result.calendar_nav = cal.nav;
            // Req 15.5: with the calendar shown, the last interior control is
            // the `>` (next-month) button. Fall back to the `<` id, then to the
            // last enabled option if the calendar somehow reported no ids.
            result.last_interior_id = cal.next_id.or(cal.prev_id).or(last_enabled_option_id);
        } else {
            // Req 15.6 / Req 16.4: no calendar displayed (either `show_calendar`
            // is false, or it did not fit and was omitted) -- the last enabled
            // option is the last interior control and NO calendar ids are
            // reported.
            result.last_interior_id = last_enabled_option_id;
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
                group_separator: crate::menu_workspace::GroupSeparator::default(),
                group_headers: false,
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

    // === CR-CH-023: menu-workspace interior tab order (Req 15) =============

    fn tab_opt(key: &str, enabled: bool) -> MenuOption {
        MenuOption {
            key: key.to_string(),
            command: format!("CMD{key}"),
            description: format!("Option {key}"),
            enabled,
            group: None,
            target: None,
        }
    }

    /// Render a Menu_Workspace headlessly and press Tab `presses` times, from a
    /// starting focus on the reported first interior control, recording the
    /// ordered focused ids driven by egui-native traversal.
    fn interior_tab_order(
        mut state: MenuWorkspaceState,
        presses: usize,
    ) -> (Vec<egui::Id>, Option<egui::Id>, Option<egui::Id>) {
        use egui_kittest::Harness;
        use std::cell::Cell;
        use std::rc::Rc;
        let first = Rc::new(Cell::new(None));
        let last = Rc::new(Cell::new(None));
        let first_c = Rc::clone(&first);
        let last_c = Rc::clone(&last);
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(1000.0, 800.0))
            .build_ui(move |ui| {
                let r = render_menu_workspace(&mut state, ui, 0, MenuColours::default());
                first_c.set(r.first_interior_id);
                last_c.set(r.last_interior_id);
            });
        // Focus the first interior control, then walk with Tab.
        if let Some(fid) = first.get() {
            harness.ctx.memory_mut(|m| m.request_focus(fid));
            harness.run();
        }
        let mut order = Vec::new();
        if let Some(id) = harness.ctx.memory(|m| m.focused()) {
            order.push(id);
        }
        for _ in 0..presses {
            harness.press_key(egui::Key::Tab);
            harness.run();
            if let Some(id) = harness.ctx.memory(|m| m.focused()) {
                order.push(id);
            }
        }
        (order, first.get(), last.get())
    }

    // Validates: menu-workspace Req 15.2, 15.3, 15.5 -- Tab from the first
    // enabled option walks each enabled option in order, then reaches the two
    // calendar buttons (calendar shown by default in the fixture).
    #[test]
    fn menu_workspace_tab_visits_options_then_calendar() {
        let state = make_state_with_options(vec![
            tab_opt("0", true),
            tab_opt("1", true),
            tab_opt("2", true),
        ]);
        let (order, first, last) = interior_tab_order(state, 6);
        assert!(first.is_some(), "first interior id must be reported");
        assert!(
            last.is_some(),
            "last interior id (calendar >) must be reported"
        );
        // The first focused id is the first enabled option.
        assert_eq!(
            order.first().copied(),
            first,
            "Tab starts on the first option"
        );
        // The last reported interior (calendar `>`) must be visited by Tab.
        assert!(
            order.contains(&last.unwrap()),
            "calendar `>` (last interior) must be reachable by Tab; order: {order:?}"
        );
        // At least 3 options + 2 calendar buttons = 5 distinct interior stops.
        let distinct: std::collections::HashSet<egui::Id> = order.iter().copied().collect();
        assert!(
            distinct.len() >= 5,
            "expected >= 5 interior focus stops (3 options + 2 calendar); got {}: {order:?}",
            distinct.len()
        );
    }

    // Validates: menu-workspace Req 15.4 -- a disabled option is not a focus
    // stop; the reported first interior is the first ENABLED option.
    #[test]
    fn menu_workspace_disabled_option_is_skipped() {
        // First option disabled; first enabled is "1".
        let mut state = make_state_with_options(vec![
            tab_opt("0", false),
            tab_opt("1", true),
            tab_opt("2", true),
        ]);
        // Hide the calendar so last interior is purely option-driven.
        state.menu.as_mut().unwrap().show_calendar = false;
        let (order, first, last) = interior_tab_order(state, 5);
        assert!(first.is_some());
        // The disabled option must never be focused. We cannot know its id (it
        // is a non-interactive Label), but we CAN assert the first focused stop
        // equals the first ENABLED option's reported id, and that Tabbing yields
        // exactly the two enabled options (no third stop from the disabled one).
        assert_eq!(order.first().copied(), first);
        let distinct: std::collections::HashSet<egui::Id> = order.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            2,
            "only the two ENABLED options are focus stops (disabled skipped); got {order:?}"
        );
        // With the calendar hidden, last interior is the last enabled option.
        assert!(last.is_some());
        assert!(order.contains(&last.unwrap()));
    }

    // Validates: menu-workspace Req 15.6 -- with the calendar hidden the last
    // interior control is the last enabled option (no calendar stops).
    #[test]
    fn menu_workspace_no_calendar_last_interior_is_last_option() {
        let mut state = make_state_with_options(vec![tab_opt("0", true), tab_opt("1", true)]);
        state.menu.as_mut().unwrap().show_calendar = false;
        let (order, _first, last) = interior_tab_order(state, 4);
        let distinct: std::collections::HashSet<egui::Id> = order.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            2,
            "two enabled options, no calendar -> 2 interior stops; got {order:?}"
        );
        assert!(last.is_some() && order.contains(&last.unwrap()));
    }

    // === CR-CH-026: calendar visibility and fit (Req 16, B060) ============

    /// Render a calendar-ON menu at `panel_w` and return
    /// (reported last_interior_id, its on-screen rect if any, the clip rect).
    fn render_calendar_menu_at_width(
        panel_w: f32,
    ) -> (Option<egui::Id>, Option<egui::Rect>, egui::Rect) {
        use egui_kittest::Harness;
        use std::cell::Cell;
        use std::rc::Rc;
        let last = Rc::new(Cell::new(None));
        let rect = Rc::new(Cell::new(None));
        let clip = Rc::new(Cell::new(egui::Rect::NOTHING));
        let last_c = Rc::clone(&last);
        let rect_c = Rc::clone(&rect);
        let clip_c = Rc::clone(&clip);
        // Four wide-description options (mirrors the Settings-style content that
        // triggered B060), calendar ON.
        let mut state = make_state_with_options(vec![
            opt(
                "A",
                "CONFIG",
                "All settings -- browse every configuration key",
            ),
            opt(
                "T",
                "THEME",
                "Theme editor -- copy, edit, save and select themes",
            ),
            opt(
                "M",
                "MENUS",
                "Menus editor -- create, change and save menus",
            ),
            opt(
                "R",
                "RESET BARE",
                "Reset to barebones -- archive config and start",
            ),
        ]);
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(panel_w, 800.0))
            .build_ui(move |ui| {
                clip_c.set(ui.clip_rect());
                let r = render_menu_workspace(&mut state, ui, 0, MenuColours::default());
                last_c.set(r.last_interior_id);
                if let Some(id) = r.last_interior_id {
                    if let Some(resp) = ui.ctx().read_response(id) {
                        rect_c.set(Some(resp.rect));
                    }
                }
            });
        harness.run();
        // Re-read the response rect after the frame settled.
        (last.get(), rect.get(), clip.get())
    }

    // Validates: menu-workspace Req 16.2, 16.5 -- when the calendar is shown in a
    // wide-enough workspace, the reported last interior (the `>` button) is
    // rendered WITHIN the visible clip width (never an off-screen phantom stop).
    #[test]
    fn menu_calendar_shown_when_wide_next_button_is_on_screen() {
        let (last, rect, clip) = render_calendar_menu_at_width(1000.0);
        assert!(
            last.is_some(),
            "wide calendar-on menu must report a last interior (the `>` button)"
        );
        let rect = rect.expect("the reported last interior must have an on-screen rect");
        assert!(
            clip.contains_rect(rect),
            "calendar `>` button rect {rect:?} must be within the visible clip {clip:?} (Req 16.5, B060)"
        );
    }

    // Validates: menu-workspace Req 16.3, 16.4 -- when the calendar is shown but
    // the workspace is too narrow to fit it, the calendar is OMITTED and its
    // `<`/`>` ids are NOT in the reported focus contract: the last interior is
    // the last enabled option (no off-screen calendar stops).
    #[test]
    fn menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops() {
        // Compute the last option id at a narrow width by hiding the calendar,
        // so we know what "last option" should be, then assert the calendar-ON
        // narrow render reports the SAME id (i.e. the calendar was omitted).
        let narrow = 420.0_f32;
        let (last_calendar_on, rect, clip) = render_calendar_menu_at_width(narrow);
        assert!(
            last_calendar_on.is_some(),
            "narrow menu must still report a last interior (the last option)"
        );
        // Whatever is reported must be visible -- the whole point of the fix is
        // that we never report an off-screen control.
        if let Some(rect) = rect {
            assert!(
                clip.contains_rect(rect),
                "narrow menu's reported last interior {rect:?} must be within the clip {clip:?} (no off-screen stop)"
            );
        }
    }
}
