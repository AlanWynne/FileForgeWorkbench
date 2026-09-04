# FileForgeWorkbench MiniX and FTSO Command Environment Design

**Document type:** Architecture and requirements specification  
**Project:** FileForgeWorkbench  
**Status:** Proposed  
**Version:** 0.1.0  
**Date:** 2026-09-03  

---

## 1. Purpose

This document defines a proposed interactive command environment for FileForgeWorkbench (FFWB), inspired by the role of the ISPF Option 6 Command Shell in a traditional mainframe working environment.

The proposed design consists of two related but separately defined components:

1. **MiniX**, a portable service environment that supplies simulated mainframe-style facilities such as catalogues, datasets, GDGs, VSAM structures, job execution, and security services.
2. **FTSO**, the FileForge Time Sharing Option, which provides an interactive command shell through which users, scripts, plugins, and FFWB panels can invoke those facilities.

The intention is not to reproduce z/OS, TSO/E, or ISPF internally. The intention is to provide a recognisable mainframe-oriented command model over FileForgeWorkbench's own cross-platform services.

---

## 2. Background

ISPF Option 6 provides a dedicated command shell from which a user can enter TSO commands and run command procedures such as CLISTs and REXX execs. Mainframe users commonly use this type of environment to allocate and inspect datasets, invoke utilities, submit jobs, and perform tasks that are not represented by a dedicated panel.

FileForgeWorkbench requires a comparable point of integration for:

- virtual mainframe datasets;
- fixed and variable record formats;
- PDS and PDSE-like libraries;
- GDGs;
- VSAM and ISAM emulation;
- IDCAMS-like services;
- JES and SDSF-style facilities;
- plugins and developer tooling;
- automation and scripting;
- host operating-system commands; and
- future ERI and AI-assisted capabilities.

Embedding Bash, PowerShell, or another native shell would provide useful host access, but would not supply the dataset semantics or mainframe conceptual model required by FileForgeWorkbench. A dedicated command framework is therefore recommended.

---

## 3. Design Decision

FileForgeWorkbench should implement an extensible command environment with the following logical relationship:

```text
+-----------------------------------------------------------+
|                    FileForgeWorkbench                     |
|                                                           |
|  +----------------------+    +-------------------------+  |
|  | FTSO Command Shell   |    | Graphical FFWB Panels   |  |
|  +----------+-----------+    +------------+------------+  |
|             |                             |               |
|             +--------------+--------------+               |
|                            |                              |
|                  +---------v----------+                   |
|                  | Command Dispatcher |                   |
|                  +---------+----------+                   |
|                            |                              |
|          +-----------------+------------------+           |
|          |                 |                  |           |
|  +-------v------+  +-------v-------+  +-------v--------+  |
|  | MiniX       |  | Plugin        |  | Host Command   |  |
|  | Services    |  | Commands      |  | Adapter        |  |
|  +-------+------+  +---------------+  +----------------+  |
|          |                                                |
|  +-------v---------------------------------------------+  |
|  | VFS, catalogue, datasets, JES, VSAM, GDG, security |  |
|  +-----------------------------------------------------+  |
+-----------------------------------------------------------+
```

The FTSO shell shall be the mainframe-oriented interactive interface. MiniX shall provide the underlying portable services. Host operating-system access shall be available only through an explicit and controlled bridge.

---

## 4. Goals

The design shall:

1. Provide an ISPF Option 6-like command experience without claiming binary or behavioural equivalence with TSO/E.
2. Make FileForgeWorkbench dataset and catalogue services available as commands.
3. provide a common command API for terminal sessions, graphical panels, scripts, plugins, and automated tests.
4. Preserve mainframe concepts such as dataset names, members, record formats, GDG generations, job submission, and catalogue operations.
5. Remain portable across Windows, Linux, and other platforms supported by FileForgeWorkbench.
6. Allow commands to be added without modifying the central shell implementation.
7. Support interactive and non-interactive execution.
8. Provide controlled host-shell integration without confusing native paths and virtual dataset names.
9. Support command history, structured diagnostics, cancellation, and session isolation.
10. Establish a foundation for later CLIST, REXX, RISL, PSL, AI, and ERI integration.

---

## 5. Non-goals

The initial implementation shall not attempt to:

- emulate the z/OS kernel;
- reproduce all TSO/E commands or ISPF services;
- provide 3270 data-stream compatibility;
- execute native z/OS load modules;
- reproduce RACF internally;
- reproduce every message identifier or return code issued by IBM products;
- expose unrestricted Bash, PowerShell, or operating-system execution by default;
- treat line-oriented host files as equivalent to FB or VB datasets; or
- promise compatibility with proprietary implementation details.

Compatibility claims shall be made only for commands and behaviours explicitly covered by FileForgeWorkbench tests and documentation.

---

## 6. Terminology

