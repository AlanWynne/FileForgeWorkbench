# Implementation Plan -- Project Analysis and Re-Structuring

Systematic, dependency-ordered analysis of all 79 sub-projects. Each sub-project
gets one analysis task (the five checks: split, consistency, completeness,
logging, task-revision) followed by a commit-and-push task (Requirement 7). The
analysis is non-destructive: it writes analysis records + revised task ordering,
never `crates/` source (Requirement 8).

Legend: each analysis task produces `units/<name>.md` and appends rows to
`checklist.md`, `consistency-matrix.md`, and `incomplete-work-register.md` per
the Analysis Record template (design section 4). The commit task stages only that
sub-project's analysis files (design section 9).

Baseline confirmed: tree compiles, `verify.ps1` CLEAN, `main` synced.

---

## Task 0: Setup

- [ ] 0.1 Create the analysis scaffolding: `checklist.md` (all 79 sub-projects =
      PENDING), empty `consistency-matrix.md` (section headers per design section 6),
      empty `incomplete-work-register.md` (extends EI-3 audit; header links to it)
- [ ] 0.2 Register `project-analysis` in `docs/specs/specs.md`; log CR-NR-056 in
      `docs/status/change-log.md`
- [ ] 0.3 Commit + push: `analysis: scaffolding + CR-NR-056`

---

## Wave 0 -- Foundations

- [ ] W0.1 Analyse `platform-core` (record + registers)
- [ ] W0.2 Commit + push `analysis(platform-core)`
- [ ] W0.3 Analyse `configuration-system`
- [ ] W0.4 Commit + push `analysis(configuration-system)`
- [ ] W0.5 Analyse `logging-subsystem`
- [ ] W0.6 Commit + push `analysis(logging-subsystem)`
- [ ] W0.7 Analyse `command-framework`
- [ ] W0.8 Commit + push `analysis(command-framework)`
- [ ] W0.9 Analyse `document-model`
- [ ] W0.10 Commit + push `analysis(document-model)`
- [ ] W0.11 Analyse `virtual-file-system`
- [ ] W0.12 Commit + push `analysis(virtual-file-system)`
- [ ] W0.13 Analyse `plugin-architecture`
- [ ] W0.14 Commit + push `analysis(plugin-architecture)`
- [ ] W0.15 Analyse `workflow-engine`
- [ ] W0.16 Commit + push `analysis(workflow-engine)`
- [ ] W0.17 Analyse `background-io`
- [ ] W0.18 Commit + push `analysis(background-io)`
- [ ] W0.19 Analyse `encoding-and-characters`
- [ ] W0.20 Commit + push `analysis(encoding-and-characters)`
- [ ] W0.21 Wave 0 task-revision pass: re-order Wave 0 incomplete tasks in
      project-master; commit + push `analysis(wave0): task re-order`

---

## Wave 1 -- Core editing and model

- [ ] W1.1 Analyse `undo-redo-transactions` + commit
- [ ] W1.2 Analyse `viewport-and-scrolling` + commit
- [ ] W1.3 Analyse `caret-and-selection` + commit
- [ ] W1.4 Analyse `display-line-mapping` + commit
- [ ] W1.5 Analyse `edit-operations` + commit
- [ ] W1.6 Analyse `line-commands` + commit
- [ ] W1.7 Analyse `navigation-commands` + commit
- [ ] W1.8 Analyse `find-and-replace` + commit
- [ ] W1.9 Analyse `sequence-numbers` + commit
- [ ] W1.10 Analyse `syntax-highlighting` + commit
- [ ] W1.11 Analyse `exclude-show-filter` + commit
- [ ] W1.12 Analyse `whitespace-and-guides` + commit
- [ ] W1.13 Analyse `auto-indentation` + commit
- [ ] W1.14 Analyse `line-wrap-toggle` + commit
- [ ] W1.15 Analyse `text-decorations` + commit
- [ ] W1.16 Wave 1 task-revision pass + commit

---

## Wave 2 -- Catalog and dataset (split-candidate heavy)

- [ ] W2.1 Analyse `dataset-catalog` (516 lines -- split candidate) + commit
- [ ] W2.2 Analyse `virtual-catalog-manager` (443 -- split candidate) + commit
- [ ] W2.3 Analyse `dataset-allocator` + commit
- [ ] W2.4 Analyse `dataset-ownership-model` + commit
- [ ] W2.5 Analyse `structure-catalog` + commit
- [ ] W2.6 Analyse `record-selection-criteria` + commit
- [ ] W2.7 Analyse `tabs-and-mask` + commit
- [ ] W2.8 Wave 2 task-revision pass + commit

---

## Wave 3 -- Shell, commands, menus, session

