# Operating Mode -- Specification

Applies when the prompt is classified as NEW REQUIREMENT or CHANGE REQUEST.
No source file outside `docs/` may be created or modified until step 9 is complete.

## Execution Loop

1. Read `docs/specs/<sub-project>/requirements.md` and `design.md` for every
   sub-project touched by the requirement.
2. Read the related source files to understand current behaviour -- do not
   assume; verify.
3. Check for contradictions with existing requirements across all affected
   sub-projects.
4. Draft acceptance criteria in EARS format: `WHEN ... THE ... SHALL ...`
   Number new criteria sequentially from the last existing number.
5. Draft the design delta -- new modules, data flows, crate dependencies,
   egui panels. If nothing changes architecturally, write
   "No design changes required" in a brief section.
6. Draft the task list -- each task independently completable, referencing
   the criterion(a) it satisfies. Use `[ ]` only; never pre-mark `[x]`.
7. Draft the `docs/specs/project-master/tasks.md` additions and the
   `docs/quality/TCR.md` rows (`🔴 NOT COVERED`) for each new criterion.
8. Check every drafted file for prohibited characters per
   `.amazonq/rules/documentation-ascii.md`.
9. Present the complete draft to the user:
   - new/changed requirement numbers and their criteria;
   - design decisions made or deferred;
   - new tasks added to project-master;
   - TCR rows to be added.
10. Wait for explicit user approval. Do not write any file before approval.
11. Write the approved documentation files:
    - `docs/specs/<sub-project>/requirements.md`
    - `docs/specs/<sub-project>/design.md`
    - `docs/specs/<sub-project>/tasks.md`
    - `docs/specs/project-master/tasks.md`
    - `docs/quality/TCR.md`
    - `docs/status/change-log.md` (if not already updated during triage)
12. Confirm each file written.
13. Stop -- do not proceed to implementation without a separate explicit
    instruction classified as TASK / IMPLEMENTATION.

## Definition of Done (Specification)

A specification task is complete only when:

- every acceptance criterion is in EARS format and numbered;
- no criterion contradicts an existing requirement;
- design.md reflects all new architectural decisions (or explicitly notes none);
- tasks.md entries are independently completable and cross-reference criteria;
- project-master/tasks.md is updated;
- TCR.md has a `🔴 NOT COVERED` row for every new criterion;
- all drafted files pass the prohibited-character check;
- the user has given explicit written approval;
- no source file outside `docs/` has been modified.

## Mandatory Stop Conditions

Stop and request human review when:

- a new requirement contradicts an existing one -- do not resolve silently;
- the scope touches more than three sub-projects -- confirm scope before drafting;
- a requirement implies an architecture decision (new crate, new external
  dependency, change to a public interface or persisted format);
- the requirement is ambiguous and cannot be resolved by reading existing specs;
- the user's description contains conflicting constraints.

## Prohibited Actions

Do not:

- write any source file (`.rs`, `Cargo.toml`, etc.) before step 13;
- mark any task `[x]` in the draft;
- invent acceptance criteria not derivable from the user's description;
- modify existing passing criteria to accommodate a new requirement;
- use prohibited characters (em dash, curly quotes, math symbols) in any
  drafted document -- see `.amazonq/rules/documentation-ascii.md`;
- proceed to code in the same session without an explicit separate instruction.
