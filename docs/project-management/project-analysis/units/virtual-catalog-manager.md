# Analysis Record: virtual-catalog-manager (W2.2)

- **Wave**: 2 (Catalog and dataset)
- **Backing code**: NO dedicated crate. This is a SHELL-UI subsystem implemented
  in `ff-desktop`: `catalog_manager_dialog.rs`, `dataset_alloc_dialog.rs`,
  `catalog_registry.rs`, `files_panel.rs` / `file_explorer_panel.rs` (the Catalog
  Explorer Context), plus a POSIX provider (`ff-desktop/src/posix_provider.rs` AND
  `ff-vfs/src/posix_provider.rs` -- see PA-CONFLICT-005). The spec's `ff-dscatalog`
  mentions are DELEGATION targets (mainframe CRUD), not this unit's crate.
- **Spec files**: requirements.md (637 lines, 15 requirements), tasks.md
  (190 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 2 pass

---

## 1. Split candidacy

The checklist pre-flag "split candidate (443)" is a STALE line count; the spec is
637 lines / 15 reqs. Assessment:

- Requirement volume: 15 reqs, 637 lines. Above thresholds. (signal 1)
- Responsibilities: this is ONE cohesive concern -- the unified Catalog Explorer
  UI (POM option 1) and its dialogs/registry. The requirements are UI-centric
  (panel, tree, toolbar, catalog-manager dialog, dataset-allocation dialog, POSIX
  file dialog, registry persistence) plus ONE non-UI piece: the POSIX VFS provider
  (Req 7). (partial)
- Crates: NOT a crate -- it lives in the ff-desktop shell. The one genuinely
  separable piece is the POSIX provider, which does NOT belong in the shell at all
  (see PA-CONFLICT-005 below). (this is a MISPLACEMENT, not a size-split)
- Cohesion: the UI is cohesive; the POSIX provider is an outlier that should be
  relocated to the VFS/connector layer.

Not a classic spec split (the UI is one concern). The real structural issue is
FILE SIZE + a MISPLACED PROVIDER, below.

### Source file size violations (PA-STD-025) -- SEVERE, ACTIONABLE

The ff-desktop VCM UI files are the largest cap violations found anywhere:

| File | non-test | note |
|------|----------|------|
| `files_panel.rs` | 1195 | ~3x the cap; the Catalog Explorer content panel |
| `file_explorer_panel.rs` | 935 | ~2.3x the cap |
| `catalog_manager_dialog.rs` | 645 | create/edit/delete dialog |
| `dataset_alloc_dialog.rs` | 419 | allocation dialog |
| `catalog_registry.rs` | 279 | (under cap) |
| `posix_provider.rs` (ff-desktop) | 224 | (under cap; but MISPLACED -- see below) |

rust-standards.md (ff-desktop module layout) explicitly prescribes splitting shell
panels into `_state`/`_render`/`_commands`/`_dialogs`. `files_panel.rs` (1195) and
`file_explorer_panel.rs` (935) badly violate this and the 400 cap. Recorded
PA-STD-025 (REFACTOR; split per the prescribed shell layout). This is the highest
line-count cap violation in the analysis to date.

---

## 2. Cross-unit consistency

### PA-CONFLICT-005 (NEW) -- duplicate + MISPLACED POSIX VfsProvider

Req 7 mandates a `posix` VFS provider implementing `VfsProvider` (Req 7.1-7.7).
There are TWO implementations:

- `ff-vfs/src/posix_provider.rs` (664 lines) -- in the VFS layer (correct home).
- `ff-desktop/src/posix_provider.rs` (404 lines) -- in the SHELL. It genuinely
  `impl VfsProvider for PosixProvider` (line 133, scheme `posix`), wraps
  `ff_connector_local_fs::LocalFsProvider`, and carries `#![allow(dead_code)]`
  with a comment "wired into the UI in Tasks 4-10; suppress until then".

Two problems:
1. DUPLICATION: two `posix`-scheme VfsProvider impls (same anti-pattern class as
   PA-CONFLICT-001/003/004).
2. LAYERING VIOLATION: a VfsProvider in the SHELL (ff-desktop) contradicts
   FFW-ARCH-001 + VFS ownership (W0.11) -- providers belong in the VFS/connector
   layer, never the shell. The `#![allow(dead_code)]` suggests the ff-desktop copy
   is an unwired/unfinished duplicate.

Recorded PA-CONFLICT-005 (owner-gated): pick ONE POSIX provider (the ff-vfs one is
the correct home), delete the ff-desktop copy, and have the VCM UI consume the
ff-vfs provider via the VFS registry. Code change + owner decision. Cross-ref: the
cross-ref table also names connector-local-fs as the Native provider backing --
confirm the posix provider composes with connector-local-fs at Wave 5.

### ADR-001 / dataset-catalog delegation -- consistent (pre-declared)

VCM delegates mainframe catalog CRUD + dataset allocation to `ff-dscatalog`
(Req 3.7, 5.3 invoke `dataset.allocate` via the command framework; Req cross-ref
"Mainframe catalog CRUD delegated to `ff-dscatalog`"). This matches dataset-catalog
ADR-001 (W2.1) and PA-WATCH-011 (verify ff-dsalloc/callers invoke rather than
duplicate). VCM is a CONSUMER of the catalog API -- correct direction. Consistent.

### Catalog registry vs dataset-catalog mount persistence -- WATCH

VCM has its own `Catalog_Registry` persisted under `[virtual_catalogs]` (Req 2.1),
while dataset-catalog persists mounted catalogs under `[catalog]` (W2.1 Req 14).
Two registries touching catalog persistence: VCM's spans all 3 catalog types
(mainframe/posix/native); dataset-catalog's is mainframe-only mount state. Possible
overlap/double-source-of-truth for mainframe catalog mounts. Recorded PA-WATCH-012:
confirm `[virtual_catalogs]` (VCM) and `[catalog].mounted_catalogs` (ff-dscatalog)
are not two competing persistence stores for the same mainframe mounts.

### file-tree-panel reuse

Cross-ref: the explorer tree is "reused/embedded" from file-tree-panel (Wave 4).
`files_panel.rs`/`file_explorer_panel.rs` in ff-desktop appear to be the VCM's own
panels; confirm at file-tree-panel (W4) whether they duplicate or embed the
file-tree-panel widget. Folded into PA-STD-025 review + a W4 cross-check.

### Cross-reference integrity

All 5 declared cross-refs (startup-and-session, virtual-file-system,
dataset-catalog, file-tree-panel, connector-local-fs) resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 190 sub-tasks `[x]`. Implementation present: catalog-manager +
dataset-alloc dialogs, catalog registry, explorer panels, posix provider. Tests
in-file (posix_provider.rs has Req-annotated tests). No PA-INCOMPLETE for
functional behaviour -- BUT the ff-desktop posix_provider carries
`#![allow(dead_code)]` "wired in Tasks 4-10", implying wiring may be incomplete
even though tasks are `[x]`. This is subsumed by PA-CONFLICT-005 (the provider
should be removed from the shell anyway).

### TCR -- moderate

18 TCR rows for VCM-related items (catalog_manager / posix_provider / Catalog
Explorer). Better than Wave-1 absences, weaker than dataset-catalog's 99. Not
raised as a gap; enumeration could be fuller once PA-STD-025 split lands.

---

## 4. Logging audit

Scan of the ff-desktop VCM files:

- `ff_logging` / `log_*!` in catalog_manager_dialog / dataset_alloc_dialog /
  posix_provider / catalog_registry: 0 each.
- These are shell UI + a provider doing catalog create/delete/allocate + POSIX
  file I/O (Req 4.5 recursive delete of backing files; Req 7 read/write). Zero
  logging on destructive operations (delete-catalog-and-files, dataset allocate/
  delete) and on the POSIX provider I/O is a gap -- these are prime CR-NR-058
  dev-logging + operational-WARN/ERROR sites. Recorded PA-LOG-013 (MEDIUM): wire
  ff-logging for catalog create/delete (esp. the recursive file delete Req 4.5),
  dataset allocate/delete, and POSIX provider errors; dev-logging on dialog-confirm
  command dispatch. Consolidate with the ff-dscatalog PA-LOG-012 effort (same
  catalog domain).

---

## 5. Task revision proposals

- **PA-STD-025 (REFACTOR, SEVERE)**: split the ff-desktop VCM panels per the
  prescribed shell layout (`_state`/`_render`/`_commands`/`_dialogs`).
  `files_panel.rs` (1195) and `file_explorer_panel.rs` (935) are the worst cap
  violations in the analysis; `catalog_manager_dialog.rs` (645) + `dataset_alloc_dialog.rs`
  (419) also over. No gate (REFACTOR).
- **PA-CONFLICT-005 (owner-gated, HIGH)**: remove the duplicate/misplaced
  `ff-desktop/src/posix_provider.rs`; keep the provider in `ff-vfs` (664 lines);
  VCM UI consumes it via the VFS registry. Layering fix + de-duplication.
- **PA-STD-026 (ASCII, ACTIONABLE -- BOM/mojibake corruption)**: heavy non-ASCII
  in the VCM files (catalog_manager_dialog 92, dataset_alloc_dialog 71,
  posix_provider 30, catalog_registry 23). Notably `ff-desktop/src/posix_provider.rs`
  contains MOJIBAKE: `Requirement 7.1a-EUR"7.7` and box-drawing banners rendering as
  `a"-a"-a"-` (U+00E2 BOM/UTF-8-artifact sequences per documentation.md, worse than
  plain em-dashes -- these are corrupted bytes, not just typographic chars).
  Replace with ASCII; strip the BOM artifacts. REFACTOR, no gate.
- **PA-LOG-013 (MEDIUM)**: wire ff-logging across catalog create/delete (recursive
  file delete Req 4.5), allocate/delete, POSIX provider errors; dev-logging on
  dialog command dispatch. Consolidate with PA-LOG-012 (ff-dscatalog).
- **PA-WATCH-012**: confirm `[virtual_catalogs]` (VCM registry) vs
  `[catalog].mounted_catalogs` (ff-dscatalog) are not competing persistence stores
  for the same mainframe mounts.

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

virtual-catalog-manager is the SHELL-UI subsystem for POM option 1 (Catalog
Explorer + catalog-manager / dataset-allocation / POSIX dialogs + registry),
implemented in `ff-desktop` (not a dedicated crate). It correctly DELEGATES
mainframe catalog CRUD to ff-dscatalog (ADR-001, consistent). Two significant
structural findings: PA-CONFLICT-005 -- a duplicate, MISPLACED `posix` VfsProvider
living in the shell (ff-desktop) that also exists in ff-vfs, a layering violation
plus duplication; and PA-STD-025 -- the worst 400-cap violations in the whole
analysis (files_panel.rs 1195, file_explorer_panel.rs 935). Plus BOM/mojibake ASCII
corruption in posix_provider.rs (PA-STD-026), zero logging on destructive catalog/
dataset ops (PA-LOG-013), and a possible double catalog-persistence store
(PA-WATCH-012). Tracking is complete and TCR moderate (18 rows). This unit shares
the catalog domain with dataset-catalog (W2.1); its logging and the posix-provider
relocation should be handled together with the W2.1 findings.
