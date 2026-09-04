# Design Document: Virtual Catalog Manager

## 1. Overview

The Virtual Catalog Manager owns POM option 1 (`[FILES]` tab). It is a new UI subsystem in
`ff-desktop` that renders a split-panel file explorer supporting three catalog types: Mainframe,
POSIX, and Native. All resource access flows through the VFS abstraction layer.

The `Native` type replaces the previous "Windows" and "Local" distinction. `connector-local-fs`
already handles Windows, Linux, and macOS path conventions transparently — no platform-specific
catalog type is needed.

## 2. Architecture

### 2.1 New TabKind Variant

`TabKind` gains a new variant `FilesPanel`. The central panel dispatch in `shell.rs` routes
`TabKind::FilesPanel` to `files_panel::render(ui, state)`.

```
TabKind::FilesPanel → files_panel::render(ui, &mut FilePanelState)
```

### 2.2 New Crate: ff-virtual-catalog (optional future extraction)

For the initial implementation, the catalog registry and POSIX provider live in `ff-desktop`
as new modules. Future extraction to a dedicated crate is deferred.

New modules in `ff-desktop/src/`:
- `files_panel.rs` — egui render function for the Files panel
- `catalog_registry.rs` — in-memory + persisted catalog registry
- `catalog_manager_dialog.rs` — New/Edit/Delete catalog dialogs
- `dataset_alloc_dialog.rs` — Mainframe dataset allocation dialog
- `posix_provider.rs` — POSIX VFS provider implementation

### 2.3 POSIX VFS Provider
The POSIX provider wraps the existing `connector-local-fs` provider, adding:
- POSIX path normalisation (forward-slash only, case-sensitive)
- Per-catalog root isolation (paths cannot escape the catalog root)
- Read-only enforcement at the provider level

Registration: `vfs://posix/{catalog-name}/{posix-path}`

### 2.4 Catalog Registry Persistence

Catalogs are persisted in `session.toml` under `[virtual_catalogs]`:

```toml
[[virtual_catalogs]]
name = "PAYROLL"
type = "Mainframe"
path = "C:/ffworkbench/catalogs/payroll"
default_hlq = "PAYROLL"
auto_mount = true

[[virtual_catalogs]]
name = "dev-posix"
type = "POSIX"
path = "C:/projects/dev"
mount_point = "/"
read_only = false
auto_mount = true

[[virtual_catalogs]]
name = "projects"
type = "Native"
path = "C:/projects"
read_only = false
auto_mount = true
```

The `type` field uses the string values `"Mainframe"`, `"POSIX"`, `"Native"`.

### 2.5 Dialog Architecture

All dialogs are egui modal windows rendered within the Files panel frame. They use a
`DialogState` enum pattern consistent with the existing toolchain panel dialogs:

```rust
enum FilesDialogState {
    None,
    NewCatalog(NewCatalogForm),
    EditCatalog(EditCatalogForm),
    DeleteCatalog(DeleteCatalogConfirm),
    AllocateDataset(AllocDatasetForm),
    PosixNewFile(PosixNewFileForm),
    PosixNewDir(PosixNewDirForm),
}
```

### 2.6 Data Flow

```
POM option 1 pressed
  → shell.rs: handle_command("1") → set active tab kind to FilesPanel
  → files_panel::render(ui, state)
      ├─ left: catalog tree (CatalogTreeState)
      │    ├─ Mainframe section → ff-dscatalog VFS provider
      │    ├─ POSIX section    → posix_provider (new)
      │    └─ Native section   → connector-local-fs (Windows/Linux/macOS)
      └─ right: content area (ContentAreaState)
           └─ VFS list() on selected node URI
```

## 3. Catalog Type Summary

| Type | VFS Scheme | Provider | New code needed? |
|---|---|---|---|
| Mainframe | `catalog` | `ff-dscatalog` | No |
| POSIX | `posix` | New `posix_provider.rs` | Yes |
| Native | `local` | `connector-local-fs` | No |

The `Native` type works identically on Windows, Linux, and macOS because `connector-local-fs`
already abstracts platform path conventions. The section header label in the UI appends the
platform name at runtime using `std::env::consts::OS`.

## 5. Catalog Storage Default Paths

### 5.1 Configuration Keys

Two new keys are registered with `ff-config` under the `[catalogs]` namespace:

| Key | Type | Built-in Default | Description |
|---|---|---|---|
| `catalogs.default_mainframe_root` | `String` | `{user_data_dir}/catalogs/mainframe` | Root directory under which new Mainframe catalog repositories are created by default |
| `catalogs.default_posix_root` | `String` | `{user_data_dir}/catalogs/posix` | Default root directory suggested when creating a new POSIX catalog |

