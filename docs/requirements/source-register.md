# Requirements Source Register

This register maps the source labels used in feature specifications to the
documents or bodies of work from which requirements were derived.

## Internal Sources

| Label | Source | Location or description | Status |
|-------|--------|------------------------|--------|
| `FFE` | FileForgeEditor | Legacy FileForgeEditor requirements and feature specifications | Reference label in current specs |
| `SCI` | Scintilla | Scintilla editor concepts and implementation documentation | Reference label in current specs |
| `SCI-LEX` | Lexilla | Lexer and syntax-highlighting concepts | Reference label in current specs |
| `SCI-STE` | SciTE | Editor configuration and indentation concepts | Reference label in current specs |
| `WB` | Workbench architecture | Workbench platform architecture brief and cross-cutting decisions | Reference label in current specs |
| `DATASET` | Dataset catalog | Dataset catalog and VFS design material | Reference label in current specs |
| `FFE-ASA` | FileForgeEditor ASA | ASA report and print-preview requirements | Reference label in current specs |

## Third-Party Reference Sources

The following external sources were used as behavioural references only.
No source text, code, or documentation from these sources is reproduced
in the repository. All requirements derived from them are re-expressed
in EARS format as independent specifications of FileForge Workbench behaviour.

| Label | Source | Notes | IP Status |
|-------|--------|-------|-----------|
| `JES` | IBM z/OS JES2/JES3 concepts | Behavioural reference for FFW-JES emulator. JES2/JES3 are IBM trademarks. No IBM code or documentation text is reproduced. | Reference only -- no IBM content in repo |
| `ISPF` | IBM z/OS ISPF | Behavioural reference for ISPF-style UI, command semantics, and edit operations. ISPF is an IBM trademark. No IBM code or documentation text is reproduced. | Reference only -- no IBM content in repo |
| `TSO` | IBM z/OS TSO/E | Behavioural reference for TSO command syntax and session model. TSO/E is an IBM trademark. No IBM code or documentation text is reproduced. | Reference only -- no IBM content in repo |
| `SDSF` | IBM z/OS SDSF | Behavioural reference for SDSF-style job monitor panels. SDSF is an IBM trademark. No IBM code or documentation text is reproduced. | Reference only -- no IBM content in repo |
| `IDCAMS` | IBM z/OS IDCAMS (Access Method Services) | Behavioural reference for IDCAMS command syntax and return codes. IDCAMS is an IBM product. No IBM code or documentation text is reproduced. | Reference only -- no IBM content in repo |
| `DBV` | DBeaver Community Edition | Behavioural reference for database tool features. DBeaver is Apache 2.0 licensed. Requirements re-expressed in EARS format; no DBeaver source code used. | Apache 2.0 reference -- no code copied |

## IBM Manuals -- Excluded from Repository

The following IBM manuals were used locally as reference material during
requirements derivation. They are listed in `.gitignore` and are NOT
committed to the repository. They must not be redistributed.

| Manual | IBM Publication Number | Notes |
|--------|----------------------|-------|
| ISPF User's Guide Volume I | SC19-3627 | z/OS ISPF V2 |
| ISPF User's Guide Volume II | SC19-3628 | z/OS ISPF V2 |
| ISPF Edit and Edit Macros | SC19-3621 | z/OS ISPF V2 |
| ISPF Services Guide | SC19-3626 | z/OS ISPF V2 |
| ISPF Dialog Developer's Guide and Reference | SC19-3619 | z/OS ISPF V2 |
| ISPF Dialog Tag Language Guide and Reference | SC19-3620 | z/OS ISPF V2 |
| ISPF Messages and Codes | SC19-3622 | z/OS ISPF V2 |
| ISPF Planning and Customizing | GC19-3629 | z/OS ISPF V2 |
| ISPF SCLM Guide and Reference | SC19-3630 | z/OS ISPF V2 |
| TSO/E Command Reference | SA32-0975 | z/OS TSO/E V3R1 |
| TSO/E User's Guide | SA32-0971 | z/OS TSO/E V3R1 |
| TSO/E Administration | SA32-0977 | z/OS TSO/E V3R2 |
| TSO/E REXX Reference | SA32-0972 | z/OS TSO/E V3R2 |
| TSO/E REXX User's Guide | SA32-0981 | z/OS TSO/E V3R1 |
| TSO/E CLISTs | SA32-0978 | z/OS TSO/E V3R1 |
| TSO/E General Information | GA32-0969 | z/OS TSO/E V3R1 |
| TSO/E Messages | SA32-0970 | z/OS TSO/E V3R1 |
| TSO/E Primer | SA32-0984 | z/OS TSO/E V3R1 |
| TSO/E Programming Guide | SA32-0973 | z/OS TSO/E V3R1 |
| TSO/E Programming Services | SA32-0974 | z/OS TSO/E V3R1 |
| TSO/E System Programming Command Reference | SA32-0976 | z/OS TSO/E V3R1 |
| TSO/E System Diagnosis Data Areas | GA32-0983 | z/OS TSO/E V3R1 |
| TSO/E Customization | SA32-0979 | z/OS TSO/E V3R1 |
| SDSF Operation and Customization | SA23-2274 | z/OS SDSF V3R1 |
| SDSF User's Guide | SA23-2273 | z/OS SDSF V3R1 |
| MVS JCL Reference | SA23-1385 | z/OS MVS |
| Advanced Catalog | (IBM internal reference) | z/OS catalog concepts |

## Repository documents to classify

These documents are currently retained in their existing locations until their
links and provenance are checked:

- [`../source-documents/dataset-catalog/FileForgeWorkbench_Mainframe_Dataset_Architecture.md`](../source-documents/dataset-catalog/FileForgeWorkbench_Mainframe_Dataset_Architecture.md)
- [`../source-documents/dataset-catalog/FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md`](../source-documents/dataset-catalog/FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md)
- [`../specs/workbench-requirements-merge/dataset-catalog-brief.md`](../specs/workbench-requirements-merge/dataset-catalog-brief.md)
- [`../specs/workbench-requirements-merge/architecture-brief.md`](../specs/workbench-requirements-merge/architecture-brief.md)

## Adding a source

For each new source, record:

1. A stable label used in requirements.
2. The original title and author or origin.
3. The repository path under `source-documents/`.
4. The version or capture date.
5. The specifications that use it.

Requirements should cite the stable label, while this register provides the
human-readable provenance.
