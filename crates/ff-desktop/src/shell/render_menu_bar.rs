//! Menu-bar dropdown item rendering and keyboard navigation.
//!
//! Split out of `render_chrome.rs` (file-size rule) and the home of the
//! menu-bar dropdown keyboard navigation (menu-workspace Req 17.5, B-fix).
//!
//! A menu-bar top-level button opens a dropdown (egui `MenuBar`/`Popup`). Each
//! dropdown item is rendered with a STABLE `egui::Id` so a focus move
//! round-trips across frames. The SHELL owns the keyboard focus walk
//! deterministically rather than relying on egui's own menu navigation, which on
//! this stack is unreliable: it double-advances when the shell also moves (the
//! first Down landed on item 1 / Aliases instead of item 0 / Config -- the
//! reported regression) and it escapes the dropdown on Up.
//!
//! The walk is a tracked INDEX (`FocusState::menu_dropdown_index`):
//! - Down advances the index (from "none selected" to item 0, else +1 clamped at
//!   the last item) -- Req 17.5a/17.5b;
//! - Up retreats it (clamped at item 0) -- Req 17.5b;
//! - the selected item's id is STASHED each frame and the bar renderer FORCES it
//!   into focus AFTER egui's `MenuBar` has run (see `render_menu_bar_into_ui`),
//!   so the shell's request is the frame's LAST focus request and egui's own
//!   navigation cannot drift it.
//!
//! The arrow keys are CONSUMED before egui's MenuBar renders, and the index is
//! reset when the dropdown closes. A focused item's built-in Enter/Space
//! activation fires `clicked()` so the SAME front-door dispatch a mouse click
//! uses runs (command parity, Req 17.5c).
//!
//! Validates: menu-workspace Requirement 17.3, 17.4, 17.5, 17.10, 17.11;
//! screen-snapshot-scrm Requirement 21 (CR-NR-098)

use eframe::egui;

use super::WorkbenchShell;

/// A single dropdown item: the `label` shown and the `command` dispatched when
/// it is activated (Req 17.4, command parity). For a peeked/plain command the
/// label equals the command (CR-NR-080); for a dynamic source (e.g. the theme
/// picker) the label is the source-provided description.
pub(super) struct MenuBarItem {
    pub label: String,
    pub command: String,
}

/// The menu-bar navigation arrow keys pressed this frame, captured and CONSUMED
/// by `render_menu_bar_into_ui` BEFORE egui's `MenuBar` renders (B-fix,
/// Req 17.5a/17.5b/17.5d/17.5e) so egui's own menu navigation never also moves
/// focus. `down`/`up` walk the open dropdown's ITEMS (and Up at item 0 returns
/// to the heading); `left`/`right` walk the top-level HEADINGS along the bar.
/// All are gated on a dropdown being open so other Left/Right/Up/Down consumers
/// (command field, config tree, nav stack) are not starved when the bar is idle.
#[derive(Clone, Copy)]
pub(super) struct MenuDropdownNav {
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
}

impl WorkbenchShell {
    /// The active POM `MenuFile`, the single source the POM-echo menu bar mirrors
    /// (screen-snapshot-scrm Req 22.1, CR-NR-114).
    ///
    /// Resolves the SAME way the POM Option_Key fastpath does
    /// (`resolve_pom_option_key`): a loaded POM tab's menu (respecting user edits
    /// / hot-reload) when present, else the on-disk `menus/pom.toml`, else the
    /// code-only `recovery_pom_menu()`. So the bar always echoes whatever the POM
    /// currently shows, and changing the POM options changes the bar.
    ///
    /// Validates: screen-snapshot-scrm Requirement 22.1 (CR-NR-114)
    pub(super) fn active_pom_menu(&self) -> crate::menu_workspace::MenuFile {
        self.tabs
            .tabs()
            .iter()
            .find(|t| t.is_home)
            .and_then(|t| t.kind.menu_workspace())
            .and_then(|mw| mw.menu.clone())
            .or_else(|| {
                let pom_path = self.menus_dir().join("pom.toml");
                crate::menu_workspace::loader::load_menu_file(&pom_path).ok()
            })
            .unwrap_or_else(crate::menu_workspace::defaults::recovery_pom_menu)
    }

