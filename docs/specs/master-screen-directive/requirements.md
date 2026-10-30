# Requirements Document -- Master Screen Design Directive (cross-cutting every-Context contract)

## Introduction

This sub-project is the MASTER SCREEN DESIGN DIRECTIVE: a single, cross-cutting
requirement set that EVERY Workspace Context in FileForgeWorkbench must satisfy.
It is the sibling of the existing focus contract already enforced per render arm
(`.kiro/steering/workspace-conformance.md`): where that contract makes the
WorkspaceContext / InteriorFocus focus-latch mandatory for every Context, this
directive binds THREE cross-cutting contracts together so a given screen can be
retrofitted ONCE for all of them rather than piecemeal.

The three contracts every Context must honour are:

1. **Focus contract** -- the WorkspaceContext / InteriorFocus single focus-latch
   path (CR-CH-023, CR-NR-078). ALREADY defined and enforced; this directive only
   references it so that "every Context complies with all three" is one statement.
2. **Save convention** -- the ISPF-style explicit-commit save model for EDITABLE
   Contexts (central requirement `docs/project-management/project-master/requirements.md`
   Requirement 11; standing owner rule `.kiro/steering/save-convention.md`,
   B081). Referenced, not duplicated.
3. **Snapshot + capture** -- a per-Context `ScreenProvider` so SNAPSHOT and the
   CAPTURE family work on EVERY Context, not only Menu_Workspaces
   (`docs/specs/screen-snapshot-scrm/` Requirement 2). Referenced, with an
   explicit documented opt-out analogous to `InteriorFocus::none()` for a Context
   that genuinely has no logical screen.

This document introduces NO new framework mechanism. It builds ON the existing
framework seams (`framework-conformance.md`, `wiring-standard.md`): the single
`WorkspaceContext` / `render_workspace_context` focus latch, the CR-CH-071
editable-Context save seam (`crates/ff-desktop/src/shell/editable_context.rs`),
and the `ff-screen-model` `ScreenProvider` trait. It is a CONFORMANCE directive:
it states what each Context must already do, and it carries the Core Context
retrofit matrix so the remaining gaps are explicit and trackable.

### Relationship to existing specs

- The FOCUS contract is owned by `.kiro/steering/workspace-conformance.md` and
  the WorkspaceContext framework (CR-NR-078, `docs/specs/workspace-framework/`).
  This directive does not restate its criteria; it REQUIRES compliance.
- The SAVE convention is owned by project-master Requirement 11 and
  `.kiro/steering/save-convention.md`. This directive does not restate the ten
  acceptance criteria; it REQUIRES compliance and cross-links.
- The SNAPSHOT + capture contract is owned by `docs/specs/screen-snapshot-scrm/`
  Requirement 2 (the `ScreenProvider` seam). This directive GENERALISES Req 2's
  per-Context provider expectation into an every-Context obligation and defines
  the documented opt-out.

### The ScreenProvider obligation (the one new obligation this directive states)

The focus contract and the save convention are already mandatory per Context.
The snapshot + capture contract is NOT yet an every-Context obligation: the live
code obtains a screen only for Menu_Workspaces
(`crates/ff-desktop/src/shell/commands_scrm.rs::active_screen_model`), which is
the root cause of the "the active workspace cannot be captured yet" message on
every non-menu Context. This directive makes the per-Context `ScreenProvider`
mandatory, exactly as `InteriorFocus` is mandatory, with a symmetric documented
opt-out. No new mechanism is introduced: the `ScreenProvider` trait already
exists in `ff-screen-model` (screen-snapshot-scrm Req 2.1); this directive
requires every Context to route through it.

### Glossary

- **Context**: A Workspace Context (the Level-3 content inside a Workspace tab),
  identified at runtime by its `KindTag` (`crates/ff-desktop/src/tab_state.rs`).
- **Editable Context**: A Context that edits a persisted backing store (its
  `KindTag` maps to an `EditableKind` via `editable_context_kind()`). Subject to
  the save convention.
- **Non-editable Context**: A read-only or navigational Context (no working copy,
  no backing store). END/RETURN there are pure navigation, not persistence.
- **ScreenProvider**: The `ff-screen-model` trait that yields a Context's logical
  `ScreenModel` on demand for SNAPSHOT / CAPTURE (screen-snapshot-scrm Req 2.1).
