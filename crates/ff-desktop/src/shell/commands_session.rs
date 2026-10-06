//! # Shell Command Dispatch -- Session / cursor-context / detached-window helpers
//!
//! Workspace-descriptor restore, cursor-context snapshotting, the detached
//! Focus_Context swap, the arrow-history step, and close-and-navigate-back.
//! Split out of `commands.rs` (TASK 2.2, pure code movement, no behaviour
//! change). All methods are on `WorkbenchShell` and keep their exact
//! signatures and visibility.

use eframe::egui;

use crate::tab_state::KindTag;

use super::helpers::*;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Close the current tab and navigate to the tab that was active immediately
    /// before it was opened (the top of `tab_history`), clamped to the remaining
    /// range. Shared by END from any Context, including a POM when other
    /// Workspaces remain open (Requirement 17.1, 17.2).
    ///
    /// `TabManager::close_tab` keeps at least one tab open, so callers must
    /// decide the last-Workspace case (terminate) before calling this.
    pub(super) fn close_current_and_navigate_back(&mut self) {
        let current = self.tabs.active_index();
        self.tabs.close_tab(current);
        if let Some(prev) = self.tab_history.pop() {
            let clamped = prev.min(self.tabs.len().saturating_sub(1));
            self.tabs.set_active(clamped);
        }
    }

    /// Build the current [`CursorContext`](ff_command::CursorContext) from live
    /// focus/selection state (CR-CH-028, command-framework Requirement 12.2).
    ///
    /// Start scope (Requirement 12.2, focused-control identity): the command
    /// line and the focused Menu_Option; other workspaces populate the workspace
    /// context name and editor cursor/selection where applicable and MAY add
    /// EXTRAS later. This is a per-invocation SNAPSHOT (Requirement 12.5).
    pub(super) fn capture_cursor_context(&self, ctx: &egui::Context) -> ff_command::CursorContext {
        let mut b = ff_command::CursorContext::builder();

        // (a) Focused Workspace context name.
        if let Some(name) = context_name_for_tab(self.tabs.active_tab()) {
            b = b.workspace_context(name);
        }

        // (b)/(c) Focused-control identity + text. The command field id mirrors
        // `cmd_field_id()` (the shell's "Command ===>" TextEdit). A focused Menu
        // _Option is recorded during render in `focused_menu_option`.
        let focused = ctx.memory(|m| m.focused());
        let cmd_field = egui::Id::new("command_field_input");
        if focused == Some(cmd_field) {
            b = b.focused_identity("command-line");
            if !self.command_text.trim().is_empty() {
                b = b.focused_text(self.command_text.clone());
            }
        } else if let Some((id, command, label)) = &self.focused_menu_option {
            if focused == Some(*id) {
                // The option's command is its semantic identity (e.g. "FILES");
                // the label is the human text.
                b = b.focused_identity(command.clone());
                if !label.trim().is_empty() {
                    b = b.focused_text(label.clone());
                }
            }
        }

        // (d) Editor cursor + selection when an editor document is active.
        if matches!(
            self.tabs.active_tab().kind.tag(),
            KindTag::FileEditor | KindTag::Untitled
        ) {
            let tab = self.tabs.active_tab();
            b = b
                .cursor_line(tab.cursor.cursor_line() as usize)
                .cursor_column(tab.cursor.cursor_column() as usize);
        }

        // (e) Active scroll setting (for the deferred CSR consumer).
        if !self.scroll_field_text.trim().is_empty() {
            b = b.scroll_setting(self.scroll_field_text.clone());
        }

        b.build()
    }

    /// Refresh the shared Cursor_Context snapshot so both the registry provider
    /// and the string-path commands see the same per-invocation package
    /// (CR-CH-028, command-framework Requirement 12.4). Called once per frame
    /// before any dispatch.
    pub(super) fn refresh_cursor_context_snapshot(&mut self, ctx: &egui::Context) {
        let cc = self.capture_cursor_context(ctx);
        if let Ok(mut guard) = self.cursor_context_snapshot.lock() {
            *guard = cc;
        }
    }
    /// Run `f` with the shell's command context temporarily switched to a
    /// Detached_Workspace's tab and its independent buffers (CR-CH-036,
    /// menu-and-statusbar Req 18.10). Saves the Primary_Window's active-tab index
    /// and the six per-window shell fields, installs `tab_index` as active and
    /// MOVES `ctx`'s buffers into the shell, runs `f` (which renders the detached
    /// command field and dispatches through the UNCHANGED command pipeline), then
    /// moves the (possibly command-modified) buffers back into `ctx` and restores
    /// the saved index + fields. Because the caller (the immediate-viewport loop)
    /// is synchronous, the whole existing pipeline transparently acts on the
    /// detached tab with the detached window's command line.
    ///
    /// Validates: menu-and-statusbar Requirement 18.10
    pub(super) fn with_workspace_context(
        &mut self,
        tab_index: usize,
        ctx: &mut crate::shell::WorkspaceCommandContext,
        f: impl FnOnce(&mut Self),
    ) {
        // CR-NR-093 Slice 2c.4 (Req 14.14/14.15): this is THE single Focus_Context
        // seam for a Detached_Workspace -- the one place that installs "which tab
        // is active + which per-window command buffers are live" and then restores
        // the Primary_Window exactly. It is one of the two faces of the
        // Focus_Context concept: this flat-active + command-buffer swap for a
        // detached window, and `TabManager::set_render_focus_leaf` (the tree-focus
        // swap) for an in-window region. Both are save/install/restore around a
        // scoped render/dispatch; neither leaks state. Preserving this exact
        // save/restore keeps all Req 18 detached behaviour intact (no regression).
        // B074: save the Primary_Window's active tab by STABLE `TabId`, not by
        // index. A command dispatched under the swap can CHANGE THE TAB COUNT
        // (e.g. `=0.m` triggers `insert_pom_tab`, which inserts a tab at index 0
        // and shifts every existing index up). Restoring a saved INDEX would then
        // land on a DIFFERENT tab, leaking the detached command's effect into the
        // Primary_Window. Restoring by id resolves back to the SAME tab regardless
        // of inserts/removes inside the closure.
        let saved_active_id = self.tabs.active_tab().id;
        let saved_active = self.tabs.active_index();
        let saved_command_text = std::mem::take(&mut self.command_text);
        let saved_scroll_text = std::mem::take(&mut self.scroll_field_text);
        let saved_scroll_amount = self.scroll_amount.clone();
        let saved_open_error = self.open_error.take();
        let saved_focus_req = self.focus.command_field_focus_requested;
        let saved_outcome = self.pending_command_line_outcome.take();

        // Install the detached window's context (active tab via the seam).
        self.tabs.set_active(tab_index);
        self.command_text = std::mem::take(&mut ctx.command_text);
        self.scroll_field_text = std::mem::take(&mut ctx.scroll_field_text);
        self.scroll_amount = ctx.scroll_amount.clone();
        self.open_error = ctx.open_error.take();
        self.focus.command_field_focus_requested = ctx.command_field_focus_requested;
        self.pending_command_line_outcome = ctx.pending_command_line_outcome.take();

        f(self);

        // Move the (possibly modified) detached-window buffers back into `ctx`.
        ctx.command_text = std::mem::take(&mut self.command_text);
        ctx.scroll_field_text = std::mem::take(&mut self.scroll_field_text);
        ctx.scroll_amount = self.scroll_amount.clone();
        ctx.open_error = self.open_error.take();
        ctx.command_field_focus_requested = self.focus.command_field_focus_requested;
        ctx.pending_command_line_outcome = self.pending_command_line_outcome.take();

        // Restore the Primary_Window context (active tab via the seam). B074:
        // resolve the saved TabId back to its CURRENT index (it may have shifted
        // if the closure changed the tab count); fall back to the clamped old
        // index only if that tab was closed inside the closure.
        let restore_index = self
            .tabs
            .index_of_id(saved_active_id)
            .unwrap_or_else(|| saved_active.min(self.tabs.len().saturating_sub(1)));
        self.tabs.set_active(restore_index);
        self.command_text = saved_command_text;
        self.scroll_field_text = saved_scroll_text;
        self.scroll_amount = saved_scroll_amount;
        self.open_error = saved_open_error;
        self.focus.command_field_focus_requested = saved_focus_req;
        self.pending_command_line_outcome = saved_outcome;
    }

    /// Drive one arrow-history step against the shared command-processor
    /// Command_Line_History (CR-NR-096, function-keys-and-history Requirement 23).
    ///
    /// Called from every command-field render path (shell primary, detached, and
    /// split region) with THIS field's live `command_text` bound to `self`
    /// (detached/split callers run inside `with_workspace_context`, so
    /// `self.command_text` is the region's buffer). The history and the
    /// In_Progress_Line are shared, matching the single shared Retrieve_Pointer
    /// (Req 23.9), so the behaviour is defined exactly once here.
    ///
    /// - Up (`HistoryStep::Older`): on the FIRST step of a cycle (pointer at
    ///   initial) capture the current field text as the In_Progress_Line
    ///   (Req 23.6), then recall one entry older (Req 23.1). At the oldest entry
    ///   the field is left unchanged (Req 23.4); an empty history is a no-op
    ///   (Req 23.5).
    /// - Down (`HistoryStep::Newer`): recall one entry newer (Req 23.2); stepping
    ///   past the newest entry restores the In_Progress_Line and ends the cycle
    ///   (Req 23.3); a Down with the pointer already at initial is a no-op.
    ///
    /// The recalled command is only PLACED in the field, never executed
    /// (Req 23.10), and the gesture records nothing in history (Req 23.8).
    pub(super) fn step_command_history(&mut self, step: super::render::HistoryStep) {
        use super::render::HistoryStep;
        match step {
            HistoryStep::Older => {
                // Req 23.6: capture the in-progress line at the start of a cycle.
                if self.command_line_history.is_at_initial() {
                    self.command_line_in_progress = Some(self.command_text.clone());
                }
                match self.command_line_history.retrieve(&self.command_text) {
                    ff_command::RetrieveResult::Recalled { command } => {
                        self.command_text = command;
                    }
                    // Oldest reached, empty history, or the LIST overlay trigger:
                    // leave the field unchanged for the arrow gesture (Req 23.4,
                    // 23.5). Arrow-Up never opens the LIST overlay (that is the
                    // RETRIEVE-verb path); if the field happens to hold "LIST"
                    // we simply leave it, which is harmless.
                    ff_command::RetrieveResult::ShowList { .. }
                    | ff_command::RetrieveResult::HistoryEmpty
                    | ff_command::RetrieveResult::NoOlderHistory => {}
                }
            }
            HistoryStep::Newer => match self.command_line_history.retrieve_newer() {
                ff_command::RetrieveNewerResult::Recalled { command } => {
                    self.command_text = command;
                }
                ff_command::RetrieveNewerResult::RestoreInProgress => {
                    // Req 23.3: past the newest entry -> restore what the user had
                    // typed when the cycle began (possibly empty), end the cycle.
                    if let Some(in_progress) = self.command_line_in_progress.take() {
                        self.command_text = in_progress;
                    }
                }
                // Req 23.3 (second sentence) / 23.5: no active cycle or empty
                // history -> no-op.
                ff_command::RetrieveNewerResult::NoNewer => {}
            },
        }
    }
    /// Reconstruct open Workspaces from persisted Workspace_Descriptors on
    /// session restore. Re-opens every visible Workspace in tab order, not only
    /// file-backed tabs. Unknown descriptors are skipped so one bad entry does
    /// not abort the whole restore (graceful degradation).
    ///
    /// Validates: startup-and-session Requirement 21.2, 21.3, 21.4, 21.5, 21.9.
    pub(super) fn restore_workspace_descriptors(
        &mut self,
        descriptors: &[ff_session::WorkspaceDescriptor],
    ) {
        use ff_session::session_state::{DescriptorValue, WorkspaceDescriptor, WorkspaceKind};

        for descriptor in descriptors {
            match descriptor {
                WorkspaceDescriptor::Menu { name } => {
                    // CR-CH-012 (Req 21.4): re-open a persisted Menu_Workspace
                    // backed by `menus/<name>.toml`. The POM is NOT reopened here
                    // -- the Home Context is guaranteed at index 0 by the separate
                    // POM-always-present guarantee (Req 21.8 / ensure_pom_tab_present),
                    // so restoring `Menu{name:"pom"}` here would create a duplicate.
                    // For any other menu we use the SAME framework opener the typed
                    // / click paths use (open_menu_workspace_tab), which loads the
                    // user file, or opens the load-error state when the file is
                    // absent (menu-workspace Req 1.5) rather than dropping the tab.
                    // Built-in menus stay code-only (CR-CH-021): no file is written.
                    if !name.eq_ignore_ascii_case("pom") {
                        let menus_dir = self.menus_dir();
                        let limits = crate::menu_workspace::loader::option_limits_from_config(
                            &self.config_handle,
                        );
                        self.tabs
                            .open_menu_workspace_tab(name, &menus_dir, limits, &self.runtime);
                    }
                }
                WorkspaceDescriptor::CustomWorkspace {
                    workspace_kind,
                    params,
                } => match workspace_kind {
                    WorkspaceKind::Editor => {
                        if let Some(DescriptorValue::String(uri)) = params.get("uri") {
                            if let Err(e) = self.shell_open_file(uri) {
                                self.open_error = Some(format!("Could not restore: {e}"));
                            }
                        }
                    }
                    WorkspaceKind::Files => {
                        self.tabs.open_files_panel_tab(&self.runtime);
                    }
                    WorkspaceKind::FileExplorer => {
                        self.tabs.open_file_explorer_panel_tab(&self.runtime);
                    }
                    WorkspaceKind::Config => {
                        let namespace = match params.get("namespace") {
                            Some(DescriptorValue::String(ns)) => Some(ns.clone()),
                            _ => None,
                        };
                        self.tabs.open_config_panel_tab(&self.runtime);
                        self.config_panel.filter = match &namespace {
                            Some(ns) => format!("{ns}."),
                            None => String::new(),
                        };
                        let title = match &namespace {
                            Some(ns) => format!("[CONFIG:{ns}]"),
                            None => "[CONFIG]".to_string(),
                        };
                        self.tabs.active_tab_mut().title = title;
                        self.config_panel.namespace_filter = namespace;
                    }
                    WorkspaceKind::Search => {
                        self.tabs.open_search_results_tab(&self.runtime);
                    }
                    WorkspaceKind::PluginManager => {
                        self.tabs.open_plugin_manager_tab(&self.runtime);
                    }
                    WorkspaceKind::EventLog => {
                        self.tabs.open_event_log_tab(&self.runtime);
                    }
                    WorkspaceKind::ScrmViewer => {
                        self.tabs.open_scrm_viewer_tab(&self.runtime);
                    }
                    WorkspaceKind::MacroLibrary => {
                        self.tabs.open_macro_library_tab(&self.runtime);
                    }
                    WorkspaceKind::CommandConfigurator => {
                        // Validates: startup-and-session Requirement 21.2, 21.3;
                        // command-configurator Requirement 2.1.
                        self.tabs.open_command_configurator_tab(&self.runtime);
                    }
                    WorkspaceKind::PrimaryOptionMenu => {
                        // POM presence is guaranteed by ensure_pom_tab_present;
                        // no explicit open needed here.
                    }
                    // Untitled and any future kind: skip (Req 21.9).
                    _ => {}
                },
            }
        }
    }

    /// The screen-collections directory (CR-NR-098 Wave 3): the test override
    /// when set, else `<User_Data_Dir>/screen-collections/`. CAPTURE EXPORT /
    /// SAVE / LOAD file operations resolve relative filenames under here so they
    /// route through one place and tests can isolate to a TempDir. Mirrors
    /// `menus_dir()` / `keymaps_dir()`.
    pub(super) fn scrm_dir(&self) -> std::path::PathBuf {
        if let Some(dir) = &self.dir_overrides.scrm {
            return dir.clone();
        }
        if let Ok(udd) = ff_session::UserDataDir::resolve(None) {
            return udd.path().join("screen-collections");
        }
        dirs::data_dir()
            .map(|base| base.join("FileForgeWorkbench").join("screen-collections"))
            .unwrap_or_else(|| std::path::PathBuf::from("screen-collections"))
    }

    /// Resolve a user-supplied CAPTURE file argument to an absolute path. An
    /// absolute path is used verbatim; a bare name is placed under `scrm_dir()`.
    /// An empty argument yields a default name derived from the collection.
    pub(super) fn resolve_scrm_path(
        &self,
        arg: &str,
        default_stem: &str,
        ext: &str,
    ) -> std::path::PathBuf {
        let arg = arg.trim();
        if arg.is_empty() {
            return self.scrm_dir().join(format!("{default_stem}.{ext}"));
        }
        let p = std::path::Path::new(arg);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.scrm_dir().join(arg)
        }
    }
}
