# FileForgeWorkbench -- Product Roadmap (Control Tower)

This is the ONE document to read to answer "what is the product, what is core,
what is a plugin, where are we, and what is being worked on right now." It sits
ABOVE the detailed tracking layers (project-analysis buckets, the PA-W phases,
and the per-sub-project task lists) and says which of them is active.

- Detailed remediation ordering: `project-analysis/remediation-plan.md`
- Cross-cutting phase tracking: `project-master/tasks.md` (PA-W0..PA-W5 phases)
- Per-feature specs: `docs/specs/<sub-project>/` (requirements / design / tasks)
- Core sign-off checklist: `core-acceptance-test-plan.md` (this folder)
- Change log (new/changed requirements): `docs/status/change-log.md`
- Bug register: `docs/status/bugs.md`

---

## 1. Product Vision

FileForgeWorkbench (FFWB) is an ISPF-style desktop workbench: a tabbed,
detachable-workspace shell driven as far as possible by a command line, in which
every action has a command equivalent so activities can be scripted as macros.
The core is a general file + text-editing environment; specialised capabilities
(mainframe/POSIX emulation, file formatting, database, job monitoring) are
delivered as PLUGINS layered on that core.

Design principles that must survive every phase:
- ISPF familiarity: Primary Option Menu (POM) style menus, a persistent
  `Command ===>` line, primary + line commands in the editor.
- Command-first: every activity has a command; commands compose into macros.
- Core vs plugin: keep the core small, stable, and thoroughly tested; add
  everything else as plugins, each built and signed off before the next.

---

## 2. CORE vs PLUGIN (the top-level split)

**CORE** = the workbench that must work and be signed off BEFORE any plugin work
resumes. See Section 3.

**PLUGINS** = everything else, built one at a time, each ending in a thorough
test sign-off before the next begins. See Section 5.

Rationale (owner-directed): the project tried to build too much at once. We
stabilise and test the core first, then add plugins in a disciplined order.

---

## 3. CORE requirements (must work + be tested first)

The six core capabilities. Status reflects the current build; "get it working +
tested" is the active goal (Section 6). Each item lists the sub-project specs it
draws on.

| # | Core capability | Status | Primary specs |
|---|-----------------|--------|---------------|
| 1 | **ISPF-like shell framework** -- open tabbed, DETACHABLE workspaces. | Built; detach + tab lifecycle need test pass | `platform-core`, `layout-and-docking`, `multi-tab-editor`, `menu-and-statusbar`, `startup-and-session`, `workspace-model` |
| 2 | **Menu + configurable workspaces** -- two workspace kinds: MENU workspaces (all modelled on the POM: option / command / description) and configurable workspaces. Plus a **Create-Menu dialog**: name a menu (e.g. "Settings"), enter a list of option/command/description rows, save as a new menu. | Menu-workspace pattern built (POM, Settings); the create/edit-menu DIALOG is the key gap | `menu-workspace`, `command-semantics` |
| 3 | **File directory navigator** -- select files for View / Edit / delete / create + standard file ops. | DONE (modern ff-file-tree explorer, CR-NR-060; edit ops incl. catalog subtrees, B042) | `file-tree-panel`, `file-operations`, `virtual-file-system`, `connector-local-fs` |
| 4 | **ISPF editor** -- primary commands + line commands, two modes: Browse/View (no mutation; display filters like INCLUDE/EXCLUDE/FIND still work) and Edit (mutation allowed). | Largely built, NOT fully working -- primary/line command execution + mode enforcement need fixing and a test pass | `edit-operations`, `line-commands`, `multi-tab-editor`, `document-model`, `undo-redo-transactions`, `caret-and-selection`, `viewport-and-scrolling` |
| 5 | **Command-line execution of everything** -- as far as reasonable, every activity has a command, so any activity can run inside a macro as a list of commands. | Command framework + palette built; coverage/parity + chaining need completion (CR-NR-057) | `command-framework`, `command-semantics`, `command-palette`, `command-completion`, `shell-command`, `function-keys-and-history` |
| 6 | **Macros as commands** -- save a macro to a file, and (in Settings) configure a saved macro as an individual command. | Lua engine + Macro Library partially built; save-macro-as-command wiring is the gap | `lua-macro-engine`, `command-configurator`, `batch-execution` |

CORE is considered DONE only when `core-acceptance-test-plan.md` is fully signed
off.

---

## 4. Where the existing tracking maps onto CORE

The prior "Bucket 1 (file + editing core)" work is essentially CORE. Nothing is
discarded; it is re-framed:

- Bucket 1 (file/editing core) -> CORE items 1, 3, 4 (+ chrome).
- The macro/command work (former Bucket 2 lead, CR-NR-057 command chaining) ->
  CORE items 5, 6.
- Buckets 3-5 (syntax highlighting, JES, Database) -> PLUGIN phases (Section 5).

So the remediation-plan buckets remain the DETAIL layer; this roadmap says the
core slices come first and must be tested before the plugin buckets proceed.

---

## 5. PLUGIN build order (after CORE sign-off)

Each plugin is built, then thoroughly tested and SIGNED OFF before the next
begins. Order (owner-directed):

1. **Utilities** -- container for in-app tools. Candidates (to be confirmed):
   IDCAMS dialog, Lua scripting, Rexx, language processors (GCC, Rust compiler).
   Open question: exact contents of Utilities.
2. **Mainframe file / dataset emulation** -- virtual dataset catalog + storage.
   Informed by the new Storage & Catalog Data Model (Section 7a).
3. **POSIX file emulation** -- POSIX-style virtual filesystem catalogs.
4. **File Formatter** -- File-AID-inspired record-structure View/Edit
   (Section 7b).
5. **Database** -- integrated Database IDE, delivered as a plugin (Section 7c).
6. **Job Monitor** -- JES / SDSF emulator, delivered as a plugin.

All new plugin capability is framed as a FileForgePlugin on the core product
(`plugin-architecture`, `plugin-manager-ui`, `connector-extensibility`).

---

## 6. Current status + the SINGLE active phase

**Active phase: CORE STABILISATION + TEST.**
Get the six core capabilities working and sign off `core-acceptance-test-plan.md`
before resuming plugin work. Priorities within it: item 4 (ISPF editor primary/
line commands + Browse/View/Edit modes) and item 2 (Create-Menu dialog), then
items 5/6 (command parity + macros-as-commands).

Recently completed (context):
- File navigator modernised onto `ff-file-tree` (CR-NR-060 Slice A); it is now
  the sole File Explorer with full edit ops.
- Bug fixes from testing: B040 (preview crash), B041 (catalog repository init),
  B042 (catalog-subtree edit ops).
- Cross-cutting doc corrections: PA-W5.5 (database-tool tracking re-open),
  PA-TRACK-007/008 (narrative reconciliation), and this docs reorganisation.

NOT active right now (deferred until after CORE sign-off): the plugin buckets
and the new-requirement gates in Section 7.

---

## 7. New / changed requirements (recorded, gated later)

These are learnings captured as source documents. They are LOGGED in
`docs/status/change-log.md` as PENDING GATE and will each go through the
requirements gate when their phase arrives -- NOT folded into CORE.

### 7a. Storage & Catalog Data Model (modifies existing catalog/dataset/VFS reqs)
- Source: `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md`
  (+ `FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md`).
- Effect: the current virtual mainframe/POSIX catalog is inadequate; this
  reworks/extends `virtual-file-system`, `dataset-catalog`, `virtual-catalog-manager`,
  `dataset-allocator`, `dataset-ownership-model`. Feeds PLUGIN phases 2 and 3.

### 7b. File Formatter plugin (new sub-project)
- Source: `docs/specs/file-formatter/design-input.md` (+
  `docs/source-documents/FileForgeWorkbench_FileAID_EARS_Requirements.md`).
- Effect: a File-AID-inspired record-structure View/Edit plugin (PLUGIN phase 4),
  bundling relevant existing specs: `record-selection-criteria`,
  `structure-catalog`, `fileforge-integration`. Change-log: CR-NR-061.

### 7c. Database as a plugin (reconfiguration)
- Source: `docs/source-documents/FileForgeWorkbench Database Plugin Specification.md`.
- Effect: reconfigure `database-tool` as a proper FileForgePlugin (PLUGIN
  phase 5). Note: database-tool tracking was re-opened (PA-W5.5) -- it is a
  foundation skeleton, not built.

### 7d. JES as a plugin (reconfiguration)
- Effect: formalise `jes-emulator` (+ `idcams-emulator`) as the Job Monitor
  plugin (PLUGIN phase 6).

---

## 8. How to use this document

- Start here. Section 6 tells you the ONE active phase.
- For the detailed ordering of remediation work, read
  `project-analysis/remediation-plan.md`.
- For per-feature detail, read the sub-project spec under `docs/specs/`.
- When a new/changed requirement is approved, it runs the requirements gate
  (`.kiro/steering/workflow.md`) and its phase is added to
  `project-master/tasks.md`; update Section 6/7 here to point at it.
