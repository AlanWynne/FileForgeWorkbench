# Current Work Dashboard

Use this page first to see what the project is working on. Keep the status
summary short and link to the detailed task list.

## Status key

| Status | Meaning |
|--------|---------|
| `ACTIVE` | Work is currently being implemented |
| `NEXT` | Ready to start after the active item |
| `BLOCKED` | Cannot proceed until a dependency or decision is resolved |
| `DONE` | Implemented and validated |
| `DEFERRED` | Deliberately outside the current release |

## Work areas

| Area | Status | Current focus | Detailed tracking |
|------|--------|---------------|-------------------|
| Phase BR -- Requirements Maintenance | DONE | CA-01/CA-02 annotations, FFW-JES rename, B009/CR-NR-035 housekeeping | [project-master tasks](../specs/project-master/tasks.md) |
| Phase BT -- Cross-File Search and Replace | DONE | GlobalReplaceEngine, Replace UI, search history | [global-search tasks](../specs/global-search/tasks.md) |
| Phase BS-A -- Workspace Model | DONE | WorkspaceState, session persistence, lifecycle commands, root management | [workspace-model tasks](../specs/workspace-model/tasks.md) |
| Phase BS-B -- Command Palette | DONE | Fuzzy engine, palette state, rendering, Ctrl+Shift+P activation | [command-palette tasks](../specs/command-palette/tasks.md) |
| Phase BS-C -- Global Search | DONE | ff-global-search crate, Search Results panel, Ctrl+Shift+F, GSEARCH command | [global-search tasks](../specs/global-search/tasks.md) |
| Phase CO -- Accessibility, Plugin Manager UI, Notification System | DONE | All 7 deliverables complete | [project-master tasks](../specs/project-master/tasks.md) |
| Phase CP -- Batch Command Execution | DONE | All 11 deliverables complete | [project-master tasks](../specs/project-master/tasks.md) |
| Phase W.5 -- Generic ToolchainPlugin Trait Validation | DONE | MockToolchain test double, trait audit, CI constraint | [compiler-toolchain-integration tasks](../specs/compiler-toolchain-integration/tasks.md) |
| Phase CQ -- Enterprise Features | DONE | audit-logging, settings export/import, locked config keys | [project-master tasks](../specs/project-master/tasks.md) |
| Phase CR -- OS Theme Follow + Macro Library | DONE | OS dark/light follow, Macro Library panel | [project-master tasks](../specs/project-master/tasks.md) |
| Phase CS -- Test Warning Cleanup | DONE | All cargo test warnings eliminated across workspace | [project-master tasks](../specs/project-master/tasks.md) |
| Phase CT -- Workbench/Workspace/Context Terminology | DONE | All 7 tasks complete -- 69 specs updated | [project-master tasks](../specs/project-master/tasks.md) |
| Phase CU -- Menu Workspace Pattern | DONE | All 8 implementation tasks complete -- TabKind, loader, hot-reload, render, dispatch, defaults | [project-master tasks](../specs/project-master/tasks.md) |
| Phase CV -- POM Redesign | Spec DONE, impl NEXT | Spec complete (CV.1-CV.5). CV-impl pending: DEFAULT_POM_TOML, route options 9/S/B, BUILT_IN_OPTIONS (Tasks 9-12) | [menu-workspace tasks](../specs/menu-workspace/tasks.md) |
| Phase CW -- Settings Menu | Spec DONE, impl NEXT | Spec complete (CW.1-CW.4). CW-impl pending: DEFAULT_SETTINGS_TOML, namespace view routing (Tasks 13-15) | [menu-workspace tasks](../specs/menu-workspace/tasks.md) |
| Phase CX -- Named Workspaces + KEYS + SPLIT | DONE | Spec + implementation complete (CX.1-CX.7) | [project-master tasks](../specs/project-master/tasks.md) |
| Phase CZ -- FFTest Script Suite | DONE | All 9 deliverables complete | [project-master tasks](../specs/project-master/tasks.md) |
| Deferred connectors | DEFERRED | Network, FTP/SFTP, mainframe, and cloud connectors | [connector specs](../specs/) |

## Before starting work

1. Choose one feature from the table above.
2. Read its `requirements.md` and `design.md`.
3. Update its `tasks.md` before implementing.
4. Record any new source material in
   [`../requirements/source-register.md`](../requirements/source-register.md).
5. Update this dashboard when the active focus changes.

## Active work item

**Current focus:** Phase CV-impl -- POM Redesign implementation. The Menu Workspace pattern (CU), POM redesign spec (CV), Settings menu spec (CW), and Named Workspaces (CX) are complete. The CV and CW specs are approved; their implementations (menu-workspace Tasks 9-15) are the next code work. Start with CV-impl Tasks 9-12 (DEFAULT_POM_TOML + option 9/S/B routing).

### Phase CQ -- Enterprise Features (next)

| Deliverable | Spec | Status |
|-------------|------|--------|
| Requirements gate | configuration-system | [x] CQ.1 |
| Audit logging implementation | Structured audit trail | [x] CQ.2 |
| Settings export/import | Save/load user config as portable TOML | [x] CQ.3 |
| Locked config keys | Admin-enforced settings | [x] CQ.4 |
| Integration tests + TCR | All new criteria | [x] CQ.5 |

### Phase CR -- OS Theme Follow + Macro Library Management (next)

| Deliverable | Spec | Status |
|-------------|------|--------|
| Requirements gate | theme-and-appearance Req 16, lua-macro-engine Req 12 | [x] CR.1 |
| OS dark/light mode follow | theme.follow_os config key + frame detection | [x] CR.2 |
| Macro Library panel | TabKind::MacroLibrary, MACROS/=6, list/run/edit/delete | [x] CR.3 |
| Integration tests + TCR | All new criteria | [x] CR.4 |

### Phase CO -- Accessibility, Plugin Manager UI, and Notification System -- COMPLETE

| Deliverable | Spec | Status |
|-------------|------|--------|
| Requirements gate (3 new sub-projects) | accessibility, plugin-manager-ui, notification-system | [x] CO.1-CO.3 |
| Accessibility implementation | WCAG AA, keyboard-only, focus indicators | [x] CO.4 |
| Plugin Manager UI | POM option 8 panel | [x] CO.5 |
| Notification System | Toast + event log | [x] CO.6 |
| Integration tests + TCR | All new criteria | [x] CO.7 |

### Phase CP -- Batch Command Execution -- COMPLETE

**Test count at completion:** 731 passing, 0 failures.

### Phase BS -- Productivity Core -- COMPLETE

| Sub-project | Spec | Tasks | Status |
|-------------|------|-------|--------|
| Workspace Model | [workspace-model](../specs/workspace-model/requirements.md) | BS-A.1 to BS-A.6 | DONE |
| Command Palette | [command-palette](../specs/command-palette/requirements.md) | BS-B.1 to BS-B.4 | DONE |
| Global Search | [global-search](../specs/global-search/requirements.md) | BS-C.1 to BS-C.5 | DONE |

**Test count at completion (Phase BS):** 655 passing, 0 failures.

### Suggested next steps

- Review open bugs in [bugs.md](bugs.md) for a bug-fix sprint.
- Review [change-log.md](change-log.md) for any pending change requests.
- Plan the next feature phase based on the [executive assessment](../reviews/requirements-review/executive-assessment.md).

## Project records

- [Bug register](bugs.md)
- [Change log](change-log.md)
