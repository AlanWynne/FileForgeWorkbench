//! # Shell Chrome Rendering
//!
//! Theme application, menu bar, and tab bar rendering for WorkbenchShell.

use eframe::egui;

use crate::primary_option_menu;
use crate::tab_state::TabKind;

use super::helpers::*;
use super::WorkbenchShell;

/// Map a Menu_Bar name to its file stem (menu-workspace Req 17.8, CR-NR-080),
/// matching the slugging the Menus editor uses for user menu names: lowercase,
/// non-alphanumerics replaced by `-`. So `MB-POM` -> `mb-pom` (loaded from
/// `menus/mb-pom.toml`). The `MB-` prefix is a convention, not enforced.
fn menu_bar_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

impl WorkbenchShell {
    pub(super) fn apply_theme(&self, ctx: &egui::Context) {
        let p = &self.palette;
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = to_egui_color(p.editor.background);
        visuals.window_fill = to_egui_color(p.ui.panel_bg);
        visuals.window_stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.ui.panel_border));
        // Use menu_bar_fg as the global text colour — in Legacy this is white (#FFFFFF),
        // which correctly colours menu bar items, tab bar, and chrome text.
        // Editor content text is applied per-element in editor_panel using palette tokens.
        visuals.override_text_color = Some(to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.noninteractive.bg_fill = to_egui_color(p.ui.panel_bg);
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.editor.foreground));
        visuals.widgets.inactive.bg_fill = to_egui_color(p.ui.button_bg);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.hovered.bg_fill = to_egui_color(p.ui.button_hover);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.widgets.active.bg_fill = to_egui_color(p.ui.input_bg);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0_f32, to_egui_color(p.ui.menu_bar_fg));
        visuals.selection.bg_fill = to_egui_color(p.editor.accent).linear_multiply(0.35);
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, to_egui_color(p.editor.accent));
        // In Legacy mode the slider track and handle are near-black on black — invisible.
        // Override with high-contrast ISPF colours: turquoise track, yellow handle.
        if p.mode == ff_theme::mode::VisualMode::Legacy {
            let track = egui::Color32::from_rgb(0, 170, 170); // ISPF turquoise
            let handle = egui::Color32::from_rgb(255, 255, 0); // ISPF yellow-hi
            visuals.widgets.inactive.bg_fill = track;
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 210, 210);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(2.0_f32, handle);
            visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 255, 255);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0_f32, handle);
        }
        ctx.set_visuals(visuals);
    }

    // ── Theme switching ───────────────────────────────────────────────────

    /// Switch to the given visual mode.
    ///
    /// Writes the mode name to the `theme.active` config key so the per-frame
    /// hot-reload block picks it up and the palette is not clobbered next frame.
    ///
    /// CR-CH-024: the `THEME` command is now name-based and routes through
    /// `set_active_theme`; this mode-based helper is retained for the mode
    /// config key path and the `follow_os` opt-out regression test.
    #[allow(dead_code)]
    pub(super) fn set_theme(&mut self, mode: ff_theme::mode::VisualMode) {
        self.palette = ff_theme::defaults::default_palette_for_mode(mode);
        let mode_str = mode.section_name().to_string();

        // An EXPLICIT theme selection opts out of "follow OS" -- otherwise the
        // per-frame follow_os block (update.rs) rebuilds the palette from the OS
        // dark/light preference every frame and clobbers the chosen mode, so the
        // selection never visibly sticks (B039 root cause, confirmed by runtime
        // logging: theme block set Legacy, follow_os reset it to Dark, every
        // frame). theme-and-appearance Req 16.4/16.7: follow_os must not override
        // an explicit user choice. Turn it off before persisting the mode.
        if self
            .config_handle
            .get_bool(ff_config::keys::theme::FOLLOW_OS)
            .unwrap_or(false)
        {
            let _ = self.config_handle.set_user_value(
                ff_config::keys::theme::FOLLOW_OS,
                ff_config::ConfigValue::Boolean(false),
            );
        }

        // Persist the active mode so the per-frame theme-active block reads the
        // same mode back and does not revert the selection. Surface a persist
        // failure instead of swallowing it (was `let _ =`, a hidden revert path).
        if let Err(e) = self.config_handle.set_user_value(
            ff_config::keys::theme::ACTIVE,
            ff_config::ConfigValue::String(mode_str),
        ) {
            self.open_error = Some(format!(
                "Theme changed to {} for this session, but could not be saved: {e}",
                mode.section_name()
            ));
        }

        // CR-NR-074 Req 19.9: keep theme.active_name consistent with the mode so
        // the file-backed resolver and the mode command agree. `THEME <mode>`
        // selects the built-in theme for that mode.
        let builtin_name = ff_theme::defaults::default_palette_for_mode(mode).name;
        let _ = self.config_handle.set_user_value(
            ff_config::keys::theme::ACTIVE_NAME,
            ff_config::ConfigValue::String(builtin_name),
        );
    }

    /// Set the active theme by NAME (a theme file / built-in), loading it,
    /// applying it immediately, and persisting `theme.active_name` so the same
    /// theme is active on the next launch. Used by the Theme editor Set_Active
    /// action (Requirement 20.6) and any name-based theme selection.
    ///
    /// Validates: theme-and-appearance Requirement 19.7
    /// Used by the Theme editor Set_Active action (task 24.5) and any name-based
    /// theme selection, sharing one activation path with startup/hot-reload.
    pub(super) fn set_active_theme(&mut self, name: &str) {
        let themes_dir = self.themes_dir();
        match crate::theme_defaults::load_theme_by_name(name, &themes_dir) {
            Some(palette) => {
                self.palette = palette;
                // An EXPLICIT theme selection opts out of "follow OS" -- otherwise
                // the per-frame follow_os block rebuilds the palette from the OS
                // dark/light preference and clobbers the chosen theme every frame
                // (B039 root cause). theme-and-appearance Req 16.4/16.7.
                if self
                    .config_handle
                    .get_bool(ff_config::keys::theme::FOLLOW_OS)
                    .unwrap_or(false)
                {
                    let _ = self.config_handle.set_user_value(
                        ff_config::keys::theme::FOLLOW_OS,
                        ff_config::ConfigValue::Boolean(false),
                    );
                }
                self.active_theme_file = {
                    let path = themes_dir
                        .join(format!("{}.toml", crate::theme_defaults::theme_slug(name)));
                    std::fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|mt| (path, mt))
                };
                if let Err(e) = self.config_handle.set_user_value(
                    ff_config::keys::theme::ACTIVE_NAME,
                    ff_config::ConfigValue::String(name.to_string()),
                ) {
                    self.open_error = Some(format!(
                        "Theme '{name}' applied for this session, but could not be saved: {e}"
                    ));
                }
            }
            None => {
                self.open_error = Some(format!("Theme '{name}' could not be loaded"));
            }
        }
    }

    // ── Legacy POM colours ────────────────────────────────────────────────

    /// Build `PomColours` for the current palette.
    ///
    /// When the Legacy theme is active, returns ISPF semantic colours.
    /// For all other themes, returns `PomColours::inherited()` so egui
    /// uses its own default colours.
    ///
    /// Validates: Requirement 13 (Legacy Theme Colour Semantics)
    pub(super) fn legacy_pom_colours(&self) -> primary_option_menu::PomColours {
        use ff_theme::mode::VisualMode;
        if self.palette.mode == VisualMode::Legacy {
            primary_option_menu::PomColours::from_palette(&self.palette)
        } else {
            primary_option_menu::PomColours::inherited()
        }
    }

    /// Semantic option + calendar colours for the shared menu renderer, derived
    /// from the same palette semantics as the POM. In Legacy mode the ISPF
    /// white/turquoise/green option scheme and turquoise/reversed calendar are
    /// used; other themes inherit egui colours (PLACEHOLDER).
    ///
    /// Validates: menu-workspace Requirement 2.1a, 2.1b; Requirement 13.4-13.8
    pub(super) fn menu_colours(&self) -> crate::menu_workspace::render::MenuColours {
        let pom = self.legacy_pom_colours();
        crate::menu_workspace::render::MenuColours {
            // Key column = POM option key (white in Legacy).
            option_key: pom.option_key,
            // Command column = POM option label (turquoise in Legacy).
            option_command: pom.option_label,
            // Description column = POM normal body text (green in Legacy).
            description: pom.normal_text,
            calendar_fg: pom.calendar_fg,
            today_bg: pom.today_bg,
            today_fg: pom.today_fg,
            use_today_reverse: pom.today_bg != eframe::egui::Color32::PLACEHOLDER,
        }
    }

    // ── Menu bar ─────────────────────────────────────────────────────────

    /// Render the Workbench menu bar (menu-workspace Requirement 17, CR-NR-080).
    ///
    /// The bar is now DATA-DRIVEN: it renders from the compiled default Menu_Bar
    /// (`default_menubar_menu`), a `MenuFile` drawn HORIZONTALLY. Later slices
    /// (B/C) add named user files and per-workspace-kind selection; Slice A
    /// always uses the default, which reproduces the previous hardcoded bar.
    ///
    /// Validates: menu-workspace Requirement 17.1, 17.7
    pub(super) fn render_menu_bar(&mut self, ctx: &egui::Context) {
        let menu = self.resolve_menu_bar_menu();
        self.render_menu_bar_from_menu(ctx, egui::Id::new("menu_bar"), &menu);
    }

    /// Render a Detached_Workspace's own Menu_Bar into its child viewport
    /// (CR-NR-089, menu-and-statusbar Req 18.12). Uses a per-window-salted panel
    /// id so it does not collide with the Primary_Window's `"menu_bar"` panel;
    /// the same data-driven renderer is reused, and menu-item dispatch routes
    /// through `handle_command` (so under the CR-CH-036 swap it acts on the
    /// detached tab).
    ///
    /// Validates: menu-and-statusbar Requirement 18.12
    pub(super) fn render_detached_menu_bar(
        &mut self,
        ctx: &egui::Context,
        tab_id: crate::tab_state::TabId,
    ) {
        let menu = self.resolve_menu_bar_menu();
        self.render_menu_bar_from_menu(ctx, egui::Id::new(("detached_menu_bar", tab_id.0)), &menu);
    }

    /// Resolve the active Menu_Bar `MenuFile` (menu-workspace Req 17.8,
    /// CR-NR-080 Slice B).
    ///
    /// The bar is a NAMED menu: it resolves the active menu-bar name to
    /// `menus/<slug>.toml` via the loader, so a user file OVERRIDES the compiled
    /// default. WHEN the file is absent or invalid, it falls back to the
    /// compiled default (`default_menubar_menu`, the barebones POM). For Slice B
    /// the name is the single default `DEFAULT_MENU_BAR_NAME` (`MB-POM`, slug
    /// `mb-pom`); Slice C will select the name per workspace kind. The `MB-`
    /// prefix is a naming convention, not enforced.
    ///
    /// Validates: menu-workspace Requirement 17.8
    pub(super) fn resolve_menu_bar_menu(&self) -> crate::menu_workspace::MenuFile {
        self.resolve_menu_bar_menu_for(self.tabs.active_tab())
    }

    /// Resolve the Menu_Bar `MenuFile` for a specific tab's Workspace Kind
    /// (CR-NR-090 B.2, workspace-kinds Req 4.1/4.2). The bar name is the Kind's
    /// effective `menu_bar` (from the registry) when set, else the compiled
    /// `DEFAULT_MENU_BAR_NAME`; it is then loaded via the existing named-menu
    /// resolver (`menus/<slug>.toml` with the compiled fallback). Behaviour-
    /// preserving for built-in Kinds (their default `menu_bar` is `None`).
    pub(super) fn resolve_menu_bar_menu_for(
        &self,
        tab: &crate::tab_state::TabState,
    ) -> crate::menu_workspace::MenuFile {
        let kind_name =
            crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind, tab.is_home).stable_name();
        let bar_name = self
            .kind_registry
            .effective(kind_name)
            .menu_bar
            .clone()
            .unwrap_or_else(|| crate::menu_workspace::defaults::DEFAULT_MENU_BAR_NAME.to_string());
        let slug = menu_bar_slug(&bar_name);
        let path = self.menus_dir().join(format!("{slug}.toml"));
        crate::menu_workspace::loader::load_menu_file(&path)
            .unwrap_or_else(|_| crate::menu_workspace::defaults::default_menubar_menu())
    }

    /// Render a `MenuFile` as a horizontal menu bar of dropdown buttons.
    ///
    /// Each top-level option becomes a `menu_button` labelled by its
    /// `description`. Opening a button PEEKS the menu its `command` references
    /// (rendering that menu's options as the dropdown items) WITHOUT navigating
    /// the active Workspace (Req 17.3); WHERE the command does not name a menu,
    /// the dropdown holds a single item that dispatches the command directly
    /// (Req 17.4). Selecting any item routes through `handle_command` (command
    /// parity) and closes the dropdown. Dropdown items are keyboard-navigable via
    /// egui-native menu behaviour (Req 17.5).
    ///
    /// The FIRST and LAST top-level button ids are captured into
    /// `menu_first_id` / `menu_last_id` for the CR-CH-023 Boundary_Policy
    /// (Req 17.6), from the data-driven loop rather than hardcoded buttons.
    ///
    /// Validates: menu-workspace Requirement 17.1, 17.3, 17.4, 17.5, 17.6
    pub(super) fn render_menu_bar_from_menu(
        &mut self,
        ctx: &egui::Context,
        panel_id: egui::Id,
        menu: &crate::menu_workspace::MenuFile,
    ) {
        egui::TopBottomPanel::top(panel_id).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // Bar shows only options flagged for the menu bar (Req 17.2):
                // e.g. a terminal `RETURN` option (show_in_menu_bar = false) is
                // kept in the vertical POM but hidden from the horizontal bar.
                let bar_options: Vec<&crate::menu_workspace::MenuOption> =
                    menu.options.iter().filter(|o| o.show_in_menu_bar).collect();
                let last_index = bar_options.len().saturating_sub(1);
                for (i, option) in bar_options.iter().enumerate() {
                    // A DYNAMIC source (e.g. `THEME LIST`) generates its children
                    // at runtime (Req 17.10); otherwise PEEK the referenced menu's
                    // options (Req 17.3). Empty peek => a plain direct command.
                    let dynamic = self.dynamic_menu_options(&option.command);
                    let peeked = if dynamic.is_some() {
                        Vec::new()
                    } else {
                        self.peek_menu_options(&option.command)
                    };
                    // Bar buttons are labelled by the option's COMMAND (the verb
                    // the user would type), not its description (CR-NR-080).
                    let btn = ui.menu_button(option.command.clone(), |ui| {
                        if let Some(children) = &dynamic {
                            // Dynamic children (Req 17.10, 17.11): labelled by the
                            // theme name (description), dispatch `THEME <name>`.
                            for child in children {
                                if ui.button(child.description.clone()).clicked() {
                                    self.handle_command(&child.command);
                                    ui.close_menu();
                                }
                            }
                        } else if peeked.is_empty() {
                            // Non-menu command: one item that dispatches it (Req 17.4).
                            if ui.button(option.command.clone()).clicked() {
                                self.handle_command(&option.command);
                                ui.close_menu();
                            }
                        } else {
                            // Peeked submenu options: each is labelled by its
                            // COMMAND and dispatches that command (Req 17.3,
                            // 17.4, command parity) -- consistent with the
                            // top-level buttons.
                            for child in &peeked {
                                if ui.button(child.command.clone()).clicked() {
                                    self.handle_command(&child.command);
                                    ui.close_menu();
                                }
                            }
                        }
                    });
                    // CR-CH-023 Boundary_Policy: capture the FIRST and LAST
                    // top-level button ids for the next frame (Req 17.6).
                    if i == 0 {
                        self.menu_first_id = Some(btn.response.id);
                    }
                    if i == last_index {
                        self.menu_last_id = Some(btn.response.id);
                    }
                }
            });
        });
    }

    /// Peek the options of the menu referenced by `command`, WITHOUT navigating.
    ///
    /// Returns the referenced menu's `[MenuOption]` when `command`'s first token
    /// names a resolvable menu (a user `menus/<name>.toml` or a compiled built-in
    /// `pom`/`settings`); otherwise returns an empty vec (the caller then treats
    /// the option as a direct command). This uses the SAME resolver the command
    /// line uses (`ShellTargetResolver::menu_name_target`), so the bar and typed
    /// commands agree on what names a menu.
    ///
    /// Validates: menu-workspace Requirement 17.3
    pub(super) fn peek_menu_options(
        &self,
        command: &str,
    ) -> Vec<crate::menu_workspace::MenuOption> {
        use ff_command::{CommandTarget, TargetResolver};
        let resolver = crate::command_config::ShellTargetResolver::new(
            &self.command_store.definitions,
            &self.cmd_registry,
            self.menus_dir(),
        );
        let name = match resolver.menu_name_target(command) {
            Some(CommandTarget::Menu { name }) => name,
            _ => return Vec::new(),
        };
        // Load the referenced menu's options: a user file if present, else the
        // compiled Recovery_Baseline for the built-in names.
        let menus_dir = self.menus_dir();
        let path = menus_dir.join(format!("{name}.toml"));
        let menu = crate::menu_workspace::loader::load_menu_file(&path)
            .ok()
            .unwrap_or_else(|| match name.as_str() {
                "settings" => crate::menu_workspace::defaults::recovery_settings_menu(),
                _ => crate::menu_workspace::defaults::recovery_pom_menu(),
            });
        menu.options
    }

    /// Generate a DYNAMIC option source for a menu-bar dropdown, if `command`
    /// names one (menu-workspace Requirement 17.10, CR-NR-080 Slice D).
    ///
    /// A dynamic source produces its child options at RUNTIME rather than from a
    /// file. The first (and currently only) source is `THEME LIST`: it yields one
    /// child per available theme (`ff_theme::list_all_themes`, in list order),
    /// each labelled by the theme name and dispatching `THEME <name>` (command
    /// parity, theme-and-appearance Requirement 17.2, applied via the shared
    /// `set_active_theme` path). This delivers the CR-NR-077 theme picker via the
    /// menu bar. Returns `None` for any command that is not a dynamic source, so
    /// the caller falls back to `peek_menu_options` / direct dispatch.
    ///
    /// Validates: menu-workspace Requirement 17.10, 17.11
    pub(super) fn dynamic_menu_options(
        &self,
        command: &str,
    ) -> Option<Vec<crate::menu_workspace::MenuOption>> {
        if !command.trim().eq_ignore_ascii_case("THEME LIST") {
            return None;
        }
        let options = ff_theme::list_all_themes(&self.themes_dir())
            .into_iter()
            .map(|t| crate::menu_workspace::MenuOption {
                key: String::new(),
                command: format!("THEME {}", t.name),
                description: t.name.clone(),
                enabled: true,
                group: None,
                show_in_menu_bar: true,
                target: None,
            })
            .collect();
        Some(options)
    }

    // ── Tab bar ──────────────────────────────────────────────────────────

    pub(super) fn render_tab_bar(&mut self, ctx: &egui::Context) {
        // Validates: Requirement 21.6 -- render_tab_bar reads the tab_bar.* palette
        // group (previously dead: bg reused ui.input_bg/panel_bg and text reused
        // editor.foreground). Active/inactive tabs now get distinct bg + text.
        let active_bg = to_egui_color(self.palette.tab_bar.active_bg);
        let inactive_bg = to_egui_color(self.palette.tab_bar.inactive_bg);
        let active_text = to_egui_color(self.palette.tab_bar.active_text);
        let inactive_text = to_egui_color(self.palette.tab_bar.inactive_text);
        let modified_color = to_egui_color(self.palette.editor.accent);

        // Collect context-menu actions outside the borrow of self.tabs.
        let mut activate_idx: Option<usize> = None;
        let mut close_idx: Option<usize> = None;
        let mut close_all_but: Option<usize> = None;
        let mut close_left_of: Option<usize> = None;
        let mut close_right_of: Option<usize> = None;
        let mut close_unchanged = false;
        // CR-CH-035 (Req 18.6): a tab whose header was dragged out of the bar.
        let mut detach_drag_idx: Option<usize> = None;

        egui::TopBottomPanel::top("tab_bar")
            .min_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let tab_count = self.tabs.len();
                    let active_idx_cur = self.tabs.active_index();

                    for i in 0..tab_count {
                        let tab = &self.tabs.tabs()[i];
                        // Validates: Requirement 18.4 — skip tabs that are floating.
                        if tab.is_floating {
                            continue;
                        }
                        let is_active = i == active_idx_cur;
                        let tab_kind = tab.kind;

                        let bg = if is_active { active_bg } else { inactive_bg };
                        let tab_text = if is_active {
                            active_text
                        } else {
                            inactive_text
                        };
                        // Validates: CX Requirement 1.4 -- show workspace_name in tab header
                        let base_title = if let Some(ref name) = tab.workspace_name {
                            match tab.kind {
                                crate::tab_state::TabKind::FileEditor
                                | crate::tab_state::TabKind::Untitled => {
                                    format!("{}: {}", name, tab.title)
                                }
                                _ => format!("[{}]", name),
                            }
                        } else if tab.kind == crate::tab_state::TabKind::MenuWorkspace
                            && !tab.is_home
                        {
                            // CR-CH-034 / B050 (Req 17.10): derive a non-Home
                            // Menu_Workspace header from its CURRENTLY loaded
                            // menu so an in-place context switch can never leave
                            // the tab header showing the previous Context's
                            // label. The cached title is the fallback only when
                            // no menu is loaded.
                            tab.menu_workspace
                                .as_ref()
                                .map(|mw| mw.tab_title())
                                .unwrap_or_else(|| tab.title.clone())
                        } else {
                            // CR-NR-090 B.1: derive the header from the Kind
                            // registry (kind_title) so it reflects the Kind's
                            // configured title (fixes the Catalogs/[FILES] smell)
                            // and updates live when a Kind is reconfigured.
                            self.kind_title(tab)
                        };
                        let label = if tab.is_modified {
                            format!("● {}", base_title)
                        } else {
                            base_title
                        };
                        let color = if tab.is_modified {
                            modified_color
                        } else {
                            tab_text
                        };

                        // CR-CH-023 Req 16.9: tab headers are clickable but not
                        // keyboard Tab stops (tab switching is via the SWAP
                        // command / mouse). CR-CH-035 (B045 drag-out, Req 18.6):
                        // also sense DRAG so the header can be dragged out of the
                        // bar to detach. `click_and_drag` keeps the click and
                        // drag but egui skips it for keyboard Tab (not FOCUSABLE).
                        let btn =
                            egui::Button::new(egui::RichText::new(&label).color(color).monospace())
                                .fill(bg)
                                .stroke(if is_active {
                                    egui::Stroke::new(1.0_f32, color)
                                } else {
                                    egui::Stroke::NONE
                                })
                                .min_size(egui::vec2(0.0, 24.0))
                                .sense(egui::Sense::click_and_drag());

                        let resp = ui.add(btn);
                        if resp.clicked() {
                            activate_idx = Some(i);
                        }
                        // CR-CH-035 (Req 18.6): dragging a Tab_Header more than
                        // 20px beyond the tab-bar boundary detaches it into a
                        // Detached_Workspace. We detect the drag on release: if
                        // the pointer moved >20px vertically below the bar (or the
                        // total drag exceeded the threshold and ended outside the
                        // bar rect), request a detach for this tab. The real
                        // cross-window release position is applied by the OS; the
                        // headless-testable part is "drag beyond threshold on a
                        // tab header sets detach_pending".
                        if resp.drag_stopped() {
                            let bar_bottom = ui.max_rect().bottom();
                            let released = ui
                                .ctx()
                                .input(|inp| inp.pointer.interact_pos())
                                .unwrap_or(resp.rect.center());
                            let moved = resp.drag_delta().length()
                                + ui.ctx().input(|inp| {
                                    inp.pointer
                                        .press_origin()
                                        .map(|o| (released - o).length())
                                        .unwrap_or(0.0)
                                });
                            let outside_bar = released.y > bar_bottom + 20.0;
                            if (outside_bar || moved > 20.0) && !tab.is_floating {
                                detach_drag_idx = Some(i);
                            }
                        }
                        // CR-CH-023: tab headers are no longer keyboard focus
                        // stops (Req 16.9), so there is no tab-header focus ring
                        // indicator here anymore.

                        // Validates: Requirement 3.8 multi-tab-editor — close button on tab header (B002/B015)
                        // CR-CH-023 Req 16.9: click-only sense (not a Tab stop).
                        let close_resp = ui.add(
                            egui::Button::new(
                                egui::RichText::new("\u{00d7}")
                                    .color(tab_text)
                                    .monospace()
                                    .small(),
                            )
                            .fill(bg)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(16.0, 24.0))
                            .sense(egui::Sense::CLICK),
                        );
                        if close_resp.clicked() {
                            close_idx = Some(i);
                        }
                        close_resp.on_hover_text("Close tab");
                        // Validates: Requirement 14.15, 14.15a, 14.15b, 14.15c
                        resp.context_menu(|ui| {
                            let tab_count_inner = self.tabs.len();
                            // ── Universal items (all tab kinds) — Req 14.15a ──
                            if ui.button("Close").clicked() {
                                close_idx = Some(i);
                                ui.close_menu();
                            }
                            ui.add_enabled_ui(tab_count_inner > 1, |ui| {
                                if ui.button("Close All BUT This").clicked() {
                                    close_all_but = Some(i);
                                    ui.close_menu();
                                }
                            });
                            ui.add_enabled_ui(i > 0, |ui| {
                                if ui.button("Close All to the Left").clicked() {
                                    close_left_of = Some(i);
                                    ui.close_menu();
                                }
                            });
                            ui.add_enabled_ui(i < tab_count_inner - 1, |ui| {
                                if ui.button("Close All to the Right").clicked() {
                                    close_right_of = Some(i);
                                    ui.close_menu();
                                }
                            });
                            if ui.button("Close All Unchanged").clicked() {
                                close_unchanged = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Clone to Other Tab").clicked() {
                                // stub — deferred
                                ui.close_menu();
                            }
                            if ui.button("Move to Other View").clicked() {
                                // Validates: Requirement 18.1, 18.7
                                if self.floating_tabs.len() < 16 {
                                    self.detach_pending = Some(i);
                                } else {
                                    self.open_error = Some(
                                        "Maximum 16 floating windows already open.".to_string(),
                                    );
                                }
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Pin Tab").clicked() {
                                // stub — deferred
                                ui.close_menu();
                            }

                            // ── Exit — Req 14.15a, 14.38 (all tab kinds) ─────
                            ui.separator();
                            if ui.button("Exit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                ui.close_menu();
                            }

                            // ── File-editor-only items — Req 14.15b ──────────
                            // Only shown when the tab is a FileEditor.
                            if tab_kind == TabKind::FileEditor {
                                ui.separator();
                                if ui.button("Open Containing Folder in Explorer").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Explorer);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in CMD").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Cmd);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in PowerShell").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::PowerShell);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Open Containing Folder in Terminal").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.as_deref() {
                                        open_containing_folder(path, FolderOpenMode::Terminal);
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("Copy Name to Clipboard").clicked() {
                                    if let Some(title) =
                                        self.tabs.tabs().get(i).map(|t| t.title.clone())
                                    {
                                        ui.ctx().copy_text(title);
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Copy Path to Clipboard").clicked() {
                                    if let Some(path) = self.tabs.tabs()[i].path.clone() {
                                        ui.ctx().copy_text(path);
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("Save").clicked() {
                                    // handled after menu closes via pending action
                                    ui.close_menu();
                                }
                                if ui.button("Save As").clicked() {
                                    ui.close_menu();
                                }
                                if ui.button("Reload").clicked() {
                                    ui.close_menu();
                                }
                            }
                        });
                    }

                    // ── Empty tab-bar space right-click — Req 14.9 ──────
                    let bar_resp = ui.interact(
                        ui.available_rect_before_wrap(),
                        ui.id().with("tab_bar_empty"),
                        egui::Sense::click(),
                    );
                    bar_resp.context_menu(|ui| {
                        if ui.button("New").clicked() {
                            self.pending_new_pom = true;
                            ui.close_menu();
                        }
                        if ui.button("New File").clicked() {
                            self.pending_new_file = true;
                            ui.close_menu();
                        }
                    });
                });
            });

        // Apply deferred tab-bar actions.
        if let Some(i) = activate_idx {
            // Track previous tab for END navigation -- Validates: Requirement 17.1
            self.tab_history.push(self.tabs.active_index());
            self.tabs.set_active(i);
            // Update key map context for the new active tab -- Validates:
            // Requirement 14.4; CR-NR-090 B.2 (workspace-kinds Req 4.3): the
            // context is the Kind's configured key_list if set, else the base
            // kind context.
            let ctx_name = self.key_list_context_for_tab(self.tabs.active_tab());
            self.key_map_resolver.set_context(ctx_name.as_deref());
            self.key_label_bar
                .update(self.key_map_resolver.active_key_map());
        }
        if let Some(i) = close_idx {
            self.tabs.close_tab(i);
        }
        if let Some(pivot) = close_all_but {
            let count = self.tabs.len();
            // Close right-of-pivot first (indices stable), then left.
            for i in (pivot + 1..count).rev() {
                self.tabs.close_tab(i);
            }
            for i in (0..pivot).rev() {
                self.tabs.close_tab(i);
            }
        }
        if let Some(pivot) = close_left_of {
            for i in (0..pivot).rev() {
                self.tabs.close_tab(i);
            }
        }
        if let Some(pivot) = close_right_of {
            let count = self.tabs.len();
            for i in (pivot + 1..count).rev() {
                self.tabs.close_tab(i);
            }
        }
        if close_unchanged {
            let count = self.tabs.len();
            for i in (0..count).rev() {
                if !self.tabs.tabs()[i].is_modified {
                    self.tabs.close_tab(i);
                }
            }
        }
        // CR-CH-035 (Req 18.6): a tab header dragged out of the bar detaches,
        // subject to the shared 16-window limit. Sets detach_pending; the frame
        // loop consumes it (same path as "Move to Other View" / SPLIT DETACH).
        if let Some(i) = detach_drag_idx {
            if self.floating_tabs.len() < 16 {
                self.detach_pending = Some(i);
                self.open_error = None;
            } else {
                self.open_error =
                    Some("Maximum number of detached Workspaces (16) reached.".to_string());
            }
        }
    }

    // ── Title line ──────────────────────────────────────────────────
}
