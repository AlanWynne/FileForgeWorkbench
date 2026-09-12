# ADR-002: Unified Navigation Model and File Explorer Modernization

**Status:** Proposed
**Date:** 2026-09-12
**Relates to:** ADR-001 (Dataset Ownership Model); specs file-tree-panel,
virtual-catalog-manager, dataset-catalog, connector-local-fs, menu-workspace,
virtual-file-system
**Raised by:** CR-NR-060

---

## Context

The workbench currently ships a file explorer implemented inline in the desktop
shell (`ff-desktop/file_explorer_panel.rs`, with its own `FileExplorerPanelState`).
A separate, clean, tested tree-model crate (`ff-file-tree`) exists but is used by
NO crate (0 external Cargo dependents, 0 source references) -- an orphan
(PA-CONFLICT-011). The result is two parallel implementations of the same concept
and a shipping explorer the owner considers "clunky and old-fashioned".

Two goals must be reconciled:

1. **Modernize** the whole file-navigation and file-selection experience (look,
   feel, interaction) while **retaining the ISPF-familiar behaviours** that the
   product's users expect -- chiefly the ubiquitous command line and the ability
   to execute a command from any Workspace.

2. **Unify** navigation across heterogeneous namespaces. The explorer must present
   both **POSIX filesystems** and **Mainframe Catalogs** (and, later, remote
   connectors) and show entries **as they appear in their native environment**:
   - POSIX: `/`-separated directories and files.
   - Mainframe: `.`-separated qualifiers, where each qualifier level reads as a
     folder; sequential datasets (DSORG=PS) read as files; partitioned datasets
     (PDS/PDSE, DSORG=PO) read as folders whose members read as files inside a
     subdirectory; GDG bases read as folders of generations.

### The core problem

A mainframe name can be **simultaneously a real dataset and a qualifier prefix
for other datasets**. Example: `SYS1.PROCLIB` may exist as a sequential dataset,
while `SYS1.PROCLIB.JCL` also exists (so `SYS1.PROCLIB` is also a container). In a
POSIX filesystem a single path is either a file or a directory, never both. Any
model that keys navigation on a reconstructed path string collides on this case.

---

## Decision

### D1. Identity is a node, not a path

The tree keys every entry on an opaque node identity (`NodeId`, and at the VFS
boundary a typed resource URI), NEVER on a concatenated/reconstructed path string.
Two entries that share a display label (e.g. the qualifier group `SYS1` and part
of a real dataset name `SYS1`) are simply different nodes. Because the tree never
re-parses a path to decide "file or folder", the dataset/qualifier duality cannot
produce an identity collision.

The existing `ff-file-tree::NodeType` already encodes this: `is_expandable()` is a
per-node-type property (a `DatasetPartitioned` is BOTH a dataset AND expandable; a
`DatasetSequential` is a leaf), not a filesystem property. This ADR adopts that
model as canonical.

### D2. Dataset/qualifier duality resolves to sibling nodes

When a mainframe name is both a real dataset AND a qualifier prefix for deeper
datasets, the explorer presents TWO sibling nodes at that qualifier level, with
distinct identities and distinct affordances:

- the **dataset node** itself (openable; a leaf for PS, expandable-to-members for
  PDS/PDSE), and
- a **qualifier group node** (expandable) holding the deeper-qualifier entries.

The group node is rendered so it is unambiguous that it is a namespace grouping
rather than a dataset (for example a distinct group affordance and a
count such as "SYS1.PROCLIB.* (N datasets)"). This is honest about the mainframe
reality and maps directly onto the existing `HlqGroup` +
`DatasetSequential`/`DatasetPartitioned`/`PdsMember` node types with no
path-string ambiguity.

### D3. Namespace mapping is provider-defined

The tree UI stays generic. Each VFS provider decides how its namespace maps to
tree nodes (per FFW-ARCH-001, all I/O flows through ff-vfs providers):

| Provider | Separator | Container nodes | Leaf nodes | Members |
|----------|-----------|-----------------|------------|---------|
| POSIX (connector-local-fs) | `/` | directory | file | n/a |
| Mainframe catalog (dataset-catalog) | `.` | qualifier group, PDS/PDSE, GDG base | PS dataset, GDG generation | PDS members read as files in a subdirectory |
| Remote connectors (future) | provider-defined | provider-defined | provider-defined | provider-defined |

The unusual mainframe rules (duality, members-as-files, qualifier-as-folder) live
in the provider mapping, not in the generic tree.

### D4. ff-file-tree becomes canonical; the inline explorer is retired

The modernized explorer is built on the `ff-file-tree` model crate. The inline
`FileExplorerPanelState` reimplementation in `ff-desktop` is replaced, removing
the duplication (PA-CONFLICT-011). ff-file-tree already anticipates the mainframe
node types; the shell becomes a thin renderer over the model plus the
provider-mapping layer.

### D5. Modernization retains the ISPF command line

The presentation is modernized (layout, visuals, interaction) but the workbench
RETAINS the ISPF-familiar command line: a `Command ===>` field available on the
explorer and the ability to execute a command from ANY Workspace/Context. This is
a durable product principle, not merely a spec artefact. (It also intersects
CR-NR-057 command chaining and the menu-workspace fastpath.)

---

## Consequences

- **Positive:** one source of truth for the tree model; the orphan crate becomes
  live; POSIX and mainframe namespaces each render natively; the dataset/qualifier
  duality has a clean, collision-free resolution; the explorer can be modernized
  without losing ISPF muscle memory.
- **Tradeoff / effort:** rewiring touches the large desktop panels
  (`file_explorer_panel.rs` ~935 lines and the files-panel code) and requires the
  provider-mapping layer to be defined for POSIX and mainframe. This is a genuine
  change to product shape, so it MUST pass the requirements gate (requirements /
  design / tasks / TCR + owner approval) before any source changes.
- **Risk:** the sibling dataset+group presentation is a new UX convention; it
  needs validation against real catalog data and ISPF-user expectations.
- **Scope note:** the other orphan crates (ff-external-mod, ff-large-file-performance,
  ff-viewers, ff-idle-processing) are confirmed future-intended and are NOT deleted;
  they are wired in during their natural build phases. This ADR concerns only the
  navigation/explorer duplication.

---

## Status of decisions

| ID | Decision | Status |
|----|----------|--------|
| D1 | Node identity, not path | Proposed |
| D2 | Dataset/qualifier duality -> sibling dataset + group nodes | Proposed |
| D3 | Provider-defined namespace mapping | Proposed |
| D4 | ff-file-tree canonical; retire inline explorer | Proposed |
| D5 | Modernize presentation, retain ISPF command line | Proposed |

Nothing in this ADR is implemented until the requirements gate is run on the
affected sub-projects and the owner approves.
