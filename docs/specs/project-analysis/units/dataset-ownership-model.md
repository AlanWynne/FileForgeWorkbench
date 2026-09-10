# Analysis Record: dataset-ownership-model (W2.4)

- **Wave**: 2 (Catalog and dataset)
- **Nature**: GOVERNANCE / ADR-001 -- NOT a shippable crate. Defines single-authority
  ownership boundaries, dependency direction, conflict resolutions, lifecycle
  ownership, interface contracts, and compliance verification for the catalog
  cluster (ff-vfs, ff-dscatalog, ff-dsalloc, ff-vsam-services, ff-idcams).
- **Backing code**: `crates/ff-governance-tests` (architecture-compliance test
  suite) + the `CatalogService`/`DynCatalogService` traits in
  `crates/ff-dataset-catalog/src/traits.rs` (the shared INTERFACE crate) + the
  `VsamService` trait in `crates/ff-vsam-services/src/traits.rs`. NO dedicated
  runtime crate (the earlier "ff-vfs" crate-name reading was the universal-dep
  reference, not this unit's home).
- **Spec files**: requirements.md (379 lines, 20 requirements), tasks.md
  (59 sub-tasks, all `[x]`), design.md, .config.kiro present
- **Analysed**: Wave 2 pass

---

## 1. Split candidacy

N/A -- this is a governance document, not a code unit. 20 reqs / 379 lines, but
they are cohesive (one ADR). No split.

---

## 2. Cross-unit consistency -- this IS the consistency authority

This unit is the source of the ADR-001 ownership rules that W2.1-W2.3 were checked
against. Compliance assessment of the actual code:

### Dependency direction (Req 7) -- ENFORCED and HONORED

- `ff-governance-tests/tests/architecture_compliance.rs` implements Req 18: it
  parses workspace Cargo.tomls and asserts `PROHIBITED_DEPENDENCIES` (ff-vfs has
  no domain deps; catalog/vsam have no ff-idcams/ff-dsalloc deps). Runnable via
  `cargo test -p ff-governance-tests`. The ADR is SELF-ENFORCING -- a strong
  positive (contrast most specs which have no fitness function).
- Verified clean in code: ff-dscatalog has NO dep on ff-dsalloc/ff-idcams (W2.4
  grep); ff-dsalloc delegates via the `CatalogProvider`/`CatalogService` trait with
  no ff-dscatalog dep (W2.3); ff-dsalloc has 0 `use rusqlite` (Req 12.3 compliant);
  ff-dsalloc's only TOML parsing is in TESTS (Req 12.4/12.5 not violated in prod).

### Interface contracts (Req 15-17) -- traits exist in the right places

- `CatalogService` + `DynCatalogService` (object-safe wrapper, Req 15.7) are in
  `ff-dataset-catalog/src/traits.rs`. IMPORTANT CORRECTION to W2.1: the
  `ff-dataset-catalog` crate (lib.rs + traits.rs) is NOT a dead legacy stub
  (PA-DEP-002) -- it is the SHARED INTERFACE crate holding the ADR-mandated
  `CatalogService` contract (Req 7.6 "defined in ff-dataset-catalog OR a shared
  interface crate"); `ff-dscatalog` is the IMPLEMENTATION. PA-DEP-002 is
  WITHDRAWN/RECLASSIFIED (see below).
- `VsamService` trait exists in `ff-vsam-services/src/traits.rs` (370 lines) per
  Req 16; `ff-vsam-services` lib is a 27-line stub -- which Req 16.7 EXPLICITLY
  permits ("UNTIL ff-vsam-services is implemented, dependent crates SHALL compile
  against the trait definition with a no-op/error stub"). So the empty impl is the
  ADR-sanctioned interim state, not a defect.

### The one real compliance GAP -- PA-CONFLICT-007 (VSAM implementation misplaced)

ADR-001 Req 5.1 says ff-vsam-services SHALL OWN VSAM behaviour (KSDS/ESDS/RRDS/LDS).
But the actual VSAM IMPLEMENTATION lives in `ff-dscatalog/src/storage/`
(esds.rs 590, rrds.rs, isam.rs, sqlite_record.rs 843, native.rs) -- NOT in
ff-vsam-services (trait-only stub). So VSAM record logic is implemented in the
catalog crate, which ADR Req 3.2 says SHALL NOT own VSAM. This is the W2.1
PA-SPLIT-008 gap, now precisely characterised as an ADR-001 COMPLIANCE gap:
the migration of VSAM code from ff-dscatalog into ff-vsam-services is PENDING
(the trait target crate exists but is empty). Recorded PA-CONFLICT-007
(owner-gated): migrate the `storage/` VSAM impl (esds/rrds/isam/sqlite_record) from
ff-dscatalog into ff-vsam-services behind the existing `VsamService` trait. This
IS PA-SPLIT-008's `ff-vsam` target = ADR's ff-vsam-services -- consolidate the two
records (PA-SPLIT-008 becomes "migrate to the already-existing ff-vsam-services").

### Compliance-test coverage GAP -- PA-WATCH-013

The governance test checks `ff-dataset-catalog` (the INTERFACE crate) but has 0
references to `ff-dscatalog` (the IMPLEMENTATION crate that holds all the code and
the 4 upstream deps ff-vfs/ff-command/ff-config/ff-logging). So the fitness
function validates the interface crate's deps but NOT the implementation crate's --
ff-dscatalog could acquire a prohibited dep (e.g., ff-dsalloc/ff-idcams) undetected.
Recorded PA-WATCH-013: extend architecture_compliance.rs to also assert
ff-dscatalog's (and ff-dsalloc's) prohibited deps, not just the interface crate.

### Spec alignment (Req 19) -- HONORED

Req 19.5 requires each subsystem spec to cross-reference this governance doc.
Confirmed: dataset-catalog (W2.1) and dataset-allocator (W2.3) both open with a
"Governance Reference: ADR-001" block. The alignment notes I saw in those specs
(Req 7/13 ownership clarifications) are the Req 19.2/19.3 updates. Consistent.

### PA-DEP-002 RECLASSIFICATION

W2.1 recorded PA-DEP-002 ("remove the legacy ff-dataset-catalog stub"). W2.4 shows
this is WRONG: ff-dataset-catalog is the intentional shared-interface crate
(CatalogService trait home). RECLASSIFY PA-DEP-002 from "remove" to "DOC: clarify
that ff-dataset-catalog = interface crate, ff-dscatalog = implementation; the
dataset-catalog spec's 'ff-dataset-catalog is wrong, read ff-dscatalog' note is
itself misleading because the interface crate legitimately uses that name."

---

## 3. Completeness

Tracking: all 59 sub-tasks `[x]`. The governance infrastructure is substantially
implemented: architecture_compliance.rs (Req 18 fitness function), CatalogService/
DynCatalogService + VsamService trait contracts (Req 15/16), spec cross-references
(Req 19). Remaining real work is the VSAM code MIGRATION (PA-CONFLICT-007) and the
compliance-coverage extension (PA-WATCH-013) -- both are downstream of the ADR, not
gaps in the ADR itself. No PA-INCOMPLETE against the governance doc.

### TCR

Governance is verified by the compliance test suite rather than per-criterion TCR
rows; a formal TCR enumeration is less applicable to an ADR. Not raised as a gap.

---

## 4. Logging audit

N/A for a governance document. The governance-tests crate is a test harness
(no runtime logging expected). No PA-LOG raised.

---

## 5. Task revision proposals

- **PA-CONFLICT-007 (owner-gated, HIGH)**: migrate the VSAM implementation from
  `ff-dscatalog/src/storage/` (esds/rrds/isam/sqlite_record/native) into the
  existing (currently trait-only) `ff-vsam-services` crate, behind the
  `VsamService` trait, per ADR-001 Req 5. This SUBSUMES/reframes PA-SPLIT-008's
  VSAM half: the target crate already exists -- this is a migration, not a new
  extraction. ff-dscatalog then depends on ff-vsam-services (Req 7.1 direction:
  catalog -> vsam -> storage). Code + owner decision.
- **PA-WATCH-013**: extend `ff-governance-tests/tests/architecture_compliance.rs`
  to assert prohibited deps for `ff-dscatalog` (impl) and `ff-dsalloc`, not only
  the `ff-dataset-catalog` interface crate. Closes the fitness-function coverage
  gap. Low-risk test addition.
- **PA-DEP-002 (RECLASSIFIED, doc-only)**: was "remove ff-dataset-catalog stub";
  now "clarify ff-dataset-catalog = shared interface crate (CatalogService),
  ff-dscatalog = implementation; correct the dataset-catalog spec's misleading
  'ff-dataset-catalog is wrong' note". Do NOT remove the crate.

No requirement CHANGE proposed for the ADR itself -- it is well-formed and
substantially enforced. The proposals are downstream code/test alignment.

---

## Summary

dataset-ownership-model is ADR-001: the GOVERNANCE authority for the catalog
cluster, and it is one of the best-enforced specs in the analysis -- it ships a
real architectural fitness function (`ff-governance-tests`, Req 18), defines the
`CatalogService`/`DynCatalogService`/`VsamService` trait contracts in the correct
interface crates (Req 15/16), and the downstream specs carry the mandated
cross-references (Req 19). Verification confirmed the dependency direction and
trait-delegation are honored in code (W2.1-W2.3 findings align). Two corrections
came out of this unit: (1) PA-DEP-002 is WITHDRAWN -- `ff-dataset-catalog` is the
intentional shared-interface crate, not a dead stub; (2) the VSAM implementation is
MISPLACED -- it lives in ff-dscatalog `storage/` while the ADR-mandated owner
`ff-vsam-services` is a trait-only stub (PA-CONFLICT-007, which subsumes the VSAM
half of PA-SPLIT-008 as a MIGRATION to an already-existing crate). One
enforcement gap: the compliance suite checks the interface crate but not the
implementation crate `ff-dscatalog` (PA-WATCH-013).
