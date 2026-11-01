# Requirements Document -- Shared ISPF Table Editor + Line-Command Component (CR-CH-075)

## Introduction

Three editable Workspace Contexts -- the Alias Editor
(command-configurator Requirement 5, CR-NR-113 / CR-CH-068), the Menus Editor
(menu-workspace Requirement 13), and the Workspace Kinds Editor
(workspace-kinds Requirement 6) -- all want the SAME ISPF-style tabular editing
surface: a `Command ===>` primary command line, a filter row, a per-row Act
prefix column that accepts ISPF line commands, and BATCH line commands that
restructure the row list (Insert, Delete, Copy, Move) plus a Sort capability.

Today this is NOT shared. The Alias Editor carries a bespoke mini line-command
flow (I / R / D only) in `crates/ff-desktop/src/shell/alias_editor_line_commands.rs`;
the Menus Editor uses structural Up / Down / Delete BUTTONS; the Kinds Editor is
a vertical form. The complete ISPF line-command ENGINE already exists in the
GUI-free crate `crates/ff-line-commands` (parse, classify, block-pair, pending
store, compatibility matrix, execute for D/I/R/C/M/X/T/U/shift) but is wired ONLY
to the text editor's prefix area.

This spec defines a SHARED, reusable ISPF Table Editor component so all three
editors present the same surface and gain Copy / Move / Sort uniformly, with ONE
line-command mechanism instead of three. It is a Context-redesign built ON the
existing framework seams; it introduces NO new dispatcher, NO new navigation
stack, NO new session-persistence format, NO new line-command engine, and NO new
save path. Per `.kiro/steering/framework-conformance.md` this is NOT a framework
change: it builds on the `WorkspaceContext` + `InteriorFocus` focus contract
(CR-NR-078, `.kiro/steering/workspace-conformance.md`), the CR-CH-071
editable-Context save seam (`.kiro/steering/save-convention.md`), and the
`ff-line-commands` engine.

### Source references

- **[CR-CH-075]** = this change request (owner: the Alias, Menu, and Kinds
  editors become ISPF table editors sharing one line-command component).
- **[CR-NR-078]** = the `WorkspaceContext` trait + `InteriorFocus` focus contract.
- **[CR-CH-071]** = the editable-Context save seam (working copy, SAVE / END /
  RETURN / CANCEL / REFRESH).
- **[FFE-CMD-22..35]** = the `ff-line-commands` engine (line-commands spec).

### Relationship to other specs

- **`line-commands`** owns the GUI-free line-command engine this component
  REUSES for parsing, classification, and block pairing. No engine change is
  required by this spec; where a table editor needs only a subset (I/D/C/M +
  Sort) it uses that subset of the engine's vocabulary.
- **`command-configurator`** (Alias Editor) consumes this component; its
  editor-specific columns, model mapping, and line-command set are specified in
  that spec (Requirement 5, the CR-CH-075 redesign section).
- **`menu-workspace`** (Menus Editor) consumes this component; see its
  Requirement 13 CR-CH-075 criteria.
- **`workspace-kinds`** (Kinds Editor) consumes this component; see its
  Requirement 6 CR-CH-075 criteria.
- **`screen-snapshot-scrm`** (Group A capture) -- the component's displayed
  columns are the authoritative column set a Context's `ScreenProvider`
  populates into a `ff-screen-model::Table`, so a snapshot of the editor matches
  what the user sees (see Requirement 7 below).

## Glossary

- **Table_Editor**: The shared ISPF tabular editing component this spec defines.
  It renders a `Command ===>` line, a filter row, a header row, and a vertically
  scrolling list of data rows, each with an Act prefix cell.
- **Act_Cell**: The per-row prefix text cell that accepts an ISPF line-command
  letter (I / D / C / M, and the subset a given editor enables). Transient; never
  persisted.
- **Row_Model**: The host editor's working-copy row type (e.g. an alias row, a
  menu option, a Kind config). The Table_Editor is generic over the host's row
  list; it owns ordering and line-command structuring, NOT the field semantics.
- **Pending_Line_Command**: An Act letter typed into an Act_Cell that has not yet
  been applied. Pending commands accumulate and apply as a batch (ISPF model).
- **Line_Command_Set**: The subset of I / D / C / M / Sort a given host editor
  enables (each editor declares its own set; see per-editor specs).
- **Filter_Row**: The row directly under the header that restricts which data
  rows are DISPLAYED, without mutating the underlying row list.
