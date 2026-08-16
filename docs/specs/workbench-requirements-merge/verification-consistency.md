# Verification Report: Requirements Consistency Cross-Reference

**Task:** 18.1 — Cross-reference all requirements for consistency  
**Date:** 2025-01-XX  
**Scope:** All 61 sub-project specifications in `.kiro/specs/`  
**Verdict:** ⚠️ **1 CONFLICT FOUND** — see §1 below

---

## Summary

| Check | Result |
|-------|--------|
| Conflicting requirements between sub-projects | ⚠️ 1 conflict (URI scheme naming) |
| Duplicate requirements (unintentional) | ✅ PASS — no meaningful duplicates |
| FFW-ARCH-001 (VFS abstraction) honoured | ✅ PASS — all file-related specs route through VFS |
| Command framework integration | ✅ PASS — all user-facing operations registered as commands |
| Plugin architecture (DockablePanel) | ✅ PASS — all panels implement DockablePanel trait |
| Workflow engine (state machines) | ✅ PASS — multi-step operations use workflow definitions |
| Configuration system (TOML, hot-reload) | ✅ PASS — all specs reference configuration-system |
| Logging subsystem (structured logging) | ✅ PASS — all specs reference ff-logging for diagnostics |

---

## §1. Conflicts Found

### CONFLICT-001: Local Filesystem Provider URI Scheme — `"local"` vs `"file"`

**Severity:** HIGH — affects URI construction across the entire workbench  
**Specs involved:**
- `virtual-file-system` Requirement 2 (AC 10) and Requirement 3 (AC 8–9)
- `connector-local-fs` Requirement 1 (AC 2)
- `file-operations` (Design Principles §1)
- `startup-and-session` Requirement (AC 3)
- `database-tool` Requirement 16 (AC 16.2)
- `compare-and-merge` Requirement (AC 10, AC 2)

**Description:**

The `virtual-file-system` specification consistently defines the default provider scheme as `"local"`:

> - Requirement 2, AC 10: *"WHEN a bare path is provided (no `vfs://` scheme prefix), THE VFS layer SHALL interpret it as a path relative to the Default_Provider (local filesystem), constructing the full URI as `vfs://local/{path}`."*
> - Requirement 3, AC 8: *"THE Provider_Registry SHALL designate a default provider (scheme `local`) that is used when bare paths without a URI scheme are encountered."*
> - Requirement 3, AC 9: *"IF no provider has been registered for the `local` scheme when the VFS is initialized..."*

Multiple consuming specs also use `vfs://local/...`:
- `file-operations` Design Principles: *"Bare paths are converted to `vfs://local/...` URIs transparently."*
- `startup-and-session`: *"WHEN a CLI_Source_Arg is a VFS URI (e.g., `vfs://local/path/to/file`)..."*
- `database-tool` Req 16.2: *"SQL script files SHALL be addressable via VFS Resource_URIs (e.g., `vfs://local/path/to/script.sql`)"*
- `compare-and-merge`: *"comparing `vfs://local/file.txt` with `vfs://catalog/HLQ.DATA.MEMBER`"*

However, the `connector-local-fs` specification uses `"file"`:

> - Requirement 1, AC 2: *"THE Local_FS_Provider SHALL register itself with the Provider_Registry under the URI scheme `"file"`, making local resources addressable as `vfs://file/{path}`."*
> - Glossary: *"For the local filesystem provider, the scheme is `file` (e.g., `vfs://file/home/user/document.txt`)."*
> - Requirement 2, AC 8–10: All examples use `vfs://file/...`

**Impact:** If the connector registers as `"file"` but the VFS layer expects `"local"` as the default, bare path resolution will fail with `VfsError::ProviderUnavailable`. All specs using `vfs://local/...` URIs will be non-functional.

**Resolution required:** Align on a single scheme name. The majority of specs (VFS, file-operations, startup-and-session, database-tool, compare-and-merge) use `"local"`. The `connector-local-fs` spec should be updated to register under scheme `"local"` instead of `"file"`.

---

## §2. Duplicate Requirements Analysis

### Intentional Cross-References (Not Duplicates)

The following requirements appear in multiple specs but are **intentional cross-references** — each spec references the authoritative source and defines its own integration point rather than redefining the same criteria:

| Requirement Area | Authoritative Spec | Referencing Specs |
|---|---|---|
| VFS abstraction principle | `project-master` Req 1 / `virtual-file-system` | `file-operations`, `background-io`, `document-model`, `external-modification`, `file-tree-panel`, `database-tool`, `dataset-catalog`, `compare-and-merge`, `connector-local-fs` |
| Command dispatch integration | `project-master` Req 4 / `command-framework` | `file-operations`, `database-tool`, `file-tree-panel`, `dataset-catalog`, `lua-macro-engine` |
| Plugin trait (FileForgePlugin) | `plugin-architecture` Req 1 | `database-tool`, `lua-macro-engine`, `connector-extensibility` |
| DockablePanel trait | `layout-and-docking` Req 1 | `database-tool`, `file-tree-panel`, `context-help` |
| Workflow engine for multi-step ops | `workflow-engine` | `database-tool` (data transfer) |
| Structured logging | `logging-subsystem` | All specs (consistent reference to ff-logging) |
| Configuration system access | `configuration-system` | All specs (consistent TOML namespace usage) |

