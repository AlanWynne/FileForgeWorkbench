# FileForge Workbench FFW-JES EARS Requirements

## Document Purpose

This document defines an initial EARS-style requirements list with acceptance criteria for a desktop-based mainframe Job Entry Subsystem and SDSF-style emulator for the FileForge Workbench project.

The proposed subsystem emulates the mainframe process where submitted jobs enter an input queue, wait for an available initiator, move into active execution, complete with return status, and expose job logs and output. It also includes a desktop dataset catalog system that maps mainframe-style dataset names such as `DSN=MY.DATASET` to physical files on Windows, Linux, or macOS.

---

## Proposed Subsystem Name

**FFW-JES: FileForge Workbench Job Entry Subsystem**

A cross-platform JES/SDSF-style emulator for Windows, Linux, and macOS.

---

## Epic

**As a developer or operator, I want to submit jobs to a local JES queue, monitor their execution, view output logs, manage datasets, and control execution resources so that I can emulate a mainframe batch environment on a desktop platform.**

---

## Architectural Components

| Component | Mainframe Equivalent | Desktop Equivalent |
|---|---|---|
| Job Entry Subsystem | JES2/JES3 | FFW-JES |
| Initiator Manager | Initiators | Worker Pool |
| SDSF | SDSF | FFW Job Monitor |
| Catalog | ICF Catalog | Dataset Catalog |
| Dataset | DSN | Catalog Entry |
| SYSOUT | SYSOUT Spool | Job Output Log |
| JES Queue | Input Queue | Job Queue |
| JESMSGLG | JES Log | Execution Log |
| JCL | JCL | FFJCL / FileForge Job Language |

---

# 1. Job Entry Subsystem Requirements

## FFW-JES-001 — Job Submission

**WHEN** a user submits a job  
**THEN** the system shall create a new job record in the Input Queue.

### Acceptance Criteria

- The job receives a unique Job ID.
- The submission timestamp is recorded.
- The submitting user or process is recorded.
- The initial job status is set to `QUEUED`.
- The job appears immediately in the Job Monitor.
- The queued job survives an application restart.
- Invalid job definitions are rejected with a meaningful validation message.

---

## FFW-JES-002 — Job Queue Visibility

**WHILE** jobs exist in the queue  
**THE SYSTEM SHALL** display all queued jobs in the Job Monitor.

### Acceptance Criteria

- The Job Monitor displays queued jobs in an Input Queue panel.
- The queue can be sorted by:
  - Job Name
  - Job ID
  - Owner/User
  - Submit Time
  - Priority
  - Status
- The queue can be refreshed without restarting the application.
- Queue state updates automatically when jobs change status.
- The user can distinguish between queued, held, active, completed, failed, and cancelled jobs.

---

## FFW-JES-003 — Initiator Pool

**WHERE** one or more initiators are configured  
**THE SYSTEM SHALL** maintain a pool of worker threads or worker processes.

### Acceptance Criteria

- The number of initiators is configurable.
- Each initiator has a unique identifier.
- The user can view each initiator's current status.
- Supported initiator statuses include:
  - `IDLE`
  - `STARTING`
  - `ACTIVE`
  - `STOPPING`
  - `STOPPED`
  - `FAILED`
- The system supports starting an initiator.
- The system supports stopping an initiator.
- The system supports pausing an initiator from taking new work.
- The system prevents more active jobs than the configured initiator capacity.

---

## FFW-JES-004 — Queue Scheduling

**WHEN** an initiator becomes available  
**AND** a queued job is eligible to execute  
**THEN** the scheduler shall dispatch the job for execution.

### Acceptance Criteria

- FIFO scheduling is supported.
- Priority-based scheduling is supported.
- Jobs marked as `HELD` are not dispatched.
- Jobs marked as `CANCELLED` are not dispatched.
- Jobs with unmet preconditions are not dispatched.
- The scheduler assigns the selected job to an available initiator.
- The job status changes from `QUEUED` to `ACTIVE` when execution begins.
- The job start timestamp is recorded.

---

## FFW-JES-005 — Active Job Monitoring

**WHILE** a job is executing  
**THE SYSTEM SHALL** display the job in `ACTIVE` status.

### Acceptance Criteria

The Job Monitor displays the following information for active jobs:

- Job Name
- Job ID
- Owner/User
- Assigned Initiator or Worker ID
- Start Time
- Elapsed Time
- Current Step
- Current Step Status
- Process ID, where applicable
- CPU usage, where available
- Memory usage, where available

Additional criteria:

- Active job information updates while the job is running.
- The user can open the live job log while the job is active.
- The user can request cancellation of an active job.

---

## FFW-JES-006 — Job Completion

**WHEN** a job completes successfully  
**THEN** the system shall move the job to `COMPLETED` status.

### Acceptance Criteria

- The job end timestamp is recorded.
- The elapsed runtime is calculated.
- The final return code is stored.
- Job logs are retained.
- Output datasets are retained.
- SYSOUT-style output is retained.
- The assigned initiator is released for the next eligible job.
- The completed job appears in the Completed or Output queue.

---

## FFW-JES-007 — Job Failure

**WHEN** a job terminates abnormally  
**THEN** the system shall place the job in `FAILED` status.

### Acceptance Criteria

- The job end timestamp is recorded.
- The failure reason is stored.
- The failing step is recorded, where applicable.
- The final return code or abnormal termination code is stored.
- Error details are written to the job log.
- Stack trace or diagnostic information is retained where available.
- Output generated before failure remains available.
- The user can inspect the failure from the Job Monitor.

---

## FFW-JES-008 — Job Cancellation

**WHEN** a user cancels a queued or active job  
**THEN** the system shall cancel or terminate the job safely.

### Acceptance Criteria

- A queued job changes to `CANCELLED` without being executed.
- An active job receives a termination request.
- The system records who requested the cancellation.
- The cancellation timestamp is recorded.
- Logs generated before cancellation are preserved.
- Resources are released after cancellation.
- The assigned initiator becomes available after cleanup.
- The Job Monitor displays the cancelled job in a cancelled or output panel.

---

## FFW-JES-009 — Job Logs

**WHEN** a user requests job output  
**THEN** the system shall display the complete execution log.

### Acceptance Criteria

The system supports viewing:

- JES-style job log
- Step log
- Application log
- SYSOUT/spool output
- Error output
- Allocation messages
- Dataset resolution messages

Additional criteria:

- Logs are viewable for active jobs where output is available.
- Logs are viewable for completed, failed, and cancelled jobs.
- Logs support search within output.
- Logs support copy/export.
- Logs are stored in a stable format.

---

## FFW-JES-010 — Retained Output

**AFTER** a job completes  
**THE SYSTEM SHALL** retain job output according to configured retention rules.

### Acceptance Criteria

- Retention rules are configurable.
- Manual purge is supported.
- Automatic purge is supported.
- Purging a job removes retained logs according to policy.
- Purging a job does not remove catalogued datasets unless explicitly requested.
- The user receives a warning before destructive purge actions.

---

# 2. Dataset Catalog Requirements

## FFW-CAT-001 — Dataset Catalog

**WHERE** datasets are used  
**THE SYSTEM SHALL** maintain a dataset catalog.

### Acceptance Criteria

The catalog stores metadata including:

- Dataset name / DSN
- Physical file path
- Dataset type
- Record format
- Logical record length / LRECL
- Block size, where applicable
- Organisation
- Encoding
- Creation timestamp
- Last modified timestamp
- Owning project or workspace, where applicable

---

## FFW-CAT-002 — Dataset Resolution

**WHEN** a job references `DSN=my.dataset`  
**THEN** the system shall resolve the DSN through the local dataset catalog.

### Acceptance Criteria

- The DSN is parsed from the job definition.
- The catalog is queried for the matching DSN.
- The catalog returns the physical file path.
- If the DSN is not found, the job fails validation or allocation according to configured rules.
- Resolution messages are written to the job log.
- Dataset resolution works consistently on Windows, Linux, and macOS.

### Example Resolution

```text
DSN=CORP.PAYROLL.MASTER
```

May resolve to a Linux/macOS path:

```text
/home/catalog/CORP/PAYROLL/MASTER.dat
```

Or to a Windows path:

```text
C:\Catalog\CORP\PAYROLL\MASTER.dat
```

---

## FFW-CAT-003 — Dataset Creation

**WHEN** a new dataset is allocated  
**THEN** a catalog entry shall be created.

### Acceptance Criteria

- Dataset name uniqueness is validated.
- Invalid dataset names are rejected.
- The target physical path is created where required.
- Required parent directories are created where allowed.
- The physical file is created according to allocation parameters.
- Metadata is stored in the catalog.
- A catalog creation message is written to the job log.

---

## FFW-CAT-004 — Dataset Browser

**WHEN** a user opens Dataset Explorer  
**THEN** the catalog hierarchy shall be displayed.

### Acceptance Criteria

