# FFWB File Routing Architecture: ff-vfs, Backend Environments, and the Editor Save Path

Read-only investigation. No code was changed. Every claim below cites a file (and crate/trait/function) actually read.

---

## 1. Summary answer (is the mental model correct?)

The user's mental model is **mostly right in spirit but imprecise on the mechanics**, and it conflates two distinct abstractions that both live in `ff-vfs`.

Restating the user's model and grading each part:

- **(a) "All editor / file-utility filesystem activity routes through ff-vfs."**
  **Partly true, with an important nuance.** `ff-vfs` is the declared single I/O abstraction (`FFW-ARCH-001`), and both halves of the editor's file lifecycle go through *types defined in `ff-vfs`*. BUT there are **two different `ff-vfs` abstractions on the two halves of the lifecycle**, and the live host-path open/save path **deliberately bypasses the `ff-vfs` runtime `ProviderRegistry`**:
  - **OPEN / READ** goes through the `ff-vfs::VfsProvider` trait, implemented by `ff-connector-local-fs::LocalFsProvider` (scheme `"local"`). But `tab_manager::open_file` constructs a `LocalFsProvider` **directly** and calls `.read()` on it; it does **not** route through the shell's live `ProviderRegistry`/`Vfs` facade.
  - **SAVE / WRITE** goes through a **different** `ff-vfs` trait: `ff-vfs::BackendEnvironment` (the "store contract" / editor SAVE seam), implemented by the `ff-ce-*` crates and the mainframe CE. It does **not** use `VfsProvider` at all.
  - `construct_provider.rs` states this explicitly: the host-path open/save path "reads through `LocalFsProvider` / `BackendEnvironment` DIRECTLY and never consults this registry."
  So "everything flows through ff-vfs" is true at the *type/trait* level, but it is **not** one unified pipeline and it is **not** the `Vfs`/`ProviderRegistry` facade that the VFS design doc depicts.

- **(b) "Configuration files bypass the VFS."**
  **Correct.** `ff-config` and `ff-session` call `std::fs` directly (e.g. `ff-config/src/audit.rs`, `ff-session/src/crash_recovery.rs`, menu/theme/session/keymap loaders in `ff-desktop`). They do not go through `ff-vfs`. (Note: this technically contradicts the aspirational `FFW-ARCH-001` "no `std::fs` in consuming crates" wording -- see section 6.)

- **(c) "ff-vfs dispatches to backend 'controllers' (native/host, posix, ntfs, etc.)."**
  **Correct in concept, but the "controllers" are two different things and the dispatch is not a single central switch:**
  - The user's "native / posix / ntfs controllers" map to the **backend Command Environments**: `ff-ce-host-fs` (a *decider*, not an FS), `ff-ce-ntfs` (`NtfsEnvironment`), `ff-ce-posix` (`PosixEnvironment`). These implement `BackendEnvironment` and are the **SAVE-side** executors.
  - There is also a separate family of **`VfsProvider`** backends keyed by URI scheme (`local` = `ff-connector-local-fs`, `catalog` = `ff-dscatalog::CatalogVfsProvider`). These are the **read/registry-side** backends.
  - So "ff-vfs dispatches to controllers" is realised as **two parallel abstractions**, not one.

**Bottom line:** There is no single unified "all file activity -> ff-vfs -> one dispatcher -> controller" pipe. There are **two `ff-vfs` seams** (a scheme-routed async `VfsProvider` registry for reads/listing/search, and a name-addressed sync `BackendEnvironment` store contract for saves), the live **host open/save path bypasses the provider registry entirely**, and **config/session bypass the VFS altogether**.

---

## 2. The central abstractions in ff-vfs

`ff-vfs` exports **two separate trait families** (`crates/ff-vfs/src/lib.rs`):

### 2a. `VfsProvider` -- the async, scheme-routed "virtual file system"
- Defined in `crates/ff-vfs/src/provider.rs` (`pub trait VfsProvider: Send + Sync`). Object-safe, async (`async_trait`), Tokio-based. Methods: `scheme`, `capabilities`, `open`, `read`, `read_stream`, `write`, `create`, `delete`, `rename`, `list`, `stat`, `exists`, `watch`, `search`.
- Routed by **URI scheme** via `ProviderRegistry` (`crates/ff-vfs/src/registry.rs`) behind the `Vfs` facade (`crates/ff-vfs/src/vfs.rs`). `Vfs::resolve_provider` looks up a provider by `ResourceUri::scheme()` (format `vfs://provider/path`, `crates/ff-vfs/src/uri.rs`).
- Key types: `ResourceUri`, `VfsError`, `VfsFile`, `VfsCapabilities`, `VfsEntry`.
- This is the abstraction the VFS **design doc** (`docs/specs/virtual-file-system/design.md`) describes as the "sole public API for all file access."

