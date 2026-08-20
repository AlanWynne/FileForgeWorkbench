# Implementation Plan: Virtual Catalog Manager

## Overview

Implements POM option 1 as a full Files panel with unified virtual catalog management.
All work is in `ff-desktop` (new modules) plus a new POSIX VFS provider.

---

## Tasks

- [x] 1. TabKind and shell routing
  - [x] 1.1 Add `FilesPanel` variant to `TabKind` enum in `tab_state.rs`
    - Validates: Requirement 11.2
  - [x] 1.2 Update `handle_command()` in `shell.rs`: route `"1"` and `"FILES"` to set active tab kind to `FilesPanel`
    - Validates: Requirement 1.1
  - [x] 1.3 Update `render_central_panel()` to dispatch `TabKind::FilesPanel` → `files_panel::render(ui, state)`
    - Validates: Requirement 1.1
  - [x] 1.4 Update `primary_option_menu.rs` option 1 label to `Files — Virtual File Catalogs — Mainframe, POSIX, Native`
    - Validates: Requirement 11.1
  - [x] 1.5 Write unit tests: `files_panel_tab_kind_exists`, `option_1_routes_to_files_panel`
    - Validates: Requirement 1.1, 11.2
  - [x] 1.6 Run `cargo test` — confirm green

- [x] 2. Catalog Registry
  - [x] 2.1 Create `crates/ff-desktop/src/catalog_registry.rs` with `VirtualCatalog` struct and `CatalogRegistry`
    - Validates: Requirement 2.1–2.5
  - [x] 2.2 Implement `CatalogRegistry::load()` from session TOML `[virtual_catalogs]` array
    - Validates: Requirement 2.2
  - [x] 2.3 Implement `CatalogRegistry::save()` to session TOML
    - Validates: Requirement 2.1
  - [x] 2.4 Implement `register()`, `update()`, `remove()`, `list()`, `list_by_type()`, `get_by_name()`
    - Validates: Requirement 2.3–2.5
  - [x] 2.5 Write unit tests for all registry operations including duplicate-name rejection
    - Validates: Requirement 2.4
  - [x] 2.6 Run `cargo test` — confirm green

- [x] 3. POSIX VFS Provider
  - [x] 3.1 Create `crates/ff-desktop/src/posix_provider.rs` implementing `VfsProvider` for scheme `posix`
    - Validates: Requirement 7.1–7.4
  - [x] 3.2 Implement path normalisation: forward-slash only, case-sensitive, root-jail (no `..` escape)
    - Validates: Requirement 7.3
  - [x] 3.3 Implement read-only enforcement: return `VfsError::PermissionDenied` on write ops when read-only
    - Validates: Requirement 7.6
  - [x] 3.4 Implement `capabilities()` returning Read, Write, List, Metadata, Create, Delete, Rename, Watch
    - Validates: Requirement 7.7
  - [x] 3.5 Write unit tests: path normalisation, root-jail escape prevention, read-only enforcement
    - Validates: Requirement 7.3, 7.6
  - [x] 3.6 Run `cargo test` — confirm green

- [x] 4. Files Panel — skeleton render
  - [x] 4.1 Create `crates/ff-desktop/src/files_panel.rs` with `FilesPanelState` struct and `render()` fn
    - Validates: Requirement 1.2
  - [x] 4.2 Implement left-side catalog tree with three section headers (Mainframe, POSIX, Native)
    - Validates: Requirement 1.4
  - [x] 4.3 Implement empty-state child nodes for sections with no catalogs
    - Validates: Requirement 1.5
  - [x] 4.4 Implement toolbar: New Catalog, Open, Refresh, Properties buttons
    - Validates: Requirement 1.3
  - [x] 4.5 Implement F3/END command to return tab to POM view
    - Validates: Requirement 1.7
  - [x] 4.6 Write unit tests for panel state initialisation and section header rendering logic
    - Validates: Requirement 1.2, 1.4
  - [x] 4.7 Run `cargo test` — confirm green

