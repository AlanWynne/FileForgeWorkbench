# Requirements Review — Task 2: Terminology Standardisation

**Phase:** Requirements Review  
**Status:** COMPLETE  
**Date:** Phase BQ  
**Reviewer:** Amazon Q Developer (Senior Requirements Engineer role)

---

## 1. Purpose

This document establishes the canonical terminology for all FileForgeWorkbench
specifications. It defines:

1. The master legacy → preferred term mapping table
2. A scan of which spec files contain legacy terms
3. Additional terminology improvements beyond the prompt's initial list
4. The canonical Capability → Feature → Requirement → Acceptance Criteria
   hierarchy that all rewritten specs will use

All requirement rewrites in Tasks 5–7 must apply this map consistently.

---

## 2. Master Term Map

### 2.1 Provided Mappings (from brief)

| Legacy Term | Preferred Term | Notes |
|-------------|---------------|-------|
| Screen | View | Use "View" for any full-panel display context (e.g. "Settings View", "Explorer View") |
| Module | Capability | Use "Capability" when describing a product-level grouping of related features |
| Feature | Capability | At product level; "Feature" is still acceptable within a single spec as a sub-unit |
| Window | Workspace View | For dockable/floating panel containers; "dialog" remains acceptable for modal dialogs |
| Tree | Navigation Pane | When referring to the left-side hierarchical browser panel |
| File Browser | Explorer | "File Explorer" or "Explorer Panel" |
| Dataset Browser | Explorer | "Dataset Explorer" or "Explorer Panel" |
| Editor | Content Editor | When referring to the editing surface as a product concept |
| Configuration | Settings | In UI-facing text and requirement statements |
| Process | Task | For background operations and workflow steps |
| Utility | Tool | "Utility functions" → "Tools"; "Utilities panel" → "Tools Panel" |
| User Preference | Profile Setting | In UI-facing text; "configuration key" remains acceptable in technical specs |

### 2.2 Additional Mappings (identified during corpus scan)

| Legacy Term | Preferred Term | Rationale | Found In |
|-------------|---------------|-----------|----------|
| Screen | Panel | When referring to a dockable workbench panel (distinct from a full-screen View) | `startup-and-session`, `menu-and-statusbar`, `function-keys-and-history` |
| PF Key / PF key | Function Key | "PF key" is IBM 3270 terminal heritage; modern term is "Function Key" | `function-keys-and-history`, `startup-and-session` |
| Key Bar | Key Label Bar | More descriptive; already used in the better specs | `function-keys-and-history` |
| Primary Option Menu | Workbench Home View | In product-facing text; "Primary Option Menu" (POM) remains acceptable as an ISPF-heritage alias in technical specs | `startup-and-session` |
| Command ===> field | Command Field | Shorter; "Command ===> field" is ISPF notation acceptable in ISPF-heritage contexts only | `startup-and-session`, `function-keys-and-history` |
| Window context | Workspace Context | Aligns with "Workspace View" terminology | `function-keys-and-history`, `startup-and-session` |
| Tab | Workspace Tab | When referring to a tab in the workbench tab bar (not a keyboard Tab key) | `startup-and-session`, `multi-tab-editor` |
| Floating window | Detached View | More descriptive of the detach/redock lifecycle | `layout-and-docking`, `startup-and-session` |
| File tree | Navigation Pane | When referring to the left-side tree panel | `file-tree-panel`, `startup-and-session` |
| File Explorer Panel | Explorer Panel | Shorter canonical form | `file-tree-panel`, `startup-and-session` |
| Files Panel | Catalog Explorer | The POM option 1 panel manages catalogs, not just files | `virtual-catalog-manager`, `startup-and-session` |
| Dataset | Dataset | Retain as-is — this is a domain term, not a legacy UI term | all mainframe specs |
| Catalog | Catalog | Retain as-is — domain term | all catalog specs |
| DBeaver | (remove) | Product name must not appear in requirement text; use "the integrated database tool" | `database-tool` |
| JES2 / JES3 | JES Emulator | Clarify these are emulation targets, not runtime dependencies | `FFW-JES` |
| DYNALLOC / SVC 99 | Dataset Allocator | Replace IBM internal API names with the FFWB component name | `dataset-allocator` |
| Mainframe | Mainframe | Retain — this is a product domain term, not a legacy UI term | all mainframe specs |
| ISPF | ISPF | Retain as a heritage/compatibility reference; do not remove | all ISPF-heritage specs |
| Plugin | Plugin | Retain — already the preferred term in the architecture | all specs |
| Crate | Crate | Retain — Rust-specific technical term, acceptable in technical specs | all specs |
| VFS | VFS | Retain — established architectural acronym; spell out on first use per spec | all specs |
| TOML | TOML | Retain — established file format name | all specs |
| egui | egui | Retain — specific GUI framework name | shell specs |
| Workbench | Workbench | Retain — this is the product name | all specs |

