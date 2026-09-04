# MiniX/FTSO Reconciliation Report
# EI-0.2 Output -- Resolution of MiniX/FTSO Design Proposals

**Status:** EI-0 analysis -- no requirements.md files modified
**Input document:** `docs/source-documents/FileForgeWorkbench_MiniX_FTSO_Command_Environment_Design.md`
**Compared against:** TSO-EARS source files, existing committed specs

---

## Summary Verdict

The MiniX/FTSO design document is a useful architectural vision but was written
in isolation. The majority of its proposals are ALREADY COVERED by existing
committed specs or by the TSO-EARS ground truth. A small number of proposals
EXTEND existing specs in useful ways. Nothing in the document is genuinely
incompatible with the project direction -- the conflicts are naming and
structural overlaps, not contradictions of intent.

**The document must NOT be transcribed directly into requirements.md.**
All requirements must trace to EARS source files or existing committed specs.
The document's value is as an organisational framework and a source of
architectural principles -- not as a source of acceptance criteria.

---

## Resolution Table

Each row covers one section or proposal from the MiniX/FTSO document.

### Section 3 / Section 9.3 -- Command Dispatcher

| Field | Detail |
|-------|--------|
| Proposal | A central command dispatcher that parses, resolves, authorises, invokes, and reports commands from both the FTSO terminal and GUI panels |
| Existing coverage | `command-framework/requirements.md` Req 1-7 defines Command_Registry, Command_Dispatch (single entry point), metadata, enabled predicates, undo integration, scripting bridge, and history. This is a complete specification of the same concept. |
| EARS ground truth | TSO-2.1: command field accepts TSO-style commands. ISPF-1.5: all panels display COMMAND ===> field. These confirm the dispatcher must exist but do not add new structural requirements beyond what `command-framework` already specifies. |
| Resolution | **ALREADY COVERED.** The FTSO command dispatcher IS `ff-command`. No new sub-project. No new requirements. The `CommandDescriptor` struct sketch in section 9.4 is an illustrative duplicate of `command-framework` Req 3 (Command_Metadata). |

---

### Section 9.1 -- FTSO Terminal Presentation Layer

| Field | Detail |
|-------|--------|
| Proposal | An interactive FTSO terminal with ISPF Option 6-style panel, prompt ("FTSO READY"), command history, PF keys, scrollback, copy/paste, tabs/split sessions, async job notifications |
| Existing coverage | `shell-command/requirements.md` Req 7 defines interactive terminal mode (SHELL with no args), Terminal_Panel as DockablePanel, multiple terminal tabs, ANSI/VT100 emulation, PTY/ConPTY. Req 5 defines command history. Req 13 defines async execution and cancellation. |
| EARS ground truth | TSO-2: READY prompt and line mode. TSO-3: 24 PF keys. ISPF-1.5/1.6: COMMAND ===> and SCROLL ===> fields on all panels. SDSF-1: COMMAND INPUT ===> field. These define the UX that the terminal must present. |
| Resolution | **ALREADY COVERED (structure) + EXTENDS (ISPF Option 6 panel presentation).** The Terminal_Panel in `shell-command` covers the interactive terminal. The ISPF Option 6 panel chrome (title line, COMMAND ===> field, SCROLL ===> field, PF key bar, "FTSO READY" prompt style) is an EXTENSION to `shell-command` Req 7 that should be added when the FTSO panel is implemented. The "FTSO READY" prompt text and MAXCC return code display (section 13.2) are new UX details not yet in any spec -- these are EXTENDS items. |

---

### Section 9.2 -- Lexer and Parser

| Field | Detail |
|-------|--------|
| Proposal | Tokeniser for quoted dataset names, named options, subcommands, continuations, variable substitution, structured command invocation, syntax errors with source positions |
| Existing coverage | `command-framework/requirements.md` Req 2 defines Command_Dispatch with typed CommandParams. `command-semantics` crate handles ISPF primary command parsing. |
| EARS ground truth | TSO-CMD-1 through TSO-CMD-14 define specific command syntax (ALLOCATE operands, FREE operands, etc.). ISPF-1.5: COMMAND ===> field. TSO-2.1: command field accepts typed commands. |
| Resolution | **ALREADY COVERED (dispatch) + EXTENDS (TSO-style operand parsing).** The existing `command-semantics` crate handles ISPF-style commands. TSO-style operand syntax (positional operands, named keyword operands like DSNAME(), RECFM(), LRECL()) is a genuine extension needed for the TSO command set. This is an EI-5 batch item (Batch 10: command-semantics), not a new sub-project. The continuation character proposal (section 8.3, trailing backslash) is a new UX detail -- EXTENDS. |

