# FileForgeWorkbench DDNAME Mapping and Virtual Dataset Allocation Design

**Document type:** Architecture and requirements specification  
**Project:** FileForgeWorkbench  
**Status:** Proposed baseline design  
**Version:** 1.0  
**Date:** 2026-09-06

---

## 1. Purpose

This document defines how FileForgeWorkbench shall emulate IBM mainframe Data Definition names (DDNAMEs) and map them to datasets, files, streams, spool destinations, virtual devices, and application resources.

The design preserves familiar z/OS naming and behaviour in the execution engine while providing user-friendly labels and modern configuration capabilities in the graphical interface.

---

## 2. Design Decision

FileForgeWorkbench shall retain recognised IBM mainframe DD names without renaming them in the execution layer.

Examples include:

```text
SYSIN
SYSOUT
SYSPRINT
SORTIN
SORTOUT
SORTWK01
SYSTSIN
SYSTSPRT
SYSUDUMP
SYSABEND
SYSMDUMP
```

A presentation layer may associate a friendly display label with each DDNAME, for example:

```text
Control Statements (SYSIN)
Console or Spool Output (SYSOUT)
Program Report (SYSPRINT)
Sort Input Dataset (SORTIN)
Sort Output Dataset (SORTOUT)
```

This two-layer design provides:

- mainframe-compatible terminology;
- recognisable behaviour for experienced z/OS users;
- easier migration of JCL, COBOL, REXX, DFSORT, IDCAMS, TSO and ISPF-style workloads;
- a friendlier interface for users without mainframe experience;
- a stable internal contract for utilities, plug-ins and automation;
- support for future JES, JCL and batch-execution emulation.

---

## 3. Core Concepts

### 3.1 DDNAME

A DDNAME is a logical name used by a program or job step to identify an allocated resource. It is not the physical filename or dataset name.

Examples:

```text
SORTIN
SORTOUT
CUSTOMER
MASTER
REPORT
```

### 3.2 Dataset name

A dataset name identifies a logical FileForgeWorkbench dataset managed by the virtual mainframe environment.

Examples:

```text
PROD.CUSTOMER.MASTER
DAILY.SORT.INPUT
DAILY.SORT.OUTPUT
USER01.REPORTS.DAILY
```

### 3.3 Physical resource

A physical resource is the underlying implementation used to store, retrieve or expose data. It may be:

- a native Windows, Linux or macOS file;
- a FileForgeWorkbench-managed sequential dataset;
- a PDS or PDSE-style library member;
- a VSAM or ISAM simulation;
- a GDG generation;
- an in-memory stream;
- a temporary workspace;
- a terminal input or output stream;
- a virtual spool destination;
- a null or discard destination;
- a plug-in-provided resource.

### 3.4 DD allocation

A DD allocation binds a DDNAME to one or more resources for the duration of a job step, interactive command, script or application session.

Example:

```text
DDNAME:   SORTIN
Dataset:  PROD.CUSTOMER.INPUT
Physical: C:\FileForge\datasets\PROD\CUSTOMER\INPUT.dat
Mode:     Read
```

### 3.5 Job-step scope

DD allocations shall normally belong to a virtual job step. A program resolves its DDNAMEs against the allocation table for the active step.

---

## 4. Mapping Model

FileForgeWorkbench shall use the following logical mapping chain:

```text
Program or utility
        |
        v
      DDNAME
        |
        v
Virtual job-step allocation
        |
        v
Dataset or stream reference
        |
        v
Storage, spool, terminal or plug-in provider
```

Example:

```text
DFSORT emulator
    -> SORTIN
    -> PROD.CUSTOMER.INPUT
    -> Native file provider
    -> C:\Data\customer-input.fb
```

The program shall request `SORTIN`; it shall not need to know the physical path.

---

## 5. DDNAME Naming Rules

### 5.1 Canonical form

The canonical DDNAME shall:

- use uppercase characters internally;
- contain one to eight characters by default for mainframe compatibility;
- begin with an alphabetic character or an accepted national character when strict compatibility mode permits it;
- contain only characters allowed by the selected compatibility profile;
- be unique within a job step, except where concatenation is explicitly configured;
- remain separate from the dataset name and physical path.

### 5.2 Case handling

The engine shall normalise DDNAMEs to uppercase unless a non-mainframe compatibility profile explicitly enables case-sensitive names.

For example:

```text
sortin  -> SORTIN
SortOut -> SORTOUT
```

