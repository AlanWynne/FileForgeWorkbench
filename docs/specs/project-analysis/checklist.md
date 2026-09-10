# Project Analysis Checklist

Status of every sub-project in the systematic analysis pass (CR-NR-056).
Status values: PENDING / IN PROGRESS / DONE. Wave is the planned analysis wave
(design section 3); it may be re-slotted mid-analysis with a recorded reason.

`project-analysis` itself is excluded (it is the analysis, not a subject).
`ears-integration` and `workbench-requirements-merge` are analysis-artifact
sub-projects reviewed in Wave 6 (review only).

**RE-BASELINED (restart) after CR-NR-057** (command chaining + Context Navigation
Stack, commit `b537737`). CR-NR-057 changed foundational core specs
(command-framework Req 10, command-semantics, lua-macro-engine, menu-workspace),
so the prior analysis pass (W0.1-W1.5, committed through `edb690d`) is
superseded. All sub-projects reset to PENDING; re-run from W0.1 against the new
baseline. The prior records remain in git history (up to `edb690d`) for
reference but are NOT carried forward.

**Wave 0 COMPLETE (W0.1-W0.21) on the re-baseline** (commits `7330b06`..`df79cd0`).
All 10 foundation sub-projects analysed against the CR-NR-057 + CR-NR-058 specs;
the Wave 0 task-revision (W0.21) recorded the dependency-ordered remediation as
`Phase PA-W0` (PROPOSAL, PA-W0.1-PA-W0.23) in
`docs/specs/project-master/tasks.md`. Wave 0 outcome: 3 clean/complete
(platform-core, document-model, plugin-architecture); 3 complete-with-proposals
(configuration-system split, virtual-file-system IWR-005-resolved,
encoding-and-characters borderline split); command-framework INCOMPLETE on THREE
gated-but-unbuilt reqs (Req 9 args, Req 10 nav-stack CR-NR-057, Req 11
per-command instrumentation CR-NR-058); workflow-engine (Req 7.6 + tokio::fs);
virtual-file-system (PA-LOG-004 eprintln defect); background-io (Req 6.6-6.9
unimplemented, false-positive tests, HIGH); 2 tracking gaps (config CQ tasks
30-31, logging Req 12 tool); logging-subsystem gained CR-NR-058 Req 13
(build-profile dev-logging gate, UNIMPLEMENTED). Top remediation priority: the
CR-NR-058 dev/debug-logging foundation (PA-W0.1 gate then PA-W0.2 per-command
instrumentation).

**Wave 1 COMPLETE (W1.1-W1.16) on the re-baseline** (commits `fea70d6`..`ec744e2`).
All 15 editor-core sub-projects analysed; the Wave 1 task-revision (W1.16) recorded
the dependency-ordered remediation as `Phase PA-W1` (PROPOSAL, PA-W1.1-PA-W1.12) in
`docs/specs/project-master/tasks.md`. Wave 1 outcome: NO spec split forced (all 15
cohesive); ALL tracking-complete EXCEPT auto-indentation (PA-INCOMPLETE-006, HIGH:
Req 9.7 WARN commented out + Req 10.7 per-decision DEBUG absent while 137/137 `[x]`
-- the clearest CR-NR-058 dev-logging-in-spec case). New cross-unit conflict
PA-CONFLICT-004 (WrapIndentMode + wrap-visual-flag DUPLICATED across ff-wrap and
ff-whitespace-guides, unbridged, enum-vs-bitfield divergence). Recurring Wave-1
patterns: three TOTAL-TCR-absence crates (exclude-show-filter, whitespace-guides,
text-decorations) + four thin ones; dead/unused ff-logging deps on several pure
models; NON-ASCII chars inside RUNTIME error/warning strings (a real output defect,
not just comment style) across 5 crates; three 400-cap violations (highlight_engine
462, exclusion_engine 535, run_styles 410). Confirmations (resolved in-analysis):
three model crates (seqnum/whitespace/auto-indent) stay OUT of PA-CONFLICT-002 via
return-data seams; fold-level ownership clean (syntax-highlighting owns,
display-line-mapping consumes); exclude-show-filter is a POSITIVE counter-example to
PA-CONFLICT-001 (consumes the canonical DisplayLineMapping); syntax/text-decorations
peer boundary clean (independent RunStyles). Carried watches: PA-WATCH-009 (HILITE
delegation, Wave 3/4), PA-WATCH-010 (exclusion+folding visibility, Wave 4).

