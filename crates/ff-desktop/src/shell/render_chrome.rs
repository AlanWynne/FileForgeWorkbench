//! # Shell Chrome Rendering
//!
//! Menu bar rendering for WorkbenchShell. Theme application lives in
//! `render_theme.rs` and tab-bar rendering in `render_tab_bar.rs` after the
//! Phase 2 task 2.2 file-size split. The per-dropdown item rendering, keyboard
//! navigation (menu-workspace Req 17.5), and the menu peek/dynamic/SCRM helpers
//! live in `render_menu_bar.rs` (file-size rule).

use eframe::egui;

use super::render_menu_bar::MenuDropdownNav;
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
            crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)
                .stable_name();
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
    /// parity) and closes the dropdown. Dropdown items are keyboard-navigable
    /// (Req 17.5).
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
        // B-fix (Req 17.5a/17.5b): CONSUME both dropdown-navigation arrow keys
        // HERE, before egui's `MenuBar` renders, so egui's own menu navigation
        // never also moves focus. egui processes the arrows inside its `MenuBar`
        // `.ui()` call -- BEFORE the per-dropdown content closure runs -- so a
        // consume inside that closure is too late, and leaving egui to navigate
        // made the shell walk double-advance (the reported regression: first Down
        // landed on item 1 / Aliases instead of item 0 / Config) and escape the
        // dropdown on Up. The shell owns the walk as a tracked index and FORCES
        // the selected item into focus after the MenuBar runs (below).
        //
        // CONSUME ONLY WHILE A DROPDOWN IS OPEN. `menu_dropdown_open_this_frame`
        // still holds the PREVIOUS frame's value at this point (it is cleared
        // just below, after the consume), so it tells us whether a dropdown was
        // open last frame. When no dropdown is open the arrows belong to other
        // consumers (command-field history recall, config-tree navigation, the
        // nav-stack walk); swallowing them here unconditionally was a regression
        // that broke those paths. The dropdown is only ever opened by a prior
        // Enter/click on a top-level button, so by the time the user can press
        // an arrow the flag is already set and the first Down is still consumed.
        // Req 17.5e: ALSO consume Left/Right while a dropdown is open, so the
        // shell owns the walk ALONG the bar (between headings) just as it owns
        // the Down/Up walk WITHIN a dropdown. Gated on `dropdown_open` for the
        // same reason as Down/Up: when the bar is idle, Left/Right belong to
        // other consumers (the command field, the config tree collapse/expand,
        // the nav stack), and swallowing them unconditionally was the prior
        // regression. The dropdown is only ever open after a prior Enter/click on
        // a heading, so by the time the user can press an arrow the flag is set.
        // Req 17.5f: ALSO handle Down when a bar heading holds keyboard focus
        // but NO dropdown is open yet (the Tab-to-heading case). The Down/Up/
        // Left/Right consume above is gated on a dropdown ALREADY being open, so
        // a heading reached by Tab (focus on the heading button, popup closed)
        // never saw Down and nothing happened. Detect that case here -- BEFORE
        // the flags are cleared -- by matching the currently focused egui id
        // against last frame's captured heading ids (`menu_bar_heading_ids`,
        // which like `menu_dropdown_open_this_frame` still holds the PREVIOUS
        // frame's value at this point). The consume stays TIGHTLY gated: it fires
        // ONLY when a heading actually holds focus, so Down is never stolen from
        // the command field, config tree, or nav-stack walk (the prior over-broad
        // arrow-consume regression).
        let dropdown_open = self.focus.menu_dropdown_open_this_frame;
        let focused_id = ui.ctx().memory(|m| m.focused());
        let focused_heading_index = focused_id.and_then(|id| {
            self.focus
                .menu_bar_heading_ids
                .iter()
                .position(|&h| h == id)
        });
        // Down is consumed when a dropdown is open (the existing walk) OR when a
        // bar heading holds focus with no dropdown open (the new Tab-to-heading
        // open). Up/Left/Right stay gated on a dropdown being open only.
        let down_for_closed_heading = !dropdown_open && focused_heading_index.is_some();
        let down_consumed = (dropdown_open || down_for_closed_heading)
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        // When Down opens a CLOSED focused heading (below), the dropdown walk is
        // SEEDED to item 0 directly, so the dropdown render must NOT also treat
        // this Down as an advance (that would double-advance to item 1). Only the
        // already-open case forwards Down into the per-item walk.
        let open_closed_heading = down_consumed && down_for_closed_heading;
        // Req 17.5g: WHILE a dropdown is open, Tab behaves like Right and
        // Shift+Tab like Left -- the user asked that Tab keep navigating the bar
        // instead of being swallowed. CONSUME the Tab event here (before egui's
        // `MenuBar` runs) so egui's own Tab focus traversal does NOT ALSO move
        // focus this frame; the consumed Tab is folded into the SAME heading-
        // move-and-open path as Left/Right (nav.left/nav.right below). The
        // consume is GATED on `dropdown_open` for the SAME reason as Left/Right:
        // when the bar is idle, Tab belongs to the unified Tab-order traversal
        // (Req 15/17.6) and must NOT be hijacked by the bar.
        //
        // Direction is read from the SHIFT modifier on the Tab event, not from
        // two separate `consume_key` calls: the plain-Tab and Shift+Tab variants
        // are both consumed here (so neither leaks to egui's own focus
        // traversal) and the SHIFT state sampled at consume time decides Right
        // (no shift) vs Left (shift). Sampling the modifier is robust to how the
        // Tab event's modifiers are normalised by the backend/harness.
        let (tab_right, tab_left) = if dropdown_open {
            let shift = ui.input(|i| i.modifiers.shift);
            let plain = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
            let with_shift =
                ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab));
            let tab_pressed = plain || with_shift;
            (tab_pressed && !shift, tab_pressed && shift)
        } else {
            (false, false)
        };
        let nav = MenuDropdownNav {
            down: down_consumed && !open_closed_heading,
            up: dropdown_open
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)),
            left: tab_left
                || dropdown_open
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)),
            right: tab_right
                || dropdown_open
                    && ui
                        .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)),
        };
        // Clear per-frame dropdown-walk flags before the dropdowns render.
        self.focus.menu_dropdown_focus_target = None;
        self.focus.menu_dropdown_open_this_frame = false;
        self.focus.menu_dropdown_owner_index = None;

        // Req 17.5f: a Down on a focused-but-closed heading OPENS that heading's
        // dropdown and lands focus on its FIRST item. Open the heading's `Popup`
        // BEFORE egui's `MenuBar` renders so the dropdown content closure runs
        // THIS frame (the 5e Left/Right path opens popups the same way, via
        // `egui::Popup::open_id`). SEED `menu_dropdown_index` to `Some(0)` so,
        // regardless of which frame the dropdown content first renders, the walk
        // forces focus onto item 0 -- the same end state as opening the heading
        // then pressing Down (criterion 5a). Record the heading as the bar's
        // keyboard index so a later Up-to-heading / Left-Right knows the open one.
        if open_closed_heading {
            if let Some(idx) = focused_heading_index {
                if let Some(&heading) = self.focus.menu_bar_heading_ids.get(idx) {
                    egui::Popup::open_id(ui.ctx(), heading.with("popup"));
                    self.focus.menu_bar_index = Some(idx);
                    self.focus.menu_dropdown_index = Some(0);
                    ui.ctx().request_repaint();
                }
            }
        }

        // Rebuilt every frame: the ids of all visible top-level headings, in bar
        // order, so the Left/Right walk can index previous/next (Req 17.5e).
        let mut heading_ids: Vec<egui::Id> = Vec::new();

        egui::MenuBar::new().ui(ui, |ui| {
            // Bar shows only options flagged for the menu bar (Req 17.2):
            // e.g. a terminal `RETURN` option (show_in_menu_bar = false) is
            // kept in the vertical POM but hidden from the horizontal bar.
            let bar_options: Vec<&crate::menu_workspace::MenuOption> =
                menu.options.iter().filter(|o| o.show_in_menu_bar).collect();
            let last_index = bar_options.len().saturating_sub(1);
            for (i, option) in bar_options.iter().enumerate() {
                // The ordered child item commands this top-level button's
                // dropdown offers (Req 17.3/17.4/17.10). Computed BEFORE opening
                // the dropdown so the id-salt (the top-level command) is stable.
                let items = self.menu_bar_dropdown_items(&option.command);
                // Bar buttons are labelled by the option's COMMAND (the verb
                // the user would type), not its description (CR-NR-080).
                let btn = ui.menu_button(option.command.clone(), |ui| {
                    // The dropdown content closure runs ONLY while the dropdown
                    // is open. Render the items with stable ids and drive the
                    // shell-owned Down/Up focus walk + Enter dispatch (Req 17.5,
                    // B-fix: egui's own menu navigation is unreliable, so the
                    // arrows were consumed above and the shell owns the move).
                    // `i` is this heading's bar index, recorded so the bar
                    // renderer's Left/Right and Up-to-heading logic (below) knows
                    // the open heading (Req 17.5d/17.5e).
                    self.render_menu_bar_dropdown(ui, &option.command, &items, nav, i);
                });
                heading_ids.push(btn.response.id);
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
        self.focus.menu_bar_heading_ids = heading_ids.clone();

        // Req 17.5e: apply the shell-owned Left/Right walk ALONG the bar, AFTER
        // egui's MenuBar has run. Determine the current heading index: the bar's
        // tracked keyboard index if set, else the index of whichever heading's
        // dropdown rendered this frame (so a mouse-opened dropdown is picked up).
        // Left/Right move the index (clamped at the first/last heading -- desktop
        // convention is clamp, not wrap); the newly-focused heading's dropdown is
        // opened and all others closed, and `menu_dropdown_index` is reset to
        // None so focus lands on the heading (not a stale item).
        if dropdown_open && !heading_ids.is_empty() && (nav.left || nav.right) {
            let current = self
                .focus
                .menu_bar_index
                .or(self.focus.menu_dropdown_owner_index)
                .unwrap_or(0);
            let last = heading_ids.len() - 1;
            let next = if nav.right {
                (current + 1).min(last)
            } else {
                current.saturating_sub(1)
            };
            self.focus.menu_bar_index = Some(next);
            self.focus.menu_dropdown_index = None;
            // Open the newly-focused heading's dropdown (closes all others): a
            // `menu_button`'s popup id is `button_id.with("popup")` in egui 0.33.
            let popup_id = heading_ids[next].with("popup");
            egui::Popup::open_id(ui.ctx(), popup_id);
            ui.ctx().request_repaint();
        }

        // B-fix (Req 17.5a/17.5b): FORCE the dropdown walk's selected ITEM into
        // focus AFTER egui's MenuBar has run, so the shell's request is the LAST
        // focus request of the frame and egui's own menu navigation cannot drift
        // it. The open dropdown stashed the id into `menu_dropdown_focus_target`.
        if let Some(target) = self.focus.menu_dropdown_focus_target.take() {
            // Only request focus (and a follow-up repaint) when the target is
            // not ALREADY focused. Re-requesting an already-focused id every
            // frame would keep scheduling repaints and spin the UI (the harness
            // caps this and real apps waste frames). Once focus has settled on
            // the selected item, this becomes a no-op and the UI goes idle.
            let already = ui.ctx().memory(|m| m.focused()) == Some(target);
            if !already {
                ui.ctx().memory_mut(|m| m.request_focus(target));
                ui.ctx().request_repaint();
            }
        } else if dropdown_open {
            // Req 17.5d/17.5e: no item is selected but a dropdown is open, so
            // focus belongs on the HEADING (Up returned from item 0, or Left/
            // Right moved along the bar). Force focus onto the active heading
            // button and keep its dropdown open for the next frame.
            if let Some(idx) = self
                .focus
                .menu_bar_index
                .or(self.focus.menu_dropdown_owner_index)
            {
                if let Some(&heading) = heading_ids.get(idx) {
                    let already = ui.ctx().memory(|m| m.focused()) == Some(heading);
                    if !already {
                        ui.ctx().memory_mut(|m| m.request_focus(heading));
                        ui.ctx().request_repaint();
                    }
                }
            }
        }
        // No dropdown rendered this frame -> it is closed; reset the walk indices
        // so the next open starts fresh (Req 17.5a: first Down -> item 0, and the
        // bar leaves keyboard mode). EXCEPTION (Req 17.5f): when this frame just
        // opened a focused-but-closed heading via `open_id`, the newly-opened
        // dropdown's content closure may not render until NEXT frame; preserve
        // the seeded `menu_dropdown_index`/`menu_bar_index` so the first item is
        // still forced into focus once it renders, instead of being wiped here.
        if !self.focus.menu_dropdown_open_this_frame && !open_closed_heading {
            self.focus.menu_dropdown_index = None;
            self.focus.menu_bar_index = None;
        }
    }
}
