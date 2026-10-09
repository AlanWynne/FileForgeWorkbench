# ff-dscatalog vs ff-dataset-catalog -- Duplicate-Crate Investigation

READ-ONLY investigation. No source was changed. Workspace root:
`c:\workspace\VSC\FileForgeWorkbench`.

---

## Summary answer (read this first)

The two crates are NOT near-duplicates and this is NOT a simple "stale copy"
situation.

- **`ff-dscatalog` is the LIVE crate.** It holds the full SQLite-backed mainframe
  dataset-catalog implementation (24 source modules) and is in the shipping
  `ffwb` dependency closure, consumed by `ff-desktop`, `ff-catalog-registry`,
  `ff-catalog-dialog`, and `ff-files-panel`. Last commit 2026-09-30.

- **`ff-dataset-catalog` is a trait-only governance fixture crate (2 files), NOT
  a shipping crate.** It declares a `CatalogService` / `DynCatalogService`
  interface plus data/error types, implements no catalog logic, and is referenced
  by exactly ONE consumer: `ff-governance-tests` as a **dev-dependency** (in
  `tests/mock_compilation.rs`). Last commit 2026-08-16.

Because `ff-dataset-catalog` still has a live (test-only) consumer AND is still
named as the catalog owner by a governance spec (ADR-001,
`dataset-ownership-model`), **removing it is NOT safe as a drop-in delete.** It
is removable, but only as a gated, owner-confirmed consolidation (update the
governance tests to target `ff-dscatalog` and fix ADR-001 first). Deleting the
directory and its member line today would break `cargo test -p
ff-governance-tests`.

A prior, deeper investigation reached the same conclusion:
`.agents/tasks/mainframe-dataset-emulation/architecture-revision-findings.md`
(Q1-Q3) -- "DELETE `ff-dataset-catalog` ... Nothing in the shipping closure
imports it ... would break ONLY `ff-governance-tests` (one dev file)."

---

## Per-crate fact table

| Fact | `ff-dscatalog` | `ff-dataset-catalog` |
|------|----------------|----------------------|
| Package name | `ff-dscatalog` | `ff-dataset-catalog` |
| Root workspace member? | YES (`Cargo.toml` line 61) | YES (`Cargo.toml` line 58) |
| Excluded? | No | No |
| In shipping ffwb closure? | YES (via ff-desktop) | NO (orphan; dev-dep only) |
| Depended on by (shipping) | ff-desktop, ff-catalog-registry, ff-catalog-dialog, ff-files-panel | (none) |
| Depended on by (test) | -- | ff-governance-tests (dev-dependency) |
| Source module count | 24 modules + `codecs/` (5) + `storage/` (6) | 2 files (`lib.rs`, `traits.rs`) |
| Contains real implementation? | YES (SQLite catalog, DSN, PDS, GDG, VSAM/ISAM backends, VFS provider, codecs, audit, transactions, security) | NO -- trait + type declarations only; the only impl is `Display` + a blanket forwarding impl |
| Dependencies | ff-vfs, ff-command, ff-config, ff-logging, rusqlite, tokio, async-trait, uuid, zip, walkdir, percent-encoding, chrono, serde | `thiserror` only (+ dev proptest, pretty_assertions) |
| Last commit date | 2026-09-30 21:45 +0200 | 2026-08-16 16:37 +0200 |
| Uncommitted changes? | None (git status --porcelain empty) | None |

---

## Evidence by question

### Q1. Cargo.toml + module listing

**`ff-dscatalog/Cargo.toml`** -- `name = "ff-dscatalog"`,
`version.workspace = true`. Rich dependency set: `ff-vfs`, `ff-command`,
`ff-config`, `ff-logging`, `rusqlite 0.31` (bundled), `tokio`, `async-trait`,
`serde`/`serde_json`, `zip`, `walkdir`, `percent-encoding`, `uuid`, `chrono`,
`futures-core`.

`ff-dscatalog/src` modules (from `lib.rs` and directory listing):
`audit, catalog, catalog_registry, codecs/, commands, config, context_menu,
dataset, dsn, encoding, error, gdg, hierarchy, integrity, listcat, pds,
properties, repository, schema, security, storage/, transactions, vfs_provider`.
`codecs/` = `binary, fixed, text, variable, mod`. `storage/` = `esds, isam,
native, rrds, sqlite_record, mod`. The crate doc-comment describes "mainframe
dataset filesystem emulation on the local desktop ... SQLite-backed catalog
database."

**`ff-dataset-catalog/Cargo.toml`** -- `name = "ff-dataset-catalog"`. Depends
ONLY on `thiserror` (+ dev `proptest`, `pretty_assertions`).

