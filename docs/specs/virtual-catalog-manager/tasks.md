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
  - [x] 10.2 Integrate `CatalogRegistry::save()` into `on_exit()` session save
    - Validates: Requirement 2.1
  - [x] 10.3 Integrate `CatalogRegistry::load()` into startup sequence
    - Validates: Requirement 2.2
  - [x] 10.4 Write unit tests for FilesPanel tab round-trip through session
    - Validates: Requirement 11.3
  - [x] 10.5 Run `cargo test --workspace` — all tests pass
  - [x] 10.6 Update `docs/TCR.md`
  - [x] 10.7 Update `docs/specs/project-master/tasks.md`
