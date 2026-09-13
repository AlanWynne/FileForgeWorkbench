# File Formatter Plugin -- Design Input (CR-NR-061)

> STATUS: DESIGN INPUT ONLY. This is NOT a requirements/design/tasks document and
> defines no acceptance criteria. It preserves the raw material for the File
> Formatter UX design session (project-master Phase (file-formatter-design),
> tasks FFMT-D.1-D.8) so nothing depends on the originating chat session
> surviving. No source code and no requirements gate until the design session
> output is approved. Change-log entry: CR-NR-061 (PENDING GATE).

---

## 1. Summary

"File Formatter" is a proposed FileForgeWorkbench plugin inspired by BMC/Compuware
File-AID for MVS. It views and edits record-oriented data files -- flat ASCII
text, VSAM ESDS, VSAM KSDS, or binary -- according to a known record structure.
The types of files of interest are any Records-Based data file that can contain
records with predefined record structures.

Core intent, in the owner's words:
- If we know the record structure, we want to View and Edit the file according to
  its record structure.
- A data file may contain many records, and records in a file may NOT all have the
  same structure. Unless stated otherwise, assume a file contains more than one
  record type.
- A file with more than one record type normally has a field that serves as a
  record identifier (a discriminant). The structure metadata must tell the
  formatter how to determine which record structure applies to each record.
- We may know only ONE of the record-type structures of a file (not all of them)
  and must still be able to view/edit what we can.

---

## 2. Source material (owner-supplied)

Web references on how File-AID works (BMC/Compuware File-AID for MVS 17.02):
- https://docs.bmc.com/xwiki/bin/view/Mainframe/DevX/BMC-Compuware-File-AIDMVS/bcfamvs1702/
- https://docs.bmc.com/xwiki/bin/view/Mainframe/DevX/BMC-Compuware-File-AIDMVS/bcfamvs1702/Getting-started/File-AID-MVS-summary/
- https://www.scribd.com/document/561218904/FILE-AID

Prior-art project (closest existing model): TextFileConverter
- Location: `C:\workspace\Kiro\TextFileConverter`
- Spec files read during triage:
  - `C:\workspace\Kiro\TextFileConverter\.kiro\specs\text-file-converter\requirements.md`
  - `C:\workspace\Kiro\TextFileConverter\.kiro\specs\text-file-converter\design.md`
  - `C:\workspace\Kiro\TextFileConverter\README.md`
  - `C:\workspace\Kiro\TextFileConverter\file_conversion_configuration_template.fc.json`
- It already solves the two hard parts: multiple record types per file, and a
  discriminant field that selects the correct layout per record. Recommendation:
  mine its FORMAT and record-identification/filter SEMANTICS; do not depend on the
  Python code (FFWB is Rust).

### TextFileConverter `.fc.json` record-structure model (reference)

Each record type is a named set of fields. Each field carries:

| Attribute     | Meaning                                                        |
|---------------|----------------------------------------------------------------|
| `offset`      | zero-based start position of the field within the record       |
| `length`      | number of characters/bytes in the field                        |
| `decimals`    | implied decimal places (0 = none); packed-integer support      |
| `data_type`   | `str` / `int` / `float` / `bool` (legacy repr normalised)      |
| `identifiers` | values that mark a line as belonging to this record type       |
| `filters`     | if non-empty, only records whose identifier value is listed    |

Record identification: a line belongs to a record type when the value at the
identifier field's offset/length matches a value in that field's `identifiers`
list. First matching record type wins. Filtering: when `filters` is non-empty,
only records whose identifier value also appears in `filters` are included.
Decimal packing: raw `020000` with `decimals: 2` displays as `200.00`; reversed
on write. Config can be JSON (`.fc.json`) or Excel (`.fc.xlsx`), looked up next to
the source file by base name.

---

## 3. Proposed capabilities

