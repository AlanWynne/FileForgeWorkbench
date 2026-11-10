---
inclusion: always
---

# Workspace Save Convention -- ISPF-Style Explicit Commit (B081)

Owner decision (standing): every editable Workspace Context follows the ISPF
convention of an EXPLICIT COMMIT against a working copy. There is NO
autosave-on-edit. This rule exists so that every screen/workspace behaves the
SAME way, and so the user always has a reliable mental model of when a change
becomes real. It was decided while aligning the Theme Editor (B081), whose
live-preview had blurred the save boundary.

## The rule -- every editable Workspace Context

An editable Context (Theme Editor, Config panel, Menus/Keys/Kinds editors,
Command Configurator, and any future editable Context) MUST:

1. **Edit a WORKING COPY.** On entry, take a working copy of the persisted state.
   All edits mutate the working copy only; the persisted state is untouched until
   a commit command fires. A live PREVIEW of the working copy is allowed and
   encouraged (e.g. the theme preview), but preview is NOT persistence.

2. **SAVE command -- commit, stay open.** The `SAVE` command persists the working
   copy to the backing store and leaves the Context open for further editing.
   After a successful SAVE the working copy and persisted state are equal (no
   dirty state).

3. **END command -- commit, then exit.** The `END` command (F3) persists the
   working copy (END IMPLIES SAVE) and then exits the Context. END never exits
   while silently dropping edits.

4. **RETURN command -- discard, then exit.** The `RETURN` command discards the
   working copy WITHOUT persisting and exits the Context. The backing store is
   left exactly as it was on entry. Any live preview is reverted to the persisted
   state on RETURN.

Summary: SAVE = commit + stay; END = commit + exit; RETURN = discard + exit.
Nothing is persisted on a bare edit.

## What this convention forbids

- Autosave-on-edit (persisting each keystroke/colour pick/toggle as it happens).
- An END that exits without committing, or a RETURN that persists.
- A SAVE button or affordance in a Context that does NOT actually gate
  persistence (the earlier blurred-boundary bug): if a Context shows SAVE, SAVE
  MUST be the commit point.
- Persisting a live preview as a side effect of rendering it.

## Dirty-state and discard

- A Context SHOULD track whether the working copy differs from the persisted
  state (dirty), so it can reflect unsaved changes and so END/RETURN behave
  correctly.
- RETURN with unsaved changes discards them by design; a confirm-on-discard
  prompt MAY be added per Context but is not required by this rule.

## Future option (NOT in scope now)

We MAY later retrofit a save-on-certain-events trigger (e.g. auto-commit on
specific lifecycle events) as an ADDITIVE enhancement. Until that is explicitly
decided and gated, the explicit-commit rule above is the ONLY convention. Do not
pre-build event-driven autosave on the assumption it is coming.

## Rationale

Mixing autosave in some panels and explicit-save in others is the worst outcome:
the user cannot predict when a change is real and loses work in whichever panel
violates their current expectation. A single ISPF-style commit model -- working
copy, SAVE/END to commit, RETURN to discard -- is consistent, matches the
product's ISPF heritage, and keeps a clear transaction boundary around every
edit. The Theme Editor's live preview is kept as a VIEW of the working copy, not
as persistence, so preview and the save boundary no longer conflict (B081).
