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
| `dev-logging` cargo feature + `BUILD_PROFILE_LEVEL` const (CR-NR-058 Req 13) | logging-subsystem | ALL crates (compile-time gate for their own TRACE/DEBUG) | Watch | UNIMPLEMENTED (PA-CR058). Compile-time gate; downstream crates key their dev diagnostics on the exported const |
| per-command instrumentation at `execute_command` (CR-NR-058 Req 11) | command-framework | all invocation sources (keyboard/menu/cmd-line/macro/plugin) | Watch | UNIMPLEMENTED (PA-INCOMPLETE-004). Uniform start/params/completion at DEBUG; leverages dev-logging gate |
| `Transaction` / `EditOperation` / `UndoStack` / `RedoStack` / `ScrapStack` / `SavePoint` | undo-redo-transactions | command-framework, document-model, edit-operations | No | Sole owner ff-undo-redo (Scintilla model) |
| `SelectionPosition` / `SelectionRange` / `Selection` (LOGICAL model) | edit-operations (SOLE owner) | caret-and-selection (renders), undo-redo (own snapshot) | No | RESOLVED W1.5: sole owner; no duplication |
| `EditorTransaction` (line-snapshot model) vs `Transaction`/`ScrapStack` (undo-redo) | edit-operations + undo-redo-transactions | ff-line-commands PRODUCES EditorTransaction | CONFLICT | PA-CONFLICT-002 CONFIRMED W1.5: two unbridged undo-unit models; EditorTransaction de-facto canonical; undo-redo model unwired (ff-line-commands never uses its ff-undo-redo dep) |
| CAPS / edit profile | edit-operations (Req 16-17) | configuration-system (persistence) | No | Sole owner ff-edit-operations |
| LineCommand / BlockCommand / PendingCommand / CompatibilityMatrix | line-commands | command-semantics (collection step, W3.1) | No | Sole owner ff-line-commands |
| undoable line-command output | line-commands PRODUCES edit-operations `EditorTransaction` | -- | No | W1.6 evidence for PA-CONFLICT-002; ff-undo-redo dep unused (PA-DEP-001) |
| X/XX exclusion visibility | line-commands (consumes) | exclude-show-filter + display-line-mapping (canonical DisplayLineMapping) | No | Confirmed W1.4/W1.6; exclude-show-filter authoritative for SHOW restore (W1.11) |
| `SelectionState` (undo snapshot) | undo-redo-transactions (Req 9) | -- | Watch | PA-WATCH-006: vs edit-operations Selection (W1.5) + caret-and-selection rendering (W1.3) |
| `RecoveryPayload` (serialize-only) | undo-redo-transactions | file-operations/background-io (perform I/O) | No | Crate serialises to bytes+CRC32; caller does the write -- GUI/IO-independent |
| `Viewport` / `CaretPolicy` / `ScrollMode` / `ViewportError` | viewport-and-scrolling | ff-desktop (renderer), editor session | No | Sole owner ff-viewport-scrolling |
| scroll commands (ScrollPageUp/Down etc.) | viewport-and-scrolling | command-framework | No | Non-undoable (Req 10.6) -- consistent with undo-redo Req 10 |
| `DisplayLineMapping` (canonical trait) | display-line-mapping (owner, Req 7.10) | ff-line-commands, ff-exclude-show-filter, ff-desktop | No | Sole owner ff-display-line-mapping; correctly consumed by 3 crates |
| `DisplayLineMapper` (viewport-local) vs `DisplayLineMapping` (canonical) | viewport-and-scrolling (DUPLICATE local) + display-line-mapping (owner) | -- | CONFLICT | PA-CONFLICT-001 CONFIRMED W1.4: viewport defines its own trait, never imports canonical; not bridged; contrary to viewport Req 11.1 |
| `ContractionState` / DocLine / DisplayLine / SubLine | display-line-mapping | consumers | No | Sole owner ff-display-line-mapping |
| Fold-level storage | syntax-highlighting/language-service (owner, W1.10) NOT display-line-mapping | display-line-mapping (stores only visibility+expanded, Req 10.7) | No | Clean boundary; confirm at W1.10 |
| scroll-amount: commands (M/MAX/n) vs field (CSR/PAGE/HALF) | navigation-commands (Req 20 commands, DELEGATES) + viewport-and-scrolling (Req 14 field + state/clamping) | -- | No | PA-WATCH-008 RESOLVED W1.7: single source of truth; nav delegates viewport state; complementary layers |
| UP/DOWN/LEFT/RIGHT/TOP/BOTTOM/LOCATE/SORT/COLS/BOUNDS commands | navigation-commands | command-framework, viewport-and-scrolling (delegate) | No | Sole owner ff-navigation-commands |
| active Bounds state + query API (Req 5.15) | navigation-commands (owner) | line-commands (bounds-aware shift), find-and-replace (CHANGE/FIND), SORT | No | Single owner, multiple readers -- consistent |
| `FindEngine` / `FindRequest` / `FindResult` / `RegexEngine` / `FindState` | find-and-replace | command-semantics, exclude-show-filter, ff-desktop | No | Sole owner ff-find-and-replace |
| `CaseFolder` / `ICaseConverter` | encoding-and-characters (declared owner, Req 10) + find-and-replace (DUPLICATE own impl) | -- | CONFLICT | PA-CONFLICT-003: find-and-replace defines its own CaseFolder instead of consuming ff-encoding's via ICaseConverter; not bridged; contrary to its own cross-ref |
| `RegexEngine` (NFA) | find-and-replace | -- | No | Sole owner; PA-SPLIT-007 proposes extracting to `ff-regex` |
| Caret/selection VISUAL config (CaretStyle, SelectionColours, BlinkState) | caret-and-selection | ff-desktop (renderer), theme | No | Sole owner ff-caret-selection -- RENDERING only |
| `Selection` / `SelectionRange` / `SelectionPosition` (LOGICAL model) | edit-operations (owner, W1.5) | caret-and-selection (renders; does NOT duplicate -- W1.3 verified), undo-redo (own snapshot) | Watch | PA-WATCH-006: confirm undo-redo SelectionState references it at W1.5 |
| `CommandId` / `CommandParams` / `CommandResult` / `CommandRegistry` / `ExecutionContext` | command-framework | all crates | No | Sole owner ff-command |
| `ShortcutBinding` / `Chord` | command-framework | ff-desktop | No | Sole owner ff-command |
| `CommandTarget` (5 variants, incl. produces_visible_workspace) | command-framework | menu-workspace, command-configurator, startup-and-session, shell-command, lua-macro-engine | Watch | PA-WATCH-001: verify Target_Resolution + Req 10.10 predicate reuse |
| Context_Navigation_Stack + `navigation.stack_max_depth` (CR-NR-057 Req 10) | command-framework | startup-and-session, menu-workspace, function-keys-and-history; paired w/ command-semantics Req 11 | Watch | PA-INCOMPLETE-003 (unbuilt) + PA-WATCH-001; config key not yet registered |
| reserved `arg` param key + verb/arg split (Req 9) | command-framework | all input sources | Watch | Req 9 DEFINED but UNIMPLEMENTED (PA-INCOMPLETE-001) |
| `Document` / `DocumentHandle` / `TextBuffer` / `GapBuffer` / `LineIndex` | document-model | edit-operations, display-line-mapping, viewport, undo-redo, ff-desktop | No | Sole owner ff-document-model |
| `BytePosition` / `LineNumber` / `CharacterExtracted` / `DocumentWatcher` | document-model | edit/nav consumers | No | Sole owner ff-document-model |
| `LineEndMode` | document-model (owns) + encoding-and-characters (validates bytes) | -- | No | No duplicate ownership |
| VFS-only I/O (Req 4.8) | document-model (consumer) + virtual-file-system (owner) | -- | No | VERIFIED no std::fs/tokio::fs (FFW-ARCH-001) |
| `Vfs` / `VfsProvider` / `StorageProvider` / `ProviderRegistry` / `ResourceUri` / `VfsError` | virtual-file-system | ALL crates (FFW-ARCH-001) | No | Sole owner ff-vfs; only crate allowed direct std::fs |
| `WatchHandle` / `WatchEvent` / `VfsTransaction` | virtual-file-system | document-model, external-modification, dataset-catalog | No | Sole owner ff-vfs |
| workspace backup manifest | virtual-file-system (Req 12.2) + dataset-catalog (ff-dscatalog Req 26.3) | -- | Watch | PA-WATCH-003: verify shared-vs-distinct manifest type at Wave 2 |
| `FileForgePlugin` / `PluginContext` / `PluginError` / `Capability` / `PluginRegistry` / `PLUGIN_API_VERSION` | plugin-architecture | all plugin crates, shell | No | Sole owner ff-plugin |
| plugin namespace `[plugins.{name}]` | plugin-architecture (Req 2.7/7.5) + configuration-system (Req 8) | -- | No | Same scoping rule both specs |
| PluginLogHandle contract | plugin-architecture (uses) + logging-subsystem (Req 10 owns) | -- | No | Consistent `[plugin:name]` prefix |
| `Workflow` / `WorkflowDefinition` / `WorkflowRunner` / `WorkflowRegistry` / `Checkpoint` | workflow-engine | command-framework, plugins, shell | No | Sole owner ff-workflow |
| `CancellationToken` (workflow) | workflow-engine | steps, async I/O | No | Distinct from ff-background-io cancellation (W0.17) |
| checkpoint storage I/O | workflow-engine (`tokio::fs`) | virtual-file-system (would-be owner) | Watch | PA-WATCH-004: direct tokio::fs vs FFW-ARCH-001 -- owner decision |
| `IoError` / `ProgressState` / `IoTaskHandle` / `BackgroundIoService` / `RetryPolicy` | background-io | file-operations, document-model, shell | No | Sole owner ff-background-io (RetryPolicy defined but NOT wired -- PA-INCOMPLETE-002) |
| `IoCancellationToken` | background-io | callers | No | Distinct from ff-workflow CancellationToken |
| VFS-only I/O (Req 1.8/4.10/8.x) | background-io (consumer) + virtual-file-system (owner) | -- | No | VERIFIED upheld (only lib.rs doc-comment mentions std::fs) |
| `EncodingFamily` / `CharClassify` / `CharacterCategoryMap` / `CaseFolder` | encoding-and-characters | document-model, find, edit, nav | No | Sole owner ff-encoding |
| `ConversionResult` / `ConversionIssue` | encoding-and-characters | file-operations, document-model | No | Sole owner ff-encoding; lossy replacements returned to caller (Req 3.4) |
| LineEndMode interaction | encoding-and-characters (validates U+2028/2029/0085) + document-model (owns LineEndMode) | -- | No | ff-encoding validates bytes; document-model decides semantics -- no duplicate ownership |

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