- **ScreenProvider opt-out**: An explicit, documented declaration that a Context
  has no logical screen, analogous to `InteriorFocus::none()`. A deliberate,
  commented exception -- never a silent gap.

---

## Requirement 1 -- The three cross-cutting contracts bind every Context

**User story:** As a maintainer, I want one directive that states the full
cross-cutting contract every Workspace Context must satisfy, so that a screen is
retrofitted once for focus, save, and capture rather than three times, and so a
new Context cannot ship missing one of them.

#### Acceptance Criteria
1.1 EVERY Workspace Context SHALL satisfy ALL THREE cross-cutting contracts
    applicable to it: the focus contract (Requirement 2), the save convention
    where the Context is editable (Requirement 3), and the snapshot + capture
    contract (Requirement 4).
1.2 THE applicability of the save convention to a given Context SHALL be
    determined by whether that Context is editable (edits a persisted backing
    store); a non-editable Context is exempt from Requirement 3 but NOT from
    Requirements 2 and 4.
1.3 THE snapshot + capture contract (Requirement 4) SHALL apply to EVERY Context
    with no editable/non-editable distinction, subject only to the documented
    opt-out of Requirement 4.3.
1.4 WHEN a NEW Workspace Context is added (core or extension-contributed), THE
    Context SHALL satisfy Requirements 2, 3 (if editable), and 4 before it ships;
    adding a Context that renders interactive content but omits an applicable
    contract SHALL be treated as a defect, not a style nit.
1.5 THIS directive SHALL introduce NO new framework mechanism: each contract is
    delivered through its EXISTING seam (the `WorkspaceContext` focus latch, the
    CR-CH-071 editable-Context save seam, the `ff-screen-model` `ScreenProvider`
    trait). Any criterion that cannot be met without altering a core mechanism
    SHALL be surfaced as an owner-confirmed FRAMEWORK CHANGE (framework-conformance)
    and SHALL NOT be undertaken as an implementation side effect.

---

## Requirement 2 -- Focus contract (reference, not restated)

**User story:** As a keyboard user, I want every Context to participate in the
unified tab-order model, so that the first Tab from the command field lands on a
real control with no phantom stop.

#### Acceptance Criteria
2.1 EVERY Workspace Context SHALL report its interior focus through the SINGLE
    focus-latch path -- the `WorkspaceContext::render -> InteriorFocus` trait
    dispatched by `render_workspace_context`, or the shared `apply_interior_focus`
    helper for a documented variance -- per `.kiro/steering/workspace-conformance.md`
    (CR-CH-023, CR-NR-078). This directive does NOT restate those criteria; it
    REQUIRES them.
2.2 EVERY Context that reports interior focus stops SHALL ship the mandatory
    full-shell `egui_kittest` first-Tab test proving the first Tab from the
    command field lands EXACTLY on the reported first interior control (no
    phantom stop), per workspace-conformance.
2.3 WHERE a Context genuinely has no interior focus stops, THE Context SHALL
    return `InteriorFocus::none()` with a documented reason, per workspace-conformance.

---

## Requirement 3 -- Save convention for editable Contexts (reference, not restated)

**User story:** As a user, I want every editable Context to persist only on an
explicit commit (SAVE or END) and discard on RETURN, so that I have one reliable
mental model of when a change becomes real.

#### Acceptance Criteria
3.1 EVERY editable Workspace Context SHALL comply with the ISPF-style
    explicit-commit save convention defined in
    `docs/project-management/project-master/requirements.md` Requirement 11 and
    `.kiro/steering/save-convention.md` (B081): working copy on entry; no
    autosave-on-edit; SAVE = commit + stay; END = commit + exit (stays open on
    persist error); RETURN = discard + exit. This directive does NOT restate the
    ten criteria of Requirement 11; it REQUIRES them.
3.2 EVERY editable Context SHALL reach SAVE / END / RETURN through the SINGLE
    shared CR-CH-071 editable-Context seam
    (`crates/ff-desktop/src/shell/editable_context.rs`:
    `editable_context_kind()` / `editable_persist` / `editable_discard` /
    `editable_reload`); NO editable Context SHALL persist exclusively through an
    in-panel button while the typed/keyed commands are no-ops (Requirement 11.7).
