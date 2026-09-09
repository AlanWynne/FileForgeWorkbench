# Project Analysis Checklist

Status of every sub-project in the systematic analysis pass (CR-NR-056).
Status values: PENDING / IN PROGRESS / DONE. Wave is the planned analysis wave
(design section 3); it may be re-slotted mid-analysis with a recorded reason.

`project-analysis` itself is excluded (it is the analysis, not a subject).
`ears-integration` and `workbench-requirements-merge` are analysis-artifact
sub-projects reviewed in Wave 6 (review only).

| # | Sub-project | Wave | Status | Analysis Record | Notes |
|---|-------------|------|--------|-----------------|-------|
| 1 | platform-core | 0 | DONE | units/platform-core.md | COMPLETE; not a split candidate; PA-LOG-001, PA-DOC-001 logged |
| 2 | configuration-system | 0 | DONE | units/configuration-system.md | COMPLETE; SPLIT CANDIDATE (spec split proposed PA-SPLIT-001); PA-STD-001, PA-DOC-002 logged |
| 3 | logging-subsystem | 0 | DONE | units/logging-subsystem.md | FUNCTIONALLY COMPLETE; not a split candidate; Req 12 tool done but tracking stale (PA-TRACK-001); PA-LOG-002 = project-wide audit data source |
| 4 | command-framework | 0 | DONE | units/command-framework.md | INCOMPLETE (Req 9 Command Arguments unimplemented, PA-INCOMPLETE-001); not a split candidate; Command_Target fan-out watch PA-WATCH-001 |
| 5 | document-model | 0 | DONE | units/document-model.md | COMPLETE (140/140 tasks, TCR PASS); not a split candidate; VFS-only verified; PA-WATCH-002 (document.rs at 400 cap); PA-LOG-003 (optional logging) |
| 6 | virtual-file-system | 0 | PENDING | -- | foundation |
| 7 | plugin-architecture | 0 | PENDING | -- | foundation |
| 8 | workflow-engine | 0 | PENDING | -- | foundation |
| 9 | background-io | 0 | PENDING | -- | foundation |
| 10 | encoding-and-characters | 0 | PENDING | -- | foundation |
| 11 | undo-redo-transactions | 1 | PENDING | -- | core editing/model |
| 12 | viewport-and-scrolling | 1 | PENDING | -- | core editing/model |
| 13 | caret-and-selection | 1 | PENDING | -- | core editing/model |
| 14 | display-line-mapping | 1 | PENDING | -- | core editing/model |
| 15 | edit-operations | 1 | PENDING | -- | core editing/model |
| 16 | line-commands | 1 | PENDING | -- | core editing/model |
| 17 | navigation-commands | 1 | PENDING | -- | core editing/model |
| 18 | find-and-replace | 1 | PENDING | -- | core editing/model |
| 19 | sequence-numbers | 1 | PENDING | -- | core editing/model |
| 20 | syntax-highlighting | 1 | PENDING | -- | core editing/model |
| 21 | exclude-show-filter | 1 | PENDING | -- | core editing/model |
| 22 | whitespace-and-guides | 1 | PENDING | -- | core editing/model |
| 23 | auto-indentation | 1 | PENDING | -- | core editing/model |
| 24 | line-wrap-toggle | 1 | PENDING | -- | core editing/model |
| 25 | text-decorations | 1 | PENDING | -- | core editing/model |
| 26 | dataset-catalog | 2 | PENDING | -- | split candidate (516) |
| 27 | virtual-catalog-manager | 2 | PENDING | -- | split candidate (443) |
| 28 | dataset-allocator | 2 | PENDING | -- | catalog/dataset |
| 29 | dataset-ownership-model | 2 | PENDING | -- | catalog/dataset |
| 30 | structure-catalog | 2 | PENDING | -- | catalog/dataset |
| 31 | record-selection-criteria | 2 | PENDING | -- | catalog/dataset |
| 32 | tabs-and-mask | 2 | PENDING | -- | catalog/dataset |
| 33 | command-semantics | 3 | PENDING | -- | shell/commands/menus |
| 34 | command-completion | 3 | PENDING | -- | shell/commands/menus |
| 35 | command-palette | 3 | PENDING | -- | shell/commands/menus |
| 36 | command-configurator | 3 | PENDING | -- | shell/commands/menus |
| 37 | menu-workspace | 3 | PENDING | -- | shell/commands/menus |
| 38 | menu-and-statusbar | 3 | PENDING | -- | shell/commands/menus |
| 39 | function-keys-and-history | 3 | PENDING | -- | shell/commands/menus |
| 40 | shell-command | 3 | PENDING | -- | shell/commands/menus |
| 41 | startup-and-session | 3 | PENDING | -- | split candidate (388) |
| 42 | workspace-model | 3 | PENDING | -- | shell/commands/menus |
| 43 | layout-and-docking | 4 | PENDING | -- | UI/panels |
| 44 | multi-tab-editor | 4 | PENDING | -- | UI/panels |
| 45 | file-tree-panel | 4 | PENDING | -- | top split candidate (674) |
| 46 | theme-and-appearance | 4 | PENDING | -- | UI/panels |
| 47 | view-zoom | 4 | PENDING | -- | UI/panels |
| 48 | hex-display | 4 | PENDING | -- | UI/panels |
| 49 | notification-system | 4 | PENDING | -- | UI/panels |
| 50 | plugin-manager-ui | 4 | PENDING | -- | UI/panels |
| 51 | accessibility | 4 | PENDING | -- | UI/panels |
| 52 | context-help | 4 | PENDING | -- | UI/panels |
| 53 | clipboard-operations | 4 | PENDING | -- | UI/panels |
| 54 | idle-processing | 4 | PENDING | -- | UI/panels |
| 55 | large-file-performance | 4 | PENDING | -- | UI/panels |
| 56 | external-modification | 4 | PENDING | -- | UI/panels |
| 57 | file-operations | 4 | PENDING | -- | UI/panels |
| 58 | jes-emulator | 5 | PENDING | -- | split candidate (398) |
| 59 | idcams-emulator | 5 | PENDING | -- | split candidate (410) |
| 60 | jcl-resolver | 5 | PENDING | -- | stub -- no requirements yet |
| 61 | database-tool | 5 | PENDING | -- | split candidate (352) |
| 62 | compiler-toolchain-integration | 5 | PENDING | -- | tools |
| 63 | batch-execution | 5 | PENDING | -- | tools |
| 64 | asa-report-preview | 5 | PENDING | -- | tools |
| 65 | lua-macro-engine | 5 | PENDING | -- | tools |
| 66 | language-service | 5 | PENDING | -- | tools |
| 67 | custom-file-viewers | 5 | PENDING | -- | tools |
| 68 | compare-and-merge | 5 | PENDING | -- | tools |
| 69 | global-search | 5 | PENDING | -- | tools |
| 70 | connector-extensibility | 5 | PENDING | -- | connectors |
| 71 | connector-local-fs | 5 | PENDING | -- | connectors |
| 72 | connector-network-fs | 5 | PENDING | -- | connectors |
| 73 | connector-ftp-sftp | 5 | PENDING | -- | connectors |
| 74 | connector-cloud | 5 | PENDING | -- | connectors |
| 75 | connector-mainframe | 5 | PENDING | -- | connectors |
| 76 | fileforge-integration | 5 | PENDING | -- | integrations |
| 77 | automated-dialog-testing | 5 | PENDING | -- | test framework (FFTest) -- added to Wave 5 |
| 78 | bootstrap-scripts | 5 | PENDING | -- | dev tooling scripts -- added to Wave 5 |
| 79 | project-master | 6 | PENDING | -- | aggregate ordering (meta) |
| 80 | workbench-requirements-merge | 6 | PENDING | -- | analysis artifact -- review only |
| 81 | ears-integration | 6 | PENDING | -- | analysis artifact -- review only |

## Wave re-slot notes

- `automated-dialog-testing` and `bootstrap-scripts` were not in the original
  design wave lists; slotted into Wave 5 (tools) here. Recorded per Requirement 1
  and design section 3 (wave membership may be adjusted).
- Total subjects: 81 folders under `docs/specs/`, minus `project-analysis`
  (the analysis itself) = 80 analysed here; `project-master`,
  `workbench-requirements-merge`, and `ears-integration` are meta/review entries.
