//! # MAINFRAME backend Command Environment construction (RC.B.8 (c)/(d))
//!
//! The ff-desktop-side helpers that CONSTRUCT the record-capable MAINFRAME
//! backend Command Environment and the catalog-scheme VFS provider, kept out of
//! `construct.rs` / `construct_provider.rs` so those files stay under the
//! 400-line rule (`rust-standards.md`).
//!
//! ## Catalog-backed construction (CR-CH-061)
//!
//! The mainframe CE (`ff_idcams::MainframeEnvironment`) OWNS an
//! `Arc<dyn ff_dscatalog::DatasetAccess>`. We build that from a
//! `ff_dscatalog::CatalogRepositoryAccess` (dataset-catalog Req 36): its
//! `resolve`/`open`/`close` consult the ON-DISK SQLite catalog + physical files
//! the Files Panel's allocate writes, opening a short-lived catalog connection
//! per `resolve` (so the shared `Arc<dyn DatasetAccess>` stays `Send + Sync`).
//! This closes the former FALSE NotFound / rc 12 gap: a dataset the Files Panel
//! created in the same session now resolves and stores with rc 0.
//!
//! The catalog repository path reaches the CE from the active tab's EXISTING
//! `(dsn, catalog)` store identity. SAVE resolution is PER-SAVE
//! (`build_mainframe_backend_for_repo`): the shell resolves the active
//! MAINFRAME tab's catalog name to its repository path and builds a backend
//! pointed at that catalog, so multiple mainframe catalogs are each served. The
//! build-time registry entry (`build_mainframe_backend`) is the fallback used
//! when no per-save catalog path is resolvable.

use std::path::PathBuf;
use std::sync::Arc;

/// Build the record-capable MAINFRAME backend Command Environment (RC.B.8 (c),
/// CR-CH-061 Req 19.1).
///
/// The CE's `Arc<dyn ff_dscatalog::DatasetAccess>` is the catalog-backed
/// `CatalogRepositoryAccess` (dataset-catalog Req 36): its `resolve`/`open`/
/// `close` consult the ON-DISK SQLite catalog + physical files the Files Panel
/// writes, NOT an isolated in-memory dataset map (closing the former FALSE
/// NotFound / rc 12). `repository_paths` are the active mainframe catalog
/// repository roots; an empty set resolves NotFound until a save threads a
/// catalog path (per-save resolution builds a fresh backend for the active
/// tab's catalog -- see `build_mainframe_backend_for_repo`).
///
/// Returned boxed so the `EnvironmentRegistry` holds it without knowing the
/// concrete type.
///
/// Validates: command-environments Requirement 16.1, 16.4, 19.1
pub(super) fn build_mainframe_backend(
    repository_paths: Vec<PathBuf>,
) -> Box<dyn ff_vfs::BackendEnvironment> {
    let access: Arc<dyn ff_dscatalog::DatasetAccess> =
        Arc::new(ff_dscatalog::CatalogRepositoryAccess::new(repository_paths));
    Box::new(ff_idcams::MainframeEnvironment::new(access))
}

/// Build a mainframe backend CE pointed at a SINGLE catalog repository path
/// (CR-CH-061 Req 19.3, per-save resolution). Used at SAVE time to resolve a
/// MAINFRAME-owned tab's dataset against the catalog that owns it, so multiple
/// mainframe catalogs are each served.
///
/// Validates: command-environments Requirement 19.1, 19.3
pub(super) fn build_mainframe_backend_for_repo(
    repository_path: PathBuf,
) -> Box<dyn ff_vfs::BackendEnvironment> {
    build_mainframe_backend(vec![repository_path])
}

/// Build the catalog-scheme `ff_dscatalog::CatalogVfsProvider` for additive
/// registration in the live Provider_Registry (RC.B.8 (d), Req 17.4).
///
/// Best-effort co-located construction: the shell holds the
/// `ff-catalog-registry` wrapper (used by the Files Panel), NOT an
/// `ff_dscatalog::catalog_registry::CatalogRegistry`, so this builds a fresh
/// empty dscatalog registry as the provider seam. The registration is ADDITIVE
/// -- the host open/save path never consults the Provider_Registry, so native
/// access is unchanged whether or not this registers (Req 17.3).
///
/// Validates: command-environments Requirement 17.4
pub(super) fn build_catalog_provider() -> Arc<dyn ff_vfs::VfsProvider> {
    use std::sync::RwLock;
    // `CatalogVfsProvider::new` REQUIRES `Arc<RwLock<CatalogRegistry>>` by its
    // public signature; `CatalogRegistry` is not `Send + Sync`, so clippy flags
    // the `Arc`. The wrapper type is mandated by the provider API (not an
    // incidental choice) and the provider is single-threaded shell state, so the
    // non-Send/Sync Arc is correct here.
    #[allow(clippy::arc_with_non_send_sync)]
    let registry = Arc::new(RwLock::new(
        ff_dscatalog::catalog_registry::CatalogRegistry::new(),
    ));
    Arc::new(ff_dscatalog::CatalogVfsProvider::new(registry))
}
