//! Navigation_Stack Context reconstruction helpers (menu-workspace Req 14,
//! CR-CH-022). Moved out of `nav_stack.rs` verbatim as part of the Phase 2 task
//! 2.2 file-size split; behaviour, method names, signatures, and visibility are
//! unchanged. These `impl WorkbenchShell` methods re-derive a Context on the
//! ACTIVE tab in place (no new tab) from a `WorkspaceDescriptor`.

use ff_session::session_state::{
    DescriptorParams, DescriptorValue, WorkspaceDescriptor, WorkspaceKind,
};

use super::WorkbenchShell;
use crate::tab_state::TabKind;

impl WorkbenchShell {
    /// Reconstruct a `WorkspaceDescriptor` as the active tab's Context, IN PLACE
    /// (no new tab). Re-derives shell-global Context state exactly as opening the
    /// Context does.
    ///
    /// Validates: menu-workspace Requirement 14.4, 14.6
    pub(super) fn reconstruct_context(&mut self, descriptor: &WorkspaceDescriptor) {
        match descriptor {
            WorkspaceDescriptor::Menu { name } => {
                // POM is the Home Context; any other name is a Menu_Workspace.
                if name.eq_ignore_ascii_case("pom") {
                    self.set_active_tab_home();
                } else if name.eq_ignore_ascii_case("settings") {
                    self.reconstruct_settings_menu();
                } else {
                    self.reconstruct_named_menu(name);
                }
            }
            WorkspaceDescriptor::CustomWorkspace {
                workspace_kind,
                params,
            } => self.reconstruct_custom(workspace_kind.clone(), params),
        }
    }

    /// Set the active tab's kind + title in place (no push, no new tab).
    pub(super) fn set_active_tab_context(&mut self, kind: TabKind, title: &str) {
        let tab = self.tabs.active_tab_mut();
        tab.kind = kind;
        tab.title = title.to_string();
        // Only the dedicated Home reconstruction sets `is_home`; any other
        // Context reset clears it so a former Home tab becomes a plain Context.
        tab.is_home = false;
    }

    /// Reconstruct the Home Context (POM) on the active tab in place: a
    /// `MenuWorkspace` tab flagged `is_home`, with the menu cleared so
    /// `ensure_pom_menu_loaded` re-seeds `pom.toml` (or the barebones fallback).
    ///
    /// Validates: menu-workspace Requirement 18.2, 18.4
    pub(super) fn set_active_tab_home(&mut self) {
        {
            let tab = self.tabs.active_tab_mut();
            tab.kind = TabKind::MenuWorkspace(None);
            tab.title = "[POM]".to_string();
            tab.is_home = true;
        }
        self.ensure_pom_menu_loaded();
    }

    /// The active UI locale tag, read the SAME way `i18n_bootstrap` reads it:
    /// `ui.locale` from the config handle, falling back to the English
    /// Identity_Base tag when unset/unreadable (CR-NR-111, localization Req
    /// 12.7, 12.7b). This is the segment used to look for a per-language saved
    /// menu file at the user-menu reconstruct seams.
    fn active_locale_tag(&self) -> String {
        self.config_handle
            .get_string(ff_config::keys::ui::LOCALE)
            .unwrap_or_else(|_| ff_i18n::IDENTITY_BASE_TAG.to_string())
    }

    /// Resolve the saved-menu path for user menu `name` under `menus_dir`,
    /// SECOND PRIZE (CR-NR-111, localization Req 12.7): the per-language file
    /// `menus/<locale>/<name>.toml` when it exists, else the base
    /// `menus/<name>.toml`. Both user-menu reconstruct seams share this ONE
    /// helper so they carry the locale segment IDENTICALLY. When `ui.locale` is
    /// the English Identity_Base (or no per-locale file exists) the base path is
    /// returned exactly as before, so the `en` default stays behaviour-neutral
    /// (Req 10.2). The POM is OUT of scope and does NOT call this (Req 12.7a).
    ///
    /// Validates: localization Requirement 12.7, 12.7b
    fn locale_qualified_menu_path(
        &self,
        menus_dir: &std::path::Path,
        name: &str,
    ) -> std::path::PathBuf {
        let base = menus_dir.join(format!("{name}.toml"));
        let locale = self.active_locale_tag();
        // The English Identity_Base never gets a per-locale directory; this keeps
        // the en path byte-identical to the pre-CR-NR-111 behaviour.
        if locale.eq_ignore_ascii_case(ff_i18n::IDENTITY_BASE_TAG) {
            return base;
        }
        let localised = menus_dir.join(&locale).join(format!("{name}.toml"));
        if localised.is_file() {
            localised
        } else {
            base
        }
    }

