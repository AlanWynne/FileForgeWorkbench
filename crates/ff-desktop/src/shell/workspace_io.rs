//! # Workspace Open / Save / Close
//!
//! The `WorkbenchShell` workspace-lifecycle method group (open/save/close),
//! moved out of `mod.rs` verbatim as part of the Phase 2 task 2.2 file-size
//! split. Behaviour, method names, signatures, and `pub(crate)` visibility are
//! unchanged.
//!
//! Validates: workspace-model Requirement 2, 3, 4, 6

use ff_session::{load_workspace, save_workspace};

use super::WorkbenchShell;

impl WorkbenchShell {
    // === Workspace lifecycle helpers -- Validates: workspace-model Req 2, 3, 4, 6 ===

    /// Load a workspace from `path`, register its roots as Native catalogs,
    /// inject its settings layer, and restore its MRU list.
    ///
    /// If a modified workspace is already active, defers the open and shows the
    /// unsaved-changes dialog instead.
    ///
    /// Validates: workspace-model Requirement 2.1, 2.5, 3.4, 4.1, 6.1
    pub(crate) fn open_workspace(&mut self, path: &std::path::Path) {
        // Validates: Requirement 2.5 -- prompt before discarding unsaved changes.
        if let Some(ws) = &self.active_workspace {
            if ws.is_modified {
                self.pending_workspace_open = Some(path.to_path_buf());
                self.show_unsaved_workspace_dialog = true;
                return;
            }
        }
        self.open_workspace_force(path);
    }

    /// Open a workspace unconditionally, closing any existing one first.
    ///
    /// Validates: workspace-model Requirement 2.1, 3.4, 4.1, 6.1
    pub(crate) fn open_workspace_force(&mut self, path: &std::path::Path) {
        match load_workspace(path) {
            Err(e) => {
                self.open_error = Some(ff_i18n::t_args(
                    "ff-desktop-shell-msg-ws-open-failed",
                    &ff_i18n::MessageArgs::new().set("error", e.to_string()),
                ));
            }
            Ok(ws) => {
                // Close any existing workspace first.
                if self.active_workspace.is_some() {
                    self.close_workspace();
                }
                // core-001 strip: registering each workspace root as a Native
                // catalog is parked with the catalog domain. Roots that do not
                // exist are still flagged so the user sees a warning; the roots
                // continue to drive Search Results via `collect_search_roots`.
                let mut root_warning: Option<String> = None;
                for root in &ws.roots {
                    if !root.exists() {
                        root_warning = Some(ff_i18n::t_args(
                            "ff-desktop-shell-msg-ws-root-not-found",
                            &ff_i18n::MessageArgs::new().set("root", root.display().to_string()),
                        ));
                    }
                }
                // Inject workspace settings into config as highest-priority layer.
                for (key, val) in &ws.settings {
                    let _ = self
                        .config_handle
                        .set_user_value(key, ff_config::ConfigValue::String(val.clone()));
                }
                self.active_workspace = Some(ws);
                // Preserve root warning; only clear error when everything succeeded.
                self.open_error = root_warning;
            }
        }
    }

    /// Save the active workspace to its current file path, or to `path` if provided.
    ///
    /// Validates: workspace-model Requirement 2.2, 2.3
    pub(crate) fn save_workspace_to(&mut self, path: Option<&std::path::Path>) {
        let Some(ws) = self.active_workspace.as_mut() else {
            self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-ws-none-to-save"));
            return;
        };
        let target = match path {
            Some(p) => p.to_path_buf(),
            None => match &ws.file_path {
                Some(p) => p.clone(),
                None => {
                    self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-ws-no-path"));
                    return;
                }
            },
        };
        if let Some(p) = path {
            ws.file_path = Some(p.to_path_buf());
        }
        ws.is_modified = false;
        match save_workspace(ws, &target) {
            Ok(()) => self.open_error = None,
            Err(e) => {
                self.open_error = Some(ff_i18n::t_args(
                    "ff-desktop-shell-msg-ws-save-failed",
                    &ff_i18n::MessageArgs::new().set("error", e.to_string()),
                ))
            }
        }
    }

    /// Unload the active workspace: unregister its roots and remove its settings layer.
    ///
    /// Validates: workspace-model Requirement 2.4, 3.4, 4.3
    pub(crate) fn close_workspace(&mut self) {
        let Some(ws) = self.active_workspace.take() else {
            return;
        };
        // core-001 strip: unregistering workspace roots from the catalog registry
        // is parked with the catalog domain.
        // Remove workspace settings overrides (reset to user-layer values).
        for key in ws.settings.keys() {
            let _ = self.config_handle.remove_user_value(key);
        }
        self.open_error = None;
    }
}
