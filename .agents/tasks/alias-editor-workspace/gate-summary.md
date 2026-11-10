# Gate Summary -- Alias Editor Workspace Context (CR-NR-113)

Requirements gate (step a) for the owner's follow-up to CR-NR-112: "We need a
workspace for creating and saving aliases." Preceded by a grounded findings pass
(step b, `.agents/tasks/alias-editor-workspace/findings.md`). Owner chose: SAVE
LOCATION = a user-scoped alias file separate from the per-locale localisation
catalogues; SCOPE v1 = the editor authors the user file ONLY (recommendation
followed). Documentation-only gate -- NO source code written.

## Allocations

- Change-request id: **CR-NR-113** (next free; re-verified -- highest existing was
  CR-NR-112).
- **command-configurator Requirement 5** -- Alias Editor Context (12 criteria
  5.1-5.12).

## Docs produced / edited

| File | Change |
|------|--------|
| `docs/status/change-log.md` | NEW entry CR-NR-113 (matches existing CR entry format). |
| `docs/specs/command-configurator/requirements.md` | NEW Requirement 5 (appended after Req 4). |
| `docs/specs/command-configurator/design.md` | NEW Section 9 (Alias Editor Context: user store location, data model, lifecycle, save path, resolver load/precedence, registration, testability, framework-conformance). |
| `docs/specs/command-configurator/tasks.md` | NEW Task 7 (7.1-7.7). |
| `docs/quality/TCR.md` | NEW CR-NR-113 coverage section -- 12 NOT-COVERED rows (one per criterion). |
| `docs/project-management/ROADMAP.md` | NEW CR-NR-113 tracking sub-bullet under the alias-layer cluster. |
| `.agents/tasks/alias-editor-workspace/findings.md` | Grounded read-only findings (step b, written earlier). |

## Key design decisions encoded

1. **New Context, modelled on the Menus Editor.** The Alias Editor copies
   `shell/menus_editor.rs` one-to-one: list -> working copy -> add/delete/edit ->
   validate-with-loader-discipline -> serialise -> write -> refresh. Low-risk,
   proven pattern.
2. **Save location = a NEW user-scoped, locale-independent store**
   (`<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`), the same data base as
   macros/commands, NOT the shipped per-locale `i18n/<locale>/aliases/` tree.
3. **v1 scope = user store only.** The per-locale localisation catalogues
   (CR-NR-109/111) are NOT read or written by the editor -- no overlap with the
   in-flight CR-NR-111 gate.
4. **Registration = configurator family, no new Kind.** `WorkspaceKind::CommandConfigurator`
   with `editor = "aliases"` (Theme/Menus/Keys/Kinds already use this pattern).
   Opened by an `ALIASES` verb / menu option through the single front door.
5. **Two genuinely new code pieces** (everything else is a copy): an alias
   SERIALISER (none exists -- the alias layer is read-only today, only a
   `#[cfg(test)]` writer exists) and a user-store LOAD path on `LocaleAliasStore`
   with a built-in-root shadowing guard and an in-place live refresh after save
   (reusing the `ACTIVE_ALIAS_STORE` reload seam).
6. **Dispatch/expansion unchanged.** The editor only authors + persists; invoking
   a saved alias runs the CR-NR-112 pre-pass with no new behaviour.

## Framework-conformance

NOT a framework change by the framework-conformance definition: the Context uses
the existing `WorkspaceContext` + `render_workspace_context` focus-latch path, the
existing `CommandConfigurator` Kind + descriptor, and the single command-dispatch
front door; no core framework type is reshaped. The ONE new mechanism is the
user-scoped alias store location + its resolver load path (a new user-writable
store + a new load order on `LocaleAliasStore`). This is additive, but because it
introduces a new persistence/load location it is surfaced here for owner sign-off
at the CONFIRM step.

## Open decisions for the owner (confirm at the gate CONFIRM step)

1. **User store path/shape**: `<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`
   (single file, locale-independent). Confirm the path and single-file choice.
2. **Resolver precedence**: the user store overlays on top of the identity base +
   per-locale overlay, and a user alias MUST NOT shadow a built-in root (rejected
   at save, skipped-with-WARN at load). Confirm the no-shadow rule and that a user
   alias may coexist with (not override) a per-locale localisation surface.
3. **v1 scope**: editor authors the user store only; per-locale catalogues out of
   scope. Confirm, or raise a follow-on CR to manage localisation catalogues too
   (would need CR-NR-111 coordination).

No finding required changing a settled owner decision or reshaping a framework
mechanism, so no `send_message` warning was raised. Awaiting the design-review
iteration.
