//! # Title / Kind Derivation Methods
//!
//! The `WorkbenchShell` title/kind method group (effective titles, Tab_Header
//! labels, Kind key-list resolution, and Kind-profile application), moved out of
//! `mod.rs` verbatim as part of the Phase 2 task 2.2 file-size split. Behaviour,
//! method names, signatures, and `pub(crate)` visibility are unchanged.
//!
//! Validates: workspace-kinds Requirement 3, 4, 5; menu-and-statusbar Req 17;
//! CR-NR-090, CR-CH-042, CR-CH-045, CR-NR-095

use super::helpers::context_name_for_tab;
use super::mod_helpers::{line_end_from_name, title_line_text};
use super::WorkbenchShell;

impl WorkbenchShell {
    /// The effective title label for a tab, consulting the Workspace Kind
    /// registry (CR-NR-090 B.1, workspace-kinds Req 3.1). For a system/panel Kind
    /// it returns the Kind's CONFIGURED title (a user Kind override wins over the
    /// compiled default); the Home Context (POM) app banner, a non-Home Menu
    /// Workspace's loaded-menu label, and the file-editor path are delegated to
    /// `title_line_text` unchanged. A per-tab `workspace_name` still takes
    /// precedence and is applied by the caller (render_tab_bar), not here.
    pub(crate) fn kind_title(&self, tab: &crate::tab_state::TabState) -> String {
        use crate::tab_state::TabKind;
        match tab.kind {
            // Panel/system Kinds: prefer the registry's effective (possibly
            // user-overridden) title, keyed by the Kind's stable name.
            TabKind::FilesPanel
            | TabKind::ConfigPanel
            | TabKind::FileExplorerPanel
            | TabKind::SearchResults
            | TabKind::PluginManager
            | TabKind::EventLog
            | TabKind::MacroLibrary
            | TabKind::CommandConfigurator
            | TabKind::ThemeEditor
            | TabKind::MenusEditor
            | TabKind::KeysEditor
            | TabKind::KindsEditor => {
                let name =
                    crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)
                        .stable_name();
                self.kind_registry.effective(name).title.clone()
            }
            // Home banner / non-Home menu label / editor path are unchanged.
            _ => title_line_text(tab),
        }
    }

    /// The Title_Line heading text for the editor/config + read-only panel
    /// Contexts covered by CR-CH-045 (menu-and-statusbar Req 17.12/17.13), and
    /// `None` for every other Context (Home/Menu/editor keep their existing
    /// derivation). For a covered Kind: a USER title override (a registry
    /// `effective(name).title` that differs from the compiled `[XXX]`
    /// `default_title`) WINS; otherwise the descriptive Title-Case
    /// `display_title` is used. This keeps the Tab_Header `[XXX]` tag
    /// (`kind_title`) intact while giving the centered Title_Line a descriptive,
    /// de-bracketed heading.
    pub(crate) fn title_line_display(&self, tab: &crate::tab_state::TabState) -> Option<String> {
        use crate::tab_state::KindTag;
        // Only the covered non-menu, non-editor Contexts get the descriptive
        // centered heading; everything else returns None (unchanged behaviour).
        let covered = matches!(
            tab.kind.tag(),
            KindTag::ConfigPanel
                | KindTag::ThemeEditor
                | KindTag::MenusEditor
                | KindTag::KeysEditor
                | KindTag::KindsEditor
                | KindTag::CommandConfigurator
                | KindTag::FilesPanel
                | KindTag::FileExplorerPanel
                | KindTag::SearchResults
                | KindTag::PluginManager
                | KindTag::EventLog
                | KindTag::MacroLibrary
        );
        if !covered {
            return None;
        }
        let builtin =
            crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home);
        let effective = self.kind_registry.effective(builtin.stable_name());
        // A user override (effective title != the compiled [XXX] tag) wins;
        // otherwise use the descriptive display title.
        if effective.title != builtin.default_title() {
            Some(effective.title.clone())
        } else {
            Some(builtin.display_title().to_string())
        }
    }

    /// The short Tab_Header label for a tab (the text on its tab-bar button),
    /// centralised so the render path has one source (CR-CH-042).
    ///
    /// Precedence: a user-assigned `workspace_name` (CX Req 1.4) wins; then a
    /// Menu Workspace derives its header from its Menu_Name (the backing
    /// `menus/<name>.toml` stem, uppercased -- what the OPENING COMMAND names,
    /// e.g. `POM`/`SETTINGS`), UNIFORMLY for the POM and every other menu with NO
    /// POM-specific branch; otherwise a system/panel Kind uses the Kind registry
    /// title (`kind_title`). The POM is NOT special here -- it is the menu named
    /// `pom`, so it derives `POM` by the same rule as any menu. Distinct from the
    /// Title_Line (`title_line_text`), which shows the full raw Menu_Title.
    pub(crate) fn tab_header_label(&self, tab: &crate::tab_state::TabState) -> String {
        use crate::tab_state::TabKind;
        if let Some(ref name) = tab.workspace_name {
            return match tab.kind {
                TabKind::FileEditor | TabKind::Untitled => format!("{}: {}", name, tab.title),
                _ => format!("[{}]", name),
            };
        }
        if tab.kind.tag() == crate::tab_state::KindTag::MenuWorkspace {
            // A Menu Workspace's header is its Menu_Name (the opening command's
            // name), uppercased: `pom` -> POM, `settings` -> SETTINGS. Same rule
            // for the POM and every other menu; no `is_home` branch. Falls back
            // to the cached title only when the menu name cannot be derived.
            return tab
                .kind
                .menu_workspace()
                .and_then(|mw| mw.menu_name_label())
                .unwrap_or_else(|| tab.title.clone());
        }
        // CR-NR-090 B.1: system/panel Kinds derive from the Kind registry.
        self.kind_title(tab)
    }

    /// The key-map context name for a tab (CR-NR-090 B.2, workspace-kinds Req
    /// 4.3): the active Kind's configured `key_list` when set, else the Kind's
    /// base context name (`context_name_for_tab`). A `key_list` naming a context
    /// with no loaded `keymaps/<name>.toml` map falls back to the global map via
    /// the resolver's existing full-replacement precedence. Behaviour-preserving
    /// for built-in Kinds (their default `key_list` is `None`).
    pub(crate) fn key_list_context_for_tab(
        &self,
        tab: &crate::tab_state::TabState,
    ) -> Option<String> {
        let kind_name =
            crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)
                .stable_name();
        if let Some(kl) = self.kind_registry.effective(kind_name).key_list.clone() {
            return Some(kl);
        }
        context_name_for_tab(tab).map(|s| s.to_string())
    }

    /// Apply the active tab's Workspace Kind profile to it (CR-NR-090 B.3,
    /// workspace-kinds Req 5). Called ONCE right after a tab is created and made
    /// active (via `shell_open_file` / `shell_new_untitled`), NOT on every
    /// activation, so a later per-tab toggle is preserved (Req 5.1).
    ///
    /// - An editor tab (FileEditor or Untitled) takes the Kind's `edit_profile`.
    /// - A NEW / Untitled buffer also takes the Kind's `line_end_mode` default; a
    ///   LOADED file keeps the mode DETECTED from its content (Req 5.2).
    /// - `tab_size` is carried in the profile but NOT applied here (no per-tab
    ///   tab-size field; Req 5.3, documented deferral).
    /// - Non-editor Kinds: no-op (edit profile is irrelevant).
    pub(crate) fn apply_kind_profile_to_active(&mut self) {
        use crate::tab_state::TabKind;
        let kind_name = {
            let t = self.tabs.active_tab();
            crate::workspace_kind::BuiltinKind::from_tab_kind(t.kind.tag(), t.is_home).stable_name()
        };
        let profile = self.kind_registry.effective(kind_name).profile.clone();
        let tab = self.tabs.active_tab_mut();
        match tab.kind {
            TabKind::Untitled => {
                tab.edit_profile = profile.edit_profile.clone();
                tab.line_end_mode = line_end_from_name(&profile.line_end_mode);
            }
            TabKind::FileEditor => {
                // Loaded file: apply the edit profile but KEEP the detected
                // line-end mode (the file's real encoding wins, Req 5.2).
                tab.edit_profile = profile.edit_profile.clone();
            }
            _ => {}
        }
    }

    /// The `Command_Line_Position` for the instance in the tab at `tab_index`
    /// (CR-NR-095, Req 8.6). Resolves from that tab's Kind's effective profile via
    /// the registry, the same resolution seam
    /// [`apply_kind_profile_to_active`](Self::apply_kind_profile_to_active) uses.
    /// Falls back to `Top` when the tab index is out of range (keeps the accessor
    /// total; a missing Kind resolves to a built-in default whose position is Top).
    pub(crate) fn command_line_position_for(
        &self,
        tab_index: usize,
    ) -> crate::workspace_kind::CommandLinePosition {
        let Some(tab) = self.tabs.tabs().get(tab_index) else {
            return crate::workspace_kind::CommandLinePosition::Top;
        };
        let kind_name =
            crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)
                .stable_name();
        self.kind_registry
            .effective(kind_name)
            .profile
            .command_line_position
    }
}
