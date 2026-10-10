//! # Shell Panel Rendering -- File Explorer node expansion and path resolution
//!
//! The edit-provider resolver, path resolution (`nav_open_path`), parent
//! refresh, and the local / catalog node expansion and listing helpers. Split
//! out of `render.rs` / `render_nav_ops.rs` (TASK 2.2, pure code movement, no
//! behaviour change).

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Resolve a node URI to a writable provider plus the provider-relative path
    /// for an edit operation (delete/rename/new/paste). Supports the Local Files
    /// subtree (`posix`/`local`, rooted at home) and POSIX/Native catalog
    /// subtrees (`catalog`, rooted at the catalog's backing path). Mainframe
    /// dataset nodes (`dataset`) resolve through their catalog's backing path too
    /// (RC.B.8 (c) lifted the former edit-gate); read-only catalogs are still
    /// rejected with a message. (B042.)
    ///
    /// Validates: Requirement 24.6 (file-tree-panel Req 16.10-16.13)
    pub(super) fn nav_edit_provider(
        &self,
        uri: &ff_vfs::ResourceUri,
    ) -> Result<(crate::posix_provider::PosixProvider, String), String> {
        use crate::catalog_registry::CatalogType;
        use crate::nav_model::split_catalog_uri_path;
        let (root_dir, rel_path) = match uri.scheme() {
            "posix" | "local" => {
                let home = dirs::home_dir()
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                (home, uri.path().to_string())
            }
            "catalog" => {
                let (name, sub_path) = split_catalog_uri_path(uri.path());
                let cat = self
                    .files_panel
                    .registry
                    .get_by_name(name)
                    .ok_or_else(|| format!("Catalog '{name}' not found"))?;
                // RC.B.8 (c): Mainframe dataset editing is now wired (routes to
                // the MAINFRAME backend via the record-store seam); the former
                // "available in a later update" edit-gate is lifted. PO member
                // browsing stays gated below (a separate slice).
                match cat.catalog_type {
                    CatalogType::Posix | CatalogType::Native | CatalogType::Mainframe => {}
                }
                if cat.read_only {
                    return Err(format!("Catalog '{name}' is read-only."));
                }
                (std::path::PathBuf::from(&cat.path), sub_path)
            }
            "dataset" => {
                // RC.B.8 (c): dataset editing is wired; the edit-gate is lifted.
                // A `dataset`-scheme node resolves through its catalog like the
                // `catalog`-scheme Mainframe branch above. The owning open flow
                // (render_body_arms / render_nav) binds the MAINFRAME backend; the
                // editable surface here resolves the host-path provider for the
                // read. Member browsing (PO) stays a separate slice.
                let (name, sub_path) = split_catalog_uri_path(uri.path());
                let cat = self
                    .files_panel
                    .registry
                    .get_by_name(name)
                    .ok_or_else(|| format!("Catalog '{name}' not found"))?;
                if cat.read_only {
                    return Err(format!("Catalog '{name}' is read-only."));
                }
                (std::path::PathBuf::from(&cat.path), sub_path)
            }
            other => return Err(format!("Editing is not supported for '{other}' resources.")),
        };
        // CR-CH-059 RC.A.4: single infallible `ff_vfs::PosixNativeProvider`.
        let provider = crate::posix_provider::PosixProvider::new(root_dir, false);
        Ok((provider, rel_path))
    }

    /// Resolve a navigator node URI to a real absolute host filesystem path so
    /// it can be opened via `file.open` (which reads through a default-rooted
    /// `LocalFsProvider`, i.e. it expects a real path, not a provider-relative
    /// one).
    ///
    /// The modern explorer seeds Local Files through a `posix` provider jailed
    /// at the home directory, so a node's `uri.path()` (e.g.
    /// `/OneDrive - Standard Bank/Clipbook.md`) is relative to that jail root,
    /// NOT an absolute path. Passing it straight to `file.open` produced
    /// "resource not found" (B047). This joins the correct root (home for
    /// posix/local; the catalog `path` for catalog subtrees) with the
    /// provider-relative path, using the host path separator.
    ///
    /// Validates: file-tree-panel Requirement 24.9 (open resolves to the real
    /// file); B047.
    pub(super) fn nav_open_path(&self, uri: &ff_vfs::ResourceUri) -> Result<String, String> {
        use crate::nav_model::split_catalog_uri_path;
        let (root_dir, rel_path) = match uri.scheme() {
            "posix" | "local" => {
                let home = dirs::home_dir()
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                (home, uri.path().to_string())
            }
            "catalog" => {
                let (name, sub_path) = split_catalog_uri_path(uri.path());
                let cat = self
                    .files_panel
                    .registry
                    .get_by_name(name)
                    .ok_or_else(|| format!("Catalog '{name}' not found"))?;
                (std::path::PathBuf::from(&cat.path), sub_path)
            }
            other => {
                return Err(format!(
                    "Cannot resolve a host path for '{other}' resources."
                ))
            }
        };
        // Join the root with each forward-slash segment of the provider-relative
        // path, using the host separator. Reject `..` traversal out of the jail.
        let mut resolved = root_dir;
        for segment in rel_path.trim_start_matches('/').split('/') {
            match segment {
                "" | "." => {}
                ".." => return Err("Invalid path (parent traversal)".to_string()),
                s => resolved.push(s),
            }
        }
        Ok(resolved.to_string_lossy().into_owned())
    }

    /// Refresh a parent node's listing after an edit, dispatching on its URI
    /// scheme (catalog subtrees re-list via the catalog path). (B042.)
    pub(super) fn refresh_nav_parent(&mut self, parent: ff_file_tree::NodeId) {
        if let Some(puri) = self.nav_model.uri_of(parent).cloned() {
            if puri.scheme() == "catalog" {
                self.expand_catalog_node(parent, &puri);
            } else {
                self.expand_local_node(parent, &puri);
            }
        }
        self.nav_model.prune_uris();
    }

    /// Expand a Local Files (posix/local) node: list its provider-relative path
    /// through a root-jailed provider rooted at the local root.
    ///
    /// Validates: Requirement 24.3, 24.5
    pub(super) fn expand_local_node(
        &mut self,
        id: ff_file_tree::NodeId,
        uri: &ff_vfs::ResourceUri,
    ) {
        use crate::nav_model::list_via_provider;
        let root_dir = dirs::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        // CR-CH-059 RC.A.4: single infallible `ff_vfs::PosixNativeProvider`.
        let provider = crate::posix_provider::PosixProvider::new(root_dir, true);
        match list_via_provider(&self.runtime, &provider, uri.path()) {
            Ok(entries) => self.nav_model.apply_listing(id, "posix", &entries),
            Err(e) => self.nav_model.apply_load_error(id, e),
        }
    }

    /// Expand a Catalogs root child generically (Req 24.8): resolve the catalog
    /// by name (URI `vfs://catalog/{name}`) and list its backing directory
    /// through a provider rooted at the catalog `path`. No mainframe qualifier/
    /// dataset duality is applied here -- that is Slice B. If the catalog is
    /// unknown or its directory cannot be listed, an error node is shown.
    ///
    /// Validates: Requirement 24.8, 24.5
    pub(super) fn expand_catalog_node(
        &mut self,
        id: ff_file_tree::NodeId,
        uri: &ff_vfs::ResourceUri,
    ) {
        use crate::catalog_registry::CatalogType;
        use crate::nav_model::split_catalog_uri_path;
        // URI path is "/{name}[/{subpath...}]": the first segment is the catalog
        // name, the remainder is a directory path relative to the catalog root.
        let (name, sub_path) = split_catalog_uri_path(uri.path());
        let resolved = self.files_panel.registry.get_by_name(name).map(|c| {
            (
                c.catalog_type,
                std::path::PathBuf::from(&c.path),
                c.read_only,
            )
        });
        let (catalog_type, root_dir, read_only) = match resolved {
            Some(v) => v,
            None => {
                self.nav_model
                    .apply_load_error(id, format!("Catalog '{name}' not found"));
                return;
            }
        };
        match catalog_type {
            // Mainframe catalog root: list the datasets registered in SQLite,
            // not the repository's internal directory layout. Members/qualifier
            // duality is Slice B; a dataset node is a leaf (or shows a Slice B
            // placeholder if expanded).
            CatalogType::Mainframe if sub_path == "/" => self.list_catalog_datasets(id, name),
            CatalogType::Mainframe => {
                // A dataset node under a Mainframe catalog was expanded: member
                // navigation is deferred to Slice B.
                self.nav_model.apply_load_error(
                    id,
                    "Dataset member browsing is available in a later update.".to_string(),
                );
            }
            // POSIX / Native catalog: a real host directory -- list it generically.
            CatalogType::Posix | CatalogType::Native => {
                self.list_catalog_directory(id, root_dir, read_only, &sub_path)
            }
        }
    }

    /// List the datasets of a Mainframe catalog (from SQLite) as tree nodes.
    /// PS -> sequential (leaf), PO -> partitioned, GDG -> GDG base. Dataset node
    /// URIs use the `dataset` scheme (`vfs://dataset/{catalog}/{DSN}`) so open/
    /// expand routing can distinguish them from directories.
    ///
    /// Validates: Requirement 24.8
    fn list_catalog_datasets(&mut self, id: ff_file_tree::NodeId, catalog: &str) {
        use crate::catalog_registry::dataset_node;
        match self.files_panel.registry.list_datasets(catalog) {
            Ok(records) => {
                let children = records
                    .iter()
                    .map(|r| dataset_node(catalog, r))
                    .collect::<Vec<_>>();
                self.nav_model.apply_child_data(id, children);
            }
            Err(e) => self.nav_model.apply_load_error(id, e.to_string()),
        }
    }

    /// List a POSIX/Native catalog's backing directory generically (Req 24.8).
    ///
    /// Validates: Requirement 24.8, 24.5
    fn list_catalog_directory(
        &mut self,
        id: ff_file_tree::NodeId,
        root_dir: std::path::PathBuf,
        read_only: bool,
        sub_path: &str,
    ) {
        use crate::nav_model::list_via_provider;
        // CR-CH-059 RC.A.4: single infallible `ff_vfs::PosixNativeProvider`.
        let provider = crate::posix_provider::PosixProvider::new(root_dir, read_only);
        match list_via_provider(&self.runtime, &provider, sub_path) {
            Ok(entries) => self.nav_model.apply_listing(id, "posix", &entries),
            Err(e) => self.nav_model.apply_load_error(id, e),
        }
    }
}