    /// Reconstruct the Settings menu (Menu_Workspace backed by settings.toml,
    /// with the compiled fallback) on the active tab in place.
    fn reconstruct_settings_menu(&mut self) {
        let menus_dir = self.menus_dir();
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        // SECOND PRIZE (CR-NR-111 Req 12.7): prefer a per-language settings menu
        // file when one exists, else the base file (unchanged at en).
        let file_path = self.locale_qualified_menu_path(&menus_dir, "settings");
        let mut mw =
            crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
        if mw.menu.is_none() {
            // Fall back to the compiled Recovery_Baseline (CR-CH-021 Req 12.4/
            // 12.5). Retain a PARSE error in load_error so the caller
            // (open_settings_menu) can surface a bypassed-file notice; clear a
            // mere "not found" (absent file is silent).
            let keep_parse_error = mw
                .load_error
                .as_deref()
                .map(|e| e.starts_with("Menu file error"))
                .unwrap_or(false);
            mw.menu = Some(crate::menu_workspace::defaults::recovery_settings_menu());
            if !keep_parse_error {
                mw.load_error = None;
            }
        }
        let tab = self.tabs.active_tab_mut();
        tab.kind = TabKind::MenuWorkspace(Some(mw));
        tab.title = "[SETTINGS]".to_string();
        tab.is_home = false;
    }

    /// Reconstruct a named Menu_Workspace on the active tab in place.
    fn reconstruct_named_menu(&mut self, name: &str) {
        let menus_dir = self.menus_dir();
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        // SECOND PRIZE (CR-NR-111 Req 12.7): prefer a per-language file for this
        // user menu when one exists, else the base file (unchanged at en).
        let file_path = self.locale_qualified_menu_path(&menus_dir, name);
        let mw = crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
        let title = mw.tab_title();
        let tab = self.tabs.active_tab_mut();
        tab.kind = TabKind::MenuWorkspace(Some(mw));
        tab.title = title;
        tab.is_home = false;
    }

