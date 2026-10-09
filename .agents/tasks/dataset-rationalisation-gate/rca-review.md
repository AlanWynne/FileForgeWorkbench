# RC.A review -- Mainframe Dataset Stack Rationalisation (CR-CH-059)

The RC.A working-tree change lands the additive-first core of the dataset-stack
consolidation: it adds reconciled `CatalogService`/`VsamService` traits (over
ff-dscatalog's own single `Dsorg`/`Recfm`), unifies the five physical storage
backends onto the one `ff-vfs::StorageProvider` seam while deleting the duplicate
`ff-dscatalog::storage::StorageProvider` trait, repoints the governance tests off
the deprecated trait crates, and collapses the two `posix` registrants to one.
Every deletion except the duplicate trait is deferred to RC.B as the plan
requires, so FFWB builds at each step and no crate disappears. The scoped checks
were run and recorded by the coder.

Watch for: one of the two `ff-dataset-catalog` doc-comment mentions RC.A.1 step 3
was told to fix remains unfixed (`catalog_bridge.rs:83`), and the verification
note incorrectly claims both were corrected (confirmed). Two record backends
(`esds.rs`, `sqlite_record.rs`) are over the 400 non-test line rule, but both
were already over at HEAD -- pre-existing, correctly disclosed, out of RC.A's
additive scope. Neither is blocking.

**Verdict**: APPROVED

## High-level view

The additive-first ordering holds exactly as planned. RC.A.2 adds the reconciled
`CatalogService` + object-safe `DynCatalogService` (blanket impl) and the
object-safe `VsamService` in new files `service.rs` / `vsam_service.rs`, built
over ff-dscatalog's own `Dsn`/`Dsorg`/`Recfm`. RC.A.3 then reshapes the five
backends onto `ff-vfs::StorageProvider` and removes the duplicate trait; because
A.3 depends on A.2 being present and the only deletion is the trait (never a
crate), the tree compiles at every step and the deprecated crates plus their root
`Cargo.toml` member lines survive.

VSAM is modelled correctly as a cluster entity. `VsamCluster` carries a
`VsamType {Ksds,Esds,Rrds,Lds}` and the tests prove by exhaustive match that
`Dsorg` is still exactly `{PS,PO,GDG}` with no `Vsam`/`Da` variant. The divergent
`ff-dataset-catalog` `{Ps,Po,Da,Vsam}` / `{F,Fb,V,Vb,U}` set is neither imported
nor re-exposed; the reconciled DTOs use only the ff-dscatalog enums.

The physical seam is genuinely single. The duplicate `trait StorageProvider`,
`ProviderCapability`, `ObjectId`, and `ObjectStat` are gone from ff-dscatalog
(only a doc-comment references the old names), capabilities map 1:1 onto
`ff-vfs::StorageCapability`, the former `workspace_root`/UUID is carried behind
each backend struct, and `CatalogError` is mapped to `VfsError` at the boundary.
The live registry (`construct_provider.rs`) still registers only `local` and
never named the deleted trait, so nothing downstream broke.

The posix collapse is clean. `ff-posix-provider` loses its own `VfsProvider`
impl and becomes path helpers plus `pub use ff_vfs::PosixNativeProvider as
PosixProvider`, so it can no longer register scheme `posix`; the four ad-hoc
ff-desktop nav call sites now construct the single provider, and the registry's
`DuplicateScheme` guard plus a dedicated test enforce the one-registrant
invariant. No crate was deleted here either.

The governance repoint follows the dscatalog-duplicate procedure: the dev-dep
swaps to `ff-dscatalog`, `mock_compilation.rs` imports the reconciled traits and
types, and `architecture_compliance.rs` is left untouched (correct -- the crates
still exist in RC.A). TDD evidence is present and the TCR is flipped with
surgical accuracy: RC.A-delivered criteria go PASS with named tests, while the
RC.B deferrals (crate deletion 35.2, VSAM-under-DatasetAccess 35.3, the standing
guard 35.5) correctly stay NOT COVERED.

<details>
<summary>Issues (2)</summary>

1. **catalog_bridge doc comment half-fixed (confirmed, non-blocking)** --
   `crates/ff-dsalloc/src/catalog_bridge.rs:83` still reads "Production
   implementation delegates to `ff-dataset-catalog`"; RC.A.1 step 3 required both
   mentions corrected to `ff-dscatalog` and the verification note claims both were
   done. Fix the line (and note the pre-existing `lib.rs:26` mention) in a
   follow-up; documentation-only, no build or behaviour impact.
2. **Two record backends over the 400-line rule (confirmed, pre-existing)** --
   `esds.rs` (652 total at HEAD) and `sqlite_record.rs` (968 total at HEAD)
   exceed the 400 non-test line limit; RC.A only added a `root` field and the
   trait reshape, it did not create the violation. Schedule the split as a
   follow-up refactor outside RC.A's additive scope.

</details>

<details>
<summary>Details</summary>

### ADDITIVE-FIRST and the single deletion (acceptance point 1)

`git grep 'trait StorageProvider' -- crates/ff-dscatalog/` returns nothing: the
duplicate trait is gone. `ProviderCapability` survives only in the `storage/mod.rs`
module doc comment that explains the retirement, not as a type. The five backends
(`native`, `esds`, `sqlite_record`, `rrds`, `isam`) now carry `impl
StorageProvider for X` where the imported `StorageProvider` is `ff_vfs`'s. The
deprecated crates are intact -- `git grep` of the root `Cargo.toml` shows
`crates/ff-dataset-catalog` (line 57) and `crates/ff-vsam-services` (line 58)
still present. This matches the plan's rule that the only RC.A deletion is the
duplicate trait.

### Dependency order and single Dsorg/Recfm (points 2, 3)

`service.rs` imports `crate::dataset::{Dsorg, Recfm}` and `crate::dsn::Dsn` only;
`DatasetAttributes` carries `Option<Dsorg>` / `Option<Recfm>` from that single
pair. `vsam_service.rs` defines `VsamCluster { dsn, vsam_type: VsamType, params }`
and the test `vsam_cluster_carries_type_and_is_not_a_dsorg_variant` performs an
exhaustive `match dsorg { Dsorg::PS | Dsorg::PO | Dsorg::GDG => {} }`, which would
fail to compile if a `Vsam`/`Da` variant had been added. The reconciled traits
(A.2) are a prerequisite the seam work (A.3) builds on, and the storage reshape
references the same enums -- order held.

### The ff-vfs seam reshape (point 1 continued, point 6 framework)

`native.rs` shows the pattern applied to all backends: `capabilities()` returns a
`HashSet<StorageCapability>` built from the 1:1-mapped variants; `allocate` /
`open` / `stat` / `rename` / `delete` / `list` / `write` / `reconcile` take the
opaque `StorageLocator` and return `VfsError`; the former workspace root is a
`root` field captured at construction (`with_root`), so no method threads it. The
`native_tests.rs` seam tests assert the backend is usable as `Box<dyn
ff_vfs::StorageProvider>`, that an allocate/write/open round-trips without the DSN
leaking into the locator, and that a traversal locator maps to `VfsError::Io` via
`From<CatalogError>`. `isam.rs` delegates the seam to its inner KSDS provider,
which is a reasonable way to keep the root behind one struct. Codecs are
untouched. `construct_provider.rs` is unchanged and still seeds only the host
`local` provider, confirming the deleted trait had no live registry consumer.

### Single posix registrant (point 5)

`ff-posix-provider/src/lib.rs` is reduced to `resolve_posix_path` /
`to_posix_path` plus `pub use ff_vfs::PosixNativeProvider as PosixProvider`; its
own `VfsProvider` impl is removed, so it cannot self-register scheme `posix`. The
single-registrant test
`posix_provider::tests::posix_native_provider_is_sole_posix_registrant` in ff-vfs
registers one provider, asserts a second `posix` registration returns
`VfsError::DuplicateScheme { scheme: "posix" }`, and that `list_schemes` reports
exactly one `posix`. The registry's `DuplicateScheme` variant and `list_schemes`
are present (`registry.rs`). The four ad-hoc ff-desktop nav call sites in
`render_nav.rs` / `render_nav_expand.rs` now build the single infallible provider
via the re-export, and the `local` provider is untouched.

### Governance repoint and TDD evidence (points 4, 7)

`ff-governance-tests/Cargo.toml` dev-deps now carry `ff-dscatalog` in place of the
two trait crates, with a CR-CH-059 note. `mock_compilation.rs` imports the
reconciled `CatalogService`, `DynCatalogService`, `VsamService`, `VsamType`,
`VsamParams`, `Record`, etc. from `ff_dscatalog`, and the mocks match the
reconciled signatures (e.g. `get_allocation_defaults(&self, _dsorg: Dsorg)`).
`architecture_compliance.rs` is correctly left alone. Tests across `service.rs`,
`vsam_service.rs`, `native_tests.rs`, `posix_provider.rs`, and
`mock_compilation.rs` carry `// Validates: Requirement X.Y` lines and cover
object-safety + `Box<dyn>`, each-backend-via-ff-vfs, the compile-level
duplicate-trait removal, and the single-posix-registrant invariant. The TCR diff
flips the RC.A criteria (22.1-22.7, 13.1-13.5, 14.1-14.4, 33.1-33.7, 35.1, 35.4)
to PASS with the specific test names and keeps the RC.B deferrals (35.2, 35.3,
35.5) at NOT COVERED, which is exactly right. Task checkboxes 38/38.1-38.4,
39/39.1-39.3 (dataset-catalog) and 17/17.1-17.3 (virtual-file-system) are flipped
to `[x]`.

### Verification discipline

Per the brief I read the coder's evidence rather than re-running the suites. The
`rca-verification.md` note records `cargo fmt` (exit 0), `cargo check` across the
six crates (exit 0), `cargo clippy -p ff-dscatalog -p ff-vfs -- -D warnings`
(clean), `cargo test -p ff-dscatalog` (262 pass), `cargo test -p ff-vfs -p
ff-posix-provider` (pass incl. the single-registrant test), `cargo test -p
ff-governance-tests` (architecture 10 + mock 7 pass), and `cargo test -p
ff-desktop` (998 pass, 4 failures). The four ff-desktop failures are the
pre-existing B048 shared-env-var flake (config/history/theme persistence racing
under multithreaded test), re-confirmed green single-threaded; per the brief this
is not counted against the change. I ran only narrow read-only spot-checks
(git-grep for the deleted trait and member lines, line counts of the two record
backends at HEAD, presence of `DuplicateScheme`/`list_schemes`) to resolve
specific doubts.

### The one doc miss (finding 1)

`git grep 'ff-dataset-catalog' -- crates/ff-dsalloc/` returns two hits:
`catalog_bridge.rs:83` ("Production implementation delegates to
`ff-dataset-catalog`") and the pre-existing `lib.rs:26`. The diff shows the coder
corrected the file-header mention (line 4) and did ASCII cleanup, but not the
trait doc comment at line 83. RC.A.1 step 3 explicitly named "the two
doc-comment mentions", and the verification note states both were fixed, so this
is a genuine (if minor) deviation. It is documentation-only -- no type, build, or
runtime behaviour depends on it -- so it does not block the gate, but it should be
cleaned up in a follow-up along with the pre-existing `lib.rs:26` reference.

</details>

<details>
<summary>File map</summary>

RC.A-relevant changes (the working tree also carries unrelated simplify-splits
shell work, which is outside this review's scope):

- `crates/ff-dscatalog/src/service.rs` (new) -- reconciled `CatalogService` +
  object-safe `DynCatalogService` + DTOs + tests.
- `crates/ff-dscatalog/src/vsam_service.rs` (new) -- object-safe `VsamService`,
  `VsamCluster`/`VsamType`, `StubVsamService`, tests.
- `crates/ff-dscatalog/src/lib.rs` -- declare + re-export the two modules.
- `crates/ff-dscatalog/src/storage/mod.rs` -- duplicate trait + support types
  removed; module doc explains the retirement.
- `crates/ff-dscatalog/src/storage/{native,esds,sqlite_record,rrds,isam}.rs` --
  reshaped onto `ff-vfs::StorageProvider`.
- `crates/ff-dscatalog/src/storage/native_tests.rs` (new) -- split-out tests incl.
  the ff-vfs seam tests.
- `crates/ff-governance-tests/Cargo.toml` + `tests/mock_compilation.rs` --
  repointed to `ff-dscatalog`.
- `crates/ff-posix-provider/src/lib.rs` + `Cargo.toml` -- reduced to helpers +
  re-export; dev-dep trimmed.
- `crates/ff-vfs/src/posix_provider.rs` -- single-registrant test added.
- `crates/ff-desktop/src/shell/render_nav.rs`, `render_nav_expand.rs` -- call
  sites switched to the single provider.
- `crates/ff-dsalloc/src/catalog_bridge.rs` -- one of two doc mentions corrected
  (see finding 1).
- `docs/quality/TCR.md`, `docs/specs/**/tasks.md`, and the touched spec
  requirements/design -- status flips and gate docs.

Full diff: `git -C c:\workspace\VSC\FileForgeWorkbench diff`.

</details>
