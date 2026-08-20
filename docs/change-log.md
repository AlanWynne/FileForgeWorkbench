# FileForge Workbench — Change Log

Tracks every new requirement and change request raised via user prompts.
Entries are appended automatically by the prompt-triage rule.
Never delete a row — update `Status` in-place.

---

## Status Values

| Status | Meaning |
|--------|---------|
| `PENDING GATE` | Logged, requirements gate not yet started |
| `IN PROGRESS` | Gate running or implementation underway |
| `DONE` | Merged and tests passing |
| `DEFERRED` | Accepted but postponed to a later phase |
| `REJECTED` | Decided not to implement |

---

## New Requirements

New capabilities that did not previously exist.

### CR-NR-001 — Prompt triage and change tracking
- **Date/Phase**: Phase AS
- **Prompt**: "Can we create a steering rule that every prompt is evaluated as a bug or a new requirement"
- **Description**: Add a steering rule that classifies every user prompt as a bug, new requirement, change request, question, task, or refactor. Bugs are logged to `docs/bugs.md`; new requirements and change requests are logged to `docs/change-log.md`.
- **Status**: DONE
- **Linked spec**: `.amazonq/rules/prompt-triage.md` (new rule file)

### CR-NR-002 — File Explorer Panel (POM option 2)
- **Date/Phase**: Phase AS
- **Prompt**: "opetion 2 needs to be a file Exploere. it should have nodes for each open catalog, and list the files that belong in the catalog in a tree view. Option 2 can be invoked by typing =files and pressing enter, Typeing =2 and pressing enter"
- **Description**: POM option 2 becomes a File Explorer panel showing all open catalogs as tree nodes with their files listed beneath. Commands `=2` and `=FILES` close the current context and switch to the Files context in-place; `FILES` (no `=`) opens a new tab in the Files context.
- **Status**: PENDING GATE
- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Requirement 19

### CR-NR-004 — Default Native catalog pointing to user home directory on first launch
- **Date/Phase**: Phase AX
- **Prompt**: "By Default the when FFWB starts up if there are no native catalogs in existence, it should create a native catalog pointing to the users home directory, and mount it immediately, so that when the files context window is opened we at least can see the users home directory. Once created this default catalog should persist and be there on next start up."
- **Description**: On first launch (or any launch where the catalog registry contains no Native catalogs), FFWB shall automatically create a Native catalog named `Home` pointing to the user's home directory, register it with `auto_mount = true`, and persist it so it survives subsequent restarts.
- **Status**: DONE

### CR-NR-003 — HLQ pre-population in Allocate Dataset dialog
- **Date/Phase**: Phase AW
- **Prompt**: "when defining a dataset catalog we are asked for a default high level qualifier, this should be pre-populated in the allocate dataset dataset name text box"
- **Description**: When the Allocate Dataset dialog opens for a Mainframe catalog that has a Default HLQ configured, the Dataset Name field shall be pre-populated with that HLQ followed by a dot, so the user only needs to type the remaining qualifiers.
- **Status**: DONE
- **Linked spec**: `docs/specs/virtual-catalog-manager/requirements.md` Requirement 5.2 (new criterion 5.7)

---

## Change Requests

Modifications to existing behaviour that already works.

### CR-CH-001 — POM option 2 description update
- **Date/Phase**: Phase AS
- **Prompt**: "opetion 2 needs to be a file Exploere..."
- **Description**: POM option 2 label updated from "View Edit Create and Delete of files" to "File Explorer — Browse catalogs and files in a tree view".
- **Affects**: `ff-desktop` `primary_option_menu.rs`, `startup-and-session/requirements.md`
- **Status**: PENDING GATE

### CR-CH-002 — Home catalog deletion blocked
- **Date/Phase**: Phase AX
- **Prompt**: "Deleting the Home catalog should not be allowed!"
- **Description**: Req 14.6 revised: the `"Home"` Native catalog is protected from deletion. The Catalog Manager Dialog shall reject any delete attempt on a catalog named `"Home"` of type `Native` with an inline error. Renaming and editing remain permitted (Req 14.7).
- **Affects**: `ff-desktop` `catalog_manager_dialog.rs`, `virtual-catalog-manager/requirements.md` Req 14.6
- **Status**: DONE

---

## Changelog

| Phase | Change |
|-------|--------|
| Phase AS | File created. CR-NR-001 logged — prompt triage steering rule. |