**Wave 2 COMPLETE (W2.1-W2.8) on the re-baseline** (commits `a78ff62`..`337e096`).
All 7 catalog/dataset sub-projects analysed; the Wave 2 task-revision (W2.8) recorded
the dependency-ordered remediation as `Phase PA-W2` (PROPOSAL, PA-W2.1-PA-W2.20) in
`docs/specs/project-master/tasks.md`. Wave 2 outcome: ALL tracking-complete, NO
functional PA-INCOMPLETE; the cluster is ADR-001-governed with a real fitness
function (`ff-governance-tests`) -- one of the best-enforced areas. Dominant theme:
DOMAIN-TYPE FRAGMENTATION -- specs say crates should share one owner but each
redefines the type: PA-CONFLICT-006 (DSN validator in ff-dsalloc vs catalog-owned
validate_dsn), PA-CONFLICT-007 (VSAM impl in ff-dscatalog storage/ vs ADR-owner
ff-vsam-services trait-only stub -- subsumes the PA-SPLIT-008 VSAM half as a
MIGRATION), PA-CONFLICT-008 (record-structure/field-type model duplicated x3 across
ff-forge/structure-catalog/ff-select), PA-CONFLICT-005 (posix VfsProvider duplicated
+ MISPLACED in the ff-desktop shell). dataset-catalog is the LARGEST unit (31 reqs/
816 lines, STRONG split PA-SPLIT-008) and the TCR EXEMPLAR (99 rows) -- vs total/thin
TCR on the other six (PA-TCR-010/011/012/013). Recurring: dead ff-logging deps +
mandated-but-absent WARN/INFO across nearly every unit (PA-LOG-012..017, top target
PA-LOG-012 on the transactional catalog); SEVERE cap violations in the ff-desktop VCM
UI (files_panel.rs 1195, PA-STD-025); non-ASCII incl. BOM/mojibake + runtime strings
(PA-STD-024..031); crate-name drift on nearly every spec (PA-W2.20). Confirmations:
ADR-001 fitness-function enforced; ff-dsalloc CRUD delegation clean (PA-WATCH-011 CRUD
half resolved); ff-dscatalog's 49 fs are LEGITIMATE (it IS the VFS provider);
PA-DEP-002 WITHDRAWN (ff-dataset-catalog = intentional interface crate). Carried
watches: PA-WATCH-011 (ff-idcams listcat, Wave 5), 012/013/014/015 (Wave 3 config +
fitness-fn coverage), 016 (Display_Artifact_Line, Wave 4).

