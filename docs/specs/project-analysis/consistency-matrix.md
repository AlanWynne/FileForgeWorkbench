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

## Dangling Cross-References

| From unit | Missing target | Correction |
|-----------|----------------|------------|
| _(none recorded yet)_ | | |
