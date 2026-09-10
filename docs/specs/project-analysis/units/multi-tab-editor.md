# Analysis Record: multi-tab-editor (W4.2)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-tabs` (the EDITOR-TAB manager -- distinct from `ff-tabmask`
  TABS/MASK display commands, W2.7)
- **Spec files**: requirements.md (424 lines, 15 requirements), tasks.md
  (206 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 15 reqs, 424 lines (marginally over the line threshold). One
cohesive concern (open documents as a tab collection: navigate/reorder/pin/split/
close/MRU/DnD + per-tab state). 16 files, well-sized (no file over 350). No split.

---

## 2. Cross-unit consistency

### Split-view delegation to ff-layout -- CLEAN

Req 12 (Split Editor -- same document in multiple views): ff-tabs deps on ff-layout
and uses `ff_layout::TabGroupId` for split requests (error.rs, split_view.rs). So
the SPLIT/tab-group mechanics live in ff-layout (W4.1, the owner); ff-tabs owns the
tab COLLECTION + per-tab editing state + the same-document-multiple-views logic.
Clean layering (ff-tabs -> ff-layout). No duplication of the split engine.

### PA-CONFLICT-010 (NEW) -- "Tab Window Chrome" triple-specified, single impl

Req 15 (Tab Window Chrome) mandates a "three-element header (Tab_Header row +
Title_Line + Command_Line) ... regardless of tab kind or docked/floating". This is
the SAME requirement text as:
- menu-and-statusbar Req 17 (Tab Window Chrome -- Title Line and Command Line per Tab)
- layout-and-docking Req 11 (Tab Window Chrome in Detached Workspaces)

THREE specs claim "Tab Window Chrome". The actual IMPLEMENTATION is in NONE of the
three model crates -- it is in `ff-desktop/src/shell/render_chrome.rs` (render_tab_bar
+ tab-header rendering; part of the PA-STD-042 shell files). ff-tabs has 0
chrome/TitleLine/CommandLine code (grep 0). This is ownership-AMBIGUITY: one shell
implementation, three claiming specs, and the actual owner (the shell) has no
requirements spec of its own for it.

Recorded PA-CONFLICT-010 (spec-ownership, owner-gated): designate ONE owner for the
Tab Window Chrome requirement. Given the chrome is per-tab, spans docked+floating,
and is rendered by the shell, the cleanest resolution is: multi-tab-editor Req 15
(or a shell spec) OWNS it; menu-and-statusbar Req 17 + layout-and-docking Req 11
CROSS-REFERENCE it. This subsumes/extends PA-WATCH-020 (which flagged the
menu-statusbar side). Spec-hygiene + a single-owner decision; no code move (the impl
is already unified in the shell).

### Session persistence contract -- consistent (ff-session)

Req 14 (Session Persistence Contract): ff-tabs provides the tab-state contract that
ff-session persists via WorkspaceDescriptor/PersistedTabKind (W3.9/W3.10). Clean
producer(contract)/consumer(persistence) split. Consistent.

### Command-framework tab integration -- consistent

Req 13: tab operations (next/prev/close/pin/split) dispatch through command-framework
(Command_Target). Consistent with the menu/command family. Deps: ff-command +
ff-document-model + ff-layout + ff-vfs + ff-config + ff-undo-redo + ff-logging --
appropriate for an editor-tab manager (each tab holds a document + undo state).

### Public types and ownership

- Tab collection, per-tab state, tab bar/title formatting, overflow, close, context
  menu, MRU, pinned, duplicate detection, split-view (same-doc), tab keyboard nav --
  sole-owned by `ff-tabs`. No duplication (split MECHANICS delegated to ff-layout;
  chrome RENDERED by the shell -- PA-CONFLICT-010).

### Cross-reference integrity

Cross-refs resolve. No dangling refs. Req 15 chrome triple-spec -> PA-CONFLICT-010.

---

## 3. Completeness

Tracking: all 206 sub-tasks `[x]`. Implementation present across all 15 reqs
(collection, per-tab isolation, tab bar/title, overflow, close, context menu, MRU,
keyboard nav, DnD, pinned, duplicate detection, split editor, command integration,
session contract, chrome-contract). Tests in-file. No PA-INCOMPLETE. Complete.
(The chrome RENDERING lives in the shell, but the tab-model contract is complete.)

### TCR gap (PA-TCR-019)

TCR.md has 1 row for ff-tabs against 15 reqs / ~120 criteria. Thin. Recorded
PA-TCR-019.

---

## 4. Logging audit

Scan of `crates/ff-tabs/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0 (tab model; persistence via ff-session contract)

Tab open/close/split are user actions dispatched through command-framework (PA-W0.2
covers them). The resource-open-failed error (error.rs:36) is a natural WARN site.
Recorded PA-LOG-028 (LOW): WARN on resource-open failure + dev-logging on tab
open/close/split; resolve dead dep. Low priority.

---

## 5. Task revision proposals

- **PA-CONFLICT-010 (owner-gated, MEDIUM)**: designate ONE owner for the "Tab Window
  Chrome" requirement (triple-specified across multi-tab-editor Req 15,
  menu-and-statusbar Req 17, layout-and-docking Req 11; implemented once in the
  ff-desktop shell render_chrome.rs). Recommend multi-tab-editor Req 15 (or a shell
  spec) OWNS it; the other two cross-reference. Subsumes/extends PA-WATCH-020.
  Spec-hygiene + single-owner decision; no code move.
- **PA-STD-045 (ASCII, runtime string)**: 15 non-ASCII bytes (1 non-comment) --
  em-dash in a runtime `#[error]` string (error.rs:36). Replace with `--`. REFACTOR.
- **PA-LOG-028 (LOW)**: resource-open WARN + tab open/close/split dev-logging;
  resolve dead dep.
- **PA-TCR-019**: enumerate per-requirement TCR rows (1 row for 15 reqs). No code.

No PA-STD size item (no file over 350). No requirement CHANGE proposed for
multi-tab-editor; PA-CONFLICT-010 is a cross-spec ownership decision.

---

## Summary

multi-tab-editor (`ff-tabs`) is the editor-tab manager (collection + per-tab state +
navigation/pin/MRU/DnD/close/split), cleanly layered: split MECHANICS delegated to
ff-layout (W4.1) via TabGroupId, session persistence via the ff-session contract
(W3.9/W3.10), commands via command-framework. Well-sized (no file over 350),
complete (206/206). The finding is PA-CONFLICT-010 (NEW): the "Tab Window Chrome"
requirement (three-element Tab_Header + Title_Line + Command_Line header) is
TRIPLE-SPECIFIED across multi-tab-editor Req 15, menu-and-statusbar Req 17, and
layout-and-docking Req 11, but implemented ONCE in the ff-desktop shell
(render_chrome.rs) -- an ownership ambiguity that extends PA-WATCH-020; pick one
owner, cross-reference the others (no code move). Minor: runtime-string ASCII
(PA-STD-045), dead ff-logging dep (PA-LOG-028, LOW), thin TCR (PA-TCR-019). No split.
