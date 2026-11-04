//! # Shell Action / Lifecycle Methods
//!
//! The remaining small `WorkbenchShell` method group that lived at the top of
//! the `impl WorkbenchShell` block in `mod.rs`: the notification-sender accessor,
//! macro directory lookup, the Macro Library and Search Results action/outcome
//! appliers, the session-lifecycle label helpers, and the shell open-file /
//! new-untitled seams. Moved verbatim as part of the Phase 2 task 2.2 file-size
//! split; behaviour, names, signatures, and visibility are unchanged.

use crate::notification::NotificationSender;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Return a cloned `NotificationSender` for use by background tasks.
    ///
    /// Validates: notification-system Requirement 3.1
    #[allow(dead_code)]
    pub fn notification_sender(&self) -> NotificationSender {
        NotificationSender::new(self.notification_tx.clone())
    }

    /// Return the list of directories to scan for Lua macro files.
    ///
    /// Validates: lua-macro-engine Requirement 12.2
    pub(crate) fn macro_dirs(&self) -> Vec<String> {
        if let Some(base) = dirs::data_dir() {
            let dir = base.join("FileForgeWorkbench").join("macros");
            vec![dir.to_string_lossy().into_owned()]
        } else {
            Vec::new()
        }
    }

    /// Apply a [`MacroLibraryAction`] produced by the Macro Library Context
    /// (CR-NR-078 WF.6). Behaviour is identical to the pre-framework inline arm:
    /// Edit opens the macro file, Run is not yet available, Delete removes the
    /// file and refreshes the inventory.
    ///
    /// Validates: lua-macro-engine Requirement 12.3, 12.8
    pub(super) fn apply_macro_library_action(
        &mut self,
        action: crate::macro_library_panel::MacroLibraryAction,
    ) {
        use crate::macro_library_panel::MacroLibraryAction as A;
        match action {
            A::Edit(path) => {
                let mut p = ff_command::CommandParams::new();
                p.insert("path", path.as_str());
                let _ = self.dispatch.execute_command("file.open", p);
            }
            A::Run(_path) => {
                self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-lua-not-available"));
            }
            A::Delete(path) => {
                if let Err(e) = std::fs::remove_file(&path) {
                    self.open_error = Some(ff_i18n::t_args(
                        "ff-desktop-shell-msg-delete-failed",
                        &ff_i18n::MessageArgs::new().set("error", e.to_string()),
                    ));
                } else {
                    let dirs = self.macro_dirs();
                    self.macro_library_panel.refresh(&dirs);
                    self.open_error = None;
                }
            }
            A::None => {}
        }
    }

    /// Apply a [`SearchPanelOutcome`] produced by the Search Results Context
    /// (CR-NR-078 WF.6). Behaviour is identical to the pre-framework inline arm:
    /// OpenMatch opens the file and scrolls to the line; ReplaceAll runs the
    /// global replace against the staged `roots`. `Cancel`/`None` are no-ops here
    /// (Cancel is handled inside the panel render).
    ///
    /// Validates: global-search Requirement 1.1, 4.1
    pub(super) fn apply_search_outcome(
        &mut self,
        roots: Vec<String>,
        outcome: crate::search_results_panel::render::SearchPanelOutcome,
    ) {
        use crate::search_results_panel::render::SearchPanelOutcome as O;
        match outcome {
            O::OpenMatch { path, line } => {
                if let Err(e) = self.shell_open_file(&path) {
                    self.open_error = Some(e);
                } else {
                    // Scroll to the matching line.
                    let idx = self.tabs.active_index();
                    if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                        tab.viewport
                            .scroll_to_line(line.saturating_sub(1).max(1), &tab.cursor.clone());
                    }
                }
            }
            O::ReplaceAll => {
                let unsaved: Vec<String> = self
                    .tabs
                    .tabs()
                    .iter()
                    .filter(|t| t.is_modified)
                    .filter_map(|t| t.path.clone())
                    .collect();
                let req = self.search_results_panel.build_request(roots).ok();
                if let Some(r) = req {
                    let results = self.search_results_panel.results.clone();
                    match ff_global_search::GlobalReplaceEngine::replace_all(
                        &results,
                        &r,
                        &self.search_results_panel.replace_text.clone(),
                        &unsaved,
                    ) {
                        Ok((summary, _conflicts)) => {
                            // Req 7.28: a result summary is informational.
                            self.post_info(ff_i18n::t_args(
                                "ff-desktop-shell-msg-replace-summary",
                                &ff_i18n::MessageArgs::new()
                                    .set("count", summary.replacements.to_string())
                                    .set("files", summary.files_modified.to_string()),
                            ));
                        }
                        Err(e) => {
                            self.open_error = Some(ff_i18n::t_args(
                                "ff-desktop-shell-msg-replace-failed",
                                &ff_i18n::MessageArgs::new().set("error", e.to_string()),
                            ));
                        }
                    }
                }
            }
            O::Cancel | O::None => {}
        }
    }

    // === Session lifecycle helpers -- Validates: Requirement 20.1, 20.2 ===

    /// Mark that a user action has just been dispatched (CR-NR-115,
    /// workspace-framework Phase CF). This is the ONE per-action hook wired into
    /// the shared dispatch seams (`begin_command_line`, `dispatch_bound_command`):
    /// it CLEARS the transient Message_Line (information messages live only until
    /// the next action, Req 7.14) and STAMPS `last_action_at` so the Status_Bar
    /// `Last: HH:MM` segment reflects the moment of the action (Req 7.16). An
    /// action is free to re-populate the Message_Line AFTER this clears it.
    ///
    /// Validates: Requirement 7.12, 7.14, 7.16
    pub(crate) fn note_action_dispatched(&mut self) {
        // Req 7.29: remember the frame as it was BEFORE this action so a
        // SNAPSHOT/CAPTURE issued by it reproduces that state. Idempotent per
        // action (the key-forward path invokes this seam twice).
        self.stash_pre_action_frame();
        self.message_line.clear();
        self.last_action_at = chrono::Local::now();
    }

    /// Stamp `last_action_at` for a SCREEN TRANSITION (Req 7.16): any change of
    /// the visible Context or active tab. Unlike an action it does not clear
    /// the Message_Line nor stash a pre-action frame, so a capture issued by an
    /// in-flight action still shows the time BEFORE that action.
    ///
    /// Shared seams: `reconstruct_context` (every `navigate_to` push/pop, END,
    /// RETURN, menu and CustomWorkspace open) and `sync_active_tab_transition`
    /// (every active-tab change, run once per frame from `update`).
    pub(crate) fn note_screen_transition(&mut self) {
        self.last_action_at = chrono::Local::now();
    }

    /// Stamp a transition when the ACTIVE TAB IDENTITY differs from the one seen
    /// last time (tab click, keyboard switch, START, close moving focus, session
    /// restore). Identity (not index) is compared so a close that leaves the
    /// index unchanged still registers. Returns true when a transition fired.
    pub(crate) fn sync_active_tab_transition(&mut self) -> bool {
        let id = self.tabs.active_tab().id;
        if self.focus.last_active_tab_id == Some(id) {
            return false;
        }
        self.focus.last_active_tab_id = Some(id);
        self.note_screen_transition();
        true
    }

    /// Format the last-action time as `Last: HH:MM` for the Status_Bar
    /// (CR-NR-115, workspace-framework Phase CF, Req 7.18). This is NOT a ticking
    /// clock -- it reflects `last_action_at`: the most recent action or screen
    /// transition, initialised to the startup time, so it is never `--:--`.
    ///
    /// Validates: Requirement 7.16, 7.18
    pub(crate) fn format_last_action(&self) -> String {
        Self::last_action_label(self.last_action_at)
    }

    /// Format a last-action stamp as `Last: HH:MM`. Shared by the live bar and
    /// the pre-action capture band.
    pub(crate) fn last_action_label(stamp: chrono::DateTime<chrono::Local>) -> String {
        format!("Last: {}", stamp.format("%H:%M"))
    }

    /// The Active_Profile label for the Status_Bar / Title_Line (CR-NR-081,
    /// startup-and-session Requirement 22.8). Shows the running Application_Profile
    /// name, or `Prof: default` when no `--profile` was given. Shortened from
    /// `Profile:` to `Prof:` for the common frame (CR-NR-115, Req 7.19).
    ///
    /// Validates: Requirement 7.19; startup-and-session Requirement 22.8
    pub(crate) fn active_profile_label(&self) -> String {
        match ff_session::active_profile() {
            Some(name) => format!("Prof: {name}"),
            None => "Prof: default".to_string(),
        }
    }

    /// The Zoom common status segment label: the signed offset from the default
    /// zoom, ALWAYS shown (`Zoom: 0`, `Zoom: +14`, `Zoom: -3`). The ONE formatter
    /// behind both the live Status_Bar and the snapshot status band. Reuses the
    /// `ff-zoom` indicator for the sign, adding the zero case it hides.
    ///
    /// Validates: Requirement 7.16, 7.16b; screen-snapshot-scrm Requirement 30.10
    pub(crate) fn format_zoom_label(&self) -> String {
        use ff_zoom::ZoomIndicatorState;
        match ZoomIndicatorState::from_offset(self.zoom.offset()) {
            ZoomIndicatorState::Visible { text, .. } => text,
            ZoomIndicatorState::Hidden => "Zoom: 0".to_string(),
        }
    }

    /// Format the logoff message as `Logoff at HH:MM -- session duration: Xm Ys`.
    ///
    /// Validates: Requirement 20.2
    pub(crate) fn format_logoff_message(&self) -> String {
        let now = chrono::Local::now();
        let duration = now.signed_duration_since(self.session_start);
        let total_secs = duration.num_seconds().max(0) as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!(
            "Logoff at {} -- session duration: {}m {}s",
            now.format("%H:%M"),
            mins,
            secs
        )
    }

    /// The Owning_Environment NAME of the active tab (CR-CH-053 Task 19,
    /// Req 15.4): the file-system Command Environment FFEDIT ADDRESSes a
    /// store-affecting verb (SAVE) to for the current resource. Defaults to the
    /// host FS environment for a host-path tab (Req 15.3/15.6), so native SAVE is
    /// unchanged.
    // core-001 strip: `active_owning_environment` fed the parked FFEDIT SAVE
    // addressing; retained (TabState keeps `owning_environment`) for re-entry.
    #[allow(dead_code)]
    pub(crate) fn active_owning_environment(&self) -> String {
        self.tabs.active_tab().owning_environment.clone()
    }

    // core-001 strip: the `provider_registry()` accessor (the live ff-vfs
    // Provider_Registry seam) is parked with the VFS provider domain.

    /// Open a file into a new tab AND apply the resulting Kind's profile
    /// (CR-NR-090 B.3). The single shell open-file seam; wraps
    /// `TabManager::open_file`. Binds the opened tab to the host FS
    /// Owning_Environment (CR-CH-053 Task 19, Req 15.3 default).
    pub(crate) fn shell_open_file(&mut self, path: &str) -> Result<(), String> {
        self.shell_open_file_with_env(path, None)
    }

    /// Open a file into a new tab, binding its Owning_Environment to
    /// `owning_env` (CR-CH-053 Task 19, Req 15.2); `None` binds the host FS
    /// environment (`DEFAULT_OWNING_ENVIRONMENT`, Req 15.3), so a plain host-path
    /// open is behaviour-preserving. FFEDIT later reads this binding to choose
    /// where to ADDRESS a store-affecting verb (Req 15.4).
    pub(crate) fn shell_open_file_with_env(
        &mut self,
        path: &str,
        owning_env: Option<&str>,
    ) -> Result<(), String> {
        let result = self.tabs.open_file(path, &self.runtime);
        if result.is_ok() {
            if let Some(env) = owning_env {
                // Capture the originating environment on the just-opened (now
                // active) tab. Default (None) leaves the constructor's host FS
                // binding untouched.
                self.tabs.active_tab_mut().owning_environment = env.to_string();
            }
            self.apply_kind_profile_to_active();
        }
        result
    }

    // core-001 strip: `apply_pending_open_identity` (binding a MAINFRAME tab's
    // real dataset identity + record format via `ff_vfs::RecordFormatKind`) is
    // parked with the catalog/VFS domains. Host-path opens never used it.

    /// Create a new untitled buffer AND apply the resulting Kind's profile
    /// (CR-NR-090 B.3). The single shell new-untitled seam.
    pub(crate) fn shell_new_untitled(&mut self) {
        self.tabs.new_untitled_tab(&self.runtime);
        self.apply_kind_profile_to_active();
    }
}
