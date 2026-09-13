# Analysis Record: structure-catalog (W2.5)

- **Wave**: 2 (Catalog and dataset)
- **Backing crate**: `ff-structure-catalog`
- **Spec files**: requirements.md (356 lines, 15 requirements), tasks.md
  (251 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 2 pass

---

## 1. Split candidacy

NOT a split candidate.

Assessment against the 2-of-4 rule:

- Requirement volume: 15 reqs, 356 lines. Marginally above thresholds. (1 weak signal)
- Responsibilities: SIX named capabilities (persistent store, CRUD, browsing
  panel, structure editor, auto-association, import/export/versioning) but all one
  cohesive concern -- a library of named `.ffs` record-structure definitions.
  Already split into 16 files (catalog/store/model/field/browsing/association/
  copybook/ffs_format/grid/...).
- Crates: single crate.
- Cohesion: high.

1 weak signal. No split.

### Source file sizes -- ALL WITHIN CAP

Largest non-test: `copybook.rs` 354, `grid.rs` 332, `ffs_format.rs` 315 -- all
UNDER the 400 cap. No PA-STD size item.

---

## 2. Cross-unit consistency

### PA-CONFLICT-008 (NEW) -- duplicate record-structure model vs fileforge-integration

The spec says structure-catalog "extends `fileforge-integration` (which owns
record parsing, field extraction, and file writing logic)" and cross-refs it for
"field types, record structures". But structure-catalog defines its OWN parallel
record-structure model and does NOT depend on the fileforge crate:

- structure-catalog: `FieldType` enum (field.rs), `FieldDefinition` (field.rs),
  `RecordStructure` (model.rs).
- fileforge-integration = crate `ff-forge`: `DataType` enum (field_def.rs),
  `FieldDefinition` (field_def.rs), `RecordStructure` (record_structure.rs),
  `StructureFile` (structure_file.rs).
- structure-catalog Cargo.toml has NO dep on ff-forge (grep empty). Deps are
  ff-logging/ff-command/ff-config/ff-vfs only.

So there are TWO `FieldDefinition` structs, TWO `RecordStructure` structs, and
parallel `FieldType`(catalog) vs `DataType`(forge) enums across two crates that
the spec says should share -- unbridged. Same anti-pattern class as
PA-CONFLICT-003/004/006 (duplicate domain types across crates that should reuse a
single owner). fileforge-integration is the declared owner of "record structures /
field types"; structure-catalog should consume ff-forge's `FieldDefinition`/
`RecordStructure`/`DataType` (or a shared interface), not redefine them. Recorded
PA-CONFLICT-008 (owner-gated): unify the record-structure model under ff-forge
(fileforge-integration); structure-catalog depends on it. Confirm the exact
divergence + reconcile at fileforge-integration (Wave 5). Note ff-forge is the
actual crate dir; the spec calls it `ff-fileforge` (naming drift).

### `.ffs` format vs ff-forge `StructureFile`

structure-catalog owns the `.ffs` (TOML) FILE FORMAT + catalog persistence (Req 2)
via `ffs_format.rs`; ff-forge has `structure_file.rs`/`StructureFile`. Possible
overlap between the two file-format models -- folds into PA-CONFLICT-008 (same
model-duplication root). The catalog LIBRARY/persistence concern is legitimately
structure-catalog's; the record-structure DATA MODEL should be ff-forge's.

### Copybook import

`copybook.rs` (COBOL copybook import, Req: import legacy formats) produces
RecordStructures -- if it parses COBOL PIC clauses into the catalog's own
FieldType, that logic may also overlap ff-forge's field model. Folded into
PA-CONFLICT-008.

### Public types and ownership (non-conflicting)

- Catalog store, `.ffs` persistence, browsing-panel summary
  (`RecordStructureSummary`), auto-association (glob/pattern matching), structure
  editor, versioning -- these LIBRARY/UI concerns are sole-owned by
  structure-catalog. No duplication of these.
- Commands `CATALOG`, `APPLY STRUCTURE` (Manual_Association_Command) registered
  via command-framework. Consistent.
- Config `catalog.locations`/`catalog.active_location` read from ff-config.
  NB potential key-namespace overlap with dataset-catalog's `[catalog]` table
  (W2.1) -- structure-catalog uses `catalog.locations`/`catalog.active_location`,
  dataset-catalog uses `[catalog].mounted_catalogs`/`default_hlq`. Both live under
  a `catalog` config root. Recorded PA-WATCH-014: confirm the two `catalog.*`
  config surfaces do not collide (structure catalog vs dataset catalog are
  different domains sharing a config prefix).
- 0 fs calls -- access via VFS (Req 1.8) / ff-config. Clean, no FFW-ARCH-001 issue.

### Cross-reference integrity

