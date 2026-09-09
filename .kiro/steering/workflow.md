---
inclusion: always
---

# Development Workflow -- Triage, Requirements Gate, Operating Modes

This is the canonical process for turning any prompt into shipped code.
It merges four concerns: prompt triage, the requirements gate, the
specification operating mode, and the code operating mode.

Related steering: `rust-standards.md`, `testing.md`, `specs.md`,
`documentation.md`, `tooling.md`.

---

## 1. Prompt Triage -- MANDATORY FOR EVERY PROMPT

Before responding, classify the prompt into exactly one category and perform
the logging action. Do this silently unless the user asks.

| Category | Trigger signals | Action |
|----------|-----------------|--------|
| **BUG** | "not working", "broken", "crash", "wrong", "doesn't", "stopped", "freeze", "error", "unexpected", "regression", or behaviour that contradicts a requirement | Log to `docs/status/bugs.md`, then diagnose read-only first |
| **NEW REQUIREMENT** | "add", "new feature", "we need", "can we have", "implement", "support for", "I want", "please create", or any capability that does not exist yet | Log to `docs/status/change-log.md`, then run the gate (section 2) |
| **CHANGE REQUEST** | Modification to working behaviour -- "change", "rename", "move", "tweak", "adjust", "instead of", "rather than" | Log to `docs/status/change-log.md`, then run the gate (section 2) |
| **QUESTION / DISCUSSION** | "how does", "what is", "explain", "why", "show me" -- no code change implied | Answer directly; no logging, no operating mode |
| **TASK / IMPLEMENTATION** | Approved follow-up after a gate -- "implement", "code it", "write the test", "do it" | No new log entry; go to code mode (section 4) |
| **REFACTOR** | Code-quality change, no observable behaviour change | No log unless behaviour changes; use code mode (section 4) |

### Logging formats

BUG -- append to the `docs/status/bugs.md` Bug Table (next `B###` id):
```
| B### | OPEN | <Severity> | `<component>` | <one-line description> | <Linked Req or --> | -- | Reported via prompt: "<first 60 chars>" |
```
Severity: `Critical` (crash/data loss), `High` (feature broken), `Medium`
(partial breakage), `Low` (cosmetic). Also add a Changelog row:
`| <Phase or date> | B### added -- <short description> |`

NEW REQUIREMENT -- append under `## New Requirements` in `docs/status/change-log.md`:
```
### CR-NR-### -- <short title>
- **Date/Phase**: <phase label>
- **Prompt**: "<first 80 chars>"
- **Description**: <one or two sentences>
- **Status**: PENDING GATE  <- IN PROGRESS when gate starts, DONE when merged
- **Linked spec**: `docs/specs/<sub-project>/requirements.md`
```

CHANGE REQUEST -- append under `## Change Requests`:
```
### CR-CH-### -- <short title>
- **Date/Phase**: <phase label>
- **Prompt**: "<first 80 chars>"
- **Description**: <one or two sentences>
- **Affects**: <crate(s) or component(s)>
- **Status**: PENDING GATE
```

Next id: read the relevant file, find the highest `B###` / `CR-NR-###` /
`CR-CH-###`, increment by 1.

### Triage edge cases

- A prompt may contain both a bug and a new requirement -- log both.
- If ambiguous, prefer the more conservative category (BUG > CHANGE REQUEST >
  NEW REQUIREMENT) so the gate is never skipped.
- A bug fix that reveals a missing criterion also logs a NEW REQUIREMENT and
  runs the gate before the code is fixed.
- "can we add X?" questions are reclassified as NEW REQUIREMENT.
- Do not log because a fix is "small"; do not log QUESTION prompts; do not log a
  direct follow-up to an already-gated item.

---

## 2. Requirements Gate -- MANDATORY BEFORE ANY CODE CHANGE

For any NEW REQUIREMENT or CHANGE REQUEST, complete this sequence in full before
touching any source file. No file outside `docs/` may change until step 7.

```
1. IDENTIFY       the correct sub-project(s) under docs/specs/
2. REQUIREMENTS   write/update docs/specs/<sub-project>/requirements.md
3. DESIGN         write/update docs/specs/<sub-project>/design.md
4. TASKS          write/update docs/specs/<sub-project>/tasks.md
5. MASTER         add the new tasks to docs/specs/project-master/tasks.md
6. TCR            add a NOT COVERED row to docs/quality/TCR.md per new criterion
7. CONFIRM        show the user the docs and wait for approval
8. CODE           only now write failing tests, then implementation
```

Step details:
- **Identify**: read `docs/specs/`; if no sub-project fits, create
  `docs/specs/<new-sub-project>/` and add it to the list in `specs.md`. One
  requirement may touch several sub-projects -- update all.
- **requirements.md**: EARS format `WHEN ... THE ... SHALL ...`; number criteria
  sequentially (e.g. Requirement 14, criteria 14.1-14.8); edit existing criteria
  in place and note the change. For a new folder, write the full file
  (Introduction, Glossary, numbered Requirements). This file is the source of
  truth -- code must never get ahead of it.
- **design.md**: read it first; add a section for new architectural decisions
  (modules, data flows, egui panels, crate deps). If unchanged, write "No design
  changes required". Never contradict an existing decision without calling it out.
- **tasks.md**: concrete, independently completable tasks; `[ ]` only, never
  pre-mark `[x]`; number continuing from the last; reference the criterion(a)
  satisfied; follow the format in `specs.md`.