`ff-dataset-catalog/src` modules: exactly two files -- `lib.rs` (doc +
`pub mod traits; pub use traits::*;`) and `traits.rs`. `traits.rs` declares
`CatalogError`, `DsnValidationError`, `DatasetId`, `Dsorg {Ps, Po, Da, Vsam}`,
`Recfm {F, Fb, V, Vb, U}`, `DatasetAttributes`, `ResolutionResult`,
`DatasetEntry`, `DatasetFilter`, `GenerationInfo`, and the traits
`CatalogService` + `DynCatalogService` (object-safe, with a blanket
`impl<T: CatalogService> DynCatalogService for T`). It implements NO concrete
catalog; its only test asserts `Box<dyn DynCatalogService>` compiles. The crate
doc-comment claims to be "the single authority for dataset metadata" per ADR-001.

### Q2. Dependency references (grep of all Cargo.toml)

`ff-dscatalog` is a dependency of (main workspace):
- `crates/ff-desktop/Cargo.toml` -> `ff-dscatalog = { path = "../ff-dscatalog" }`
- `crates/ff-catalog-registry/Cargo.toml` -> `ff-dscatalog`
- `crates/ff-catalog-dialog/Cargo.toml` -> `ff-dscatalog`
- `crates/ff-files-panel/Cargo.toml` -> `ff-dscatalog`

`ff-dataset-catalog` is a dependency of:
- `crates/ff-governance-tests/Cargo.toml` -> under `[dev-dependencies]`:
  `ff-dataset-catalog = { path = "../ff-dataset-catalog" }` (NOT a normal
  dependency). No other crate references it.

Source-level `use`: `use ff_dataset_catalog::...` appears ONLY in
`crates/ff-governance-tests/tests/mock_compilation.rs` (imports `CatalogService,
DynCatalogService, DatasetAttributes, Dsorg, ...` to build a `MockCatalogService`
proving trait-based coupling). `use ff_dscatalog::...` appears across
`ff-catalog-registry` and `ff-files-panel` using concrete types (`Catalog`,
`CatalogMount`, `dataset::*`, `dsn`, `repository`, etc.).

### Q3. Reachability from the ffwb binary closure

`ff-desktop` (the `ffwb` binary crate) directly depends on `ff-dscatalog`, and
transitively through `ff-catalog-registry` / `ff-catalog-dialog` /
`ff-files-panel`. So **`ff-dscatalog` is wired into the live app.**

`ff-dataset-catalog` is reached by NO member of the ff-desktop closure. Its only
reverse edge is `ff-governance-tests`' dev-dependency, and `ff-governance-tests`
itself is `publish = false` and not a dependency of ff-desktop. So
**`ff-dataset-catalog` is an orphan relative to the shipping binary** (a
test-only governance fixture).

### Q4. Root workspace membership

Root `Cargo.toml [workspace].members` lists BOTH:
- line 57 `"crates/ff-dataset-catalog"`
- line 61 `"crates/ff-dscatalog"`

Neither is in an `exclude` list; there is no `exclude` key. Both compile as
workspace members.

### Q5. Git staleness signals (no mutation)

Captured via the pwsh7 non-interactive wrapper to `tools/logs/` and read back:
- `git log -1 --format=%ci -- crates/ff-dscatalog` -> **2026-09-30 21:45:15 +0200**
- `git log -1 --format=%ci -- crates/ff-dataset-catalog` -> **2026-08-16 16:37:08 +0200**
- `git status --porcelain crates/ff-dscatalog crates/ff-dataset-catalog` ->
  **empty** (no uncommitted or untracked changes in either crate).

`ff-dscatalog` is ~6 weeks newer. `ff-dataset-catalog` is older but NOT abandoned
in the "orphaned by accident" sense -- it is intentionally a stable trait fixture
that rarely changes.

### Q6. High-level content comparison

Not near-duplicates. `ff-dscatalog` contains the real implementation mapping to
dataset-catalog spec tasks 1-32: DSN parsing/validation (`dsn.rs`), repository
layout (`repository.rs`), catalog DB + schema (`catalog.rs`, `schema.rs`,
`catalog_registry.rs`), CRUD + mount/unmount (`catalog.rs`), PDS members
(`pds.rs`), GDG (`gdg.rs`), VFS provider (`vfs_provider.rs`, scheme `catalog`),
LISTCAT/LISTDS (`listcat.rs`), StorageProvider backends + VSAM/ISAM
(`storage/{native,esds,sqlite_record,rrds,isam}.rs`), record codecs (`codecs/`),
staged transactions (`transactions.rs`), audit (`audit.rs`), integrity/reconcile
(`integrity.rs`), security scrubbing (`security.rs`).

