---
inclusion: always
---

# Project Specifications and Task Format

All feature specifications live under `docs/specs/<sub-project>/`. Each folder
contains up to three files:

| File | Purpose |
|------|---------|
| `requirements.md` | Acceptance criteria in EARS format (`WHEN ... THE ... SHALL ...`). Source of truth for what must be implemented and tested. |
| `design.md` | Architecture and design decisions for the sub-project. |
| `tasks.md` | Ordered implementation tasks with `[ ]` / `[x]` state. |

## Sub-projects
The following specs exist under `docs/specs/`:

- asa-report-preview
- automated-dialog-testing
- batch-execution
- bootstrap-scripts
- auto-indentation
- background-io
- caret-and-selection
- clipboard-operations
- command-completion
- command-configurator
- command-framework
- command-semantics
- command-palette
- compare-and-merge
- compiler-toolchain-integration
- configuration-system
- connector-cloud
- connector-extensibility
- connector-ftp-sftp
- connector-local-fs
- connector-mainframe
- connector-network-fs
- context-help
- custom-file-viewers
- database-tool
- dataset-allocator
- dataset-catalog
- dataset-ownership-model
- display-line-mapping
- document-model
- ears-integration (workflow.md, and outputs: minix-ftso-reconciliation.md, source-of-truth-map.md, gap-analysis.md, coverage-classification.md, incomplete-work-audit.md, integration-plan.md)
- edit-operations
- encoding-and-characters
- exclude-show-filter
- external-modification
- global-search
- jes-emulator (folder was FFW-JES -- renamed Phase BR to match kebab-case convention)
- file-operations
- file-tree-panel
- fileforge-integration
- find-and-replace
- function-keys-and-history
- hex-display
- idcams-emulator
- idle-processing
- language-service
- large-file-performance
- layout-and-docking
- line-commands
- line-wrap-toggle
- logging-subsystem
- lua-macro-engine
- menu-and-statusbar
- multi-tab-editor
- navigation-commands
- platform-core
- plugin-architecture
- project-master
- record-selection-criteria
- sequence-numbers
- shell-command
- startup-and-session
- structure-catalog
- syntax-highlighting
- tabs-and-mask
- text-decorations
- theme-and-appearance
- undo-redo-transactions
- view-zoom
- viewport-and-scrolling
- virtual-catalog-manager
- virtual-file-system
- whitespace-and-guides
- workbench-requirements-merge (architecture docs and validation reports -- no deliverable crate)
- workflow-engine
- workspace-model
- accessibility
- plugin-manager-ui
- notification-system
- jcl-resolver (stub -- no requirements yet)
- menu-workspace

## Canonical UI Terminology (Phase CT)
Three-level UI model:

| Level | Term | Description |
|-------|------|-------------|
| 1 | **Workbench** | The application window -- outermost container. |
| 2 | **Workspace** | A single tab in the Workbench tab bar. |
| 3 | **Context** | Content inside a Workspace (Home, Editor, Settings, etc.). |

A Workspace moved to a separate OS window is a **Detached Workspace** (previously
"floating window" or "Detached View").

Context names by Workspace kind:
- Home Context (POM) -- ISPF-style Primary Option Menu; "POM" is the retained alias
- Editor Context -- text editing surface
- Settings Context -- browsable/editable configuration
- Catalog Explorer Context -- POM option 1, virtual catalog management
- File Explorer Context -- POM option 2, VFS tree browser
- Compiler Context -- toolchain status and build output
- Search Results Context -- global search results
- Database Context, Plugin Manager Context, Macro Library Context, Event Log Context, Hex Context

See `docs/reviews/requirements-review/terminology-map.md` for the full glossary.

## Working with specs
- All documentation MUST use plain ASCII (see `documentation.md`).
- For any new requirement, run the gate in `workflow.md` before touching source.
- Before implementing, read the sub-project `requirements.md` and update criteria first.
- Before writing tests, confirm criteria exist -- map each test with
  `// Validates: Requirement X.Y` (see `testing.md`).
- Before proposing a design, read the sub-project `design.md` to avoid contradictions.
- For task status, read the sub-project `tasks.md`.
- `docs/specs/project-master/` holds cross-cutting reports and master requirements --
  read it for work spanning multiple sub-projects.
- Reuse maintenance scripts from `tools/` per `tooling.md`.

---

## Task File Format
When generating or editing `tasks.md` files under `docs/specs/`, follow strictly.

### Allowed checkbox markers
Only two states are valid:

| Marker | Meaning |
|--------|---------|
| `[ ]` | Pending / not started |
| `[x]` | Completed |

Do NOT use `[~]`, `[-]`, `[/]`, or any other symbol -- they display as "untitled task".

### Task title rule
Every task line MUST have descriptive title text immediately after the number:
```markdown
- [ ] 3. Implement persistence layer
  - [ ] 3.1 Create persistence module with load/save logic
```
Never leave a task line without a title.

### Tracking intermediate states
- In-progress: leave as `[ ]` -- the first unchecked task is implicitly active.
- Blocked: add a note on a sub-bullet (never inside the brackets):
  ```markdown
  - [ ] 5.2 Write property test for undock position
    - BLOCKED: waiting on egui viewport API stabilisation
  ```
- Partially complete: split into smaller subtasks so each can be checked off.

### Summary checklist
- Every checkbox uses `[ ]` or `[x]` only.
- Every task line has a human-readable title after the number.
- Top-level tasks have a concise summary title; subtasks have specific actions.
- Status notes go on indented sub-bullets, never inside the checkbox brackets.