### 5.3 Reserved and user-defined DDNAMEs

FileForgeWorkbench shall support:

1. **Recognised system DDNAMEs**, for example `SYSIN` and `SYSPRINT`.
2. **Utility-specific DDNAMEs**, for example `SORTIN` and `SORTOUT`.
3. **Subsystem-specific DDNAMEs**, for example ISPF library allocations.
4. **Application-defined DDNAMEs**, for example `CUSTOMER`, `MASTER` and `REPORT`.

Recognised names shall receive default metadata and suggested behaviour, but shall not prevent an application from defining other valid names.

---

## 6. Standard DDNAME Registry

The registry shall describe recognised DDNAMEs, their default roles and their expected resource types. These definitions are defaults rather than a claim that every mainframe program uses the same DDNAMEs.

### 6.1 Core system DDNAMEs

| DDNAME | Friendly label | Default role | Suggested default target |
|---|---|---|---|
| `SYSIN` | Control Statements | Program or utility control input | Inline data, dataset or editor buffer |
| `SYSOUT` | System Output | General program output | Virtual spool |
| `SYSPRINT` | Program Report | Reports, listings and diagnostics | Virtual spool |
| `SYSUDUMP` | User Dump | User-format diagnostic dump | Dump dataset or spool |
| `SYSABEND` | Abend Dump | Diagnostic output following abnormal termination | Dump dataset or spool |
| `SYSMDUMP` | Machine Dump | Machine-oriented dump output | Binary dump dataset |
| `CEEDUMP` | Language Environment Dump | Language Environment diagnostic output | Dump dataset or spool |

### 6.2 Program library DDNAMEs

| DDNAME | Friendly label | Default role | Suggested default target |
|---|---|---|---|
| `STEPLIB` | Step Program Library | Program libraries for one step | Ordered library concatenation |
| `JOBLIB` | Job Program Library | Program libraries for a job | Ordered library concatenation |

### 6.3 Sort and merge DDNAMEs

| DDNAME or pattern | Friendly label | Default role | Suggested default target |
|---|---|---|---|
| `SORTIN` | Sort Input | Primary sort or copy input | Sequential dataset |
| `SORTINnn` | Sort or Merge Input nn | Multiple input datasets | Sequential dataset |
| `SORTOUT` | Sort Output | Primary processed output | Sequential dataset |
| `SORTWKnn` | Sort Work nn | Sort work storage | Managed temporary dataset |
| `SORTCNTL` | Sort Control Statements | Additional sort control input | Inline data or sequential dataset |
| `DFSPARM` | DFSORT Parameters | Sort run-time options and controls | Inline data or sequential dataset |
| User-defined `OUTFIL` DDNAMEs | Additional Sort Output | One or more named output streams | Sequential dataset or spool |

The `nn` component shall be represented as a configurable numeric suffix. The compatibility profile shall determine the accepted range.

### 6.4 TSO-style DDNAMEs

| DDNAME | Friendly label | Default role | Suggested default target |
|---|---|---|---|
| `SYSTSIN` | TSO Command Input | Commands supplied to a TSO-style command processor | Terminal, script or dataset |
| `SYSTSPRT` | TSO Command Output | Command results and messages | Terminal and/or virtual spool |

### 6.5 ISPF-style DDNAMEs

| DDNAME | Friendly label | Default role | Suggested default target |
|---|---|---|---|
| `ISPPLIB` | ISPF Panel Library | Panel definitions | Ordered PDS-style library concatenation |
| `ISPMLIB` | ISPF Message Library | Message definitions | Ordered PDS-style library concatenation |
| `ISPSLIB` | ISPF Skeleton Library | File-tailoring skeletons | Ordered PDS-style library concatenation |
| `ISPTLIB` | ISPF Table Input Library | Input tables | Ordered PDS-style library concatenation |
| `ISPTABL` | ISPF Table Output Library | Output tables | Writable PDS-style library |
| `ISPPROF` | ISPF Profile Library | User profile data | Writable user library |
| `ISPLOG` | ISPF Log | Session logging | Sequential dataset or spool |
| `ISPLIST` | ISPF List Output | Generated list output | Sequential dataset or spool |
| `ISPFILE` | ISPF File-tailoring Output | File-tailoring output | Sequential dataset |

The ISPF emulation component may extend this registry as its compatibility scope grows.

### 6.6 Common application DDNAMEs

