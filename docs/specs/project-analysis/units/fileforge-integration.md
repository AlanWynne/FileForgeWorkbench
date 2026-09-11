# Analysis Record: fileforge-integration (W5.15-late / row 76)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-forge` (FileForge flat-file processing: fixed-width record
  structures, structure overlay, grid edit, EBCDIC/COMP-3/VB handling, ASA detection,
  .ffs structure files, multi-record-type, validation, navigation)
- **Spec files**: requirements.md (360 lines, 16 requirements), tasks.md
  (198 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy

16 reqs / 360 lines, 20 files, NO file over 350 -- well-decomposed already (record
structure, overlay, grid, ebcdic, comp3, vb, asa, validation, navigation, .ffs, ...).
Not a split candidate (the concern is broad but the files are small + cohesive). No split.

---

## 2. Cross-unit consistency -- PA-CONFLICT-022 (EBCDIC + ASA DUPLICATION -- reimplements peers)

`ff-forge` reimplements TWO capabilities that OTHER crates already own, with NO dep on
either:

1. **EBCDIC (vs ff-encoding)**: ff-forge has its own `ebcdic.rs` codepage tables +
   conversion (95 EBCDIC refs) and does NOT depend on `ff-encoding` (0). But
   `ff-encoding` (encoding-and-characters, Wave 1) OWNS EBCDIC/codepage conversion
   (81 EBCDIC refs). So EBCDIC is implemented TWICE.

2. **ASA carriage control (vs ff-asa + ff-viewers)**: ff-forge has its own `asa.rs`
   (`AsaControl::parse_asa_char`, ASA auto-detection, Req 7) and does NOT depend on
   `ff-asa` (0). Combined with ff-asa (W5.7, the wired ASA preview) and ff-viewers/
   asa_report.rs (W5.10), ASA is now implemented in THREE places.

Recorded PA-CONFLICT-022 (owner-gated, MEDIUM): make ff-forge DEPEND ON + reuse
ff-encoding for EBCDIC and ff-asa for ASA detection (delete its inline ebcdic.rs codepage
tables + asa.rs), OR, if ff-forge's flat-file EBCDIC/ASA needs differ materially from the
general crates (e.g. field-level COMP-3-aware conversion), document WHY the duplication is
intentional + share the low-level tables. This is the SAME reimplement-instead-of-reuse
pattern as ff-viewers (W5.10). Extends the ASA-duplication half of PA-CONFLICT-019 to a
THREE-way finding (ff-asa / ff-viewers / ff-forge). Code + owner decision.

### WIRED (NOT an orphan) + clean-seam grid

`ff-forge` is referenced 172x outside the crate (FileForge command integration Req 15 +
grid). NOT an orphan. Despite Req 3 (Grid_Edit_Mode tabular), it has NO egui dep (sole
deps thiserror + serde) -- the grid is a DATA MODEL the shell renders (clean-seam, like
ff-asa / compare-and-merge). GUI-independent. Positive.

### COMP-3 / VB / .ffs -- sole-owned (no duplication)

Packed-decimal (COMP-3, Req 5), variable-length binary records (VB, Req 6), .ffs
structure-file association (Req 12), multi-record-type (Req 13) are FileForge-specific and
sole-owned -- no overlap. Only EBCDIC + ASA are duplicated (PA-CONFLICT-022).

### Naming drift (PA-DOC-010)

Spec calls it `ff-fileforge`; the crate dir is `ff-forge`. Naming-reconciliation set.

### Cross-reference integrity

ff-forge does NOT cross-reference ff-encoding / ff-asa despite duplicating them
(PA-CONFLICT-022). FileForge command integration (Req 15) is wired (172 refs).

---

## 3. Completeness

Tracking: all 198 sub-tasks `[x]`. Implementation present across all 16 reqs (structure,
overlay, grid, EBCDIC, COMP-3, VB, ASA, auto-detect, validation, navigation, insert/delete,
.ffs, multi-record-type, filtering, command integration, error handling) + WIRED. No
PA-INCOMPLETE. Complete.

### TCR gap (PA-TCR-036)

TCR.md has 1 row for ff-forge across 16 reqs -- thin, despite 198 tasks + tests.
Recorded PA-TCR-036.

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 0; `ff-logging` Cargo dep: ABSENT (not dead).
- `std::fs`: 0.

Flat-file parsing/conversion is a meaningful logging site: EBCDIC/COMP-3 conversion
failures, VB RDW errors, structure-overlay mismatches, auto-detect decisions (Req 8) --
these are the "why did my flat file render wrong" diagnostics. Recorded PA-LOG-051
(LOW-MEDIUM): add ff-logging + dev-logging on conversion failures / RDW errors / overlay
mismatches / auto-detect under the `dev-logging` gate.

---

## 5. Task revision proposals

- **PA-CONFLICT-022 (owner-gated, MEDIUM)**: ff-forge reimplements EBCDIC (vs ff-encoding)
  + ASA (vs ff-asa) with 0 deps on either. Reuse ff-encoding + ff-asa (delete inline
  ebcdic.rs / asa.rs), OR document why flat-file needs differ + share the low-level tables.
  Extends PA-CONFLICT-019 ASA duplication to THREE-way. Code + owner decision.
- **PA-DOC-010 (naming)**: reconcile spec `ff-fileforge` to crate dir `ff-forge`. Docs.
- **PA-TCR-036**: enumerate per-requirement TCR rows (1 for 16 reqs). No code.
- **PA-STD-072 (ASCII)**: 16 non-comment non-ASCII -- mostly EBCDIC codepage COMMENT
  annotations (documenting mappings; the code correctly uses `\u{...}` escapes) + an arrow
  (U+2192 "NEL -> LF") in a comment + a test emoji literal + em-dashes in error.rs strings.
  Replace comment arrows/em-dashes with ASCII; the codepage comment glyphs + test emoji are
  borderline (document the mapping in ASCII, e.g. "0x4A -> U+00A2 (cent)"). REFACTOR, no gate.
- **PA-LOG-051 (LOW-MEDIUM)**: conversion/RDW/overlay/auto-detect dev-logging.

No split (files under cap). No orphan (wired). No raw-fs.

---

## Summary

fileforge-integration (`ff-forge`) is a complete, WIRED (172 refs) flat-file processing
crate (fixed-width structures, overlay, grid edit, EBCDIC/COMP-3/VB, ASA, .ffs,
multi-record-type; 16 reqs, 20 files, no cap issue, 198/198). Clean-seam grid (no egui dep,
data model the shell renders). The headline finding is PA-CONFLICT-022 (MEDIUM): ff-forge
REIMPLEMENTS two capabilities other crates own, with 0 deps on either -- EBCDIC (its own
ebcdic.rs codepage tables vs ff-encoding's 81 EBCDIC refs) and ASA carriage control (its
own asa.rs vs ff-asa the wired ASA crate). This makes ASA implemented in THREE places
(ff-asa / ff-viewers / ff-forge -- extending PA-CONFLICT-019) and EBCDIC in TWO. Reuse
ff-encoding + ff-asa (or document why flat-file needs differ + share the tables). Same
reimplement-instead-of-reuse pattern as ff-viewers (W5.10). Hygiene: PA-DOC-010 (name drift
ff-fileforge vs ff-forge), PA-TCR-036 (thin 1/16), PA-STD-072 (comment arrows/em-dashes +
EBCDIC codepage comment glyphs -- code correctly uses \u escapes), PA-LOG-051 (LOW-MED --
conversion/RDW/overlay/auto-detect logging). COMP-3/VB/.ffs are correctly sole-owned. No
orphan, no incompleteness.