### 2.3 Terminology Decision Rules

The following rules govern how terms are applied during rewrites:

1. **Product-facing text** (user stories, acceptance criteria visible to end users)
   must use the preferred term exclusively.

2. **Technical specs** (glossary entries, cross-reference tables, source reference
   annotations) may retain legacy terms where they are the authoritative name of
   an external system (e.g. "ISPF Primary Option Menu", "z/OS FTP", "JES2").

3. **Acronyms** must be spelled out on first use within each spec file, then the
   acronym may be used throughout. Example: "Virtual File System (VFS)".

4. **ISPF heritage terms** (ISPF, POM, ISPF-style, ISPF-authentic) are retained
   as compatibility references — they communicate intentional design heritage and
   must not be removed.

5. **Rust technical terms** (crate, trait, struct, enum, `cargo`, `Cargo.toml`)
   are retained in technical specs. They must not appear in user stories or
   acceptance criteria.

---

## 3. Legacy Term Scan by Spec File

The following table records which specs contain legacy terms requiring a
terminology pass during Tasks 5–7. Specs not listed here are clean.

| Spec | Legacy Terms Found | Priority |
|------|--------------------|----------|
| `startup-and-session` | "Screen" (×3), "Window context" (×5), "PF3/F3" used inconsistently with "Function Key", "Tab" (ambiguous — keyboard vs workspace), "Primary Option Menu" used without alias note | High |
| `function-keys-and-history` | "PF Key" (×2 in intro), "Key Bar" (×1), "window context" (×4), "screen" (×1) | High |
| `virtual-catalog-manager` | "Windows catalog" (×2 — should be "Native catalog"), "Files Panel" (×8 — should be "Catalog Explorer"), implementation file references (`files_panel.rs`, `context_menu.rs`) | High |
| `file-tree-panel` | "File Explorer Panel" and "File Tree Panel" used interchangeably (×12), "File Browser" (×1), implementation file references (`context_menu.rs`) | High |
| `database-tool` | "DBeaver" in requirement text (×6), "feature" used where "capability" is preferred (×4) | Medium |
| `FFW-JES` | "JES2/JES3" without emulation qualifier (×3), "process" where "task" is preferred (×2) | Medium |
| `dataset-allocator` | "DYNALLOC/SVC 99" without FFWB equivalent (×2), "feature" (×3) | Medium |
| `compiler-toolchain-integration` | "feature" (×5), "module" (×2), implementation details in requirement text | Medium |
| `dataset-ownership-model` | "feature" (×3), "module" (×1) | Low |
| `menu-and-statusbar` | "screen" (×1), "window" (×2 — ambiguous) | Low |
| `layout-and-docking` | "floating window" (×4 — should be "Detached View") | Low |
| `multi-tab-editor` | "tab" (×15 — acceptable but should note "Workspace Tab" in glossary) | Low |
| `context-help` | "screen" (×1), "feature" (×2) | Low |
| `configuration-system` | "User Preference" (×1), "feature" (×2) | Low |
| `plugin-architecture` | "module" (×2), "feature" (×3) | Low |

---

## 4. Canonical Hierarchy

All rewritten specifications must use the following four-level hierarchy.
This aligns with the product architecture and produces a clean repository
structure.

### 4.1 Hierarchy Definition

```
Capability
  └── Feature
        └── Requirement (FR-XXXX or NFR-XXXX)
              └── Acceptance Criterion (numbered, EARS format)
```

| Level | Definition | Example |
|-------|-----------|---------|
| **Capability** | A major product-level grouping that maps to one of the six architectural layers. Owned by the product architecture. | "Explorer Layer", "Content Editor", "Task Layer" |
| **Feature** | A discrete, user-visible capability within a Capability. Maps to one sub-project spec. | "File Explorer", "Dataset Catalog", "Hex Display" |
| **Requirement** | A single, atomic, testable statement of what the system must do. Numbered FR-XXXX (functional) or NFR-XXXX (non-functional). | FR-0142: File Explorer — Keyboard Navigation |
| **Acceptance Criterion** | A single, independently verifiable condition in EARS format. Numbered within its parent requirement. | 1. WHEN the user presses Tab... THE system SHALL... |