The following names are conventions only and shall not be reserved:

```text
INFILE
OUTFILE
INPUT
OUTPUT
INPUT1
INPUT2
MASTER
TRANFILE
REPORT
WORKFILE
CUSTOMER
```

Application-defined DDNAMEs shall use exactly the same allocation and resolution mechanism as recognised DDNAMEs.

---

## 7. Resource Types

A DD allocation shall be able to target the following resource classes.

### 7.1 Sequential dataset

Used for record-oriented or stream-oriented sequential data, including:

- fixed (F);
- fixed blocked (FB);
- variable (V);
- variable blocked (VB);
- undefined (U);
- text or line-sequential data when explicitly selected.

Record format, logical record length and block size shall belong to dataset metadata rather than the DDNAME.

### 7.2 Library or member

Used for PDS or PDSE-style libraries and individual members.

Example:

```text
//SYSIN DD DSN=USER01.CONTROL(SORT01),DISP=SHR
```

### 7.3 Concatenation

A single DDNAME may map to an ordered collection of compatible datasets or libraries.

Example:

```text
STEPLIB -> APP.LOAD
           COMMON.LOAD
           SYSTEM.LOAD
```

The resource resolver shall search or read the concatenation in its defined order according to the resource type.

### 7.4 Inline data

`SYSIN` and other input DDNAMEs may receive inline control data associated with a job step.

### 7.5 Terminal stream

A DDNAME may map to an interactive terminal input or output channel. This is particularly relevant to `SYSTSIN`, `SYSTSPRT`, `SYSIN` and `SYSOUT`.

### 7.6 Virtual spool

Output DDNAMEs may map to FileForgeWorkbench's virtual JES spool. Spool entries shall retain job, step, procedure, DDNAME and creation metadata.

### 7.7 Temporary dataset

A DDNAME may target a temporary dataset whose lifetime is controlled by the virtual job or session.

### 7.8 Null destination

A discard provider shall be available for output that is intentionally suppressed.

### 7.9 Plug-in resource

The resource-provider interface shall permit plug-ins to resolve DD allocations to additional systems without changing DDNAME semantics.

---

## 8. Allocation Attributes

A DD allocation should support the following attributes where relevant:

| Attribute | Purpose |
|---|---|
| `ddname` | Canonical logical name |
| `displayLabel` | Friendly user-facing name |
| `resourceUri` | Provider-neutral resource reference |
| `datasetName` | Logical dataset name, when applicable |
| `memberName` | Library member, when applicable |
| `accessMode` | Read, write, update or append |
| `dispositionStatus` | New, old, shared or modify-style status |
| `normalDisposition` | Action after normal completion |
| `abnormalDisposition` | Action after abnormal completion |
| `recordFormat` | F, FB, V, VB, U or line-sequential |
| `logicalRecordLength` | Maximum or fixed logical record length |
| `blockSize` | Physical or simulated block size |
| `encoding` | EBCDIC code page, ASCII, UTF encoding or binary |
| `concatenationOrder` | Resource sequence within a concatenation |
| `spoolClass` | Virtual spool routing class |
| `temporary` | Indicates job- or session-scoped storage |
| `optional` | Indicates whether absence is permitted |
| `providerOptions` | Provider-specific configuration |

---

## 9. Proposed Internal Data Model

The implementation language may evolve, but the logical model should resemble the following Rust structures:

```rust
pub struct DdAllocation {
    pub ddname: DdName,
    pub display_label: Option<String>,
    pub resources: Vec<DdResourceBinding>,
    pub access_mode: AccessMode,
    pub disposition: Disposition,
    pub optional: bool,
}

pub struct DdResourceBinding {
    pub resource_uri: String,
    pub dataset_name: Option<String>,
    pub member_name: Option<String>,
    pub record_attributes: Option<RecordAttributes>,
    pub encoding: Option<String>,
    pub provider_options: ProviderOptions,
}

pub struct RecordAttributes {
    pub record_format: RecordFormat,
    pub logical_record_length: Option<u32>,
    pub block_size: Option<u32>,
}
```

The model shall allow multiple resources for DD concatenation and shall avoid embedding operating-system-specific paths into program logic.

---

## 10. Resolution Rules

When a program requests a DDNAME, FileForgeWorkbench shall:

