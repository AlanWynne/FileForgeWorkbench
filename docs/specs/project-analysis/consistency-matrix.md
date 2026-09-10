# Consistency Matrix

Cross-unit record of shared/public artifacts and their owning Logical_Unit
(Requirement 3). A **Conflict** is any row with more than one claimed owner or a
contradiction. Populated incrementally as each sub-project is analysed; finalized
in Wave 6.

Where `docs/specs/project-master/api-consistency-report.md` and
`docs/reviews/requirements-review/terminology-map.md` already establish an owner,
cite them in the Resolution column rather than re-deriving.

Status: EMPTY (re-baselined after CR-NR-057, commit `b537737`) -- populated per
sub-project during Waves 0-5, finalized in Wave 6.

---

## Public Types

| Type | Owner unit | Referencing units | Conflict? | Resolution |
|------|-----------|-------------------|-----------|------------|
| `WorkbenchApp` | platform-core | ff-desktop, (plugin ctx) | No | Sole owner ff-core |
| `ServiceRegistry` | platform-core | ff-plugin (PluginContext) | No | Sole owner ff-core |
| `EventBus` | platform-core | ff-desktop | No | Sole owner ff-core |
| `LifecyclePhase` / `ThreadContext` / `CoreError` | platform-core | ff-desktop | No | Sole owner ff-core |
| `ConfigProvider` (trait) | platform-core (defined) | ff-config (impl) | No | Trait in ff-core, impl in ff-config -- correct direction |
| 5-layer model (Foundation/Core/Editor/Feature/Shell) | platform-core (Req 4) | whole workspace | No | Authoritative dependency-direction reference; enforced by Cargo.toml. Req 4.1 crate names illustrative -- PA-DOC-001 |
| `ConfigValue` / `ConfigTable` / `ConfigLayer` / `ConfigHandle` / `ConfigError` | configuration-system | all consumer crates | No | Sole owner ff-config |
| `EffectiveValue` / `Provenance` / `SchemaEntry` | configuration-system | ff-desktop (Settings) | No | Sole owner ff-config |
| `PluginConfigHandle` | configuration-system | ff-plugin (PluginContext) | No | Sole owner ff-config |
| `AuditEntry` / `ExportScope` / `ImportTarget` / `ImportSummary` | configuration-system | ff-desktop | No | Sole owner ff-config (Req 16/17) |
| `ConfigProvider` (trait) | platform-core (defined) | ff-config (impl) | No | Correct direction (W0.1) |
| plugin namespace `[plugins.{name}]` | configuration-system (Req 8) + plugin-architecture (Req 2.7/7.5) | -- | No | Same scoping rule both specs |
| Settings_Menu / Settings_Namespace_View | configuration-system (Req 15) + menu-workspace | -- | Watch | Settings UI shared -- PA-SPLIT-001 folds Req 15 into menu-workspace |
| `LogLevel` / `LogConfig` / `LogRecord` / `PluginLogHandle` | logging-subsystem | all crates | No | Sole owner ff-logging |
| `log_trace!`/`log_debug!`/`log_info!`/`log_warn!`/`log_error!` (macros) + `log()`/`log_lazy()` | logging-subsystem | all crates | No | Sole owner ff-logging |
| logging.* config keys | logging-subsystem (consumer) + configuration-system (owns `logging` namespace) | -- | No | Two-phase init (CR-CH-015, B033); ff-logging inits before ff-config |
| `CommandId` / `CommandParams` / `CommandResult` / `CommandRegistry` / `ExecutionContext` | command-framework | all crates | No | Sole owner ff-command |
| `ShortcutBinding` / `Chord` | command-framework | ff-desktop | No | Sole owner ff-command |
| `CommandTarget` (5 variants, incl. produces_visible_workspace) | command-framework | menu-workspace, command-configurator, startup-and-session, shell-command, lua-macro-engine | Watch | PA-WATCH-001: verify Target_Resolution + Req 10.10 predicate reuse |
| Context_Navigation_Stack + `navigation.stack_max_depth` (CR-NR-057 Req 10) | command-framework | startup-and-session, menu-workspace, function-keys-and-history; paired w/ command-semantics Req 11 | Watch | PA-INCOMPLETE-003 (unbuilt) + PA-WATCH-001; config key not yet registered |
| reserved `arg` param key + verb/arg split (Req 9) | command-framework | all input sources | Watch | Req 9 DEFINED but UNIMPLEMENTED (PA-INCOMPLETE-001) |
| `Document` / `DocumentHandle` / `TextBuffer` / `GapBuffer` / `LineIndex` | document-model | edit-operations, display-line-mapping, viewport, undo-redo, ff-desktop | No | Sole owner ff-document-model |
| `BytePosition` / `LineNumber` / `CharacterExtracted` / `DocumentWatcher` | document-model | edit/nav consumers | No | Sole owner ff-document-model |
| `LineEndMode` | document-model (owns) + encoding-and-characters (validates bytes) | -- | No | No duplicate ownership |
| VFS-only I/O (Req 4.8) | document-model (consumer) + virtual-file-system (owner) | -- | No | VERIFIED no std::fs/tokio::fs (FFW-ARCH-001) |

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
| _(none recorded yet)_ | | | | |

## Dangling Cross-References

| From unit | Missing target | Correction |
|-----------|----------------|------------|
| _(none recorded yet)_ | | |
