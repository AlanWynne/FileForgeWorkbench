# RC.B.8 Part 2 -- Record-aware MAINFRAME editor SAVE: STOP-AND-REPORT

**Feature:** FEAT-002 (CR-CH-059, RC.B.8).
**Type:** docs-only finding. NO `.rs` behaviour change. Native/host SAVE stays
byte-identical.
**Determination:** The record-aware MAINFRAME editor SAVE is NOT achievable by
wiring a `BackendEnvironment` sibling at the existing seam. It requires a
FRAMEWORK CHANGE this CR does not fund, plus a larger
editor/registry/provider build. Part 2 therefore RECORDS the finding and the
exact prerequisites, DEFERS the build, and leaves native SAVE untouched.

This note was authored from the live code (re-confirmed by reading the files
below), not in the abstract.

---

## The five code-grounded evidence points

### 1. The `BackendEnvironment` contract is a BYTE write, not record-aware

`crates/ff-vfs/src/backend_environment.rs`:

```rust
fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
```

A record-aware save through `ff_dscatalog::DatasetAccess` needs the DSN, the
dataset's RECFM/LRECL/catalog identity, and `Record` boundaries -- NONE of which
`(path, bytes)` carries. The trait's own doc comment acknowledges a mainframe
backend "would instead pack records per RECFM/LRECL", but the SIGNATURE gives it
only a path and an opaque byte slice. Making the MAINFRAME CE route records
through `DatasetAccess` would require CHANGING this `ff-vfs` trait signature (or
adding a record-aware addressing variant alongside it). `ff-vfs::BackendEnvironment`
is a load-bearing core framework type, so altering it is a FRAMEWORK CHANGE
requiring express owner confirmation (framework-conformance mechanism).

### 2. The registry is CLOSED to arbitrary named backends

`crates/ff-desktop/src/shell/environment_registry.rs`: `EnvironmentRegistry`
holds a closed `enum RegisteredEnv { FfCmdBase, FfEdit, HostFsPlaceholder }`
plus a SINGLE boxed backend `host_fs: Box<dyn ff_vfs::BackendEnvironment>`
seeded with `ff_ce_host_fs::native_backend_environment()`. There is no map of
named boxed backends and no "MAINFRAME" entry or constant.

`crates/ff-desktop/src/shell/commands_environment.rs`:
`dispatch_to_environment` matches that closed enum and only the
`HostFsPlaceholder` arm performs a `save` (via `host_fs_save`); `FfCmdBase` and
`FfEdit` do not save. Addressing SAVE to a MAINFRAME backend needs this registry
+ dispatcher EXTENDED to resolve and invoke additional named boxed backends.

### 3. A mainframe-catalog dataset is NOT bound to a MAINFRAME environment today

`crates/ff-desktop/src/shell/render_body.rs::open_mainframe_dsn` resolves the
DSN to a HOST PATH and returns it as a `String`. In
`crates/ff-desktop/src/shell/render_body_arms.rs` (and the mirror in
`render_nav.rs`) that path is then opened via:

```rust
let mut p = ff_command::CommandParams::new();
p.insert("path", path_str.as_str());
let _ = self.dispatch.execute_command("file.open", p);
```

Only `path` is passed -- NO `owning_env`. So the tab binds to
`DEFAULT_OWNING_ENVIRONMENT = "HOSTFS"` and SAVE already (correctly) goes to the
host-FS byte write. There is no live code path that produces a MAINFRAME-owned
editable tab to save, which is also why the "first-fail" test for a future slice
(a MAINFRAME-owned dataset SAVE driving `DatasetAccess::put`) cannot be written
today -- no MAINFRAME-owned tab exists to assert against.

### 4. The provider prerequisite (Req 17) is only partially met

`crates/ff-desktop/src/shell/construct_provider.rs::build_live_provider_registry`
registers the `ff-vfs` `ProviderRegistry` LIVE at startup (Req 17.1 met) but
seeds it ONLY with the host-FS `local` provider (`LocalFsProvider`). The
mainframe VFS provider is NOT registered. command-environments Req 17.2/17.4
state the non-host parts of Req 14-16 DEPEND ON a mainframe provider being
registered -- not yet done. The host-path open/save path reads through
`LocalFsProvider` / `BackendEnvironment` directly and never consults this
registry (Req 17.3), which is why native access is unaffected either way.