1. normalise and validate the requested DDNAME;
2. identify the active virtual job and job step;
3. search the step-level allocation table;
4. if allowed by the execution profile, search inherited procedure, job or session allocations;
5. resolve the allocation through its configured resource provider;
6. validate access mode, disposition, record attributes and encoding;
7. return an appropriate record, stream, library, terminal or spool interface;
8. produce a clear allocation error when resolution fails.

Programs shall resolve resources by DDNAME rather than by physical path.

---

## 11. Scope and Precedence

Recommended allocation precedence, from highest to lowest, is:

1. explicit job-step override;
2. procedure-step override;
3. job-level allocation;
4. active session allocation;
5. workspace profile default;
6. recognised DDNAME default.

The effective allocation view shall show both the selected binding and the lower-precedence definitions it overrides.

---

## 12. Graphical User Interface

### 12.1 DD allocation table

The user interface should display:

| Purpose | DDNAME | Dataset or resource | Mode | Status |
|---|---|---|---|---|
| Control Statements | `SYSIN` | `JOB.CONTROL` | Read | Allocated |
| Sort Input Dataset | `SORTIN` | `CUSTOMER.INPUT` | Read | Allocated |
| Sort Output Dataset | `SORTOUT` | `CUSTOMER.OUTPUT` | Write | Allocated |
| Program Report | `SYSPRINT` | Virtual spool | Write | Allocated |

### 12.2 Display options

The GUI shall support these views:

- **Mainframe view:** DDNAME is the primary label.
- **Friendly view:** friendly label is primary and DDNAME appears in parentheses.
- **Technical view:** displays DDNAME, dataset, provider URI, record metadata, disposition and scope.
- **Effective allocation view:** shows the binding that will be used at execution time.

### 12.3 Alias handling

A friendly alias shall not replace the canonical DDNAME.

Example:

```json
{
  "ddname": "SORTIN",
  "displayLabel": "Customer Input File"
}
```

The GUI may display:

```text
Customer Input File (SORTIN)
```

Scripts, JCL and programs shall continue to address the resource as `SORTIN`.

### 12.4 Validation

The editor should identify:

- invalid DDNAME syntax;
- duplicate allocations in the same scope;
- unresolved resources;
- incompatible concatenations;
- read-only resources allocated for output;
- record-format conflicts;
- missing required DDNAMEs;
- invalid or conflicting disposition settings;
- cyclic aliases or provider references.

---

## 13. Virtual JES and Job-Step Context

Each submitted or interactively launched workload shall execute within a virtual context similar to:

```text
Job:      DAILY01
Step:     SORTCUST
Program:  SORT

DDNAME       Resource
------------ --------------------------------
SYSIN        DAILY01.SORT.CONTROL
SORTIN       PROD.CUSTOMER.INPUT
SORTOUT      PROD.CUSTOMER.SORTED
SYSPRINT     spool://DAILY01/SORTCUST/SYSPRINT
SORTWK01     temp://DAILY01/SORTCUST/SORTWK01
```

The virtual context shall own:

- DD allocations;
- environment variables and execution parameters;
- return code and completion status;
- virtual spool outputs;
- temporary resources;
- diagnostic and dump destinations;
- audit and trace information.

---

## 14. Behaviour by DDNAME Category

### 14.1 Input DDNAMEs

Typical input DDNAMEs include `SYSIN`, `SORTIN`, `SORTINnn`, `SYSTSIN` and application-defined input names.

Expected behaviour:

- open for read unless explicitly configured otherwise;
- preserve record boundaries for record-oriented datasets;
- process concatenated inputs in allocation order;
- fail clearly when a required allocation is absent;
- allow an optional empty-input policy only when explicitly configured.

### 14.2 Output DDNAMEs

Typical output DDNAMEs include `SYSOUT`, `SYSPRINT`, `SORTOUT`, `SYSTSPRT`, `SYSUDUMP`, `SYSABEND`, `SYSMDUMP` and application-defined output names.

Expected behaviour:

- route output to a dataset, terminal, spool or discard provider;
- respect access and disposition settings;
- preserve record metadata where the destination is record-oriented;
- retain spool metadata for virtual JES output;
- prevent accidental overwrite unless the selected disposition permits it.

### 14.3 Work DDNAMEs

Typical work DDNAMEs include `SORTWKnn` and application-defined work files.

Expected behaviour:

- allocate managed temporary storage by default;
- isolate storage by job and step;
- clean up storage according to completion and diagnostic-retention policy;
- permit explicit persistent allocation for debugging.