### 4.2 Requirement ID Scheme

Requirements are numbered globally across the entire specification corpus
using a four-digit zero-padded scheme:

```
FR-XXXX   Functional Requirement
NFR-XXXX  Non-Functional Requirement
```

The number space is allocated by architectural layer to avoid collisions:

| Layer | FR Range | NFR Range |
|-------|----------|-----------|
| Core Platform | FR-0001 – FR-0199 | NFR-0001 – NFR-0099 |
| Workbench Shell | FR-0200 – FR-0399 | NFR-0100 – NFR-0199 |
| Explorer Layer | FR-0400 – FR-0599 | NFR-0200 – NFR-0299 |
| Content Layer | FR-0600 – FR-0799 | NFR-0300 – NFR-0399 |
| Task Layer | FR-0800 – FR-0999 | NFR-0400 – NFR-0499 |
| Integration Layer | FR-1000 – FR-1199 | NFR-0500 – NFR-0599 |
| UX Layer | FR-1200 – FR-1399 | NFR-0600 – NFR-0699 |

### 4.3 Requirement Template — Functional

```markdown
#### FR-XXXX: Requirement Name

**Capability:** [Capability name]  
**Feature:** [Feature name]  
**Statement:** The system shall [observable behaviour].  
**Rationale:** [Why this capability exists — one sentence.]  

**Acceptance Criteria:**

1. WHEN [trigger] THE [actor] SHALL [observable outcome].
2. WHEN [trigger] THE [actor] SHALL [observable outcome].
3. IF [condition] THEN THE [actor] SHALL [observable outcome].

**Dependencies:** FR-XXXX, FR-XXXX  
**Architectural Domain:** [Layer name]  
**Original ID:** [e.g. Req 8.3 file-tree-panel] ← traceability back to source
```

### 4.4 Requirement Template — Non-Functional

```markdown
#### NFR-XXXX: Requirement Name

**Capability:** [Capability name]  
**Feature:** [Feature name]  
**Statement:** The system shall [quality attribute statement].  
**Measurement:** [Objective, measurable criterion — number, percentage, time.]  
**Verification Method:** [How compliance is demonstrated — test, benchmark, inspection.]  
**Architectural Domain:** [Layer name]  
**Original ID:** [traceability]
```

### 4.5 Acceptance Criterion Format Rules

All acceptance criteria must use EARS (Easy Approach to Requirements Syntax):

| Pattern | Template | Use When |
|---------|----------|----------|
| Event-driven | `WHEN <trigger> THE <actor> SHALL <response>` | Normal system response to a user or system event |
| State-driven | `WHILE <state> THE <actor> SHALL <behaviour>` | Continuous behaviour during a state |
| Conditional | `IF <condition> THEN THE <actor> SHALL <response>` | Error handling, edge cases |
| Ubiquitous | `THE <actor> SHALL <behaviour>` | Always-true constraints |
| Optional | `WHERE <feature is included> THE <actor> SHALL <behaviour>` | Conditional feature inclusion |

Rules:
- Each criterion must be independently testable
- Each criterion must assert exactly one observable behaviour
- No criterion may reference implementation details (file names, struct names, function names)
- No criterion may use vague qualifiers ("quickly", "efficiently", "appropriately") without a measurable definition
- Measurements belong in NFRs, not in FR criteria

---

## 5. Find/Replace Reference

The following substitutions should be applied mechanically during the rewrite
passes in Tasks 5–7. Apply in the order listed (longer phrases first to avoid
partial matches).