- **Command_Line**: The `Command ===>` primary command field at the top (or, per
  a Kind's command-line position, the bottom) of the Context.
- **Block_Pairing**: The ISPF convention that a doubled marker (CC / MM) spans a
  range; the Table_Editor reuses the `ff-line-commands` pairing rules where a
  host enables block forms.

---

## Requirements

### Requirement 1: A reusable ISPF table-editor surface

**User Story:** As a workbench developer, I want one reusable table-editor
component so that the Alias, Menu, and Kinds editors present an identical
ISPF-style surface without each re-implementing it.

**Source:** [CR-CH-075]

#### Acceptance Criteria

1. THE workbench SHALL provide a reusable Table_Editor component, generic over a
   host editor's Row_Model list, that renders top-to-bottom: a `Command ===>`
   line, a column HEADER row, a FILTER row immediately under the header, and a
   vertically scrolling list of DATA rows (each with a leading Act_Cell), with a
   scrollbar when the rows exceed the visible area.

2. THE Table_Editor SHALL be rendered through the `WorkspaceContext` framework
   (CR-NR-078): a host editor's pure `render` draws the surface and RETURNS its
   editor Action plus an `InteriorFocus`, and the shell applies the Action via
   the owned-panel swap (`mem::take` -> `render_workspace_context` -> put back ->
   `apply_<editor>_action`). THE Table_Editor SHALL NOT mutate shell state from
   inside render and SHALL NOT introduce a second dispatcher or navigation stack.

3. THE Table_Editor SHALL own the GENERIC concerns (the `Command ===>` field, the
   filter row, the Act column, pending-line-command accumulation, the batch apply
   pass, and row ordering) and SHALL delegate the FIELD-SPECIFIC concerns (which
   columns exist, each column's widget, validation, and the backing Row_Model
   fields) to the host editor. A host editor SHALL declare its columns, its
   Line_Command_Set, and whether it enables Sort.

4. THE Table_Editor SHALL add NO new on-disk format: it operates purely on the
   host editor's in-memory working-copy row list, which the host persists through
   its own existing save path (Requirement 6).

### Requirement 2: The `Command ===>` line

**User Story:** As an ISPF user, I want a `Command ===>` primary command line in
each table editor so that I drive the editor the mainframe way.

**Source:** [CR-CH-075]

#### Acceptance Criteria

1. THE Table_Editor SHALL render a `Command ===>` field with a STABLE `egui::Id`
   as the Context's ENTRY control, mirroring the shell command line; on Enter
   (field `lost_focus()` + the Enter key) it SHALL emit a command-line Action
   carrying the typed text to the host editor's apply pass.

2. WHEN the user presses Enter on the `Command ===>` line, THE Table_Editor SHALL
   FIRST apply any pending line commands (Requirement 4) and THEN interpret the
   typed command text, so an Act letter typed on a row and a `SAVE` typed on the
   command line in the same cycle apply in that order (ISPF: line commands, then
   the primary command).

3. THE `Command ===>` line SHALL interpret, case-insensitively, at least: `SAVE`
   (commit, stay), `END` (commit, exit), `CANCEL` / `RETURN` (discard, exit), and
   a bare Enter (apply pending line commands only). THE actual persistence is the
   host editor's CR-CH-071 save path (Requirement 6); the Table_Editor only
   routes the verb to it. A `FILTER` / `RESET` filter command (Requirement 3) and
   a `SORT` command (Requirement 5) are RECOMMENDED command-line forms.

4. AFTER a batch of line commands or a structural change is applied, focus SHALL
   return to the `Command ===>` line (the ISPF convention), via egui
   `request_focus` on the command-line id.

5. WHERE the host Context's Kind sets `Command_Line_Position = Bottom`
   (workspace-kinds Requirement 8), THE `Command ===>` line SHALL render at the
   foot of the Context instead of the top; field behaviour (submit, focus, the
   Boundary_Policy Tab order) SHALL be identical in both positions.

### Requirement 3: The filter row

**User Story:** As a user with a long list, I want a filter row so that I can
narrow the visible rows without changing the underlying data.

**Source:** [CR-CH-075]

#### Acceptance Criteria

1. THE Table_Editor SHALL render a FILTER row directly under the header whose
   inputs are declared by the host editor (which columns are filterable). The
   displayed data-row set SHALL be the subset of the row list matching the active
   filter; an empty filter SHALL display all rows.

2. Filtering SHALL be DISPLAY-only: it SHALL NEVER mutate, drop, or reorder the
   underlying row list, so a filtered-out row is preserved and saved unchanged.

3. WHEN multiple filter inputs are set, THE match SHALL be their AND (every set
   filter input must match). The host declares the comparison per column
   (equality / case-insensitive contains); columns the host does not declare as
   filterable SHALL NOT be filter inputs.

4. Line commands (Requirement 4) SHALL operate on the row's BACKING index into
   the full row list, NOT its display position, so a `D` on a visible (filtered)
   row deletes the correct underlying row.

5. THE filter SHALL be applied on Enter or an explicit `FILTER <args>` / `RESET`
   command (the ISPF command-driven model), not necessarily live-as-you-type; a
   host MAY additionally filter live from the field bindings, but the
   command-driven form SHALL be available.

### Requirement 4: The Act prefix column and batch line commands I / D / C / M

**User Story:** As an ISPF user, I want to type a line-command letter in each
row's prefix and apply the batch, so I can insert, delete, copy, and move rows
the mainframe way instead of using buttons.

**Source:** [CR-CH-075]

#### Acceptance Criteria

1. THE Table_Editor SHALL render a leading Act_Cell on each data row that accepts
   a line-command letter. The Act letter SHALL be TRANSIENT (never persisted) and
   SHALL NOT restructure the row list mid-frame; typed letters SHALL accumulate
   as Pending_Line_Commands on the rows.

2. THE Table_Editor SHALL apply pending line commands as a single DETERMINISTIC
   batch pass, triggered on Enter on the `Command ===>` line (Requirement 2.2),
   supporting at least these line commands where the host's Line_Command_Set
   enables them:
   - `I` (Insert) -- insert a blank Row_Model AFTER the row.
   - `D` (Delete) -- remove the row.
   - `C` (Copy) -- copy the row (its field values) to the location designated by
     a target marker.
   - `M` (Move) -- move the row to the location designated by a target marker.

3. THE batch SHALL be applied so that pending commands do not corrupt one
   another's indices (e.g. by processing non-pairing commands by DESCENDING row
   index), matching the engine's deterministic semantics; the result SHALL be
   independent of the order in which the user typed the letters within one batch.

