# Design -- Master Screen Design Directive

## Overview

The master screen design directive is a CONFORMANCE directive, not a new
subsystem. Its design is therefore about HOW each of the three cross-cutting
contracts is realised on the EXISTING framework seams, and about the ONE new
obligation it introduces -- a per-Context `ScreenProvider` with a documented
opt-out, mirroring the already-mandatory `InteriorFocus` contract. No new
framework type, dispatcher, navigation stack, or persistence format is created.
The directive sits beside `.kiro/steering/workspace-conformance.md` as the second
"every screen" guardrail: where that steering file makes focus mandatory per
render arm, this directive binds focus + save + capture together so a screen is
retrofitted once.

The technology stack is LOCKED to the existing workbench: Rust, egui/eframe,
`egui_kittest` for full-shell UI tests, the `ff-screen-model` / `ff-scrm` crates
for the logical screen model and capture engine, and the `ff-desktop` shell for
the Context wiring. The directive introduces no new dependency.

## The three contracts and their seams

### Contract 1 -- Focus (existing, referenced)

The focus contract is delivered entirely by the `WorkspaceContext` framework
(CR-NR-078). A Context implements `WorkspaceContext::render(&mut self, ui,
&mut ShellServices) -> InteriorFocus`, and the shell dispatches it through
`WorkbenchShell::render_workspace_context`, which applies the single focus latch.
A Context with render inputs the trait cannot express uses the shared
`apply_interior_focus` helper with an explicit `InteriorFocus`. The directive
adds nothing here; it references workspace-conformance and requires the mandatory
first-Tab `egui_kittest` test per Context.

### Contract 2 -- Save convention (existing, referenced)

The save convention is delivered by the CR-CH-071 editable-Context seam in
`crates/ff-desktop/src/shell/editable_context.rs`. `editable_context_kind()` maps
the active `KindTag` to an `EditableKind`; `editable_persist` / `editable_discard`
/ `editable_reload` route SAVE / END / RETURN / REFRESH to the owning Context.
The ladder arms (`crates/ff-desktop/src/shell/commands_ladder_a.rs`) implement
END = persist-then-exit (stays open on error), RETURN = discard, REFRESH =
discard + reload + stay. A new editable Context complies by adding an
`EditableKind` arm and a whole-store working copy; the directive references
project-master Requirement 11 for the behaviour and does not restate it.

### Contract 3 -- Snapshot + capture (the one new obligation)

This is the only contract the directive actively extends. Today the capture
engine reaches a screen ONLY for Menu_Workspaces:
`crates/ff-desktop/src/shell/commands_scrm.rs::active_screen_model()` returns
`Some` only when `self.tabs.active_tab().kind.menu_workspace()` is `Some`, so
every other Context yields the "cannot be captured yet" message. The directive
mandates that `active_screen_model()` instead DISPATCH to the active Context for
its `ScreenModel` via the `ff-screen-model` `ScreenProvider` trait
(screen-snapshot-scrm Req 2.1), with a per-Context provider impl for every
Context and an explicit documented opt-out for a Context that has no logical
screen.

The opt-out is designed to mirror `InteriorFocus::none()`: a Context declares, in
code with a comment stating why, that it has no logical screen. Only then does
the capture engine return the "not capturable" response (screen-snapshot-scrm Req
2.3). A Context that renders logical content but omits a provider is a defect,
caught by the conformance harness (below). This keeps the opt-out a deliberate,
auditable exception rather than a silent gap -- the exact property that made the
focus contract robust.

## Why bind the three together

The focus, save, and capture contracts all apply to the SAME set of screens. The
investigation found the two "every screen" contracts (save + capture) touch the
same Contexts the focus contract already governs. Binding them in one directive
with one retrofit matrix (Requirement 5) lets each screen be modified once for
all applicable contracts, and lets a single conformance harness assert all of
them. The alternative -- three independent sweeps -- would touch each screen three
times and risk a screen passing one contract while silently failing another, the
precise failure mode (phantom focus stops, B056-B059) the focus contract was
created to prevent.

## Files and integration points

The directive itself is documentation. Its enforcement, when the implementation
streams run, touches:

- `crates/ff-desktop/src/shell/commands_scrm.rs` -- `active_screen_model()`
  becomes a per-Context `ScreenProvider` dispatch (the Requirement 4 change).
- `crates/ff-desktop/src/screen_snapshot.rs` -- the existing menu provider
  remains the Menu_Workspace case of that dispatch.
- Each Context panel module under `crates/ff-desktop/src/` -- a `ScreenProvider`
  impl (or the documented opt-out).
- `crates/ff-desktop/src/shell/editable_context.rs` -- a new `EditableKind` arm
  for any editable Context not yet wired (Command Configurator, Config/Settings
  on merge).
- A reusable full-shell `egui_kittest` conformance harness
  (`crates/ff-desktop/src/shell/tests_screen_directive.rs`, new) asserting per
  Context: the first-Tab focus contract and SNAPSHOT-returns-a-model (or the
  documented opt-out).

These are the implementation-stream touch points; they are listed so the design
is concrete, but the gate at hand is docs-only and writes none of them.

## Error handling

The directive governs two observable error surfaces, each concrete:

- **SNAPSHOT / CAPTURE on a non-capturable Context.** Recoverable, user-facing.
  When a Context carries the documented opt-out, `active_screen_model()` yields
  `None` and the command returns the existing "the active workspace cannot be
  captured yet" status message (screen-snapshot-scrm Req 2.3, Req 6.3); nothing
  is copied. This is NOT a crash and NOT logged as an error -- it is a normal
  response for an opt-out Context. The defect case (a logical screen with no
  provider) is caught at test time by the conformance harness, not at runtime.
- **SAVE / END persistence failure on an editable Context.** Recoverable. Per
  project-master Requirement 11.4, END on a persist failure leaves the Context
  OPEN and reports the error; the working copy is retained. The caller (the
  ladder END arm) receives a `Result` and does not exit the Context. Logging is
  at the level the owning Context already uses for a persist failure; the
  directive adds no new logging obligation.

## Validation rules

The directive validates conformance, not user input. The validation is at TEST
time via the conformance harness:

- For EACH Context (`KindTag`): required that EITHER SNAPSHOT returns a
  `ScreenModel` OR the Context is on the documented opt-out list; failure on
  both is a test failure.
- For EACH editable Context (`EditableKind`): required that SAVE/END/RETURN route
  through the CR-CH-071 seam (the project-master Req 11 test pattern); a Context
  that persists only via a button is a test failure.
- For EACH Context reporting interior focus: the existing first-Tab test
  (workspace-conformance) must pass.

The matrix (Requirement 5) is the authoritative checklist the harness walks; a
row flips to COMPLIANT only when its Context passes the applicable assertions.

## Invariant ownership

- The FOCUS invariant (no phantom Tab stop) is owned by the `WorkspaceContext` /
  `render_workspace_context` latch (workspace-conformance). The directive does
  not re-own it.
- The SAVE invariant (no autosave; commit only on SAVE/END; discard on RETURN) is
  owned by the CR-CH-071 seam (project-master Req 11). The directive does not
  re-own it.
- The CAPTURE invariant (every Context is capturable or on a documented opt-out)
  is the invariant this directive OWNS, enforced at the `active_screen_model()`
  dispatch and the conformance harness.

## Testability

The directive is highly testable because every contract has an existing or
specified automated harness: the first-Tab focus test, the save-convention seam
tests, and the new SNAPSHOT-returns-a-model conformance harness. The design is
deliberately built around a single reusable harness that walks the matrix so that
adding a Context forces a row and an assertion, making a missing contract a
compile/test failure rather than a latent gap.

## No framework change

This directive proposes no framework change. Making `active_screen_model()`
dispatch to a per-Context `ScreenProvider` is building ON the existing
`ScreenProvider` trait (screen-snapshot-scrm Req 2) -- it replaces a hard-coded
menu-only lookup with the trait the spec already defines, which is conformance,
not a new mechanism. Should any Context's provider require reshaping a public
framework type to express its screen, that is an owner-confirmed framework change
per framework-conformance and is surfaced, not assumed.
