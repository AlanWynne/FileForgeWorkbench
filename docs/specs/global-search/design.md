# Design -- Global Search

## Architectural Decisions

### 1. New crate: ff-global-search

Global Search introduces a new library crate `ff-global-search` that owns the search
orchestration logic: file enumeration, per-file search delegation, result aggregation,
and the replace pipeline. It depends on `ff-find-and-replace` (reuses FindEngine),
`ff-bgio` (background execution), and `ff-file-ops` (file read/write).

The Search Results panel UI lives in `ff-desktop`.

### 2. Background execution via ff-bgio

Search runs as a Tokio task spawned via `ff-bgio`. Results are streamed back to the UI
via an `mpsc::channel` -- the panel polls the receiver each frame and appends new matches
to the displayed list. This keeps the egui render loop unblocked.

### 3. File enumeration

File enumeration uses the `ignore` crate (already a transitive dependency via `ff-bgio`)
which respects `.gitignore` and supports glob include/exclude patterns natively.

### 4. Per-file search

Each file is read into a `String` buffer and passed to `FindEngine::find_all()` from
`ff-find-and-replace`. This reuses all existing literal/regex/case-fold logic without
duplication.

### 5. Replace pipeline

Cross-file replace reads each file, applies substitutions via `FindEngine::replace_all()`,
and writes the result via `ff-file-ops`. Each file write is wrapped in a single undo
transaction for that file's editor tab (if open). Files not open in an editor are written
directly to disk with no undo record (the user can re-run the search to verify).

### 6. Search Results panel

A new `TabKind::SearchResults` is added to `ff-desktop`. The panel state holds:
- The current query and options
- A `Vec<FileMatches>` (file path + `Vec<SearchResult>`)
- A `Receiver<SearchEvent>` for streaming results from the background task
- Replace input state

## New Crate

```
crates/ff-global-search/
  Cargo.toml   -- deps: ff-find-and-replace, ff-bgio, ff-file-ops, ignore, tokio
  src/
    lib.rs
    search.rs  -- GlobalSearchRequest, GlobalSearchEngine, search()
    replace.rs -- GlobalReplaceEngine, replace_all()
    result.rs  -- SearchResult, FileMatches, SearchEvent
```

## Module Layout (ff-desktop)

```
ff-desktop/src/
  search_results_panel/
    mod.rs     -- SearchResultsPanelState
    render.rs  -- render(), render_file_section(), render_match_row()
    state.rs   -- query, options, results, receiver
```

## SEARCH as the FFCMD canonical root verb (CR-NR-110)

This section designs the CR-NR-110 verb split for the SEARCH side. It is an ADDITIVE
design on top of the existing Global Search above: no existing crate boundary, dispatch
path, persistence format, or navigation model changes. The FIND side (in-buffer search)
is designed in `docs/specs/find-and-replace/`; the verb-split decision itself is recorded
in `docs/specs/command-semantics/` (cross-referenced below).

### Dispatch: the single front door, no second dispatcher

SEARCH is an FFCMD (workbench) verb. It is dispatched through the SAME single front door
every workbench verb uses: `dispatch_command_string` -> `resolve_target`
(`crates/ff-command/src/command_target.rs`) -> `dispatch_command_target`
(`crates/ff-desktop/src/shell/target_dispatch.rs`). FFCMD IS `resolve_target` driven by
`ShellTargetResolver` / `builtin_workspace_target_for` (see command-environments design);
SEARCH is resolved there, not by a bespoke `if upper == "SEARCH"` intercept.

Two resolution shapes are available and both fit the front door; this design CHOOSES the
Function-target shape for the engine-bearing behaviour and keeps the existing
builtin-workspace activation for opening the panel:

- Opening the Search Results Context (Requirement 1.1/1.2) is already a builtin workspace
  activation (`GSEARCH`/`SEARCH` open the `TabKind::SearchResults` panel). That seam is
  retained unchanged.
- The cross-backend SEARCH execution (Requirement 9) is modelled as a registered
  Command_Id resolved to a `CommandTarget::Function` by `resolve_target`. This keeps the
  verb inside the one classifier (a `Function` target, not a new path) and lets the
  handler accept the search string and options via the existing `TargetParams`/`CommandParams`
  argument-passing, then drive the `ff-global-search` engine and stream results to the
  Search Results Context exactly as the panel does today.

Chosen because: the panel-open keyword already exists (no churn), and expressing the
executing verb as a `Function` target is the framework-sanctioned way to add a new
workbench command with arguments without touching `CommandTarget` or adding a dispatcher.
The alternative -- a bespoke handler arm in the command ladder -- is rejected by
framework-conformance (it would be a parallel intercept outside `resolve_target`).

Because SEARCH is FFCMD-owned and NOT an FFEDIT verb, the active-environment rule
(command-environments Requirement 3/5) gives the correct result automatically: on an
editor Context FFEDIT gets first crack, does not own SEARCH, declines it, and the FFCMD
base receives it -- so SEARCH always runs the cross-backend search, never the editor's
in-buffer FIND (Requirement 7.4). FIND on an editor Context is claimed by FFEDIT (its own
alias-seed verb) and never reaches this SEARCH path.