    /// Compute the ordered dropdown items for a top-level menu-bar button whose
    /// option command is `command` (Req 17.3/17.4/17.10).
    ///
    /// A DYNAMIC source (`THEME LIST`) generates its children at runtime;
    /// otherwise the referenced menu is PEEKED (so a POM option that opens a
    /// submenu, e.g. `SCRM` or `SETTINGS`, shows that submenu's options as its
    /// dropdown -- screen-snapshot-scrm Req 22.2); an empty peek means the
    /// command is a plain direct command and the dropdown holds a single item
    /// that dispatches it.
    pub(super) fn menu_bar_dropdown_items(&self, command: &str) -> Vec<MenuBarItem> {
        if let Some(children) = self.dynamic_menu_options(command) {
            // Dynamic children (Req 17.10, 17.11): labelled by their description
            // (e.g. the theme name), dispatch their own command.
            return children
                .into_iter()
                .map(|c| MenuBarItem {
                    label: c.description,
                    command: c.command,
                })
                .collect();
        }
        let peeked = self.peek_menu_options(command);
        if peeked.is_empty() {
            // Non-menu command: one item that dispatches it (Req 17.4).
            vec![MenuBarItem {
                label: command.to_string(),
                command: command.to_string(),
            }]
        } else {
            // Peeked submenu options: each labelled by (and dispatching) its
            // COMMAND (Req 17.3, 17.4, command parity).
            peeked
                .into_iter()
                .map(|o| MenuBarItem {
                    label: o.command.clone(),
                    command: o.command,
                })
                .collect()
        }
    }

    /// Render the items of an OPEN menu-bar dropdown, dispatch an activated item,
    /// and drive the shell-owned keyboard focus walk (menu-workspace Req 17.5).
    ///
    /// Each item is a `Button` with a STABLE id (salted by the owning top-level
    /// `top_command` and the item index). The selected item is tracked in
    /// `self.focus.menu_dropdown_index`: `nav.down` advances it (None -> 0, else
    /// +1 clamped at the last), `nav.up` retreats it. The selected item's id is
    /// STASHED into `self.focus.menu_dropdown_focus_target` and the bar renderer
    /// FORCES it into focus AFTER egui's MenuBar has run, so the shell's request
    /// is the frame's LAST focus request and wins over egui's own navigation (the
    /// B-fix -- see the module docs).
    ///
    /// `owner_index` is this dropdown's heading index on the bar; it is recorded
    /// into `menu_dropdown_owner_index` so the bar renderer knows which heading
    /// is currently open (a mouse click opens a heading without setting
    /// `menu_bar_index`). Req 17.5d: when `nav.up` is pressed while item 0 is
    /// selected, focus RETURNS to the parent heading -- the index is cleared and
    /// `menu_bar_index` is set to `owner_index` so the bar renderer focuses the
    /// heading button (instead of clamping on item 0).
    ///
    /// Enter/Space on the focused item fires the button's `clicked()`, so
    /// dispatch goes through the single front door, identical to a mouse click
    /// (Req 17.4/17.5c, command parity).
    pub(super) fn render_menu_bar_dropdown(
        &mut self,
        ui: &mut egui::Ui,
        top_command: &str,
        items: &[MenuBarItem],
        nav: MenuDropdownNav,
        owner_index: usize,
    ) {
        // Mark the dropdown open so the bar renderer can tell open from closed
        // (and reset the walk index on close), and record which heading owns the
        // open dropdown so the bar renderer's Left/Right/Up-to-heading logic
        // knows the current heading even when it was opened by a mouse click.
        self.focus.menu_dropdown_open_this_frame = true;
        self.focus.menu_dropdown_owner_index = Some(owner_index);

        // Render every item with a deterministic id, capturing the ids in order,
        // and dispatch any activated (clicked, or Enter/Space on the focused
        // item -- egui makes both fire `clicked()`) through the single front door
        // (Req 17.4/17.5c, command parity).
        let mut item_ids: Vec<egui::Id> = Vec::with_capacity(items.len());
        for (j, item) in items.iter().enumerate() {
            let salt = ("mb_dropdown_item", top_command, j);
            let resp = ui.push_id(salt, |ui| ui.button(item.label.clone())).inner;
            item_ids.push(resp.id);
            if resp.clicked() {
                // B080 Step 7 / Req 17.5c: front-door dispatch, identical to the
                // typed line, then close the dropdown.
                self.dispatch_command_string(&item.command);
                ui.close();
            }
        }

        if item_ids.is_empty() {
            return;
        }
        let last = item_ids.len() - 1;

        // Advance / retreat the tracked selection index (Req 17.5a/17.5b/17.5d).
        if nav.down {
            // Down advances: None (focus on heading) -> item 0, else +1 clamped.
            // Entering the dropdown also leaves bar keyboard mode (focus is on an
            // item now, not the heading), so Left/Right would NOT fire until Up
            // returns to the heading.
            self.focus.menu_dropdown_index = Some(match self.focus.menu_dropdown_index {
                None => 0,
                Some(i) => (i + 1).min(last),
            });
            self.focus.menu_bar_index = None;
        } else if nav.up {
            match self.focus.menu_dropdown_index {
                // Req 17.5d: Up at the FIRST item (or with nothing selected yet)
                // returns focus to the PARENT heading. Clear the item index and
                // put the bar into keyboard mode on this heading; the bar
                // renderer then focuses the heading button and keeps the dropdown
                // open.
                None | Some(0) => {
                    self.focus.menu_dropdown_index = None;
                    self.focus.menu_bar_index = Some(owner_index);
                }
                // Up elsewhere retreats to the previous item (Req 17.5b).
                Some(i) => {
                    self.focus.menu_dropdown_index = Some(i - 1);
                    self.focus.menu_bar_index = None;
                }
            }
        }

        // FORCE focus onto the selected item every frame (stashed; applied by the
        // bar renderer after egui's MenuBar runs so it is the LAST focus request
        // and egui cannot drift it). Nothing selected (index None) -> no forced
        // item focus: the dropdown opens with focus still on its top-level
        // heading until the first Down (Req 17.5a), and Up-to-heading (above)
        // likewise leaves focus for the bar renderer to put on the heading.
        if let Some(i) = self.focus.menu_dropdown_index {
            let i = i.min(last);
            self.focus.menu_dropdown_index = Some(i);
            self.focus.menu_dropdown_focus_target = Some(item_ids[i]);
        }
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
                // screen-snapshot-scrm Req 22.5 / 25: the SCRM bar item's dropdown
                // is built from the code-only SCRM menu screen when no user
                // `menus/scrm.toml` overrides it.
                "scrm" => crate::menu_workspace::defaults::recovery_scrm_menu(),
                _ => crate::menu_workspace::defaults::recovery_pom_menu(),
            });
        menu.options
    }

    /// Generate a DYNAMIC option source for a menu-bar dropdown, if `command`
    /// names one (menu-workspace Requirement 17.10, CR-NR-080 Slice D).
    ///
    /// A dynamic source produces its child options at RUNTIME rather than from a
    /// file. One source exists:
    ///
    /// - `THEME LIST`: yields one child per available theme
    ///   (`ff_theme::list_all_themes`, in list order), each labelled by the theme
    ///   name and dispatching `THEME <name>` (command parity,
    ///   theme-and-appearance Requirement 17.2, via the shared `set_active_theme`
    ///   path). Delivers the CR-NR-077 theme picker.
    ///
    /// (The former `SNAPSHOT` dynamic bar source was RETIRED by CR-NR-114: the
    /// menu bar is now a runtime echo of the POM, and SCRM is surfaced as a POM
    /// option whose dropdown is the code-only SCRM menu screen -- not a dynamic
    /// bar entry. See screen-snapshot-scrm Req 22.4.)
    ///
    /// Returns `None` for any command that is not a dynamic source, so the caller
    /// falls back to `peek_menu_options` / direct dispatch.
    ///
    /// Validates: menu-workspace Requirement 17.10, 17.11
    pub(super) fn dynamic_menu_options(
        &self,
        command: &str,
    ) -> Option<Vec<crate::menu_workspace::MenuOption>> {
        let trimmed = command.trim();
        if trimmed.eq_ignore_ascii_case("THEME LIST") {
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
            return Some(options);
        }
        None
    }
}