### 2b. `BackendEnvironment` -- the sync, name-addressed "store contract" (the editor SAVE seam)
- Defined in `crates/ff-vfs/src/backend_environment.rs` (`pub trait BackendEnvironment: Send + Sync`). **Shell-free, synchronous, NOT async, NOT URI-routed.** Deliberately distinct from `VfsProvider`.
- Methods:
  - `name(&self) -> &str` -- the stable addressing name (`"NTFS"`, `"POSIX"`, `"HOSTFS"`, `"MAINFRAME"`).
  - `is_case_sensitive(&self) -> bool`.
  - `save(&self, path: &Path, bytes: &[u8]) -> io::Result<()>` -- the **byte** store entry (retained for host CEs).
  - `save_records(&self, target: &StoreTarget, records: &mut dyn RecordSource, attrs: &RecordAttrs) -> RecordStoreOutcome` -- the **record-aware** store entry (CR-CH-060), with a provided default returning `NotRecordCapable`.
  - `record_capable(&self) -> bool` (default `false`).
- Supporting types (all `ff-vfs`-local, intentionally dependency-free): `StoreTarget` (dataset identity: `dsn`, `owning_env`, `catalog_id` -- NOT a host path), `RecordFormatKind` (Fixed/Variable/Undefined), `RecordAttrs` (recfm/lrecl/encoding), `RecordSource` (object-safe pull stream), `RecordStoreOutcome` (`Stored { rc }` | `NotRecordCapable`).
- This is the trait called out in steering `framework-conformance.md` (CR-CH-060) as a load-bearing core type.

**The concrete backends that exist today:**

| User's term | Concrete type | Crate | Trait implemented | Role |
|---|---|---|---|---|
| "native" (host FS) | *decider function* `native_backend_environment()` | `ff-ce-host-fs` | returns `Box<dyn BackendEnvironment>` | Resolves the "native" role at startup: Windows -> NTFS, else POSIX. NOT itself an FS. |
| "ntfs" | `NtfsEnvironment` | `ff-ce-ntfs` | `BackendEnvironment` | Light, OS-backed byte `save`; case-insensitive. |
| "posix" | `PosixEnvironment` | `ff-ce-posix` | `BackendEnvironment` | Light, OS-backed byte `save`; case-sensitive. |
| mainframe | `MainframeEnvironment` | `ff-idcams` | `BackendEnvironment` (`record_capable()==true`, overrides `save_records`) | Record-aware store over `ff_dscatalog::DatasetAccess`. |
| "local" (read side) | `LocalFsProvider` (scheme `"local"`) | `ff-connector-local-fs` | `VfsProvider` | The primary read/list/watch provider. |
| catalog (read side) | `CatalogVfsProvider` (scheme `catalog`) | `ff-dscatalog` | `VfsProvider` | Catalog-scheme provider, additively registered. |

Evidence: `crates/ff-ce-host-fs/src/lib.rs`, `crates/ff-ce-ntfs/src/lib.rs`, `crates/ff-ce-posix/src/lib.rs`, `crates/ff-idcams/src/mainframe_env.rs`, `crates/ff-connector-local-fs/src/lib.rs`.

Note the `ff-ce-*` backends are **"light"**: their only FS-specific behaviour today is the `is_case_sensitive` default and a plain `std::fs::write`. Deep NTFS/POSIX/APFS emulation and cross-emulation are **deferred** (asserted by `deep_fs_emulation_and_cross_emulation_are_deferred` in `ff-ce-host-fs`). So the user's picture of rich "ntfs/posix controllers" is **aspirational** -- the abstraction exists, the deep semantics do not.

---

## 3. The editor file lifecycle: open -> read -> edit -> save