```
"File Explorer Panel"          → "Explorer Panel"
"File Tree Panel"              → "Explorer Panel"  (when referring to the panel component)
"Files Panel"                  → "Catalog Explorer"  (when referring to POM option 1)
"Primary Option Menu"          → "Workbench Home View"  (in user stories only; retain POM alias in glossary)
"Command ===> field"           → "Command Field"  (except in ISPF-heritage context sections)
"floating window"              → "Detached View"
"floating OS window"           → "Detached View"
"window context"               → "Workspace Context"
"PF key"                       → "Function Key"
"PF3"                          → "F3"  (normalise PF-prefix to F-prefix throughout)
"Key Bar"                      → "Key Label Bar"
"Windows catalog"              → "Native catalog"
"DBeaver"                      → "the integrated database tool"  (in requirement text only)
"DYNALLOC/SVC 99"              → "the Dataset Allocator"
"User Preference"              → "Profile Setting"
"Configuration"                → "Settings"  (in UI-facing text only)
"Utility"                      → "Tool"  (when referring to the Utilities panel/menu)
"module"                       → "capability"  (when used as a product-level grouping)
"feature"                      → "capability"  (when used as a product-level grouping)
"screen"                       -> "view"  (when referring to a full-panel display context)
"tree"                         -> "Navigation Pane"  (when referring to the left-side panel)
"editor"                       -> "Content Editor"  (in product-level descriptions)
```

Phase CT additional substitutions (apply after the above):

```
"Workspace View"               -> "Workspace"
"Detached View"                -> "Detached Workspace"
"Content Editor"               -> "Editor Context"  (when referring to tab content)
"Settings Panel"               -> "Settings Context"
"Catalog Explorer"             -> "Catalog Explorer Context"
"Explorer Panel"               -> "File Explorer Context"
"File Explorer Panel"          -> "File Explorer Context"
"Toolchain Panel"              -> "Compiler Context"
"Workbench Home View"          -> "Home Context"  (product-facing; POM alias retained)
"Primary Option Menu"          -> "Home Context"  (product-facing; POM alias retained)
```

---

## 6. Glossary of Canonical Terms