/// Compute the mnemonic-letter accelerator (Alt+<letter>) for each bar-visible
/// POM option, in bar order (screen-snapshot-scrm Requirement 26, CR-NR-114).
///
/// Returns `(letter, command)` pairs: pressing Alt+`letter` opens the bar item
/// whose top-level `command` is `command` (through the same dispatch seam a
/// click uses, Req 26.5).
///
/// Mnemonic selection (Req 26.1, 26.3, 26.4):
/// - The candidate is the option's `key` when it is an ASCII LETTER; when the
///   `key` is a DIGIT (a POM numeric option key, which must NOT be overloaded,
///   Req 26.4) the candidate is the first ASCII letter of the option's `command`.
/// - On a COLLISION with an already-assigned mnemonic, the next unused ASCII
///   letter is chosen -- first from the remaining letters of the command, then
///   from `A..=Z` -- so each mnemonic is distinct (Req 26.3). An option with no
///   assignable letter is skipped (no accelerator).
///
/// Only bar-visible options participate (a `show_in_menu_bar = false` option like
/// the terminate `X` is off the bar, so it gets no accelerator).
///
/// Validates: screen-snapshot-scrm Requirement 26.1, 26.3, 26.4 (CR-NR-114)
pub(super) fn menu_bar_mnemonics(
    options: &[crate::menu_workspace::MenuOption],
) -> Vec<(char, String)> {
    let mut used: std::collections::BTreeSet<char> = std::collections::BTreeSet::new();
    let mut out: Vec<(char, String)> = Vec::new();
    for option in options.iter().filter(|o| o.show_in_menu_bar) {
        // Candidate letters, in preference order: the option key when it is a
        // letter, then the command's letters, then the full alphabet.
        let key_letter = option
            .key
            .chars()
            .next()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase());
        let command_letters = option
            .command
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| c.to_ascii_uppercase());
        let alphabet = ('A'..='Z').collect::<Vec<_>>().into_iter();
        let candidate = key_letter
            .into_iter()
            .chain(command_letters)
            .chain(alphabet)
            .find(|c| !used.contains(c));
        if let Some(letter) = candidate {
            used.insert(letter);
            out.push((letter, option.command.clone()));
        }
    }
    out
}