- [ ] W3.1 Analyse `command-semantics` + commit
- [ ] W3.2 Analyse `command-completion` + commit
- [ ] W3.3 Analyse `command-palette` + commit
- [ ] W3.4 Analyse `command-configurator` + commit
- [ ] W3.5 Analyse `menu-workspace` + commit
- [ ] W3.6 Analyse `menu-and-statusbar` + commit
- [ ] W3.7 Analyse `function-keys-and-history` + commit
- [ ] W3.8 Analyse `shell-command` + commit
- [ ] W3.9 Analyse `startup-and-session` (388 -- split candidate) + commit
- [ ] W3.10 Analyse `workspace-model` + commit
- [ ] W3.11 Wave 3 task-revision pass + commit

---

## Wave 4 -- UI, panels, layout

- [ ] W4.1 Analyse `layout-and-docking` + commit
- [ ] W4.2 Analyse `multi-tab-editor` + commit
- [ ] W4.3 Analyse `file-tree-panel` (674 -- top split candidate) + commit
- [ ] W4.4 Analyse `theme-and-appearance` + commit
- [ ] W4.5 Analyse `view-zoom` + commit
- [ ] W4.6 Analyse `hex-display` + commit
- [ ] W4.7 Analyse `notification-system` + commit
- [ ] W4.8 Analyse `plugin-manager-ui` + commit
- [ ] W4.9 Analyse `accessibility` + commit
- [ ] W4.10 Analyse `context-help` + commit
- [ ] W4.11 Analyse `clipboard-operations` + commit
- [ ] W4.12 Analyse `idle-processing` + commit
- [ ] W4.13 Analyse `large-file-performance` + commit
- [ ] W4.14 Analyse `external-modification` + commit
- [ ] W4.15 Analyse `file-operations` + commit
- [ ] W4.16 Wave 4 task-revision pass + commit

---

## Wave 5 -- Emulators, tools, connectors, integrations

- [ ] W5.1 Analyse `jes-emulator` (398 -- split candidate) + commit
- [ ] W5.2 Analyse `idcams-emulator` (410 -- split candidate) + commit
- [ ] W5.3 Analyse `jcl-resolver` + commit
- [ ] W5.4 Analyse `database-tool` (352 -- split candidate) + commit
- [ ] W5.5 Analyse `compiler-toolchain-integration` + commit
- [ ] W5.6 Analyse `batch-execution` + commit
- [ ] W5.7 Analyse `asa-report-preview` + commit
- [ ] W5.8 Analyse `lua-macro-engine` + commit
- [ ] W5.9 Analyse `language-service` + commit
- [ ] W5.10 Analyse `custom-file-viewers` + commit
- [ ] W5.11 Analyse `compare-and-merge` + commit
- [ ] W5.12 Analyse `global-search` + commit
- [ ] W5.13 Analyse `connector-extensibility` + commit
- [ ] W5.14 Analyse `connector-local-fs` + commit
- [ ] W5.15 Analyse `connector-network-fs` + commit
- [ ] W5.16 Analyse `connector-ftp-sftp` + commit
- [ ] W5.17 Analyse `connector-cloud` + commit
- [ ] W5.18 Analyse `connector-mainframe` + commit
- [ ] W5.19 Analyse `fileforge-integration` + commit
- [ ] W5.20 Wave 5 task-revision pass + commit

---

## Wave 6 -- Meta and consolidation

- [ ] W6.1 Analyse `project-master` (aggregate ordering) + commit
- [ ] W6.2 Review `workbench-requirements-merge` and `ears-integration` (analysis
      artifacts, not shippable units); note stale sections + commit
- [ ] W6.3 Finalize `consistency-matrix.md` (all conflicts + proposed resolutions)
- [ ] W6.4 Finalize `incomplete-work-register.md` (full backlog, dependency-ordered)
- [ ] W6.5 Produce the revised master task ordering in `project-master/tasks.md`
- [ ] W6.6 Produce a decisions-for-owner summary (all split proposals, conflict
      reconciliations, superseded tasks, new-requirement findings)
- [ ] W6.7 Final consolidation commit + push `analysis: consolidation`

---

## Notes

- Each "Analyse X + commit" line is two operations: write the Analysis_Record and
  register rows, then commit + push staging only that unit's files (Requirement 7).
- Split proposals, conflict reconciliations, and new-requirement findings are
  RECORDED, not executed; each is surfaced for owner approval and (where needed)
  its own requirements gate (Requirement 8.5).
- Wave membership may be adjusted mid-analysis if a dependency is discovered; any
  change is noted in `checklist.md`.
- The clean `verify.ps1` baseline is the reference for completeness checks; if a
  later code change breaks it, re-establish green before continuing completeness
  classification.
