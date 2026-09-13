# Analysis Record: dataset-catalog (W2.1)

- **Wave**: 2 (Catalog and dataset -- largest cluster)
- **Backing crate**: `ff-dscatalog` (spec is explicit: `ff-dataset-catalog` in
  older docs is WRONG; the near-empty `crates/ff-dataset-catalog` (lib.rs+traits.rs)
  is a legacy stub -- see PA-DEP note)
- **Spec files**: requirements.md (816 lines, 31 requirements + governance ref to
  dataset-ownership-model ADR-001), tasks.md (236 sub-tasks, all `[x]`),
  design.md present
- **Analysed**: Wave 2 pass (re-baseline continues)

---

## 1. Split candidacy -- STRONG SPLIT CANDIDATE (PA-SPLIT-008)

Assessment against the 2-of-4 rule -- meets ALL FOUR:

- Requirement volume: 31 requirements, 816 lines. FAR above thresholds
  (12 reqs / ~350 lines). The single largest spec analysed to date. (signal 1)
- Responsibilities: FOUR-FIVE clearly separable concerns, and the spec itself
  declares two of the seams:
  1. **Catalog metadata + lifecycle** (Req 1-15): SQLite catalog DB, DSN naming,
     dataset types, repository, mount/unmount, catalog CRUD/export/import, dataset
     CRUD, PDS members, GDG, VFS provider, properties panel, context menus,
     LISTCAT/LISTDS, config, allocation defaults.
  2. **Record codecs** (Req 16-17): Fixed/Variable/Binary/Text codecs. Req 17.1
     EXPLICITLY states codecs SHALL be "a separate module (or crate) independent of
     any storage provider ... no dependency on SQLite, the filesystem, or egui."
     Self-declared split boundary. Already isolated in `src/codecs/`.
  3. **Storage providers** (Req 18-20): `StorageProvider` trait, NativeFileProvider,
     SqliteRecordProvider, UUID-based physical layout. Already in `src/storage/`.
  4. **VSAM/ISAM** (Req 21-24): KSDS/RRDS/ESDS/ISAM providers (SQLite-backed keyed/
     relative + append-native). In `src/storage/{esds,rrds,isam,sqlite_record}.rs`.
  5. **Cross-cutting infra** (Req 25-31): staged transaction protocol, integrity/
     backup/restore, reconciliation, security/audit, master/user catalog hierarchy,
     portability NFRs. (signal 2 -- far more than 3 responsibilities)
- Crates: it implements the VfsProvider (scheme `catalog`) AND a full record-storage
  engine (codecs + VSAM) that is arguably its own crate(s). (signal 3)
- Low-cohesion clusters: the codec layer (pure byte<->record, GUI/DB-free by Req
  17.1) and the VSAM storage engine are cohesive INTERNALLY but only loosely
  coupled to catalog metadata management. (signal 4)

4-of-4. STRONG split candidate. Recorded PA-SPLIT-008 (owner-gated).

### Proposed split (owner decision)

- Keep `ff-dscatalog` = catalog metadata + lifecycle + VFS provider (Req 1-15,
  25-31 catalog-facing parts).
- Extract `ff-record-codec` = Req 16-17 codecs (the spec already mandates
  independence, Req 17.1). Highest-confidence, lowest-risk extraction.
- Extract `ff-record-store` (or `ff-vsam`) = Req 18-24 StorageProvider +
  NativeFileProvider + SqliteRecordProvider + KSDS/RRDS/ESDS/ISAM. The `storage/`
  subdir is already this.
- ff-dscatalog then DEPENDS on both, wiring providers behind the `StorageProvider`
  trait (Req 19). This matches the spec's own layering (Req 17-19).

Note: this is a SPEC + crate split, owner-gated with its own gate. It also
interacts with the Wave-2 sibling specs (dataset-allocator ff-dsalloc,
dataset-ownership-model = ADR-001 governance) -- coordinate the split with those
(W2.3/W2.4).

### Source file size violations (PA-STD-023) -- MANY, ACTIONABLE

NINE files exceed the 400 non-test cap (worst offenders of the whole analysis so
far):

| File | non-test | note |
|------|----------|------|
| `storage/sqlite_record.rs` | 843 | worst; VSAM keyed/relative record store |
| `catalog.rs` | 609 | catalog metadata core |
| `storage/esds.rs` | 590 | ESDS append store |
| `transactions.rs` | 503 | staged transaction protocol (Req 25) |
| `integrity.rs` | 501 | integrity/backup/restore (Req 26) |
| `storage/vfs_provider.rs` | 482 | VfsProvider impl |
| `gdg.rs` | 459 | GDG management |
| `dsn.rs` | 454 | DSN parsing/validation |
| `schema.rs` | 331 | (under cap; near) |