4. FOR `C` (Copy) and `M` (Move), THE Table_Editor SHALL follow the ISPF pairing
   convention: a source marker (`C` / `M`, or the block forms `CC` / `MM` where a
   host enables blocks) is PENDING until the user supplies a target marker (`A`
   after / `B` before) on another row; an unresolved source marker SHALL be
   retained (not discarded) across the cycle and surfaced to the user, reusing the
   `ff-line-commands` pairing and pending-retention rules (line-commands
   Requirement 33 / 35).

5. WHEN an Act letter is not in the host's enabled Line_Command_Set or is
   unrecognised, THE Table_Editor SHALL IGNORE it and CLEAR the Act_Cell (ISPF
   tolerates and clears stray line commands), leaving the row list unchanged and
   surfacing a non-blocking notice where appropriate.

6. WHEN the row list becomes empty after a batch, THE Table_Editor SHALL still
   render an insertable surface (the header, filter row, and at least one blank
   insertable data row) so the user can always add the first row; a transient
   scratch blank row SHALL be used for display when the backing list is empty and
   promoted into the list on first edit or `I`.

7. THE Table_Editor SHALL reuse the `ff-line-commands` engine for line-command
   PARSING, CLASSIFICATION, and BLOCK PAIRING; it SHALL NOT introduce a second
   line-command parser or a parallel pending-command store. Where a host enables
   only the immediate subset (I / D), no pairing state is needed; where a host
   enables C / M, the engine's source/target pairing is used.

### Requirement 5: Sort

**User Story:** As a user, I want to sort a table editor's rows so that I can
order a long list by a column instead of moving rows one at a time.

**Source:** [CR-CH-075]

#### Acceptance Criteria

1. WHERE a host editor enables Sort in its Line_Command_Set, THE Table_Editor
   SHALL provide a SORT capability that reorders the underlying row list by a
   host-declared key column, invoked via a `SORT <column> [ASC|DESC]`
   command-line form (ascending by default) and/or a header affordance.

2. Sort SHALL reorder the BACKING row list (a persisted reordering), consistent
   with how `M` (Move) changes order; it SHALL be a stable sort so rows equal on
   the key retain their relative order.

3. WHERE a host editor does NOT enable Sort (it declares no sortable key), THE
   `SORT` command SHALL produce a non-blocking notice and leave the row order
   unchanged. (The Alias Editor enables Sort; the Menu and Kinds editors rely on
   Move for ordering -- see the per-editor specs.)