1. An initial View/Edit Dataset Specification workspace: the data file name, a
   file containing its record structure, and optional filter/selection criteria.
   Once set up, the data file name + selected record-structure file should be
   saved in a configuration file.
2. Filter the file by record structure and view only records of a particular type.
3. File Explorer right-click integration on a file name: "View with FF" and
   "Edit with FF".
4. Build a File Structure from a COBOL copybook. FFWB must NOT be restricted to
   COBOL copybooks -- it should take layouts from various sources and map them to
   an FFWB-native format (TOML or JSON). This needs a workspace where a layout is
   provided in various formats and converted to a common FFWB format. Scope of
   which sources are supported and how the structure is stored is to be decided in
   the design session.
5. With at least one record-type structure known (not necessarily all), view or
   edit the file in a field/data-value display, presented VERTICALLY or in a
   HORIZONTAL/columnar format.
6. Edit mode: edit fields individually with data-type enforcement (numeric fields
   stay numeric). If a field's data does not conform to its data type, display the
   hexadecimal equivalent (e.g. `2/NUM  X'4040'`).
7. In CHAR view, all the normal FFWB editor editing commands work.

---

## 4. Built-in commands (owner-specified)

| Command | Behaviour                                                                          |
|---------|------------------------------------------------------------------------------------|
| VFMT    | Change to Vertical Format                                                          |
| HFMT    | Change to Horizontal/Columnar Format                                               |
| CHAR    | Display without formatting (standard FFWB editor; all normal edit commands work)   |
| MAP     | If a structure is defined for the file, change to VFMT view. Else change context to a window to select or define a structure for the file. |

---

## 5. Screen mockups (owner-supplied, verbatim)