The following terms are the canonical vocabulary for all FFWB specifications.
Each rewritten spec must include a Glossary section using these definitions
(adapted as needed for the spec's scope).

### 6.1 Three-Level UI Model (Phase CT -- canonical)

The Workbench UI is described using exactly three levels:

| Level | Term | Definition |
|-------|------|------------|
| 1 | **Workbench** | The FileForgeWorkbench application window -- the outermost container. |
| 2 | **Workspace** | A single tab in the Workbench tab bar. Each Workspace has a kind that determines what it displays. |
| 3 | **Context** | The content displayed inside a Workspace -- named by what it shows (e.g. Home Context, Editor Context). |

Workspace kinds and their Context names:

| Workspace kind | Context name | Activated by |
|---------------|-------------|-------------|
| Home | Home Context | Default on launch; `=0` through `=8` fastpath |
| Editor | Editor Context | `EDIT <path>`, `File > Open`, CLI argument |
| Settings | Settings Context | `0` / `SETTINGS` / `=0` / POM option 0 |
| Catalog Explorer | Catalog Explorer Context | `1` / `FILES` / `=1` / POM option 1 |
| File Explorer | File Explorer Context | `2` / `=2` / `=FILES` / POM option 2 |
| Search Results | Search Results Context | `Ctrl+Shift+F` / `GSEARCH` |
| Compiler | Compiler Context | `5` / `COMPILERS` / `=5` / POM option 5 |
| Database | Database Context | `8` / `DATABASES` / `=8` / POM option 8 |
| Plugin Manager | Plugin Manager Context | `8` / `PLUGINS` / `=8` / POM option 8 |
| Macro Library | Macro Library Context | `6` / `MACROS` / `=6` / POM option 6 |
| Event Log | Event Log Context | `LOG` command |
| Hex | Hex Context | `HEX ON` within an Editor Context |

A Workspace that has been moved to a separate OS window is called a **Detached Workspace**.

### 6.2 Full Term Glossary

| Term | Definition |
|------|-----------|
| **Workbench** | The FileForgeWorkbench application as a whole -- the desktop platform that hosts all Capabilities, Features, and Tools. |
| **Workspace** | A single tab in the Workbench tab bar. Each Workspace displays exactly one Context. |
| **Context** | The content displayed inside a Workspace, named by what it shows (e.g. Home Context, Editor Context, Settings Context). |
| **Detached Workspace** | A Workspace that has been moved to a separate OS window, retaining its content and state. Previously called "Detached View" or "floating window". |
| **Home Context** | The ISPF-style Primary Option Menu displayed in the default Workspace on launch. Also referred to as "POM" in ISPF-heritage contexts. |
| **Editor Context** | The text editing surface within a Workspace, providing insert/overstrike editing, syntax highlighting, and ISPF-style commands. Previously called "Content Editor". |
| **Settings Context** | The browsable, editable view of all Profile Settings within a Workspace. Previously called "Settings Panel". |
| **Catalog Explorer Context** | The POM option 1 view providing unified management of Virtual Catalogs (Mainframe, POSIX, Native). Previously called "Catalog Explorer" or "Files Panel". |
| **File Explorer Context** | The POM option 2 view providing hierarchical navigation of resources from all registered VFS providers. Previously called "Explorer Panel" or "File Explorer Panel". |
| **Search Results Context** | The view displaying global search results and replace controls. |
| **Compiler Context** | The view displaying toolchain status, build output, and diagnostics. Previously called "Toolchain Panel". |
| **Database Context** | The view providing integrated database tool access. |
| **Plugin Manager Context** | The view listing installed and available plugins. |
| **Macro Library Context** | The view listing, running, and editing Lua macros. |
| **Event Log Context** | The view displaying the notification and event log. |
| **Hex Context** | The hex display mode within an Editor Context. |
| **Capability** | A major product-level grouping of related Features, aligned to one of the six architectural layers. |
| **Feature** | A discrete, user-visible unit of functionality within a Capability, corresponding to one sub-project specification. |
| **Navigation Pane** | The left-side hierarchical tree within the File Explorer Context. |
| **Command Field** | The single-line text input labelled "Command ===>" used for direct ISPF-style command entry. |
| **Key Label Bar** | The footer region displaying current Function Key assignments as labelled slots. |
| **Function Key** | A keyboard key in the set F1-F24, assignable to any registered command. |
| **Profile Setting** | A user-configurable preference stored in the layered TOML configuration system. |
| **Tool** | A specialised workbench capability providing a focused function (e.g. Database Tool, Compiler Tool). |
| **Task** | A background operation managed by the workflow engine (e.g. file copy, build, search). |
| **Plugin** | An independently loadable extension that registers Capabilities, Features, or Connectors with the Workbench. |
| **Connector** | A VFS provider plugin that exposes a remote or specialised storage system through the VFS abstraction layer. |
| **Virtual Catalog** | A named, typed container registered with the VFS that groups related files or datasets. |
| **VFS** | Virtual File System -- the abstraction layer through which all resource access is routed, regardless of backing store. |
| **Dataset** | A mainframe-style named data container (PS, PDS, PDSE, GDG) managed by the Dataset Catalog. |
| **Session** | The persisted snapshot of the user's Workspace state, restored on next launch. |
| **Settings** | The collection of Profile Settings configurable by the user, accessible via the Settings Context. |

---

## 7. Phase CT Additions (Workbench/Workspace/Context model)

Phase CT (CR-CH-009) extends this map with the three-level UI model defined in §6.1.
The following additional substitutions apply during the Phase CT terminology pass:

```
"Workspace View"               -> "Workspace"  (a tab is now simply a Workspace)
"Detached View"                -> "Detached Workspace"
"floating window"              -> "Detached Workspace"
"Content Editor"               -> "Editor Context"  (when referring to the tab content)
"Settings Panel"               -> "Settings Context"  (when referring to the tab content)
"Catalog Explorer"             -> "Catalog Explorer Context"  (when referring to the tab content)
"Explorer Panel"               -> "File Explorer Context"  (when referring to the tab content)
"File Explorer Panel"          -> "File Explorer Context"
"Toolchain Panel"              -> "Compiler Context"
"Workbench Home View"          -> "Home Context"  (in product-facing text; POM alias retained)
"Primary Option Menu"          -> "Home Context"  (in product-facing text; POM alias retained)
"panel"                        -> "Context"  (when referring to the content of a Workspace tab)
"view"                         -> "Context"  (when referring to the content of a Workspace tab)
```

Note: "panel" and "view" are only substituted when they refer to the content of a Workspace
tab. They remain valid in other contexts (e.g. "Navigation Pane", "split view").

## 8. Next Steps

This terminology map feeds directly into:

- **Task 3** -- Architectural Domain Classification (uses the Capability hierarchy from §4)
- **Tasks 5-7** -- Requirement Rewrites (apply the find/replace table from §5 and templates from §4)
- **Task 8** -- Traceability Matrix (uses the FR-XXXX / NFR-XXXX numbering scheme from §4.2)
- **Phase CT** -- Workbench/Workspace/Context pass across all 69 sub-project specs

The find/replace table in §5 should be applied as a first pass on each spec
file before the structural rewrite begins, to ensure terminology is consistent
before criteria are renumbered and reformatted.
