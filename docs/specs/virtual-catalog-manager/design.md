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

## 6. No Contradictions with Existing Architecture

- `ff-dscatalog` is unchanged — the dialog calls its existing command API
- `connector-local-fs` is unchanged — Native catalogs reuse it directly
- The VFS provider registry gains one new provider (`posix`)
- `TabKind` gains one new variant (`FilesPanel`)
- `shell.rs` handle_command routes `"1"` and `"FILES"` to set `TabKind::FilesPanel`
- `ff-config` schema gains two new keys under `[catalogs]` — no reserved namespace conflict
- `ff-core` `user_data_dir` is already resolved at startup; the default values are computed
  once and passed as strings to `register_schema`, keeping `ff-config` free of any `ff-core` dependency
