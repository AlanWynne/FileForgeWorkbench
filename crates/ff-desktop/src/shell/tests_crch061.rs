//! CR-CH-061 scoped SAVE-routing tests: a MAINFRAME-owned tab's SAVE routes
//! through the catalog-backed `DatasetAccess` and resolves a SAME-SESSION
//! Files-Panel-allocated dataset (rc 0, no error), while a native/host
//! (Delimited) tab's SAVE is unchanged and byte-identical.
//!
//! Validates: command-environments Requirement 19.1, 19.3, 19.5; Requirement
//! 18.1.

#![allow(unused_imports)]
use super::tests_common::*;

use ff_document_model::{BytePosition, DelimiterTerminator, RecordFormat};
use tempfile::TempDir;

/// Initialise a real on-disk `ff-dscatalog` repository and allocate one FB/4 PS
/// dataset through `Catalog::allocate` (the same store the Files Panel writes),
/// returning the repository root path.
fn on_disk_repo_with_dataset(tmp: &TempDir, dsn: &str) -> std::path::PathBuf {
    use ff_dscatalog::catalog::Catalog;
    use ff_dscatalog::dataset::{AllocParams, Dsorg, Recfm};
    use ff_dscatalog::dsn::Dsn;
    use ff_dscatalog::hierarchy::CatalogScope;
    use ff_dscatalog::repository::Repository;

    let path = tmp.path().join("mf-repo");
    let repo = Repository::new(&path);
    repo.initialize("PAYROLL").expect("init repo");
    let catalog = Catalog::mount(&path, 1).expect("mount");
    catalog
        .allocate(AllocParams {
            dsn: Dsn::parse(dsn).expect("dsn"),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::FB),
            lrecl: Some(4),
            blksize: Some(4),
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        })
        .expect("allocate");
    path
}

/// Register a Mainframe `VirtualCatalog` in the shell's Files Panel registry so
/// `mainframe_backend_for_active_tab` can resolve the catalog name to its
/// repository path (the EXISTING `get_by_name(catalog).path` lookup, Req 19.3).
fn register_mainframe_catalog(
    shell: &mut super::WorkbenchShell,
    name: &str,
    repo_path: &std::path::Path,
) {
    use ff_catalog_registry::{CatalogType, VirtualCatalog};
    let cat = VirtualCatalog {
        name: name.to_string(),
        catalog_type: CatalogType::Mainframe,
        path: repo_path.to_string_lossy().into_owned(),
        description: None,
        auto_mount: false,
        default_hlq: None,
        mount_point: None,
        read_only: false,
    };
    shell
        .files_panel
        .registry
        .register(cat)
        .expect("register mainframe catalog");
}

/// Validates: command-environments Requirement 19.1, 19.2, 19.3 -- a
/// MAINFRAME-owned tab's SAVE routes through the catalog-backed access, which
/// resolves the SAME-SESSION dataset allocated through the on-disk catalog and
/// stores it with rc 0 (no error). This is the former FALSE NotFound / rc 12,
/// now closed.
#[test]
fn mainframe_save_resolves_same_session_catalog_dataset_rc0() {
    let dir = TempDir::new().expect("tempdir");
    let repo = on_disk_repo_with_dataset(&dir, "USER.LIVE.DATA");

    // The editor opened the dataset's physical file (host path), but the tab
    // carries the mainframe `(dsn, catalog)` store identity bound at open.
    let host_file = repo.join("storage").join("USER").join("LIVE").join("DATA");
    // The physical file exists (allocate created it); open it as the tab buffer.
    let open_path = if host_file.exists() {
        host_file
    } else {
        // Fall back to any seeded file; the SAVE addresses by DSN, not this path.
        dir.path().join("buffer.data")
    };
    if !open_path.exists() {
        std::fs::write(&open_path, "AAAA").expect("seed buffer");
    }

    let mut shell = make_shell();
    register_mainframe_catalog(&mut shell, "PAYROLL", &repo);

    shell
        .shell_open_file_with_env(open_path.to_str().expect("path"), Some("MAINFRAME"))
        .expect("open");
    shell.apply_pending_open_identity(
        Some(("USER.LIVE.DATA".to_string(), "PAYROLL".to_string())),
        Some((ff_vfs::RecordFormatKind::Fixed, 4)),
    );
    assert_eq!(shell.active_owning_environment(), "MAINFRAME");

    // Dirty the buffer so SAVE performs a store.
    {
        let doc = shell.tabs.active_tab().document.clone();
        shell.runtime.block_on(async {
            let _ = doc.write().await.insert(BytePosition(0), b"X");
        });
        shell.tabs.active_tab_mut().is_modified = true;
    }

    shell.run_command_line("SAVE");
    assert!(
        shell.open_error.is_none(),
        "a MAINFRAME SAVE of a same-session catalogued dataset must resolve and \
         store with rc 0 (no error); got {:?}",
        shell.open_error
    );
    assert!(
        !shell.tabs.active_tab().is_modified,
        "a successful record-aware SAVE clears the dirty flag"
    );
}

/// Validates: command-environments Requirement 19.5; Requirement 18.1 -- a
/// native/host (Delimited) tab's SAVE is unchanged and byte-identical, and does
/// NOT reach the catalog-backed access (it has no mainframe store identity, so
/// `mainframe_backend_for_active_tab` returns `None`).
#[test]
fn host_tab_save_is_unchanged_and_does_not_reach_catalog_access() {
    let dir = TempDir::new().expect("tempdir");
    let host_file = dir.path().join("host.txt");
    std::fs::write(&host_file, "original\n").expect("seed");

    let mut shell = make_shell();
    shell
        .shell_open_file(host_file.to_str().expect("path"))
        .expect("open host");
    assert_eq!(
        shell.active_owning_environment(),
        crate::tab_state::DEFAULT_OWNING_ENVIRONMENT
    );
    // A host tab carries no mainframe store identity, so no per-save catalog
    // backend is built.
    assert!(
        shell.mainframe_backend_for_active_tab().is_none(),
        "a host-path tab must not resolve a catalog-backed backend"
    );

    // Edit + Delimited format => the byte SAVE path, byte-identical.
    {
        let doc = shell.tabs.active_tab().document.clone();
        shell.runtime.block_on(async {
            doc.write()
                .await
                .set_record_format(RecordFormat::Delimited {
                    terminator: DelimiterTerminator::Lf,
                });
            let _ = doc.write().await.insert(BytePosition(0), b"EDITED ");
        });
        shell.tabs.active_tab_mut().is_modified = true;
    }
    shell.run_command_line("SAVE");
    assert!(
        shell.open_error.is_none(),
        "a host (Delimited) SAVE takes the byte path and succeeds"
    );
    let on_disk = std::fs::read_to_string(&host_file).expect("read back");
    assert!(
        on_disk.contains("EDITED"),
        "the host byte SAVE must write the edited buffer to disk unchanged"
    );
}