---

### Section 9.4 -- Command Registry

| Field | Detail |
|-------|--------|
| Proposal | Metadata-driven command registration via CommandDescriptor struct (name, aliases, namespace, syntax, capabilities, execution mode, stream types, provider ID, compatibility tags) |
| Existing coverage | `command-framework/requirements.md` Req 1 (Command_Registry), Req 3 (Command_Metadata: display name, description, category, shortcut, icon, enabled predicate, visibility predicate). Req 1.7 supports deregistration (plugin unload). |
| EARS ground truth | No EARS file specifies registry internals -- this is an architectural concern. |
| Resolution | **ALREADY COVERED.** The `CommandDescriptor` in the design doc is a near-duplicate of `command-framework` Req 3. The additions (stream types, compatibility tags, provider ID) are useful for the TSO command set and should be added to `command-framework` Req 3 as extensions when the TSO command batches are gated. Not a new sub-project. |

---

### Section 9.5 -- MiniX Service Layer

| Field | Detail |
|-------|--------|
| Proposal | MiniX as a named service environment providing: Catalogue Service, Dataset Service, Member/Library Service, Record I/O Service, GDG Service, VSAM/ISAM Service, IDCAMS-like Utility Service, Job Entry Service, Spool Service, Security/Capability Service, Session Service, Script Service, Event/Notification Service, Audit Service |
| Existing coverage | These services map directly to existing crates: `ff-dscatalog` (catalogue), `ff-dsalloc` (dataset/member allocation), `ff-jes` (job entry, spool), `ff-vfs` (record I/O abstraction), `ff-idcams` (IDCAMS), `ff-lua` (scripting), `ff-workflow` (events/notifications), `ff-logging` (audit). Phase BS adds record codecs, StorageProvider, VSAM/ISAM, staged transactions. |
| EARS ground truth | TSO-CMD-1 through TSO-CMD-9 define catalogue/dataset commands. SDSF files define job/spool services. REXX-4 (EXECIO) defines record I/O. These confirm the services must exist but they are already being built. |
| Resolution | **ALREADY COVERED.** "MiniX" is a name for the integration contract between existing crates -- it is not a new crate or sub-project. The name should be used only as an internal architecture label in design docs (e.g., `dataset-catalog/design.md` may reference "the MiniX service layer" as a conceptual grouping). It must never appear in user-facing text or as a crate name. |

---

### Section 9.6 -- Host Command Adapter

| Field | Detail |
|-------|--------|
| Proposal | Native OS execution behind an explicit HOST command with double-hyphen boundary (HOST -- ls -la), policy controls (disabled by default, allowlists, working directory, env filtering, audit), visually distinct from FTSO output |
| Existing coverage | `shell-command/requirements.md` Req 1 defines SHELL/TSO alias. Req 2 defines Shell_Mode security control (disabled/prompt/enabled). Req 3 defines platform shell detection. Req 4 defines command execution mode. Req 11-12 define working directory and environment. Req 13 defines async execution and cancellation. |
| EARS ground truth | No TSO-EARS file defines a HOST command -- TSO/ISPF does not have one. The HOST concept is a FFWB-specific addition to provide OS access. |
| Resolution | **ALREADY COVERED.** HOST and SHELL/TSO are the same concept. The `shell-command` spec already covers this completely. The double-hyphen syntax (HOST -- cmd) is a cosmetic variant of SHELL cmd. Decision: retain SHELL/TSO as the command name (ISPF-compatible); do not add a separate HOST command. The `shell.mode` security control already provides the "disabled by default" behaviour the design doc requires. The visual distinction between FTSO and host output is an EXTENDS item for the Output_Panel (shell-command Req 15). |

---

### Section 10 -- Command Model (Categories A through J)