### 5.1 View/Edit Dataset Specification (initial screen)

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ Menu  Utilities  Compilers  Options  Status  Help                             │
│ ───────────────────────────────────────────────────────────────────────────── │
│ File-Formatter ----------- View/Edit - Dataset Specification -----------------│
│                                                                               │
│ Edit Mode                  ===> C          (F=Fmt; C=Char; V=Vfmt; U=Unfmt)   │
│ Specify Edit Information:                                                     │
│   Dataset name or zFS path ===> FFWB.A.DATASET                                │
│   Member name              ===>            (Blank or pattern for member list) │
│   Volume serial            ===>            (If dataset is not cataloged)      │
│   Disposition              ===> SHR        (OLD or SHR)                       │
│   Create audit trail       ===> N          (Y = Yes; N = No)                  │
│                                                                               │
│ Specify File Record Strucutres:                                               │
│   Record layout usage      ===> S          (S = Single; X = XREF; N = None)   │
│   Record layout dataset    ===> FFWB.DATASET.METADATA                         │
│   Member name              ===> ADATASET   (Blank or pattern for member list) │
│   XREF dataset name        ===>                                               │
│   Member name              ===>            (Blank or pattern for member list) │
│                                                                               │
│ Specify Filter Criteria:     (E = Existing; T = Temporary;                    │
│   Selection criteria usage ===> N           M = Modify; Q = Quick; N = None)  │
│   Selection dataset name   ===> FFWB.DATASET.STRUCTUR                         │
│   Member name              ===> SELECTCR   (Blank or pattern for member list) │
│                                                                               │
└───────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Vertical format (View/Edit)

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ Menu  Utilities  Compilers  Options  Status  Help                             │
│ ───────────────────────────────────────────────────────────────────────────── │
│File-AID - Edit - FFWB.A.DATASET                       ALREADY AT FIRST RECORD │
│COMMAND ===>                                                  SCROLL ===> CSR  │
│RECORD:       1                     ADATASET-RECORD             LENGTH:     180│
│---- FIELD LEVEL/NAME ------- -FORMAT- ----+----1----+----2----+----3----+----4│
│5 FIELD-NAME-001                3/AN   001                                     │
│5 FIELD-NAME-002                1/AN   2                                       │
│5 FIELD-NAME-003               53/AN   0003                                    │
│                           (POS 41-53)                                         │
│5 FIELD-NAME-004                3/PS   004                                     │
│5 FIELD-NAME-005               16/AN   0604                                    │
│5 FIELD-NAME-006                2/NUM  X'4040'                                 │
│5 FIELD-NAME-007               11/AN                                           │
│5 FIELD-NAME-008               10/AN                                           │
│5 FIELD-NAME-009               15/GRP FIELD-Value-009                          │
│  10 FIELD-NAME-010-of-009      5/AN  FIELD                                    │
│  10 FIELD-NAME-011-of-009     10/AN  -Value-009                               │
│************** END OF DATA - LAYOUT EXCEEDS DATA BY 319 BYTES *****************│
└───────────────────────────────────────────────────────────────────────────────┘
```

Notes on the vertical format:
- Columns: field level + name, FORMAT (e.g. `3/AN`, `1/AN`, `3/PS`, `2/NUM`,
  `15/GRP`), then the value against a ruler.
- Group fields (`GRP`) have subordinate fields at a deeper level (e.g. `10`
  under `5`).
- A field whose data is invalid for its type shows the hex equivalent, e.g.
  `2/NUM  X'4040'`.
- An end-of-data banner reports when the layout exceeds the data length.

---

## 6. Initial design direction (assistant, to seed the session -- NOT decisions)

- Do NOT invent a third parallel record-structure/field-type model. Reuse or
  extend the existing field-model crates -- `ff-forge` (fileforge-integration)
  and/or `ff-structure-catalog` (structure-catalog) -- and let File Formatter be a
  consumer. This aligns with the project-analysis domain-type unification item
  PA-W2.3 (ONE FieldDefinition/RecordStructure owner).
- Structure format: TOML fits FFWB config conventions better than JSON, but this
  is a session decision. Whatever is chosen must express: multiple named record
  types; per-field offset/length/data_type/decimals; a discriminant/identifier
  mechanism; optional per-record-type filters.
- Vertical view = one field per row (level/name, format, value, hex fallback).
  Horizontal/columnar view = one record per row, fields as columns.
- CHAR drops to the standard FFWB editor so all normal edit commands work; MAP
  either switches to VFMT or opens the structure-select/define context when no
  structure is known.
- Right-click "View with FF" / "Edit with FF" hooks into plugin-architecture and
  file-tree-panel.
- Edit-in-place over record-oriented data (VSAM/binary) rides on the
  record-oriented VFS/StorageProvider from CR-NR-016; flat text is simpler.

---

## 7. Open questions for the design session (from CR-NR-061)

- (a) Native structure format: TOML vs JSON; reuse/extend `ff-structure-catalog`
  and `ff-forge` rather than a parallel model (reconcile with PA-W2.3).
- (b) Import sources beyond COBOL copybook: PL/I, assembler DSECT, CSV header,
  hand-authored -- which are in scope and how each maps to the native format.
- (c) Per-file spec storage/discovery: companion file next to the data file vs
  catalog metadata vs a File Formatter config file; how the spec (data file +
  structure + filter) is saved and reloaded.
- (d) Edit-in-place semantics over the record-oriented VFS/StorageProvider
  (CR-NR-016) for VSAM/binary vs flat text; data-type enforcement rules and the
  hex fallback display (e.g. X'4040').
- (e) Relationship to record-selection-criteria (`ff-select`) for the
  filter/selection-criteria usage field.

---

## 8. Sequencing

Per the owner: the UX design session for File Formatter is scheduled BEFORE the
JES emulator and Database tool plugin BUILD work. Tracked as project-master
Phase (file-formatter-design), tasks FFMT-D.1-D.8. The requirements gate
(create `docs/specs/file-formatter/requirements.md`, `design.md`, `tasks.md`;
add TCR rows; add an implementation phase) runs only after the design session
output is approved.
