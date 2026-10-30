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
- command-environments (CR-CH-053: Command Environment model, REXX ADDRESS-inspired -- framework + FFCMD + FFEDIT built, FFLINE named-only, rest vision-only; see environments-vision.md)
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
- extension-integration (CR-CH-074: the keystone extension-integration contract -- how an extension contributes commands/menus/Contexts/Kinds through the framework seams; two owner-approved additive framework extension points. Consolidates plugin-architecture + plugin-manager-ui)
- external-modification
- global-search
- jes-emulator (folder was FFW-JES -- renamed Phase BR to match kebab-case convention)
- file-formatter (File-AID-inspired record-structure View/Edit plugin -- CR-NR-061, design-input only, gate not yet run)
- file-operations
- file-tree-panel
- ffmdx-app (CR-NR-102: rebuild standalone ffmdx as a cut-down FFWB -- inherits ff-theme/ff-config/ff-logging/ff-session/ff-help via a shared ff-app-bootstrap crate; consumes markdown-rendering; requirements drafted, gate pending)
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
- localization (CR-NR-103: i18n -- convertible UI language via message catalogue + locale config; command verbs stay source-language, descriptions/chrome translate. Forward-build directive effective now (write extraction-friendly); audit + framework gate pending. See localization-vision.md)
- logging-subsystem
- lua-macro-engine
- markdown-rendering (CR-NR-101: configurable Markdown element styling -- TOML config consumed by FFWB + ffmdx + HTML/PDF export; composes ff-theme Colour_Tokens, owns markdown typography/layout; requirements drafted, gate pending)
- master-screen-directive (CR-NR-114: cross-cutting every-Context directive -- mandates the WorkspaceContext/InteriorFocus focus contract + ISPF save convention (central Req 11) for editable Contexts + per-Context ScreenProvider snapshot/capture with a documented opt-out; sibling of the focus contract; carries the Core Context retrofit matrix; requirements/design/tasks drafted, gate pending)
- menu-and-statusbar
- multi-tab-editor
- navigation-commands
- platform-core
- plugin-architecture
- record-selection-criteria
- screen-snapshot-scrm (CR-NR-098: Screen Snapshot Service + Screen Collection/Replay Manager -- logical copy/paste-able TEXT capture, NOT raster; delivered together with the ff-desktop decomposition. Requirements drafted, gate pending owner approval; design/tasks not yet written)
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
- volume-model (CR-NR-105: first-class Volume layer -- VOLSER/status/capacity/emulated geometry/extents-as-metadata/x37 + volume-full failures/DEFINE VOLUME; split not rename; new ff-volume crate; gate authored)
- whitespace-and-guides
- workbench-requirements-merge (architecture docs and validation reports -- no deliverable crate)
- workflow-engine
- workspace-framework (CR-NR-078: WorkspaceContext trait framework -- design proposal, gate pending)
- workspace-kinds (CR-NR-090: configurable Workspace Kinds -- per-Kind title/menu-bar/key-list/profile + user Kinds "modelled on" a built-in base; sliced B.1-B.4; distinct from workspace-model)
- workspace-model
- accessibility
- plugin-manager-ui
- notification-system
- jcl-resolver (stub -- no requirements yet)
- menu-workspace

NOTE: `project-master` and `project-analysis` are NOT specifications -- they are
project-management data and now live under `docs/project-management/` (see
`docs/project-management/ROADMAP.md`, the control-tower doc).

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
- Catalog Explorer Context -- virtual catalog management (typed-command reachable
  via CATALOGS; no longer a bare-core POM option after CR-CH-073)
- File Explorer Context -- VFS tree browser (typed-command reachable via FILES;
  no longer a bare-core POM option after CR-CH-073)
- Compiler Context -- toolchain status and build output
- Search Results Context -- global search results
- Database Context, Plugin Manager Context, Macro Library Context, Event Log Context, Hex Context

See `docs/reviews/requirements-review/terminology-map.md` for the full glossary.

## Working with specs
- All documentation MUST use plain ASCII (see `documentation-charset.md`).
- For any new requirement, run the gate in `workflow.md` before touching source.
- Before implementing, read the sub-project `requirements.md` and update criteria first.
- Before writing tests, confirm criteria exist -- map each test with
  `// Validates: Requirement X.Y` (see `testing.md`).
- Before proposing a design, read the sub-project `design.md` to avoid contradictions.
- For task status, read the sub-project `tasks.md`.
- `docs/project-management/project-master/` holds cross-cutting reports and master
  requirements -- read it for work spanning multiple sub-projects. `docs/project-management/`
  also holds `project-analysis/` and the control-tower `ROADMAP.md`.
- Reuse maintenance scripts from `tools/` per `tooling.md`.

---

## Task File Format
When generating or editing `tasks.md` files under `docs/specs/`, follow the
machine-wide task-file rules in the user-level steering `task-format.md` (the
single source of truth for checkbox markers, titles, and status notes):

- Only `[ ]` (pending) and `[x]` (completed) are valid markers -- never `[~]`,
  `[-]`, `[/]`, or any other symbol (they display as "untitled task").
- Every task line has descriptive title text immediately after the number.
- Intermediate states (in-progress / blocked / partially complete) are expressed
  per `task-format.md` -- status notes on indented sub-bullets, never inside the
  brackets.

The spec-specific point is only that these rules apply to every `tasks.md` under
`docs/specs/<sub-project>/`. For the full rules, markers table, and examples, see
`task-format.md`.