3.3 A NON-editable Context (read-only or navigational -- no working copy, no
    backing store) SHALL be exempt from this requirement; its END / RETURN SHALL
    be pure navigation (pop the Navigation_Stack back to the opener), NOT
    persistence.
3.4 WHEN a NEW editable Context is added, THE Context SHALL wire the CR-CH-071
    seam before it ships (the forward directive cross-linked from project-master
    Requirement 11); a new editable Context SHALL NOT introduce a per-panel
    autosave.

---

## Requirement 4 -- Snapshot + capture contract (per-Context ScreenProvider)

**User story:** As a user, I want SNAPSHOT and CAPTURE to work on EVERY screen,
not only on menu screens, so that I can document, capture, and replay any
Context in the workbench.

#### Acceptance Criteria
4.1 EVERY Workspace Context SHALL provide its logical screen to the capture
    engine through the `ff-screen-model` `ScreenProvider` seam
    (screen-snapshot-scrm Req 2.1, 2.2): the capture engine SHALL obtain the
    Context's `ScreenModel` by asking the ACTIVE Context for it, and SHALL NOT
    inspect egui widget trees.
4.2 WHEN the user issues `SNAPSHOT` or a `CAPTURE` command on ANY Context that
    provides a `ScreenProvider`, THE system SHALL capture that Context's screen
    (it SHALL NOT report "the active workspace cannot be captured yet" for a
    Context that has a provider). This replaces the current Menu_Workspace-only
    behaviour of `active_screen_model()`.
4.3 WHERE a Context genuinely has NO logical screen, THE Context SHALL declare an
    EXPLICIT, DOCUMENTED opt-out (analogous to `InteriorFocus::none()`): a
    commented declaration that the Context is not capturable and why. Only a
    Context with the documented opt-out SHALL produce the "not capturable"
    response (screen-snapshot-scrm Req 2.3); a Context that renders a logical
    screen but omits a provider SHALL be treated as a defect.
4.4 THE `ScreenProvider` opt-out SHALL be the RARE exception, not the default:
    the default for a Context that renders any logical screen content is to
    provide a `ScreenModel`.
4.5 THE snapshot + capture contract SHALL be realised ONLY through the existing
    `ScreenProvider` trait and the existing SNAPSHOT / CAPTURE command dispatch
    (screen-snapshot-scrm Req 11); NO Context SHALL expose snapshot/capture
    through a bespoke path outside the `ScreenProvider` seam and the single
    command front door.
4.6 THE snapshot + capture contract SHALL ALSO bind TRANSIENT surfaces that are
    NOT Workspace Contexts -- message boxes and modal dialog boxes -- so that
    their text is capturable and copy/paste-able. Because such overlays are not
    Contexts, this every-Context requirement does not reach them by itself; the
    obligation and its one exception (credential-secret redaction) are stated in
    screen-snapshot-scrm Requirement 27. EVERYTHING a message box or dialog box
    shows SHALL be copy/paste-able EXCEPT a declared credential-secret field,
    which SHALL be redacted (never emitted in clear).

---

## Requirement 5 -- Core Context retrofit matrix (the trackable gap list)

**User story:** As a maintainer, I want one table that records, per Context, its
current state against the save convention and the snapshot/capture contract, so
that each screen is touched ONCE for both and no gap is lost.

#### Acceptance Criteria
5.1 THE directive SHALL carry a retrofit matrix with one row per Context
    (`KindTag`) and columns for the save-convention state and the snapshot/capture
    state, reflecting current reality (the matrix below).
5.2 A Context's row SHALL record whether it is editable (and thus save-convention
    applicable) and its current `ScreenProvider` state (provider present, needs a
    provider, or documented opt-out).
5.3 THE matrix SHALL be the single trackable list the implementation streams flip
    to COMPLIANT as each Context is retrofitted; it SHALL NOT duplicate the
    criteria of Requirement 11 or screen-snapshot-scrm Req 2 (it references them).

### Core Context retrofit matrix

"Editable" = must wire the CR-CH-071 save seam (Requirement 3). "Snapshot" = must
provide a `ScreenProvider` (Requirement 4). Source: `crates/ff-desktop/src/tab_state.rs`
(`KindTag`), `crates/ff-desktop/src/shell/editable_context.rs`,
`crates/ff-desktop/src/shell/commands_scrm.rs`.