| Term | Meaning in this design |
|---|---|
| **FFWB** | FileForgeWorkbench. |
| **MiniX** | The proposed portable runtime and service environment behind FTSO. This name is provisional and must not imply that FFWB embeds or derives from the MINIX operating system. |
| **FTSO** | FileForge Time Sharing Option, the proposed FFWB command environment. |
| **Command provider** | A built-in module or plugin that registers one or more commands. |
| **Command dispatcher** | The service that parses, resolves, authorises, invokes, and reports the result of a command. |
| **VFS** | The FileForgeWorkbench virtual filesystem and dataset abstraction. |
| **Dataset reference** | A mainframe-style dataset name, optionally including a member or GDG relative generation. |
| **Host path** | A native Windows, Linux, or other operating-system filesystem path. |
| **Job** | A unit of work submitted to the MiniX job execution service. |
| **Session** | An isolated interactive or non-interactive FTSO execution context. |

---

## 7. Naming Considerations

### 7.1 MiniX

`MiniX` is a useful working name because it suggests a compact execution environment. However, `MINIX` is also the name of an existing operating system. Before the name is used publicly, the project should perform a naming and legal review.

Possible alternatives include:

- FileForge Runtime Environment (`FFRE`);
- FileForge Mainframe Services (`FFMS`);
- FileForge Virtual Operating Environment (`FFVOE`); or
- ForgeOS Services.

For the remainder of this document, **MiniX** is retained as the provisional architecture name.

### 7.2 FTSO

`FTSO` is intended to communicate the relationship to the TSO style of interaction while remaining explicitly FileForge-specific. The user-facing documentation shall state that FTSO is inspired by mainframe command environments and is not IBM TSO/E.

---

## 8. User Experience

### 8.1 Default terminal

The terminal shall open with an FTSO prompt rather than a native operating-system prompt.

```text
FTSO READY
FTSO>
```

A session may optionally use an ISPF Option 6-inspired panel:

```text
------------------- FILEFORGE COMMAND SHELL -------------------
Command ===>

Enter an FTSO command, script, utility, or HELP command.

PF1=Help  PF3=Exit  PF7=Backward  PF8=Forward  PF10=Actions
```

The terminal and panel presentations shall both use the same command dispatcher.

### 8.2 Example interaction

```text
FTSO> LISTCAT LEVEL(ALAN)
ALAN.COBOL
ALAN.JCL
ALAN.REXX
ALAN.TEST.DATA
MAXCC=0

FTSO> EDIT 'ALAN.COBOL(PROG001)'
FFWB EDIT SESSION OPENED
MAXCC=0

FTSO> SUBMIT 'ALAN.JCL(TESTJOB)'
JOB00042 SUBMITTED
MAXCC=0

FTSO> JES STATUS JOB00042
JOB00042 EXECUTING CLASS=A
MAXCC=0
```

The exact syntax and output above are proposed examples, not assertions of complete TSO compatibility.

### 8.3 Command continuation

The shell should support long commands using an explicit continuation character. A proposed default is a trailing backslash:

```text
FTSO> ALLOC DSNAME('ALAN.TEST.DATA') \
 ...> RECFM(FB) LRECL(80) \
 ...> SPACE(TRACKS,(5,2))
```

The continuation convention shall be configurable if future compatibility profiles require an alternative.

### 8.4 Command history

The shell shall maintain per-user and per-workspace command history, subject to security controls. It should support:

```text
HISTORY
HISTORY 20
RECALL 12
```

Commands containing secrets shall be excluded or redacted according to command metadata and policy.

---

## 9. Functional Architecture

### 9.1 FTSO terminal presentation layer

Responsibilities:

- render prompts, command output, warnings, progress, and diagnostics;
- provide keyboard handling and command history;
- support copy, paste, selection, search, and scrollback;
- support tabs or split sessions;
- map configurable PF keys to actions;
- display asynchronous job and command notifications; and
- render plain text while allowing future structured views.

The terminal presentation layer shall not contain command-specific business logic.

### 9.2 Lexer and parser

Responsibilities:

- tokenise command text;
- preserve quoted dataset names and operands;
- identify commands, subcommands, positional operands, and named options;
- process continuations;
- perform variable substitution when enabled;
- create a structured command invocation; and
- report syntax errors with source positions.

The parser shall preserve the original command text for audit and diagnostics, subject to secret redaction.

### 9.3 Command dispatcher

Responsibilities:

1. resolve aliases and compatibility profiles;
2. identify the command provider;
3. validate syntax and typed operands;
4. authorise the operation;
5. establish the execution context;
6. invoke the command handler;
7. stream output and progress events;
8. process cancellation;
9. normalise completion status; and
10. record audit events where policy requires them.

### 9.4 Command registry

Every command shall be registered using metadata rather than a hard-coded dispatcher branch.

Suggested metadata:

```rust
pub struct CommandDescriptor {
    pub name: String,
    pub aliases: Vec<String>,
    pub namespace: String,
    pub summary: String,
    pub syntax: Vec<CommandSyntax>,
    pub required_capabilities: Vec<Capability>,
    pub execution_mode: ExecutionMode,
    pub input_type: StreamType,
    pub output_type: StreamType,
    pub provider_id: String,
    pub compatibility_tags: Vec<String>,
}
```