| Field | Detail |
|-------|--------|
| Proposal | Ten command categories: Session/help, Catalogue/dataset, Library/member, Record utilities, GDG, VSAM/IDCAMS, JES/spool, Script/program execution, Plugin commands, Host commands |
| Existing coverage | Categories A (session/help), B (catalogue/dataset), C (library/member), D (record utilities), E (GDG), F (VSAM/IDCAMS), G (JES/spool) all map to TSO-EARS requirements and existing crates. Category H (script execution) maps to `lua-macro-engine`. Category I (plugin commands) maps to `plugin-architecture`. Category J (host commands) maps to `shell-command`. |
| EARS ground truth | TSO-CMD-1 through TSO-CMD-14 define categories B/C/D/E/F/G commands with authoritative syntax and behaviour. REXX-1 through REXX-5 define category H. SDSF files define category G. |
| Resolution | **ALREADY COVERED (structure).** The category framework is a useful organisational tool for the EI-5 batches. The EARS files are the authoritative source for what each command in each category must do. The design doc's command examples (LISTCAT, ALLOC, SUBMIT, etc.) are illustrative -- the EARS files define the actual acceptance criteria. |

---

### Section 11 -- Dataset and Record Semantics

| Field | Detail |
|-------|--------|
| Proposal | Fixed-record semantics (exact logical length, no implicit CR/LF, padding/truncation policy), variable-record semantics (record descriptors, max length validation), dataset reference parsing (quoted/unquoted, member notation, GDG relative), URI-like scheme disambiguation (ds://, file://) |
| Existing coverage | Phase BS (dataset-catalog) defines record codecs (FixedCodec, VariableCodec, BinaryCodec, TextCodec), StorageProvider, VSAM/ISAM. `virtual-file-system` Req 9-12 define record-oriented storage. |
| EARS ground truth | TSO-CMD-1 (ALLOCATE) defines RECFM, LRECL, BLKSIZE, DSORG operands. REXX-4 (EXECIO) defines record-oriented read/write. SDSF-BROWSE files define record display. These confirm record semantics are required. |
| Resolution | **ALREADY COVERED (Phase BS).** The record semantics in the design doc align with and reinforce Phase BS requirements. The ds:// URI scheme is a new detail not yet in any spec -- this is an EXTENDS item for `virtual-file-system` or `command-semantics` when the TSO command batches are gated. |

---

### Section 12 -- Pipelines and Redirection

| Field | Detail |
|-------|--------|
| Proposal | Record-aware pipelines (text, binary, logical record, tabular, structured event streams), typed stream declarations, explicit conversion policy, typed redirection (> file://, > ds://) |
| Existing coverage | `command-framework` Req 2.8 defines typed CommandParams. `shell-command` Req 14 defines stdin piping from document. No existing spec defines record-aware inter-command pipelines. |
| EARS ground truth | No TSO-EARS file defines pipeline syntax -- TSO/ISPF does not have Unix-style pipes. REXX-4 (EXECIO) defines sequential record I/O but not pipelines. |
| Resolution | **GENUINELY NEW (but P3 -- deferred).** Record-aware pipelines are architecturally sound but not required by any EARS source file. This is a future capability. Add to `deferred-requirements.md` when that file is created in EI-4. Do not gate now. |

---

### Section 13 -- Completion Status and Diagnostics

| Field | Detail |
|-------|--------|
| Proposal | Structured CommandResult (outcome, return_code, reason_code, message_id, summary, diagnostics), MAXCC display (MAXCC=0, MAXCC=4, MAXCC=8, MAXCC=12, MAXCC=16), FFWB message identifiers (FTSO0001I, MINIX0312E) |
| Existing coverage | `command-framework` Req 2.6 defines CommandResult with success/failure and error propagation. `shell-command` Req 17 defines exit code reporting. |
| EARS ground truth | No TSO-EARS file defines MAXCC display -- this is a FFWB-specific UX choice inspired by mainframe conventions. |
| Resolution | **EXTENDS.** The MAXCC display convention and structured message identifiers (FTSO####X format) are new UX details not yet in any spec. These should be added to `command-framework` Req 2 (CommandResult) and `shell-command` Req 17 (exit code reporting) when the TSO command batches are gated. The message identifier scheme (FTSO prefix) is a useful convention -- record as a design decision. |

---

### Section 14 -- Sessions and Concurrency

| Field | Detail |
|-------|--------|
| Proposal | Per-session state (workspace, catalogue context, dataset prefix, env vars, command history, allocated handles, working directory, cancellation scope, output stream), SPLIT/SWAP workflow, session tabs |
| Existing coverage | `shell-command` Req 7.7 defines multiple terminal tabs. `startup-and-session` defines session state. `function-keys-and-history` Req 12 defines PF2=SPLIT, PF9=SWAP. |
| EARS ground truth | ISPF-3: split screen (PF2=SPLIT, PF9=SWAP). TSO-4: scrolling. TSO-3.1: PF key defaults including PF2=SPLIT, PF9=SWAP. |
| Resolution | **ALREADY COVERED (tabs/split) + EXTENDS (per-session dataset prefix).** The dataset prefix per session (SET PREFIX ALAN) is a new concept not yet in any spec -- this is an EXTENDS item for `command-semantics` or a new `ftso-session` sub-section. The SPLIT/SWAP behaviour is already in `function-keys-and-history`. |

---

### Section 15 -- Scripting Strategy

| Field | Detail |
|-------|--------|
| Proposal | Phase 1: FFCMD command files (sequential execution, variables, conditionals, return-code inspection). Phase 2: embedded scripting language (Lua). Phase 3: REXX/CLIST compatibility adapter. |
| Existing coverage | `lua-macro-engine/requirements.md` Req 1-10 defines the Lua scripting engine (MACRO/EXEC/RUN commands, event hooks, security modes, per-buffer state). This is Phase 2 already implemented. |
| EARS ground truth | REXX-1 through REXX-5 define REXX exec execution, host command environments, external functions (LISTDSI, OUTTRAP, SYSDSN, etc.), EXECIO, and data stack. These are Phase 3 requirements. |
| Resolution | **ALREADY COVERED (Phase 2 -- Lua) + EARS-DEFINED (Phase 3 -- REXX).** Phase 1 (FFCMD files) is a lightweight scripting format not yet in any spec -- this is an EXTENDS item for `lua-macro-engine` (simple command-file execution before full Lua). Phase 3 REXX requirements come from REXX-1 through REXX-5 in the EARS files, not from the design doc. The design doc's scripting strategy is a useful roadmap but the EARS files define what Phase 3 must actually do. |

---

### Section 16 -- Plugin Model

| Field | Detail |
|-------|--------|
| Proposal | Plugin command providers register commands, publish help/syntax metadata, declare capabilities/permissions, declare stream types, receive constrained execution context, emit output/progress/diagnostic events, honour cancellation, report structured completion, unregister on unload. Namespace conflict resolution (CORE:LISTCAT, IDCAMS:LISTCAT). |
| Existing coverage | `plugin-architecture/requirements.md` defines the plugin lifecycle (initialize, activate, deactivate, shutdown), capability declarations, and plugin registry. `command-framework` Req 1.3 supports plugin registration. Req 1.7 supports deregistration on shutdown. |
| EARS ground truth | No EARS file defines plugin internals. |
| Resolution | **ALREADY COVERED (lifecycle) + EXTENDS (namespace conflict resolution).** The namespace conflict resolution (CORE:LISTCAT vs IDCAMS:LISTCAT) is a new detail not yet in `command-framework` Req 1 -- this is an EXTENDS item. The rest of the plugin model is already covered by `plugin-architecture` and `command-framework`. |

---

### Section 17 -- Security and Governance

| Field | Detail |
|-------|--------|
| Proposal | Capability model (catalogue.read, dataset.write, job.submit, host.execute, etc.), host command controls (allowlists, working-directory restrictions, env filtering, time limits, process-tree termination, audit logging), secret handling (not written to history, redacted from logs), confirmation policies for destructive commands, audit events (timestamp, session, principal, command, resources, authorisation outcome, correlation ID) |
| Existing coverage | `shell-command` Req 2 defines Shell_Mode security control. Req 12 defines environment inheritance. Req 13 defines cancellation and process-tree termination. `command-framework` Req 3.4 defines enabled predicates. `lua-macro-engine` Req 7 defines security modes. |
| EARS ground truth | SDSF-SET-3 (SET CONFIRM) defines confirmation for destructive actions. SDSF-SET-13 (QUERY AUTH) defines capability display. These confirm security controls are required. |
| Resolution | **ALREADY COVERED (shell security, macro security) + EXTENDS (capability model, secret handling, audit events).** The capability model (fine-grained per-command capability declarations), secret operand handling (redaction from history/logs), and structured audit events are new details not yet in any spec. These are EXTENDS items for `command-framework` and `shell-command` when the TSO command batches are gated. The confirmation policy (SET CONFIRM) is an EARS-defined requirement (SDSF-SET-3) that goes into `FFW-JES`. |

---

### Section 18 -- Graphical Integration

| Field | Detail |
|-------|--------|
| Proposal | EDIT opens FFWB editor, BROWSE opens read-only view, SDSF opens spool monitor, LISTCAT output allows dataset open from context action, clicking a message navigates to source location, graphical operation can display equivalent FTSO command, command completion uses catalogue/workspace/plugin metadata |
| Existing coverage | `shell-command` Req 15.7 defines click-to-navigate in Output_Panel. `command-completion` spec defines completion. `FFW-JES` Req 9 defines Job Monitor panel. |
| EARS ground truth | ISPF-2: panel hierarchy and navigation. SDSF-4: main panel and MGRP. These confirm graphical integration is required. |
| Resolution | **ALREADY COVERED (navigation, completion, JES panel) + EXTENDS (show equivalent FTSO command for GUI operations).** The "show equivalent FTSO command" feature (a GUI action displays its command-line equivalent) is a new UX concept not yet in any spec -- this is an EXTENDS item, P2 priority. |

---

### Section 19 -- Proposed Rust Component Structure

| Field | Detail |
|-------|--------|
| Proposal | New crates: ffwb-command-model, ffwb-command-parser, ffwb-command-runtime, ffwb-ftso, ffwb-minix-services, ffwb-host-adapter, ffwb-script-runtime, ffwb-terminal-ui, ffwb-command-testkit |
| Existing coverage | These map to: `ff-command` (command-model + command-runtime), `ff-command-semantics` (command-parser), `ff-shell` (host-adapter + terminal-ui), `ff-lua` (script-runtime), existing test infrastructure (command-testkit). |
| EARS ground truth | No EARS file defines crate structure. |
| Resolution | **ALREADY COVERED.** The proposed crate structure duplicates the existing workspace. No new crates are needed. The `ffwb-ftso` crate concept maps to extending `ff-shell` with the ISPF Option 6 panel presentation. |

---

### Section 21 -- Functional Requirements (FTSO-FR-001 through FTSO-FR-064)

| Field | Detail |
|-------|--------|
| Proposal | 64 functional requirements across 7 groups: core shell (FR-001-008), command registration (FR-010-014), MiniX integration (FR-020-025), record integrity (FR-030-034), host integration (FR-040-044), scripting (FR-050-054), diagnostics/audit (FR-060-064) |
| Existing coverage | FR-001-008 (core shell): covered by `shell-command` Req 1-7 and `command-framework` Req 1-2. FR-010-014 (registration): covered by `command-framework` Req 1, 3. FR-020-025 (MiniX integration): covered by existing crates and Phase BS. FR-030-034 (record integrity): covered by Phase BS record codecs. FR-040-044 (host integration): covered by `shell-command` Req 1-4. FR-050-054 (scripting): covered by `lua-macro-engine` Req 5. FR-060-064 (diagnostics/audit): partially covered by `command-framework` Req 7 and `shell-command` Req 17. |
| EARS ground truth | The TSO-EARS files provide the authoritative behavioural ground truth for the commands these requirements govern. |
| Resolution | **ALREADY COVERED (majority) + EXTENDS (audit events, secret handling, MAXCC display, stream type declarations).** The 64 FRs are not new requirements -- they are a restatement of existing specs plus a small number of extensions. The extensions are identified in the rows above. Do not transcribe these FRs into requirements.md. Use the EARS files as the source for new criteria. |

---

### Section 22 -- Non-Functional Requirements (FTSO-NFR-001 through FTSO-NFR-012)

| Field | Detail |
|-------|--------|
| Proposal | Cross-platform consistency, UI responsiveness during long commands, streamable output, unit-testable without terminal UI, fuzz-testable parser, isolated host adapter, versioned descriptors, plugin failure isolation, large result set paging, keyboard-only operation, consistent terminology, British/South African English |
| Existing coverage | Cross-platform: project-wide requirement. UI responsiveness: `shell-command` Req 13, `FFW-JES` Req 15. Testability: project-wide TDD rule. Keyboard-only: `menu-and-statusbar` Req 16. |
| EARS ground truth | No EARS file defines NFRs at this level. |
| Resolution | **ALREADY COVERED (majority).** The British/South African English requirement (NFR-012) is already the project convention. The fuzz-testable parser (NFR-005) is a useful addition to `command-framework` or `command-semantics` testing requirements -- EXTENDS. Plugin failure isolation (NFR-008) is already in `plugin-architecture`. |

---

### Section 24 -- Implementation Roadmap (Phases 1-6)

| Field | Detail |
|-------|--------|
| Proposal | 6 implementation phases: command foundation, dataset/catalogue commands, jobs/spool, scripting/pipelines, host/plugin extensions, compatibility/intelligent tooling |
| Existing coverage | This maps directly to the EI-5 batch sequence already defined in the workflow. |
| Resolution | **ALREADY COVERED.** The roadmap phases align with EI-5 batches 10-16. Use the EI-5 batch sequence, not the design doc phases. |

---

### Section 27 -- Open Design Decisions (14 ADRs)

| Field | Detail |
|-------|--------|
| Proposal | 14 open decisions: MiniX/FTSO names, grammar/continuation, dataset prefix rules, completion-code model, initial command set, pipeline stream types, scripting language, plugin ABI, host execution policy, history persistence, audit schema, terminal widget, ISPF PF-key emulation extent, SDSF as command vs panel launcher |
| Resolution | Decisions resolved by this document: (1) MiniX = internal label only; (2) FTSO = internal label for the ISPF Option 6 panel, not a separate product; (3) grammar/continuation = extend `command-semantics`; (4) dataset prefix = extend `command-semantics` session context; (5) completion-code model = MAXCC convention, extend `command-framework` Req 2; (6) initial command set = TSO-EARS P1 commands; (7) scripting language = Lua (already decided); (8) plugin ABI = existing `plugin-architecture`; (9) host execution policy = existing `shell-command` Req 2; (10) history persistence = existing `command-framework` Req 7; (11) audit schema = EXTENDS item; (12) terminal widget = extend `shell-command` Terminal_Panel; (13) ISPF PF-key emulation = existing `function-keys-and-history`; (14) SDSF = panel launcher (existing `FFW-JES` Req 9). |

---

## Summary of Resolutions

| Resolution | Count | Items |
|------------|-------|-------|
| ALREADY COVERED | 12 | Command dispatcher, MiniX service layer, command registry, host adapter, command categories, dataset/record semantics (Phase BS), plugin lifecycle, shell security, macro security, graphical integration (navigation/completion), crate structure, implementation roadmap |
| EXTENDS | 9 | ISPF Option 6 panel chrome + MAXCC display, TSO-style operand parsing, continuation character, ds:// URI scheme, dataset prefix per session, FFCMD command files, namespace conflict resolution, capability model + secret handling + audit events, "show equivalent command" UX |
| EARS-DEFINED | 3 | REXX scripting (REXX-1 through REXX-5), SDSF confirmation (SDSF-SET-3), SDSF capability display (SDSF-SET-13) |
| GENUINELY NEW (deferred) | 1 | Record-aware inter-command pipelines (P3) |
| CONFLICTS | 0 | None -- no proposals contradict existing specs |

---

## Decisions Recorded

**D1 -- MiniX naming:** "MiniX" is used as an internal architecture label in design
documents only. It never appears in user-facing text, crate names, or
requirements.md files. No `minix-services` sub-project is created.

**D2 -- FTSO naming:** "FTSO" is used as an internal label for the ISPF Option 6
panel presentation layer. It is not a separate product, crate, or sub-project.
The implementation extends `shell-command` (Terminal_Panel) and `command-semantics`
(TSO command set). User-facing text uses "Command Shell" or "ISPF Command Shell".

**D3 -- No new sub-projects from this document:** All proposals map to existing
sub-projects. The EXTENDS items are added to existing specs during EI-5 batches.

**D4 -- EARS files are authoritative:** For every command the design doc mentions
(LISTCAT, ALLOC, SUBMIT, etc.), the TSO-EARS files define the acceptance criteria.
The design doc's command examples are illustrative only.

**D5 -- HOST command not added:** The HOST concept is already covered by
`shell-command` SHELL/TSO with `shell.mode` security control. No separate HOST
command is introduced.

**D6 -- Record-aware pipelines deferred:** No EARS file requires inter-command
pipelines. This is a P3 item for `deferred-requirements.md`.