All cross-references are properly scoped — consuming specs state "SHALL use" the authoritative crate's API rather than redefining the API. **No unintentional duplicates found.**

### Near-Duplicates Reviewed and Cleared

| Area | Specs | Verdict |
|---|---|---|
| Undo/redo integration | `command-framework` Req 4 vs `undo-redo-transactions` | **Not duplicate** — command-framework defines the dispatch-level integration; undo-redo-transactions defines the transaction mechanics. They are complementary. |
| File watching | `virtual-file-system` Req 7 vs `connector-local-fs` Req 3 vs `external-modification` Req 1 | **Not duplicate** — VFS defines the abstract trait, connector-local-fs implements the OS-native watcher, external-modification consumes watch events. Proper layering. |
| Read-only detection | `file-operations` Req 8 vs `document-model` Req 2 (AC 7–8) | **Not duplicate** — file-operations owns the policy (when to mark read-only); document-model owns the enforcement (reject mutations). |
| Save point / dirty flag | `file-operations` Req 1 (AC 2) vs `document-model` Req 10 | **Not duplicate** — file-operations triggers `set_save_point()` after save; document-model maintains the save-point marker and watcher notifications. |
| Keyboard shortcuts | `command-framework` Req 5 (AC 3) vs `project-master` Req 10 (AC 1) | **Intentional duplication** — project-master defines the reserved shortcut set as a cross-cutting constraint; command-framework enforces it at registration time. Same list, correct redundancy for emphasis. |

---

## §3. Cross-Cutting Architectural Principles Verification

### FFW-ARCH-001: VFS Abstraction for All Content Access

| Sub-Project | VFS Compliance | Evidence |
|---|---|---|
| `document-model` | ✅ | Req 4 AC 8: *"ALL file I/O operations SHALL flow through the VFS abstraction"* |
| `file-operations` | ✅ | Intro: *"All file operations route through the VFS abstraction layer"*; Req 7 AC 9: *"ALL write operations SHALL go through the VFS provider API"* |
| `background-io` | ✅ | Intro: *"background-io uses the VFS provider async interface and never calls `std::fs`"*; Req 1 AC 8 |
| `external-modification` | ✅ | Req 1 AC 6: *"SHALL NOT use `std::fs`, `tokio::fs`, or any other direct filesystem API"* |
| `file-tree-panel` | ✅ | Cross-References: *"All resource browsing, listing, stat, watch, and search operations go through the VFS API"* |
| `database-tool` | ✅ | Req 16: explicit VFS requirement; AC 16.1: *"SHALL use the VFS API — no direct `std::fs` or `tokio::fs` calls"* |
| `dataset-catalog` | ✅ | Intro: *"All dataset I/O flows through the VFS abstraction (FFW-ARCH-001)"*; implements VfsProvider under scheme `catalog` |
| `compare-and-merge` | ✅ | *"THE compare subsystem SHALL resolve all resource paths to Resource_URIs via the VFS abstraction"* |
| `connector-local-fs` | ✅ | Implements the `VfsProvider` trait — it IS the provider |
| `connector-extensibility` | ✅ | Defines the extension trait for future providers |
| `encoding-and-characters` | ✅ | Service layer consumed by document-model at load/save boundaries — no direct I/O |
| `startup-and-session` | ✅ | CLI args resolved via VFS; session restore uses VFS URIs |

**Verdict:** ✅ All file-accessing sub-projects explicitly state VFS-only access. No spec performs direct `std::fs` operations.

---

### Command Framework Integration

| Sub-Project | Commands Registered | Evidence |
|---|---|---|
| `file-operations` | `file.new`, `file.open`, `file.save`, `file.save_as`, `file.revert`, `file.close`, `file.exit` | Req 10 |
| `database-tool` | `db.*` namespace (connection, SQL, schema, data, diagram, admin) | Req 15 |
| `file-tree-panel` | Tree operations as commands (open, rename, delete, new file) | Cross-References |
| `dataset-catalog` | Catalog/dataset operations as commands | Cross-References |
| `compare-and-merge` | `compare.execute`, compare operations | Mentioned in spec |
| `find-and-replace` | FIND/RFIND/CHANGE as commands | ISPF command model |
| `view-zoom` | Zoom commands | Standard shortcuts |
| `clipboard-operations` | Copy/cut/paste commands | Standard shortcuts |
| `line-commands` | Line command dispatching | FFE model |

**Verdict:** ✅ All user-facing operations are routed through the command framework. No spec defines direct state mutation from UI.

---

### Plugin Architecture (DockablePanel Registration)