The split (PA-SPLIT-008) will naturally resolve most of these by moving
sqlite_record/esds into a record-store crate; the remainder (catalog.rs,
transactions.rs, integrity.rs, gdg.rs, dsn.rs) still need concern-splits.
Recorded PA-STD-023 (REFACTOR; several files, do alongside the split).

---

## 2. Cross-unit consistency

### FFW-ARCH-001 (all I/O via VFS) -- NOT a violation here

The crate has 49 `std::fs`/`tokio::fs` calls (catalog.rs 8, integrity.rs 10,
vfs_provider.rs 13, storage/native.rs 7, gdg.rs 4, others). This is NOT an
FFW-ARCH-001 violation: this crate IS the terminal VFS PROVIDER for scheme
`catalog` (Req 10) plus the NativeFileProvider (Req 19.5). FFW-ARCH-001 requires
CONSUMERS to go through VFS; the provider itself legitimately touches the disk.
Contrast workflow-engine (PA-WATCH-004) where a NON-provider did raw checkpoint
I/O. Here the fs usage is concentrated in the provider/storage/integrity layers,
which is correct. Recorded as a CLEAN consistency note (distinct from PA-WATCH-004).

### ADR-001 governance (dataset-ownership-model) -- ownership boundaries stated

The spec opens with a governance reference to dataset-ownership-model (ADR-001):
ff-dscatalog is "the single authority for dataset metadata, catalog entries,
naming validation, and resolution APIs." Two explicit ownership clarifications:
- Req 7: low-level catalog CRUD primitives are ff-dscatalog; JCL-driven allocation
  workflows (DD parsing, DISP, symbolic subst) are owned by `ff-dsalloc`
  (dataset-allocator). The allocator INVOKES catalog primitives, does not duplicate.
- Req 13: `catalog.listcat`/`catalog.listds` are workbench-native tools; the
  IDCAMS-faithful `idcams.listcat` is owned by `ff-idcams` and CALLS catalog
  query APIs. Both read the same catalog API.

These are clean, pre-declared boundaries. Confirm the actual code honours them at
W2.3 (dataset-allocator) and Wave 5 (idcams). Recorded as WATCH PA-WATCH-011
(verify ff-dsalloc invokes rather than duplicates catalog CRUD; verify ff-idcams
listcat delegates).

### Public types and ownership

- Catalog/dataset/DSN/GDG/PDS types, VfsProvider(scheme `catalog`),
  StorageProvider trait + Native/SqliteRecord providers, record codecs -- sole-owned
  by `ff-dscatalog` today (pending PA-SPLIT-008 which would re-home codecs +
  storage). No duplication found against Wave 0/1 units.
- Implements `ff_vfs::VfsProvider` + declares `StorageProvider` (Req 19) -- correct
  direction (consumes the ff-vfs abstraction). Consistent with virtual-file-system
  ownership (W0.11).
- Config `[catalog]` + `[catalog.defaults]` namespace -- sole owner; consumed from
  ff-config. No key collision.
- Commands `catalog.*`, `dataset.*`, `member.*`, `gdg.*` -- registered via
  command-framework. Consistent.

### Legacy stub crate (PA-DEP-002)

`crates/ff-dataset-catalog` exists with only lib.rs + traits.rs. The spec says any
`ff-dataset-catalog` reference is wrong and means `ff-dscatalog`. This stub is
dead/legacy. Recorded PA-DEP-002: confirm the stub is unused and remove it (or
document why it exists) -- avoids confusion with the real `ff-dscatalog`.

### Cross-reference integrity

All 6 declared cross-refs (virtual-file-system, connector-extensibility,
file-tree-panel, fileforge-integration, command-framework, configuration-system)
resolve to existing sub-projects. No dangling references.

---

## 3. Completeness

Tracking: all 236 sub-tasks `[x]`. Implementation is extensive and matches the
31 reqs: catalog/dsn/gdg/pds/repository/vfs_provider + codecs/ (binary/fixed/text/
variable) + storage/ (native/esds/rrds/isam/sqlite_record) + transactions/integrity/
reconciliation(hierarchy)/security(audit)/listcat/properties/context_menu/schema.
Tests in-file. No false-positive pattern detected. No PA-INCOMPLETE raised.

### TCR -- STRONG (contrast Wave 1)

TCR.md has 99 rows for `ff-dscatalog` / dataset-catalog -- by far the best coverage
enumeration seen so far (Wave 1 had multiple total-absences). No TCR gap raised.
This crate is a positive exemplar for TCR discipline.

---

## 4. Logging audit

Scan of `crates/ff-dscatalog/src` (recursive):