- [x] 5. Catalog Manager Dialog — Create
  - [x] 5.1 Create `crates/ff-desktop/src/catalog_manager_dialog.rs` with `NewCatalogForm` and render fn
    - Validates: Requirement 3.1–3.8
  - [x] 5.2 Implement catalog type selector (Mainframe / POSIX / Native)
    - Validates: Requirement 3.2
  - [x] 5.3 Implement common fields: Name, Description, Auto-mount
    - Validates: Requirement 3.3
  - [x] 5.4 Implement Mainframe-specific fields: Repository Path, Default HLQ, Create repository now
    - Validates: Requirement 3.4
  - [x] 5.5 Implement POSIX-specific fields: Root Directory, Mount Point, Read-Only
    - Validates: Requirement 3.5
  - [x] 5.6 Implement Native-specific fields: Root Path, Read-Only
    - Validates: Requirement 3.6
  - [x] 5.7 Implement validation and inline error display
    - Validates: Requirement 3.8
  - [x] 5.8 Wire confirm action to `CatalogRegistry::register()` and VFS provider mount
    - Validates: Requirement 3.7
  - [x] 5.9 Write unit tests for form validation logic (duplicate name, empty path, invalid chars)
    - Validates: Requirement 3.8
  - [x] 5.10 Run `cargo test` — confirm green

- [x] 6. Catalog Manager Dialog — Edit and Delete
  - [x] 6.1 Implement `EditCatalogForm` pre-populated from existing catalog properties
    - Validates: Requirement 4.1–4.2
  - [x] 6.2 Implement `DeleteCatalogConfirm` dialog with three-option confirmation
    - Validates: Requirement 4.3–4.5
  - [x] 6.3 Wire delete-catalog-only to `CatalogRegistry::remove()` without file deletion
    - Validates: Requirement 4.4
  - [x] 6.4 Wire delete-catalog-and-files to `CatalogRegistry::remove()` plus recursive directory delete
    - Validates: Requirement 4.5
  - [x] 6.5 Write unit tests for edit/delete form logic
    - Validates: Requirement 4.1–4.5
  - [x] 6.6 Run `cargo test` — confirm green

- [x] 7. Dataset Allocation Dialog
  - [x] 7.1 Create `crates/ff-desktop/src/dataset_alloc_dialog.rs` with `AllocDatasetForm`
    - Validates: Requirement 5.1–5.6
  - [x] 7.2 Implement all ISPF-style fields with conditional visibility (Directory Blocks, GDG Limit, Scratch)
    - Validates: Requirement 5.2
  - [x] 7.3 Implement validation per dataset-catalog Requirement 7 and Requirement 2 rules
    - Validates: Requirement 5.3
  - [x] 7.4 Implement `Allocate Like` pre-population mode
    - Validates: Requirement 5.6
  - [x] 7.5 Wire confirm to `dataset.allocate` command dispatch
    - Validates: Requirement 5.3–5.4
  - [x] 7.6 Write unit tests for field validation (LRECL range, BLKSIZE >= LRECL, GDG limit range)
    - Validates: Requirement 5.3
  - [x] 7.7 Run `cargo test` — confirm green

- [x] 8. Mainframe context menus and POSIX file management
  - [x] 8.1 Implement Mainframe dataset context menus (PS, PDS, member, GDG) in files_panel.rs
    - Validates: Requirement 6.1–6.4
  - [x] 8.2 Implement inline rename for datasets and members
    - Validates: Requirement 6.5
  - [x] 8.3 Implement delete confirmation for datasets
    - Validates: Requirement 6.6
  - [x] 8.4 Implement POSIX context menus (file, directory) with New File / New Directory inline inputs
    - Validates: Requirement 8.1–8.5
  - [x] 8.5 Implement Native catalog context menus (platform-appropriate shell actions)
    - Validates: Requirement 9.3–9.4
  - [x] 8.6 Write unit tests for context menu item visibility logic per catalog type
    - Validates: Requirement 6.1–6.4, 8.1, 9.3
  - [x] 8.7 Run `cargo test` — confirm green

- [x] 9. Content area and unified explorer view
  - [x] 9.1 Implement right-side content area with Name/Type/Size/Modified columns
    - Validates: Requirement 10.1
  - [x] 9.2 Implement column-header sort
    - Validates: Requirement 10.2
  - [x] 9.3 Implement double-click to open file / navigate into directory
    - Validates: Requirement 10.3–10.4
  - [x] 9.4 Implement breadcrumb path bar
    - Validates: Requirement 10.5
  - [x] 9.5 Implement content area filter input
    - Validates: Requirement 10.6
  - [x] 9.6 Write unit tests for sort logic and filter logic
    - Validates: Requirement 10.2, 10.6
  - [x] 9.7 Run `cargo test` — confirm green

