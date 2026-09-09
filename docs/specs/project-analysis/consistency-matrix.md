# Consistency Matrix

Cross-unit record of shared/public artifacts and their owning Logical_Unit
(Requirement 3). A **Conflict** is any row with more than one claimed owner or a
contradiction. Populated incrementally as each sub-project is analysed; finalized
in Wave 6.

Where `docs/specs/project-master/api-consistency-report.md` and
`docs/reviews/requirements-review/terminology-map.md` already establish an owner,
cite them in the Resolution column rather than re-deriving.

Status: EMPTY -- populated per sub-project during Waves 0-5, finalized in Wave 6.

---

## Public Types

| Type | Owner unit | Referencing units | Conflict? | Resolution |
|------|-----------|-------------------|-----------|------------|
| `WorkbenchApp` | platform-core | ff-desktop, (plugin ctx) | No | Sole owner ff-core (api-consistency-report.md) |
| `ServiceRegistry` | platform-core | ff-plugin (PluginContext) | No | Sole owner ff-core |
| `EventBus` | platform-core | ff-desktop | No | Sole owner ff-core |
| `LifecyclePhase` | platform-core | ff-desktop | No | Sole owner ff-core |
| `ThreadContext` | platform-core | -- | No | Sole owner ff-core |
| `CoreError` | platform-core | -- | No | Sole owner ff-core |
| `ConfigValue` / `ConfigTable` | configuration-system | ff-desktop, ff-plugin | No | Sole owner ff-config |
| `ConfigLayer` | configuration-system | ff-desktop | No | Sole owner ff-config |
| `ConfigHandle` | configuration-system | all consumer crates | No | Sole owner ff-config |
| `ConfigError` | configuration-system | consumers | No | Sole owner ff-config |
| `EffectiveValue` / `Provenance` | configuration-system | ff-desktop (Settings) | No | Sole owner ff-config |
| `SchemaEntry` / `Constraints` | configuration-system | ff-desktop (Settings) | No | Sole owner ff-config |
| `PluginConfigHandle` | configuration-system | ff-plugin (PluginContext) | No | Sole owner ff-config |
| `AuditEntry` / `ExportScope` / `ImportTarget` / `ImportSummary` | configuration-system | ff-desktop | No | Sole owner ff-config |
| `ConfigProvider` (trait) | platform-core (defined) | ff-config (impl) | No | Trait in ff-core, impl in ff-config -- correct direction |
| `LogLevel` | logging-subsystem | all crates | No | Sole owner ff-logging |
| `LogConfig` | logging-subsystem | ff-desktop (reconfigure) | No | Sole owner ff-logging; fed by ff-config `logging.*` keys |
| `LogRecord` | logging-subsystem | -- | No | Sole owner ff-logging |
| `log_trace!`/`log_debug!`/`log_info!`/`log_warn!`/`log_error!` (macros) | logging-subsystem | all crates | No | Sole owner ff-logging |
| `PluginLogHandle` | logging-subsystem | ff-plugin (PluginContext) | No | Sole owner ff-logging |
| `CommandId` / `CommandParams` / `CommandResult` | command-framework | all crates | No | Sole owner ff-command |
| `CommandRegistry` / `CommandMetadata` | command-framework | consumers | No | Sole owner ff-command |
| `ExecutionContext` | command-framework | editor subsystems | No | Sole owner ff-command |
| `ShortcutBinding` / `Chord` | command-framework | ff-desktop (key handling) | No | Sole owner ff-command |
| `CommandTarget` (5 variants) | command-framework | menu-workspace, command-configurator, startup-and-session, shell-command, lua-macro-engine | Watch | PA-WATCH-001: verify Target_Resolution consistency in each consumer |
| reserved `arg` param key + verb/arg split | command-framework | all input sources (cmd line, menu, key, macro) | Watch | Req 9 DEFINED but UNIMPLEMENTED (PA-INCOMPLETE-001) -- watch for divergent ad-hoc parsing |
| `Document` / `DocumentHandle` / `TextBuffer` / `GapBuffer` | document-model | edit-operations, display-line-mapping, viewport, undo-redo, ff-desktop | No | Sole owner ff-document-model (Arc<RwLock<Document>>) |
| `LineIndex` / `SparseLineIndex` | document-model | display-line-mapping (consumes LineIndex) | No | Sole owner ff-document-model |
| `BytePosition` / `LineNumber` / `CharacterExtracted` | document-model | edit/nav consumers | No | Sole owner ff-document-model |
| `LineEndMode` | document-model | encoding-and-characters (broader encoding) | No | document-model owns UTF-8 nav; encoding-and-characters owns broad encoding |
| `DocumentWatcher` (trait) | document-model | views, plugins | No | Sole owner ff-document-model |
| VFS-only I/O (Req 4.8) | document-model (consumer) | virtual-file-system (owner) | No | VERIFIED no std::fs/tokio::fs; all I/O via ff-vfs (FFW-ARCH-001) |
| `Vfs` / `VfsProvider` / `StorageProvider` / `ProviderRegistry` | virtual-file-system | ALL crates (FFW-ARCH-001) | No | Sole owner ff-vfs; ff-vfs is the only crate allowed direct std::fs |
| `ResourceUri` | virtual-file-system | all consumers | No | Sole owner ff-vfs; `vfs://provider/path` |
| `VfsError` | virtual-file-system | all consumers | No | Sole owner ff-vfs; providers map into it (no provider types leak) |
| `WatchHandle` / `WatchEvent` | virtual-file-system | document-model, external-modification | No | Sole owner ff-vfs |
| `VfsTransaction` (staged protocol) | virtual-file-system | dataset-catalog, virtual-catalog-manager | No | Sole owner ff-vfs (Req 11) |
| workspace backup manifest | virtual-file-system (Req 12.2) | dataset-catalog (ff-dscatalog Req 26.3) | Watch | PA-WATCH-003: verify shared-vs-distinct manifest type in Wave 2 |
| `FileForgePlugin` / `PluginContext` / `PluginError` | plugin-architecture | all plugin crates | No | Sole owner ff-plugin |
| `Capability` / `CapabilityRegistry` / `PluginState` | plugin-architecture | shell, subsystems | No | Sole owner ff-plugin |
| `PluginRegistry` / `PLUGIN_API_VERSION` | plugin-architecture | platform-core, shell | No | Sole owner ff-plugin |
| plugin namespace `[plugins.{name}]` | plugin-architecture (Req 2.7/7.5) + configuration-system (Req 8) | -- | No | Same scoping rule in both specs -- consistent |
| PluginLogHandle contract | plugin-architecture (Req 2 uses) + logging-subsystem (Req 10 owns) | -- | No | Consistent `[plugin:name]` prefix |
| `Workflow` / `WorkflowDefinition` / `WorkflowRunner` | workflow-engine | command-framework, plugins, shell | No | Sole owner ff-workflow |
| `WorkflowRegistry` / `Checkpoint` / `WorkflowError` | workflow-engine | consumers | No | Sole owner ff-workflow |
| `CancellationToken` (workflow) | workflow-engine | steps, async I/O | No | Sole owner ff-workflow (distinct from ff-background-io cancellation) |
| checkpoint storage I/O | workflow-engine (`tokio::fs`) | virtual-file-system (would-be owner) | Watch | PA-WATCH-004: direct tokio::fs vs FFW-ARCH-001 -- owner decision |
| `IoError` / `ProgressState` / `IoTaskHandle` | background-io | file-operations, document-model, shell | No | Sole owner ff-background-io |
| `BackgroundIoService` / `RetryPolicy` | background-io | platform-core (singleton), file-operations | No | Sole owner ff-background-io (RetryPolicy defined but NOT wired -- PA-INCOMPLETE-002) |
| `IoCancellationToken` | background-io | callers | No | Distinct from ff-workflow CancellationToken (confirmed W0.17) -- no conflict |
| VFS-only I/O (Req 1.8/4.10/8.x) | background-io (consumer) | virtual-file-system (owner) | No | VERIFIED upheld; only lib.rs doc-comment mentions std::fs (stating it is NOT used) |

