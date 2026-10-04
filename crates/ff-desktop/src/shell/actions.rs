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
                self.open_error = Some("Lua execution not yet available".to_string());
            }
            A::Delete(path) => {
                if let Err(e) = std::fs::remove_file(&path) {
                    self.open_error = Some(format!("Delete failed: {e}"));
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
                            self.open_error = Some(format!(
                                "Replaced {} occurrence(s) in {} file(s)",
                                summary.replacements, summary.files_modified
                            ));
                        }
                        Err(e) => {
                            self.open_error = Some(format!("Replace failed: {e}"));
                        }
                    }
                }
            }
            O::Cancel | O::None => {}
        }
    }

    // === Session lifecycle helpers -- Validates: Requirement 20.1, 20.2 ===

    /// Format the session start time as `Started: HH:MM`.
    ///
    /// Validates: Requirement 20.1
    pub(crate) fn format_session_start(&self) -> String {
        format!("Started: {}", self.session_start.format("%H:%M"))
    }

    /// The Active_Profile label for the Status_Bar / Title_Line (CR-NR-081,
    /// startup-and-session Requirement 22.8). Shows the running Application_Profile
    /// name, or `Profile: default` when no `--profile` was given.
    ///
    /// Validates: startup-and-session Requirement 22.8
    pub(crate) fn active_profile_label(&self) -> String {
        match ff_session::active_profile() {
            Some(name) => format!("Profile: {name}"),
            None => "Profile: default".to_string(),
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

    /// Open a file into a new tab AND apply the resulting Kind's profile
    /// (CR-NR-090 B.3). The single shell open-file seam; wraps
    /// `TabManager::open_file`.
    pub(crate) fn shell_open_file(&mut self, path: &str) -> Result<(), String> {
        let result = self.tabs.open_file(path, &self.runtime);
        if result.is_ok() {
            self.apply_kind_profile_to_active();
        }
        result
    }

    /// Create a new untitled buffer AND apply the resulting Kind's profile
    /// (CR-NR-090 B.3). The single shell new-untitled seam.
    pub(crate) fn shell_new_untitled(&mut self) {
        self.tabs.new_untitled_tab(&self.runtime);
        self.apply_kind_profile_to_active();
    }
}