### Open / Read (uses `VfsProvider`, bypasses the registry)
`TabManager::open_file` in `crates/ff-desktop/src/tab_manager.rs`:
```rust
let provider = LocalFsProvider::with_defaults()?;   // constructed directly
let bytes = provider.read(path).await?;             // VfsProvider::read
```
The bytes are inserted into a `ff_document_model` document (`new_document()`, `doc.insert(...)`), and a `TabState::for_file(...)` is created. This path uses the `VfsProvider` trait but **does not** go through the shell's live `ProviderRegistry` / `Vfs` facade -- it instantiates the provider inline.

### Edit
Editing mutates the in-memory `ff_document_model` document (via the editor/document-model crates). No VFS involvement until save.

### Save (uses `BackendEnvironment`, the single SAVE-addressing seam)
This is the well-developed, framework-governed path. Call chain:

1. **Command dispatch** -> FFEDIT claims the SAVE verb. The active-environment gate routes through `dispatch_to_environment(FFEDIT_NAME, cmd)` (`crates/ff-desktop/src/shell/commands.rs`, `commands_environment.rs`).

2. **`WorkbenchShell::host_fs_save`** (`crates/ff-desktop/src/shell/dispatch_ffedit.rs`) -- the single SAVE-addressing seam:
   - Clean buffer (`!is_modified`) -> no-op (dirty-aware contract).
   - Resolves the backend by the **active tab's `owning_environment`**:
     - For a MAINFRAME tab with a `store_identity` whose catalog resolves to a repo path, builds a **per-save** mainframe backend (`mainframe_backend_for_active_tab` -> `mainframe_backend::build_mainframe_backend_for_repo`).
     - Otherwise falls back to `EnvironmentRegistry::backend_for(&owning)`, which yields the host-FS backend (`ff_ce_host_fs::native_backend_environment()`), byte-identical native save.
   - Calls `tabs.save_active_tab_via_backend(backend, runtime)`.

3. **`TabManager::save_active_tab_via_backend`** (`crates/ff-desktop/src/tab_manager.rs`) -- crosses the VFS boundary and performs the **byte-vs-record selection** (CR-CH-060 BRC.3):
   - Reads the document's **open-supplied** `RecordFormat` (`Document::record_format()`); the save does **not** re-derive framing.
   - **Selection rule:** record path is taken iff the format maps to a record format (Fixed/Variable) **AND** `backend.record_capable()`. Otherwise the **byte** path.
   - **Record path:** builds a `StoreTarget` (real DSN + catalog from `store_identity` for a MAINFRAME tab, else a host-path placeholder), wraps the framed records in a `RecordImageSource` (`&mut dyn RecordSource`), builds `RecordAttrs { recfm, lrecl, encoding }`, and calls `backend.save_records(...)`. `Stored { rc: 0 }` -> clear dirty; `Stored { rc != 0 }` -> error; `NotRecordCapable` -> defensive fall-through to the byte path.
   - **Byte path:** `backend.save(&path, &bytes)` with the raw contiguous document image -- byte-identical to the pre-CR-CH-060 native save.
   - Both paths end at `clear_dirty_after_save` (clears `is_modified`, sets the save point).

So the **VFS boundary on save is `BackendEnvironment::save` / `save_records`**, reached through the one `host_fs_save` -> `save_active_tab_via_backend` seam. The `ff-ce-*` host backends ultimately call `std::fs::write`; the mainframe CE writes records through `ff_dscatalog::DatasetAccess`.

### The record-aware vs byte save contract (CR-CH-060)
- `save(path, bytes)` -- the **byte** entry, **retained unchanged**. Host CEs (NTFS/POSIX) perform a plain OS byte write; native SAVE is byte-identical.
- `save_records(target, records, attrs)` -- the **additive** record-aware entry. Carries **dataset identity** (`StoreTarget`, not a host path), an **object-safe record stream** (`RecordSource`), and **RECFM/LRECL/encoding** (`RecordAttrs`). Has a provided default (`NotRecordCapable`) so host CEs compile unchanged; only a record-capable CE (the mainframe CE) overrides it. Confirmed by `MainframeEnvironment` (`ff-idcams/src/mainframe_env.rs`): `record_capable()==true`, overrides `save_records`, and its byte `save` is an error (a mainframe resource has no host-path byte write). The whole contract is kept **object-safe** so it works as `&dyn BackendEnvironment` in the registry.

---

## 4. How Command Environments relate to VFS backends