### 14.4 Library DDNAMEs

Typical library DDNAMEs include `STEPLIB`, `JOBLIB` and the ISPF library DDNAMEs.

Expected behaviour:

- support ordered concatenation;
- search members in allocation order where appropriate;
- distinguish read-only input libraries from writable output libraries;
- expose the winning library and member in diagnostics.

---

## 15. Error Handling

Allocation errors shall identify at least:

- job and step identity;
- requested DDNAME;
- allocation scope searched;
- intended operation;
- resource being resolved;
- provider involved;
- reason for failure;
- corrective suggestion where one can be determined safely.

Example:

```text
FFW-DD-0004: Required DDNAME SORTIN is not allocated for job DAILY01,
step SORTCUST. Allocate SORTIN to a readable sequential dataset or mark
it optional in the execution profile.
```

Diagnostics shall distinguish between:

- DDNAME not allocated;
- dataset not found;
- member not found;
- incompatible resource type;
- access denied;
- encoding failure;
- record-format mismatch;
- disposition conflict;
- provider unavailable;
- concatenation error;
- spool-routing error.

---

## 16. Configuration and Persistence

DD allocation definitions should be serialisable in a provider-neutral format such as JSON, YAML or TOML.

Example JSON:

```json
{
  "job": "DAILY01",
  "step": "SORTCUST",
  "allocations": [
    {
      "ddname": "SYSIN",
      "displayLabel": "Sort Control Statements",
      "resources": [
        {
          "resourceUri": "dataset://DAILY01.SORT.CONTROL"
        }
      ],
      "accessMode": "read"
    },
    {
      "ddname": "SORTIN",
      "displayLabel": "Customer Input File",
      "resources": [
        {
          "resourceUri": "dataset://PROD.CUSTOMER.INPUT"
        }
      ],
      "accessMode": "read"
    },
    {
      "ddname": "SORTOUT",
      "displayLabel": "Sorted Customer File",
      "resources": [
        {
          "resourceUri": "dataset://PROD.CUSTOMER.SORTED"
        }
      ],
      "accessMode": "write"
    },
    {
      "ddname": "SYSPRINT",
      "displayLabel": "Sort Report",
      "resources": [
        {
          "resourceUri": "spool://current-job/current-step/SYSPRINT"
        }
      ],
      "accessMode": "write"
    }
  ]
}
```

Physical paths should be stored by the relevant provider rather than exposed as the primary portable identifier.

---

## 17. JCL Compatibility

The JCL parser shall preserve DDNAME text and create equivalent virtual allocations.

Example input:

```jcl
//SORTSTEP EXEC PGM=SORT
//SYSOUT   DD SYSOUT=*
//SYSPRINT DD SYSOUT=*
//SYSIN    DD *
  SORT FIELDS=(1,10,CH,A)
/*
//SORTIN   DD DSN=PROD.CUSTOMER.INPUT,DISP=SHR
//SORTOUT  DD DSN=PROD.CUSTOMER.SORTED,
//            DISP=(NEW,CATLG,DELETE)
```

Conceptual FileForgeWorkbench mapping:

```text
SYSOUT   -> virtual spool
SYSPRINT -> virtual spool
SYSIN    -> inline control records
SORTIN   -> existing shared dataset
SORTOUT  -> newly allocated catalogued dataset
```

The compatibility layer shall retain the original DDNAMEs even if the GUI displays friendly aliases.

---

## 18. Command-Line and Scripting Interface

A command-line or dialog automation interface should permit allocation, inspection and release operations.

Illustrative syntax:

```text
ffw dd allocate SORTIN dataset://PROD.CUSTOMER.INPUT --mode read
ffw dd allocate SORTOUT dataset://PROD.CUSTOMER.SORTED --mode write
ffw dd allocate SYSPRINT spool://current-job/current-step/SYSPRINT
ffw dd list --effective
ffw dd inspect SORTIN
ffw dd free SORTIN
```

The exact command syntax is subject to the FileForgeWorkbench command architecture, but the DDNAME and allocation semantics shall remain stable.

---

## 19. EARS-Style Requirements

### 19.1 Core DDNAME requirements

**FFW-DD-001**  
The FileForgeWorkbench execution engine **shall** represent a DDNAME as a first-class logical resource identifier.

**FFW-DD-002**  
The FileForgeWorkbench execution engine **shall** preserve recognised IBM mainframe DD names in their canonical form.