This is an illustrative interface. Its final form shall follow the established Rust conventions and plugin ABI of FileForgeWorkbench.

### 9.5 MiniX service layer

MiniX shall expose service interfaces rather than terminal-specific implementations.

Proposed services:

```text
MiniX
├── Catalogue Service
├── Dataset Service
├── Member and Library Service
├── Record I/O Service
├── GDG Service
├── VSAM/ISAM Service
├── IDCAMS-like Utility Service
├── Job Entry Service
├── Spool Service
├── Security and Capability Service
├── Session Service
├── Script Service
├── Event and Notification Service
└── Audit Service
```

Graphical panels, FTSO commands, scripts, and plugins shall call the same service contracts. This avoids implementing an operation separately for the terminal and GUI.

### 9.6 Host command adapter

Native operating-system execution shall reside behind a separate adapter. It shall not be part of the core dataset service and shall require explicit invocation.

Proposed syntax:

```text
HOST -- ls -la
HOST -- powershell -NoProfile -Command Get-Location
```

The double hyphen marks the boundary between FTSO operands and the native command. The adapter shall pass an argument vector to the operating system where possible, rather than constructing an unescaped command string.

---

## 10. Command Model

### 10.1 Command categories

#### A. Session and help commands

```text
HELP
HELP ALLOC
SESSION NEW
SESSION LIST
SESSION SWITCH 2
SESSION CLOSE 2
HISTORY
CLEAR
EXIT
```

#### B. Catalogue and dataset commands

```text
LISTCAT
ALLOC
FREE
DELETE
RENAME
COPY
MOVE
BROWSE
VIEW
EDIT
INFO
```

#### C. Library and member commands

```text
MEMBER LIST 'ALAN.COBOL'
MEMBER CREATE 'ALAN.COBOL(PROG001)'
MEMBER DELETE 'ALAN.COBOL(PROG001)'
MEMBER RENAME 'ALAN.COBOL(PROG001)' TO(PROG002)
```

#### D. Record-oriented utilities

```text
SORT
SEARCH
COMPARE
GENERATE
REPRO
PRINT
```

These utilities shall consume record-aware streams. They shall not implicitly insert or remove CR, LF, or CRLF record terminators when operating on FB, VB, F, or V dataset organisations.

#### E. GDG commands

```text
GDG DEFINE BASE('ALAN.BACKUP') LIMIT(10)
GDG LIST BASE('ALAN.BACKUP')
ALLOC DSNAME('ALAN.BACKUP(+1)') RECFM(VB) LRECL(4096)
DELETE 'ALAN.BACKUP(-5)'
```

#### F. VSAM and IDCAMS-like commands

```text
IDCAMS
DEFINE CLUSTER
DELETE CLUSTER
LISTCAT ENTRIES
REPRO INDATASET(...) OUTDATASET(...)
VERIFY DATASET(...)
```

A dedicated IDCAMS compatibility command may delegate to the same typed service operations exposed through native FTSO commands.

#### G. JES and spool commands

```text
SUBMIT 'ALAN.JCL(TESTJOB)'
JES STATUS JOB00042
JES CANCEL JOB00042
JES HOLD JOB00042
JES RELEASE JOB00042
SDSF
SPOOL LIST JOB00042
SPOOL VIEW JOB00042 DDNAME(SYSPRINT)
```

`SDSF` may open a graphical or terminal-mode spool browser rather than behaving as a simple text command.

#### H. Script and program execution

```text
EXEC 'ALAN.FTSO.CLIST(BUILD)'
RUN script.ffcmd
REXX 'ALAN.REXX(TEST)'
```

Script engines shall be implemented as providers over the command and service contracts.

#### I. Plugin commands

Namespaced command examples:

```text
GIT STATUS
SQL QUERY --file query.sql
AI EXPLAIN DATASET('ALAN.COBOL(PROG001)')
ERI TRACE REQUIREMENT(REQ-001)
```

The examples identify potential extension points. They do not require the initial FTSO release to include Git, SQL, AI, or ERI providers.

#### J. Host commands

```text
HOST -- cmd.exe /c dir
HOST -- powershell -NoProfile -File build.ps1
HOST -- bash -lc "cargo test"
```

Host commands shall be disabled, restricted, or sandboxed according to workspace policy.

---

## 11. Dataset and Record Semantics

FTSO commands shall use the FileForgeWorkbench VFS and record I/O abstractions. Commands shall not assume that datasets are ordinary line-delimited files.

### 11.1 Fixed records

For fixed-record datasets:

- each logical record has an exact logical length;
- record boundaries are determined by metadata and record length;
- CR, LF, and CRLF are data unless a conversion operation explicitly interprets them;
- short input records require a declared padding policy; and
- long input records require an explicit reject, truncate, or wrap policy.

### 11.2 Variable records

For variable-record datasets:

- record boundaries are represented through the VFS record model;
- storage adapters may use record descriptors or equivalent internal metadata;
- logical record data shall be kept separate from storage framing;
- maximum record length shall be validated; and
- import and export conversion shall be explicit.