### 5. The spec itself defers this

command-environments `docs/specs/command-environments/requirements.md`:

- Req 16 Marking: "the mainframe CE (housed in `ff-idcams`) is the first CE that
  genuinely diverges and is built in a later phase."
- Req 16.1 / 16.4: the mainframe CE is HOUSED IN `ff-idcams`, NOT a dedicated
  `ff-ce-mainframe` sibling crate. A file search confirms no `ff-ce-mainframe`
  crate exists, so even the "sibling to ff-ce-ntfs/ff-ce-posix/ff-ce-host-fs"
  crate shape some notes imply contradicts the spec's "housed in ff-idcams" and
  would itself be an owner design decision.
- Req 17 is explicitly a PREREQUISITE/dependency note, flagged per the
  DESIGN-BRIEF section 5 "NON-NEGOTIABLE PREREQUISITE".

---

## Prerequisites a future slice needs (the deferred build)

A future owner-confirmed slice to deliver record-aware MAINFRAME SAVE must land
ALL of the following:

- **(a) Record-aware store contract (FRAMEWORK CHANGE -- needs express owner
  confirmation).** Either change `ff-vfs::BackendEnvironment::save` from
  `(path, bytes)` to a record-aware signature, OR add a record-aware addressing
  variant alongside it that carries the DSN + dataset attributes (RECFM/LRECL/
  catalog identity) + `Record` boundaries -- NOT `(path, bytes)`. Because
  `ff-vfs::BackendEnvironment` is a load-bearing core framework type, this is a
  FRAMEWORK CHANGE and MUST NOT be made without express owner confirmation.
- **(b) Open named-backend registry + dispatch arm.** Replace/extend the closed
  `RegisteredEnv` enum + single `host_fs` backend with an OPEN map of named
  boxed backends, and add a `dispatch_to_environment` arm that resolves and
  invokes a "MAINFRAME" named backend's store write.
- **(c) Bind the tab to the mainframe environment.** `open_mainframe_dsn` (and
  its `render_body_arms.rs` / `render_nav.rs` callers) must pass
  `owning_env = "MAINFRAME"` into `file.open` so the tab binds to the mainframe
  environment instead of defaulting to HOSTFS. (The editor nav currently rejects
  mainframe dataset editing as "available in a later update".)
- **(d) Register the mainframe VFS provider live (Req 17.4).**
  `build_live_provider_registry` must additionally register the mainframe VFS
  provider so a non-host Owning_Environment can read/write its store.
- **(e) Implement the mainframe CE in `ff-idcams` (Req 16.1/16.4).** The
  mainframe Command Environment is HOUSED IN `ff-idcams` and implements the
  record-aware contract from (a) over `ff_dscatalog::DatasetAccess`
  (allocate/open/get/put/point/close/dispose) -- not a new `ff-ce-mainframe`
  crate unless the owner expressly decides otherwise.

---

## Why Part 2 is safe (no regression)

- NO `.rs` behaviour change: native/host SAVE stays byte-identical; no new crate;
  no `ff-vfs` trait change; no registry/dispatcher change.
- The guard is the NEGATIVE evidence above plus the existing host-save tests,
  which must remain green and prove native SAVE is untouched:
  - `crates/ff-desktop/src/tab_manager.rs` -- `save_active_tab_via_backend` tests
    (TestBackend plain byte write).
  - `crates/ff-desktop/src/shell/tests_session.rs` --
    `host_path_open_is_unchanged_by_the_live_provider_registry`.
- Verification run for this FEAT: `cargo test -p ff-desktop` (scoped, via the
  pwsh7 non-interactive wrapper).

## The decision requested of the owner

STOP-and-report. The owner decides between:

1. **Accept the deferral** and proceed to FEAT-003 (Part 3: delete
   `ff-dataset-catalog` + re-express governance). Part 2 is recorded as deferred
   so FEAT-003 and the merge step do not treat Part 2 as built; Req 28 itself is
   satisfied by Part 1 (the repoint) -- the MAINFRAME SAVE is a COMPOSING concern
   recorded as deferred, not a 28.x criterion.
2. **Open a separate owner-confirmed FRAMEWORK-change slice** to build the
   record-aware MAINFRAME SAVE, carrying prerequisites (a)-(e) above, with (a)
   expressly confirmed as a framework change.
