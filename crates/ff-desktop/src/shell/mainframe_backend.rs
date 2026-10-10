//! # MAINFRAME backend Command Environment construction (RC.B.8 (c)/(d))
//!
//! The ff-desktop-side helpers that CONSTRUCT the record-capable MAINFRAME
//! backend Command Environment and the catalog-scheme VFS provider, kept out of
//! `construct.rs` / `construct_provider.rs` so those files stay under the
//! 400-line rule (`rust-standards.md`).
//!
//! ## Co-located-by-DSN construction (plan Risk 2)
//!
//! The mainframe CE (`ff_idcams::MainframeEnvironment`) OWNS an
//! `Arc<dyn ff_dscatalog::DatasetAccess>`. We build that from a
//! `ff_dscatalog::CatalogDatasetAccess::new(provider, volume_registry)` composed
//! of existing, already-unit-tested types: a native `ff_vfs::PosixNativeProvider`
//! over the host root and a single-Volume `ff_volume::VolumeRegistry`. This is
//! the bounded "co-located-by-DSN" default: the CE resolves/stores a dataset BY
//! DSN through its own access layer.
//!
//! KNOWN LIMITATION (recorded for the owner's gate, NOT a parallel store): a
//! `CatalogDatasetAccess` resolves a DSN only against the datasets IT itself
//! allocated/resolved (its in-memory `AccessState`). It does NOT share the live
//! SQLite catalog + volume handles the Files Panel's `ff-catalog-registry`
//! opened, so a SAVE through this freshly-built access resolves `NotFound`
//! (rc = 12) for a dataset the Files Panel created in the same session. Making a
//! SAVE visible to a later open in the same session needs the shell to share the
//! EXACT live catalog/volume handles -- a cross-crate handle the shell does not
//! currently own. The wiring, routing, identity threading, and record-store
//! contract are all in place and fully exercised by the scoped routing tests
//! (ff-desktop) + the real-DatasetAccess round-trip (ff-idcams, FEAT-001); only
//! the same-session store-visibility fidelity is bounded by this.

use std::sync::Arc;

use ff_volume::{AccessMode, Volser, Volume, VolumeId, VolumeRegistry};

/// Build the record-capable MAINFRAME backend Command Environment (RC.B.8 (c)).
///
/// Composes a native storage provider over `root` with a single read-write
/// Volume registry into a `CatalogDatasetAccess`, wraps it as the object-safe
/// `Arc<dyn ff_dscatalog::DatasetAccess>`, and hands it to
/// `ff_idcams::MainframeEnvironment::new`. Returned boxed so the
/// `EnvironmentRegistry` holds it without knowing the concrete type.
///
/// Validates: command-environments Requirement 16.1, 16.4
pub(super) fn build_mainframe_backend(
    root: &std::path::Path,
) -> Box<dyn ff_vfs::BackendEnvironment> {
    let provider: Arc<dyn ff_vfs::StorageProvider> =
        Arc::new(ff_vfs::PosixNativeProvider::new(root, false));
    let registry = single_volume_registry();
    let access: Arc<dyn ff_dscatalog::DatasetAccess> =
        Arc::new(ff_dscatalog::CatalogDatasetAccess::new(provider, registry));
    Box::new(ff_idcams::MainframeEnvironment::new(access))
}

/// A minimal single-Volume read-write `VolumeRegistry` for the mainframe CE's
/// dataset access (mirrors the `CatalogDatasetAccess` fixture). The Volume is
/// large enough that a normal editor SAVE does not hit a space abend; a genuine
/// x37/volume-full is still surfaced by the dataset access and mapped to a
/// non-zero rc by the CE (FEAT-001).
fn single_volume_registry() -> VolumeRegistry {
    let mut reg = VolumeRegistry::new();
    let volser = Volser::try_new("FFWB01").unwrap_or_else(|_| {
        // VOLSER is a fixed valid literal; this fallback never runs.
        Volser::try_new("VOL001").expect("valid volser literal")
    });
    let mut vol = Volume::new(VolumeId(1), volser, "ffwb-mainframe-store", 1_000_000);
    vol.set_access_mode(AccessMode::ReadWrite);
    // A fresh registry define cannot collide; ignore the never-taken Err arm.
    let _ = reg.define(vol);
    reg
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
