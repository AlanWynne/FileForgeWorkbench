# Project Analysis Checklist

Status of every sub-project in the systematic analysis pass (CR-NR-056).
Status values: PENDING / IN PROGRESS / DONE. Wave is the planned analysis wave
(design section 3); it may be re-slotted mid-analysis with a recorded reason.

`project-analysis` itself is excluded (it is the analysis, not a subject).
`ears-integration` and `workbench-requirements-merge` are analysis-artifact
sub-projects reviewed in Wave 6 (review only).

**RE-BASELINED (restart) after CR-NR-057** (command chaining + Context Navigation
Stack, commit `b537737`). CR-NR-057 changed foundational core specs
(command-framework Req 10, command-semantics, lua-macro-engine, menu-workspace),
so the prior analysis pass (W0.1-W1.5, committed through `edb690d`) is
superseded. All sub-projects reset to PENDING; re-run from W0.1 against the new
baseline. The prior records remain in git history (up to `edb690d`) for
reference but are NOT carried forward.

**Wave 0 COMPLETE (W0.1-W0.21) on the re-baseline** (commits `7330b06`..`df79cd0`).
All 10 foundation sub-projects analysed against the CR-NR-057 + CR-NR-058 specs;
the Wave 0 task-revision (W0.21) recorded the dependency-ordered remediation as
`Phase PA-W0` (PROPOSAL, PA-W0.1-PA-W0.23) in
`docs/specs/project-master/tasks.md`. Wave 0 outcome: 3 clean/complete
(platform-core, document-model, plugin-architecture); 3 complete-with-proposals
(configuration-system split, virtual-file-system IWR-005-resolved,
encoding-and-characters borderline split); command-framework INCOMPLETE on THREE
gated-but-unbuilt reqs (Req 9 args, Req 10 nav-stack CR-NR-057, Req 11
per-command instrumentation CR-NR-058); workflow-engine (Req 7.6 + tokio::fs);
virtual-file-system (PA-LOG-004 eprintln defect); background-io (Req 6.6-6.9
unimplemented, false-positive tests, HIGH); 2 tracking gaps (config CQ tasks
30-31, logging Req 12 tool); logging-subsystem gained CR-NR-058 Req 13
(build-profile dev-logging gate, UNIMPLEMENTED). Top remediation priority: the
CR-NR-058 dev/debug-logging foundation (PA-W0.1 gate then PA-W0.2 per-command
instrumentation).

