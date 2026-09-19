//! `TabManager` — manages the ordered list of open tabs.
//!
//! Owns all `TabState` instances, tracks which tab is active, and provides
//! `open_file` to load a file from the local filesystem into a new tab.

use ff_connector_local_fs::LocalFsProvider;
use ff_document_model::{new_document, BytePosition};
use ff_vfs::VfsProvider;
use tokio::runtime::Runtime;

use crate::tab_state::{TabId, TabKind, TabState};

/// Manages all open tabs and the active tab index.
pub struct TabManager {
    tabs: Vec<TabState>,
    active: usize,
    /// The tab index that was active immediately BEFORE the current `active`
    /// (CR-CH-031, multi-tab-editor Req 18.9). Updated on every real active-tab
    /// change so bare `SWAP` can toggle to the previously active workspace.
    /// `None` until a second distinct tab has been activated.
    previous_active: Option<usize>,
    next_id: u64,
}

impl TabManager {
    /// Create a manager with a single untitled welcome tab.
    pub fn new(runtime: &Runtime, welcome: &str) -> Self {
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), welcome.as_bytes());
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let tab = TabState::untitled(TabId(0), document, line_count);
        Self {
            tabs: vec![tab],
            active: 0,
            previous_active: None,
            next_id: 1,
        }
    }

    /// Set the active tab index, recording the outgoing index as the
    /// Previous_Active_Tab (CR-CH-031, Req 18.9). The single internal seam every
    /// activation path uses. A no-op activation (same index) does NOT clobber
    /// the previous pointer, so a bare-`SWAP` toggle target survives. Clamps to
    /// the valid range.
    fn activate(&mut self, index: usize) {
        let clamped = index.min(self.tabs.len().saturating_sub(1));
        if clamped != self.active {
            self.previous_active = Some(self.active);
            self.active = clamped;
        }
    }

    /// The tab index that was active immediately before the current one, or
    /// `None` when there is no distinct previous tab (CR-CH-031, Req 18.9).
    /// Returns `None` if the recorded index no longer resolves or equals the
    /// current active (so a stale/dangling pointer never toggles).
    pub fn previous_active_index(&self) -> Option<usize> {
        self.previous_active
            .filter(|&p| p < self.tabs.len() && p != self.active)
    }

    /// Number of open tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Active tab index.
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// Find the current index of the tab with the given stable `TabId`, or
    /// `None` if no such tab exists. Used by detach/redock to track a detached
    /// tab across reorderings by identity rather than a drifting index
    /// (CR-CH-035, menu-and-statusbar Req 18.9).
    pub fn index_of_id(&self, id: crate::tab_state::TabId) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == id)
    }

    /// Set the active tab by index. Clamps to valid range. Records the outgoing
    /// tab as the Previous_Active_Tab (CR-CH-031).
    pub fn set_active(&mut self, index: usize) {
        self.activate(index);
    }

    /// Immutable slice of all tabs (for rendering the tab bar).
    pub fn tabs(&self) -> &[TabState] {
        &self.tabs
    }

    /// Mutable slice of all tabs.
    pub fn tabs_mut(&mut self) -> &mut Vec<TabState> {
        &mut self.tabs
    }

    /// Mutable reference to the active tab.
    pub fn active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active]
    }

    /// Immutable reference to the active tab.
    pub fn active_tab(&self) -> &TabState {
        &self.tabs[self.active]
    }

    /// Close the initial welcome/placeholder tab if it is the only tab and has no path.
    ///
    /// Called before inserting the POM tab on first launch so the POM is the
    /// sole tab at index 0 rather than sitting behind a blank welcome tab.
    /// Validates: Requirement 14.1 — POM is always in first position on launch.
    pub fn close_welcome_tab(&mut self) {
        if self.tabs.len() == 1 && self.tabs[0].path.is_none() {
            // Replace the single placeholder tab with an empty vec; insert_pom_tab
            // will add the real first tab immediately after.
            // We cannot call close_tab (it guards len >= 1), so swap directly.
            self.tabs.clear();
            self.active = 0;
            self.previous_active = None;
        }
    }

    /// Insert a Primary Option Menu tab at index 0 and make it active.
    ///
    /// Inserts a new POM tab and makes it active.
    /// Validates: Requirement 14.1, 14.13
    pub fn insert_pom_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::pom(id, document);
        self.tabs.insert(0, tab);
        // Insert-at-0 shifts every existing index up by one, invalidating the
        // Previous_Active_Tab pointer; a fresh POM has no meaningful previous
        // (CR-CH-031). Reset it rather than track the shift.
        self.active = 0;
        self.previous_active = None;
        let _ = runtime;
    }

    /// Insert a new untitled editor tab and make it active.
    ///
    /// Validates: Requirement 14.9
    pub fn new_untitled_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::untitled(id, document, 1);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Files Panel (Virtual Catalog Manager) tab.
    ///
    /// If a FilesPanel tab already exists, activates it instead of inserting a duplicate.
    /// Validates: Requirement 1.1, 11.2
    pub fn open_files_panel_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self.tabs.iter().position(|t| t.kind == TabKind::FilesPanel) {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::files_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Config Panel tab (the flat config-key browser).
    ///
    /// If a ConfigPanel tab already exists, activates it instead of inserting a duplicate.
    /// Validates: Requirement 15.1, 15.9
    pub fn open_config_panel_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::ConfigPanel)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::config_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the File Explorer Panel (POM option 2) tab.
    ///
    /// Always opens a new tab (unlike FilesPanel which deduplicates).
    /// Validates: Requirement 19.3, 19.11
    pub fn open_file_explorer_panel_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::file_explorer_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open a new Search Results panel tab.
    ///
    /// Validates: global-search Requirement 1.1
    pub fn open_search_results_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::search_results_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Plugin Manager panel tab (POM option 8).
    ///
    /// If a PluginManager tab already exists, activates it instead of inserting a duplicate.
    /// Validates: plugin-manager-ui Requirement 1.1
    pub fn open_plugin_manager_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::PluginManager)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::plugin_manager(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Event Log panel tab.
    ///
    /// If an EventLog tab already exists, activates it instead of inserting a duplicate.
    /// Validates: notification-system Requirement 2.1
    pub fn open_event_log_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self.tabs.iter().position(|t| t.kind == TabKind::EventLog) {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::event_log(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Macro Library panel tab (POM option 6 / MACROS / =6).
    ///
    /// If a MacroLibrary tab already exists, activates it instead of inserting a duplicate.
    /// Validates: lua-macro-engine Requirement 12.1
    pub fn open_macro_library_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::MacroLibrary)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::macro_library(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Command Configurator tab (COMMANDS).
    ///
    /// If a CommandConfigurator tab already exists, activates it instead of
    /// inserting a duplicate.
    /// Validates: command-configurator Requirement 2.1
    pub fn open_command_configurator_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::CommandConfigurator)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::command_configurator(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Theme Editor tab, activating an existing one rather than
    /// inserting a duplicate.
    /// Validates: theme-and-appearance Requirement 20.1
    // CR-CH-022: navigation now transforms in place (navigate_to); this
    // dedicated-tab opener is retained for session restore and potential
    // detached-window use, but is not called by the in-place navigation path.
    #[allow(dead_code)]
    pub fn open_theme_editor_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::ThemeEditor)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::theme_editor(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Menus Editor tab, activating an existing one rather than
    /// inserting a duplicate.
    ///
    /// Validates: menu-workspace Requirement 13.1 (CR-NR-075)
    // CR-CH-022: retained for session restore / detached windows; in-place
    // navigation (navigate_to) does not use it.
    #[allow(dead_code)]
    pub fn open_menus_editor_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::MenusEditor)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::menus_editor(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open a data-driven Menu Workspace tab backed by `<menus_dir>/<name>.toml`.
    ///
    /// If a Menu Workspace tab already backed by the same file exists, it is
    /// activated instead of opening a duplicate. A missing file opens the tab in
    /// its load-error state (`MenuWorkspaceState` retains `load_error`), rather
    /// than doing nothing (menu-workspace Requirement 11.4).
    ///
    /// Validates: menu-workspace Requirement 11.2, 11.4
    pub fn open_menu_workspace_tab(
        &mut self,
        name: &str,
        menus_dir: &std::path::Path,
        limits: crate::menu_workspace::OptionLimits,
        runtime: &Runtime,
    ) {
        let file_path = menus_dir.join(format!("{name}.toml"));
        if let Some(idx) = self.tabs.iter().position(|t| {
            t.kind == TabKind::MenuWorkspace
                && t.menu_workspace
                    .as_ref()
                    .map(|mw| mw.file_path == file_path)
                    .unwrap_or(false)
        }) {
            self.activate(idx);
            return;
        }
        let mw_state =
            crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::menu_workspace_tab(id, document, mw_state);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Transform the active tab in-place from the Home Context (POM) to a new
    /// kind.
    ///
    /// No-op if the active tab is not the Home Context. Clears the `is_home`
    /// marker so the tab becomes an ordinary Context of the requested kind.
    /// Validates: Requirement 14.6
    // CR-CH-022: superseded by the shell-level navigate_to (which transforms ANY
    // current tab in place and pushes the Navigation_Stack). Retained as a
    // low-level primitive exercised by tab-manager unit tests.
    #[allow(dead_code)]
    pub fn transform_active_pom_tab(&mut self, kind: TabKind, title: &str) {
        let tab = &mut self.tabs[self.active];
        if tab.is_home {
            tab.kind = kind;
            tab.title = title.to_string();
            tab.is_home = false;
            tab.menu_workspace = None;
        }
    }

    /// Open a data-driven Menu Workspace backed by `<menus_dir>/<name>.toml`,
    /// transforming the active tab in place when it is a `MenuWorkspace` (this
    /// includes the Home Context) or a `ConfigPanel` (so the POM -> Settings_Menu
    /// / Config chain stays on one
    /// tab and F3/END transforms back to the POM), otherwise
    /// opening (or activating) a dedicated tab.
    ///
    /// Validates: cw-requirements.md Requirement 9.1, 10.4; menu-workspace Req 11.2
    // CR-CH-022: superseded by the shell reconstruct path (reconstruct_settings_menu
    // / reconstruct_named_menu drive Menu_Workspace transforms in place). Retained
    // for reference / potential reuse.
    #[allow(dead_code)]
    pub fn open_menu_workspace_here(
        &mut self,
        name: &str,
        menus_dir: &std::path::Path,
        limits: crate::menu_workspace::OptionLimits,
        runtime: &Runtime,
    ) {
        let active_kind = self.active_tab().kind;
        let transform_in_place = matches!(
            active_kind,
            TabKind::ConfigPanel | TabKind::MenuWorkspace | TabKind::MenusEditor
        );
        if transform_in_place {
            let file_path = menus_dir.join(format!("{name}.toml"));
            let mw_state =
                crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
            let title = mw_state.tab_title();
            let tab = &mut self.tabs[self.active];
            tab.kind = TabKind::MenuWorkspace;
            tab.title = title;
            tab.is_home = false;
            tab.menu_workspace = Some(mw_state);
        } else {
            self.open_menu_workspace_tab(name, menus_dir, limits, runtime);
        }
    }

    ///
    /// If the file is already open (same path), activates the existing tab
    /// instead of opening a duplicate.
    ///
    /// Returns `Err(message)` if the file cannot be read.
    pub fn open_file(&mut self, path: &str, runtime: &Runtime) -> Result<(), String> {
        // Duplicate detection — activate existing tab if already open.
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.path.as_deref() == Some(path))
        {
            self.activate(idx);
            return Ok(());
        }

        let bytes = runtime.block_on(async {
            let provider =
                LocalFsProvider::with_defaults().map_err(|e| format!("VFS init failed: {e}"))?;
            provider
                .read(path)
                .await
                .map_err(|e| format!("Cannot read '{path}': {e}"))
        })?;

        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), &bytes);
        });
        let (line_count, line_end_mode) = runtime.block_on(async {
            let doc = document.read().await;
            (doc.line_count(), doc.line_end_mode())
        });

        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::for_file(id, path.to_string(), document, line_count, line_end_mode);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        Ok(())
    }

    /// Save the active tab's document to its associated file path.
    ///
    /// Returns `Err` if the tab has no path (untitled) or the write fails.
    /// On success, clears `is_modified` and marks the document save point.
    pub fn save_active_tab(&mut self, runtime: &Runtime) -> Result<(), String> {
        let tab = &mut self.tabs[self.active];
        let path = tab
            .path
            .as_deref()
            .ok_or_else(|| "Cannot save: no file path (untitled document)".to_string())?;
        let path = path.to_string();

        let bytes = runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let view = doc.contiguous_view().to_vec();
            view
        });

        runtime.block_on(async {
            let provider =
                LocalFsProvider::with_defaults().map_err(|e| format!("VFS init failed: {e}"))?;
            provider
                .write(&path, &bytes)
                .await
                .map_err(|e| format!("Save failed: {e}"))
        })?;

        // Clear dirty flag and mark save point
        tab.is_modified = false;
        runtime.block_on(async {
            tab.document.write().await.set_save_point();
        });
        Ok(())
    }

    /// Close the tab at `index`. If it is the active tab, activates the
    /// nearest remaining tab. Always keeps at least one tab open.
    #[allow(dead_code)]
    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(index);

        // Repair the active index for the removal shift (indices > index shift
        // down by one). This is index bookkeeping, not a user activation, so it
        // does NOT go through `activate`.
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        } else if index < self.active {
            self.active -= 1;
        }

        // Repair the Previous_Active_Tab pointer for the same shift (CR-CH-031):
        // clear it if it pointed at the closed tab, else shift it down when it
        // was after the removed index. `previous_active_index()` also guards
        // against an out-of-range/equal-to-active value, so this is belt-and-braces.
        self.previous_active = match self.previous_active {
            Some(p) if p == index => None,
            Some(p) if p > index => Some(p - 1),
            other => other,
        };
    }

    /// Remove and return the tab at `index`, repairing `active` /
    /// `previous_active` for the removal shift (indices above `index` shift down
    /// by one). Unlike [`close_tab`], this has NO minimum-one guard and does not
    /// choose a neighbour to activate: it is a low-level reordering primitive for
    /// faithful redock (CR-CH-035, menu-and-statusbar Req 18.9). Panics only if
    /// `index` is out of range (caller's contract).
    ///
    /// Validates: menu-and-statusbar Requirement 18.9
    pub fn remove_at(&mut self, index: usize) -> TabState {
        let tab = self.tabs.remove(index);
        if self.active > index {
            self.active -= 1;
        } else if self.active == index {
            // The removed tab was active; clamp into range so `active` stays
            // valid. The caller (redock) typically reinserts immediately.
            self.active = self.active.min(self.tabs.len().saturating_sub(1));
        }
        self.previous_active = match self.previous_active {
            Some(p) if p == index => None,
            Some(p) if p > index => Some(p - 1),
            other => other,
        };
        tab
    }

    /// Insert `tab` at `index` (clamped to `0..=len`), repairing `active` /
    /// `previous_active` for the insertion shift (indices at or above `index`
    /// shift up by one). The inserted tab is NOT auto-activated; the caller
    /// decides. Companion to [`remove_at`] for faithful redock at the origin
    /// position (CR-CH-035, menu-and-statusbar Req 18.9).
    ///
    /// Validates: menu-and-statusbar Requirement 18.9
    pub fn insert_at(&mut self, index: usize, tab: TabState) {
        let idx = index.min(self.tabs.len());
        self.tabs.insert(idx, tab);
        if self.active >= idx {
            self.active += 1;
        }
        self.previous_active = self
            .previous_active
            .map(|p| if p >= idx { p + 1 } else { p });
    }

    /// Move the tab at `from` to position `to` (clamped to the valid range),
    /// preserving the relative order of the other tabs (a remove-then-insert,
    /// NOT a positional swap). Used by redock to restore a detached tab to its
    /// origin index (CR-CH-035, menu-and-statusbar Req 18.9). Returns the tab's
    /// final index. The moved tab keeps its identity/content/cursor/profile
    /// (same `TabState`). A no-op when `from == to`.
    ///
    /// Validates: menu-and-statusbar Requirement 18.9
    pub fn move_tab(&mut self, from: usize, to: usize) -> usize {
        if from >= self.tabs.len() {
            return from.min(self.tabs.len().saturating_sub(1));
        }
        let was_active = self.active == from;
        let tab = self.remove_at(from);
        let target = to.min(self.tabs.len());
        self.insert_at(target, tab);
        if was_active {
            self.active = target;
        }
        target
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    /// Validates: Requirement 14.13 — POM tab title is [POM].
    #[test]
    fn pom_tab_title_is_pom() {
        // Validates: Requirement 14.13
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.tabs()[0].title, "[POM]");
    }

    /// Validates: Requirement 14.1, menu-workspace 18.1 -- the Home tab is a
    /// MenuWorkspace flagged `is_home`.
    #[test]
    fn pom_tab_has_kind_primary_option_menu() {
        // Validates: Requirement 14.1; menu-workspace Requirement 18.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert!(mgr.tabs()[0].is_home);
        assert_eq!(mgr.tabs()[0].kind, TabKind::MenuWorkspace);
    }

    /// Validates: Requirement 14.1 — POM tab is inserted at index 0.
    #[test]
    fn pom_tab_inserted_at_index_zero() {
        // Validates: Requirement 14.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.active_index(), 0);
        assert!(mgr.tabs()[0].is_home);
    }

    /// Validates: Requirement 14.1 — inserting POM twice opens two POM tabs.
    #[test]
    fn insert_pom_tab_twice_opens_two_pom_tabs() {
        // Validates: Requirement 14.1 — START always opens a new POM tab
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        let count_after_first = mgr.len();
        mgr.insert_pom_tab(&runtime);
        assert_eq!(
            mgr.len(),
            count_after_first + 1,
            "second insert_pom_tab must open a new POM tab"
        );
        assert!(mgr.active_tab().is_home);
    }

    /// Validates: Requirement 14.9 — new_untitled_tab adds an Untitled tab.
    #[test]
    fn new_untitled_tab_adds_untitled_kind() {
        // Validates: Requirement 14.9
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        let before = mgr.len();
        mgr.new_untitled_tab(&runtime);
        assert_eq!(mgr.len(), before + 1);
        assert_eq!(mgr.active_tab().kind, TabKind::Untitled);
    }

    /// Validates: Requirement 14.1 — file tab has kind FileEditor.
    #[test]
    fn file_tab_has_kind_file_editor() {
        // Validates: Requirement 14.1
        use std::io::Write;
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "hello").expect("write");
        let path = tmp.path().to_string_lossy().into_owned();
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_file(&path, &runtime).expect("open");
        assert_eq!(mgr.active_tab().kind, TabKind::FileEditor);
    }

    #[test]
    fn save_writes_document_content_to_file() {
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_string_lossy().into_owned();

        let mut mgr = TabManager::new(&runtime, "");
        // Replace the welcome tab with a file-backed tab at the temp path
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"saved content");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let tab = TabState::for_file(
            TabId(1),
            path.clone(),
            document,
            line_count,
            ff_document_model::LineEndMode::Default,
        );
        mgr.tabs[0] = tab;

        let result = mgr.save_active_tab(&runtime);
        assert!(result.is_ok(), "save should succeed: {result:?}");

        let written = std::fs::read(&path).expect("read back");
        assert_eq!(written, b"saved content");
    }

    /// Validates: file-operations Requirement 1.2 — save clears the modified flag.
    #[test]
    fn save_clears_modified_flag() {
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_string_lossy().into_owned();

        let mut mgr = TabManager::new(&runtime, "");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::for_file(
            TabId(1),
            path.clone(),
            document,
            line_count,
            ff_document_model::LineEndMode::Default,
        );
        tab.is_modified = true;
        mgr.tabs[0] = tab;

        mgr.save_active_tab(&runtime).expect("save");
        assert!(!mgr.active_tab().is_modified);
    }

    /// Validates: file-operations Requirement 1.4 — save on untitled tab returns error.
    #[test]
    fn save_on_untitled_tab_is_noop() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "some content");
        // The default welcome tab is untitled (no path)
        let result = mgr.save_active_tab(&runtime);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("untitled"));
    }

    /// Validates: task 18.4 — TabManager starts with one untitled tab.
    #[test]
    fn new_manager_has_one_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "hello\n");
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_tab().title, "Untitled");
    }

    /// Validates: task 18.4 — opening a missing file returns an error.
    #[test]
    fn open_nonexistent_file_returns_error() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        let result = mgr.open_file("/nonexistent/path/file.txt", &runtime);
        assert!(result.is_err());
        assert_eq!(mgr.len(), 1); // no new tab added
    }

    /// Validates: task 18.5 — closing a tab reduces count; last tab is preserved.
    #[test]
    fn close_tab_preserves_minimum_one() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.close_tab(0);
        assert_eq!(mgr.len(), 1); // cannot go below 1
    }

    /// Helper: a manager with N titled tabs (title = "T{i}") for order assertions.
    #[cfg(test)]
    fn mgr_with_titled(runtime: &Runtime, n: usize) -> TabManager {
        let mut mgr = TabManager::new(runtime, "");
        mgr.tabs[0].title = "T0".to_string();
        for i in 1..n {
            mgr.new_untitled_tab(runtime);
            let last = mgr.tabs.len() - 1;
            mgr.tabs[last].title = format!("T{i}");
        }
        mgr
    }

    /// Validates: menu-and-statusbar Req 18.9 -- remove_at returns the tab and
    /// shifts higher indices down, preserving the order of the rest.
    #[test]
    fn remove_at_returns_tab_and_preserves_order() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4); // T0 T1 T2 T3
        let removed = mgr.remove_at(1);
        assert_eq!(removed.title, "T1");
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["T0", "T2", "T3"]);
    }

    /// Validates: menu-and-statusbar Req 18.9 -- insert_at places the tab at the
    /// index and shifts the rest up, preserving order.
    #[test]
    fn insert_at_places_tab_and_preserves_order() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 3); // T0 T1 T2
        let removed = mgr.remove_at(2); // T2 out; [T0 T1]
        mgr.insert_at(1, removed); // -> [T0 T2 T1]
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["T0", "T2", "T1"]);
    }

    /// Validates: menu-and-statusbar Req 18.9 -- move_tab restores a tab to its
    /// origin index preserving the order of the other tabs (NOT a swap).
    #[test]
    fn move_tab_restores_to_origin_preserving_order() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4); // T0 T1 T2 T3
                                                    // Simulate a tab that was detached from index 1 and now sits at the end
                                                    // (as if appended); move it back to origin 1.
        let t1 = mgr.remove_at(1); // [T0 T2 T3]
        mgr.insert_at(mgr.len(), t1); // [T0 T2 T3 T1]
        let moved_from = mgr.tabs().iter().position(|t| t.title == "T1").unwrap();
        mgr.move_tab(moved_from, 1); // back to origin 1
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["T0", "T1", "T2", "T3"],
            "move_tab must reinsert at origin, not swap (order preserved)"
        );
    }

    /// Validates: menu-and-statusbar Req 18.9 -- move_tab keeps the moved tab
    /// active when it was active, tracking its new index.
    #[test]
    fn move_tab_follows_active_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4); // active = 3 (last)
        mgr.set_active(3);
        mgr.move_tab(3, 0);
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_tab().title, "T3");
    }

    /// Validates: menu-and-statusbar Req 18.9 -- move_tab clamps an origin index
    /// beyond the current count to the end (append), per Req 18.3.
    #[test]
    fn move_tab_clamps_origin_beyond_count_to_end() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 3); // T0 T1 T2
        let final_idx = mgr.move_tab(0, 99); // origin beyond count -> append
        assert_eq!(final_idx, 2);
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["T1", "T2", "T0"]);
    }

    // Validates: multi-tab-editor Req 18.9 (CR-CH-031) -- activating a different
    // tab records the outgoing index as the Previous_Active_Tab; a no-op
    // activation does not clobber it.
    #[test]
    fn previous_active_tracks_last_other_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.new_untitled_tab(&runtime); // now 2 tabs, active = 1, prev = 0
        assert_eq!(mgr.active_index(), 1);
        assert_eq!(mgr.previous_active_index(), Some(0));

        mgr.set_active(0); // active = 0, prev = 1
        assert_eq!(mgr.previous_active_index(), Some(1));

        // A no-op activation (same index) must NOT change the previous pointer.
        mgr.set_active(0);
        assert_eq!(mgr.previous_active_index(), Some(1));
    }

    // Validates: multi-tab-editor Req 18.9 -- a single-tab manager has no
    // distinct previous tab.
    #[test]
    fn previous_active_none_with_single_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "");
        assert_eq!(mgr.previous_active_index(), None);
    }

    // Validates: multi-tab-editor Req 18.9 -- closing a tab repairs the
    // Previous_Active_Tab pointer (clears it if it pointed at the closed tab).
    #[test]
    fn previous_active_repaired_on_close() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.new_untitled_tab(&runtime); // 2 tabs, active 1, prev 0
        mgr.new_untitled_tab(&runtime); // 3 tabs, active 2, prev 1
        assert_eq!(mgr.previous_active_index(), Some(1));
        // Close the previous tab (index 1); the pointer must not dangle.
        mgr.close_tab(1);
        assert_eq!(
            mgr.previous_active_index(),
            None,
            "closing the previous tab clears the dangling pointer"
        );
    }

    /// Validates: task 18.5 — set_active clamps to valid range.
    #[test]
    fn set_active_clamps_to_valid_range() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.set_active(999);
        assert_eq!(mgr.active_index(), 0);
    }

    /// Validates: Requirement 18.7 — untitled tab reports correct line_count.
    #[test]
    fn untitled_tab_line_count_matches_content() {
        // Validates: Requirement 7.4 — total line count segment shows real count
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "line1\nline2\nline3\n");
        // 3 newlines → 4 lines (last empty line after final \n)
        assert_eq!(mgr.active_tab().line_count, 4);
    }

    /// Validates: Requirement 18.7 — untitled tab encoding_label defaults to UTF-8.
    #[test]
    fn untitled_tab_encoding_label_is_utf8() {
        // Validates: Requirement 7.3 — encoding segment shows detected encoding
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "hello\n");
        assert_eq!(mgr.active_tab().encoding_label(), "UTF-8");
    }

    // ── Phase AM: Detachable tab windows ────────────────────────────────────

    /// Validates: Requirement 18.4 — is_floating defaults to false on new tabs.
    #[test]
    fn floating_tab_is_floating_flag_defaults_to_false() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "");
        assert!(!mgr.active_tab().is_floating);
    }

    /// Validates: Requirement 18.4 — is_floating can be set to true.
    #[test]
    fn floating_tab_is_floating_flag_can_be_set() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.tabs_mut()[0].is_floating = true;
        assert!(mgr.active_tab().is_floating);
    }

    /// Validates: Requirement 18.4 — POM tab also defaults is_floating to false.
    #[test]
    fn pom_tab_is_floating_defaults_to_false() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert!(!mgr.tabs()[0].is_floating);
    }

    /// Validates: Requirement 1.1, 11.2 — option 1 opens a FilesPanel tab with title [FILES].
    #[test]
    fn files_panel_tab_has_kind_files_panel_and_title() {
        // Validates: Requirement 1.1, 11.2
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_files_panel_tab(&runtime);
        let tab = mgr.active_tab();
        assert_eq!(tab.kind, TabKind::FilesPanel);
        assert_eq!(tab.title, "[FILES]");
    }

    /// Validates: Requirement 1.1 — opening FilesPanel twice does not duplicate it.
    #[test]
    fn open_files_panel_tab_twice_does_not_duplicate() {
        // Validates: Requirement 1.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_files_panel_tab(&runtime);
        let count = mgr.len();
        mgr.open_files_panel_tab(&runtime);
        assert_eq!(mgr.len(), count, "second open must not add a duplicate");
    }
}
