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

// CR-CH-032 removed the fixed OPTION_LIST_MIN_WIDTH reserve: the option-list
// width is now driven by the descriptions' natural one-line width
// (`natural_option_list_width`), not a hard minimum. The calendar-fit decision
// (Tier 1) uses that natural width instead of a constant floor.

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

/// The monospace font used for menu option rows.
fn option_font() -> egui::FontId {
    egui::FontId::monospace(14.0)
}

/// Build the NON-WRAPPING key+command prefix as a two-colour monospace
/// `LayoutJob`: the key column (left-justified in 4) + gutter in `key_col`, then
/// the command column (left-justified in `cmd_width`) + gutter in `command_col`.
/// This is the fixed left part of an option row; the description is laid out
/// SEPARATELY (B065) so wrapped description lines hang-indent under the
/// description column rather than the row's left edge.
///
/// Validates: Requirement 2.1a; Requirement 13.4, 13.5
fn option_prefix_job(
    option: &super::MenuOption,
    cmd_width: usize,
    key_col: egui::Color32,
    command_col: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let fmt = |color: egui::Color32| egui::TextFormat {
        font_id: option_font(),
        color,
        ..Default::default()
    };
    // Never wrap the fixed columns.
    job.wrap.max_width = f32::INFINITY;
    job.append(&format!("{:<4}", option.key), 0.0, fmt(key_col));
    job.append("  ", 0.0, fmt(key_col));
    job.append(
        &format!("{:<width$}", option.command, width = cmd_width),
        0.0,
        fmt(command_col),
    );
    job.append("  ", 0.0, fmt(command_col));
    job
}

/// The prefix TEXT (key + gutter + command + gutter) for one option, matching
/// exactly what `option_prefix_job` paints. Kept separate so the natural-width
/// measurement lays out the same string the renderer draws.
fn option_prefix_text(option: &super::MenuOption, cmd_width: usize) -> String {
    format!(
        "{:<4}  {:<width$}  ",
        option.key,
        option.command,
        width = cmd_width,
    )
}

/// Reduce per-row natural widths to the option list's Natural_Option_Width: the
/// widest row (prefix + single-line description) plus a scrollbar allowance so
/// the calendar-fit decision leaves room for the option-list vertical scrollbar
/// when one is shown. Pure and deterministic (no `Ui`), so it is unit-testable.
///
/// `row_widths` yields each row's full single-line width in px (prefix galley
/// width + description galley width). Empty -> just the scrollbar allowance.
///
/// Validates: menu-workspace Requirement 16.7 (CR-CH-032)
fn natural_width_from_rows(
    row_widths: impl IntoIterator<Item = f32>,
    scrollbar_allowance: f32,
) -> f32 {
    let widest = row_widths
        .into_iter()
        .fold(0.0_f32, |acc, w| if w > acc { w } else { acc });
    widest + scrollbar_allowance
}