- **project-master/tasks.md**: add/extend a Phase section, one line per logical
  deliverable, `[ ]` only, update the Summary counts.
- **TCR**: one NOT COVERED row per new criterion, in the correct crate section:
  `` | `ff-desktop` | 🔴 | -- | Req X.Y: <one-line description> | ``
- **Confirm**: summarise all doc changes, new requirement numbers/criteria, and
  new master tasks. Wait for explicit approval before any code.

### What counts as a new requirement
Any feature with no existing criterion; any change to existing behaviour (even a
"small tweak"); any bug fix revealing a missing/incorrect criterion; any UI
change (panel, menu item, shortcut, layout); any new crate or architectural add.

### What does NOT need the gate
Refactors with no observable behaviour change and existing coverage; fixing a
failing test where the criterion already exists; comment/doc/formatting-only edits.

Skipping any step is a violation. If a prior session skipped steps, correct the
documentation before continuing.

---

## 3. Operating Mode -- Specification (NEW REQUIREMENT / CHANGE REQUEST)

Detailed procedure for the specification steps of the gate. No source file
outside `docs/` may change until step 9.

1. Read `requirements.md` and `design.md` for every sub-project touched.
2. Read related source to understand current behaviour -- verify, do not assume.
3. Check for contradictions with existing requirements across all sub-projects.
4. Draft acceptance criteria in EARS format, numbered from the last existing.
5. Draft the design delta (modules, data flows, deps, panels) or "No design
   changes required".
6. Draft the task list -- independently completable, cross-referencing criteria,
   `[ ]` only.
7. Draft the `project-master/tasks.md` additions and the `TCR.md` NOT COVERED rows.
8. Check every drafted file for prohibited characters per `documentation.md`.
9. Present the complete draft: requirement numbers/criteria, design decisions
   made or deferred, master tasks added, TCR rows to add.
10. Wait for explicit approval. Write nothing before approval.
11. Write the approved files: `requirements.md`, `design.md`, `tasks.md`,
    `project-master/tasks.md`, `TCR.md`, and `change-log.md` if not already updated.
12. Confirm each file written.
13. Stop -- do not implement without a separate TASK / IMPLEMENTATION instruction.

Definition of done (spec): every criterion is EARS and numbered; none contradicts
an existing one; design reflects new decisions (or notes none); tasks are
independently completable and cross-reference criteria; master and TCR updated;
all files pass the character check; user approved; no source outside `docs/` changed.

Stop for human review when: a new requirement contradicts an existing one; scope
touches more than three sub-projects; a requirement implies an architecture
decision; the requirement is ambiguous and unresolved by existing specs;
constraints conflict.

Do not: write source before step 13; mark tasks `[x]` in a draft; invent criteria
not derivable from the request; modify passing criteria to fit a new requirement;
use prohibited characters; proceed to code in the same session without a separate
instruction.

---

## 4. Operating Mode -- Code (TASK / IMPLEMENTATION / REFACTOR)

The gate is complete and a task is approved. Work autonomously within the task;
perform the next safe step rather than only describing it.

1. Read the referenced requirements and criteria from `requirements.md`.
2. Inspect existing implementation and tests for the task.
3. Produce a concise implementation plan (bullets, no padding).
4. Write/update failing tests first; every test carries `// Validates: Requirement X.Y`.
5. Make the smallest coherent change that satisfies the criteria.
6. Run format, compile, lint, and the relevant tests using the commands in
   `testing.md` (`cargo fmt`, `cargo check`, `cargo clippy -- -D warnings`,
   `cargo test -p <crate>`).
7. If validation fails, diagnose, correct, and rerun.
8. Continue the repair loop while the failure is attributable to this change and
   progress is being made.
9. Update `docs/quality/TCR.md` -- set each covered row to its correct status.
10. Record a concise summary: requirements implemented, files changed, commands
    run, test results, known limitations, follow-ups.
11. Run `tools\powershell\verify.ps1` (fmt-check, clippy, and the test suite via
    cargo-nextest; it clears tools\logs\*.log first and accumulates any
    problems into ai-review.log). Use `-Fast` for a quick inner-loop signal only
    (see `testing.md`); the completion gate REQUIRES a clean full run without
    `-Fast` so proptests keep their >=100-iteration coverage.
12. While `tools\logs\ai-review.log` contains errors: fix every reported problem,
    then rerun `verify.ps1`.
13. Compact history.
14. Stop -- describe what was done and what is next.

Definition of done (code): all criteria satisfied; `cargo fmt -- --check` passes;
`cargo build` succeeds; `cargo clippy -- -D warnings` clean; all relevant tests
pass; TCR updated for every criterion touched; docs updated where behaviour
changed; no secrets/artefacts/unrelated changes; `verify.ps1` exits clean.

Stop for human review when: requirements are contradictory or materially
ambiguous; a public interface or persisted format must change; an architecture
decision is required; a dependency licence is uncertain; a security or crypto
boundary changes; destructive file/db operations would be needed; credentials or
production access are required; an existing test appears incorrect (report, do
not modify); the same failure survives three materially different repair attempts;
the task needs changes outside allowed paths; the next task is not marked ready.

Do not: disable/delete tests to pass a build; weaken assertions without evidence;
suppress warnings without a justification comment; modify requirements to match
code; commit secrets; merge/publish/release/deploy; change licensing files without
approval; pick a new phase or architectural direction independently.