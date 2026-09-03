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
| Workspace and project master | ACTIVE | Keep implementation status and dependencies current | [project-master tasks](../specs/project-master/tasks.md) |
| Feature specifications | NEXT | Select the next incomplete feature task | [feature specs](../specs/) |
| Requirements quality | NEXT | Resolve review findings and maintain traceability | [requirements review](../specs/requirements-review/) |
| Deferred connectors | DEFERRED | Network, FTP/SFTP, mainframe, and cloud connectors | [connector specs](../specs/) |

## Before starting work

1. Choose one feature from the table above.
2. Read its `requirements.md` and `design.md`.
3. Update its `tasks.md` before implementing.
4. Record any new source material in
   [`../requirements/source-register.md`](../requirements/source-register.md).
5. Update this dashboard when the active focus changes.

## Active work item

**Current focus:** Select and record the next implementation task from the
project-master dashboard.

**Blockers:** None recorded here. Feature-specific blockers belong in the
relevant `tasks.md` file.