### 11.3 Dataset references

The parser should recognise:

```text
'ALAN.TEST.DATA'
'ALAN.COBOL(PROG001)'
'ALAN.BACKUP(+1)'
ALAN.TEST.DATA
```

Quoted names shall be treated as fully qualified. Resolution rules for unquoted names shall be defined by the active compatibility profile and session prefix.

### 11.4 Host paths versus dataset names

Ambiguity shall be prevented with explicit URI-like schemes or typed options when necessary:

```text
ds://'ALAN.TEST.DATA'
file:///home/alan/test.dat
file:///C:/work/test.dat
```

Commands may provide concise forms when the operand type is unambiguous, but internal APIs shall use typed resource references.

---

## 12. Pipelines and Redirection

Pipelines are useful but must be record-aware.

Proposed examples:

```text
LISTCAT LEVEL(ALAN) | FILTER "COBOL"
READ 'ALAN.INPUT' | SORT KEY(1,10,ASC) | WRITE 'ALAN.OUTPUT'
```

The pipeline implementation shall distinguish at least:

- text streams;
- binary byte streams;
- logical record streams;
- tabular streams; and
- structured event streams.

Commands shall declare accepted input and produced output types. The dispatcher shall reject incompatible pipelines or insert an explicit, user-approved converter.

Redirection should use typed destinations:

```text
LISTCAT LEVEL(ALAN) > file://catalogue.txt
LISTCAT LEVEL(ALAN) > ds://'ALAN.REPORT'
```

Implicit conversion between a text stream and an FB or VB dataset shall not occur without a defined conversion policy.

---

## 13. Completion Status and Diagnostics

### 13.1 Return model

Commands shall return a structured result, not only printed text.

Suggested fields:

```rust
pub struct CommandResult {
    pub outcome: CommandOutcome,
    pub return_code: i32,
    pub reason_code: Option<i32>,
    pub message_id: Option<String>,
    pub summary: String,
    pub diagnostics: Vec<Diagnostic>,
}
```

### 13.2 Mainframe-style status

For interactive familiarity, successful commands may display:

```text
MAXCC=0
```

Warnings and failures may use configurable return-code conventions:

```text
MAXCC=4
MAXCC=8
MAXCC=12
MAXCC=16
```

The mapping shall be documented and shall not imply that every FTSO return code has an identical TSO or utility meaning.

### 13.3 Messages

FFWB-specific messages should use a stable identifier convention, for example:

```text
FTSO0001I COMMAND COMPLETED
FTSO0204E DATASET NOT FOUND: ALAN.TEST.DATA
MINIX0312E JOB SERVICE UNAVAILABLE
```

Messages shall include:

- stable message identifier;
- severity;
- concise user message;
- optional explanation;
- suggested corrective action where known; and
- structured context for logs and tests.

---

## 14. Sessions and Concurrency

Each FTSO session shall have its own:

- current workspace;
- active catalogue context;
- dataset prefix;
- environment variables;
- command history;
- allocated resource handles;
- working host directory, if host access is enabled;
- cancellation scope; and
- output stream.

Sessions should support tabs and an ISPF-inspired split/swap workflow:

```text
SPLIT
SWAP
SESSION LIST
SESSION SWITCH 2
```

A command running in one session shall not block interaction with other sessions, provided that the underlying resources permit concurrent access.

Dataset locking and member-level locking shall be provided by MiniX services rather than implemented independently by the terminal.

---

## 15. Scripting Strategy

### 15.1 Phase 1: FFWB command scripts

The first scripting format should be a simple FTSO command file:

```text
/* BUILD.FFCMD */
ALLOC DSNAME('ALAN.BUILD.OUT') RECFM(FB) LRECL(133)
SUBMIT 'ALAN.JCL(BUILD)'
JES WAIT LAST TIMEOUT(300)
```

The initial language should support:

- sequential command execution;
- variables;
- comments;
- command return-code inspection;
- conditional execution;
- controlled failure handling; and
- script parameters.

### 15.2 Phase 2: embedded scripting provider

A lightweight embeddable language may later expose the command API and MiniX services. The choice should be based on:

- Rust integration quality;
- sandboxing;
- portability;
- debugging support;
- startup cost;
- plugin compatibility; and
- licensing.

### 15.3 Phase 3: compatibility scripting

Optional REXX or CLIST-inspired support may be introduced through an adapter. Compatibility shall be defined feature by feature and tested. The architecture shall not require native REXX support for the core command shell to operate.

### 15.4 RISL and PSL integration

Future RISL or PSL execution should use the same command and service contracts. This would permit requirements and intent models to invoke governed, testable operations without embedding terminal text parsing into the ERI layer.

---

## 16. Plugin Model

A plugin command provider shall be able to:

1. register commands and aliases;
2. publish help and syntax metadata;
3. declare capabilities and permissions;
4. declare supported input and output stream types;
5. receive a constrained execution context;
6. emit output, progress, and diagnostic events;
7. honour cancellation;
8. report structured completion status; and
9. unregister commands during controlled unload.

