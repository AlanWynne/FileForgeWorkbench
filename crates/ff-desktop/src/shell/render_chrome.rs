//! # Shell Chrome Rendering
//!
//! Menu bar rendering for WorkbenchShell. Theme application lives in
//! `render_theme.rs` and tab-bar rendering in `render_tab_bar.rs` after the
//! Phase 2 task 2.2 file-size split.

use eframe::egui;

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

// The theme-application and colour-derivation methods (apply_theme, set_theme,
// set_active_theme, legacy_pom_colours, menu_colours) now live in
// `render_theme.rs`; the tab-bar method (render_tab_bar) now lives in
// `render_tab_bar.rs`. Both are `impl WorkbenchShell` blocks moved verbatim as
// part of the Phase 2 task 2.2 file-size split.

impl WorkbenchShell {
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
            self.render_menu_bar_into_ui(ui, menu);
        });
    }

    /// Render a `MenuFile` as a horizontal menu bar of dropdown buttons INTO an
    /// existing `Ui` (CR-CH-041 IRP-a: the ctx-level-panel-free core of
    /// [`render_menu_bar_from_menu`]).
    ///
    /// This is the reusable building block: [`render_menu_bar_from_menu`] wraps
    /// it in a `TopBottomPanel` for the ctx-level Primary_Window / detached bars
    /// (byte-identical to before this extraction), and the in-region chrome path
    /// (IRP-b) will call it directly inside a split region's sub-`Ui`. Behaviour
    /// -- including the `menu_first_id` / `menu_last_id` Boundary_Policy capture
    /// (Req 17.6) and command-parity dispatch -- is unchanged; only the surface
    /// it draws into differs.
    ///
    /// Validates: menu-workspace Requirement 17.1, 17.3, 17.4, 17.5, 17.6
    pub(super) fn render_menu_bar_into_ui(
        &mut self,
        ui: &mut egui::Ui,
        menu: &crate::menu_workspace::MenuFile,
    ) {
        egui::MenuBar::new().ui(ui, |ui| {
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
                                ui.close();
                            }
                        }
                    } else if peeked.is_empty() {
                        // Non-menu command: one item that dispatches it (Req 17.4).
                        if ui.button(option.command.clone()).clicked() {
                            self.handle_command(&option.command);
                            ui.close();
                        }
                    } else {
                        // Peeked submenu options: each is labelled by its
                        // COMMAND and dispatches that command (Req 17.3,
                        // 17.4, command parity) -- consistent with the
                        // top-level buttons.
                        for child in &peeked {
                            if ui.button(child.command.clone()).clicked() {
                                self.handle_command(&child.command);
                                ui.close();
                            }
                        }
                    }
                });
                // CR-CH-023 Boundary_Policy: capture the FIRST and LAST
                // top-level button ids for the next frame (Req 17.6).
                if i == 0 {
                    self.focus.menu_first_id = Some(btn.response.id);
                }
                if i == last_index {
                    self.focus.menu_last_id = Some(btn.response.id);
                }
            }
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
}