`{user_data_dir}` is resolved at startup by `ff-core::Platform::user_data_dir()` and substituted
before the default is registered. The resolved string (not the template) is stored as the schema
default so that `ff-config` never needs to know about `ff-core`.

### 5.2 Schema Registration

The keys are registered during the `catalog_registry` initialisation path in `ff-desktop`,
after `ff-config` is available:

```rust
config.register_schema(SchemaEntry {
    key: "catalogs.default_mainframe_root".to_string(),
    value_type: ValueType::String,
    default: ConfigValue::String(resolved_mainframe_default),
    description: "Default parent directory for new Mainframe catalog repositories. \
                  The catalog name is appended as a subdirectory.".to_string(),
    constraints: None,
})?;

config.register_schema(SchemaEntry {
    key: "catalogs.default_posix_root".to_string(),
    value_type: ValueType::String,
    default: ConfigValue::String(resolved_posix_default),
    description: "Default root directory suggested when creating a new POSIX catalog.".to_string(),
    constraints: None,
})?;
```

### 5.3 Dialog Pre-population

When `catalog_manager_dialog.rs` opens for a new catalog:

- **Mainframe**: read `catalogs.default_mainframe_root` from config, append `/{catalog-name}`
  (using the name field's current value, updated live as the user types).
- **POSIX**: read `catalogs.default_posix_root` from config, place it directly in the
  `Root Directory` field.

The pre-populated value is editable — it is a suggestion, not a constraint.

### 5.4 Hot-Reload

Both keys participate in `ff-config` hot-reload. The dialog reads the config value at open
time; no subscription is needed inside the dialog itself.

### 5.5 Settings Panel Exposure

Because both keys are registered in the schema with descriptions, the auto-generated Settings
panel (when implemented) will surface them under a `Catalogs` section without additional code.

## 7. Allocated Dataset Store (Requirement 13)

> **Revised by CR-CH-006 (Phase BU).** The in-memory HashMap and session-TOML store are
> replaced by the SQLite catalog database in `ff-dscatalog`. The `AllocatedDataset` struct
> and `datasets` field are removed from `FilesPanelState`.

### 7.1 Allocation Confirm Flow

When `AllocOutcome::Confirmed` fires in `shell.rs`, the shell calls
`CatalogRegistry::allocate(catalog_name, alloc_params)`. This delegates to
`ff-dscatalog`'s `Catalog::allocate()`, which writes the dataset entry to the SQLite
`catalog.db` and creates the physical object via `NativeFileProvider`. No in-memory
state is updated beyond refreshing the content area.

### 7.2 Content Area Population

When a Mainframe catalog node is selected, `render_content_area` calls
`CatalogRegistry::list_datasets(catalog_name)`, which executes a `SELECT` against the
SQLite `datasets` table and returns `Vec<DatasetRecord>`. Each record is converted to a
`ContentEntry` (DSN as name, DSORG as type, size/modified from catalog metadata).

The same `list_datasets` call is used by `render_mainframe_content()` in
`file_explorer_panel.rs` for Option 2.

### 7.3 Session Persistence

Dataset persistence is provided by the SQLite `catalog.db`. No TOML serialisation of
dataset entries is needed. The `[catalog_datasets]` section is removed from
`session.toml` and from `SessionManager`.

### 7.4 CatalogRegistry API additions

Two new methods are added to `CatalogRegistry` in `catalog_registry.rs`:

```rust
// Allocate a dataset in the named catalog via ff-dscatalog.
pub fn allocate(&self, catalog_name: &str, params: AllocParams)
    -> Result<(), CatalogError>;

// List all datasets in the named catalog from SQLite.
pub fn list_datasets(&self, catalog_name: &str)
    -> Result<Vec<DatasetRecord>, CatalogError>;
```

Both methods look up the catalog by name in the registry, obtain the `ff-dscatalog`
`Catalog` handle, and delegate to its existing API.

## 8. Default Home Catalog (Requirement 14)

### 8.1 Trigger Point

The check runs inside the one-shot startup block in `update.rs`, immediately after
`session.load_catalog_registry()` assigns the loaded registry to `self.files_panel.registry`.

### 8.2 Logic

```rust
if self.files_panel.registry.list_by_type(CatalogType::Native).is_empty() {
    let home = dirs_home_dir().unwrap_or_else(|| std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from(".")));
    let catalog = VirtualCatalog {
        name: "Home".to_string(),
        catalog_type: CatalogType::Native,
        path: home.to_string_lossy().into_owned(),
        description: Some("Default home directory catalog".to_string()),
        auto_mount: true,
        default_hlq: None,
        mount_point: None,
        read_only: false,
    };
    // register() only fails on duplicate name or invalid name — neither applies here
    let _ = self.files_panel.registry.register(catalog);
    // Persist immediately so the catalog survives restart
    if let Some(session) = &self.session {
        session.save_catalog_registry(&self.files_panel.registry);
    }
}
```

### 8.3 Home Directory Resolution

The `dirs` crate is already a transitive dependency via `ff-session`. We use
`dirs::home_dir()` directly in `update.rs`. No new crate dependency is required.

### 8.4 Deletion Guard

In `catalog_manager_dialog.rs`, `execute_delete()` gains an early guard:

```rust
if confirm.name == "Home" && confirm.catalog_type == CatalogType::Native {
    return Err("The Home catalog cannot be deleted. Rename or edit it instead.".to_string());
}
```

This is a name+type check, not a special flag on `VirtualCatalog`, so no schema change is
needed. Once the user renames the catalog the guard no longer fires.

### 8.5 No Contradictions

- The check is purely additive — it only fires when `list_by_type(Native).is_empty()`.
- Existing catalogs are never modified.
- The `register()` call is idempotent in the sense that it only runs when no Native
  catalog exists, so the `"Home"` name cannot collide with an existing Native catalog.
  (A Mainframe or POSIX catalog named `"Home"` would cause `DuplicateName` — the `let _ =`
  silently ignores this edge case, which is acceptable.)
- The immediate `save_catalog_registry()` call reuses the existing persistence path.

## 9. Catalog Properties — Repository Path Display (Requirement 15)

The `EditCatalogForm` already holds all `VirtualCatalog` fields. The only change needed is to
render the `path` field as a read-only labelled row in `render_edit()` in
`catalog_manager_dialog.rs`. No new state is required.

```rust
// In render_edit(), after the Name row:
ui.horizontal(|ui| {
    ui.label("Repository Path:");
    ui.label(egui::RichText::new(&form.path).monospace().weak());
});
```

The field is read-only (label, not TextEdit) because the path is set at creation time and
cannot be changed without re-creating the catalog.

## 10. VFS Dataset Path Resolution (Requirement 16)

> **Revised by CR-CH-006 (Phase BU).** The `resolve_dataset_path()` DSN-to-path function
> is replaced by a SQLite catalog lookup. The catalog is the sole authority for physical
> location.

### 10.1 Resolution Rule

Given a DSN, the physical path is obtained by calling
`CatalogRegistry::resolve(dsn) -> Result<ResolvedDataset, CatalogError>`.
`ResolvedDataset` carries the `physical_locator` (UUID-based path of the form
`{workspace}/datasets/objects/<uuid>.dat`) returned from the SQLite `datasets` table.

The old dot-to-directory-separator mapping is removed entirely.

### 10.2 Helper Function

```rust
/// Resolve a DSN via the SQLite catalog and open or create the physical file.
/// Returns the resolved PathBuf on success, or an error string for display.
pub fn resolve_and_open_dataset(
    registry: &CatalogRegistry,
    dsn: &str,
) -> Result<std::path::PathBuf, String>;
```

This function is in `files_panel.rs`, has no egui dependency, and is independently
testable with a mock registry.

### 10.3 Open Flow

In `render.rs`, the `FilesPanelAction::OpenFile(dsn)` handler for Mainframe catalogs:

1. Call `resolve_and_open_dataset(&registry, &dsn)`.
2. On `Ok(path)` and `path.exists()` -- dispatch `file.open`.
3. On `Ok(path)` and `!path.exists()` -- call `create_dataset_file(&path)`, then dispatch
   `file.open`; on creation failure show the `cannot create` error message.
4. On `Err(msg)` -- show `msg` in the status bar.

The same flow applies to the File Explorer Panel double-click handler in
`render_dataset_children()`.

### 10.4 Crate Dependencies

`ff-dscatalog` is already a dependency of `ff-desktop`. No new crate dependencies are
required.



- `ff-dscatalog` is unchanged — the dialog calls its existing command API
- `connector-local-fs` is unchanged — Native catalogs reuse it directly
- The VFS provider registry gains one new provider (`posix`)
- `TabKind` gains one new variant (`FilesPanel`)
- `shell.rs` handle_command routes `"1"` and `"FILES"` to set `TabKind::FilesPanel`
- `ff-config` schema gains two new keys under `[catalogs]` — no reserved namespace conflict
- `ff-core` `user_data_dir` is already resolved at startup; the default values are computed
  once and passed as strings to `register_schema`, keeping `ff-config` free of any `ff-core` dependency