Command conflicts shall be resolved using namespaces and explicit precedence rules.

Example:

```text
CORE:LISTCAT
IDCAMS:LISTCAT
PLUGINX:LISTCAT
```

The user may omit the namespace when only one unambiguous command is active.

---

## 17. Security and Governance

The command environment is a privileged integration point and shall be designed accordingly.

### 17.1 Capability model

Commands shall declare capabilities such as:

```text
catalogue.read
dataset.read
dataset.write
dataset.delete
job.submit
job.cancel
spool.read
host.execute
plugin.manage
script.execute
```

The dispatcher shall evaluate capability grants before invoking a handler.

### 17.2 Host command controls

The host bridge shall support policy controls including:

- disabled by default in restricted workspaces;
- per-workspace enablement;
- executable allowlists or denylists;
- working-directory restrictions;
- environment-variable filtering;
- argument validation;
- execution time limits;
- process-tree termination;
- network restrictions where the platform permits them;
- output and resource limits; and
- audit logging.

The terminal shall visibly distinguish host execution from FTSO command execution.

### 17.3 Secret handling

Commands and scripts shall support secret-valued arguments that:

- are not written to history;
- are redacted from logs;
- are not shown in process diagnostics where avoidable; and
- can be sourced from an approved secret provider rather than plain text.

### 17.4 Confirmation policies

Destructive commands may require confirmation in interactive mode:

```text
DELETE 'ALAN.PROD.DATA'
```

Automation shall use explicit flags or policy-approved non-interactive execution rather than responding to interactive prompts.

### 17.5 Audit

Audit events should include:

- timestamp;
- session and workspace identifiers;
- authenticated principal;
- command identifier and provider;
- redacted invocation;
- affected resources;
- authorisation outcome;
- completion status; and
- correlation identifier.

Audit retention and export shall be controlled by project or organisational policy.

---

## 18. Graphical Integration

FTSO shall integrate with the rest of FileForgeWorkbench rather than operate as an isolated terminal widget.

Examples:

- `EDIT` opens the normal FFWB editor.
- `BROWSE` opens a read-only dataset view.
- `SDSF` opens the spool and job monitor.
- `LISTCAT` output allows a dataset to be opened from a context action.
- clicking a message with a source location navigates to the corresponding file, member, or record.
- a graphical operation can display or copy its equivalent FTSO command.
- command completion can use the catalogue, active workspace, plugins, and command metadata.

This provides an ISPF-like relationship between panels and commands while preserving a modern desktop experience.

---

## 19. Proposed Rust Component Structure

```text
crates/
├── ffwb-command-model/
│   ├── descriptors
│   ├── invocation
│   ├── result
│   ├── diagnostics
│   └── stream-types
├── ffwb-command-parser/
│   ├── lexer
│   ├── parser
│   ├── completion
│   └── formatting
├── ffwb-command-runtime/
│   ├── registry
│   ├── dispatcher
│   ├── authorisation
│   ├── pipelines
│   └── cancellation
├── ffwb-ftso/
│   ├── core-commands
│   ├── session
│   ├── history
│   └── compatibility
├── ffwb-minix-services/
│   ├── catalogue
│   ├── datasets
│   ├── gdg
│   ├── vsam
│   ├── jobs
│   ├── spool
│   └── security
├── ffwb-host-adapter/
├── ffwb-script-runtime/
├── ffwb-terminal-ui/
└── ffwb-command-testkit/
```

This is a proposed logical decomposition. It may be adjusted to match the actual FileForgeWorkbench workspace and plugin boundaries.

---

## 20. Core Interface Sketches

### 20.1 Command handler

```rust
#[async_trait]
pub trait CommandHandler: Send + Sync {
    fn descriptor(&self) -> &CommandDescriptor;

    async fn execute(
        &self,
        context: CommandContext,
        invocation: CommandInvocation,
        input: CommandInput,
        output: CommandOutput,
        cancellation: CancellationToken,
    ) -> CommandResult;
}
```

### 20.2 Execution context

```rust
pub struct CommandContext {
    pub session_id: SessionId,
    pub workspace_id: WorkspaceId,
    pub principal: Principal,
    pub dataset_prefix: Option<DatasetName>,
    pub current_host_directory: Option<PathBuf>,
    pub services: MiniXServices,
    pub capabilities: CapabilitySet,
    pub correlation_id: CorrelationId,
}
```

### 20.3 Typed resource reference

```rust
pub enum ResourceReference {
    Dataset(DatasetReference),
    HostFile(PathBuf),
    SpoolFile(SpoolReference),
    Uri(Url),
}
```

These sketches are non-binding. Their purpose is to demonstrate separation between parsing, execution, services, and presentation.

---

## 21. Functional Requirements

### 21.1 Core shell