- The catalog is displayed as a hierarchical tree.
- Dataset qualifiers are represented as tree levels.
- The user can search for a dataset by DSN.
- The user can open a dataset in FileForge Workbench.
- The user can view dataset metadata.
- The user can reveal the physical file location.

### Example Tree

```text
CORP
 ├─ PAYROLL
 │   ├─ MASTER
 │   └─ HISTORY
 └─ HR
     └─ EMPLOYEE
```

---

## FFW-CAT-005 — Generation Dataset Support

**WHEN** a dataset is defined as a generation dataset  
**THEN** generation management shall be supported.

### Acceptance Criteria

The system supports generation references such as:

```text
MY.FILE.GDG(+1)
MY.FILE.GDG(0)
MY.FILE.GDG(-1)
```

Additional criteria:

- The current generation can be resolved.
- A new generation can be allocated.
- Previous generations can be retained according to limit rules.
- Generation metadata is visible in the Dataset Explorer.
- Invalid generation references produce clear validation messages.

---

# 3. SDSF-Style Emulator Requirements

## FFW-SDSF-001 — Job Monitor View

**WHEN** the Job Monitor opens  
**THEN** all job queues shall be displayed.

### Acceptance Criteria

The monitor provides panels for:

- Input Queue
- Active Jobs
- Held Jobs
- Output Jobs
- Failed Jobs
- Completed Jobs
- Cancelled Jobs

Additional criteria:

- Each panel shows job count.
- The user can switch between panels.
- The user can open job details from any panel.
- The user can open job logs from any applicable panel.

---

## FFW-SDSF-002 — Job Filtering

**WHEN** filters are applied  
**THEN** only matching jobs shall be displayed.

### Acceptance Criteria

The user can filter jobs by:

- Owner/User
- Job Name
- Job ID
- Status
- Submit Date
- Start Date
- End Date
- Return Code
- Queue

Additional criteria:

- Filters can be cleared.
- Filtering does not alter stored job state.
- Filter results update when job state changes.

---

## FFW-SDSF-003 — View SYSOUT

**WHEN** a user selects a completed, failed, or cancelled job  
**THEN** SYSOUT-style output shall be displayed.

### Acceptance Criteria

- Output is displayed in a dedicated viewer.
- Output may be separated into tabs or sections.
- The viewer supports search.
- The viewer supports copy.
- The viewer supports export.
- The viewer supports save as.
- Large logs can be opened without freezing the application.

---

## FFW-SDSF-004 — Real-Time Refresh

**WHILE** the monitor is open  
**THE SYSTEM SHALL** refresh job status automatically.

### Acceptance Criteria

- Refresh interval is configurable.
- Push-style updates are preferred where technically feasible.
- Manual refresh remains available.
- Refresh does not reset user-selected filters.
- Refresh does not collapse expanded tree nodes unnecessarily.
- Refresh does not interrupt log viewing.

---

# 4. Plugin and Integration Requirements

## FFW-PLG-001 — FileForge Workbench Integration

**WHEN** FileForge Workbench loads  
**THEN** the Job Entry Subsystem shall load as a plugin.

### Acceptance Criteria

- The subsystem is packaged as a FileForge Workbench plugin.
- The plugin can be enabled or disabled.
- The plugin supports independent versioning.
- The plugin can expose menu entries, panels, and commands.
- The plugin does not require changes to unrelated FileForge Workbench modules.

---

## FFW-PLG-002 — Dataset API

**THE SYSTEM SHALL** expose Dataset Catalog APIs.

### Acceptance Criteria

Other FileForge Workbench components and plugins can:

- Allocate a dataset.
- Read a dataset.
- Write a dataset.
- Delete a dataset.
- Resolve a DSN to a physical file path.
- Query dataset metadata.
- Open a dataset in the FileForge editor.

---

## FFW-PLG-003 — Job API

**THE SYSTEM SHALL** expose job management APIs.

### Acceptance Criteria

Other FileForge Workbench components and plugins can:

- Submit a job.
- Hold a job.
- Release a job.
- Cancel a job.
- Query job status.
- Retrieve job logs.
- Retrieve job output.
- Subscribe to job status changes.

---

# 5. Future Phase: Mainframe and Remote Connectivity

## FFW-JES-FUT-001 — Provider Abstraction

**WHERE** jobs may originate from different execution environments  
**THE SYSTEM SHALL** define a provider abstraction for job queues and job logs.

### Acceptance Criteria