"Command Environment" and "VFS backend/controller" are **overlapping but distinct layers**. The CR-CH-053 model (documented in `crates/ff-vfs/src/backend_environment.rs` and `docs/specs/command-environments/`) splits Command Environments into **two categories**:

- **Interactive / context environments** -- `FFCMD` (base), `FFEDIT` (editor), `FFNAV` (navigator). The user *focuses* a Workspace Context that makes one active; verbs mutate shell state. These implement the **shell-side** `CommandEnvironment` trait (takes `&mut WorkbenchShell`), live in `ff-desktop` (`shell/environment*.rs`), and are registered in the shell's `EnvironmentRegistry` (`shell/environment_registry.rs`). **These are NOT VFS backends.**

- **Backend / resource environments** -- `ff-ce-ntfs`, `ff-ce-posix` (via the `ff-ce-host-fs` decider), and the mainframe CE (`ff-idcams`). **No user ever focuses these**; they are reached only by **addressing by name** (`dispatch_to_environment(name, cmd)`). These implement `ff-vfs::BackendEnvironment`. **These ARE the VFS "controllers" the user means -- on the save/store side.**

So: the **backend** Command Environments and the save-side VFS `BackendEnvironment` implementations are **the same objects** (a backend CE *is* a `BackendEnvironment` impl). The **interactive** Command Environments (FFEDIT etc.) are a **separate shell-side layer** that *addresses* SAVE to the owning backend CE. The relationship: FFEDIT (interactive) --addresses--> HOSTFS/MAINFRAME (backend CE = VFS store contract). There is **no central hub**: backend CEs register themselves by name into the shell's `EnvironmentRegistry`; the `BackendEnvironment` trait "knows about NO concrete backend" (module docs, `backend_environment.rs`).

Note this is a **third** meaning of "backend," separate from the `VfsProvider` scheme backends in section 2a. The read path (`VfsProvider`) and the save path (`BackendEnvironment`) are genuinely different trait hierarchies that happen to both live in `ff-vfs`.

---

## 5. Datasets and the mainframe path (ff-dscatalog / DatasetAccess)

- `DatasetAccess` is the object-safe trait in `crates/ff-dscatalog/src/dataset_access/trait_def.rs` (`pub trait DatasetAccess: Send + Sync`), the dataset-level allocate/resolve/open/close/store contract.
- The mainframe backend CE `ff_idcams::MainframeEnvironment` **owns** an `Arc<dyn ff_dscatalog::DatasetAccess>` (`mainframe_env.rs`). Its `save_records` stores each framed record through that `DatasetAccess`.
- `ff-vfs` deliberately does **not** depend on `ff-dscatalog`/`ff-volume` (acyclic DAG). That is exactly why `StoreTarget`/`RecordAttrs` are `ff-vfs`-local plain-data types carrying the dataset identity as strings; the **mainframe CE** maps them onto a real `DatasetAccess` (module docs in `backend_environment.rs`).
- Wiring (`crates/ff-desktop/src/shell/mainframe_backend.rs`): the CE's `DatasetAccess` is a catalog-backed `ff_dscatalog::CatalogRepositoryAccess` that consults the on-disk SQLite catalog + physical files the Files Panel's allocate writes. SAVE resolution is **per-save**: `host_fs_save` resolves the active MAINFRAME tab's `(dsn, catalog)` identity to its repository path and builds a backend pointed at that catalog.
- A `catalog`-scheme `CatalogVfsProvider` is **additively** registered in the live `ProviderRegistry` (`construct_provider.rs`, `mainframe_backend::build_catalog_provider`) -- but, per the same file, the host open/save path never consults that registry, so it is a seam for future use, not the active save route.

---

## 6. Corrections and implemented-vs-aspirational notes

**Corrections to the user's model:**

1. It is **not one unified pipeline**. There are two distinct `ff-vfs` seams: the async scheme-routed `VfsProvider`/`ProviderRegistry` (read/list/watch/search) and the sync name-addressed `BackendEnvironment` store contract (save). The editor uses `VfsProvider` to open and `BackendEnvironment` to save -- different traits, different dispatch mechanisms.

