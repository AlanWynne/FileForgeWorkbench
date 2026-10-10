//! # Live Provider_Registry Builder
//!
//! The `build_live_provider_registry` free function extracted VERBATIM from
//! `construct.rs` as part of the behaviour-preserving file-size split (pure code
//! movement -- no logic edits). It builds the live `ff-vfs` Provider_Registry
//! seeded with the host-FS `local` provider at shell startup.
//!
//! Validates: command-environments Requirement 17.1, 17.2, 17.3

use std::sync::Arc;

use tokio::runtime::Runtime;

/// Build the live `ff-vfs` Provider_Registry registered at shell startup
/// (CR-CH-053 Task 22, Req 17.1). The registry is seeded with the host-FS
/// `local` provider so a provider is resolvable by scheme at runtime and a
/// plugin-provided `VfsProvider` has a seam to register into (Req 17.2).
///
/// This is ADDITIVE wiring: the host-path open/save path reads through
/// `LocalFsProvider` / `BackendEnvironment` DIRECTLY and never consults this
/// registry, so native file access is unchanged whether or not a non-host
/// provider is later registered (Req 17.3, 17.4). A failure to construct the
/// host provider leaves an empty-but-live registry (startup is best-effort and
/// never aborts on this): native access still works via the direct path, and a
/// plugin can still register a provider later.
///
/// The host provider construction spawns a filesystem watcher that requires a
/// running Tokio reactor, so this is called with the shell's `runtime` and
/// constructs the provider inside `runtime.enter()` (the same reason
/// `tab_manager::open_file` builds its provider inside `runtime.block_on`).
///
/// Validates: command-environments Requirement 17.1, 17.2, 17.3
pub(super) fn build_live_provider_registry(runtime: &Runtime) -> Arc<ff_vfs::ProviderRegistry> {
    use ff_connector_local_fs::LocalFsProvider;
    use ff_vfs::VfsProvider;

    let registry = ff_vfs::ProviderRegistry::new();
    // Enter the runtime so the host provider's watcher can register with the
    // reactor during construction.
    let _guard = runtime.enter();
    match LocalFsProvider::with_defaults() {
        Ok(provider) => {
            let provider: Arc<dyn VfsProvider> = Arc::new(provider);
            if let Err(e) = registry.register(provider) {
                ff_logging::log_warn!(
                    "[vfs] live provider registry: host-FS 'local' provider registration failed: {e}"
                );
            }
        }
        Err(e) => {
            ff_logging::log_warn!(
                "[vfs] live provider registry: host-FS 'local' provider unavailable ({e}); registry live but empty"
            );
        }
    }
    // RC.B.8 (d): additively register the catalog-scheme VFS provider so a
    // MAINFRAME owning-environment has a provider resolvable by scheme (Req
    // 17.4). Best-effort: a failure logs a warning like the host provider.
    // Additive -- the host open/save path never consults the registry, so native
    // access is unchanged whether or not this registers (Req 17.3).
    let catalog_provider = super::mainframe_backend::build_catalog_provider();
    if let Err(e) = registry.register(catalog_provider) {
        ff_logging::log_warn!(
            "[vfs] live provider registry: catalog-scheme provider registration failed: {e}"
        );
    }
    Arc::new(registry)
}