**FFW-DD-003**  
When a DDNAME is entered using lowercase or mixed case, the FileForgeWorkbench execution engine **shall** normalise it according to the active compatibility profile.

**FFW-DD-004**  
The FileForgeWorkbench execution engine **shall** keep DDNAMEs separate from dataset names and physical resource paths.

**FFW-DD-005**  
The FileForgeWorkbench execution engine **shall** support application-defined DDNAMEs in addition to recognised system and utility DDNAMEs.

**FFW-DD-006**  
Where strict mainframe compatibility is enabled, the FileForgeWorkbench execution engine **shall** validate DDNAME length and character rules against the selected compatibility profile.

### 19.2 Mapping and resolution requirements

**FFW-DD-010**  
The FileForgeWorkbench execution engine **shall** resolve a program's DDNAME request through the active virtual job-step allocation table.

**FFW-DD-011**  
When an allocation references a provider-neutral resource URI, the FileForgeWorkbench execution engine **shall** delegate resource resolution to the corresponding provider.

**FFW-DD-012**  
When multiple allocation scopes define the same DDNAME, the FileForgeWorkbench execution engine **shall** apply the configured allocation-precedence rules.

**FFW-DD-013**  
When a required DDNAME cannot be resolved, the FileForgeWorkbench execution engine **shall** stop the affected operation and return an allocation diagnostic.

**FFW-DD-014**  
Where a DDNAME is marked optional and no binding exists, the FileForgeWorkbench execution engine **shall** apply the configured optional-resource policy.

**FFW-DD-015**  
The FileForgeWorkbench execution engine **shall** prevent program logic from depending directly on an operating-system-specific physical path when a DD allocation is available.

### 19.3 Friendly-label requirements

**FFW-DD-020**  
The FileForgeWorkbench user interface **shall** permit a friendly display label to be associated with a canonical DDNAME.

**FFW-DD-021**  
When a friendly display label is shown, the FileForgeWorkbench user interface **shall** also make the canonical DDNAME available.

**FFW-DD-022**  
The FileForgeWorkbench user interface **shall not** replace the underlying canonical DDNAME with its friendly display label.

**FFW-DD-023**  
The FileForgeWorkbench user interface **shall** provide mainframe, friendly, technical and effective-allocation views.

### 19.4 Resource requirements

**FFW-DD-030**  
The FileForgeWorkbench execution engine **shall** support DD allocations to sequential datasets.

**FFW-DD-031**  
The FileForgeWorkbench execution engine **shall** support DD allocations to PDS or PDSE-style libraries and members.

**FFW-DD-032**  
The FileForgeWorkbench execution engine **shall** support DD allocations to inline data.

**FFW-DD-033**  
The FileForgeWorkbench execution engine **shall** support DD allocations to terminal input and output streams.

**FFW-DD-034**  
The FileForgeWorkbench execution engine **shall** support DD allocations to virtual spool destinations.

**FFW-DD-035**  
The FileForgeWorkbench execution engine **shall** support DD allocations to managed temporary datasets.

**FFW-DD-036**  
The FileForgeWorkbench execution engine **shall** support DD allocations to a null or discard destination.

**FFW-DD-037**  
The FileForgeWorkbench execution engine **shall** provide a provider interface for plug-in-defined DD resources.

### 19.5 Concatenation requirements

**FFW-DD-040**  
The FileForgeWorkbench execution engine **shall** permit a DDNAME to reference an ordered concatenation of compatible resources.

**FFW-DD-041**  
When a concatenated input DDNAME is read, the FileForgeWorkbench execution engine **shall** process the resources in allocation order.

**FFW-DD-042**  
When a concatenated library DDNAME is searched, the FileForgeWorkbench execution engine **shall** search libraries in allocation order.

**FFW-DD-043**  
When concatenated resources are incompatible, the FileForgeWorkbench execution engine **shall** reject the allocation and identify the conflicting attributes.

### 19.6 Record and encoding requirements

**FFW-DD-050**  
The FileForgeWorkbench dataset provider **shall** retain record format, logical record length and block size as dataset metadata.

**FFW-DD-051**  
When a DD allocation opens an FB dataset, the FileForgeWorkbench dataset provider **shall** preserve fixed-length logical record boundaries without requiring line terminators.

**FFW-DD-052**  
When a DD allocation opens a VB dataset, the FileForgeWorkbench dataset provider **shall** preserve variable-length logical record boundaries and the configured descriptor semantics without requiring line terminators.

