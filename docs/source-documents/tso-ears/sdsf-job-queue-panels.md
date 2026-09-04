# SDSF Job Queue Panels -- EARS Requirements

Source documents: ikja100 (SDSF User Guide) -- panels I, O, H, ST, DA, JDS, JS.

Priority: P1 -- Core Emulation.
Sub-project mapping: FFW-JES (primary), dataset-catalog, virtual-file-system (secondary).

---

## Requirement SDSF-JQ-1: Input Queue Panel (I)

WHEN the user enters the I command,
THE workbench SHALL display the Input Queue panel showing jobs awaiting execution.

Criteria:
- 1.1 THE Input Queue panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Class, Position, PrtDest, Remote, Node.
- 1.2 THE panel SHALL support filtering by PREFIX, DEST, OWNER, SYSNAME.
- 1.3 THE panel SHALL support action characters: C (Cancel), H (Hold), A (Release), D (Display), S (Browse), ? (Job Data Sets), E (Edit/Restart), J (Start), P (Purge).
- 1.4 THE panel SHALL display arrival time (ARRTIME) and current queue time (CQTIME) columns.
- 1.5 THE panel SHALL display execution start and end times when available.

---

## Requirement SDSF-JQ-2: Output Queue Panel (O)

WHEN the user enters the O command,
THE workbench SHALL display the Output Queue panel showing jobs with output awaiting printing.

Criteria:
- 2.1 THE Output Queue panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Class, Position, PrtDest, Remote, Node, Max-CC.
- 2.2 THE panel SHALL support action characters: P (Purge), S (Browse output), X (Print), D (Display), H (Hold), A (Release).
- 2.3 THE panel SHALL support the ? action character to display the Job Data Set panel for a job.

---

## Requirement SDSF-JQ-3: Held Output Panel (H)

WHEN the user enters the H command,
THE workbench SHALL display the Held Output panel showing jobs with held output.

Criteria:
- 3.1 THE Held Output panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Class, Position, PrtDest, Remote, Node, Max-CC, ResGroup.
- 3.2 THE panel SHALL support action characters: A (Release), P (Purge), S (Browse), X (Print), D (Display), ? (Job Data Sets).
- 3.3 THE panel SHALL display execution start and end times.

---

## Requirement SDSF-JQ-4: Status Panel (ST)

WHEN the user enters the ST command,
THE workbench SHALL display the Status panel showing all jobs in all queues.

Criteria:
- 4.1 THE Status panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Queue, Class, Position, SAff, ASys, Status, Max-CC, ResGroup.
- 4.2 THE panel SHALL support all action characters available on the I, O, and H panels.
- 4.3 THE panel SHALL support the JRL action character to display resource limits for a job.
- 4.4 THE panel SHALL display arrival time, current queue time, execution start time, and execution end time.
- 4.5 THE panel SHALL display LimitsImpact and LimitsRaised indicators.
- 4.6 THE panel SHALL support the JESCANCEL overtypeable column to control JES cancel options.

---

## Requirement SDSF-JQ-5: Display Active Users Panel (DA)

WHEN the user enters the DA command,
THE workbench SHALL display the Display Active Users panel showing all active address spaces.

Criteria:
- 5.1 THE DA panel SHALL display columns: NP, JOBNAME, StepName, ProcStep, JobID, Owner, Class, Position, DP, Real, Paging, SIO, CPU%, ElapsedTime, OutTime.
- 5.2 THE panel SHALL support action characters: C (Cancel), D (Display), S (Browse), ? (Job Data Sets), JS (Job Steps), JM (Job Memory), JD (Job Devices), JDD (Job DDNames), JT (Job Tasks).
- 5.3 THE panel title line SHALL display system name, paging rate, CPU utilization, and zIIP utilization.
- 5.4 THE panel SHALL support the FJ action character to fetch module information by job name.
- 5.5 THE panel SHALL support the JCM action character to list job common memory objects.
- 5.6 THE panel SHALL support the LE action character to list enclaves.
- 5.7 THE panel SHALL support the LU action character to list user ID information.

---

## Requirement SDSF-JQ-6: Job Data Set Panel (JDS)

WHEN the user types ? in the NP column of a job on any queue panel,
THE workbench SHALL display the Job Data Set panel for that job.

Criteria:
- 6.1 THE JDS panel SHALL display all SYSOUT data sets for the job with columns: NP, DDName, StepName, ProcStep, DSName, Class, Dest, Copies, Records, Bytes.
- 6.2 THE panel SHALL support action characters: S (Browse), SB (ISPF Browse), SE (ISPF Edit), X (Print), P (Purge).
- 6.3 THE panel SHALL display job name and job ID in the title line.

---

## Requirement SDSF-JQ-7: Job Step Panel (JS)

WHEN the user types JS in the NP column of a job,
THE workbench SHALL display the Job Step panel for that job.

Criteria:
- 7.1 THE JS panel SHALL display all steps for the job with columns: NP, StepName, ProcStep, Program, StepCC, StartTime, EndTime, CPU, SIO.
- 7.2 THE panel SHALL support the S action character to browse step output.
