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
| Fold-level storage/computation | syntax-highlighting (owner: `Lexer::fold_text`+`FoldContext`+`fold_level_at`, Req 8/15) | display-line-mapping (CONSUMES via API; stores only visibility+expanded+collapsed-label, Req 10.7) | No | CONFIRMED W1.10: clean single-owner boundary. display-line-mapping's 18 `fold_*` refs are ALL `fold_text` = collapsed-label STRING field, NOT level computation. Matches Req 15.1 (query exclusively, never compute). W1.4 PA-WATCH RESOLVED. NB: `fold_text` spelling reused across crates for different things (method vs field) -- coincidental, no collision |
| `SyntaxHighlighter`/`Lexer`/`LexerRegistry`/`StyleBuffer`/`StyleContext`/`WordList`/`HighlightEngine`/`FoldData`/`FoldContext`/`SubStyleAllocator` | syntax-highlighting (`ff-syntax-highlighting`) | theme (slot resolution), display-line-mapping (fold API), viewport (ensure_styled_to), text-decorations (peer) | No | Sole owner; W1.10 -- no duplication. GUI-independent (0 egui/colour refs, Req 11/12.1 verified) |
| `StyleSlotIndex` (u8) as theme boundary | syntax-highlighting (produces) + theme-and-appearance (resolves to attributes) | viewport renderer | No | Engine emits abstract slots only; theme owns colour/font. Clean layering (Req 12) |
| HILITE command (Req 16) | syntax-highlighting (executes modes: ON/OFF/LOGIC/PAREN/FIND) + edit-operations (parses/stores, delegates -- Req 16.12) | command-framework, find-and-replace (HILITE FIND extends highlight-all) | Watch | PA-WATCH: HILITE split across edit-operations(parse/store) and syntax-highlighting(execute); confirm delegation seam at Wave 3/4 command wiring |
| Logical line-exclusion layer (`ExclusionEngine`, EXCLUDE/SHOW/RESET/X commands, blocks, placeholder text) | exclude-show-filter (`ff-exclude-show-filter`) | command-semantics (matrix), line-commands (X/Xn/XX), find-and-replace (scope iterators), viewport (placeholders) | No | Sole owner of exclusion SEMANTICS; W1.11 |
| `DisplayLineMapping` trait consumed for visibility storage | display-line-mapping (owner, W1.4) | exclude-show-filter (CONSUMES CANONICAL correctly, W1.11) | No | POSITIVE counter-example to PA-CONFLICT-001: exclude-show-filter imports `ff_display_line_mapping::DisplayLineMapping` and drives set_visible/get_visible/hidden_lines/show_all -- no local duplicate. Reinforces that VIEWPORT is the outlier that duplicates the trait |
| shared visibility bit: flat exclusion vs nested code-folding | display-line-mapping (arbitrates combined result) | exclude-show-filter (flat, W1.11) + syntax-highlighting/fold (nested levels, W1.10) | Watch | PA-WATCH-010: both write visibility into the same layer; confirm no double-toggle race when a line is both folded and excluded (Wave 4) |
| FIND/CHANGE EXCLUDED\|VISIBLE scope | exclude-show-filter (exposes `visible_lines_iter`/`excluded_lines_iter`) | find-and-replace (consumes) | No | Single owner, many readers -- consistent with W1.8 find-and-replace record |
| `WhitespaceSettings` + mode enums (WhitespaceVisibility/TabDrawMode/IndentGuideMode/EdgeMode/WrapVisualFlag/WrapVisualLocation/WrapIndentMode) + query result types | whitespace-and-guides (`ff-whitespace-guides`) | GUI shell (renderer), command-framework (toggles) | No | Sole owner; W1.12. GUI-independent (0 egui/winit/wgpu, Req 9 verified) |
| wrap sub-line count for wrap-marker computation | display-line-mapping (produces) -> GUI shell -> whitespace-and-guides (consumes as PARAMETER) | -- | No | CLEAN (W1.12): whitespace-guides has NO crate dep on display-line-mapping; `compute_wrap_markers(sub_line_count, ...)` takes the count as a param. "Consumer" relationship realized at caller layer, not a crate edge. Contrast PA-CONFLICT-001 (viewport duplicates the trait) |
| `editor.whitespace_*`/`editor.tab_draw_mode`/`editor.indent_guides`/`editor.edge_*`/`editor.wrap_*` config keys | whitespace-and-guides (consumer) + configuration-system (owns namespace) | -- | No | Sole reader; hot-reload; no key collision with other units |
| Toggle commands (ToggleWhitespace/ToggleIndentGuides/ToggleEdgeColumn) | whitespace-and-guides | command-framework, configuration-system (user-layer persist Req 8.4) | No | Sole owner ff-whitespace-guides |
| `IndentDecision`/`CommentContinuation`/`BlockExpansion` (indent decision data) | auto-indentation (`ff-auto-indent`) | edit-operations/shell (wraps into EditorTransaction) | No | Sole owner; W1.13. Returns plain data; caller applies via edit-operations API (Req 10.4) |
| auto-indent transaction wrapping | auto-indentation PRODUCES decision data -> caller wraps in edit-operations `EditorTransaction` | -- | No | CLEAN seam for PA-CONFLICT-002 (W1.13): ff-auto-indent has NO dep on ff-edit-operations/ff-undo (deps = ff-logging+regex only); decoupled by design (same pattern as sequence-numbers W1.9). Third Wave-1 model crate that stays out of PA-CONFLICT-002 via return-data injection |
| `edit.indent`/`edit.unindent` (Tab/Shift+Tab) | auto-indentation (computes) + command-framework (registers) | edit-operations (applies) | No | Consistent; caller registers, auto-indent computes indent delta |
| `editor.auto_indent`/`editor.indent_size`/`editor.tab_size`/`editor.use_tabs` + `[indent]`/`[comment]` TOML | auto-indentation (consumer) + configuration-system + language-service | -- | No | Read via param injection; no key collision |
| `WrapMode` (None/Word/Character) + `WrapBoundary` (Viewport/Column(n)) + WRAP command (`view.wrap`) | line-wrap-toggle (`ff-wrap`) | display-line-mapping (set_height, caller-bridged), viewport, menu/statusbar, session | No | Sole owner of wrap MODE state; W1.14. Non-undoable, not in history (Req 3.12/3.13). No coupling deps (thiserror+serde only) |
| `WrapIndentMode` (Fixed/Same/Indent/DeepIndent) | line-wrap-toggle (`ff-wrap` indent.rs) + whitespace-and-guides (`ff-whitespace-guides` wrap_indent_mode.rs) -- BOTH define it | -- | CONFLICT | PA-CONFLICT-004 (W1.14): duplicate unbridged enum in two crates; neither imports the other. Recommend whitespace-and-guides as sole owner (its Req 6-7 + line-wrap Req 10.7 "rendered using whitespace-guides infrastructure"); ff-wrap consumes |
| wrap visual flags (None/End/Start/Margin) | line-wrap-toggle (`ff-wrap::WrapVisualFlags` ENUM) + whitespace-and-guides (`ff-whitespace-guides::WrapVisualFlag(u8)` BITFIELD) | -- | CONFLICT | PA-CONFLICT-004 (W1.14): duplicated AND structurally divergent (enum vs bitfield newtype). Unify shape under one owner (whitespace-and-guides recommended) |
| wrap config keys (`[view.wrap]` vs `editor.wrap_visual_flags`/`editor.wrap_indent_mode`) | line-wrap-toggle (`[view.wrap]`) + whitespace-and-guides (`editor.wrap_*`) | configuration-system | CONFLICT | PA-CONFLICT-004 (W1.14): TWO config surfaces for the same wrap indent/flag settings. Unify key ownership with the type ownership |
| wrap-inactive -> no-markers gate | line-wrap-toggle (Wrap None -> set_height 1, Req 6.2/10.5) + whitespace-and-guides (sub_line_count<=1 -> None, Req 6.9) | -- | No | RESOLVED W1.14: the two specs agree; Wrap None collapses every line to height 1, so whitespace-guides' guard fires. W1.12 PA-WATCH cleared |
| `IndicatorStyle`/`Indicator`/`Decoration`/`DecorationList`/`RunStyles`/`LineMarker`/`DecorationRenderer` | text-decorations (`ff-text-decorations`) | find-and-replace (producer), viewport (renderer), theme, display-line-mapping | No | Sole owner; W1.15. `RunStyles` verified UNIQUE (no duplicate in any crate) -- contrast the wrap/CaseFolder conflicts |
| decoration storage vs syntax style buffer (`under` property + layer order) | text-decorations (Decoration/RunStyles) + syntax-highlighting (StyleBuffer) -- SEPARATE stores | -- | No | CONFIRMED clean PEER W1.15/W1.10: independent storage; `under` (Req 2.2) + layer order (Req 14.1) compose them; matches syntax-highlighting Req 15.3-15.5 |
| indicator number namespace (0-7 lexer, 8-31 container, 32-35 IME, 36-43 history) | text-decorations (Req 13) | syntax-highlighting (owns 0-7), plugins (8-31) | No | Clean namespace split; separate from syntax STYLE-SLOT u8 space (no collision) |
| search-match indicators (INDICATOR_SEARCH_CURRENT/ALL) | text-decorations (owns storage) | find-and-replace (fills via fill_range), HILITE FIND (syntax-highlighting Req 16.4) | No | Single owner, producer writes. Ties PA-WATCH-009 (HILITE FIND toggles these) |
| Dataset catalog metadata / DSN / GDG / PDS / `catalog` VfsProvider / StorageProvider / record codecs | dataset-catalog (`ff-dscatalog`) | file-tree-panel, fileforge-integration, ff-idcams (query), ff-dsalloc (CRUD) | No | Sole authority per ADR-001 (dataset-ownership-model). W2.1. STRONG split PA-SPLIT-008 would re-home codecs+storage |
| catalog CRUD primitives vs JCL-driven allocation | dataset-catalog (`ff-dscatalog`, low-level CRUD, ADR-001) | dataset-allocator (`ff-dsalloc`, JCL/DISP workflows -- INVOKES, does not duplicate) | Watch | PA-WATCH-011: verify ff-dsalloc invokes catalog CRUD (W2.3); pre-declared boundary Req 7 |
| `catalog.listcat`/`listds` (workbench-native) vs `idcams.listcat` (IDCAMS-faithful) | dataset-catalog (native tabular) + ff-idcams (formatted, CALLS catalog API) | -- | Watch | PA-WATCH-011: verify ff-idcams delegates to catalog query API (Wave 5); pre-declared Req 13 |
| `StorageProvider` trait + NativeFileProvider + SqliteRecordProvider + record codecs | dataset-catalog today (`ff-dscatalog` storage/ + codecs/) | -- | Watch | PA-SPLIT-008 proposes extracting to `ff-record-codec` (Req 17.1 mandates independence) + `ff-record-store`/`ff-vsam` (Req 18-24); ff-dscatalog then consumes |
| direct fs I/O in a VFS provider | dataset-catalog (`ff-dscatalog`, 49 fs calls) | -- | No | CLEAN (W2.1): this crate IS the terminal `catalog` VfsProvider + NativeFileProvider (Req 10/19.5); FFW-ARCH-001 governs CONSUMERS, not the provider. Contrast PA-WATCH-004 (workflow non-provider raw I/O) |
| `posix` VfsProvider (scheme `posix`) | ff-vfs (`ff-vfs/src/posix_provider.rs`, 664 -- CORRECT home) + ff-desktop (`ff-desktop/src/posix_provider.rs`, 404 -- DUPLICATE, `impl VfsProvider`, `#[allow(dead_code)]`) | virtual-catalog-manager UI | CONFLICT | PA-CONFLICT-005 (W2.2): two posix-scheme provider impls AND a VfsProvider living in the SHELL (layering violation vs FFW-ARCH-001 / VFS ownership W0.11). Delete the ff-desktop copy; VCM consumes the ff-vfs provider via the registry |
| Virtual Catalog Manager UI (Catalog Explorer Context, catalog-manager/dataset-alloc/POSIX dialogs, `Catalog_Registry`) | virtual-catalog-manager (in `ff-desktop`, no dedicated crate) | dataset-catalog (delegatee), file-tree-panel (embed), connector-local-fs (Native) | No | Sole owner (shell UI); W2.2. Delegates mainframe CRUD to ff-dscatalog (ADR-001) |
| catalog mount/registry persistence | virtual-catalog-manager (`[virtual_catalogs]`, all 3 types) + dataset-catalog (`[catalog].mounted_catalogs`, mainframe-only) | configuration-system | Watch | PA-WATCH-012 (W2.2): confirm these are not two competing persistence stores for the same mainframe mounts |
| `CatalogProvider`/`CatalogService` trait (catalog access seam) | dataset-allocator (`ff-dsalloc` defines the trait + MockCatalog) | shell wires production impl -> ff-dscatalog | No | CLEAN (W2.3): PA-WATCH-011 CRUD half resolved. ff-dsalloc delegates all catalog CRUD/resolution/GDG via trait (ADR-001 Req 4.7); no ff-dscatalog dep, no duplicate catalog storage |
| DSN naming validation (`DatasetName::parse` / `validate_dsn`) | dataset-catalog (ADR-001 Req 3.1 SOLE owner: `dsn.rs` + `validate_dsn`) + dataset-allocator (`ff-dsalloc/dsn.rs` OWN `DatasetName::parse`) | -- | CONFLICT | PA-CONFLICT-006 (W2.3): ff-dsalloc duplicates the DSN syntax validation ADR-001 assigns exclusively to ff-dscatalog. Route via CatalogProvider::validate_dsn; delete/justify the local validator. (DD-statement parsing IS legitimately the allocator's) |
| VSAM record ops (KSDS/ESDS/RRDS/LDS) | ADR-001 Req 5 owner = `ff-vsam-services` (trait-only stub today) BUT implementation lives in `ff-dscatalog/src/storage/` | -- | CONFLICT | PA-CONFLICT-007 (W2.4): VSAM impl MISPLACED -- ff-vsam-services exists with the `VsamService` trait (Req 16.7 interim OK) but the code (esds/rrds/isam/sqlite_record 843) is in ff-dscatalog. Migrate the impl into ff-vsam-services. Subsumes PA-SPLIT-008 VSAM half as a MIGRATION (target crate already exists) |
| `CatalogService`/`DynCatalogService`/`VsamService` trait contracts | dataset-ownership-model ADR-001: `ff-dataset-catalog/src/traits.rs` (CatalogService -- INTERFACE crate) + `ff-vsam-services/src/traits.rs` (VsamService) | ff-dsalloc, ff-idcams (consume via trait) | No | W2.4: ff-dataset-catalog is the SHARED-INTERFACE crate (PA-DEP-002 WITHDRAWN -- not a dead stub); ff-dscatalog is the impl |
| ADR-001 dependency-direction fitness function | dataset-ownership-model (`ff-governance-tests/tests/architecture_compliance.rs`, Req 18) | whole catalog cluster | Watch | PA-WATCH-013 (W2.4): the suite checks `ff-dataset-catalog` (interface) but NOT `ff-dscatalog` (impl) or ff-dsalloc -- impl crates could acquire a prohibited dep undetected. Extend coverage |
| record-structure model (`FieldDefinition`/`RecordStructure`/field-type enum) | fileforge-integration (`ff-forge`: `DataType`/`FieldDefinition`/`RecordStructure`, declared owner) + structure-catalog (`ff-structure-catalog`: OWN `FieldType`/`FieldDefinition`/`RecordStructure`, NO dep on ff-forge) | -- | CONFLICT | PA-CONFLICT-008 (W2.5): two unbridged record-structure models across crates the spec says should share; structure-catalog "extends fileforge-integration" in prose but redefines its types. Unify under ff-forge; structure-catalog adds only the catalog library/persistence/.ffs layer |
| `.ffs` structure file format + catalog persistence | structure-catalog (`ff-structure-catalog`, `ffs_format.rs`) | -- | No | Library/persistence concern legitimately structure-catalog's; the record DATA MODEL it persists should be ff-forge's (PA-CONFLICT-008). ff-forge `StructureFile` overlap folds into PA-CONFLICT-008 |
| `catalog.*` config root | structure-catalog (`catalog.locations`/`catalog.active_location`) + dataset-catalog (`[catalog].mounted_catalogs`/`default_hlq`) | configuration-system | Watch | PA-WATCH-014 (W2.5): two different domains (record-structure library vs dataset emulation) share the `catalog` config prefix; confirm no key collision |
| field-type model (3rd parallel enum) | ff-forge `DataType` (owner) / structure-catalog `FieldType` / record-selection-criteria `FieldDataType` (`ff-select`) | -- | CONFLICT | PA-CONFLICT-008 EXTENSION (W2.6): THREE parallel field-type enums across ff-forge / structure-catalog / ff-select, none sharing. ff-select has NO ff-forge dep so cannot delegate COMP-3 decode as its spec requires. Unify under ff-forge |
| Criteria_Store persistence | record-selection-criteria (`ff-select`, raw `std::fs`) -- spec says configuration-system-managed (Req 9.1) | configuration-system (declared, NOT a dep) | Watch | PA-WATCH-015 (W2.6): config mediation BYPASSED -- store spec'd as config-user-layer but implemented with raw std::fs + no ff-config dep; .criteria.json outside VFS. Route through ff-config/ff-vfs or document caller-mediated design |
| Criteria_Scope (FIND/CHANGE restriction) | record-selection-criteria (`ff-select` exposes filter) | find-and-replace (consumes as SearchScope) | No | Single-owner-many-readers; mirrors exclude-show-filter EXCLUDED/VISIBLE (W1.11). Confirm wiring at Wave 5 |
| scroll-amount: commands (M/MAX/n) vs field (CSR/PAGE/HALF) | navigation-commands (Req 20 commands, DELEGATES) + viewport-and-scrolling (Req 14 field + state/clamping) | -- | No | PA-WATCH-008 RESOLVED W1.7: single source of truth; nav delegates viewport state; complementary layers |
| UP/DOWN/LEFT/RIGHT/TOP/BOTTOM/LOCATE/SORT/COLS/BOUNDS commands | navigation-commands | command-framework, viewport-and-scrolling (delegate) | No | Sole owner ff-navigation-commands |
| active Bounds state + query API (Req 5.15) | navigation-commands (owner) | line-commands (bounds-aware shift), find-and-replace (CHANGE/FIND), SORT | No | Single owner, multiple readers -- consistent |
| `FindEngine` / `FindRequest` / `FindResult` / `RegexEngine` / `FindState` | find-and-replace | command-semantics, exclude-show-filter, ff-desktop | No | Sole owner ff-find-and-replace |
| `CaseFolder` / `ICaseConverter` | encoding-and-characters (declared owner, Req 10) + find-and-replace (DUPLICATE own impl) | -- | CONFLICT | PA-CONFLICT-003: find-and-replace defines its own CaseFolder instead of consuming ff-encoding's via ICaseConverter; not bridged; contrary to its own cross-ref |
| `RegexEngine` (NFA) | find-and-replace | -- | No | Sole owner; PA-SPLIT-007 proposes extracting to `ff-regex` |
| `ColumnRange`/`SequenceFormat`/`DetectionResult`/`SeqNumState`/`SideTableEntry`/`SeqNumConfig`/`NumberEngine`/`SeqNumIndicator` | sequence-numbers (`ff-seqnum`) | ff-desktop (renderer/statusbar), viewport (overlay) | No | Sole owner; W1.9 -- no duplication found |
| `UndoRecorder` (trait) / `ColumnChange` | sequence-numbers (defines trait) | shell supplies concrete recorder | No | CLEAN seam for PA-CONFLICT-002 (W1.9): ff-seqnum does NOT bind to `Transaction` or `EditorTransaction`; concrete transaction model injected at runtime. Shell wiring must back it with the winning model |
| `sequence.unnum`/`sequence.number`/`sequence.number_show` (+ NUM/AUTONUM aliases) | sequence-numbers | command-framework (registry), command-semantics (matrix Req 14) | No | Sole owner ff-seqnum |
| `editor.sequence_numbers.*` config keys | sequence-numbers (consumer) + configuration-system (owns namespace) | -- | No | Layered override model; consistent with config-system ownership |
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
