# New Requirements Gate — MANDATORY BEFORE ANY CODE CHANGE

When the user provides a new requirement, a feature request, or a change request of any kind,
the following gate sequence MUST be completed in full before touching any source file.

## Gate Sequence

```
1. IDENTIFY   the correct sub-project(s) under docs/specs/
2. REQUIREMENTS  write or update docs/specs/<sub-project>/requirements.md
3. DESIGN     write or update docs/specs/<sub-project>/design.md
4. TASKS      write or update docs/specs/<sub-project>/tasks.md
5. MASTER     add the new tasks to docs/specs/project-master/tasks.md
6. TCR add a 🔴 NOT COVERED row to docs/quality/TCR.md for each new criterion
7. CONFIRM    show the user the completed documentation and ask for approval
8. CODE       only now write failing tests, then implementation
```

**No file outside `docs/` may be created or modified until step 7 is complete.**

---

## Step-by-step rules

### Step 1 — Identify the sub-project

- Read `docs/specs/` to find the sub-project whose scope covers the new requirement.
- If no existing sub-project fits, create a new folder `docs/specs/<new-sub-project>/`
  and add it to the sub-project list in `.amazonq/rules/specs.md`.
- A single requirement may touch more than one sub-project — update all of them.

### Step 2 — requirements.md

- Use EARS format for every acceptance criterion: `WHEN … THE … SHALL …`
- Number new criteria sequentially within the requirement (e.g. Requirement 14, criteria 14.1–14.8).
- If the change adjusts an existing criterion, edit it in place and note the change inline.
- If a new sub-project folder is created, write the full `requirements.md` from scratch
  including an Introduction, Glossary, and numbered Requirements sections.
- This file is the **source of truth** — code must never get ahead of it.

### Step 3 — design.md

- Read the existing `design.md` before writing anything.
- Add a new section for the feature if it introduces new architectural decisions
  (new modules, new data flows, new egui panels, new crate dependencies).
- If the design is unchanged (e.g. a pure data-model addition), note "No design changes required"
  in a brief section rather than leaving the file untouched.
- Never contradict an existing architectural decision without explicitly calling it out
  and explaining the reason for the change.

### Step 4 — tasks.md (sub-project)

- Each task must be a concrete, independently completable unit of work.
- Use `[ ]` for all new tasks — never pre-mark them `[x]`.
- Number tasks sequentially continuing from the last existing task number.
- Each task must reference the requirement criterion(a) it satisfies.
- Follow the format rules in `.amazonq/rules/spec-task-format.md`.

### Step 5 — project-master/tasks.md

- Add a new Phase section (or extend an existing active phase) for the new work.
- One line per logical deliverable — not one line per sub-task.
- Use `[ ]` for all new entries.
- Update the Summary counts table at the bottom.

### Step 6 — docs/quality/TCR.md

- Add one `🔴 NOT COVERED` row per new acceptance criterion.
- Format: `| \`ff-desktop\` | 🔴 | — | Req X.Y: <one-line description> |`
- Place rows in the correct crate section.

### Step 7 — Confirm with the user

- Present a summary of all documentation changes made.
- List the new requirement numbers and their criteria.
- List the new tasks added to project-master/tasks.md.
- **Wait for explicit user approval before writing any code.**

### Step 8 — Code (TDD)

- Follow the TDD cycle in `.amazonq/rules/tdd-and-testing.md`.
- Follow the full gate sequence in `.amazonq/rules/rust-coding-standards.md`.

---

## What counts as a "new requirement"

- Any feature the user describes that does not already have a criterion in `requirements.md`
- Any change to existing behaviour (even a "small tweak")
- Any bug fix that reveals a missing or incorrect criterion
- Any UI change (new panel, new menu item, new keyboard shortcut, new screen layout)
- Any new crate dependency or architectural addition

## What does NOT require this gate

- Refactoring that changes no observable behaviour and has existing test coverage
- Fixing a failing test where the criterion already exists in `requirements.md`
- Updating comments, documentation strings, or formatting only

---

## Violations

Skipping any step is a violation of project standards. If a previous session skipped steps,
correct the documentation before continuing with implementation.