- Desktop jobs use the same monitor abstraction as remote jobs.
- The Job Monitor can support multiple providers.
- Providers can be added without redesigning the monitor.
- Provider-specific details are isolated behind provider interfaces.

### Candidate Providers

```text
Job Provider Interface

  Desktop JES Provider
  z/OS JES Provider
  USS Provider
  Linux Batch Provider
  Windows Batch Provider
```

---

## FFW-JES-FUT-002 — Unified Job Monitor

**WHEN** multiple job providers are configured  
**THEN** the system shall display jobs from each provider in a unified SDSF-style monitor.

### Acceptance Criteria

The monitor can display:

- Desktop JES Queue
- Mainframe JES Queue
- Remote Linux Queue
- Remote Windows Queue

Additional criteria:

- The provider source is visible for each job.
- Filtering by provider is supported.
- Job actions are limited to actions supported by the relevant provider.
- Provider connection errors are visible without crashing the monitor.

---

# 6. Recommended Module Structure

The subsystem should be implemented as a separate FileForge Workbench plugin named:

```text
ffw-jes
```

Recommended submodules:

```text
ffw-jes-core
ffw-jes-scheduler
ffw-jes-catalog
ffw-jes-monitor
ffw-jes-api
```

## Module Responsibilities

| Module | Responsibility |
|---|---|
| `ffw-jes-core` | Core job model, queue model, status model, persistence contracts |
| `ffw-jes-scheduler` | Scheduling rules, initiator pool, dispatch, lifecycle transitions |
| `ffw-jes-catalog` | Dataset catalog, DSN resolution, GDG support, metadata storage |
| `ffw-jes-monitor` | SDSF-style user interface, queue panels, log viewer |
| `ffw-jes-api` | Plugin-facing APIs for job and dataset operations |

---

# 7. Suggested Job Lifecycle

```text
SUBMITTED
   |
   v
QUEUED -----> HELD
   |            |
   |            v
   |         RELEASED
   |            |
   v            |
ACTIVE <--------
   |
   +-----> COMPLETED
   |
   +-----> FAILED
   |
   +-----> CANCELLED
```

---

# 8. Suggested Minimum Viable Product Scope

## MVP-1: Local Queue and Monitor

- Submit local job definition.
- Queue job.
- Display queued jobs.
- Run jobs using a configurable worker pool.
- Display active jobs.
- Mark jobs as completed or failed.
- View job logs.

## MVP-2: Dataset Catalog

- Create dataset catalog entries.
- Resolve DSN to physical path.
- Browse datasets.
- Open catalogued files in FileForge Workbench.

## MVP-3: SDSF-Style Experience

- Input, Active, Output, Held, Failed, and Completed panels.
- Filtering and sorting.
- Live refresh.
- SYSOUT-style log viewer.

## MVP-4: Plugin Stabilisation

- Define plugin interface.
- Expose Job API.
- Expose Dataset API.
- Package as `ffw-jes` plugin.

---

# 9. Design Notes

- The subsystem should emulate mainframe behaviour conceptually, not necessarily reproduce every z/OS implementation detail.
- Job state transitions should be explicit and auditable.
- Dataset resolution should be deterministic and logged.
- Job logs should be retained independently from physical output datasets.
- The architecture should allow future providers for real mainframe JES, USS, Linux, or Windows execution environments.
- The initial desktop implementation should remain fully cross-platform.

---

# 10. Glossary

| Term | Meaning |
|---|---|
| JES | Job Entry Subsystem; mainframe subsystem responsible for managing batch job input, execution, and output queues. |
| SDSF | System Display and Search Facility; mainframe interface commonly used to monitor and manage jobs. |
| Initiator | Mainframe execution resource that selects eligible jobs from the queue and starts them. |
| Worker | Desktop equivalent of an initiator. |
| DSN | Dataset Name; mainframe-style logical dataset identifier. |
| Dataset Catalog | Desktop metadata store that maps logical DSNs to physical files. |
| SYSOUT | System output produced by a batch job. |
| Job Log | Execution log containing scheduling, allocation, step, and return-code messages. |
| GDG | Generation Data Group; a dataset concept that supports multiple generations of the same logical dataset. |

---

# 11. Summary

The FFW-JES subsystem provides a practical and extensible Windows, Linux, and macOS emulator for mainframe-style job submission, queue monitoring, initiator-based execution, dataset cataloguing, and SDSF-style job log viewing. It should be designed as a FileForge Workbench plugin so that job queues, dataset catalogues, job logs, and future remote providers can all be integrated into the broader FileForge editing and workbench environment.