`ff-dataset-catalog` is an interface shell: trait + DTOs, no storage, no I/O, no
runtime deps. The two even define INCOMPATIBLE enums -- `ff-dataset-catalog`
`Dsorg {Ps, Po, Da, Vsam}` / `Recfm {F, Fb, V, Vb, U}` vs `ff-dscatalog`
`Dsorg {PS, PO, GDG}` / `Recfm {F, FB, V, VB, U}` -- so they are not two copies
of one thing but two different models of "the catalog."

### Spec history confirming the LIVE crate

`docs/specs/dataset-catalog/requirements.md` line 36:
> Crate name: The actual workspace crate is `ff-dscatalog`. Any reference to
> `ff-dataset-catalog` in older documents is incorrect and should be read as
> `ff-dscatalog`.

`docs/specs/dataset-catalog/tasks.md` tasks 31.3 / 31.4 (both `[x]`) renamed all
in-spec references from `ff-dataset-catalog` to `ff-dscatalog` "to match the
actual workspace crate name." The spec authority is unambiguously `ff-dscatalog`.

Counter-signal: the GOVERNANCE spec
`docs/specs/dataset-ownership-model/requirements.md` (ADR-001) and
`ff-dataset-catalog/src/lib.rs` still name `ff-dataset-catalog` as the dataset
authority. This stale governance pointer is the reason the trait crate still
exists and still has a (test) consumer.

---

## VERDICT

- **LIVE crate: `ff-dscatalog`.** Full implementation, in the ffwb shipping
  closure (ff-desktop + 3 catalog/UI crates), most recently committed, and named
  authoritative by the dataset-catalog spec (incl. the completed 31.3/31.4
  rename).

- **`ff-dataset-catalog`: a redundant, orphan-to-the-binary trait fixture.** It
  is not a stale copy of `ff-dscatalog`; it is a separate trait-only crate used
  only by `ff-governance-tests` (dev-dependency) and still cited by the stale
  ADR-001 governance text. It carries no shipping code.

**Removal is NOT a safe drop-in delete today**, because (1) `ff-governance-tests`
dev-depends on it and imports its traits/types in `tests/mock_compilation.rs`,
and (2) a governance spec (ADR-001) still names it as the catalog owner. Deleting
the directory + member line now would break `cargo test -p ff-governance-tests`
and deepen the spec contradiction.

---

## RECOMMENDED removal procedure (gated consolidation, owner-confirmed)

This matches the phased plan already drafted in
`.agents/tasks/mainframe-dataset-emulation/architecture-revision-findings.md`
(Q3, Q7). It is a FRAMEWORK/governance change per
`.kiro/steering/framework-conformance.md` (removing a public crate + amending a
governance ADR), so it requires the requirements gate and explicit owner
confirmation -- do NOT perform it as part of this read-only task.

1. **Spec reconciliation (docs only, gated).** Amend ADR-001
   (`docs/specs/dataset-ownership-model/requirements.md`) to name `ff-dscatalog`
   as the catalog authority; mark `ff-dataset-catalog` as deprecated-for-merge.
2. **Add a reconciled `CatalogService` / `DynCatalogService` trait to
   `ff-dscatalog`** using `ff-dscatalog`'s own types (`Dsorg {PS,PO,GDG}`, `Recfm`,
   `Dsn`). Additive; nothing breaks.
3. **Repoint the only consumer.** Update
   `crates/ff-governance-tests/Cargo.toml` dev-dependency and
   `crates/ff-governance-tests/tests/mock_compilation.rs` from
   `ff_dataset_catalog` to the new `ff-dscatalog` trait; adjust the mock to the
   reconciled types. Verify `cargo test -p ff-governance-tests`.
4. **Delete the crate once no consumer references it:**
   - Remove directory `crates/ff-dataset-catalog/`.
   - Remove the member line `"crates/ff-dataset-catalog",` from the root
     `Cargo.toml` (`[workspace].members`, line 58).
   - Grep-verify no remaining `ff-dataset-catalog` / `ff_dataset_catalog`
     references (spec text, Cargo.toml, `.rs`).

If the owner does not want the consolidation, the safe alternative is to LEAVE
`ff-dataset-catalog` in place and instead fix ADR-001's wording to acknowledge
`ff-dscatalog` as the implementation, documenting the trait crate as the
governance interface fixture. Either way, nothing should be deleted without first
removing the `ff-governance-tests` dependency.

---

## Note on method

Git facts were gathered with the pwsh7 non-interactive wrapper, redirected to
`tools/logs/` and read back (the shell reported `Exit Code: -1` from PSReadLine
mangling, but the log files were written correctly, confirming the commands ran).
No `cargo build`/`test` was run; no file was edited or deleted.