- [x] 11. Catalog storage default paths
  - [x] 11.1 Register `catalogs.default_mainframe_root` schema key in `main.rs` `register_builtin_schema()`,
          pre-populated with `{user_data_dir}/catalogs/mainframe`
    - Validates: Requirement 12.3, 12.5
  - [x] 11.2 Register `catalogs.default_posix_root` schema key in `main.rs` `register_builtin_schema()`,
          pre-populated with `{user_data_dir}/catalogs/posix`
    - Validates: Requirement 12.4, 12.5
  - [x] 11.3 In `catalog_manager_dialog.rs` new-Mainframe path: read `catalogs.default_mainframe_root`
          from config and pre-populate `Repository Path` field with `{default}/{catalog-name}`,
          updating live as the catalog name field changes
    - Validates: Requirement 12.1, 12.7
  - [x] 11.4 In `catalog_manager_dialog.rs` new-POSIX path: read `catalogs.default_posix_root`
          from config and pre-populate `Root Directory` field
    - Validates: Requirement 12.2, 12.7
  - [x] 11.5 Write unit tests: default path computed correctly for Mainframe (name appended),
          POSIX (root used directly), and that the field remains editable
    - Validates: Requirement 12.1, 12.2
  - [x] 11.6 Run `cargo test` — confirm green

- [x] 10. Session persistence for FilesPanel tab and catalog registry
  - [x] 10.1 Update `session_manager.rs` to persist/restore `FilesPanel` tab kind
    - Validates: Requirement 11.3
  - [ ] 10.2 Integrate `CatalogRegistry::save()` into `on_exit()` session save
    - Validates: Requirement 2.1
    - ⚠️ BLOCKED: marked done in error — wiring code was never written (B010)
  - [ ] 10.3 Integrate `CatalogRegistry::load()` into startup sequence
    - Validates: Requirement 2.2
    - ⚠️ BLOCKED: marked done in error — wiring code was never written (B010)
  - [x] 10.4 Write unit tests for FilesPanel tab round-trip through session
    - Validates: Requirement 11.3
  - [x] 10.5 Run `cargo test --workspace` — all tests pass
  - [x] 10.6 Update `docs/TCR.md`
  - [x] 10.7 Update `docs/specs/project-master/tasks.md`