All 5 declared cross-refs (fileforge-integration, configuration-system,
layout-and-docking, command-framework, virtual-file-system) resolve. No dangling
refs -- though the fileforge dependency is DECLARED in prose but ABSENT in
Cargo.toml (PA-CONFLICT-008).

---

## 3. Completeness

Tracking: all 251 sub-tasks `[x]`. Implementation present across all 15 reqs
(store, `.ffs` format, CRUD, browsing, editor, association, copybook import,
export, versioning, grid browse/edit). Tests in-file with `// Validates:`
annotations. No false-positive pattern. No PA-INCOMPLETE for functional behaviour.

### TCR gap (PA-TCR-011) -- TOTAL ABSENCE

TCR.md has ZERO rows for `ff-structure-catalog` (grep = 0) across 15 reqs /
~110 criteria, despite well-annotated in-file tests. Another total absence (like
the Wave-1 cluster PA-TCR-005/006/009). RECORDING gap, not test gap. Recorded
PA-TCR-011.

---

## 4. Logging audit

Scan of `crates/ff-structure-catalog/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

MANDATED logging is unimplemented (like ff-wrap PA-LOG-010): Req 1.5 (INFO on
catalog-dir auto-create), Req 1.6 (WARN on inaccessible location -> skip), Req 2.5
(WARN on invalid TOML -> exclude), Req 2.6 (WARN on schema-validation failure ->
exclude). These are exactly the catalog-load error paths where WARN/INFO belong,
and the crate does catalog-directory + `.ffs` file loading. The dead ff-logging dep
+ four mandated-but-absent log records = PA-LOG-015 (MEDIUM): wire ff-logging for
the Req 1.5/1.6/2.5/2.6 records; add dev-logging on CATALOG/APPLY STRUCTURE command
dispatch. Consolidate with the catalog-domain logging effort (PA-LOG-012/013).

---

## 5. Task revision proposals

- **PA-CONFLICT-008 (owner-gated, MEDIUM-HIGH)**: unify the record-structure model
  (`FieldDefinition`/`RecordStructure`/field-type enum) under fileforge-integration
  (`ff-forge`); structure-catalog depends on it and adds only the catalog LIBRARY/
  persistence/`.ffs`/UI layer on top. Reconcile `FieldType`(catalog) vs
  `DataType`(forge) and the two `StructureFile`/`.ffs` models. Confirm exact
  divergence at fileforge-integration (Wave 5). Owner decision + code.
- **PA-LOG-015 (MEDIUM)**: wire ff-logging (dead dep) for the mandated Req 1.5
  (INFO), 1.6/2.5/2.6 (WARN) records + dev-logging command dispatch. Consolidate
  with catalog-domain PA-LOG-012/013.
- **PA-STD-029 (ASCII, comment-only this time)**: 211 non-ASCII bytes in `.rs` --
  206 em-dashes in doc comments (`//! ... -- ...`) + 5 box-drawing. ALL in comments
  (0 in runtime strings -- better than the W2.1/W2.2 runtime-string defects). Still
  a documentation.md violation (em-dash + box-drawing prohibited in `.rs`). Replace
  with `--` and `// === ===`. REFACTOR, no gate.
- **PA-TCR-011**: enumerate per-requirement TCR rows (0 rows for 15 reqs). No code.
- **PA-WATCH-014**: confirm `catalog.locations`/`catalog.active_location`
  (structure-catalog) vs `[catalog].mounted_catalogs`/`default_hlq` (dataset-catalog)
  do not collide under the shared `catalog` config root.
- **PA-DOC (naming)**: fileforge-integration spec says crate `ff-fileforge`; actual
  is `ff-forge`. Fold into the naming reconciliation set.

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

structure-catalog (`ff-structure-catalog`) is a cohesive persistent library of
named `.ffs` record-structure definitions (CRUD, browse panel, editor,
auto-association, copybook import, versioning). Not a split candidate; no size
violation; 0 fs (VFS-accessed). The significant finding is PA-CONFLICT-008: it
defines its OWN `FieldDefinition`/`RecordStructure`/`FieldType` model with NO
dependency on fileforge-integration (`ff-forge`, which has parallel
`FieldDefinition`/`RecordStructure`/`DataType`) -- despite the spec saying it
"extends fileforge-integration which owns record structures / field types". Two
unbridged record-structure models that should share one owner (ff-forge). Other
items: dead ff-logging dep with FOUR mandated-but-absent log records
(PA-LOG-015), comment-only ASCII violation (PA-STD-029, no runtime-string defect
this time), total TCR absence (PA-TCR-011), a shared-`catalog.*`-config-prefix
watch vs dataset-catalog (PA-WATCH-014), and the ff-forge/ff-fileforge naming
drift. The record-structure model unification (PA-CONFLICT-008) should be resolved
at fileforge-integration (Wave 5).