**FFW-DD-053**  
When a resource encoding is configured, the FileForgeWorkbench resource provider **shall** apply that encoding at the provider boundary without changing the DDNAME.

**FFW-DD-054**  
When record metadata conflicts with the selected operation or destination, the FileForgeWorkbench execution engine **shall** reject the operation or require an explicit conversion policy.

### 19.7 Standard-registry requirements

**FFW-DD-060**  
The FileForgeWorkbench DDNAME registry **shall** provide metadata for the core system DDNAMEs listed in this specification.

**FFW-DD-061**  
The FileForgeWorkbench DDNAME registry **shall** provide metadata for the sort and merge DDNAMEs listed in this specification.

**FFW-DD-062**  
The FileForgeWorkbench DDNAME registry **shall** provide metadata for the TSO-style DDNAMEs listed in this specification.

**FFW-DD-063**  
The FileForgeWorkbench DDNAME registry **shall** provide metadata for the ISPF-style DDNAMEs listed in this specification.

**FFW-DD-064**  
The FileForgeWorkbench DDNAME registry **shall** permit additional DDNAME definitions to be installed without modifying the core execution engine.

**FFW-DD-065**  
The FileForgeWorkbench DDNAME registry **shall not** reserve common application DDNAME conventions unless a compatibility profile explicitly requires reservation.

### 19.8 Virtual JES and spool requirements

**FFW-DD-070**  
When a workload starts, the FileForgeWorkbench job service **shall** create a virtual job-step context containing its effective DD allocations.

**FFW-DD-071**  
When an output DDNAME is allocated to virtual spool, the FileForgeWorkbench spool service **shall** retain the associated job, step and DDNAME metadata.

**FFW-DD-072**  
When a job step ends normally, the FileForgeWorkbench job service **shall** apply each allocation's normal disposition.

**FFW-DD-073**  
When a job step ends abnormally, the FileForgeWorkbench job service **shall** apply each allocation's abnormal disposition.

**FFW-DD-074**  
When an abnormal termination occurs, the FileForgeWorkbench job service **shall** route configured diagnostic output to the allocated dump or spool DDNAMEs.

### 19.9 Temporary resource requirements

**FFW-DD-080**  
When a work DDNAME has no explicit persistent target, the FileForgeWorkbench execution engine **shall** allocate isolated temporary storage for the active job step.

**FFW-DD-081**  
When temporary storage is no longer required, the FileForgeWorkbench job service **shall** release it according to the configured retention policy.

**FFW-DD-082**  
Where diagnostic retention is enabled, the FileForgeWorkbench job service **shall** permit selected temporary resources to be retained after step completion.

### 19.10 JCL and application integration requirements

**FFW-DD-090**  
When the JCL parser processes a DD statement, it **shall** preserve the DDNAME and create an equivalent virtual allocation.

**FFW-DD-091**  
When a COBOL-style file assignment identifies a DDNAME, the FileForgeWorkbench runtime **shall** resolve the assignment through the active DD allocation table.

**FFW-DD-092**  
When the DFSORT emulator is invoked, it **shall** recognise the configured sort and merge DDNAME patterns.

**FFW-DD-093**  
When the TSO-style command processor is invoked, it **shall** use `SYSTSIN` for configured command input and `SYSTSPRT` for configured command output.

**FFW-DD-094**  
When the ISPF-style environment resolves panel, message, skeleton, table or profile resources, it **shall** use the corresponding configured library DD allocations.

### 19.11 Validation and diagnostics requirements

**FFW-DD-100**  
When an invalid DDNAME is entered, the FileForgeWorkbench user interface **shall** identify the violated naming rule.

**FFW-DD-101**  
When duplicate non-concatenated allocations exist in the same scope, the FileForgeWorkbench execution engine **shall** reject the configuration.

**FFW-DD-102**  
When an output operation targets a read-only resource, the FileForgeWorkbench execution engine **shall** reject the operation before writing data.

**FFW-DD-103**  
When a required standard DDNAME is absent for a selected utility profile, the FileForgeWorkbench user interface **shall** identify the missing allocation before execution where possible.

**FFW-DD-104**  
When allocation resolution fails, the FileForgeWorkbench diagnostic **shall** include the job, step, DDNAME, intended operation, resource and failure reason where available.

**FFW-DD-105**  
The FileForgeWorkbench user interface **shall** provide an effective-allocation inspector showing the selected binding and its source scope.