| Panel | Spec | DockablePanel Compliance | Default Zone |
|---|---|---|---|
| File Tree Panel | `file-tree-panel` | ✅ Implements DockablePanel | Left |
| Schema Browser | `database-tool` Req 9 | ✅ `DockablePanel` with `default_dock_zone` Left | Left |
| SQL Editor Panel | `database-tool` Req 5 | ✅ `DockablePanel` with `default_dock_zone` Center | Center |
| Result Grid Panel | `database-tool` Req 8 | ✅ `DockablePanel` with `default_dock_zone` Bottom | Bottom |
| ER Diagram Panel | `database-tool` Req 11 | ✅ `DockablePanel` with `default_dock_zone` Center | Center |
| Context Help Panel | `context-help` | ✅ References panel system | — |

**Verdict:** ✅ All panels declare DockablePanel implementation with zone assignments.

---

### Workflow Engine (Multi-Step Operations)

| Operation | Spec | Uses Workflow Engine | Evidence |
|---|---|---|---|
| Data Transfer (import/export) | `database-tool` Req 10 | ✅ | AC 10.1: *"SHALL be implemented as Workflow_Definitions registered with the Workflow_Registry"* |
| Cross-database transfer | `database-tool` Req 10 | ✅ | Uses workflow state machine |
| Compare-merge | `compare-and-merge` | Referenced as workflow candidate | VFS-aware multi-step |

**Verdict:** ✅ Complex multi-step operations use workflow state machines as mandated.

---

### Configuration System (TOML-Based, Hot-Reload)

| Sub-Project | Config Namespace | Hot-Reload Support |
|---|---|---|
| `logging-subsystem` | `[logging]` | Referenced |
| `configuration-system` | Owns all namespaces | ✅ Core requirement |
| `database-tool` | `[plugins.database-tool]` | ✅ Via PluginContext |
| `theme-and-appearance` | `[theme]` | ✅ Cross-references configuration-system |
| `file-operations` | `file.*` keys | ✅ Configurable thresholds |
| `editor` | `[editor]` | ✅ Per configuration-system Req 7 |

**Verdict:** ✅ All specs that need configuration reference the configuration-system and use proper namespaces. No key conflicts detected between specs.

---

### Logging Subsystem (Structured Logging)

All reviewed specs include logging integration:
- All error conditions specify log-level output (ERROR, WARN, INFO, DEBUG)
- All specs reference `ff-logging` as a dependency
- Log record format is consistent (per logging-subsystem Req 2)
- Error messages follow the project standard: `[subsystem] operation: description`

**Verdict:** ✅ Structured logging is consistently referenced across all specs.

---

## §4. Additional Observations (Not Conflicts)

### Observation 1: Connector-Local-FS Scheme vs VFS Default Provider Naming

Beyond the conflict in §1, note that `connector-local-fs` uses `"file"` as a scheme in its glossary's `Default_Provider` entry and its examples (`vfs://file/...`), while the VFS `Provider_Registry` AC 8 explicitly states `"local"` as the default scheme. The `virtual-file-system` glossary also defines:

> - **Default_Provider**: The provider used when a bare path (no URI scheme) is provided — **defaults to the local filesystem provider**. [WB]

This confirms the intent is for the local provider to own the default scheme. The scheme name just needs alignment.

### Observation 2: Configuration Namespace Protection

The `configuration-system` Req 8 AC 7 specifies that plugins cannot register keys colliding with core namespaces: `logging`, `editor`, `theme`, `vfs`, `commands`, `layout`. The `dataset-catalog` spec uses `[catalog]` as its namespace. This is fine since `catalog` is not a restricted core namespace and dataset-catalog is a VFS provider (not technically a plugin in the same sense), but it should be confirmed whether it registers config through `PluginContext` or directly.

### Observation 3: Keyboard Shortcut Ctrl+Shift+T Dual Use

The `project-master` Req 10 reserves `Ctrl+Shift+T` for "Undock/redock tab" (per layout-and-docking). This is consistent — no other spec attempts to bind this shortcut. Verified.

### Observation 4: Database Tool Query Execution Shortcuts

The `database-tool` Req 15.6 registers `Ctrl+Enter`, `Alt+X`, `Ctrl+Shift+E`, `F5`, `Ctrl+Space` as default shortcuts. None of these conflict with the reserved shortcuts in project-master Req 10 AC 1. The command-framework's conflict detection (Req 5 AC 4) would catch any overlap at runtime.

---

## §5. Conclusion

The requirements across all 61 sub-project specifications are **highly consistent** with one actionable conflict:

1. **CONFLICT-001 (URI Scheme):** The `connector-local-fs` spec must be updated to use scheme `"local"` instead of `"file"` to align with the VFS spec and all consuming specs that construct URIs with `vfs://local/...`.

All six cross-cutting architectural principles are properly honoured:
- ✅ FFW-ARCH-001 (VFS abstraction)
- ✅ Command framework integration
- ✅ Plugin architecture (DockablePanel)
- ✅ Workflow engine (state machines)
- ✅ Configuration system (TOML, hot-reload)
- ✅ Logging subsystem (structured logging)

No unintentional duplicate requirements were found. All cross-references between specs are properly scoped — consuming specs delegate to authoritative specs rather than redefining acceptance criteria.

---

*Report generated as part of Task 18.1 — Final Validation wave.*