2. The live **host-path open and save bypass the `Vfs`/`ProviderRegistry` facade**. `open_file` builds a `LocalFsProvider` inline; `host_fs_save` addresses a `BackendEnvironment` by name. The registry built at startup is **additive** and "never consulted" by the native path (`construct_provider.rs`). So the design doc's picture of everything flowing through the `Vfs` facade is **not** how the host path currently works.

3. **Config files bypass the VFS -- confirmed, and this contradicts the stated `FFW-ARCH-001` principle.** The VFS design doc (`docs/specs/virtual-file-system/design.md`) and `ff-vfs/src/lib.rs` both assert "no consuming crate ever calls `std::fs`." In reality `ff-config`, `ff-session`, and many `ff-desktop` loaders (menus, themes, session, keymaps, command store) call `std::fs` directly. The user's instinct ("apart from its own configuration files") is correct; the codebase's own ARCH-001 wording is aspirational/overstated.

4. The "native/posix/ntfs controllers" are the **backend Command Environments** (save side), and "native" is a **role resolved by a decider** (`ff-ce-host-fs`), not a standalone FS. They are **light** today -- only a case-sensitivity default plus a byte write; deep per-FS emulation is explicitly deferred.

**Implemented today:**
- `VfsProvider` trait + `ProviderRegistry` + `Vfs` facade + `ResourceUri`; `LocalFsProvider` (scheme `local`) with read/write/watch/streaming.
- `BackendEnvironment` byte + record-aware contract (CR-CH-060) with all supporting types, fully tested.
- `ff-ce-host-fs` decider + `ff-ce-ntfs` + `ff-ce-posix` light host backends.
- The one SAVE-addressing seam (`host_fs_save` -> `save_active_tab_via_backend`) with byte-vs-record selection.
- `ff-idcams::MainframeEnvironment` record-capable CE over catalog-backed `DatasetAccess`; per-save catalog resolution; additive `catalog`-scheme provider registration.

**Planned / aspirational / seams-only:**
- Routing native open/save **through** the `Vfs`/`ProviderRegistry` facade (today it is bypassed; the registry is an additive plugin seam).
- Deep NTFS/POSIX/APFS semantics and cross-emulation (deferred, Req 16.6).
- A configurable "addressable environments" Context managing the open registry (noted as a pending owner idea in `backend_environment.rs`).
- FFNAV as a registered claiming environment (today it degrades to the FFCMD base).
- Connector crates for FTP/SFTP, network FS, cloud, mainframe transport (spec folders exist; only local-fs + catalog providers are wired).

**Recommendations (no code changed):**
- Reconcile the `FFW-ARCH-001` "no `std::fs`" claim with reality: either carve out an explicit, documented exception for config/session/bootstrap files (recommended -- matches the user's mental model), or route them through the VFS. The current docs overstate the invariant.
- Document the **two-seam** model (async `VfsProvider` read side vs sync `BackendEnvironment` save side) prominently, since the single-facade design doc diagram is misleading about the live host path.
- If a unified pipeline is desired long-term, consider having `open_file` resolve through the live `ProviderRegistry` rather than constructing `LocalFsProvider` inline, so the read path and plugin providers converge.

---

## Appendix: key files read
- `crates/ff-vfs/src/lib.rs`, `backend_environment.rs`, `provider.rs`, `vfs.rs`
- `crates/ff-ce-host-fs/src/lib.rs`, `crates/ff-ce-ntfs/src/lib.rs`, `crates/ff-ce-posix/src/lib.rs`
- `crates/ff-connector-local-fs/src/lib.rs`
- `crates/ff-idcams/src/mainframe_env.rs`, `mainframe_env_tests.rs`
- `crates/ff-dscatalog/src/dataset_access/trait_def.rs`
- `crates/ff-desktop/src/tab_manager.rs` (`open_file`, `save_active_tab_via_backend`)
- `crates/ff-desktop/src/shell/dispatch_ffedit.rs` (`host_fs_save`, `mainframe_backend_for_active_tab`)
- `crates/ff-desktop/src/shell/environment_registry.rs`, `construct_provider.rs`, `mainframe_backend.rs`, `commands.rs`
- `crates/ff-config/src/audit.rs`, `crates/ff-session/src/cli.rs`, `crash_recovery.rs`
- `docs/specs/virtual-file-system/design.md`
- `.kiro/steering/framework-conformance.md` (CR-CH-060)

Content was rephrased from source comments/docs for compliance with licensing restrictions.
