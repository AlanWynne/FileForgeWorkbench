# Operating Mode -- Code

Applies when the prompt is classified as TASK / IMPLEMENTATION or REFACTOR.
The requirements gate has already been completed and a task is explicitly approved.

## Execution Loop

Work autonomously within the current approved task. Do not stop after describing
the next step -- perform the next safe step when the required tool is available.

For each task:

1. Read the referenced requirements and acceptance criteria from
   `docs/specs/<sub-project>/requirements.md`.
2. Inspect the existing implementation and tests relevant to the task.
3. Produce a concise implementation plan (bullet list, no prose padding).
4. Write or update the failing tests before writing any implementation code.
   Every test must carry `// Validates: Requirement X.Y`.
5. Make the smallest coherent change that satisfies the acceptance criteria.
6. Run formatting, compilation, linting, and the relevant test suites:
   ```
   cargo fmt
   cargo check
   cargo clippy -- -D warnings
   cargo test -p <crate>
   ```
7. If validation fails, diagnose the failure, correct the implementation,
   and rerun validation.
8. Continue the repair loop while the failure is attributable to the current
   change and progress is being made.
9. Update `docs/quality/TCR.md` -- set each covered row to the correct status.
10. Record a concise summary: requirements implemented, files changed, commands
    executed, test results, known limitations, recommended follow-up.
11. Run `C:\workspace\VSC\FileForgeWorkbench\tools\powershell\verify.ps1`.
    (verify.ps1 deletes ai-review.log before running -- no separate delete step needed.)
12. While `C:\workspace\VSC\FileForgeWorkbench\tools\logs\ai-review.log`
    contains errors:
    12.1 Fix every problem reported in `ai-review.log`.
    12.2 Run `C:\workspace\VSC\FileForgeWorkbench\tools\powershell\verify.ps1`.
13. Compact history.
14. Stop -- describe what was done and what is next.

## Definition of Done

A task is complete only when:

- all acceptance criteria are satisfied;
- `cargo fmt -- --check` passes;
- `cargo build` succeeds;
- `cargo clippy -- -D warnings` produces no new warnings;
- all relevant unit and integration tests pass;
- `docs/quality/TCR.md` is updated for every criterion touched;
- documentation is updated where observable behaviour changed;
- no secrets, generated artefacts, or unrelated changes are included;
- `verify.ps1` exits with no errors.

## Mandatory Stop Conditions

Stop and request human review when:

- requirements are contradictory or materially ambiguous;
- a public interface or persisted data format must change;
- an architecture decision is required;
- a dependency licence is uncertain;
- a security boundary or cryptographic behaviour changes;
- destructive file or database operations would be required;
- credentials or production access are required;
- an existing test appears incorrect (do not modify it -- report it);
- the same failure remains after three materially different repair attempts;
- the task requires changes outside the allowed paths;
- the next task has not been explicitly marked ready.

## Prohibited Actions

Do not:

- disable or delete tests to obtain a passing build;
- weaken assertions without requirement evidence;
- suppress compiler or linter warnings without a justification comment;
- modify requirements to match the produced implementation;
- commit secrets or credentials;
- merge, publish, release, or deploy;
- change licensing files without human approval;
- select a new phase or architectural direction independently.