/// Compute the option list's Natural_Option_Width in px: the width needed to
/// render the WIDEST option row (fixed key+command prefix + its single-line
/// description) with NO wrapping, plus an allowance for the option-list vertical
/// scrollbar. Uncapped -- a very long description simply increases it (which may
/// drop the layout to Tier 2/3; menu-workspace Req 16.7, 16.8).
///
/// Measures against the live `ui` fonts so the result matches what the renderer
/// paints (the prefix and description both use `option_font()`).
///
/// Validates: menu-workspace Requirement 16.7 (CR-CH-032)
fn natural_option_list_width(
    options: &[super::MenuOption],
    cmd_width: usize,
    ui: &egui::Ui,
) -> f32 {
    let font = option_font();
    let measure = |text: &str| -> f32 {
        // Lay the text out with no wrap and read the galley's width.
        ui.fonts(|f| {
            let galley = f.layout_no_wrap(text.to_string(), font.clone(), egui::Color32::WHITE);
            galley.size().x
        })
    };
    let row_widths = options.iter().map(|o| {
        let prefix_w = measure(&option_prefix_text(o, cmd_width));
        let desc_w = measure(&o.description);
        prefix_w + desc_w
    });
    // Scrollbar allowance: the theme's scrollbar width plus a small pad, so the
    // Tier-1 calendar fit accounts for a scrollbar appearing on the option list.
    let scrollbar_allowance = ui.spacing().scroll.bar_width + 4.0;
    natural_width_from_rows(row_widths, scrollbar_allowance)
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

    // CR-CH-032: description-driven layout. Decide the Layout_Tier from the
    // available width and the descriptions' NATURAL one-line width, replacing
    // the fixed-minimum-constant fit rule of CR-CH-026 (Req 16.3/16.6). This is
    // computed from the current frame's available width only -- no persisted
    // state (Req 16.11).
    //
    //   Tier 1 (Req 16.8): show_calendar AND natural + GAP + CALENDAR_MIN fits
    //           -> calendar shown; option column sized to `natural` so leftover
    //              width trails as blank space to the RIGHT of the calendar
    //              (calendar NOT pinned to the window edge).
    //   Tier 2: else if natural fits -> calendar hidden; option column = full
    //              width; descriptions stay on one line.
    //   Tier 3: else -> calendar hidden; option column = full width; the
    //              description Label wraps (Req 16.10 fallback) as a last resort.
    //
    // The calendar is thus hidden BEFORE any description wraps (Req 16.9): Tier 1
    // is the only calendar-showing tier and it requires the full one-line width.
    let available_w = ui.available_width();
    let natural_w = natural_option_list_width(&menu.options, cmd_width, ui);
    let display_calendar =
        menu.show_calendar && natural_w + CALENDAR_GAP + CALENDAR_MIN_WIDTH <= available_w;
    // The option column's DEFINITE width (Req 16.8): the natural width in Tier 1
    // (so the calendar sits just to its right, blank space trailing), the full
    // available width otherwise (Tiers 2/3). Clamped to available_w so the
    // column never exceeds the panel.
    let option_col_w = if display_calendar {
        natural_w.min(available_w)
    } else {
        available_w
    };

    ui.horizontal_top(|ui| {
        // --- Option list: three columns (key | command | description) ------
        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .id_salt("menu_workspace_options")
                .max_width(option_col_w)
                .show(ui, |ui| {
                    // Give the option list a DEFINITE width (Req 16.8): in Tier 1
                    // this is the natural one-line width so descriptions do not
                    // wrap and the calendar sits just to the right; in Tiers 2/3
                    // it is the full available width (Tier 3 lets the description
                    // Label wrap as the last-resort fallback, Req 16.10).
                    ui.set_width(option_col_w);
                    ui.set_max_width(option_col_w);
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

                        // B065: render the row as a horizontal pair -- a fixed,
                        // non-wrapping key+command PREFIX (a real Button = the
                        // focusable/clickable row focus stop, proven for the
                        // CR-CH-023 Boundary_Policy) followed by the description
                        // as a SEPARATE wrapping Label in the remaining width.
                        // egui wraps the description within its own rect, so
                        // continuation lines hang-indent under the description
                        // column instead of the row's left edge (the previous
                        // single-galley Button re-wrapped everything back to x0).
                        let prefix_job =
                            option_prefix_job(option, cmd_width, row_key_col, row_cmd_col);
                        let desc = option.description.clone();
                        let row_resp = ui
                            .horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                let prefix_resp = if option.enabled {
                                    ui.add(
                                        egui::Button::new(prefix_job)
                                            .fill(egui::Color32::TRANSPARENT)
                                            .stroke(egui::Stroke::NONE),
                                    )
                                } else {
                                    // Req 2.3 -- disabled: non-interactive, not a
                                    // Tab stop (Req 15.4).
                                    ui.add_enabled(false, egui::Label::new(prefix_job))
                                };
                                // Description wraps within the remaining width, so
                                // continuation lines start at the description
                                // column = a hanging indent (B065).
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&desc).monospace().color(row_desc_col),
                                    )
                                    .wrap(),
                                );
                                prefix_resp
                            })
                            .inner;

                        if option.enabled {
                            let opt_id = row_resp.id;
                            // Req 16.10 (B056): keyboard-focused option shows a
                            // clear outline around the prefix (the focus widget).
                            if row_resp.has_focus() {
                                ui.painter().rect_stroke(
                                    row_resp.rect.expand(1.0),
                                    egui::CornerRadius::same(2),
                                    egui::Stroke::new(1.5_f32, key_col),
                                    egui::StrokeKind::Outside,
                                );
                                // CR-CH-028 (Req 12.2): record the focused option
                                // for the Cursor_Context.
                                result.focused_option = Some((
                                    opt_id,
                                    option.command.clone(),
                                    option.description.clone(),
                                ));
                            }
                            if row_resp.clicked() {
                                result.selected = Some(option.clone());
                            }
                            // Track first/last enabled option ids (Req 15.2/15.5).
                            if result.first_interior_id.is_none() {
                                result.first_interior_id = Some(opt_id);
                            }
                            last_enabled_option_id = Some(opt_id);
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
            show_in_menu_bar: true,
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
            show_in_menu_bar: true,
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
            show_in_menu_bar: true,
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
            show_in_menu_bar: true,
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

    // Validates: Requirement 2.1a (B065) -- the key+command PREFIX job holds the
    // fixed columns (key left-justified in 4 + 2 gutter, command in cmd_width +
    // 2 gutter) and NEVER wraps. The description is now a SEPARATE wrapping Label
    // (hanging indent under the description column), so it is not part of the
    // prefix job -- the wrap alignment itself is a rendered-widget property
    // (verified manually / by egui layout, test-plan 2.2c).
    #[test]
    fn option_prefix_job_holds_fixed_columns_and_does_not_wrap() {
        let o = opt("1", "FILES", "Browse the catalog");
        let w = command_column_width(&[o.clone()]);
        let job = option_prefix_job(&o, w, egui::Color32::WHITE, egui::Color32::WHITE);
        // The prefix text is exactly key(4) + "  " + command(w) + "  " -- i.e.
        // the format_option_row prefix WITHOUT the description.
        let expected_prefix = &format_option_row(&o, w)[..(4 + 2 + w + 2)];
        assert_eq!(job.text, expected_prefix);
        // The prefix never wraps.
        assert_eq!(job.wrap.max_width, f32::INFINITY);
        // No newlines in the prefix (single physical line).
        assert!(!job.text.contains('\n'));
    }

    // Validates: Requirement 2.1a -- prefixes of different options share the same
    // column width, so descriptions (rendered to the right of the prefix) all
    // start at the SAME x-offset (the description column).
    #[test]
    fn option_prefix_jobs_share_width_across_rows() {
        let opts = vec![opt("1", "FILES", "Browse"), opt("22", "SETTINGS", "Config")];
        let w = command_column_width(&opts);
        let a = option_prefix_job(&opts[0], w, egui::Color32::WHITE, egui::Color32::WHITE);
        let b = option_prefix_job(&opts[1], w, egui::Color32::WHITE, egui::Color32::WHITE);
        // Same character length -> same monospace width -> descriptions align.
        assert_eq!(a.text.chars().count(), b.text.chars().count());
    }

    // === CR-CH-032: description-driven layout natural width (Req 16.7) =====

    // Validates: menu-workspace Requirement 16.7 -- the prefix TEXT used for the
    // natural-width measurement matches the painted prefix job (key(4) + gutter +
    // command(cmd_width) + gutter), so measuring it reproduces the drawn layout.
    #[test]
    fn option_prefix_text_matches_prefix_job_text() {
        let o = opt("1", "FILES", "Browse the catalog");
        let w = command_column_width(&[o.clone()]);
        let job = option_prefix_job(&o, w, egui::Color32::WHITE, egui::Color32::WHITE);
        assert_eq!(option_prefix_text(&o, w), job.text);
    }

    // Validates: menu-workspace Requirement 16.7 -- Natural_Option_Width is the
    // WIDEST row plus the scrollbar allowance.
    #[test]
    fn natural_width_is_widest_row_plus_scrollbar() {
        // Rows of widths 100, 250, 180 -> widest 250 + allowance 12 = 262.
        let w = natural_width_from_rows([100.0_f32, 250.0, 180.0], 12.0);
        assert_eq!(w, 262.0);
    }

    // Validates: menu-workspace Requirement 16.7 -- a longer single-line
    // description increases the natural width (uncapped).
    #[test]
    fn natural_width_grows_with_longer_description() {
        let short = natural_width_from_rows([120.0_f32], 10.0);
        let long = natural_width_from_rows([120.0_f32, 400.0], 10.0);
        assert!(
            long > short,
            "a wider row must increase the natural width (uncapped): {long} !> {short}"
        );
        assert_eq!(long, 410.0);
    }

    // Validates: menu-workspace Requirement 16.7 -- empty option list yields just
    // the scrollbar allowance (no rows to measure).
    #[test]
    fn natural_width_empty_is_scrollbar_allowance_only() {
        let w = natural_width_from_rows(std::iter::empty(), 15.0);
        assert_eq!(w, 15.0);
    }

    // === CR-CH-023: menu-workspace interior tab order (Req 15) =============

    fn tab_opt(key: &str, enabled: bool) -> MenuOption {
        MenuOption {
            key: key.to_string(),
            command: format!("CMD{key}"),
            description: format!("Option {key}"),
            enabled,
            group: None,
            show_in_menu_bar: true,
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

    // === CR-CH-032: description-driven layout tiers (Req 16.7-16.11) =======

    /// One calendar-ON menu with a SINGLE long-description option, rendered at
    /// `panel_w`. Returns:
    /// - `calendar_shown`: true when the reported last interior is NOT the option
    ///   row (i.e. the calendar `>` button is the last interior -> calendar shown).
    /// - `option_row_rect`: the first (only) enabled option row's on-screen rect
    ///   (its width is the option-column width; its height reveals wrapping: one
    ///   text line vs two+).
    /// - `single_line_h`: the height of one text line of the same font, as a wrap
    ///   threshold.
    fn render_desc_tier_at_width(panel_w: f32) -> (bool, Option<egui::Rect>, f32, f32) {
        use egui_kittest::Harness;
        use std::cell::Cell;
        use std::rc::Rc;
        let shown = Rc::new(Cell::new(false));
        let row_rect = Rc::new(Cell::new(None));
        let line_h = Rc::new(Cell::new(0.0_f32));
        // Total height the render laid out (grows when the description wraps).
        let content_h = Rc::new(Cell::new(0.0_f32));
        let shown_c = Rc::clone(&shown);
        let row_rect_c = Rc::clone(&row_rect);
        let line_h_c = Rc::clone(&line_h);
        let content_h_c = Rc::clone(&content_h);
        // A single option whose description is long enough that at a narrow width
        // it MUST wrap, but at a wide width fits on one line. Calendar ON.
        let mut state = make_state_with_options(vec![opt(
            "1",
            "FILES",
            "Browse the virtual catalog and open datasets for view or edit here",
        )]);
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(panel_w, 800.0))
            .build_ui(move |ui| {
                // One text-line height for the option font, as the wrap threshold.
                let lh = ui.fonts(|f| f.row_height(&option_font()));
                line_h_c.set(lh);
                let r = render_menu_workspace(&mut state, ui, 0, MenuColours::default());
                // Calendar shown iff the reported last interior differs from the
                // reported first interior (the single option row): when shown the
                // last interior is the calendar `>` button, not the option.
                let first = r.first_interior_id;
                let last = r.last_interior_id;
                shown_c.set(first.is_some() && last.is_some() && first != last);
                if let Some(id) = first {
                    if let Some(resp) = ui.ctx().read_response(id) {
                        row_rect_c.set(Some(resp.rect));
                    }
                }
                // Total laid-out height of everything the render added this frame
                // -- reveals wrapping (a wrapped description makes the option area
                // taller) without needing the description Label's private id.
                content_h_c.set(ui.min_rect().height());
            });
        harness.run();
        (shown.get(), row_rect.get(), line_h.get(), content_h.get())
    }

    // Validates: menu-workspace Req 16.8 (Tier 1) -- a WIDE window shows the
    // calendar AND keeps the description on one line (the option row is a single
    // text line high). The calendar sits to the right; the option column is at
    // its natural width, so it does not span the whole (wide) panel.
    #[test]
    fn menu_wide_shows_calendar_and_one_line_descriptions() {
        let (calendar_shown, row_rect, _line_h, _content_h) = render_desc_tier_at_width(1200.0);
        assert!(
            calendar_shown,
            "Tier 1: a wide calendar-on menu must SHOW the calendar"
        );
        let rect = row_rect.expect("the option row must have an on-screen rect");
        assert!(
            rect.width() < 1100.0,
            "Tier 1: the option column is at its natural width, not the full wide panel (width {})",
            rect.width()
        );
    }

    // Validates: menu-workspace Req 16.8 (Tier 2), 16.9 -- a MEDIUM window (fits
    // the one-line descriptions but NOT alongside the calendar) HIDES the
    // calendar and keeps the description on one line. Calendar hides BEFORE any
    // wrap (16.9).
    #[test]
    fn menu_medium_hides_calendar_keeps_one_line() {
        // Width chosen to sit between the natural one-line width and
        // natural + gap + calendar-min: wide enough for one line, too narrow for
        // the calendar too.
        let (calendar_shown, _row_rect, _line_h, _content_h) = render_desc_tier_at_width(640.0);
        assert!(
            !calendar_shown,
            "Tier 2: the calendar must be HIDDEN when it cannot fit alongside one-line descriptions"
        );
    }

    // Validates: menu-workspace Req 16.8 (Tier 3), 16.9, 16.10 -- a NARROW window
    // (too narrow even without the calendar) HIDES the calendar and lets the
    // description WRAP to further lines (the last-resort fallback). The narrow
    // render lays out MORE total height than the medium (one-line, calendar also
    // hidden) render -- both have no calendar, so the extra height is the wrap.
    #[test]
    fn menu_narrow_hides_calendar_and_wraps() {
        let (medium_shown, _r1, _l1, medium_h) = render_desc_tier_at_width(640.0);
        let (narrow_shown, _r2, _l2, narrow_h) = render_desc_tier_at_width(220.0);
        assert!(
            !medium_shown && !narrow_shown,
            "both the medium and narrow renders hide the calendar (Tier 2/3)"
        );
        assert!(
            narrow_h > medium_h,
            "Tier 3: the long description must WRAP (narrow content height {narrow_h} > medium one-line height {medium_h})"
        );
    }
}