| Context (KindTag) | Save convention (current -> needed) | Snapshot/capture (current -> needed) |
|---|---|---|
| MenuWorkspace (POM/Settings/menus) | n/a (not editable) | capturable, but TRIMMED -> enrich to full chrome/calendar/PF-keys (screen-snapshot-scrm Req 3.8) |
| AliasEditor | COMPLIANT (reference impl) | not capturable -> add provider |
| ThemeEditor | COMPLIANT | not capturable -> add provider |
| MenusEditor | COMPLIANT | not capturable -> add provider |
| KeysEditor | COMPLIANT | not capturable -> add provider |
| KindsEditor | COMPLIANT | not capturable -> add provider |
| ConfigPanel (Settings/Config) | PENDING branch merge (config-save-convention) | not capturable -> add provider (ANSI colour matters here) |
| CommandConfigurator | NON-COMPLIANT -> retrofit the CR-CH-071 seam | not capturable -> add provider |
| FilesPanel (Virtual Catalog Mgr) | n/a (not an editable text store) | not capturable -> add provider |
| FileExplorerPanel | n/a | not capturable -> add provider |
| SearchResults | n/a (read-only) | not capturable -> add provider |
| PluginManager (-> ExtensionManager) | n/a | not capturable -> add provider |
| EventLog | n/a (read-only) | not capturable -> add provider |
| MacroLibrary | n/a | not capturable -> add provider |
| HelpContext | n/a (navigational) | not capturable -> add provider |
| ScrmViewer | n/a | add provider OR documented opt-out (decide in SCRM stream) |
| FileEditor / Untitled (editor) | FFEDIT owns SAVE (active-wins) | not capturable -> add provider when the editor re-enters (parked) |

---

## Requirement 6 -- Extension-contributed Contexts inherit the directive

**User story:** As an extension developer, I want the same every-Context contract
to bind a Context I contribute, so that extension Contexts are first-class and
cannot re-open the gaps this directive closes.

#### Acceptance Criteria
6.1 AN extension-contributed Workspace Context (extension-integration Req 5)
    SHALL satisfy this master screen directive in full: the focus contract
    (Requirement 2), the save convention if the Context is editable (Requirement
    3), and the snapshot + capture contract (Requirement 4).
6.2 THE extension-integration spec Req 5 SHALL cross-reference this directive as
    the single statement of the Context contract, rather than enforcing only the
    focus latch; this requirement is the master-directive side of that
    cross-reference (the extension side is extension-integration Req 5).

---

## Non-Functional Requirements

1. ASCII-only in all spec documentation (`.kiro/steering/documentation-charset.md`).
2. This directive adds NO runtime cost of its own: it is a conformance statement
   over existing seams; the per-Context `ScreenProvider` impls it mandates follow
   screen-snapshot-scrm Req 18 (capture must not visibly interrupt interaction).
3. The directive stays GUI-independent at the contract level: the
   `ScreenProvider` and `InteriorFocus` seams it references are the existing
   trait boundaries, not new ones.

---

## Testability

- Requirement 2 is covered by the existing per-Context full-shell first-Tab
  `egui_kittest` focus test (workspace-conformance), one per Context.
- Requirement 3 is covered by the per-Context save-convention tests at the
  CR-CH-071 seam (project-master Req 11 test pattern).
- Requirement 4 is covered by a reusable full-shell conformance harness that, per
  Context, asserts SNAPSHOT returns a model (no "cannot be captured yet") OR the
  Context carries the documented opt-out. The matrix (Requirement 5) is the
  authoritative checklist the harness walks.

---

## Out of scope

- The focus-contract criteria themselves (owned by workspace-conformance /
  workspace-framework); this directive references them.
- The ten save-convention criteria themselves (owned by project-master
  Requirement 11); this directive references them.
- The SCRM capability detail -- screen model content, picture formats, capture
  Save-As, replay, menu family (owned by `docs/specs/screen-snapshot-scrm/`);
  this directive only mandates that every Context provides a `ScreenProvider`.
- The implementation sequencing (worktrees/streams) lives in the investigation
  plan, not this requirements doc.
