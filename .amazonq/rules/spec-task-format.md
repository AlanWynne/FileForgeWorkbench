# Spec and Task File Format

When generating or editing `tasks.md` files inside `.kiro/specs/`, follow these rules strictly.

## Allowed Checkbox Markers

Only two states are valid:

| Marker | Meaning |
|--------|---------|
| `[ ]`  | Pending / not started |
| `[x]`  | Completed |

Do NOT use `[~]`, `[-]`, `[/]`, or any other symbol — they cause tasks to display as "untitled task".

## Task Title Rule

Every task line MUST have descriptive title text immediately after the number:

```markdown
- [ ] 3. Implement persistence layer
  - [ ] 3.1 Create persistence module with load/save logic
```

Never leave a task line without a title.

## Tracking Intermediate States

- In-progress: leave as `[ ]` — the first unchecked task in sequence is implicitly active
- Blocked: add a note on a sub-bullet:
  ```markdown
  - [ ] 5.2 Write property test for undock position
    - ⚠️ BLOCKED: waiting on egui viewport API stabilisation
  ```
- Partially complete: split into smaller subtasks so each can be individually checked off

## Summary Checklist

- Every checkbox uses `[ ]` or `[x]` only
- Every task line has a human-readable title after the number
- Top-level tasks have a concise summary title
- Subtasks have specific action descriptions
- Status notes go on indented sub-bullets, never inside the checkbox brackets