| ID | Requirement |
|---|---|
| FTSO-FR-001 | The system shall provide an interactive FTSO command shell within FileForgeWorkbench. |
| FTSO-FR-002 | The shell shall submit parsed commands to a central command dispatcher. |
| FTSO-FR-003 | The shell shall support quoted operands, named options, subcommands, and command continuation. |
| FTSO-FR-004 | The shell shall provide context-sensitive help derived from command metadata. |
| FTSO-FR-005 | The shell shall maintain command history under the active security policy. |
| FTSO-FR-006 | The shell shall support command cancellation. |
| FTSO-FR-007 | The shell shall support multiple isolated sessions. |
| FTSO-FR-008 | The system shall support interactive and non-interactive command execution through the same dispatcher. |

### 21.2 Command registration

| ID | Requirement |
|---|---|
| FTSO-FR-010 | Commands shall be registered using command descriptors. |
| FTSO-FR-011 | A command descriptor shall identify syntax, help, provider, capabilities, and stream types. |
| FTSO-FR-012 | The registry shall support namespaced commands. |
| FTSO-FR-013 | The registry shall reject or explicitly resolve ambiguous command names. |
| FTSO-FR-014 | Plugins shall be able to register commands through the supported plugin API. |

### 21.3 MiniX integration

| ID | Requirement |
|---|---|
| FTSO-FR-020 | Dataset commands shall use the FileForgeWorkbench VFS and dataset services. |
| FTSO-FR-021 | Catalogue commands shall use the shared MiniX catalogue service. |
| FTSO-FR-022 | Job commands shall use the shared MiniX job entry service. |
| FTSO-FR-023 | Spool commands and graphical job views shall use the same spool service. |
| FTSO-FR-024 | GDG commands shall resolve generations through the shared GDG service. |
| FTSO-FR-025 | VSAM and IDCAMS-like commands shall use shared typed service operations. |

### 21.4 Record integrity

| ID | Requirement |
|---|---|
| FTSO-FR-030 | Record-oriented commands shall preserve logical record boundaries. |
| FTSO-FR-031 | Commands shall not introduce line terminators into FB or VB datasets unless explicitly requested by a conversion operation. |
| FTSO-FR-032 | Imports into fixed-record datasets shall require a defined policy for short and oversized input records. |
| FTSO-FR-033 | Pipelines shall declare and validate stream types. |
| FTSO-FR-034 | Conversion between text, byte, and record streams shall be explicit and testable. |

### 21.5 Host integration

| ID | Requirement |
|---|---|
| FTSO-FR-040 | Native operating-system commands shall require an explicit host-command invocation. |
| FTSO-FR-041 | Host command execution shall be governed by workspace security policy. |
| FTSO-FR-042 | Host commands shall execute with a controlled environment and working directory. |
| FTSO-FR-043 | The host adapter shall support cancellation and process-tree termination where supported by the host platform. |
| FTSO-FR-044 | Host command output shall be distinguishable from FTSO command output. |

### 21.6 Scripting

| ID | Requirement |
|---|---|
| FTSO-FR-050 | The system shall support execution of FTSO command files. |
| FTSO-FR-051 | Scripts shall be able to inspect a command's structured completion status. |
| FTSO-FR-052 | Script execution shall be subject to the same authorisation checks as interactive execution. |
| FTSO-FR-053 | Script cancellation shall propagate to active commands. |
| FTSO-FR-054 | Script diagnostics shall identify the script name, line, and command associated with a failure. |

### 21.7 Diagnostics and audit

| ID | Requirement |
|---|---|
| FTSO-FR-060 | Every command shall return a structured completion result. |
| FTSO-FR-061 | Diagnostics shall include severity, message identifier, and human-readable text. |
| FTSO-FR-062 | Commands marked as auditable shall produce an audit event. |
| FTSO-FR-063 | Secret operands shall be redacted from history, logs, diagnostics, and audit output. |
| FTSO-FR-064 | Related command, job, service, and audit events shall share a correlation identifier. |

---

## 22. Non-functional Requirements

| ID | Requirement |
|---|---|
| FTSO-NFR-001 | Core commands shall behave consistently on all supported FFWB host platforms. |
| FTSO-NFR-002 | The shell shall remain responsive while long-running commands execute. |
| FTSO-NFR-003 | Command output shall be streamable and shall not require the complete result to be held in memory. |
| FTSO-NFR-004 | The architecture shall permit commands and service implementations to be unit tested without a terminal UI. |
| FTSO-NFR-005 | The parser and dispatcher shall be fuzz-testable with untrusted command input. |
| FTSO-NFR-006 | The host command adapter shall be isolated from the core MiniX service implementation. |
| FTSO-NFR-007 | Command descriptors and compatibility profiles shall be versioned. |
| FTSO-NFR-008 | Plugin failures shall not corrupt the command registry or another session. |
| FTSO-NFR-009 | Output rendering shall support large result sets through paging, streaming, or virtualisation. |
| FTSO-NFR-010 | The terminal shall support keyboard-only operation. |
| FTSO-NFR-011 | User-visible terminology shall remain consistent across command help, panels, messages, and documentation. |
| FTSO-NFR-012 | The implementation shall use British or South African English in project-authored user-facing documentation unless a compatibility profile requires fixed command terminology. |

