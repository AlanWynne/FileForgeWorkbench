# Analysis Record: file-tree-panel (W4.3)

- **Wave**: 4 (UI, panels, layout)
- **Backing code**: SPLIT across (a) `ff-file-tree` crate = tree MODEL (node/state/
  filter/sort/keyboard/context_menu, 8 files, ~1200 lines) + (b) `ff-desktop`
  panels `file_explorer_panel.rs` (935) + `files_panel.rs` (1195) = actual RENDER.
- **Spec files**: requirements.md (1015 lines, 23 requirements -- LARGEST Wave-4
  spec), tasks.md (259 sub-tasks, all `[x]`; target BOTH ff-file-tree + ff-desktop),
  design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

The checklist pre-flag "top split candidate (674)" is a stale count; the spec is
1015 lines / 23 reqs -- the largest Wave-4 spec. BUT the real structural problem is
NOT a size split -- it is DUPLICATION (below). Assessment:

- Requirement volume: 23 reqs, 1015 lines. FAR above thresholds. (signal 1)
- Responsibilities: tree model + async loading + file-watching + rendering +
  context menus + DnD + search/filter + catalog browsing + path bar +
  accessibility + native-file-dialog integration (Req 22/23 egui-file-dialog).
  Several separable concerns. (signal 2)
- Crates: the MODEL is `ff-file-tree` (deps: thiserror/serde/serde_json ONLY -- no
  ff-vfs, no egui); the RENDER is ff-desktop shell panels. (signal 3 -- two homes)
- Cohesion: the model is cohesive; but it is DISCONNECTED from the render (below).

Meets 3-of-4, but the split proposal is subsumed by the duplication finding: the
render should CONSUME the `ff-file-tree` model rather than reimplement it. Recorded
PA-SPLIT-012 (owner-gated) noting the spec could split native-dialog integration
(Req 22/23) from the tree model, but PA-CONFLICT-011 (below) is the priority.

### Source file sizes

`ff-file-tree` crate: largest `state.rs` 322 non-test -- ALL UNDER cap (well-sized
model). The ff-desktop panels `files_panel.rs` 1195 + `file_explorer_panel.rs` 935
are SEVERELY over cap -- already recorded under VCM PA-STD-025 (W2.2). No NEW PA-STD
for the ff-file-tree crate itself.

---

## 2. Cross-unit consistency

### PA-CONFLICT-011 (NEW) -- `ff-file-tree` model is ORPHANED; shell reimplements it

The spec builds a `ff-file-tree` crate = a well-modeled multi-root tree (node/state/
filter/sort/keyboard/context_menu). BUT:

- `ff-file-tree` is used by NO crate (grep of all Cargo.toml: only its own entry).
  It is an ORPHAN model -- consumed by nobody.
- The ACTUAL, shipping file-tree UI is ff-desktop `file_explorer_panel.rs` (935) +
  `files_panel.rs` (1195), which reference `ff_file_tree` / `FileTreeState` /
  `TreeNode` ZERO times -- they define their OWN `FileExplorerPanelState` and roll
  their own tree.
- ff-desktop does NOT dep on ff-file-tree (0).

So there are TWO parallel file-tree implementations: the ff-file-tree crate model
(orphaned, well-decomposed, tested) and the ff-desktop shell panels (shipping,
severely over-cap, their own state). The spec's tasks target BOTH crates (5
ff-file-tree + 7 ff-desktop tasks) -- so the intent was model-crate + shell-render,
but the shell render DIVERGED and never consumed the model.

This is the WORST duplication found so far in the analysis: an entire tested model
crate abandoned in favour of a parallel shell reimplementation. Same anti-pattern
class as PA-CONFLICT-005 (VCM posix provider) but crate-scale. Recorded
PA-CONFLICT-011 (owner-gated, HIGH): either (a) rewire the ff-desktop file-explorer
panels to CONSUME `ff-file-tree` (node/state/filter/sort/keyboard) and delete the
duplicated shell state -- which ALSO resolves ~half of the PA-STD-025 over-cap
(files_panel 1195 / file_explorer_panel 935) by moving model logic into the crate;
OR (b) if the shell panels are canonical, DELETE the orphan ff-file-tree crate.
Option (a) is strongly preferred (the crate is tested + well-decomposed; the shell
panels are the cap violators).

### VCM overlap (W2.2) -- same panels

`files_panel.rs` is the Virtual Catalog Manager's panel (W2.2, PA-STD-025 /
PA-CONFLICT-005). So file-tree-panel + virtual-catalog-manager BOTH live in the
same ff-desktop file-explorer panels. The "unified resource explorer" (file-tree
Req 2 multi-root) and the VCM "Catalog Explorer" (POM option 1) are the SAME UI
surface. Recorded: PA-CONFLICT-011 + PA-STD-025 + PA-CONFLICT-005 all converge on
the ff-desktop file-explorer/files panels -- these should be untangled together
(the file-explorer panel refactor is a single high-value effort touching
file-tree-panel + VCM).

### Dataset catalog browsing (Req 10) -- consumes VFS catalog provider

