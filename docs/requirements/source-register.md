# Requirements Source Register

This register maps the source labels used in feature specifications to the
documents or bodies of work from which requirements were derived.

| Label | Source | Location or description | Status |
|-------|--------|------------------------|--------|
| `FFE` | FileForgeEditor | Legacy FileForgeEditor requirements and feature specifications | Reference label in current specs |
| `SCI` | Scintilla | Scintilla editor concepts and implementation documentation | Reference label in current specs |
| `SCI-LEX` | Lexilla | Lexer and syntax-highlighting concepts | Reference label in current specs |
| `SCI-STE` | SciTE | Editor configuration and indentation concepts | Reference label in current specs |
| `WB` | Workbench architecture | Workbench platform architecture brief and cross-cutting decisions | Reference label in current specs |
| `DATASET` | Dataset catalog | Dataset catalog and VFS design material | Reference label in current specs |
| `FFE-ASA` | FileForgeEditor ASA | ASA report and print-preview requirements | Reference label in current specs |

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