---

## 23. Acceptance Criteria for the Initial Release

The initial release shall be accepted when all of the following are demonstrated:

1. A user can open an FTSO terminal within FileForgeWorkbench.
2. `HELP` lists commands registered by the core provider.
3. `HELP <command>` is generated from command metadata.
4. The parser handles quoting, named operands, continuations, and syntax errors.
5. `LISTCAT` obtains results from the shared catalogue service.
6. `ALLOC`, `INFO`, and `DELETE` operate against a test VFS catalogue under capability controls.
7. `VIEW` or `EDIT` can open a selected virtual dataset through the standard FFWB UI.
8. An FB dataset can be copied through a command without introducing CR, LF, or CRLF bytes.
9. A VB dataset can be read and written without losing logical record boundaries.
10. A long-running command can stream output and be cancelled.
11. Two terminal sessions maintain independent history and session state.
12. A sample plugin can register and execute a namespaced command.
13. A command file can execute multiple commands and inspect completion status.
14. Host command access is disabled unless explicitly enabled by policy.
15. Secrets marked by command metadata are absent from history and logs.
16. Automated tests cover command dispatch, authorisation, record integrity, and cancellation.

---

## 24. Implementation Roadmap

### Phase 1: Command foundation

- Define command descriptors, invocations, results, diagnostics, and cancellation.
- Implement the parser, registry, dispatcher, and test kit.
- Implement the terminal presentation with history and help.
- Add core session commands.

**Exit criterion:** Built-in commands can be registered, discovered, executed, cancelled, and tested independently of the UI.

### Phase 2: Dataset and catalogue commands

- Connect FTSO to VFS, catalogue, member, and record services.
- Implement `LISTCAT`, `ALLOC`, `INFO`, `DELETE`, `COPY`, `VIEW`, and `EDIT`.
- Add dataset-name completion.
- Add record-integrity tests for FB and VB datasets.

**Exit criterion:** A user can manage and inspect virtual datasets without falling back to host file semantics.

### Phase 3: Jobs and spool

- Connect `SUBMIT`, `JES`, and `SPOOL` to MiniX job services.
- Add asynchronous job notifications.
- Add an SDSF-style graphical and terminal view.

**Exit criterion:** A user can submit a supported job, monitor it, and inspect its spool output.

### Phase 4: Scripting and pipelines

- Implement FTSO command files.
- Add structured pipeline streams.
- Add conditionals, variables, and completion-status handling.
- Implement non-interactive invocation for automation.

**Exit criterion:** A repeatable dataset or job workflow can run as a governed script.

### Phase 5: Host and plugin extensions

- Implement the policy-controlled host adapter.
- Finalise command-provider plugin APIs.
- Provide a reference plugin and compatibility tests.

**Exit criterion:** An approved host command and a third-party plugin command can execute without bypassing dispatcher security.

### Phase 6: Compatibility and intelligent tooling

- Evaluate REXX or CLIST-inspired support.
- Add optional compatibility profiles.
- Integrate ERI, RISL, PSL, or AI providers where separately approved.
- Add command explanation and command-generation safeguards.

**Exit criterion:** Compatibility and intelligent extensions use the same governed interfaces as core commands.

---

## 25. Testing Strategy

### 25.1 Unit tests

- lexer and parser cases;
- quoted dataset names;
- option validation;
- alias and namespace resolution;
- capability checks;
- completion-code mapping;
- stream-type compatibility;
- secret redaction; and
- cancellation propagation.

### 25.2 Property and fuzz tests

- arbitrary command text shall not crash the parser;
- serialisation and deserialisation of typed command models shall round-trip;
- dataset reference parsing shall not silently reinterpret host paths;
- record pipelines shall preserve record counts and boundaries under supported transformations; and
- malformed plugin metadata shall be rejected safely.

### 25.3 Integration tests

- command to MiniX service calls;
- VFS catalogue operations;
- FB and VB copy fidelity;
- GDG resolution;
- job submission and spool retrieval;
- multi-session isolation;
- policy-controlled host execution; and
- plugin registration and unload.

### 25.4 Compatibility tests

Where an FTSO command deliberately models an established mainframe command, fixtures shall define:

- supported syntax;
- expected semantic behaviour;
- known differences;
- return-code mapping; and
- diagnostic mapping.

No compatibility claim shall rely solely on command-name similarity.

---

## 26. Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Scope expands into full z/OS emulation | Delivery becomes impractical | Maintain explicit non-goals and implement only service-backed use cases. |
| Users assume full TSO compatibility | Incorrect expectations and scripts | Label compatibility levels and document behavioural differences. |
| Host execution bypasses security | Data loss or system compromise | Require explicit `HOST`, capabilities, policy controls, redaction, and audit. |
| Dataset records are treated as text lines | Record corruption | Use typed record streams and explicit import/export conversion. |
| Plugin commands conflict | Ambiguous or unsafe execution | Use namespaces, stable provider IDs, and explicit precedence. |
| Long-running commands freeze the UI | Poor usability | Use asynchronous execution, streaming output, and cancellation. |
| MiniX name creates confusion | Branding or legal concerns | Treat the name as provisional and perform a naming review. |
| GUI and terminal behaviours diverge | Inconsistent results | Route both through common MiniX services and command contracts. |
| Compatibility quirks pollute the core | Reduced maintainability | Isolate quirks in versioned compatibility profiles and adapters. |