4. Sort SHALL respect the filter DISPLAY contract (Requirement 3.2): sorting
   reorders the full backing list; the filtered view then reflects the new order.

### Requirement 6: Integration with the CR-CH-071 editable-Context save seam

**User Story:** As a user, I want every table editor to save the same ISPF way so
that SAVE / END / RETURN behave identically across the Alias, Menu, and Kinds
editors.

**Source:** [CR-CH-075]; `.kiro/steering/save-convention.md`, [CR-CH-071]

#### Acceptance Criteria

1. THE Table_Editor SHALL operate over the host editor's WORKING COPY only; all
   line commands, sorts, and field edits SHALL mutate the working copy, never the
   persisted backing store, until a commit command fires.

2. THE Table_Editor SHALL route its `Command ===>` verbs to the host editor's
   EXISTING CR-CH-071 save path: `SAVE` commits the working copy and STAYS; `END`
   (F3) commits then EXITS; `CANCEL` / `RETURN` discards the working copy and
   EXITS; a `REFRESH` where the host provides it discards and reloads. THE
   Table_Editor SHALL add NO second save path and SHALL NOT persist on a bare
   edit, line command, or sort (those change the working copy only).

3. BEFORE a commit, THE Table_Editor SHALL apply any pending line commands
   (Requirement 4) so a `SAVE` includes the batch, consistent with the
   ISPF "line commands then primary command" ordering (Requirement 2.2).

4. THE Table_Editor SHALL NOT change the host editor's on-disk shape, its
   dispatch front door, its WorkspaceKind / descriptor, the per-tab
   Navigation_Stack return behaviour, or the `pending_action` effect pattern.

### Requirement 7: Focus contract and snapshot-column alignment

**User Story:** As a workbench developer, I want the table editor to satisfy the
one focus contract and to expose its columns so capture stays faithful.

**Source:** [CR-CH-075]; `.kiro/steering/workspace-conformance.md`;
`screen-snapshot-scrm` (Group A capture)

#### Acceptance Criteria

1. THE host Context's `WorkspaceContext::render` SHALL report `InteriorFocus`
   through the ONE latch path (`render_workspace_context` /
   `apply_interior_focus`), with the FIRST interior control being the FILTER
   row's first focusable control (captured as its fresh same-frame `egui::Id`) so
   the first Tab from the `Command ===>` line lands EXACTLY on it with NO phantom
   stop. THE Table_Editor SHALL NOT hand-wire a per-row focus ring.

2. THE Tab ring SHALL walk controls in visual order: `Command ===>` ->
   FILTER row fields -> for each data row, Act_Cell -> the host's column widgets
   in left-to-right order. Each per-row control SHALL carry a stable salted id
   keyed by the row's backing index so ids do not collide across rows.

3. THE displayed column set of a table editor (header order and per-row values)
   SHALL be the authoritative column set the host Context's `ScreenProvider`
   populates into a `ff-screen-model::Table` for snapshot/capture (Group A), so a
   captured snapshot of the editor renders the SAME columns in the SAME order the
   user sees.

### Requirement 8: Scope and non-goals

**User Story:** As a reviewer, I want the component's boundaries stated so that
its blast radius is clear.

**Source:** [CR-CH-075]

#### Acceptance Criteria

1. THE component SHALL be consumed by exactly the Alias, Menu, and Kinds editors
   in this change; adding a fourth consumer is additive and out of scope here.

2. THE Config editor ISPF redesign (investigation Group B item 4) is explicitly
   OUT OF SCOPE for this gate and is recorded as a follow-up; this spec SHALL NOT
   specify it.

3. THE component SHALL NOT change the `ff-line-commands` engine's public API or
   its line-command vocabulary; it REUSES the engine as-is. Any need to change the
   engine SHALL be raised as a separate change and flagged per
   `framework-conformance.md`.

4. THE component SHALL introduce no new crate dependency and no new persistence
   format; it lives in `ff-desktop` as a shared panel helper reused by the three
   editor panels (the design chooses the exact location).

## Out of Scope

- The Config editor ISPF redesign (a documented follow-up).
- Any change to the `ff-line-commands` engine's public API or vocabulary.
- Expanded key-assignment model and named reusable keylists (investigation
  Group C) -- unrelated to this component.
- Theme editor redesign and the colour-name question (investigation items in
  Groups A / C) -- unrelated to this component.