## Command IDs

| Command_ID | Owner unit | Referencing units | Conflict? | Resolution |
|------------|-----------|-------------------|-----------|------------|
| _(none recorded yet)_ | | | | |

## Configuration Keys

| Config key | Owner unit | Referencing units | Conflict? | Resolution |
|------------|-----------|-------------------|-----------|------------|
| _(none recorded yet)_ | | | | |

## Panels / Contexts

| Panel/Context name | Owner unit | Referencing units | Conflict? | Resolution |
|--------------------|-----------|-------------------|-----------|------------|
| _(none recorded yet)_ | | | | |

## Glossary Terms

| Term | Owner unit | Definition source | Conflict? | Resolution |
|------|-----------|-------------------|-----------|------------|
| Platform_Core | platform-core | platform-core/requirements.md | No | -- |
| Service_Registry | platform-core | platform-core/requirements.md | No | -- |
| Event_Bus | platform-core | platform-core/requirements.md | No | -- |
| Startup_Sequence / Shutdown_Sequence | platform-core | platform-core/requirements.md | No | -- |
| Five-layer model (Foundation/Core/Editor/Feature/Shell) | platform-core | platform-core/requirements.md Req 4 | Watch | Layer-member crate names drift from actual crates -- see PA-DOC-001 |
| Configuration_Layer / Layer_Precedence | configuration-system | configuration-system/requirements.md | No | Defaults->System->User->Profile->Project->Workspace |
| Effective_Value / Provenance | configuration-system | configuration-system/requirements.md | No | -- |
| Plugin_Namespace | configuration-system | plugin-architecture | No | Reserved namespaces align with core crate names |
| EditorConfig | configuration-system | edit-operations, document-model | No | Per-file resolution, overrides layers for its properties |
| Settings_Menu / Settings_Namespace_View | configuration-system + menu-workspace | Req 15 + menu-workspace Phase CW | Watch | Settings UI shared between specs -- SPLIT proposal PA-SPLIT-001 folds Req 15 into menu-workspace |
| Log_Level / Log_Record / Log_File / Log_Rotation | logging-subsystem | all crates | No | Foundation; consumed everywhere via macros |
| logging.* config keys | logging-subsystem (consumer) | configuration-system (owner of `logging` namespace) | No | Two-phase init: ff-logging defaults then reconfigure from ff-config (CR-CH-015, B033) |
| Reserved shortcut set (Req 5.3) | command-framework | layout-and-docking, view-zoom, edit, clipboard | No | ff-command owns reserved set (cross-cutting Req 10); consumers must not override |
| commands.* config keys (history_depth, key map) | command-framework (consumer) | configuration-system (owner of `commands` namespace) | No | Consistent with reserved namespace list |

## Dangling Cross-References

| From unit | Missing target | Correction |
|-----------|----------------|------------|
| _(none recorded yet)_ | | |
