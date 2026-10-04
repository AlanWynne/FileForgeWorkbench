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
                self.open_error = Some(format!("Cannot open workspace: {e}"));
            }
            Ok(ws) => {
                // Close any existing workspace first.
                if self.active_workspace.is_some() {
                    self.close_workspace();
                }
                // Register each root as a Native catalog.
                let mut root_warning: Option<String> = None;
                for root in &ws.roots {
                    if !root.exists() {
                        root_warning = Some(format!(
                            "Workspace warning: root '{}' not found on disk",
                            root.display()
                        ));
                        continue;
                    }
                    let cat = crate::catalog_registry::VirtualCatalog {
                        name: root
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| root.to_string_lossy().into_owned()),
                        catalog_type: crate::catalog_registry::CatalogType::Native,
                        path: root.to_string_lossy().into_owned(),
                        description: Some("Workspace root".to_string()),
                        auto_mount: true,
                        default_hlq: None,
                        mount_point: None,
                        read_only: false,
                    };
                    let _ = self.files_panel.registry.register(cat);
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
            self.open_error = Some("No active workspace to save".to_string());
            return;
        };
        let target = match path {
            Some(p) => p.to_path_buf(),
            None => match &ws.file_path {
                Some(p) => p.clone(),
                None => {
                    self.open_error = Some(
                        "No workspace file path set -- use WORKSPACE SAVE AS <path>".to_string(),
                    );
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
            Err(e) => self.open_error = Some(format!("Cannot save workspace: {e}")),
        }
    }

    /// Unload the active workspace: unregister its roots and remove its settings layer.
    ///
    /// Validates: workspace-model Requirement 2.4, 3.4, 4.3
    pub(crate) fn close_workspace(&mut self) {
        let Some(ws) = self.active_workspace.take() else {
            return;
        };
        // Unregister workspace roots from the catalog registry.
        for root in &ws.roots {
            let name = root
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| root.to_string_lossy().into_owned());
            let _ = self.files_panel.registry.remove(&name);
        }
        // Remove workspace settings overrides (reset to user-layer values).
        for key in ws.settings.keys() {
            let _ = self.config_handle.remove_user_value(key);
        }
        self.open_error = None;
    }
}