| # | Sub-project | Wave | Status | Analysis Record | Notes |
|---|-------------|------|--------|-----------------|-------|
| 1 | platform-core | 0 | DONE | units/platform-core.md | COMPLETE (100/100, TCR PASS); not a split; GUI-independence VERIFIED; logging EXEMPLARY; refactor PA-STD-001 (event_bus.rs 435); PA-DOC-001 (Req 4.1 crate-name drift); PA-LOG-001 (project-wide non-ASCII) |
| 2 | configuration-system | 0 | DONE | units/configuration-system.md | FUNCTIONALLY COMPLETE; SPLIT CANDIDATE (PA-SPLIT-001, 4/4); TRACKING GAP PA-TRACK-001 (Phase CQ tasks 30-31 stale, code+TCR done); refactor PA-STD-002 (6 files); PA-DOC-002 (Req 10-14 gap) |
| 3 | logging-subsystem | 0 | DONE | units/logging-subsystem.md | FUNCTIONALLY COMPLETE; Req 12 tool done but tracking stale (PA-TRACK-002); CR-NR-058 Req 13 build-profile gate IMPL DONE + verified (commit 4bf6fbc, tasks 25.1-25.4) but tests 25.6/25.7 + TCR + release-wiring 25.5 PENDING (PA-CR058); PA-LOG-002 audit data; refactor PA-STD-003 |
| 4 | command-framework | 0 | DONE | units/command-framework.md | INCOMPLETE (3 gaps): Req 9 Command Arguments (PA-INCOMPLETE-001) + Req 10 Context Navigation Stack CR-NR-057 (PA-INCOMPLETE-003) + Req 11 Uniform Command Instrumentation CR-NR-058 (PA-INCOMPLETE-004, HIGH-leverage dev/debug logging), all gate-complete/unbuilt; Req 1-8 done; not a split |
| 5 | document-model | 0 | DONE | units/document-model.md | COMPLETE (140/140, TCR PASS); not a split; VFS-only verified; PA-WATCH-002 (document.rs at 400 cap); PA-LOG-003 (optional logging); zero-log defensible |
| 6 | virtual-file-system | 0 | DONE | units/virtual-file-system.md | Feature-COMPLETE (91/91, Req 1-12 TCR-PASS); IWR-005 RESOLVED; LOGGING DEFECT PA-LOG-004 PERSISTS (eprintln + unmet Req 3.3 WARN); weak split PA-SPLIT-002; PA-WATCH-003 |
| 7 | plugin-architecture | 0 | DONE | units/plugin-architecture.md | COMPLETE (169/169, TCR PASS); not a split; logging EXEMPLARY (PA-LOG-REF-001); refactor PA-STD-004 (registry.rs 698) |
| 8 | workflow-engine | 0 | DONE | units/workflow-engine.md | Tracking-COMPLETE (124/124, TCR PASS); not a split; LOGGING GAP PA-LOG-005 (Req 7.6 unmet, checkpoint silent errors); FFW-ARCH-001 watch PA-WATCH-004 (checkpoint tokio::fs); refactor PA-STD-005 |
| 9 | background-io | 0 | DONE | units/background-io.md | INCOMPLETE despite 133/133 tasks: Req 6.6-6.9 (error log/retry/resume) NOT in load path (PA-INCOMPLETE-002, false-positive tests, HIGH); no TCR rows (PA-TCR-001); FFW-ARCH-001 verified upheld; not a split candidate |
| 10 | encoding-and-characters | 0 | DONE | units/encoding-and-characters.md | COMPLETE (111/111, TCR PASS); BORDERLINE SPLIT (PA-SPLIT-003, 14 reqs/3 bands); zero-log CORRECT (spec-mandated stateless); convert.rs refactor PA-STD-006; PA-WATCH-005 |
| 11 | undo-redo-transactions | 1 | DONE | units/undo-redo-transactions.md | COMPLETE (159/159, TCR PASS); SPLIT CANDIDATE (PA-SPLIT-004, 19 reqs/553 lines); logging gap PA-LOG-006 (Req 3.5 WARNING); refactor PA-STD-007 (manager.rs 600); crate-name drift PA-DOC-003; PA-CONFLICT-002 (two transaction models, resolve W1.5) |
| 12 | viewport-and-scrolling | 1 | DONE | units/viewport-and-scrolling.md | COMPLETE (129/129, TCR PASS); borderline (14 reqs) but NOT a split -- fix is refactor PA-STD-008 (viewport.rs 572); CONFLICT PA-CONFLICT-001 (own DisplayLineMapper trait, resolve W1.4); zero-log defensible; PA-DOC-004; watch PA-WATCH-008 |
| 13 | caret-and-selection | 1 | DONE | units/caret-and-selection.md | COMPLETE (127/127, TCR PASS); NOT a split (cohesive rendering layer, well-factored, no file over cap); zero-log defensible; PA-WATCH-006 partially resolved (visual layer, no logical-model duplication; final check W1.5) |
| 14 | display-line-mapping | 1 | DONE | units/display-line-mapping.md | COMPLETE (87/87, TCR PASS); not a split; CONFLICT PA-CONFLICT-001 CONFIRMED (viewport's own DisplayLineMapper vs canonical DisplayLineMapping owned here); refactor PA-STD-009 (contraction_state.rs 614); zero-log defensible |
| 15 | edit-operations | 1 | DONE | units/edit-operations.md | COMPLETE (280/280, TCR PASS); SPLIT CANDIDATE (PA-SPLIT-005, 17 reqs/553 lines); PA-WATCH-006 RESOLVED (selection ownership clean); CONFLICT PA-CONFLICT-002 CONFIRMED (EditorTransaction de-facto canonical vs unwired undo-redo model); zero-log defensible |
| 16 | line-commands | 1 | DONE | units/line-commands.md | COMPLETE (215/215, 13 TCR rows PASS); NOT a split (cohesive engine); corroborates PA-CONFLICT-002 (produces EditorTransaction, ff-undo-redo dep UNUSED = PA-DEP-001); refactor PA-STD-010 (resolution.rs 422); zero-log defensible |
| 17 | navigation-commands | 1 | DONE | units/navigation-commands.md | INCOMPLETE (Req 20 Scroll Amount Args, Phase DF, PA-INCOMPLETE-005, ships with cmd Req 9); Req 1-19 done; SPLIT CANDIDATE (PA-SPLIT-006, 20 reqs/624 lines -- SORT + COLS/BOUNDS seams); PA-WATCH-008 RESOLVED (delegates to viewport, no duplication); zero-log defensible |
| 18 | find-and-replace | 1 | DONE | units/find-and-replace.md | Tracking-COMPLETE (185/185); SPLIT CANDIDATE (PA-SPLIT-007, 20 reqs/437 lines -- extract NFA regex); CONFLICT PA-CONFLICT-003 (duplicate CaseFolder vs ff-encoding); refactor PA-STD-011 (regex.rs 993, engine.rs 838); TCR thin PA-TCR-002 (1 row/20 reqs); zero-log defensible |
| 19 | sequence-numbers | 1 | DONE | units/sequence-numbers.md | Tracking-COMPLETE (22/22 tasks); NOT a split candidate (14 reqs but single cohesive pipeline); undo is trait-decoupled -- CLEAN seam for PA-CONFLICT-002 (no new conflict); ASCII violation PA-STD-013 (60 non-ASCII bytes incl. box-drawing in .rs); size WATCH PA-STD-012 (number_cmd.rs 355 non-test); TCR thin PA-TCR-003 (3 rows/14 reqs); logging PA-LOG-006 (2 mandated WARNs Req 1.4/2.8 unemitted + CR-NR-058 dev-logging); crate is `ff-seqnum` not `ff-sequence-numbers` (doc drift) |
| 20 | syntax-highlighting | 1 | DONE | units/syntax-highlighting.md | Tracking-COMPLETE (185/185); engine fully implemented (21 files, 6 submodules); NOT a spec split (well-decomposed); FOLD-LEVEL BOUNDARY CONFIRMED CLEAN (owner=syntax-highlighting; display-line-mapping only consumes+stores collapsed-label; W1.4 PA-WATCH RESOLVED); PA-STD-014 highlight_engine.rs=462 non-test EXCEEDS 400 cap (REFACTOR); PA-STD-015 ASCII (17); TCR thin PA-TCR-004 (Req 16 only, 1-15 unlisted); PA-LOG-007 (Req 10.7 DEBUG unemitted + dev-logging); HILITE delegation WATCH (edit-ops parse -> syntax exec); GUI-independent verified |
| 21 | exclude-show-filter | 1 | DONE | units/exclude-show-filter.md | Tracking-COMPLETE (120/120); crate is `ff-exclude-show-filter` (not `ff-filter`); NOT a split candidate (10 reqs, cohesive); SHOW-restore boundary CONFIRMED (owns EXCLUDE/SHOW/RESET semantics; consumes CANONICAL DisplayLineMapping trait -- positive counter-example to PA-CONFLICT-001); PA-STD-016 exclusion_engine.rs=535 EXCEEDS 400 cap (REFACTOR); PA-STD-017 ASCII incl. non-ASCII in runtime error string; PA-TCR-005 TOTAL TCR absence (0 rows/10 reqs); PA-LOG-008 dead ff-logging dep -> dev-logging; PA-WATCH-010 exclusion+folding coexistence |
| 22 | whitespace-and-guides | 1 | DONE | units/whitespace-and-guides.md | Tracking-COMPLETE (100/100); NOT a split candidate; EXEMPLARY module decomposition (23 files, none near 400 cap -- model for the rule); display-line-mapping "Consumer" realized by PARAMETER injection (sub_line_count) not a crate dep -- CLEAN, GUI-independent verified; PA-STD-018 ASCII incl. em-dash in 7 runtime #[error] strings; PA-TCR-006 TOTAL TCR absence (0 rows/9 reqs); PA-LOG-009 dead ff-logging dep -> config-coercion WARN + dev-logging; wrap-gating WATCH (W1.14) |
| 23 | auto-indentation | 1 | DONE | units/auto-indentation.md | Tracking-COMPLETE (137/137) BUT PA-INCOMPLETE-006 (HIGH): Req 9.7 WARN commented out (patterns.rs:47) + Req 10.7 per-decision DEBUG absent -- MANDATED logging stubbed while tasks all [x]; clearest CR-NR-058 dev-logging-in-spec case; NOT a split candidate; no size violation; CLEAN seam for PA-CONFLICT-002 (returns IndentDecision data, caller wraps in EditorTransaction; deps = ff-logging+regex only); PA-STD-019 ASCII incl runtime error string; PA-TCR-007 thin (1 row/10 reqs) |
| 24 | line-wrap-toggle | 1 | DONE | units/line-wrap-toggle.md | Tracking-COMPLETE (189/189); crate is `ff-wrap` (not `ff-line-wrap-toggle`); NOT a split candidate; no size violation; PA-CONFLICT-004 (NEW, HIGH): WrapIndentMode + wrap-visual-flag DUPLICATED vs whitespace-guides, unbridged, enum-vs-bitfield shape divergence -- recommend whitespace-guides as owner; W1.12 wrap-inactive->no-markers gate CONFIRMED consistent (Wrap None -> height 1 -> sub_line_count<=1 guard); PA-LOG-010 (MED) 4 reqs mandate config warning "via logging-subsystem" but NO ff-logging dep; PA-STD-020 ASCII incl runtime warning/error strings; PA-TCR-008 thin (1 row/13 reqs); no coupling deps (all cross-refs caller-injected) |
| 25 | text-decorations | 1 | DONE | units/text-decorations.md | Tracking-COMPLETE (155/155); NOT a spec split (cohesive, 20 files); syntax-highlighting PEER boundary CONFIRMED clean (independent RunStyles storage -- unique to crate; matching under/layer-order contract; 0-7/8-31/32-43 namespace split, no slot collision); storage backing for find search-highlight + HILITE FIND (PA-WATCH-009); PA-STD-021 run_styles.rs=410 EXCEEDS 400 cap (REFACTOR); PA-STD-022 ASCII incl runtime #[error] strings; PA-TCR-009 TOTAL TCR absence (0 rows/15 reqs, 3rd in Wave1); PA-LOG-011 dead ff-logging dep + Req 15.8 mandated theme-WARN unimplemented |
| 26 | dataset-catalog | 2 | DONE | units/dataset-catalog.md | Tracking-COMPLETE (236/236); crate is `ff-dscatalog` (ff-dataset-catalog = legacy stub, PA-DEP-002); LARGEST unit (31 reqs/816 lines); STRONG SPLIT PA-SPLIT-008 (4-of-4: catalog-meta + ff-record-codec Req17.1-mandated + ff-record-store/vsam + infra); 9 files over 400 cap PA-STD-023 (sqlite_record.rs 843 worst); PA-STD-024 ellipsis in runtime menu labels; 49 fs calls LEGITIMATE (IS the VFS provider, NOT FFW-ARCH-001 violation); TCR EXCELLENT (99 rows -- exemplar); PA-LOG-012 (MED-HIGH) DEAD ff-logging on disk/DB/transactional subsystem, top Wave-2 logging target; ADR-001 boundaries -> PA-WATCH-011 (ff-dsalloc/ff-idcams delegation) |
| 27 | virtual-catalog-manager | 2 | DONE | units/virtual-catalog-manager.md | Tracking-COMPLETE (190/190); NO dedicated crate -- SHELL-UI in ff-desktop (POM option 1 Catalog Explorer + dialogs + registry); delegates mainframe CRUD to ff-dscatalog (ADR-001, consistent); PA-CONFLICT-005 (NEW, HIGH) duplicate+MISPLACED posix VfsProvider in ff-desktop AND ff-vfs (layering violation); PA-STD-025 SEVERE cap violations (files_panel.rs 1195, file_explorer_panel.rs 935 -- worst in analysis); PA-STD-026 BOM/mojibake in posix_provider.rs; PA-LOG-013 (MED) zero logging on destructive catalog/dataset ops; PA-WATCH-012 double catalog-persistence store ([virtual_catalogs] vs [catalog]); TCR moderate (18) |
| 28 | dataset-allocator | 2 | DONE | units/dataset-allocator.md | Tracking-COMPLETE (181/181); crate `ff-dsalloc` (spec says ff-dataset-allocator); NOT a split candidate; PA-WATCH-011 CRUD half RESOLVED CLEAN (CatalogProvider trait, no ff-dscatalog dep, MockCatalog only, delegates per ADR-001 Req 4); PA-CONFLICT-006 (NEW) own DSN validator duplicates ADR-001 catalog-owned validate_dsn; PA-STD-027 pipeline.rs 412 over cap; PA-STD-028 ASCII runtime strings; PA-LOG-014 (LOW) ff-logging spec-vs-impl drift (dep claimed, absent); PA-TCR-010 thin (1 row/16 reqs); ADR-001 Req5 ff-vsam-services reinforces PA-SPLIT-008 |
| 29 | dataset-ownership-model | 2 | DONE | units/dataset-ownership-model.md | Tracking-COMPLETE (59/59); GOVERNANCE/ADR-001 (not a crate); BEST-enforced spec -- real fitness function ff-governance-tests (Req 18) + CatalogService/DynCatalogService/VsamService trait contracts (Req 15/16); dep direction + trait delegation HONORED in code (W2.1-W2.3 align); PA-DEP-002 WITHDRAWN (ff-dataset-catalog = intentional shared-interface crate, not dead stub); PA-CONFLICT-007 (NEW) VSAM impl MISPLACED in ff-dscatalog storage/ while ADR-owner ff-vsam-services is trait-only stub -- subsumes PA-SPLIT-008 VSAM half as MIGRATION; PA-WATCH-013 compliance suite checks interface crate not impl (ff-dscatalog) |
| 30 | structure-catalog | 2 | DONE | units/structure-catalog.md | Tracking-COMPLETE (251/251); NOT a split candidate; no size violation; 0 fs (VFS); PA-CONFLICT-008 (NEW) own FieldDefinition/RecordStructure/FieldType model, NO dep on fileforge-integration (ff-forge has parallel DataType/FieldDefinition/RecordStructure) despite spec "extends fileforge-integration"; PA-LOG-015 (MED) dead ff-logging + 4 mandated WARN/INFO absent (Req 1.5/1.6/2.5/2.6); PA-STD-029 ASCII comment-only (211, no runtime defect); PA-TCR-011 TOTAL absence (0 rows/15 reqs); PA-WATCH-014 shared catalog.* config prefix vs dataset-catalog; ff-forge/ff-fileforge naming drift |
| 31 | record-selection-criteria | 2 | DONE | units/record-selection-criteria.md | Tracking-COMPLETE (158/158); crate `ff-select` (spec says ff-criteria); NOT a split candidate; BEST-sized Wave-2 (no file near cap); PA-CONFLICT-008 EXTENSION (3rd parallel field-type enum FieldDataType, no ff-forge dep, cannot delegate COMP-3 as spec requires); PA-WATCH-015 (MED) Criteria_Store spec'd config-system-managed but raw std::fs + no ff-config dep (config mediation bypassed); PA-LOG-016 (MED) dead sole-dep ff-logging + Req 9.9 WARN absent; PA-STD-030 ASCII runtime strings; PA-TCR-012 thin (1 row/14) |
| 32 | tabs-and-mask | 2 | DONE | units/tabs-and-mask.md | Tracking-COMPLETE (152/152); crate `ff-tabmask` (spec ff-tabs-and-mask; NOT the ff-tabs editor-tab crate = Wave 4); NOT a split (cohesive TABS+MASK pair); no size violation; 0 fs; clean minimal-dep + ConfigProvider/DocumentContext injection traits (3rd Wave-2 unit w/ this seam); PA-WATCH-016 (LOW) Display_Artifact_Line unified in glossary but REPLICATED in code (COLS/BNDS + Placeholder + TABS/MASK, no shared abstraction) -> Wave-4 consolidation; PA-LOG-017 (LOW) dead sole-dep ff-logging; PA-STD-031 ASCII runtime strings; PA-TCR-013 thin (1 row/18) |
| 33 | command-semantics | 3 | DONE | units/command-semantics.md | 173/181 tasks -- 8 OPEN; NOT a split; GOOD TCR (39 rows); PA-INCOMPLETE-007 (HIGH) Req 11 CR-NR-057 command chaining (Task 28, chain.rs) entirely UNBUILT -- honest [ ] tracking; pairs with PA-INCOMPLETE-003 (nav stack, cmd-framework Req 10) + menu-workspace fastpath (PA-WATCH-017); PA-WATCH-009 NARROWED (cmd-semantics NOT in HILITE chain, 0 refs); PA-STD-032 tso.rs 447 over cap; PA-LOG-018 (MED) command-pipeline dev-logging (mostly covered by PA-W0.2); 0 fs |
| 34 | command-completion | 3 | DONE | units/command-completion.md | Tracking-COMPLETE (191/191); crate `ff-completion` (spec ff-command-completion); NOT a split; CLEANEST cross-unit story W2-3 -- CompletionProvider trait injects all upstream (registry/VFS/macro/line-cmd/keyword), ZERO duplication, 0 fs (VFS caller-supplied, no FFW-ARCH-001 issue); PA-STD-033 engine.rs 478 over cap; PA-LOG-019 (LOW-MED) dead ff-logging + 3 mandated logs unimpl (notably Req 10.5 provider-failure catch); PA-STD-034 ASCII comment-only (60); PA-TCR-014 thin (1 row/10) |
| 35 | command-palette | 3 | DONE | units/command-palette.md | Tracking-COMPLETE (20/20); NO crate -- ff-desktop/src/command_palette/ modal overlay (reads ff-command registry); small/complete; clean size/ASCII/fs/logging; PA-CONFLICT-009 (NEW, LOW-MED) own fuzzy.rs duplicates command-completion matching/fuzzy.rs (2 fuzzy engines, diverged, no shared crate) -> extract shared ff-fuzzy; PA-TCR-015 (LOW) 0 rows/5 reqs; PA-LOG-020 (LOW) optional palette dev-logging (invocations covered by PA-W0.2) |
| 36 | command-configurator | 3 | DONE | units/command-configurator.md | 24/26 tasks -- 2 OPEN but pure BOOKKEEPING (PA-TRACK-003: 6.2 TCR + 6.3 project-master, NOT feature work); NO crate -- ff-desktop/src/command_config/; clean size/ASCII (Req 1.8 ASCII honoured); 17 TCR rows; external exec DELEGATED to shell-command (Req 19, W3.8), security via shell.mode -> PA-WATCH-018; raw std::fs commands.toml (menu-workspace pattern) -> PA-WATCH-019; PA-LOG-021 (LOW) store-event dev-logging; no split, no conflict |
| 37 | menu-workspace | 3 | DONE | units/menu-workspace.md | 75/90 tasks -- 15 OPEN; NO crate -- ff-desktop/src/menu_workspace/ + primary_option_menu.rs; NOT a split; REAL logging (4 calls, not dead dep); GOOD TCR (63); PA-INCOMPLETE-008 (HIGH) CR-NR-057 separator-aware Chained_Path (Task 21 + DF.1-5) UNBUILT -- 3rd leg of chaining bundle w/ PA-INCOMPLETE-007 (cmd-semantics Req 11) + PA-INCOMPLETE-003 (cmd-framework Req 10); PA-WATCH-017 shared split_chain now explicit task; reference for menus/ raw-TOML store family (PA-WATCH-019 CONFIRMED); PA-STD-035 primary_option_menu.rs 537 over cap; PA-LOG-022 dispatch dev-logging; PA-TRACK-004 bookkeeping |
| 38 | menu-and-statusbar | 3 | DONE | units/menu-and-statusbar.md | Tracking-COMPLETE (208/208); crate `ff-menu` (GUI-independent, no egui dep -- correct layering); NOT a size split; core (Reqs 1-11/13/16) complete; PA-WATCH-020 (MED) Reqs 17-19 (tab chrome/detach/ISPF split) SPEC SCOPE-CREEP -- no ff-menu impl, belong to Wave-4 layout-and-docking/multi-tab-editor -> relocate/cross-ref; menu-command routing CLEAN (Command_Target, consistent w/ menu family); PA-STD-036 ASCII runtime string; PA-LOG-023 (LOW) dead ff-logging + recent-files WARN; PA-TCR-016 thin (2 rows/16); recent-files raw-fs (PA-WATCH-019 family) |
| 39 | function-keys-and-history | 3 | DONE | units/function-keys-and-history.md | 247/271 tasks -- 24 OPEN; crate `ff-keys` (spec ff-function-keys); well-sized; GOOD TCR (33); PA-INCOMPLETE-009 (MED) Req 17 (END/RETURN-from-POM revision, CR-NR-057 nav-model adjacent) + Req 19 (RETRIEVE recall API + History_List overlay, partial -- LIST logic exists) + bookkeeping; honest tracking; PA-SPLIT-009 (LOW, deferred) function-keys vs command-history; PA-STD-037 ASCII runtime strings; PA-LOG-024 dead dep + error-typed history/config paths; PA-WATCH-021 command-field overlay consistency; history-TOML raw-store (PA-WATCH-019 family) |
| 40 | shell-command | 3 | DONE | units/shell-command.md | Tracking-COMPLETE (184/184); crate `ff-shell`; SECURITY-CRITICAL exec engine + POSITIVE exemplar; PA-WATCH-018 RESOLVED CLEAN -- shell.mode gate (disabled/prompt/enabled, safe prompt default, macro dual-gate) ENFORCED at engine entry before all 13 spawn sites; configurator inherits it; ACTUALLY LOGS (10 calls, not dead dep); adequate TCR (25); PA-STD-038 engine.rs 523 + emulator.rs 535 over cap; PA-SPLIT-010 (LOW) emulator split; PA-WATCH-022 TSO-verb disambiguation vs cmd-semantics; PA-LOG-025 audit gate-denial logging; PA-STD-039 ASCII runtime string |
| 41 | startup-and-session | 3 | DONE | units/startup-and-session.md | Tracking-COMPLETE (68/68); crate `ff-session` (564 lines/16 reqs); PA-SPLIT-011 (MED, owner-gated) session-state vs startup-lifecycle; PA-STD-040 session_state.rs 529 over cap; PA-WATCH-012 REFINED (clean 3-layer: ff-session tab descriptors / VCM registry / ff-dscatalog mounts -- confirm restore ordering); PA-LOG-026 (MED-HIGH) DEAD ff-logging on FAULT-TOLERANCE subsystem (crash-recovery + graceful-degradation log NOTHING) -- 2nd-highest logging target after ff-dscatalog; 37 fs LEGITIMATE (session persistence layer); PA-STD-041 ASCII runtime strings; PA-WATCH-023 persistence-vs-functionality scope (Reqs 13/14/19/20) |
| 42 | workspace-model | 3 | PENDING | -- | shell/commands/menus |
| 43 | layout-and-docking | 4 | PENDING | -- | UI/panels |
| 44 | multi-tab-editor | 4 | PENDING | -- | UI/panels |
| 45 | file-tree-panel | 4 | PENDING | -- | top split candidate (674) |
| 46 | theme-and-appearance | 4 | PENDING | -- | UI/panels |
| 47 | view-zoom | 4 | PENDING | -- | UI/panels |
| 48 | hex-display | 4 | PENDING | -- | UI/panels |
| 49 | notification-system | 4 | PENDING | -- | UI/panels |
| 50 | plugin-manager-ui | 4 | PENDING | -- | UI/panels |
| 51 | accessibility | 4 | PENDING | -- | UI/panels |
| 52 | context-help | 4 | PENDING | -- | UI/panels |
| 53 | clipboard-operations | 4 | PENDING | -- | UI/panels |
| 54 | idle-processing | 4 | PENDING | -- | UI/panels |
| 55 | large-file-performance | 4 | PENDING | -- | UI/panels |
| 56 | external-modification | 4 | PENDING | -- | UI/panels |
| 57 | file-operations | 4 | PENDING | -- | UI/panels |
| 58 | jes-emulator | 5 | PENDING | -- | split candidate (398) |
| 59 | idcams-emulator | 5 | PENDING | -- | split candidate (410) |
| 60 | jcl-resolver | 5 | PENDING | -- | stub -- no requirements yet |
| 61 | database-tool | 5 | PENDING | -- | split candidate (352) |
| 62 | compiler-toolchain-integration | 5 | PENDING | -- | tools |
| 63 | batch-execution | 5 | PENDING | -- | tools |
| 64 | asa-report-preview | 5 | PENDING | -- | tools |
| 65 | lua-macro-engine | 5 | PENDING | -- | tools; CR-NR-057 change |
| 66 | language-service | 5 | PENDING | -- | tools |
| 67 | custom-file-viewers | 5 | PENDING | -- | tools |
| 68 | compare-and-merge | 5 | PENDING | -- | tools |
| 69 | global-search | 5 | PENDING | -- | tools |
| 70 | connector-extensibility | 5 | PENDING | -- | connectors |
| 71 | connector-local-fs | 5 | PENDING | -- | connectors |
| 72 | connector-network-fs | 5 | PENDING | -- | connectors |
| 73 | connector-ftp-sftp | 5 | PENDING | -- | connectors |
| 74 | connector-cloud | 5 | PENDING | -- | connectors |
| 75 | connector-mainframe | 5 | PENDING | -- | connectors |
| 76 | fileforge-integration | 5 | PENDING | -- | integrations |
| 77 | automated-dialog-testing | 5 | PENDING | -- | test framework (FFTest) -- added to Wave 5 |
| 78 | bootstrap-scripts | 5 | PENDING | -- | dev tooling scripts -- added to Wave 5 |
| 79 | project-master | 6 | PENDING | -- | aggregate ordering (meta) |
| 80 | workbench-requirements-merge | 6 | PENDING | -- | analysis artifact -- review only |
| 81 | ears-integration | 6 | PENDING | -- | analysis artifact -- review only |

## Wave re-slot notes

- `automated-dialog-testing` and `bootstrap-scripts` were not in the original
  design wave lists; slotted into Wave 5 (tools) here. Recorded per Requirement 1
  and design section 3 (wave membership may be adjusted).
- Total subjects: 81 folders under `docs/specs/`, minus `project-analysis`
  (the analysis itself) = 80 analysed here; `project-master`,
  `workbench-requirements-merge`, and `ears-integration` are meta/review entries.
- **CR-NR-057 re-baseline**: restart triggered because CR-NR-057 modified
  foundational specs (command-framework Req 10 Context Navigation Stack,
  command-semantics, lua-macro-engine, menu-workspace). Prior pass superseded at
  commit `edb690d`.