- [ ] 14. Default Home Catalog on First Launch (Req 14)
  - [ ] 14.1 Write failing test `no_native_catalogs_triggers_home_catalog_creation` in `session_manager.rs` or a new `startup_tests.rs` — verifies that after the startup logic runs on an empty registry, a Native catalog named `"Home"` is present
    - Validates: Requirement 14.1, 14.2
  - [ ] 14.2 Write failing test `existing_native_catalog_suppresses_home_creation` — verifies that when a Native catalog already exists, no `"Home"` catalog is added
    - Validates: Requirement 14.4
  - [ ] 14.3 Write failing test `home_catalog_persisted_immediately` — verifies that `save_catalog_registry` is called and the catalog survives a load round-trip
    - Validates: Requirement 14.3
  - [ ] 14.4 Write failing test `delete_home_native_catalog_is_rejected` — verifies that `execute_delete` returns an error when the catalog name is `"Home"` and type is `Native`
    - Validates: Requirement 14.6
  - [ ] 14.5 Write failing test `delete_renamed_home_catalog_is_permitted` — verifies that a Native catalog formerly named `"Home"` but now renamed can be deleted
    - Validates: Requirement 14.7
  - [ ] 14.6 Add `ensure_default_home_catalog()` free function in `shell/update.rs` that encapsulates the check-and-create logic; takes `&mut CatalogRegistry` and `home_path: PathBuf`
    - Validates: Requirement 14.1, 14.4, 14.5
  - [ ] 14.7 Call `ensure_default_home_catalog()` in the one-shot startup block in `update.rs`, immediately after `self.files_panel.registry = session.load_catalog_registry()`; follow with `session.save_catalog_registry()` when a catalog was added
    - Validates: Requirement 14.2, 14.3
  - [ ] 14.8 In `catalog_manager_dialog.rs` `execute_delete()`: guard against deleting a catalog named `"Home"` of type `Native`; return `Err("The Home catalog cannot be deleted. Rename or edit it instead.".to_string())`
    - Validates: Requirement 14.6
  - [ ] 14.9 Run `cargo test -p ff-desktop` — all tests pass
  - [ ] 14.10 Run `cargo clippy -p ff-desktop -- -D warnings` — clean
  - [ ] 14.11 Update `docs/TCR.md` and `docs/specs/project-master/tasks.md`

  - [x] 13.1 Add `catalogs_path()` helper to `SessionManager` returning `{session_dir}/catalogs.toml`
    - Validates: Requirement 2.1
  - [x] 13.2 Add `save_catalog_registry()` to `SessionManager` that calls `registry.save_to_toml()` and writes to `catalogs.toml`
    - Validates: Requirement 2.1
  - [x] 13.3 Add `load_catalog_registry()` to `SessionManager` that reads `catalogs.toml` and returns a `CatalogRegistry`
    - Validates: Requirement 2.2
  - [x] 13.4 Call `session.save_catalog_registry(&self.files_panel.registry)` in `WorkbenchShell::on_exit()`
    - Validates: Requirement 2.1
  - [x] 13.5 Call `session.load_catalog_registry()` in startup sequence and assign to `self.files_panel.registry`
    - Validates: Requirement 2.2
  - [x] 13.6 Write unit tests: `save_and_load_catalog_registry_round_trips`, `load_missing_catalog_file_returns_empty_registry`
    - Validates: Requirement 2.1, 2.2
  - [x] 13.7 Run `cargo test -p ff-desktop` — 382 tests pass
  - [x] 13.8 Run `cargo clippy -p ff-desktop -- -D warnings` — clean
  - [x] 13.9 Update `docs/TCR.md` and `docs/specs/project-master/tasks.md`
  - [ ] 12.1 Add `AllocatedDataset` struct to `files_panel.rs` with fields: `name`, `dsorg`, `recfm`, `lrecl`, `blksize`, `description`
    - Validates: Requirement 13.1
  - [ ] 12.2 Add `datasets: HashMap<String, Vec<AllocatedDataset>>` field to `FilesPanelState`; add `pending_alloc_catalog: Option<String>` to track which catalog opened the dialog
    - Validates: Requirement 13.1, 13.2
  - [ ] 12.3 In `shell.rs` `AllocateDataset` action handler: store the catalog name in `files_panel.pending_alloc_catalog` before opening the dialog
    - Validates: Requirement 13.2
  - [ ] 12.4 In `shell.rs` `AllocOutcome::Confirmed` handler: read `AllocParams` from form, call `files_panel.add_dataset(catalog_name, params)` to insert into the map
    - Validates: Requirement 13.2
  - [ ] 12.5 Add `add_dataset()` method to `FilesPanelState` that converts `AllocParams` into `AllocatedDataset` and appends to the correct catalog entry
    - Validates: Requirement 13.2
  - [ ] 12.6 In `render_content_area()`: when a catalog is selected, call `load_entries_from_datasets()` to populate `ContentAreaState::entries` from the datasets map
    - Validates: Requirement 13.3
  - [ ] 12.7 Add `load_entries_from_datasets()` helper that converts `AllocatedDataset` to `ContentEntry` (dsorg as type string; `is_container = true` for PO/PDSE/GDG)
    - Validates: Requirement 13.3
  - [ ] 12.8 Extend `CatalogRegistry::save_to_toml()` / `load_from_toml()` to serialise/deserialise the `datasets` map under `[catalog_datasets]`
    - Validates: Requirement 13.4
  - [ ] 12.9 Wire dataset save/load into `session_manager.rs` `save()` and `load()` paths
    - Validates: Requirement 13.4
  - [ ] 12.10 In catalog delete handler: remove all datasets for the deleted catalog from the map
    - Validates: Requirement 13.5
  - [ ] 12.11 Write unit tests: `add_dataset_inserts_into_map`, `load_entries_populates_content_area`, `delete_catalog_removes_datasets`, `dataset_map_round_trips_through_toml`
    - Validates: Requirement 13.1–13.5
  - [ ] 12.12 Run `cargo test -p ff-desktop` — all tests pass
  - [ ] 12.13 Run `cargo clippy -p ff-desktop -- -D warnings` — clean
  - [ ] 12.14 Update `docs/TCR.md` and `docs/specs/project-master/tasks.md`