### The GREP alias through the existing FFCMD AliasTable

GREP is recorded as an ALIAS surface for the canonical SEARCH verb through the EXISTING
AliasTable mechanism (`crates/ff-desktop/src/shell/environment.rs`, the same
`AliasTable` type whose `ffedit_english` seed maps `X -> EXCLUDE`, `INCLUDE -> SHOW`). The
FFCMD environment gains (or already carries) its own alias seed in which the identity row
`SEARCH -> SEARCH` and the alias row `GREP -> SEARCH` are added. `AliasTable::canonical_verb`
resolves `grep` (case-insensitively) to the canonical `SEARCH` BEFORE dispatch, so the
handler only ever sees SEARCH; history and persistence record SEARCH (Requirement 8.2/8.3).
Collisions are rejected at construction by the existing `ffedit_english`-style assertion
and `apply_locale_overlay` discipline (Requirement 8.4). No new alias mechanism is
introduced; this is the same seam CR-NR-103 localisation loads per-locale verb data into.

Note on placement: the FFEDIT alias seed lives in `environment.rs` today. The FFCMD alias
seed is the natural sibling; whether it is a new `AliasTable::ffcmd_english()` constructor
or an extension of the FFCMD resolver's verb table is an implementation detail for the
engine gate -- the CONTRACT (GREP resolves to canonical SEARCH via the existing AliasTable
type, behaviour-neutral) is fixed here.

### Results route to the existing Search Results Context

SEARCH surfaces matches in the existing `TabKind::SearchResults` panel (Requirement 7.3,
9.2). It reuses the existing streaming design above: the engine sends `SearchEvent` messages
over the `mpsc::channel`; the panel polls the receiver each frame and appends matches
grouped by file. Cross-backend matches from different VFS_Backends are merged into the one
results list, grouped by file, with no new Context and no new result model.

### Cross-backend scope via the VFS / catalog abstraction

The cross-backend CONTRACT (Requirement 9) requires SEARCH's file enumeration to go through
FFWB's own VFS / catalog abstraction (`ff-vfs` providers, including the `catalog` provider
from `ff-dscatalog`) rather than a raw filesystem walk or a shell-out. The existing engine
enumerates POSIX/NTFS files with the `ignore` crate; extending enumeration to span VFS
backends (so a mainframe catalog path and a POSIX root can be searched in one operation) is
part of the deferred engine work, because the dataset side depends on the Volume Model and
catalog resolution (below). This gate fixes the contract: scope = a set of files across VFS
backends; enumeration flows through the VFS abstraction; results land in the Search Results
Context.

### Deferred: engine + mainframe traversal (dependency on CR-NR-105 + dataset-catalog)

The following are explicitly DEFERRED to a follow-on gate and are NOT designed here:

- The concrete Search_Engine: whether and how a grep engine is embedded for the POSIX/NTFS
  backends (the existing per-file `FindEngine::find_all()` + `ignore` walk is a candidate
  starting point, but the embedding decision is deferred), and the per-backend matching
  implementation.
- Mainframe DATASET traversal and record-search semantics: how datasets are enumerated
  across volumes and how records within a dataset are matched. This DEPENDS ON:
  - the Volume Model (CR-NR-105, `docs/specs/volume-model/`) -- the first-class Volume layer
    and VTOC/enumeration that a dataset search must traverse; and
  - the dataset-catalog resolution mechanisms (`docs/specs/dataset-catalog/`) -- resolving a
    dataset name to its backing store and record attributes (RECFM/LRECL/encoding).
  Until both are finalised, record-aware dataset search cannot be specified without
  guessing at those contracts.
- Any backend-specific scope-addressing syntax for datasets.

When that gate runs, it will slot the engine beneath this same contract: the same SEARCH
Function target, the same AliasTable GREP alias, and the same Search Results Context remain
unchanged; only the enumeration/matching internals are added.

### Framework-conformance position (CR-NR-110)

This design BUILDS ON the framework and changes none of its load-bearing mechanisms:

- SEARCH dispatches through the single front door (`resolve_target` / `dispatch_command_target`)
  as a `Function` target; no second dispatcher.
- GREP is recorded through the existing FFCMD AliasTable alias mechanism; no new alias
  mechanism.
- Results use the existing Search Results Context (`TabKind::SearchResults`); no new
  Context and no new navigation stack.
- No new persistence format: SEARCH persists through command history / session exactly as
  other verbs, recording the canonical verb.
- The cross-backend enumeration goes through the existing `ff-vfs` provider seam; no raw
  shell-out as the primary mechanism.

No framework change is required by CR-NR-110. If the deferred engine gate later needs a
change to a core mechanism (for example a new VFS enumeration contract), that is a separate,
owner-confirmed decision at that gate, not part of this one.