### 19.12 Persistence and portability requirements

**FFW-DD-110**  
The FileForgeWorkbench configuration service **shall** serialise DD allocations in a documented provider-neutral representation.

**FFW-DD-111**  
The FileForgeWorkbench configuration service **shall** keep portable resource identifiers separate from provider-specific physical-path settings.

**FFW-DD-112**  
When a workspace is moved between supported operating systems, FileForgeWorkbench **shall** preserve DDNAME semantics even where the underlying physical resource mapping changes.

**FFW-DD-113**  
The FileForgeWorkbench configuration service **shall** version the persisted DD allocation schema.

**FFW-DD-114**  
When an older allocation schema is opened, the FileForgeWorkbench configuration service **shall** either migrate it or report an actionable compatibility error.

---

## 20. Acceptance Criteria

The baseline implementation shall be accepted when:

1. A program can request `SORTIN` and read a bound sequential dataset without knowing its physical path.
2. A program can write to `SORTOUT` and the configured disposition is applied at step completion.
3. `SYSIN` can receive both inline data and a referenced dataset.
4. `SYSPRINT` and `SYSOUT` can route records to the virtual spool.
5. `SYSTSIN` and `SYSTSPRT` can connect to the FileForgeWorkbench command environment.
6. `SORTWK01` can be created as isolated temporary job-step storage.
7. `STEPLIB` can resolve program modules through an ordered library concatenation.
8. Friendly labels can be changed without changing canonical DDNAMEs.
9. The effective-allocation view identifies the resource selected for each DDNAME.
10. Missing or incompatible allocations produce structured diagnostics.
11. FB and VB resources preserve record boundaries without CR, LF or CRLF terminators.
12. JCL DD statements can create equivalent runtime allocations.
13. Allocation configuration can be saved, reopened and used on a supported operating system.
14. Application-defined DDNAMEs use the same allocation interface as recognised DDNAMEs.

---

## 21. Testing Strategy

### 21.1 Unit tests

Test:

- DDNAME normalisation;
- naming validation;
- scope precedence;
- alias separation;
- resource-provider selection;
- concatenation order;
- disposition processing;
- record metadata validation;
- structured error generation.

### 21.2 Integration tests

Test complete flows for:

- `SYSIN` inline control records;
- `SORTIN` to `SORTOUT` processing;
- `SYSPRINT` virtual spool output;
- `SYSTSIN` scripted command input;
- `SYSTSPRT` terminal and spool output;
- `SORTWKnn` temporary allocation and cleanup;
- `STEPLIB` concatenated module resolution;
- ISPF-style library concatenations;
- JCL-to-allocation translation;
- FB and VB dataset access.

### 21.3 Cross-platform tests

The same logical allocation configuration shall be tested on Windows, Linux and macOS with provider-specific physical mappings.

### 21.4 Negative tests

Test:

- missing required DDNAME;
- duplicate allocation;
- invalid DDNAME;
- nonexistent dataset;
- output to read-only resource;
- incompatible concatenation;
- invalid disposition;
- record-format mismatch;
- unsupported provider URI;
- unavailable spool provider.

---

## 22. Extensibility

Future versions may add:

- dynamic allocation compatible with an `ALLOCATE` or `FREE` command model;
- richer JES2 or JES3-style spool routing;
- subsystem-specific DDNAME profiles;
- DB2, CICS, IMS or MQ resource adapters;
- remote z/OS dataset providers;
- security labels and allocation authorisation policies;
- dataset-enqueue and sharing emulation;
- installation-specific DDNAME registries;
- import and export of job allocation templates;
- visual tracing from JCL DD statement to physical provider.

These additions shall extend the provider and registry model without changing the fundamental rule that programs address logical resources through DDNAMEs.

---

## 23. Final Recommendation

FileForgeWorkbench should preserve standard and application-defined DDNAMEs exactly as logical mainframe-facing names. A modern alias, metadata and resource-provider layer should be added around them rather than replacing them.

The canonical model is:

```text
Authentic DDNAME
    + friendly display label
    + scoped allocation
    + provider-neutral resource URI
    + dataset and record metadata
    + virtual JES/job-step context
```

This provides mainframe authenticity, cross-platform portability, user-friendly presentation, testability and a stable foundation for JCL, JES, TSO, ISPF, COBOL, REXX and utility emulation within FileForgeWorkbench.
