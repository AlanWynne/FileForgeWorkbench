# Discussion: FTSO and MiniX Command Environment

**Status:** Future development discussion  
**Date captured:** 2026-09-03

## Summary

Explore a FileForgeWorkbench-native command environment inspired by IBM ISPF
Option 6 and TSO. The goal is not to emulate TSO literally or embed a generic
Bash/PowerShell terminal, but to provide a familiar mainframe-style command
experience for FileForgeWorkbench resources.

The proposed separation is:

- **FTSO (FileForge TSO):** The user-facing command shell and terminal
  experience.
- **MiniX:** A possible name for the underlying virtual runtime and service
  layer. The name is provisional because MINIX is also an existing operating
  system.

## Concept

FTSO would provide a prompt and command dispatcher for operations such as:

```text
FTSO> LISTCAT USER.*
FTSO> EDIT USER.COBOL(PROG1)
FTSO> SUBMIT USER.JCL(TESTJOB)
FTSO> JES STATUS
FTSO> SDSF
```

Commands would be provided through several layers:

1. Native FileForgeWorkbench commands for catalogs, datasets, members, and
   files.
2. Utility commands for searching, sorting, copying, comparing, and related
   operations.
3. A controlled host-command bridge with explicit security boundaries.
4. Plugin-provided commands for tools and integrations such as Git, SQL, AI,
   and ERI.

The underlying MiniX concept could provide catalog, dataset, GDG, VSAM, JES,
SDSF, security, scheduling, and command-processing services. The design should
also consider command history, multiple sessions, scripting, cancellation,
auditing, and record-aware handling of FB/VB datasets.

## Relationship to JCL execution

The JCL executor should make use of the same command framework and execution
services as FTSO. However, FTSO and JCL should not be implemented as one
function:

- **FTSO** parses and executes interactive commands entered by a user.
- **JCL** parses a job definition, validates job and DD statements, resolves
  datasets, applies step and condition semantics, and orchestrates execution
  through JES initiators.
- **Shared execution core** provides command registration, dispatch, execution
  context, results, errors, logging, cancellation, and access to platform
  services.

For example, a JCL `EXEC` step might invoke a registered FileForge command or a
provider-backed program, while the JCL executor remains responsible for job
step ordering, return codes, `COND`/`IF` processing, resource disposition, and
spool output. This keeps interactive command behavior consistent with batch
behavior without coupling the terminal UI to the batch scheduler.

The two capabilities should therefore be developed together around a shared
execution contract, but retained as separate modules or crates with explicit
interfaces.

## Why this belongs in future discussions

This concept crosses several existing project areas, including:

- `command-framework` and `command-semantics`
- dataset catalog and allocation
- IDCAMS and JES
- plugin architecture
- shell command execution
- session and layout management

It should remain a discussion item until the scope, naming, compatibility
goals, security model, and implementation phases are agreed. If approved, it
can later be split into focused specifications and ADRs under `docs/specs/`.

## Suggested next discussion

Decide whether the project should pursue:

- an FTSO command environment only,
- an FTSO shell backed by a MiniX-style service layer,
- or a smaller first release focused on dataset and job commands.
