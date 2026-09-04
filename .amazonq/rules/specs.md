# Project Specifications

All feature specifications live under `docs/specs/<sub-project>/`.

Each sub-project folder contains up to three files:

| File | Purpose |
|------|---------|
| `requirements.md` | Acceptance criteria in EARS format (`WHEN … THE … SHALL …`). This is the source of truth for what must be implemented and tested. |
| `design.md` | Architecture and design decisions for the sub-project. |
| `tasks.md` | Ordered implementation tasks with `[ ]` / `[x]` completion state. |

## Sub-projects

The following specs are available under `docs/specs/`:

- asa-report-preview
- auto-indentation
- background-io
- caret-and-selection
- clipboard-operations
- command-completion
- command-framework
- command-semantics
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
- FFW-JES
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
- workflow-engine

## Rules for Amazon Q

- **All documentation files MUST use plain ASCII characters only.** Follow `.amazonq/rules/documentation-ascii.md` for the full list of prohibited characters and their substitutes. This applies to every `.md` file written or edited.
- **When given any new requirement**, follow the full gate in `.amazonq/rules/new-requirements-gate.md` BEFORE touching any source file.
- **Before implementing any feature**, read `docs/specs/<sub-project>/requirements.md` for that sub-project AND update it with any new or adjusted acceptance criteria before writing any code.
- Before writing tests, confirm the acceptance criteria in `requirements.md` — every test must map to a criterion via `// Validates: Requirement X.Y`.
- Before proposing a design, read `docs/specs/<sub-project>/design.md` to avoid contradicting existing architectural decisions.
- When asked about task status, read `docs/specs/<sub-project>/tasks.md`.
- The `docs/specs/project-master/` folder contains cross-cutting validation reports and the master requirements — read it for any work that spans multiple sub-projects.
- Reuse project-specific maintenance scripts from `tools/`; follow `.amazonq/rules/project-tools.md` before creating, moving, or deleting scripts.
