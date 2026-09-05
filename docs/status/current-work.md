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
| Phase BS-A -- Workspace Model | DONE | WorkspaceState, session persistence, lifecycle commands, root management | [workspace-model tasks](../specs/workspace-model/tasks.md) |
| Phase BS-B -- Command Palette | DONE | Fuzzy engine, palette state, rendering, Ctrl+Shift+P activation | [command-palette tasks](../specs/command-palette/tasks.md) |
| Phase BS-C -- Global Search | DONE | ff-global-search crate, Search Results panel, Ctrl+Shift+F, GSEARCH command | [global-search tasks](../specs/global-search/tasks.md) |
| Deferred connectors | DEFERRED | Network, FTP/SFTP, mainframe, and cloud connectors | [connector specs](../specs/) |

## Before starting work

1. Choose one feature from the table above.
2. Read its `requirements.md` and `design.md`.
3. Update its `tasks.md` before implementing.
4. Record any new source material in
   [`../requirements/source-register.md`](../requirements/source-register.md).
5. Update this dashboard when the active focus changes.

## Active work item

**Current focus:** No active work item. Phase BS (Productivity Core) is complete.

### Phase BS -- Productivity Core -- COMPLETE

| Sub-project | Spec | Tasks | Status |
|-------------|------|-------|--------|
| Workspace Model | [workspace-model](../specs/workspace-model/requirements.md) | BS-A.1 to BS-A.6 | DONE |
| Command Palette | [command-palette](../specs/command-palette/requirements.md) | BS-B.1 to BS-B.4 | DONE |
| Global Search | [global-search](../specs/global-search/requirements.md) | BS-C.1 to BS-C.5 | DONE |

**Test count at completion:** 655 passing (644 ff-desktop + 11 ff-global-search), 0 failures.

### Suggested next steps

- Review open bugs in [bugs.md](bugs.md) for a bug-fix sprint.
- Review [change-log.md](change-log.md) for any pending change requests.
- Plan the next feature phase based on the [executive assessment](../reviews/requirements-review/executive-assessment.md).

## Project records

- [Bug register](bugs.md)
- [Change log](change-log.md)
