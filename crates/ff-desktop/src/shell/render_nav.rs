//! # Shell Panel Rendering -- File Explorer (NavModel) Context
//!
//! The NavModel-backed File Explorer render (seed / body / apply-effects), its
//! create/delete/rename dialogs, paste/edit providers, and the catalog/local
//! node expansion helpers. Split out of `render.rs` (TASK 2.2, pure code
//! movement, no behaviour change).

use eframe::egui;

use super::render_body::open_mainframe_dsn;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the NavModel-backed File Explorer as the primary File Explorer
    /// Context content (CR-NR-060 Slice A). Seeds the Local Files root from the
    /// local/POSIX provider on first display, renders the modern tree in the
    /// CentralPanel, and applies the returned effects (expand -> async VFS list;
    /// collapse; open -> file.open dispatch, OS default app, or dataset resolve;
    /// copy path; reveal). This is the default explorer; the legacy inline tree
    /// remains available as a fallback when `use_legacy_explorer` is set.
    ///
    /// Validates: Requirement 24.1, 24.3, 24.5, 24.7, 24.9
    pub(super) fn render_nav_explorer(&mut self, ctx: &egui::Context) {
        // CR-NR-093 B071: the File Explorer is now rendered in THREE parts --
        // seed (state), body (into a Ui), apply (effects + dialogs) -- so it can
        // render either as this full-window CentralPanel (unsplit) OR inside a
        // split region's Ui (via render_active_tab_body). This ctx-level path is
        // the unsplit case; behaviour is unchanged.
        self.nav_explorer_seed();
        let mut effects = Vec::new();
        egui::CentralPanel::default().show(ctx, |ui| {
            effects = self.render_nav_explorer_body(ui);
        });
        self.apply_nav_explorer_effects(ctx, effects);
    }

    /// Seed the modern File Explorer's NavModel on first display (CR-NR-093 B071
    /// split of `render_nav_explorer`): associate the Local Files root URI and
    /// load its children via the provider, and populate the Catalogs root from
    /// the registry. Idempotent -- a no-op once `children_loaded` is set.
    pub(super) fn nav_explorer_seed(&mut self) {
        use crate::nav_model::list_via_provider;

        // Seed the Local Files root on first display: associate its URI and load
        // its immediate children via the provider (never std::fs).
        let local_root = self.nav_model.tree.root_categories[0];
        let needs_seed = self
            .nav_model
            .tree
            .get_node(local_root)
            .map(|n| !n.children_loaded)
            .unwrap_or(false);
        if needs_seed {
            let root_dir = dirs::home_dir()
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            // CR-CH-059 RC.A.4: the single `posix` provider is the infallible
            // `ff_vfs::PosixNativeProvider` (re-exported as PosixProvider). It
            // does not spawn a watcher at construction, so no runtime guard is
            // needed here.
            let provider = crate::posix_provider::PosixProvider::new(root_dir, true);
            let root_uri = ff_vfs::ResourceUri::new("posix", "/");
            self.nav_model.set_uri(local_root, root_uri.clone());
            match list_via_provider(&self.runtime, &provider, root_uri.path()) {
                Ok(entries) => self.nav_model.apply_listing(local_root, "posix", &entries),
                Err(e) => self.nav_model.apply_load_error(local_root, e),
            }

            // Seed the Catalogs root generically: one CatalogRoot node per
            // registered catalog (Slice A -- no mainframe qualifier/dataset
            // duality; that is Slice B). Requirement 24.8.
            let catalogs_root = self.nav_model.tree.root_categories[1];
            for cat in self.files_panel.registry.list() {
                let uri = ff_vfs::ResourceUri::new("catalog", format!("/{}", cat.name));
                self.nav_model.add_child(
                    catalogs_root,
                    cat.name.clone(),
                    ff_file_tree::NodeType::CatalogRoot,
                    uri,
                );
            }
            if let Some(n) = self.nav_model.tree.get_node_mut(catalogs_root) {
                n.children_loaded = true;
            }
        }
    }

    /// Render the modern File Explorer tree into `ui` and return the interaction
    /// effects to apply (CR-NR-093 B071 split of `render_nav_explorer`). Used by
    /// both the full-window (unsplit) path and a split region's Ui.
    pub(super) fn render_nav_explorer_body(
        &mut self,
        ui: &mut egui::Ui,
    ) -> Vec<crate::explorer_view::ExplorerEffect> {
        use crate::explorer_view::{keyboard_effects, render_tree};
        let mut effects = Vec::new();
        ui.label(egui::RichText::new("File Explorer").monospace().strong());
        ui.separator();
        // Keyboard navigation (Req 8/20) when the explorer is hovered/
        // focused, then mouse interactions from the tree.
        if ui.rect_contains_pointer(ui.max_rect()) {
            effects.extend(keyboard_effects(
                ui,
                &self.nav_model,
                &mut self.nav_ui.nav_selection,
            ));
        }
        effects.extend(render_tree(
            ui,
            &self.nav_model,
            &mut self.nav_ui.nav_selection,
            &self.palette,
        ));
        effects
    }

    /// Apply the File Explorer interaction effects and render its modal dialogs
    /// (CR-NR-093 B071 split of `render_nav_explorer`).
    pub(super) fn apply_nav_explorer_effects(
        &mut self,
        ctx: &egui::Context,
        effects: Vec<crate::explorer_view::ExplorerEffect>,
    ) {
        use crate::explorer_view::{resolve_open, ExplorerEffect, OpenTarget};
        // Apply interaction effects outside the render borrow.
        for eff in effects {
            match eff {
                ExplorerEffect::Expand(id) => {
                    if let Some(uri) = self.nav_model.uri_of(id).cloned() {
                        match uri.scheme() {
                            // Catalogs root child: resolve the catalog by name and
                            // list its backing directory generically (Req 24.8 --
                            // no mainframe qualifier/dataset duality; that is
                            // Slice B). URI is vfs://catalog/{name}.
                            "catalog" => self.expand_catalog_node(id, &uri),
                            // Local Files (posix/local): root-jailed provider at the
                            // local root; URI path is provider-relative (Req 24.3).
                            _ => self.expand_local_node(id, &uri),
                        }
                    } else {
                        self.nav_model.tree.toggle_expand(id);
                    }
                }
                ExplorerEffect::Collapse(id) => self.nav_model.tree.toggle_expand(id),
                ExplorerEffect::Open(id) => match resolve_open(&self.nav_model, id) {
                    OpenTarget::Editor(uri) => {
                        // Resolve the provider-relative navigator URI to a real
                        // absolute host path before dispatching file.open, which
                        // reads through a default-rooted provider (B047).
                        match self.nav_open_path(&uri) {
                            Ok(host_path) => {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", host_path.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                            Err(e) => self.open_error = Some(e),
                        }
                    }
                    OpenTarget::External(uri) => {
                        // Launch the OS default app with the real host path.
                        match self.nav_open_path(&uri) {
                            Ok(host_path) => crate::context_menu::launch_default_app(&host_path),
                            Err(e) => self.open_error = Some(e),
                        }
                    }
                    OpenTarget::Dataset { catalog, dsn } => {
                        // Resolve the DSN to its physical file (create if
                        // missing) and open it in the editor -- same path the
                        // legacy panel uses. Req 16.1/16.3.
                        match open_mainframe_dsn(&self.files_panel.registry, &catalog, &dsn) {
                            Ok(path_str) => {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", path_str.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                            Err(e) => self.open_error = Some(e),
                        }
                    }
                    OpenTarget::None => {}
                },
                ExplorerEffect::CopyPath(id) => {
                    // Req 16 Copy Full Path: copy the node's resource path.
                    if let Some(uri) = self.nav_model.uri_of(id) {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text(uri.path());
                        }
                    }
                }
                ExplorerEffect::CopySelection(text) => {
                    // Req 19.5/19.6: copy the indented text tree of the current
                    // multi-selection to the OS clipboard.
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&text);
                    }
                }
                ExplorerEffect::MarkCopy(id) => {
                    // Req 21.1: mark the selection (or this node) for a file copy.
                    // Record the source URIs of the selected local/POSIX files.
                    let ids: Vec<ff_file_tree::NodeId> =
                        if self.nav_ui.nav_selection.selected.is_empty() {
                            vec![id]
                        } else {
                            self.nav_ui.nav_selection.selected.iter().copied().collect()
                        };
                    self.nav_ui.nav_file_clipboard = ids
                        .into_iter()
                        .filter_map(|n| self.nav_model.uri_of(n).cloned())
                        .filter(|u| u.scheme() == "posix" || u.scheme() == "local")
                        .collect();
                }
                ExplorerEffect::Paste(anchor) => {
                    self.apply_nav_paste(anchor);
                }
                ExplorerEffect::Rename(id) => {
                    // Open the rename dialog seeded with the node's current label
                    // (Req 16 Rename). The rename is applied on dialog confirm.
                    if let Some(label) = self.nav_model.tree.get_node(id).map(|n| n.label.clone()) {
                        self.nav_ui.nav_rename = Some((id, label));
                    }
                }
                ExplorerEffect::Delete(id) => {
                    // Open the delete-confirmation dialog (Req 16 Delete). The
                    // delete is applied only on explicit confirm.
                    if let Some(label) = self.nav_model.tree.get_node(id).map(|n| n.label.clone()) {
                        self.nav_ui.nav_delete = Some((id, label));
                    }
                }
                ExplorerEffect::NewChild { anchor, is_dir } => {
                    // Resolve the containing directory: the anchor itself if it
                    // is a directory, else the anchor's parent (Req 16 New).
                    if let Some(node) = self.nav_model.tree.get_node(anchor) {
                        let parent_dir = if node.node_type.is_expandable() {
                            anchor
                        } else {
                            node.parent
                        };
                        self.nav_ui.nav_new = Some((parent_dir, is_dir, String::new()));
                    }
                }
                ExplorerEffect::Reveal(id) => {
                    // Req 16 Reveal in Explorer: open the OS file manager at the
                    // node. Only local/POSIX nodes map to a real host path.
                    if let Some(uri) = self.nav_model.uri_of(id).cloned() {
                        if uri.scheme() == "posix" || uri.scheme() == "local" {
                            let root_dir = dirs::home_dir()
                                .or_else(|| std::env::current_dir().ok())
                                .unwrap_or_else(|| std::path::PathBuf::from("."));
                            let rel = uri.path().trim_start_matches('/');
                            let full = root_dir.join(rel);
                            crate::context_menu::reveal_in_explorer(&full.to_string_lossy());
                        }
                    }
                }
                ExplorerEffect::None => {}
            }
        }

        // Rename + Delete + New dialogs (Req 16) -- modal, applied on confirm.
        self.render_nav_rename_dialog(ctx);
        self.render_nav_delete_dialog(ctx);
        self.render_nav_new_dialog(ctx);
    }
}