- `ff_logging` / `log_*!` macro CALLS: 0
- `ff-logging` Cargo dep: PRESENT (Cargo.toml:13) -- 0 uses. DEAD (same pattern as
  the Wave-1 dead-dep cluster PA-LOG-008/009/011).
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 49 (legitimate provider I/O, see section 2)

This is a MORE serious dead-log-dep than the Wave-1 pure models: ff-dscatalog is a
DISK-TOUCHING, DB-backed, transactional subsystem (staged transactions Req 25,
integrity/backup Req 26, reconciliation Req 27, roll-off Req 9.3, corruption
handling in NFRs) with 49 fs calls and a SQLite DB -- exactly the kind of code
where operational logging is essential for testing/debugging (CR-NR-058). Several
reqs describe error/recovery paths (Req 5.5 mount validation, Req 6.7 import
integrity, Req 16.7 codec diagnostic, Req 25 staged-transaction recovery, Req 26
integrity, NFR "corrupt database ... descriptive error") that should log at
WARN/ERROR. Currently NONE do. Recorded PA-LOG-012 (MEDIUM-HIGH): wire ff-logging
across mount/allocate/delete/roll-off/transaction/integrity paths -- ERROR on
I/O + DB failures, WARN on roll-off/reconcile/import-mismatch, dev-logging DEBUG
on command start/params/result. This is the highest-value logging target in Wave 2
so far.

---

## 5. Task revision proposals

- **PA-SPLIT-008 (owner-gated, HIGH)**: split ff-dscatalog into catalog-metadata
  (Req 1-15,25-31) + `ff-record-codec` (Req 16-17, spec-mandated independent) +
  `ff-record-store`/`ff-vsam` (Req 18-24). Coordinate with dataset-allocator
  (W2.3) and dataset-ownership-model ADR-001 (W2.4). SPEC + crate split, own gate.
- **PA-STD-023 (REFACTOR)**: 9 files over the 400 cap (sqlite_record.rs 843 worst).
  Much resolves via the split; residual concern-splits for catalog.rs/transactions.rs/
  integrity.rs/gdg.rs/dsn.rs. No gate.
- **PA-STD-024 (ASCII, ACTIONABLE -- runtime UI strings)**: 34 non-ASCII bytes
  (10 non-comment). The non-comment set is ellipsis (U+2026) in `context_menu.rs`
  menu-item LABELS ("Mount Catalog...", "New Dataset...", etc.) -- runtime
  user-facing strings, a genuine defect. Replace with ASCII `...`; fix doc-comment
  dashes too. REFACTOR, no gate.
- **PA-LOG-012 (MEDIUM-HIGH)**: wire ff-logging (dead dep) across the I/O/DB/
  transaction/integrity/roll-off paths (ERROR/WARN) + dev-logging command trace.
  Highest-value Wave-2 logging target. Depends on the Phase PA-W0.1 dev-logging gate.
- **PA-DEP-002 (cleanup)**: remove/justify the legacy `ff-dataset-catalog` stub
  crate (lib.rs+traits.rs) that the spec says is superseded by `ff-dscatalog`.
- **PA-WATCH-011**: verify ADR-001 boundaries in code -- ff-dsalloc invokes (not
  duplicates) catalog CRUD (W2.3); ff-idcams listcat delegates to catalog API (Wave 5).

No requirement CHANGE proposed; the spec is internally consistent (governed by
ADR-001) and complete. The split is a structural proposal, not a spec correction.

---

## Summary

dataset-catalog (`ff-dscatalog`) is the largest and most complex unit analysed so
far: 31 reqs / 816 lines spanning catalog metadata, record codecs, native + SQLite
storage providers, full VSAM (KSDS/RRDS/ESDS/ISAM), staged transactions, integrity,
and master/user hierarchy. It is a 4-of-4 STRONG split candidate (PA-SPLIT-008) --
the spec itself mandates codec independence (Req 17.1) and a StorageProvider seam
(Req 19). Tracking is complete and TCR coverage is EXCELLENT (99 rows -- a positive
exemplar). The 49 fs calls are LEGITIMATE (this crate IS the VFS provider +
NativeFileProvider), NOT an FFW-ARCH-001 violation. Key gaps: nine 400-cap files
(PA-STD-023, mostly resolved by the split), ellipsis in runtime menu labels
(PA-STD-024), and -- most significantly -- a DEAD ff-logging dep on a
disk/DB/transactional subsystem whose error/recovery paths log nothing
(PA-LOG-012, MEDIUM-HIGH, the top Wave-2 logging target). ADR-001 ownership
boundaries with ff-dsalloc and ff-idcams are pre-declared and carried as
PA-WATCH-011. Legacy `ff-dataset-catalog` stub flagged for removal (PA-DEP-002).