---

## 27. Open Design Decisions

The following decisions should be captured as ADRs before their respective implementation phases:

1. Final names for MiniX and FTSO.
2. Exact FTSO grammar and continuation convention.
3. Dataset prefix and qualification rules.
4. Completion-code and severity model.
5. Initial command set and syntax compatibility goals.
6. Pipeline stream types and conversion rules.
7. Scripting language and sandbox model.
8. Plugin command ABI and versioning strategy.
9. Host execution policy and sandboxing per supported platform.
10. Persistence format for history and session state.
11. Audit event schema and configurable retention.
12. Terminal widget or rendering technology.
13. Extent of ISPF PF-key and split-screen emulation.
14. Whether `SDSF` is a command, a panel launcher, or both.

---

## 28. Recommended Architectural Principles

1. **Service-first:** MiniX operations are services; commands and panels are clients.
2. **Typed internally:** Convert command text into typed invocations as early as possible.
3. **Record-aware by default:** Never reduce a dataset to lines unless explicitly converting it.
4. **Explicit host boundary:** Native commands must be visibly and technically separate.
5. **Metadata-driven:** Syntax, help, completion, capability, and stream behaviour come from descriptors.
6. **Secure by construction:** Authorisation happens before handler execution, not inside terminal presentation code.
7. **Structured results:** Text is a rendering of a result, not the result itself.
8. **Compatibility by profile:** Mainframe-like behaviour is versioned, documented, and tested.
9. **Extensible by provider:** Core, utility, job, script, plugin, ERI, and AI commands share one registration model.
10. **One operation, one service implementation:** GUI, commands, scripts, and APIs must not duplicate domain logic.

---

## 29. Conclusion

FileForgeWorkbench should implement a dedicated mainframe-oriented command environment rather than treating an embedded Bash or PowerShell terminal as the primary interface.

The recommended design is:

```text
FTSO Command Shell
        |
Command Dispatcher and Provider Registry
        |
MiniX Service Environment
        |
FFWB VFS, catalogue, record, GDG, VSAM, JES, spool, and security services
```

This structure preserves a familiar Option 6-style workflow while remaining portable, testable, extensible, and aligned with FileForgeWorkbench's larger objective of providing a modern workstation for mainframe-style files, utilities, jobs, and requirements-aware tooling.

A native host shell remains useful, but it should be a carefully governed adapter behind an explicit `HOST` boundary rather than the foundation of the FileForgeWorkbench terminal experience.

---

## Appendix A: Candidate Initial Commands

```text
HELP
CLEAR
EXIT
HISTORY
SESSION
SET
SHOW
LISTCAT
ALLOC
FREE
INFO
DELETE
RENAME
COPY
MEMBER
VIEW
BROWSE
EDIT
SUBMIT
JES
SPOOL
EXEC
HOST
```

---

## Appendix B: Example FTSO Session

```text
FTSO READY
FTSO> SET PREFIX ALAN
PREFIX SET TO ALAN
MAXCC=0

FTSO> LISTCAT LEVEL(ALAN)
ALAN.COBOL                  LIBRARY
ALAN.JCL                    LIBRARY
ALAN.TEST.DATA              SEQUENTIAL  FB  LRECL=80
MAXCC=0

FTSO> MEMBER LIST 'ALAN.COBOL'
PROG001
PROG002
COPYBOOK
MAXCC=0

FTSO> INFO 'ALAN.TEST.DATA'
NAME=ALAN.TEST.DATA
ORGANISATION=SEQUENTIAL
RECFM=FB
LRECL=80
RECORDS=1250
MAXCC=0

FTSO> SUBMIT 'ALAN.JCL(TESTJOB)'
JOB00042 SUBMITTED
MAXCC=0

FTSO> JES STATUS JOB00042
JOB00042 COMPLETED MAXCC=0000
MAXCC=0

FTSO> SPOOL VIEW JOB00042 DDNAME(SYSPRINT)
FFWB SPOOL VIEW OPENED
MAXCC=0
```

---

## Appendix C: Suggested ADRs

```text
ADR-FTSO-001  Adopt a service-backed FTSO command environment
ADR-FTSO-002  Separate FTSO commands from host-shell execution
ADR-FTSO-003  Use typed record streams for command pipelines
ADR-FTSO-004  Register commands through metadata-driven providers
ADR-FTSO-005  Use structured command results and diagnostics
ADR-FTSO-006  Treat TSO compatibility as explicit versioned profiles
ADR-FTSO-007  Provide capability-based command authorisation
ADR-FTSO-008  Share MiniX services between GUI, commands, and automation
```