| # | Sub-project | Wave | Status | Analysis Record | Notes |
|---|-------------|------|--------|-----------------|-------|
| 1 | platform-core | 0 | DONE | units/platform-core.md | COMPLETE (100/100, TCR PASS); not a split; GUI-independence VERIFIED; logging EXEMPLARY; refactor PA-STD-001 (event_bus.rs 435); PA-DOC-001 (Req 4.1 crate-name drift); PA-LOG-001 (project-wide non-ASCII) |
| 2 | configuration-system | 0 | DONE | units/configuration-system.md | FUNCTIONALLY COMPLETE; SPLIT CANDIDATE (PA-SPLIT-001, 4/4); TRACKING GAP PA-TRACK-001 (Phase CQ tasks 30-31 stale, code+TCR done); refactor PA-STD-002 (6 files); PA-DOC-002 (Req 10-14 gap) |
| 3 | logging-subsystem | 0 | DONE | units/logging-subsystem.md | FUNCTIONALLY COMPLETE; Req 12 tool done but tracking stale (PA-TRACK-002); CR-NR-058 Req 13 build-profile gate IMPL DONE + verified (commit 4bf6fbc, tasks 25.1-25.4) but tests 25.6/25.7 + TCR + release-wiring 25.5 PENDING (PA-CR058); PA-LOG-002 audit data; refactor PA-STD-003 |
| 4 | command-framework | 0 | DONE | units/command-framework.md | INCOMPLETE (3 gaps): Req 9 Command Arguments (PA-INCOMPLETE-001) + Req 10 Context Navigation Stack CR-NR-057 (PA-INCOMPLETE-003) + Req 11 Uniform Command Instrumentation CR-NR-058 (PA-INCOMPLETE-004, HIGH-leverage dev/debug logging), all gate-complete/unbuilt; Req 1-8 done; not a split |
| 5 | document-model | 0 | DONE | units/document-model.md | COMPLETE (140/140, TCR PASS); not a split; VFS-only verified; PA-WATCH-002 (document.rs at 400 cap); PA-LOG-003 (optional logging); zero-log defensible |
| 6 | virtual-file-system | 0 | DONE | units/virtual-file-system.md | Feature-COMPLETE (91/91, Req 1-12 TCR-PASS); IWR-005 RESOLVED; LOGGING DEFECT PA-LOG-004 PERSISTS (eprintln + unmet Req 3.3 WARN); weak split PA-SPLIT-002; PA-WATCH-003 |
| 7 | plugin-architecture | 0 | DONE | units/plugin-architecture.md | COMPLETE (169/169, TCR PASS); not a split; logging EXEMPLARY (PA-LOG-REF-001); refactor PA-STD-004 (registry.rs 698) |
| 8 | workflow-engine | 0 | DONE | units/workflow-engine.md | Tracking-COMPLETE (124/124, TCR PASS); not a split; LOGGING GAP PA-LOG-005 (Req 7.6 unmet, checkpoint silent errors); FFW-ARCH-001 watch PA-WATCH-004 (checkpoint tokio::fs); refactor PA-STD-005 |
| 9 | background-io | 0 | DONE | units/background-io.md | INCOMPLETE despite 133/133 tasks: Req 6.6-6.9 (error log/retry/resume) NOT in load path (PA-INCOMPLETE-002, false-positive tests, HIGH); no TCR rows (PA-TCR-001); FFW-ARCH-001 verified upheld; not a split candidate |
| 10 | encoding-and-characters | 0 | DONE | units/encoding-and-characters.md | COMPLETE (111/111, TCR PASS); BORDERLINE SPLIT (PA-SPLIT-003, 14 reqs/3 bands); zero-log CORRECT (spec-mandated stateless); convert.rs refactor PA-STD-006; PA-WATCH-005 |
| 11 | undo-redo-transactions | 1 | DONE | units/undo-redo-transactions.md | COMPLETE (159/159, TCR PASS); SPLIT CANDIDATE (PA-SPLIT-004, 19 reqs/553 lines); logging gap PA-LOG-006 (Req 3.5 WARNING); refactor PA-STD-007 (manager.rs 600); crate-name drift PA-DOC-003; PA-CONFLICT-002 (two transaction models, resolve W1.5) |
| 12 | viewport-and-scrolling | 1 | DONE | units/viewport-and-scrolling.md | COMPLETE (129/129, TCR PASS); borderline (14 reqs) but NOT a split -- fix is refactor PA-STD-008 (viewport.rs 572); CONFLICT PA-CONFLICT-001 (own DisplayLineMapper trait, resolve W1.4); zero-log defensible; PA-DOC-004; watch PA-WATCH-008 |
| 13 | caret-and-selection | 1 | DONE | units/caret-and-selection.md | COMPLETE (127/127, TCR PASS); NOT a split (cohesive rendering layer, well-factored, no file over cap); zero-log defensible; PA-WATCH-006 partially resolved (visual layer, no logical-model duplication; final check W1.5) |
| 14 | display-line-mapping | 1 | DONE | units/display-line-mapping.md | COMPLETE (87/87, TCR PASS); not a split; CONFLICT PA-CONFLICT-001 CONFIRMED (viewport's own DisplayLineMapper vs canonical DisplayLineMapping owned here); refactor PA-STD-009 (contraction_state.rs 614); zero-log defensible |
| 15 | edit-operations | 1 | DONE | units/edit-operations.md | COMPLETE (280/280, TCR PASS); SPLIT CANDIDATE (PA-SPLIT-005, 17 reqs/553 lines); PA-WATCH-006 RESOLVED (selection ownership clean); CONFLICT PA-CONFLICT-002 CONFIRMED (EditorTransaction de-facto canonical vs unwired undo-redo model); zero-log defensible |
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
| 33 | command-semantics | 3 | PENDING | -- | shell/commands/menus; CR-NR-057 change |
| 34 | command-completion | 3 | PENDING | -- | shell/commands/menus |
| 35 | command-palette | 3 | PENDING | -- | shell/commands/menus |
| 36 | command-configurator | 3 | PENDING | -- | shell/commands/menus |
| 37 | menu-workspace | 3 | PENDING | -- | shell/commands/menus; CR-NR-057 change |
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
| 65 | lua-macro-engine | 5 | PENDING | -- | tools; CR-NR-057 change |
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
- **CR-NR-057 re-baseline**: restart triggered because CR-NR-057 modified
  foundational specs (command-framework Req 10 Context Navigation Stack,
  command-semantics, lua-macro-engine, menu-workspace). Prior pass superseded at
  commit `edb690d`.