    /// Reconstruct a CustomWorkspace kind on the active tab in place, re-deriving
    /// any shell-global state from params.
    fn reconstruct_custom(&mut self, kind: WorkspaceKind, params: &DescriptorParams) {
        match kind {
            // Legacy sessions persisted the POM as this custom kind; route it to
            // the unified Home Context (menu-workspace Req 18.8).
            WorkspaceKind::PrimaryOptionMenu => {
                self.set_active_tab_home();
            }
            WorkspaceKind::Config => {
                let namespace = match params.get("namespace") {
                    Some(DescriptorValue::String(ns)) => Some(ns.clone()),
                    _ => None,
                };
                let title = match &namespace {
                    Some(ns) => format!("[CONFIG:{ns}]"),
                    None => "[CONFIG]".to_string(),
                };
                self.config_panel.filter = match &namespace {
                    Some(ns) => format!("{ns}."),
                    None => String::new(),
                };
                self.config_panel.namespace_filter = namespace;
                self.set_active_tab_context(TabKind::ConfigPanel, &title);
            }
            WorkspaceKind::Files => self.set_active_tab_context(TabKind::FilesPanel, "[CATALOGS]"),
            WorkspaceKind::FileExplorer => {
                self.set_active_tab_context(TabKind::FileExplorerPanel, "[FILES]")
            }
            WorkspaceKind::Search => {
                self.set_active_tab_context(TabKind::SearchResults, "[SEARCH]")
            }
            WorkspaceKind::PluginManager => {
                self.set_active_tab_context(TabKind::PluginManager, "[PLUGINS]")
            }
            WorkspaceKind::EventLog => self.set_active_tab_context(TabKind::EventLog, "[LOG]"),
            WorkspaceKind::ScrmViewer => {
                self.set_active_tab_context(TabKind::ScrmViewer, "[REPLAY]")
            }
            WorkspaceKind::MacroLibrary => {
                self.set_active_tab_context(TabKind::MacroLibrary, "[MACROS]")
            }
            WorkspaceKind::CommandConfigurator => {
                // The transient editors are encoded as CommandConfigurator with an
                // `editor` marker param (see descriptor_for_current_context). Set
                // the kind/title directly; the editors' shell-global working
                // state is still present, so no re-open (which would re-navigate)
                // is needed.
                match params.get("editor") {
                    Some(DescriptorValue::String(e)) if e == "theme" => {
                        self.set_active_tab_context(TabKind::ThemeEditor, "[THEME]")
                    }
                    Some(DescriptorValue::String(e)) if e == "menus" => {
                        self.set_active_tab_context(TabKind::MenusEditor, "[MENUS]")
                    }
                    Some(DescriptorValue::String(e)) if e == "keys" => {
                        self.set_active_tab_context(TabKind::KeysEditor, "[KEYS]")
                    }
                    Some(DescriptorValue::String(e)) if e == "kinds" => {
                        self.set_active_tab_context(TabKind::KindsEditor, "[KINDS]")
                    }
                    Some(DescriptorValue::String(e)) if e == "aliases" => {
                        self.set_active_tab_context(TabKind::AliasEditor, "[ALIASES]")
                    }
                    _ => self.set_active_tab_context(TabKind::CommandConfigurator, "[COMMANDS]"),
                }
            }
            WorkspaceKind::Editor | WorkspaceKind::Untitled => {
                // An editor Context cannot be reconstructed without its document;
                // fall back to the POM rather than leave a blank editor.
                self.set_active_tab_home();
            }
            // WorkspaceKind is #[non_exhaustive]; any future kind falls back to
            // the POM so navigation never lands on an unhandled Context.
            _ => {
                self.set_active_tab_home();
            }
        }
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use crate::shell::tests_common::make_shell;

    /// Set the active UI locale on the shell's config handle (user layer).
    fn set_locale(shell: &super::WorkbenchShell, tag: &str) {
        shell
            .config_handle
            .set_user_value(
                ff_config::keys::ui::LOCALE,
                ff_config::ConfigValue::String(tag.to_string()),
            )
            .expect("set ui.locale");
    }

    // Validates: localization Requirement 12.7, 10.2 -- SECOND PRIZE user-menu
    // path resolution: with ui.locale = en the base menus/<name>.toml is returned
    // byte-identically to the pre-CR-NR-111 behaviour (no per-locale directory is
    // ever consulted for the English Identity_Base).
    #[test]
    fn locale_qualified_menu_path_en_uses_base_file() {
        let shell = make_shell();
        set_locale(&shell, "en");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let menus = dir.path().to_path_buf();
        assert_eq!(
            shell.locale_qualified_menu_path(&menus, "mymenu"),
            menus.join("mymenu.toml"),
            "at en the base path is used, never a per-locale directory (Req 10.2)"
        );
    }

    // Validates: localization Requirement 12.7, 12.7b -- SECOND PRIZE: with a
    // non-en locale AND a per-language file present, the locale-qualified path
    // menus/<locale>/<name>.toml is returned; with the file ABSENT it falls back
    // to the base menus/<name>.toml.
    #[test]
    fn locale_qualified_menu_path_prefers_per_locale_file_else_base() {
        let shell = make_shell();
        set_locale(&shell, "af");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let menus = dir.path().to_path_buf();

        // No per-locale file yet -> base path.
        assert_eq!(
            shell.locale_qualified_menu_path(&menus, "settings"),
            menus.join("settings.toml"),
            "with no per-locale file the base path is used (Req 12.7 fallback)"
        );

        // Create menus/af/settings.toml -> the per-locale path is now preferred.
        let af_dir = menus.join("af");
        std::fs::create_dir_all(&af_dir).expect("create af dir");
        std::fs::write(af_dir.join("settings.toml"), "title = \"Instellings\"\n")
            .expect("write af settings");
        assert_eq!(
            shell.locale_qualified_menu_path(&menus, "settings"),
            af_dir.join("settings.toml"),
            "a present per-language file is preferred (Req 12.7)"
        );
    }

    // Validates: localization Requirement 12.7a -- the POM load path is OUT of
    // scope for the second-prize FILE override: ensure_pom_menu_loaded builds
    // menus_dir()/pom.toml directly and NEVER consults a per-locale directory.
    // Even with a non-en locale AND a menus/<locale>/pom.toml present, the POM
    // must load from the base pom.toml (the per-locale file is ignored for POM).
    #[test]
    fn pom_load_path_ignores_per_locale_file() {
        let mut shell = make_shell();
        set_locale(&shell, "af");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let menus = dir.path().join("menus");
        std::fs::create_dir_all(&menus).expect("create menus dir");
        // Base POM with one option.
        std::fs::write(
            menus.join("pom.toml"),
            "title = \"Base POM\"\nshow_calendar = false\n\
             [[options]]\nkey = \"1\"\ncommand = \"CATALOGS\"\ndescription = \"Catalogs\"\n",
        )
        .expect("write base pom");
        // A per-locale POM that must be IGNORED (Req 12.7a).
        let af = menus.join("af");
        std::fs::create_dir_all(&af).expect("create af dir");
        std::fs::write(
            af.join("pom.toml"),
            "title = \"AF POM\"\nshow_calendar = false\n\
             [[options]]\nkey = \"9\"\ncommand = \"ZZZ\"\ndescription = \"Afr\"\n",
        )
        .expect("write af pom");

        shell.dir_overrides.menus = Some(menus.clone());
        // Fresh Home tab, menu cleared, so ensure_pom_menu_loaded re-seeds.
        shell.set_active_tab_home();

        let mw = shell
            .tabs
            .active_tab()
            .kind
            .menu_workspace()
            .expect("home is a menu workspace");
        let menu = mw.menu.as_ref().expect("pom menu loaded");
        assert_eq!(
            menu.title, "Base POM",
            "the POM must load the BASE pom.toml, never the per-locale file (Req 12.7a)"
        );
        assert_eq!(
            menu.options[0].command, "CATALOGS",
            "the POM options come from the base file, not the per-locale POM (Req 12.7a)"
        );
    }
}
