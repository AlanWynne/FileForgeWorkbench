//! # Shell Panel Rendering -- File Explorer dialogs and node operations
//!
//! The File Explorer create/delete/rename dialogs and their apply handlers,
//! paste, the edit provider, path resolution, and the local/catalog node
//! expansion helpers. Split out of `render.rs` / `render_nav.rs` (TASK 2.2,
//! pure code movement, no behaviour change).

use eframe::egui;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the modern-explorer new-file / new-folder dialog and create the
    /// child on confirm (local/POSIX only), then refresh the parent listing.
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 16 New File / New Folder)
    pub(super) fn render_nav_new_dialog(&mut self, ctx: &egui::Context) {
        let Some((parent, is_dir, mut buffer)) = self.nav_ui.nav_new.take() else {
            return;
        };
        let title = if is_dir { "New Folder" } else { "New File" };
        let mut still_open = true;
        let mut confirm = false;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("Name:");
                let resp = ui.text_edit_singleline(&mut buffer);
                resp.request_focus();
                let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() || enter {
                        confirm = true;
                    }
                    if ui.button("Cancel").clicked() {
                        still_open = false;
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    still_open = false;
                }
            });

        if confirm {
            self.apply_nav_new(parent, is_dir, buffer.trim());
            return;
        }
        if still_open {
            self.nav_ui.nav_new = Some((parent, is_dir, buffer));
        }
    }

    /// Create a new child (`is_dir` chooses folder vs file) named `name` under
    /// the `parent` directory node via a writable provider, then refresh the
    /// parent listing. Local/POSIX only.
    ///
    /// Validates: Requirement 24.2, 24.3 (Req 16 New File / New Folder)
    fn apply_nav_new(&mut self, parent: ff_file_tree::NodeId, is_dir: bool, name: &str) {
        use crate::nav_model::child_uri;
        use ff_vfs::{CreateOptions, VfsProvider};
        if name.is_empty() {
            return;
        }
        let Some(parent_uri) = self.nav_model.uri_of(parent).cloned() else {
            return;
        };
        let child = child_uri(&parent_uri, name);
        let (provider, child_path) = match self.nav_edit_provider(&child) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("New file/folder: {e}"));
                return;
            }
        };
        let opts = CreateOptions {
            create_parents: false,
            is_directory: is_dir,
        };
        let result = self.runtime.block_on(provider.create(&child_path, opts));
        match result {
            Ok(()) => {
                // Ensure the parent is expanded, then refresh its listing.
                if let Some(n) = self.nav_model.tree.get_node_mut(parent) {
                    n.expanded = true;
                }
                self.refresh_nav_parent(parent);
            }
            Err(e) => self.open_error = Some(format!("Create failed: {e}")),
        }
    }

    /// Paste the file clipboard into the directory resolved from `anchor` (the
    /// anchor if a directory, else its parent). Each source file is read and
    /// written into the target directory via a writable provider, then the
    /// target listing is refreshed. Local/POSIX only (Req 21.2/21.3).
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 21)
    pub(super) fn apply_nav_paste(&mut self, anchor: ff_file_tree::NodeId) {
        use crate::explorer_view::paste_target;
        use crate::nav_model::child_uri;
        use ff_vfs::{CreateOptions, VfsProvider};
        if self.nav_ui.nav_file_clipboard.is_empty() {
            return;
        }
        let Some(target) = paste_target(&self.nav_model, anchor) else {
            return;
        };
        let Some(target_uri) = self.nav_model.uri_of(target).cloned() else {
            return;
        };
        // Resolve the writable target provider (Local Files or a POSIX/Native
        // catalog rooted at its backing path); rejects Mainframe/read-only.
        let (target_provider, _target_path) = match self.nav_edit_provider(&target_uri) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("Paste: {e}"));
                return;
            }
        };
        let sources = self.nav_ui.nav_file_clipboard.clone();
        let mut errors = 0;
        for src in &sources {
            // Derive the destination file name from the source path's last segment.
            let name = src.path().rsplit('/').next().unwrap_or("");
            if name.is_empty() {
                continue;
            }
            // Destination URI inherits the target scheme; resolve both source and
            // destination to their (possibly different) providers + relative paths.
            let dest = child_uri(&target_uri, name);
            if dest.path() == src.path() && dest.scheme() == src.scheme() {
                continue; // no-op copy onto itself
            }
            let (src_provider, src_path) = match self.nav_edit_provider(src) {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            let (_dp, dest_path) = match self.nav_edit_provider(&dest) {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            let copy = self.runtime.block_on(async {
                let bytes = src_provider.read(&src_path).await?;
                target_provider
                    .create(
                        &dest_path,
                        CreateOptions {
                            create_parents: false,
                            is_directory: false,
                        },
                    )
                    .await?;
                target_provider.write(&dest_path, &bytes).await
            });
            if copy.is_err() {
                errors += 1;
            }
        }
        if errors > 0 {
            self.open_error = Some(format!("Paste: {errors} file(s) could not be copied"));
        }
        // Refresh the target directory listing so the pasted files appear.
        if let Some(n) = self.nav_model.tree.get_node_mut(target) {
            n.expanded = true;
        }
        self.refresh_nav_parent(target);
    }

    /// Render the modern-explorer delete-confirmation dialog and apply the delete
    /// on confirm. Local/POSIX nodes only; directories delete recursively.
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 16 Delete)
    pub(super) fn render_nav_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some((id, label)) = self.nav_ui.nav_delete.take() else {
            return;
        };
        let mut decision: Option<bool> = None; // Some(true)=delete, Some(false)=cancel
        egui::Window::new("Delete")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("Delete '{label}'? This cannot be undone."));
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        decision = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        decision = Some(false);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    decision = Some(false);
                }
            });

        match decision {
            Some(true) => self.apply_nav_delete(id),
            Some(false) => {} // cancelled -- dialog already taken (closed)
            None => self.nav_ui.nav_delete = Some((id, label)), // keep open
        }
    }

    /// Apply a delete of node `id` (local/POSIX only) via a writable provider,
    /// then refresh the parent listing. Directories are deleted recursively.
    ///
    /// Validates: Requirement 24.2, 24.3 (Req 16 Delete)
    fn apply_nav_delete(&mut self, id: ff_file_tree::NodeId) {
        use ff_vfs::{DeleteOptions, VfsProvider};
        let Some(uri) = self.nav_model.uri_of(id).cloned() else {
            return;
        };
        let recursive = self
            .nav_model
            .tree
            .get_node(id)
            .map(|n| n.node_type.is_expandable())
            .unwrap_or(false);
        let parent = self.nav_model.tree.get_node(id).map(|n| n.parent);
        let (provider, path) = match self.nav_edit_provider(&uri) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("Delete: {e}"));
                return;
            }
        };
        let result = self
            .runtime
            .block_on(provider.delete(&path, DeleteOptions { recursive }));
        match result {
            Ok(()) => {
                if let Some(parent) = parent {
                    self.refresh_nav_parent(parent);
                }
            }
            Err(e) => self.open_error = Some(format!("Delete failed: {e}")),
        }
    }

    /// Render the modern-explorer rename dialog when a rename is in progress and
    /// apply the rename on confirm. Local/POSIX nodes are renamed via a writable
    /// provider then the parent listing is refreshed. Non-local schemes (catalog
    /// roots, datasets) are not renamable here in Slice A.
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 16 Rename)
    pub(super) fn render_nav_rename_dialog(&mut self, ctx: &egui::Context) {
        let Some((id, mut buffer)) = self.nav_ui.nav_rename.take() else {
            return;
        };
        let mut still_open = true;
        let mut confirm = false;
        egui::Window::new("Rename")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("New name:");
                let resp = ui.text_edit_singleline(&mut buffer);
                resp.request_focus();
                let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    if ui.button("Rename").clicked() || enter {
                        confirm = true;
                    }
                    if ui.button("Cancel").clicked() {
                        still_open = false;
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    still_open = false;
                }
            });

        if confirm {
            self.apply_nav_rename(id, buffer.trim());
            return; // dialog closed
        }
        if still_open {
            self.nav_ui.nav_rename = Some((id, buffer));
        }
    }

    /// Apply a rename of node `id` to `new_name` (local/POSIX only), then refresh
    /// the parent listing so the model reflects the new name.
    ///
    /// Validates: Requirement 24.2, 24.3 (Req 16 Rename)
    fn apply_nav_rename(&mut self, id: ff_file_tree::NodeId, new_name: &str) {
        use crate::nav_model::rename_uri;
        use ff_vfs::VfsProvider;
        if new_name.is_empty() {
            return;
        }
        let Some(uri) = self.nav_model.uri_of(id).cloned() else {
            return;
        };
        // `rename_uri` preserves scheme + parent, so the new URI resolves to the
        // same provider; take its provider-relative path for the destination.
        let new_uri = rename_uri(&uri, new_name);
        let (provider, old_path) = match self.nav_edit_provider(&uri) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("Rename: {e}"));
                return;
            }
        };
        let new_path = match self.nav_edit_provider(&new_uri) {
            Ok((_, p)) => p,
            Err(e) => {
                self.open_error = Some(format!("Rename: {e}"));
                return;
            }
        };
        let result = self.runtime.block_on(provider.rename(&old_path, &new_path));
        match result {
            Ok(()) => {
                let parent = self.nav_model.tree.get_node(id).map(|n| n.parent);
                if let Some(parent) = parent {
                    self.refresh_nav_parent(parent);
                }
            }
            Err(e) => self.open_error = Some(format!("Rename failed: {e}")),
        }
    }
}