Req 10 (Dataset Catalog Browsing) + Req 2 (multi-root over all VFS providers): the
tree renders the ff-dscatalog `catalog` provider + local + future remotes via VFS.
The MODEL (ff-file-tree) has no ff-vfs dep -- so VFS provider data is caller-injected
(consistent with the injection-trait pattern). Clean at the model layer.

### Public types and ownership

- `ff-file-tree`: TreeNode, FileTreeState, filter/sort/keyboard/context_menu model
  -- sole-owned but ORPHANED (PA-CONFLICT-011).
- ff-desktop panels: FileExplorerPanelState + duplicate tree logic -- the shipping
  impl, over-cap (PA-STD-025).
- Native file dialog (Req 22/23): egui-file-dialog integration (the workspace has an
  `egui-file-dialog` vendored crate) in the shell. Separate concern.

### Cross-reference integrity

Cross-refs (virtual-file-system, dataset-catalog, layout-and-docking) resolve. The
model-render disconnect is the issue (PA-CONFLICT-011).

---

## 3. Completeness

Tracking: all 259 sub-tasks `[x]`. The ff-file-tree MODEL is complete + tested; the
ff-desktop panels render a working file explorer. FUNCTIONALLY complete (the user
sees a working tree). But the model/render DUPLICATION (PA-CONFLICT-011) means the
259 `[x]` span two parallel implementations rather than one integrated stack -- not
a false-positive (both exist + work), but architecturally the crate is dead weight.
No PA-INCOMPLETE (functionality exists); the issue is the duplication.

### TCR gap (PA-TCR-020)

TCR.md has 5 rows for ff-file-tree/file-tree-panel against 23 reqs / ~180 criteria.
Thin relative to the largest Wave-4 spec (the orphan crate's tests exist but the
shell-panel side is under-covered). Recorded PA-TCR-020.

---

## 4. Logging audit

`ff-file-tree` crate: `ff_logging` 0, NO ff-logging dep (deps = thiserror/serde
only), 0 fs, 0 non-comment non-ASCII (73 comment-only). The MODEL is a pure data
crate -- zero-log defensible, no dead dep (it doesn't even declare ff-logging).
The ff-desktop panels' logging is part of the shell (separate). Recorded PA-LOG-029
(LOW): if the panels are rewired onto ff-file-tree (PA-CONFLICT-011 option a), add
dev-logging on async directory load / file-watch events + WARN on load failure
(Req 3/5) at that point. No standalone dead-dep issue for the model crate.

---

## 5. Task revision proposals

- **PA-CONFLICT-011 (owner-gated, HIGH)**: the `ff-file-tree` model crate is ORPHANED
  (used by no crate) while ff-desktop reimplements the file tree
  (file_explorer_panel 935 + files_panel 1195, no ff-file-tree ref). Rewire the
  shell panels to CONSUME ff-file-tree (preferred -- resolves ~half of PA-STD-025 by
  relocating model logic into the tested crate) OR delete the orphan crate if the
  shell is canonical. Untangle TOGETHER with VCM PA-STD-025 + PA-CONFLICT-005 (same
  panels). Code + owner decision.
- **PA-SPLIT-012 (owner-gated, LOW)**: the spec could split native-file-dialog
  integration (Req 22/23) from the tree model; subsumed by PA-CONFLICT-011 priority.
- **PA-TCR-020**: enumerate per-requirement TCR rows (5 rows for 23 reqs). No code.
- **PA-LOG-029 (LOW)**: dev-logging on async load / file-watch once the panels are
  rewired onto ff-file-tree.

The ff-file-tree crate itself is CLEAN (well-sized, 0 non-comment non-ASCII, no dead
dep) -- no PA-STD/ASCII for it. No requirement CHANGE proposed; the issue is
architectural duplication (PA-CONFLICT-011).

---

## Summary

file-tree-panel is the largest Wave-4 spec (23 reqs / 1015 lines) for a unified
multi-root VFS resource explorer. The `ff-file-tree` crate is a clean, well-decomposed,
tested tree MODEL (node/state/filter/sort/keyboard, all under cap, 0 non-comment
non-ASCII, no dead dep) -- BUT it is ORPHANED (used by no crate). The ACTUAL shipping
file explorer is the ff-desktop `file_explorer_panel.rs` (935) + `files_panel.rs`
(1195) shell panels, which roll their OWN tree state and reference ff-file-tree
ZERO times. PA-CONFLICT-011 (NEW, HIGH): an entire tested model crate abandoned for
a parallel shell reimplementation -- the worst duplication in the analysis. These
panels are ALSO the VCM panels (W2.2, PA-STD-025 over-cap + PA-CONFLICT-005), so
file-tree-panel + virtual-catalog-manager are the SAME ff-desktop file-explorer
surface and should be untangled together: rewire the shell to consume ff-file-tree
(preferred -- also relieves the cap violations) or delete the orphan crate. Minor:
thin TCR (PA-TCR-020), a LOW native-dialog split (PA-SPLIT-012), and load/watch
dev-logging deferred to the rewire (PA-LOG-029).
