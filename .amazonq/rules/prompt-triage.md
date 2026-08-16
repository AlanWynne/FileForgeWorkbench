# Prompt Triage — MANDATORY FOR EVERY PROMPT

Before responding to any user prompt, classify it into exactly one of the
categories below and perform the corresponding logging action.
Do this silently — do not narrate the classification to the user unless they ask.

---

## Classification Categories

| Category | Trigger signals |
|----------|----------------|
| **BUG** | Words like "not working", "broken", "crash", "wrong", "doesn't", "stopped", "freeze", "error", "unexpected", "regression", or a description of observed behaviour that contradicts a requirement |
| **NEW REQUIREMENT** | Words like "add", "new feature", "we need", "can we have", "implement", "support for", "I want", "please create", or any capability that does not exist yet |
| **CHANGE REQUEST** | Modification to existing behaviour that already works — "change", "rename", "move", "tweak", "adjust", "instead of", "rather than" |
| **QUESTION / DISCUSSION** | Pure information request — "how does", "what is", "explain", "why", "show me", no code change implied |
| **TASK / IMPLEMENTATION** | Follow-up work explicitly approved after a requirements gate — "implement", "code it", "write the test", "do it" after a gate has been completed |
| **REFACTOR** | Code quality improvement with no observable behaviour change |

---

## Logging Rules

### BUG → append to `docs/bugs.md`

Append a new row to the Bug Table using the next available `B###` ID:

```
| B### | OPEN | <Severity> | `<component>` | <one-line description> | <Linked Req or —> | — | Reported via prompt: "<first 60 chars of user prompt>" |
```

Severity guide:
- `Critical` — crash or data loss
- `High` — feature completely broken
- `Medium` — partial breakage or awkward workaround
- `Low` — cosmetic or minor inconvenience

Also append a row to the `docs/bugs.md` Changelog section:
```
| <Phase or date> | B### added — <short description> |
```

### NEW REQUIREMENT → append to `docs/change-log.md`

Append a new entry under the `## New Requirements` section:

```
### CR-NR-### — <short title>
- **Date/Phase**: <current phase label, e.g. "Phase AS">
- **Prompt**: "<first 80 chars of user prompt>"
- **Description**: <one or two sentences describing what was asked for>
- **Status**: PENDING GATE  ← changes to IN PROGRESS once gate starts, DONE once merged
- **Linked spec**: `docs/specs/<sub-project>/requirements.md` (to be created/updated)
```

Then follow the full gate in `.amazonq/rules/new-requirements-gate.md`.

### CHANGE REQUEST → append to `docs/change-log.md`

Append a new entry under the `## Change Requests` section:

```
### CR-CH-### — <short title>
- **Date/Phase**: <current phase label>
- **Prompt**: "<first 80 chars of user prompt>"
- **Description**: <one or two sentences>
- **Affects**: <crate(s) or component(s)>
- **Status**: PENDING GATE
```

Then follow the full gate in `.amazonq/rules/new-requirements-gate.md`
(change requests that alter observable behaviour require the same gate as new requirements).

### QUESTION / DISCUSSION → no logging required

Answer directly. No file changes unless the answer reveals a bug or gap.

### TASK / IMPLEMENTATION → no new log entry

The requirement or change request was already logged when the gate was run.
Proceed directly to TDD implementation per `.amazonq/rules/tdd-and-testing.md`.

### REFACTOR → no new log entry unless behaviour changes

If during refactoring a behaviour change is discovered, reclassify as CHANGE REQUEST
and log it before proceeding.

---

## Determining the Next ID

Before appending:
- For bugs: read `docs/bugs.md`, find the highest `B###` number, increment by 1.
- For new requirements: read `docs/change-log.md`, find the highest `CR-NR-###`, increment by 1.
- For change requests: read `docs/change-log.md`, find the highest `CR-CH-###`, increment by 1.

---

## Edge Cases

- A single prompt may contain **both a bug report and a new requirement** — log both.
- If classification is ambiguous, prefer the more conservative category
  (BUG > CHANGE REQUEST > NEW REQUIREMENT) to avoid skipping the gate.
- If a bug fix reveals a missing acceptance criterion, also log a NEW REQUIREMENT entry
  and run the gate before fixing the code.
- QUESTION prompts that end with "can we add X?" should be reclassified as NEW REQUIREMENT.

---

## What NOT to do

- Do not skip logging because the fix is "small" or "obvious".
- Do not log QUESTION / DISCUSSION prompts as bugs or requirements.
- Do not create a new log entry for a prompt that is a direct follow-up
  to an already-logged and gate-approved item (TASK / IMPLEMENTATION category).
