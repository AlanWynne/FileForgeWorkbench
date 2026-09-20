//! `TabManager` — manages the ordered list of open tabs.
//!
//! Owns all `TabState` instances, tracks which tab is active, and provides
//! `open_file` to load a file from the local filesystem into a new tab.

use ff_connector_local_fs::LocalFsProvider;
use ff_document_model::{new_document, BytePosition};
use ff_layout::{TabGroup, TabGroupId, TabGroupTree};
use ff_vfs::VfsProvider;
use tokio::runtime::Runtime;

use ff_layout::SplitDirection;

use crate::tab_state::{TabId, TabKind, TabState};

/// The `TabGroupId` of the root Tab_Group. When unsplit this is the sole group
/// (CR-NR-091, Slice 2a). When split it is the initial leaf; new leaves created
/// by splits are allocated from [`FIRST_SPLIT_GROUP_ID`] upward (CR-NR-093).
const ROOT_GROUP_ID: TabGroupId = TabGroupId::new(0);

/// Opaque save token for the render-only focus swap (CR-NR-092/093). Returned by
/// [`TabManager::set_render_focus_leaf`] and consumed by
/// [`TabManager::restore_render_focus`] so the split render can draw a
/// non-focused region's body without disturbing the authoritative focus.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RenderFocusToken(TabGroupId);

/// The `TabGroupId` of the FIRST group id allocated by a split (CR-NR-093).
/// [`ROOT_GROUP_ID`] is 0; the first new leaf created by a split is 1, and
/// subsequent leaves increment from there (see `next_group_id`).
const FIRST_SPLIT_GROUP_ID: u32 = 1;

/// A persistable, IDENTITY-FREE descriptor of the split arrangement (CR-NR-093,
/// Slice 2c.3). The session restore path does NOT preserve `TabId`s across a
/// restart (tabs are reopened with fresh ids), so the layout is persisted by
/// STRUCTURE -- the tree shape, each split's direction/proportion, and each
/// leaf's tab COUNT -- rather than by tab identity. On restore the tree shape is
/// rebuilt and the restored store tabs are distributed across the leaves in
/// order by these counts (then reconciled by `sync_layout`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum LayoutShape {
    /// A leaf region holding `count` tabs (in store order).
    Leaf { count: usize },
    /// A split of two child shapes.
    Split {
        /// Side-by-side (`true`) or stacked (`false`) -- serialised as a bool so
        /// the descriptor does not depend on `ff-layout` enum serde naming.
        horizontal: bool,
        /// Relative size of the first child in [0.05, 0.95].
        proportion: f32,
        /// First (left/top) child shape.
        first: Box<LayoutShape>,
        /// Second (right/bottom) child shape.
        second: Box<LayoutShape>,
    },
}

/// The persisted layout descriptor: the split [`LayoutShape`] plus the focused
/// leaf's pre-order index (CR-NR-093, Slice 2c.3). Serialised into
/// `SessionState.layout`'s `data` field.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct LayoutDescriptor {
    /// The structural shape of the split tree.
    pub shape: LayoutShape,
    /// Pre-order index of the focused leaf (0-based, left-to-right/top-to-bottom).
    pub focused_leaf: usize,
}

/// Manages all open tabs and the active tab index.
///
/// CR-NR-091 (B046 Slice 2a): the shell now models tab arrangement as a
/// [`TabGroupTree`] (the `ff-layout` layout tree). In THIS slice the tree is
/// always a single `Leaf` Tab_Group mirroring the flat store, so behaviour is
/// identical to the pre-slice flat model. The flat `tabs`/`active`/`previous_active`
/// remain the AUTHORITATIVE state; `layout` is a mirror rebuilt by
/// [`TabManager::sync_layout`] after every mutation, and `active_tab()` /
/// `active_index()` resolve THROUGH the focused group (which, with one leaf, is
/// exactly `active`). The visible split (multiple leaves) is Slice 2b.
pub struct TabManager {
    tabs: Vec<TabState>,
    active: usize,
    /// The tab index that was active immediately BEFORE the current `active`
    /// (CR-CH-031, multi-tab-editor Req 18.9). Updated on every real active-tab
    /// change so bare `SWAP` can toggle to the previously active workspace.
    /// `None` until a second distinct tab has been activated.
    previous_active: Option<usize>,
    next_id: u64,
    /// The layout tree (CR-NR-091/092/093). A single `Leaf` when unsplit; a
    /// recursive `Split` tree of arbitrary depth when split (CR-NR-093, Slice
    /// 2c.1). This is the AUTHORITATIVE arrangement model: leaves own which
    /// `TabId`s live where and which is active per group; the flat `tabs` store
    /// stays authoritative for tab CONTENT. Reconciled against the store by
    /// [`sync_layout`](Self::sync_layout) after every mutation.
    layout: TabGroupTree,
    /// The focused Tab_Group leaf id (CR-NR-091). [`ROOT_GROUP_ID`] when unsplit;
    /// a leaf id in the tree when split. `active_tab()`/`active_index()` resolve
    /// through this leaf.
    focused_group: TabGroupId,
    /// Monotonic allocator for new Tab_Group leaf ids (CR-NR-093, Slice 2c.1).
    /// [`ROOT_GROUP_ID`] (0) is the initial leaf; splits allocate from
    /// [`FIRST_SPLIT_GROUP_ID`] (1) upward so every leaf id is unique for the
    /// lifetime of the split arrangement.
    next_group_id: u32,
}

impl TabManager {
    /// Create a manager with a single untitled welcome tab.
    pub fn new(runtime: &Runtime, welcome: &str) -> Self {
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), welcome.as_bytes());
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let tab = TabState::untitled(TabId(0), document, line_count);
        let mut mgr = Self {
            tabs: vec![tab],
            active: 0,
            previous_active: None,
            next_id: 1,
            // Provisional single leaf; sync_layout below rebuilds it from the store.
            layout: TabGroupTree::Leaf(TabGroup::new(ROOT_GROUP_ID, Vec::new())),
            focused_group: ROOT_GROUP_ID,
            next_group_id: FIRST_SPLIT_GROUP_ID,
        };
        mgr.sync_layout();
        mgr
    }

    /// Rebuild the layout tree from the flat store (CR-NR-091, Slice 2a).
    ///
    /// In Slice 2a the tree is ALWAYS a single `Leaf` Tab_Group whose tab list
    /// mirrors the store's tab order (by `TabId`, stringified) and whose
    /// `active_tab` index mirrors `self.active`. Called at the end of every
    /// mutating lifecycle method so the tree can never drift from the store.
    /// The flat store remains authoritative; this keeps the mirror honest.
    ///
    /// Validates: layout-and-docking Requirement 12.1, 12.3, 12.5
    fn sync_layout(&mut self) {
        if matches!(self.layout, TabGroupTree::Leaf(_)) && self.focused_group == ROOT_GROUP_ID {
            // Unsplit path (Slice 2a/2b identical): a single Leaf mirroring the
            // flat store, byte-for-byte the pre-split behaviour.
            let tab_ids: Vec<String> = self.tabs.iter().map(|t| t.id.0.to_string()).collect();
            let active = if self.tabs.is_empty() {
                0
            } else {
                self.active.min(self.tabs.len() - 1)
            };
            let mut group = TabGroup::new(ROOT_GROUP_ID, tab_ids);
            group.active_tab = active;
            self.layout = TabGroupTree::Leaf(group);
            self.focused_group = ROOT_GROUP_ID;
            return;
        }

        // Split path (CR-NR-093, Slice 2c.1): the tree is authoritative for
        // arrangement. Reconcile it against the store: (1) drop leaf ids no
        // longer in the store; (2) clamp per-leaf actives; (3) collapse empty
        // leaves via remove_empty_groups; (4) place any store tab not present in
        // ANY leaf into the focused (or first) leaf so no tab is lost; (5) keep
        // focused_group pointing at a real leaf.
        let store_ids: std::collections::HashSet<u64> = self.tabs.iter().map(|t| t.id.0).collect();

        // (1) + (2): retain only present ids in each leaf, clamp active.
        Self::retain_leaf_ids(&mut self.layout, &store_ids);

        // (3): collapse empty leaves. If the whole tree collapses (no tabs at
        // all -- should not happen while a tab exists), fall back to a single
        // root leaf.
        let cleaned = std::mem::replace(
            &mut self.layout,
            TabGroupTree::Leaf(TabGroup::new(ROOT_GROUP_ID, Vec::new())),
        );
        self.layout = cleaned
            .remove_empty_groups()
            .unwrap_or_else(|| TabGroupTree::Leaf(TabGroup::new(ROOT_GROUP_ID, Vec::new())));

        // If only one leaf remains, we are effectively unsplit again: normalise
        // to the ROOT leaf so the unsplit fast-path above applies next time.
        if let TabGroupTree::Leaf(_) = self.layout {
            self.focused_group = ROOT_GROUP_ID;
            let tab_ids: Vec<String> = self.tabs.iter().map(|t| t.id.0.to_string()).collect();
            let active = self.active.min(self.tabs.len().saturating_sub(1));
            let mut group = TabGroup::new(ROOT_GROUP_ID, tab_ids);
            group.active_tab = active;
            self.layout = TabGroupTree::Leaf(group);
            return;
        }

        // (5): ensure focused_group names a real leaf; else focus the first leaf.
        let leaf_ids = self.layout.all_group_ids();
        if !leaf_ids.contains(&self.focused_group) {
            if let Some(first) = leaf_ids.first() {
                self.focused_group = *first;
            }
        }

        // (4): any store tab not referenced by a leaf goes into the focused leaf
        // (Req 14.13 -- no tab lost, no dangling id).
        let referenced: std::collections::HashSet<u64> = self
            .layout
            .all_tabs()
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .collect();
        let orphans: Vec<String> = self
            .tabs
            .iter()
            .filter(|t| !referenced.contains(&t.id.0))
            .map(|t| t.id.0.to_string())
            .collect();
        if !orphans.is_empty() {
            let target = self.focused_group;
            if let Some(group) = self.layout.find_group_mut(target) {
                group.tabs.extend(orphans);
            }
        }
    }

    /// Recursively retain only the leaf tab ids present in `store_ids`, clamping
    /// each leaf's `active_tab` into range (CR-NR-093 reconciliation helper).
    fn retain_leaf_ids(tree: &mut TabGroupTree, store_ids: &std::collections::HashSet<u64>) {
        match tree {
            TabGroupTree::Leaf(group) => {
                group.tabs.retain(|s| {
                    s.parse::<u64>()
                        .map(|id| store_ids.contains(&id))
                        .unwrap_or(false)
                });
                if group.tabs.is_empty() {
                    group.active_tab = 0;
                } else {
                    group.active_tab = group.active_tab.min(group.tabs.len() - 1);
                }
            }
            TabGroupTree::Split { first, second, .. } => {
                Self::retain_leaf_ids(first, store_ids);
                Self::retain_leaf_ids(second, store_ids);
            }
        }
    }

    /// Resolve the store index of the focused Tab_Group's active tab (CR-NR-091,
    /// Slice 2a). With a single leaf this is exactly `self.active`; the
    /// resolution goes through the layout tree so the same code path serves the
    /// multi-group case in Slice 2b. Falls back to `self.active` if the tree is
    /// somehow out of sync (belt-and-braces; sync_layout keeps them equal).
    ///
    /// Validates: layout-and-docking Requirement 12.4
    fn focused_active_index(&self) -> usize {
        let resolved = self.layout.find_group(self.focused_group).and_then(|g| {
            g.tabs
                .get(g.active_tab)
                .and_then(|id_str| id_str.parse::<u64>().ok())
                .and_then(|id| self.tabs.iter().position(|t| t.id.0 == id))
        });
        resolved
            .unwrap_or(self.active)
            .min(self.tabs.len().saturating_sub(1))
    }

    /// The layout tree (CR-NR-091). Slice 2a: always a single `Leaf`. Exposed for
    /// unit tests asserting the single-leaf invariant.
    #[cfg(test)]
    pub(crate) fn layout(&self) -> &TabGroupTree {
        &self.layout
    }

    /// The focused Tab_Group id (CR-NR-091). Slice 2a: always [`ROOT_GROUP_ID`].
    #[cfg(test)]
    pub(crate) fn focused_group_id(&self) -> TabGroupId {
        self.focused_group
    }

    // === CR-NR-093 (B046 Slice 2c.1): recursive in-window split ==============

    /// True when the Workspace area is split (the layout tree is not a single
    /// `Leaf`).
    pub fn is_split(&self) -> bool {
        !matches!(self.layout, TabGroupTree::Leaf(_))
    }

    /// The store index of a `TabId`, or `None`.
    fn index_of(&self, id: TabId) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == id)
    }

    /// The ordered leaf ids of the split tree (left-to-right / top-to-bottom),
    /// or a single-element vec of [`ROOT_GROUP_ID`] when unsplit. Focus traversal
    /// and full-shell tests use this; Slice 2c.2 (drag-move) will use it in the
    /// render layer too (CR-NR-093, Req 14.5).
    #[allow(dead_code)]
    pub(crate) fn leaf_ids(&self) -> Vec<TabGroupId> {
        self.layout.all_group_ids()
    }

    /// The focused leaf id (CR-NR-093). Render layer uses this to highlight the
    /// Focused_Group.
    pub(crate) fn focused_leaf_id(&self) -> TabGroupId {
        self.focused_group
    }

    /// Read-only view of the layout tree for the render walk (CR-NR-093).
    pub(crate) fn layout_tree(&self) -> &TabGroupTree {
        &self.layout
    }

    /// Split the FOCUSED Tab_Group in `direction` (CR-NR-093, Req 14.1).
    ///
    /// Unlike Slice 2b, this nests to ARBITRARY depth: the focused leaf is
    /// replaced in-tree by a `Split` whose first child is that leaf (keeping its
    /// tabs) and whose second child is a new leaf holding a fresh POM tab; focus
    /// moves to the new leaf. Always succeeds (returns `true`).
    ///
    /// Validates: layout-and-docking Requirement 14.1, 14.2
    pub fn split_focused(&mut self, direction: SplitDirection, runtime: &Runtime) -> bool {
        // A fresh POM tab for the new group (Req 13.3, retained for 2c).
        let document = ff_document_model::new_document();
        let new_tab_id = TabId(self.next_id);
        self.next_id += 1;
        self.tabs.push(TabState::pom(new_tab_id, document));

        // Ensure the tree reflects the current store before splitting (the
        // unsplit fast-path builds the root leaf from the store).
        if matches!(self.layout, TabGroupTree::Leaf(_)) && self.focused_group == ROOT_GROUP_ID {
            // Build the root leaf WITHOUT the just-pushed new tab (it belongs to
            // the new second group, not the first).
            let ids: Vec<String> = self
                .tabs
                .iter()
                .filter(|t| t.id != new_tab_id)
                .map(|t| t.id.0.to_string())
                .collect();
            let active = self.active.min(ids.len().saturating_sub(1));
            let mut root = TabGroup::new(ROOT_GROUP_ID, ids);
            root.active_tab = active;
            self.layout = TabGroupTree::Leaf(root);
        }

        let new_group_id = TabGroupId::new(self.next_group_id);
        self.next_group_id += 1;
        let new_group = TabGroup::new(new_group_id, vec![new_tab_id.0.to_string()]);

        let target = self.focused_group;
        let split_ok = self.layout.split_leaf(target, direction, 0.5, new_group);
        if split_ok {
            self.focused_group = new_group_id; // focus the new group (Req 13.3)
                                               // Make the store `active` point at the new group's tab.
            if let Some(idx) = self.index_of(new_tab_id) {
                self.active = idx;
            }
        }
        let _ = runtime;
        split_ok
    }

    /// Collapse the split around the FOCUSED leaf (CR-NR-093, Req 14.3).
    ///
    /// No-op when unsplit. The focused leaf's tabs are moved into the NEXT leaf
    /// in tree order (so no tab is lost), the focused leaf is emptied, and
    /// `remove_empty_groups` collapses it -- when only one leaf remains the tree
    /// returns to a single `Leaf` (fully unsplit). Focus moves to the leaf that
    /// absorbed the tabs.
    ///
    /// Validates: layout-and-docking Requirement 14.3
    pub fn unsplit(&mut self) {
        if !self.is_split() {
            return;
        }
        let leaves = self.layout.all_group_ids();
        let focused = self.focused_group;
        // Pick the sibling to absorb the focused leaf's tabs: the next leaf in
        // order, else the previous one.
        let focused_pos = leaves.iter().position(|id| *id == focused).unwrap_or(0);
        let absorber = leaves
            .get(focused_pos + 1)
            .or_else(|| focused_pos.checked_sub(1).and_then(|p| leaves.get(p)))
            .copied();
        let Some(absorber) = absorber else {
            return; // only one leaf -- already effectively unsplit
        };
        // Move the focused leaf's tab ids into the absorber (append), then empty
        // the focused leaf so remove_empty_groups (in sync_layout) collapses it.
        let moved: Vec<String> = self
            .layout
            .find_group(focused)
            .map(|g| g.tabs.clone())
            .unwrap_or_default();
        if let Some(dst) = self.layout.find_group_mut(absorber) {
            dst.tabs.extend(moved);
        }
        if let Some(src) = self.layout.find_group_mut(focused) {
            src.tabs.clear();
        }
        self.focused_group = absorber;
        self.sync_layout();
        // Keep the store `active` pointing at the focused leaf's active tab.
        self.active = self.focused_active_index();
        self.previous_active = None;
    }

    /// Move focus to the NEXT Tab_Group leaf in tree order, cycling (CR-NR-093,
    /// Req 14.5). No-op if unsplit.
    ///
    /// Validates: layout-and-docking Requirement 14.5
    pub fn focus_other_group(&mut self) {
        if !self.is_split() {
            return;
        }
        let leaves = self.layout.all_group_ids();
        if leaves.len() < 2 {
            return;
        }
        let pos = leaves
            .iter()
            .position(|id| *id == self.focused_group)
            .unwrap_or(0);
        self.focused_group = leaves[(pos + 1) % leaves.len()];
        self.active = self.focused_active_index();
    }

    /// Re-attach a re-docked tab into a Tab_Group leaf (CR-NR-093, Slice 2c.4,
    /// Req 14.16). Used by the `DOCK` command when the Workspace is split: a
    /// Detached_Workspace's tab, on re-dock, lands in a tree leaf rather than at
    /// a flat index. The detached tab typically still sits in the leaf it was
    /// detached from (detach only flags `is_floating`, it does not remove the tab
    /// from the tree), so this focuses that owning leaf and makes the tab active
    /// there. If the tab is in NO leaf (e.g. reconciled out), it is placed in the
    /// focused leaf so it is never lost. Returns the leaf the tab ended up in, or
    /// `None` when unsplit / the tab is unknown (the caller then uses the flat
    /// re-dock path).
    ///
    /// Validates: layout-and-docking Requirement 14.16
    pub(crate) fn dock_tab_into_leaf(&mut self, tab_id: TabId) -> Option<TabGroupId> {
        if !self.is_split() {
            return None;
        }
        let store_idx = self.index_of(tab_id)?;
        let id_str = tab_id.0.to_string();
        // Which leaf, if any, currently owns the tab?
        let owner = self.layout.all_group_ids().into_iter().find(|gid| {
            self.layout
                .find_group(*gid)
                .map(|g| g.tabs.contains(&id_str))
                .unwrap_or(false)
        });
        match owner {
            Some(leaf) => {
                // Already in a leaf (its origin leaf): focus it and make active.
                self.focus_leaf_and_activate(leaf, store_idx);
                Some(leaf)
            }
            None => {
                // Not in any leaf: place it in the focused leaf so it is not lost.
                let target = self.focused_group;
                if self.layout.find_group(target).is_some() {
                    if let Some(g) = self.layout.find_group_mut(target) {
                        g.tabs.push(id_str);
                        g.active_tab = g.tabs.len() - 1;
                    }
                    self.sync_layout();
                    self.active = self.focused_active_index();
                    Some(target)
                } else {
                    None
                }
            }
        }
    }

    /// Move the tab `tab_id` into Tab_Group `target` (CR-NR-093, Slice 2c.2,
    /// Req 14.6). Removes the tab id from whichever leaf currently owns it,
    /// appends it to the target leaf, makes it the target's active tab, and
    /// focuses the target. If the source leaf empties, it is collapsed via
    /// `sync_layout` (`remove_empty_groups`, Req 14.7). No-op when unsplit, when
    /// `target` is not a leaf, when `tab_id` is not in the store, or when the tab
    /// already belongs to `target` (Req 14.8: a drop onto its own region).
    ///
    /// Returns `true` when a move actually happened.
    ///
    /// Validates: layout-and-docking Requirement 14.6, 14.7, 14.8
    pub(crate) fn move_tab_to_group(&mut self, tab_id: TabId, target: TabGroupId) -> bool {
        if !self.is_split() {
            return false;
        }
        // The tab must exist in the store.
        if self.index_of(tab_id).is_none() {
            return false;
        }
        // The target must be a real leaf.
        if self.layout.find_group(target).is_none() {
            return false;
        }
        let id_str = tab_id.0.to_string();
        // Find the leaf that currently owns the tab.
        let owner = self.layout.all_group_ids().into_iter().find(|gid| {
            self.layout
                .find_group(*gid)
                .map(|g| g.tabs.contains(&id_str))
                .unwrap_or(false)
        });
        let Some(owner) = owner else {
            return false; // not in any leaf (should not happen while split)
        };
        if owner == target {
            return false; // Req 14.8: drop onto own region is a no-op
        }
        // Remove from the source leaf.
        if let Some(src) = self.layout.find_group_mut(owner) {
            src.tabs.retain(|s| *s != id_str);
            if src.active_tab >= src.tabs.len() {
                src.active_tab = src.tabs.len().saturating_sub(1);
            }
        }
        // Append to the target leaf and make it active there.
        if let Some(dst) = self.layout.find_group_mut(target) {
            dst.tabs.push(id_str);
            dst.active_tab = dst.tabs.len() - 1;
        }
        // Focus the target; sync collapses an emptied source (Req 14.7) and
        // reconciles. Then point the flat active at the focused leaf's tab.
        self.focused_group = target;
        self.sync_layout();
        self.active = self.focused_active_index();
        self.previous_active = None;
        true
    }

    /// Persist the current split arrangement as an identity-free structural
    /// descriptor (CR-NR-093, Slice 2c.3, Req 14.10). Returns `None` when unsplit
    /// (an unsplit workbench writes no layout, Req 14.12). The descriptor records
    /// the tree shape, per-split direction/proportion, per-leaf tab COUNT, and the
    /// focused leaf's pre-order index -- NOT tab ids (which do not survive a
    /// restart).
    ///
    /// Validates: layout-and-docking Requirement 14.10, 14.12
    pub(crate) fn layout_snapshot(&self) -> Option<LayoutDescriptor> {
        if !self.is_split() {
            return None;
        }
        let shape = Self::shape_of(&self.layout);
        // Focused leaf pre-order index.
        let focused_leaf = self
            .layout
            .all_group_ids()
            .iter()
            .position(|id| *id == self.focused_group)
            .unwrap_or(0);
        Some(LayoutDescriptor {
            shape,
            focused_leaf,
        })
    }

    /// Build the structural [`LayoutShape`] of a tree (CR-NR-093 helper).
    fn shape_of(tree: &TabGroupTree) -> LayoutShape {
        match tree {
            TabGroupTree::Leaf(g) => LayoutShape::Leaf {
                count: g.tabs.len(),
            },
            TabGroupTree::Split {
                direction,
                proportion,
                first,
                second,
            } => LayoutShape::Split {
                horizontal: matches!(direction, SplitDirection::Horizontal),
                proportion: *proportion,
                first: Box::new(Self::shape_of(first)),
                second: Box::new(Self::shape_of(second)),
            },
        }
    }

    /// Restore a split arrangement from a persisted [`LayoutDescriptor`]
    /// (CR-NR-093, Slice 2c.3, Req 14.11). Rebuilds the tree SHAPE and distributes
    /// the CURRENT (restored) store tabs across the leaves in order by the saved
    /// per-leaf counts; `sync_layout` then reconciles any count mismatch (leftover
    /// store tabs go to the focused/first leaf; empty leaves collapse -- Req
    /// 14.13). No-op when there are fewer than two tabs to place or the descriptor
    /// is a single leaf (nothing to split).
    ///
    /// Validates: layout-and-docking Requirement 14.11, 14.13
    pub(crate) fn restore_layout(&mut self, desc: &LayoutDescriptor) {
        // Nothing to restore into an empty/single-tab store, or a non-split shape.
        if self.tabs.len() < 2 || matches!(desc.shape, LayoutShape::Leaf { .. }) {
            return;
        }
        // Store tab ids in current order; distributed across leaves by count.
        let store_ids: Vec<String> = self.tabs.iter().map(|t| t.id.0.to_string()).collect();
        let mut cursor = 0usize;
        // Reset the group-id allocator so rebuilt leaves get fresh sequential ids.
        self.next_group_id = FIRST_SPLIT_GROUP_ID;
        let mut root_used = false;
        let tree = self.build_from_shape(&desc.shape, &store_ids, &mut cursor, &mut root_used);
        self.layout = tree;
        // Any store tabs not yet placed (count mismatch) are appended to the
        // first leaf so none is lost; sync_layout also handles this, but place
        // them deterministically here first.
        if cursor < store_ids.len() {
            let leftover: Vec<String> = store_ids[cursor..].to_vec();
            if let Some(first_leaf_id) = self.layout.all_group_ids().first().copied() {
                if let Some(g) = self.layout.find_group_mut(first_leaf_id) {
                    g.tabs.extend(leftover);
                }
            }
        }
        // Focus the saved leaf by pre-order index (clamped).
        let leaves = self.layout.all_group_ids();
        if !leaves.is_empty() {
            self.focused_group = leaves[desc.focused_leaf.min(leaves.len() - 1)];
        }
        // Reconcile (clamp actives, drop empties, place orphans) and re-resolve
        // the flat active through the focused leaf.
        self.sync_layout();
        if self.is_split() {
            self.active = self.focused_active_index();
        }
    }

    /// Recursively build a `TabGroupTree` from a [`LayoutShape`], consuming
    /// `store_ids[*cursor..]` by each leaf's count (CR-NR-093 helper). The first
    /// leaf built reuses [`ROOT_GROUP_ID`]; subsequent leaves allocate fresh ids.
    fn build_from_shape(
        &mut self,
        shape: &LayoutShape,
        store_ids: &[String],
        cursor: &mut usize,
        root_used: &mut bool,
    ) -> TabGroupTree {
        match shape {
            LayoutShape::Leaf { count } => {
                let end = (*cursor + *count).min(store_ids.len());
                let ids: Vec<String> = store_ids[*cursor..end].to_vec();
                *cursor = end;
                let id = if *root_used {
                    let gid = TabGroupId::new(self.next_group_id);
                    self.next_group_id += 1;
                    gid
                } else {
                    *root_used = true;
                    ROOT_GROUP_ID
                };
                TabGroupTree::Leaf(TabGroup::new(id, ids))
            }
            LayoutShape::Split {
                horizontal,
                proportion,
                first,
                second,
            } => {
                let f = self.build_from_shape(first, store_ids, cursor, root_used);
                let s = self.build_from_shape(second, store_ids, cursor, root_used);
                TabGroupTree::Split {
                    direction: if *horizontal {
                        SplitDirection::Horizontal
                    } else {
                        SplitDirection::Vertical
                    },
                    proportion: proportion.clamp(0.05, 0.95),
                    first: Box::new(f),
                    second: Box::new(s),
                }
            }
        }
    }

    /// Render-support (CR-NR-093): set `proportion` on the split node identified
    /// by `first_leaf` (the first leaf id of its first child), clamped. Used by
    /// the recursive render walk so each Splitter drags its OWN node.
    pub(crate) fn set_node_proportion(&mut self, first_leaf: TabGroupId, proportion: f32) {
        Self::set_node_proportion_rec(&mut self.layout, first_leaf, proportion.clamp(0.05, 0.95));
    }

    fn set_node_proportion_rec(
        tree: &mut TabGroupTree,
        first_leaf: TabGroupId,
        value: f32,
    ) -> bool {
        if let TabGroupTree::Split {
            proportion,
            first,
            second,
            ..
        } = tree
        {
            if first.all_group_ids().first() == Some(&first_leaf) {
                *proportion = value;
                return true;
            }
            return Self::set_node_proportion_rec(first, first_leaf, value)
                || Self::set_node_proportion_rec(second, first_leaf, value);
        }
        false
    }

    /// Render-support (CR-NR-093): the store indices of the tabs in leaf `id`, in
    /// display order. Ids no longer present are skipped. Empty when the leaf is
    /// unknown.
    pub(crate) fn leaf_tab_store_indices(&self, id: TabGroupId) -> Vec<usize> {
        let Some(group) = self.layout.find_group(id) else {
            return Vec::new();
        };
        group
            .tabs
            .iter()
            .filter_map(|s| s.parse::<u64>().ok())
            .filter_map(|raw| self.tabs.iter().position(|t| t.id.0 == raw))
            .collect()
    }

    /// Render-support (CR-NR-093): the store index of leaf `id`'s active tab, or
    /// `None` when the leaf is unknown / empty.
    pub(crate) fn leaf_active_store_index(&self, id: TabGroupId) -> Option<usize> {
        let group = self.layout.find_group(id)?;
        group
            .tabs
            .get(group.active_tab)
            .and_then(|s| s.parse::<u64>().ok())
            .and_then(|raw| self.tabs.iter().position(|t| t.id.0 == raw))
    }

    /// Render-support (CR-NR-093, Req 14.6 focus routing): focus leaf `id` and
    /// make the tab at store index `store_index` its active tab. Used when a tab
    /// header in a specific region is clicked. No-op when unsplit or the leaf/
    /// index does not resolve.
    pub(crate) fn focus_leaf_and_activate(&mut self, id: TabGroupId, store_index: usize) {
        if !self.is_split() {
            return;
        }
        if self.layout.find_group(id).is_some() {
            self.focused_group = id;
        }
        self.activate(store_index);
    }

    /// Render-support (CR-NR-093): focus leaf `id` without changing its active
    /// tab (used when clicking a region's body area). No-op when the leaf is
    /// unknown.
    pub(crate) fn focus_leaf(&mut self, id: TabGroupId) {
        if self.layout.find_group(id).is_some() {
            self.focused_group = id;
            self.active = self.focused_active_index();
        }
    }

    /// Render-only (CR-NR-093): retarget `active_tab()`/`active_tab_mut()` at leaf
    /// `id`'s active tab, returning an opaque token to restore the real focus
    /// resolution via [`restore_render_focus`]. Lets the split render draw EACH
    /// region's body through the shared `render_active_tab_body` without mutating
    /// the authoritative focus. The caller MUST pair every call with
    /// `restore_render_focus`.
    #[must_use = "restore the focus with restore_render_focus"]
    pub(crate) fn set_render_focus_leaf(&mut self, id: TabGroupId) -> RenderFocusToken {
        let saved = RenderFocusToken(self.focused_group);
        if self.layout.find_group(id).is_some() {
            self.focused_group = id;
        }
        saved
    }

    /// Restore the focus resolution saved by [`set_render_focus_leaf`]
    /// (CR-NR-093, render-only).
    pub(crate) fn restore_render_focus(&mut self, token: RenderFocusToken) {
        self.focused_group = token.0;
    }

    /// Set the active tab index, recording the outgoing index as the
    /// Previous_Active_Tab (CR-CH-031, Req 18.9). The single internal seam every
    /// activation path uses. A no-op activation (same index) does NOT clobber
    /// the previous pointer, so a bare-`SWAP` toggle target survives. Clamps to
    /// the valid range.
    fn activate(&mut self, index: usize) {
        let clamped = index.min(self.tabs.len().saturating_sub(1));
        if clamped != self.active {
            self.previous_active = Some(self.active);
            self.active = clamped;
        }
        // CR-NR-093: while split, the activation targets the FOCUSED leaf. Map
        // the store index to a TabId; if that tab belongs to ANOTHER leaf, leave
        // focus where it is (activating an other-leaf tab is not an interaction
        // path). If the tab is in the focused leaf, set that leaf's active to it;
        // if it is in NO leaf (a just-opened tab -- openers push to the store then
        // activate the last index), add it to the focused leaf and make it active
        // there. This keeps open/switch acting on the focused region (Req 14.5).
        if self.is_split() {
            if let Some(raw) = self.tabs.get(clamped).map(|t| t.id.0) {
                let id_str = raw.to_string();
                let focused = self.focused_group;
                // Which leaf, if any, currently owns this tab?
                let owner = self.layout.all_group_ids().into_iter().find(|gid| {
                    self.layout
                        .find_group(*gid)
                        .map(|g| g.tabs.contains(&id_str))
                        .unwrap_or(false)
                });
                match owner {
                    Some(gid) if gid == focused => {
                        if let Some(g) = self.layout.find_group_mut(focused) {
                            if let Some(pos) = g.tabs.iter().position(|s| *s == id_str) {
                                g.active_tab = pos;
                            }
                        }
                    }
                    Some(_) => { /* belongs to another leaf -- do not steal */ }
                    None => {
                        if let Some(g) = self.layout.find_group_mut(focused) {
                            g.tabs.push(id_str);
                            g.active_tab = g.tabs.len() - 1;
                        }
                    }
                }
            }
        }
        // CR-NR-091/092/093: keep the layout mirror in lockstep with the store.
        self.sync_layout();
    }

    /// The tab index that was active immediately before the current one, or
    /// `None` when there is no distinct previous tab (CR-CH-031, Req 18.9).
    /// Returns `None` if the recorded index no longer resolves or equals the
    /// current active (so a stale/dangling pointer never toggles).
    pub fn previous_active_index(&self) -> Option<usize> {
        self.previous_active
            .filter(|&p| p < self.tabs.len() && p != self.active)
    }

    /// Number of open tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Active tab index -- resolved through the focused Tab_Group (CR-NR-091).
    ///
    /// With the single-leaf layout of Slice 2a this equals `self.active`, so
    /// every existing caller is unaffected. The resolution goes through the
    /// layout tree so the same accessor serves the multi-group case later.
    ///
    /// Validates: layout-and-docking Requirement 12.4
    pub fn active_index(&self) -> usize {
        self.focused_active_index()
    }

    /// Find the current index of the tab with the given stable `TabId`, or
    /// `None` if no such tab exists. Used by detach/redock to track a detached
    /// tab across reorderings by identity rather than a drifting index
    /// (CR-CH-035, menu-and-statusbar Req 18.9).
    pub fn index_of_id(&self, id: crate::tab_state::TabId) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == id)
    }

    /// Set the active tab by index. Clamps to valid range. Records the outgoing
    /// tab as the Previous_Active_Tab (CR-CH-031).
    pub fn set_active(&mut self, index: usize) {
        self.activate(index);
    }

    /// Immutable slice of all tabs (for rendering the tab bar).
    pub fn tabs(&self) -> &[TabState] {
        &self.tabs
    }

    /// Mutable slice of all tabs.
    pub fn tabs_mut(&mut self) -> &mut Vec<TabState> {
        &mut self.tabs
    }

    /// Mutable reference to the active tab -- the focused Tab_Group's active tab
    /// (CR-NR-091; single-leaf Slice 2a resolves to `self.active`).
    ///
    /// Validates: layout-and-docking Requirement 12.4
    pub fn active_tab_mut(&mut self) -> &mut TabState {
        let idx = self.focused_active_index();
        &mut self.tabs[idx]
    }

    /// Immutable reference to the active tab -- the focused Tab_Group's active
    /// tab (CR-NR-091; single-leaf Slice 2a resolves to `self.active`).
    ///
    /// Validates: layout-and-docking Requirement 12.4
    pub fn active_tab(&self) -> &TabState {
        &self.tabs[self.focused_active_index()]
    }

    /// Close the initial welcome/placeholder tab if it is the only tab and has no path.
    ///
    /// Called before inserting the POM tab on first launch so the POM is the
    /// sole tab at index 0 rather than sitting behind a blank welcome tab.
    /// Validates: Requirement 14.1 — POM is always in first position on launch.
    pub fn close_welcome_tab(&mut self) {
        if self.tabs.len() == 1 && self.tabs[0].path.is_none() {
            // Replace the single placeholder tab with an empty vec; insert_pom_tab
            // will add the real first tab immediately after.
            // We cannot call close_tab (it guards len >= 1), so swap directly.
            self.tabs.clear();
            self.active = 0;
            self.previous_active = None;
            self.sync_layout(); // CR-NR-091
        }
    }

    /// Insert a Primary Option Menu tab at index 0 and make it active.
    ///
    /// Inserts a new POM tab and makes it active.
    /// Validates: Requirement 14.1, 14.13
    pub fn insert_pom_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::pom(id, document);
        self.tabs.insert(0, tab);
        // Insert-at-0 shifts every existing index up by one, invalidating the
        // Previous_Active_Tab pointer; a fresh POM has no meaningful previous
        // (CR-CH-031). Reset it rather than track the shift.
        self.active = 0;
        self.previous_active = None;
        self.sync_layout(); // CR-NR-091
        let _ = runtime;
    }

    /// Insert a new untitled editor tab and make it active.
    ///
    /// Validates: Requirement 14.9
    pub fn new_untitled_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::untitled(id, document, 1);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Files Panel (Virtual Catalog Manager) tab.
    ///
    /// If a FilesPanel tab already exists, activates it instead of inserting a duplicate.
    /// Validates: Requirement 1.1, 11.2
    pub fn open_files_panel_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self.tabs.iter().position(|t| t.kind == TabKind::FilesPanel) {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::files_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Config Panel tab (the flat config-key browser).
    ///
    /// If a ConfigPanel tab already exists, activates it instead of inserting a duplicate.
    /// Validates: Requirement 15.1, 15.9
    pub fn open_config_panel_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::ConfigPanel)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::config_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the File Explorer Panel (POM option 2) tab.
    ///
    /// Always opens a new tab (unlike FilesPanel which deduplicates).
    /// Validates: Requirement 19.3, 19.11
    pub fn open_file_explorer_panel_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::file_explorer_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open a new Search Results panel tab.
    ///
    /// Validates: global-search Requirement 1.1
    pub fn open_search_results_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::search_results_panel(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Plugin Manager panel tab (POM option 8).
    ///
    /// If a PluginManager tab already exists, activates it instead of inserting a duplicate.
    /// Validates: plugin-manager-ui Requirement 1.1
    pub fn open_plugin_manager_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::PluginManager)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::plugin_manager(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Event Log panel tab.
    ///
    /// If an EventLog tab already exists, activates it instead of inserting a duplicate.
    /// Validates: notification-system Requirement 2.1
    pub fn open_event_log_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self.tabs.iter().position(|t| t.kind == TabKind::EventLog) {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::event_log(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Macro Library panel tab (POM option 6 / MACROS / =6).
    ///
    /// If a MacroLibrary tab already exists, activates it instead of inserting a duplicate.
    /// Validates: lua-macro-engine Requirement 12.1
    pub fn open_macro_library_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::MacroLibrary)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::macro_library(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Command Configurator tab (COMMANDS).
    ///
    /// If a CommandConfigurator tab already exists, activates it instead of
    /// inserting a duplicate.
    /// Validates: command-configurator Requirement 2.1
    pub fn open_command_configurator_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::CommandConfigurator)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::command_configurator(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Theme Editor tab, activating an existing one rather than
    /// inserting a duplicate.
    /// Validates: theme-and-appearance Requirement 20.1
    // CR-CH-022: navigation now transforms in place (navigate_to); this
    // dedicated-tab opener is retained for session restore and potential
    // detached-window use, but is not called by the in-place navigation path.
    #[allow(dead_code)]
    pub fn open_theme_editor_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::ThemeEditor)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::theme_editor(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open the Menus Editor tab, activating an existing one rather than
    /// inserting a duplicate.
    ///
    /// Validates: menu-workspace Requirement 13.1 (CR-NR-075)
    // CR-CH-022: retained for session restore / detached windows; in-place
    // navigation (navigate_to) does not use it.
    #[allow(dead_code)]
    pub fn open_menus_editor_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::MenusEditor)
        {
            self.activate(idx);
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::menus_editor(id, document);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Open a data-driven Menu Workspace tab backed by `<menus_dir>/<name>.toml`.
    ///
    /// If a Menu Workspace tab already backed by the same file exists, it is
    /// activated instead of opening a duplicate. A missing file opens the tab in
    /// its load-error state (`MenuWorkspaceState` retains `load_error`), rather
    /// than doing nothing (menu-workspace Requirement 11.4).
    ///
    /// Validates: menu-workspace Requirement 11.2, 11.4
    pub fn open_menu_workspace_tab(
        &mut self,
        name: &str,
        menus_dir: &std::path::Path,
        limits: crate::menu_workspace::OptionLimits,
        runtime: &Runtime,
    ) {
        let file_path = menus_dir.join(format!("{name}.toml"));
        if let Some(idx) = self.tabs.iter().position(|t| {
            t.kind == TabKind::MenuWorkspace
                && t.menu_workspace
                    .as_ref()
                    .map(|mw| mw.file_path == file_path)
                    .unwrap_or(false)
        }) {
            self.activate(idx);
            return;
        }
        let mw_state =
            crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = crate::tab_state::TabState::menu_workspace_tab(id, document, mw_state);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        let _ = runtime;
    }

    /// Transform the active tab in-place from the Home Context (POM) to a new
    /// kind.
    ///
    /// No-op if the active tab is not the Home Context. Clears the `is_home`
    /// marker so the tab becomes an ordinary Context of the requested kind.
    /// Validates: Requirement 14.6
    // CR-CH-022: superseded by the shell-level navigate_to (which transforms ANY
    // current tab in place and pushes the Navigation_Stack). Retained as a
    // low-level primitive exercised by tab-manager unit tests.
    #[allow(dead_code)]
    pub fn transform_active_pom_tab(&mut self, kind: TabKind, title: &str) {
        let tab = &mut self.tabs[self.active];
        if tab.is_home {
            tab.kind = kind;
            tab.title = title.to_string();
            tab.is_home = false;
            tab.menu_workspace = None;
        }
    }

    /// Open a data-driven Menu Workspace backed by `<menus_dir>/<name>.toml`,
    /// transforming the active tab in place when it is a `MenuWorkspace` (this
    /// includes the Home Context) or a `ConfigPanel` (so the POM -> Settings_Menu
    /// / Config chain stays on one
    /// tab and F3/END transforms back to the POM), otherwise
    /// opening (or activating) a dedicated tab.
    ///
    /// Validates: cw-requirements.md Requirement 9.1, 10.4; menu-workspace Req 11.2
    // CR-CH-022: superseded by the shell reconstruct path (reconstruct_settings_menu
    // / reconstruct_named_menu drive Menu_Workspace transforms in place). Retained
    // for reference / potential reuse.
    #[allow(dead_code)]
    pub fn open_menu_workspace_here(
        &mut self,
        name: &str,
        menus_dir: &std::path::Path,
        limits: crate::menu_workspace::OptionLimits,
        runtime: &Runtime,
    ) {
        let active_kind = self.active_tab().kind;
        let transform_in_place = matches!(
            active_kind,
            TabKind::ConfigPanel | TabKind::MenuWorkspace | TabKind::MenusEditor
        );
        if transform_in_place {
            let file_path = menus_dir.join(format!("{name}.toml"));
            let mw_state =
                crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
            let title = mw_state.tab_title();
            let tab = &mut self.tabs[self.active];
            tab.kind = TabKind::MenuWorkspace;
            tab.title = title;
            tab.is_home = false;
            tab.menu_workspace = Some(mw_state);
        } else {
            self.open_menu_workspace_tab(name, menus_dir, limits, runtime);
        }
    }

    ///
    /// If the file is already open (same path), activates the existing tab
    /// instead of opening a duplicate.
    ///
    /// Returns `Err(message)` if the file cannot be read.
    pub fn open_file(&mut self, path: &str, runtime: &Runtime) -> Result<(), String> {
        // Duplicate detection — activate existing tab if already open.
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.path.as_deref() == Some(path))
        {
            self.activate(idx);
            return Ok(());
        }

        let bytes = runtime.block_on(async {
            let provider =
                LocalFsProvider::with_defaults().map_err(|e| format!("VFS init failed: {e}"))?;
            provider
                .read(path)
                .await
                .map_err(|e| format!("Cannot read '{path}': {e}"))
        })?;

        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), &bytes);
        });
        let (line_count, line_end_mode) = runtime.block_on(async {
            let doc = document.read().await;
            (doc.line_count(), doc.line_end_mode())
        });

        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::for_file(id, path.to_string(), document, line_count, line_end_mode);
        self.tabs.push(tab);
        self.activate(self.tabs.len() - 1);
        Ok(())
    }

    /// Save the active tab's document to its associated file path.
    ///
    /// Returns `Err` if the tab has no path (untitled) or the write fails.
    /// On success, clears `is_modified` and marks the document save point.
    pub fn save_active_tab(&mut self, runtime: &Runtime) -> Result<(), String> {
        let tab = &mut self.tabs[self.active];
        let path = tab
            .path
            .as_deref()
            .ok_or_else(|| "Cannot save: no file path (untitled document)".to_string())?;
        let path = path.to_string();

        let bytes = runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let view = doc.contiguous_view().to_vec();
            view
        });

        runtime.block_on(async {
            let provider =
                LocalFsProvider::with_defaults().map_err(|e| format!("VFS init failed: {e}"))?;
            provider
                .write(&path, &bytes)
                .await
                .map_err(|e| format!("Save failed: {e}"))
        })?;

        // Clear dirty flag and mark save point
        tab.is_modified = false;
        runtime.block_on(async {
            tab.document.write().await.set_save_point();
        });
        Ok(())
    }

    /// Close the tab at `index`. If it is the active tab, activates the
    /// nearest remaining tab. Always keeps at least one tab open.
    #[allow(dead_code)]
    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(index);

        // Repair the active index for the removal shift (indices > index shift
        // down by one). This is index bookkeeping, not a user activation, so it
        // does NOT go through `activate`.
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        } else if index < self.active {
            self.active -= 1;
        }

        // Repair the Previous_Active_Tab pointer for the same shift (CR-CH-031):
        // clear it if it pointed at the closed tab, else shift it down when it
        // was after the removed index. `previous_active_index()` also guards
        // against an out-of-range/equal-to-active value, so this is belt-and-braces.
        self.previous_active = match self.previous_active {
            Some(p) if p == index => None,
            Some(p) if p > index => Some(p - 1),
            other => other,
        };
        self.sync_layout(); // CR-NR-091
    }

    /// Remove and return the tab at `index`, repairing `active` /
    /// `previous_active` for the removal shift (indices above `index` shift down
    /// by one). Unlike [`close_tab`], this has NO minimum-one guard and does not
    /// choose a neighbour to activate: it is a low-level reordering primitive for
    /// faithful redock (CR-CH-035, menu-and-statusbar Req 18.9). Panics only if
    /// `index` is out of range (caller's contract).
    ///
    /// Validates: menu-and-statusbar Requirement 18.9
    pub fn remove_at(&mut self, index: usize) -> TabState {
        let tab = self.tabs.remove(index);
        if self.active > index {
            self.active -= 1;
        } else if self.active == index {
            // The removed tab was active; clamp into range so `active` stays
            // valid. The caller (redock) typically reinserts immediately.
            self.active = self.active.min(self.tabs.len().saturating_sub(1));
        }
        self.previous_active = match self.previous_active {
            Some(p) if p == index => None,
            Some(p) if p > index => Some(p - 1),
            other => other,
        };
        self.sync_layout(); // CR-NR-091
        tab
    }

    /// Insert `tab` at `index` (clamped to `0..=len`), repairing `active` /
    /// `previous_active` for the insertion shift (indices at or above `index`
    /// shift up by one). The inserted tab is NOT auto-activated; the caller
    /// decides. Companion to [`remove_at`] for faithful redock at the origin
    /// position (CR-CH-035, menu-and-statusbar Req 18.9).
    ///
    /// Validates: menu-and-statusbar Requirement 18.9
    pub fn insert_at(&mut self, index: usize, tab: TabState) {
        let idx = index.min(self.tabs.len());
        self.tabs.insert(idx, tab);
        if self.active >= idx {
            self.active += 1;
        }
        self.previous_active = self
            .previous_active
            .map(|p| if p >= idx { p + 1 } else { p });
        self.sync_layout(); // CR-NR-091
    }

    /// Move the tab at `from` to position `to` (clamped to the valid range),
    /// preserving the relative order of the other tabs (a remove-then-insert,
    /// NOT a positional swap). Used by redock to restore a detached tab to its
    /// origin index (CR-CH-035, menu-and-statusbar Req 18.9). Returns the tab's
    /// final index. The moved tab keeps its identity/content/cursor/profile
    /// (same `TabState`). A no-op when `from == to`.
    ///
    /// Validates: menu-and-statusbar Requirement 18.9
    pub fn move_tab(&mut self, from: usize, to: usize) -> usize {
        if from >= self.tabs.len() {
            return from.min(self.tabs.len().saturating_sub(1));
        }
        let was_active = self.active == from;
        let tab = self.remove_at(from);
        let target = to.min(self.tabs.len());
        self.insert_at(target, tab);
        if was_active {
            self.active = target;
        }
        self.sync_layout(); // CR-NR-091 (after the final active assignment)
        target
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    /// Validates: Requirement 14.13 — POM tab title is [POM].
    #[test]
    fn pom_tab_title_is_pom() {
        // Validates: Requirement 14.13
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.tabs()[0].title, "[POM]");
    }

    /// Validates: Requirement 14.1, menu-workspace 18.1 -- the Home tab is a
    /// MenuWorkspace flagged `is_home`.
    #[test]
    fn pom_tab_has_kind_primary_option_menu() {
        // Validates: Requirement 14.1; menu-workspace Requirement 18.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert!(mgr.tabs()[0].is_home);
        assert_eq!(mgr.tabs()[0].kind, TabKind::MenuWorkspace);
    }

    /// Validates: Requirement 14.1 — POM tab is inserted at index 0.
    #[test]
    fn pom_tab_inserted_at_index_zero() {
        // Validates: Requirement 14.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.active_index(), 0);
        assert!(mgr.tabs()[0].is_home);
    }

    /// Validates: Requirement 14.1 — inserting POM twice opens two POM tabs.
    #[test]
    fn insert_pom_tab_twice_opens_two_pom_tabs() {
        // Validates: Requirement 14.1 — START always opens a new POM tab
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        let count_after_first = mgr.len();
        mgr.insert_pom_tab(&runtime);
        assert_eq!(
            mgr.len(),
            count_after_first + 1,
            "second insert_pom_tab must open a new POM tab"
        );
        assert!(mgr.active_tab().is_home);
    }

    /// Validates: Requirement 14.9 — new_untitled_tab adds an Untitled tab.
    #[test]
    fn new_untitled_tab_adds_untitled_kind() {
        // Validates: Requirement 14.9
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        let before = mgr.len();
        mgr.new_untitled_tab(&runtime);
        assert_eq!(mgr.len(), before + 1);
        assert_eq!(mgr.active_tab().kind, TabKind::Untitled);
    }

    /// Validates: Requirement 14.1 — file tab has kind FileEditor.
    #[test]
    fn file_tab_has_kind_file_editor() {
        // Validates: Requirement 14.1
        use std::io::Write;
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "hello").expect("write");
        let path = tmp.path().to_string_lossy().into_owned();
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_file(&path, &runtime).expect("open");
        assert_eq!(mgr.active_tab().kind, TabKind::FileEditor);
    }

    #[test]
    fn save_writes_document_content_to_file() {
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_string_lossy().into_owned();

        let mut mgr = TabManager::new(&runtime, "");
        // Replace the welcome tab with a file-backed tab at the temp path
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"saved content");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let tab = TabState::for_file(
            TabId(1),
            path.clone(),
            document,
            line_count,
            ff_document_model::LineEndMode::Default,
        );
        mgr.tabs[0] = tab;

        let result = mgr.save_active_tab(&runtime);
        assert!(result.is_ok(), "save should succeed: {result:?}");

        let written = std::fs::read(&path).expect("read back");
        assert_eq!(written, b"saved content");
    }

    /// Validates: file-operations Requirement 1.2 — save clears the modified flag.
    #[test]
    fn save_clears_modified_flag() {
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_string_lossy().into_owned();

        let mut mgr = TabManager::new(&runtime, "");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::for_file(
            TabId(1),
            path.clone(),
            document,
            line_count,
            ff_document_model::LineEndMode::Default,
        );
        tab.is_modified = true;
        mgr.tabs[0] = tab;

        mgr.save_active_tab(&runtime).expect("save");
        assert!(!mgr.active_tab().is_modified);
    }

    /// Validates: file-operations Requirement 1.4 — save on untitled tab returns error.
    #[test]
    fn save_on_untitled_tab_is_noop() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "some content");
        // The default welcome tab is untitled (no path)
        let result = mgr.save_active_tab(&runtime);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("untitled"));
    }

    /// Validates: task 18.4 — TabManager starts with one untitled tab.
    #[test]
    fn new_manager_has_one_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "hello\n");
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_tab().title, "Untitled");
    }

    /// Validates: task 18.4 — opening a missing file returns an error.
    #[test]
    fn open_nonexistent_file_returns_error() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        let result = mgr.open_file("/nonexistent/path/file.txt", &runtime);
        assert!(result.is_err());
        assert_eq!(mgr.len(), 1); // no new tab added
    }

    /// Validates: task 18.5 — closing a tab reduces count; last tab is preserved.
    #[test]
    fn close_tab_preserves_minimum_one() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.close_tab(0);
        assert_eq!(mgr.len(), 1); // cannot go below 1
    }

    /// Helper: a manager with N titled tabs (title = "T{i}") for order assertions.
    #[cfg(test)]
    fn mgr_with_titled(runtime: &Runtime, n: usize) -> TabManager {
        let mut mgr = TabManager::new(runtime, "");
        mgr.tabs[0].title = "T0".to_string();
        for i in 1..n {
            mgr.new_untitled_tab(runtime);
            let last = mgr.tabs.len() - 1;
            mgr.tabs[last].title = format!("T{i}");
        }
        mgr
    }

    /// Validates: menu-and-statusbar Req 18.9 -- remove_at returns the tab and
    /// shifts higher indices down, preserving the order of the rest.
    #[test]
    fn remove_at_returns_tab_and_preserves_order() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4); // T0 T1 T2 T3
        let removed = mgr.remove_at(1);
        assert_eq!(removed.title, "T1");
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["T0", "T2", "T3"]);
    }

    /// Validates: menu-and-statusbar Req 18.9 -- insert_at places the tab at the
    /// index and shifts the rest up, preserving order.
    #[test]
    fn insert_at_places_tab_and_preserves_order() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 3); // T0 T1 T2
        let removed = mgr.remove_at(2); // T2 out; [T0 T1]
        mgr.insert_at(1, removed); // -> [T0 T2 T1]
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["T0", "T2", "T1"]);
    }

    /// Validates: menu-and-statusbar Req 18.9 -- move_tab restores a tab to its
    /// origin index preserving the order of the other tabs (NOT a swap).
    #[test]
    fn move_tab_restores_to_origin_preserving_order() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4); // T0 T1 T2 T3
                                                    // Simulate a tab that was detached from index 1 and now sits at the end
                                                    // (as if appended); move it back to origin 1.
        let t1 = mgr.remove_at(1); // [T0 T2 T3]
        mgr.insert_at(mgr.len(), t1); // [T0 T2 T3 T1]
        let moved_from = mgr.tabs().iter().position(|t| t.title == "T1").unwrap();
        mgr.move_tab(moved_from, 1); // back to origin 1
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["T0", "T1", "T2", "T3"],
            "move_tab must reinsert at origin, not swap (order preserved)"
        );
    }

    /// Validates: menu-and-statusbar Req 18.9 -- move_tab keeps the moved tab
    /// active when it was active, tracking its new index.
    #[test]
    fn move_tab_follows_active_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4); // active = 3 (last)
        mgr.set_active(3);
        mgr.move_tab(3, 0);
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_tab().title, "T3");
    }

    /// Validates: menu-and-statusbar Req 18.9 -- move_tab clamps an origin index
    /// beyond the current count to the end (append), per Req 18.3.
    #[test]
    fn move_tab_clamps_origin_beyond_count_to_end() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 3); // T0 T1 T2
        let final_idx = mgr.move_tab(0, 99); // origin beyond count -> append
        assert_eq!(final_idx, 2);
        let titles: Vec<&str> = mgr.tabs().iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["T1", "T2", "T0"]);
    }

    // Validates: multi-tab-editor Req 18.9 (CR-CH-031) -- activating a different
    // tab records the outgoing index as the Previous_Active_Tab; a no-op
    // activation does not clobber it.
    #[test]
    fn previous_active_tracks_last_other_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.new_untitled_tab(&runtime); // now 2 tabs, active = 1, prev = 0
        assert_eq!(mgr.active_index(), 1);
        assert_eq!(mgr.previous_active_index(), Some(0));

        mgr.set_active(0); // active = 0, prev = 1
        assert_eq!(mgr.previous_active_index(), Some(1));

        // A no-op activation (same index) must NOT change the previous pointer.
        mgr.set_active(0);
        assert_eq!(mgr.previous_active_index(), Some(1));
    }

    // Validates: multi-tab-editor Req 18.9 -- a single-tab manager has no
    // distinct previous tab.
    #[test]
    fn previous_active_none_with_single_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "");
        assert_eq!(mgr.previous_active_index(), None);
    }

    // Validates: multi-tab-editor Req 18.9 -- closing a tab repairs the
    // Previous_Active_Tab pointer (clears it if it pointed at the closed tab).
    #[test]
    fn previous_active_repaired_on_close() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.new_untitled_tab(&runtime); // 2 tabs, active 1, prev 0
        mgr.new_untitled_tab(&runtime); // 3 tabs, active 2, prev 1
        assert_eq!(mgr.previous_active_index(), Some(1));
        // Close the previous tab (index 1); the pointer must not dangle.
        mgr.close_tab(1);
        assert_eq!(
            mgr.previous_active_index(),
            None,
            "closing the previous tab clears the dangling pointer"
        );
    }

    /// Validates: task 18.5 — set_active clamps to valid range.
    #[test]
    fn set_active_clamps_to_valid_range() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.set_active(999);
        assert_eq!(mgr.active_index(), 0);
    }

    /// Validates: Requirement 18.7 — untitled tab reports correct line_count.
    #[test]
    fn untitled_tab_line_count_matches_content() {
        // Validates: Requirement 7.4 — total line count segment shows real count
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "line1\nline2\nline3\n");
        // 3 newlines → 4 lines (last empty line after final \n)
        assert_eq!(mgr.active_tab().line_count, 4);
    }

    /// Validates: Requirement 18.7 — untitled tab encoding_label defaults to UTF-8.
    #[test]
    fn untitled_tab_encoding_label_is_utf8() {
        // Validates: Requirement 7.3 — encoding segment shows detected encoding
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "hello\n");
        assert_eq!(mgr.active_tab().encoding_label(), "UTF-8");
    }

    // ── Phase AM: Detachable tab windows ────────────────────────────────────

    /// Validates: Requirement 18.4 — is_floating defaults to false on new tabs.
    #[test]
    fn floating_tab_is_floating_flag_defaults_to_false() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "");
        assert!(!mgr.active_tab().is_floating);
    }

    /// Validates: Requirement 18.4 — is_floating can be set to true.
    #[test]
    fn floating_tab_is_floating_flag_can_be_set() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.tabs_mut()[0].is_floating = true;
        assert!(mgr.active_tab().is_floating);
    }

    /// Validates: Requirement 18.4 — POM tab also defaults is_floating to false.
    #[test]
    fn pom_tab_is_floating_defaults_to_false() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert!(!mgr.tabs()[0].is_floating);
    }

    /// Validates: Requirement 1.1, 11.2 — option 1 opens a FilesPanel tab with title [FILES].
    #[test]
    fn files_panel_tab_has_kind_files_panel_and_title() {
        // Validates: Requirement 1.1, 11.2
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_files_panel_tab(&runtime);
        let tab = mgr.active_tab();
        assert_eq!(tab.kind, TabKind::FilesPanel);
        // CR-NR-090 B.1: the Virtual Catalog Manager (Catalogs Kind) title.
        assert_eq!(tab.title, "[CATALOGS]");
    }

    /// Validates: Requirement 1.1 — opening FilesPanel twice does not duplicate it.
    #[test]
    fn open_files_panel_tab_twice_does_not_duplicate() {
        // Validates: Requirement 1.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_files_panel_tab(&runtime);
        let count = mgr.len();
        mgr.open_files_panel_tab(&runtime);
        assert_eq!(mgr.len(), count, "second open must not add a duplicate");
    }

    // === CR-NR-091 (B046 Slice 2a): Shell Layout Tree foundation invariants ===

    use ff_layout::TabGroupTree;

    /// The leaf's `(tab-id list in order, active index)`, or panics if the tree
    /// is not a single `Leaf` (the Slice 2a invariant).
    fn leaf_snapshot(mgr: &TabManager) -> (Vec<u64>, usize) {
        match mgr.layout() {
            TabGroupTree::Leaf(g) => {
                let ids: Vec<u64> = g.tabs.iter().map(|s| s.parse::<u64>().unwrap()).collect();
                (ids, g.active_tab)
            }
            TabGroupTree::Split { .. } => panic!("Slice 2a invariant violated: tree is a Split"),
        }
    }

    /// Assert the single-leaf invariant: the tree is one `Leaf` whose tab-id
    /// order equals the store's tab order and whose active index equals the
    /// store's active index; and `active_tab()` returns the focused group's
    /// active tab (Req 12.4/12.8).
    fn assert_layout_mirrors_store(mgr: &TabManager) {
        let (leaf_ids, leaf_active) = leaf_snapshot(mgr);
        let store_ids: Vec<u64> = mgr.tabs().iter().map(|t| t.id.0).collect();
        assert_eq!(leaf_ids, store_ids, "leaf tab order must mirror the store");
        assert_eq!(
            leaf_active,
            mgr.active_index(),
            "leaf active index must mirror active_index()"
        );
        // active_tab() resolves through the focused group to the store tab.
        let expected_id = store_ids[mgr.active_index()];
        assert_eq!(
            mgr.active_tab().id.0,
            expected_id,
            "active_tab() must resolve through the focused group"
        );
        assert_eq!(mgr.focused_group_id().value(), 0, "single leaf is group 0");
    }

    /// Validates: layout-and-docking Req 12.1/12.8 -- a fresh manager is a single
    /// Leaf mirroring the store.
    #[test]
    fn layout_tree_is_single_leaf_on_new() {
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "hello\n");
        assert_layout_mirrors_store(&mgr);
    }

    /// Validates: layout-and-docking Req 12.5/12.8 -- the single-leaf invariant
    /// holds after each lifecycle operation (open/new/set_active/close).
    #[test]
    fn layout_tree_mirrors_store_after_each_operation() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        assert_layout_mirrors_store(&mgr);

        mgr.insert_pom_tab(&runtime);
        assert_layout_mirrors_store(&mgr);

        mgr.new_untitled_tab(&runtime);
        assert_layout_mirrors_store(&mgr);

        mgr.open_files_panel_tab(&runtime);
        assert_layout_mirrors_store(&mgr);

        mgr.set_active(0);
        assert_layout_mirrors_store(&mgr);

        mgr.set_active(2);
        assert_layout_mirrors_store(&mgr);

        mgr.close_tab(1);
        assert_layout_mirrors_store(&mgr);
    }

    /// Validates: layout-and-docking Req 12.5 -- the invariant survives the
    /// detach/redock reorder primitives (remove_at / insert_at / move_tab).
    #[test]
    fn layout_tree_mirrors_store_through_detach_redock_primitives() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4); // T0 T1 T2 T3
        assert_layout_mirrors_store(&mgr);

        let t1 = mgr.remove_at(1); // [T0 T2 T3]
        assert_layout_mirrors_store(&mgr);

        mgr.insert_at(mgr.len(), t1); // [T0 T2 T3 T1]
        assert_layout_mirrors_store(&mgr);

        let from = mgr.tabs().iter().position(|t| t.title == "T1").unwrap();
        mgr.move_tab(from, 1); // back to origin
        assert_layout_mirrors_store(&mgr);
    }

    /// Validates: layout-and-docking Req 12.4 -- active_tab() returns EXACTLY the
    /// store tab at active_index() (behaviour-identical shim), for every active
    /// index in a multi-tab manager.
    #[test]
    fn active_tab_resolves_through_focused_group_for_all_indices() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 4);
        for i in 0..mgr.len() {
            mgr.set_active(i);
            assert_eq!(mgr.active_index(), i);
            assert_eq!(
                mgr.active_tab().title,
                format!("T{i}"),
                "active_tab() must equal the store tab at the active index"
            );
        }
    }

    // === CR-NR-092 (B046 Slice 2b): visible in-window split ==================

    use ff_layout::SplitDirection;

    /// Validates: layout-and-docking Req 13.1/13.3 -- SPLIT creates a two-leaf
    /// Split node; the first group keeps the current tabs; the second is a new
    /// POM tab; focus moves to the new (second) group.
    #[test]
    fn split_focused_creates_two_groups_second_is_pom() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 2); // T0 T1, one group
        assert!(!mgr.is_split());
        let created = mgr.split_focused(SplitDirection::Horizontal, &runtime);
        assert!(created, "first SPLIT must create a split");
        assert!(mgr.is_split());
        match mgr.layout() {
            ff_layout::TabGroupTree::Split {
                direction,
                proportion,
                first,
                second,
            } => {
                assert_eq!(*direction, SplitDirection::Horizontal);
                assert!((*proportion - 0.5).abs() < f32::EPSILON);
                // First group keeps the two original tabs.
                match first.as_ref() {
                    ff_layout::TabGroupTree::Leaf(g) => assert_eq!(g.tabs.len(), 2),
                    _ => panic!("first child must be a leaf"),
                }
                // Second group has exactly the new POM tab.
                match second.as_ref() {
                    ff_layout::TabGroupTree::Leaf(g) => assert_eq!(g.tabs.len(), 1),
                    _ => panic!("second child must be a leaf"),
                }
            }
            _ => panic!("layout must be a Split after split_focused"),
        }
        // Focus is on the new group -> active tab is the POM (is_home).
        assert!(
            mgr.active_tab().is_home,
            "focus moves to the new group whose active tab is the POM"
        );
    }

    /// Validates: layout-and-docking Req 14.1/14.2 -- a second SPLIT NESTS (no
    /// "one split only" limit): splitting the focused leaf again produces three
    /// leaves at depth >= 2.
    #[test]
    fn second_split_nests_to_three_leaves() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 2);
        assert!(mgr.split_focused(SplitDirection::Vertical, &runtime));
        assert_eq!(mgr.leaf_ids().len(), 2, "first split -> two leaves");
        // Second SPLIT on the (now focused, new) leaf nests it.
        assert!(
            mgr.split_focused(SplitDirection::Horizontal, &runtime),
            "a second SPLIT must succeed (nesting, not rejected)"
        );
        assert_eq!(mgr.leaf_ids().len(), 3, "second split -> three leaves");
        // The tree contains a nested Split (depth >= 2).
        fn max_depth(tree: &ff_layout::TabGroupTree) -> usize {
            match tree {
                ff_layout::TabGroupTree::Leaf(_) => 1,
                ff_layout::TabGroupTree::Split { first, second, .. } => {
                    1 + max_depth(first).max(max_depth(second))
                }
            }
        }
        assert!(
            max_depth(mgr.layout()) >= 3,
            "nesting must produce a tree of depth >= 3 (two split levels)"
        );
    }

    /// Validates: layout-and-docking Req 13.7/13.8 -- focus_other_group flips the
    /// focused group, and active_tab() follows the focused group.
    #[test]
    fn focus_other_group_flips_focus_and_active_tab_follows() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 1); // T0
        mgr.split_focused(SplitDirection::Horizontal, &runtime);
        // After split, focus is on the new POM group.
        assert!(mgr.active_tab().is_home);
        // Flip focus to the first group -> active tab is T0.
        mgr.focus_other_group();
        assert_eq!(mgr.active_tab().title, "T0");
        // Flip back -> POM.
        mgr.focus_other_group();
        assert!(mgr.active_tab().is_home);
    }

    /// Validates: layout-and-docking Req 13.9 -- UNSPLIT collapses to a single
    /// Leaf, preserving the focused (surviving) group's active tab.
    #[test]
    fn unsplit_collapses_to_single_leaf_preserving_survivor() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 2); // T0 T1; active = T1 (last opened)
        mgr.set_active(0); // make T0 the first group's active before splitting
        assert_eq!(mgr.active_tab().title, "T0");
        mgr.split_focused(SplitDirection::Horizontal, &runtime); // focus = POM group
        mgr.focus_other_group(); // survivor = the T0/T1 group, active T0
        assert_eq!(mgr.active_tab().title, "T0");
        mgr.unsplit();
        assert!(!mgr.is_split());
        // Single leaf again.
        assert!(matches!(mgr.layout(), ff_layout::TabGroupTree::Leaf(_)));
        // The survivor's active tab (T0) is the active tab; the POM tab survives
        // in the store (no open tab lost).
        assert_eq!(mgr.active_tab().title, "T0");
        assert!(
            mgr.tabs().iter().any(|t| t.is_home),
            "the other group's POM tab is preserved after unsplit"
        );
    }

    /// Validates: layout-and-docking Req 13.9 -- closing the focused group's last
    /// tab auto-collapses the split (via sync_layout reconciliation).
    #[test]
    fn closing_last_tab_of_a_group_auto_collapses() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 1); // T0
        mgr.split_focused(SplitDirection::Horizontal, &runtime); // second = POM, focused
                                                                 // The focused (second) group has a single tab (POM). Close it from the
                                                                 // store: find the POM tab's index and close_tab it.
        let pom_idx = mgr.tabs().iter().position(|t| t.is_home).expect("pom");
        mgr.close_tab(pom_idx);
        assert!(
            !mgr.is_split(),
            "emptying a group collapses the split back to one region"
        );
        assert!(matches!(mgr.layout(), ff_layout::TabGroupTree::Leaf(_)));
    }

    /// Validates: layout-and-docking Req 13.5/14.4 -- set_node_proportion clamps
    /// to [0.05, 0.95] on the addressed split node (root node's first leaf id is
    /// ROOT_GROUP_ID = 0).
    #[test]
    fn set_split_proportion_clamps() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 1);
        mgr.split_focused(SplitDirection::Horizontal, &runtime);
        mgr.set_node_proportion(ROOT_GROUP_ID, 2.0);
        assert!(
            (root_proportion(mgr.layout()) - 0.95).abs() < f32::EPSILON,
            "clamped to 0.95, got {}",
            root_proportion(mgr.layout())
        );
        mgr.set_node_proportion(ROOT_GROUP_ID, -1.0);
        assert!(
            (root_proportion(mgr.layout()) - 0.05).abs() < f32::EPSILON,
            "clamped to 0.05, got {}",
            root_proportion(mgr.layout())
        );
    }

    /// Validates: layout-and-docking Req 14.5 -- while split, opening a new tab
    /// adds it to the FOCUSED leaf and makes it active there (does not touch the
    /// other leaf).
    #[test]
    fn opening_a_tab_while_split_targets_the_focused_group() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 1); // T0 in the root leaf
        mgr.split_focused(SplitDirection::Horizontal, &runtime); // focus new leaf (POM)
        let focused = mgr.focused_leaf_id();
        let other = mgr
            .leaf_ids()
            .into_iter()
            .find(|id| *id != focused)
            .expect("two leaves");
        // Open an untitled tab: it must join the focused leaf.
        mgr.new_untitled_tab(&runtime);
        assert_eq!(mgr.active_tab().kind, TabKind::Untitled);
        // Focused leaf now has POM + the new untitled = 2 tabs; the other leaf 1.
        assert_eq!(mgr.leaf_tab_store_indices(focused).len(), 2);
        assert_eq!(mgr.leaf_tab_store_indices(other).len(), 1);
    }

    /// Test helper: the proportion of the outermost `Split` node, or NaN if the
    /// tree is a single leaf.
    fn root_proportion(tree: &ff_layout::TabGroupTree) -> f32 {
        match tree {
            ff_layout::TabGroupTree::Split { proportion, .. } => *proportion,
            ff_layout::TabGroupTree::Leaf(_) => f32::NAN,
        }
    }

    // === CR-NR-093 Slice 2c.2: move a tab between groups =====================

    /// Validates: layout-and-docking Req 14.6 -- moving a tab to another leaf
    /// removes it from the source, appends it to the target as the active tab,
    /// and focuses the target.
    #[test]
    fn move_tab_to_group_moves_and_focuses_target() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 2); // T0 T1 in the root leaf
        mgr.set_active(0);
        mgr.split_focused(SplitDirection::Horizontal, &runtime); // new POM leaf focused
        let leaves = mgr.leaf_ids();
        let root = leaves[0]; // holds T0 T1
        let other = leaves[1]; // holds the new POM, currently focused
                               // Move T0 from the root leaf into the other (POM) leaf.
        let t0 = mgr.tabs().iter().find(|t| t.title == "T0").unwrap().id;
        let moved = mgr.move_tab_to_group(t0, other);
        assert!(moved, "move must succeed");
        // Root leaf now holds only T1; the other leaf holds POM + T0.
        assert_eq!(mgr.leaf_tab_store_indices(root).len(), 1);
        assert_eq!(mgr.leaf_tab_store_indices(other).len(), 2);
        // Target is focused and T0 is its active tab.
        assert_eq!(mgr.focused_leaf_id(), other);
        assert_eq!(mgr.active_tab().title, "T0");
    }

    /// Validates: layout-and-docking Req 14.7 -- a move that empties the source
    /// leaf collapses the split (via sync_layout / remove_empty_groups).
    #[test]
    fn move_tab_emptying_source_collapses_split() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 1); // T0 in the root leaf
        mgr.split_focused(SplitDirection::Horizontal, &runtime); // root=[T0], other=[POM] focused
        let leaves = mgr.leaf_ids();
        let root = leaves[0];
        let other = leaves[1];
        // Move T0 (the root leaf's only tab) into the other leaf -> root empties.
        let t0 = mgr.tabs().iter().find(|t| t.title == "T0").unwrap().id;
        assert!(mgr.move_tab_to_group(t0, other));
        assert!(
            !mgr.is_split(),
            "emptying the source leaf must collapse the split"
        );
        assert!(matches!(mgr.layout(), ff_layout::TabGroupTree::Leaf(_)));
        // No tab lost: both T0 and the POM survive in the store.
        assert!(mgr.tabs().iter().any(|t| t.title == "T0"));
        assert!(mgr.tabs().iter().any(|t| t.is_home));
        let _ = root;
    }

    /// Validates: layout-and-docking Req 14.8 -- a move to the leaf that already
    /// owns the tab is a no-op (returns false, arrangement unchanged).
    #[test]
    fn move_tab_to_own_group_is_noop() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 2); // T0 T1
        mgr.set_active(0);
        mgr.split_focused(SplitDirection::Horizontal, &runtime);
        let root = mgr.leaf_ids()[0]; // holds T0 T1
        let t0 = mgr.tabs().iter().find(|t| t.title == "T0").unwrap().id;
        // T0 already lives in `root`; moving it there is a no-op.
        let moved = mgr.move_tab_to_group(t0, root);
        assert!(!moved, "move to own group must be a no-op (false)");
        assert_eq!(mgr.leaf_tab_store_indices(root).len(), 2);
    }

    /// Validates: layout-and-docking Req 14.6 -- move is a no-op for an unknown
    /// tab id or when unsplit.
    #[test]
    fn move_tab_noop_when_unsplit_or_unknown() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 1);
        // Unsplit: any move is a no-op.
        let t0 = mgr.tabs()[0].id;
        assert!(!mgr.move_tab_to_group(t0, ROOT_GROUP_ID));
        // Split, then try an unknown tab id.
        mgr.split_focused(SplitDirection::Horizontal, &runtime);
        let other = mgr.leaf_ids()[1];
        assert!(!mgr.move_tab_to_group(TabId(9_999), other));
    }

    // === CR-NR-093 Slice 2c.3: split persistence ============================

    /// Validates: layout-and-docking Req 14.12 -- an unsplit manager produces no
    /// layout snapshot (so the session writes no layout and opens unsplit).
    #[test]
    fn layout_snapshot_is_none_when_unsplit() {
        let runtime = Runtime::new().expect("runtime");
        let mgr = mgr_with_titled(&runtime, 3);
        assert!(mgr.layout_snapshot().is_none());
    }

    /// Validates: layout-and-docking Req 14.10 -- a split produces a structural
    /// snapshot capturing the tree shape, direction, and focused leaf.
    #[test]
    fn layout_snapshot_captures_shape_and_focus() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = mgr_with_titled(&runtime, 2);
        mgr.split_focused(SplitDirection::Vertical, &runtime); // 2 leaves, focus = leaf 1
        let snap = mgr.layout_snapshot().expect("split -> snapshot");
        assert_eq!(snap.focused_leaf, 1, "focused leaf index recorded");
        match snap.shape {
            LayoutShape::Split {
                horizontal,
                first,
                second,
                ..
            } => {
                assert!(!horizontal, "SPLIT DOWN -> vertical (horizontal=false)");
                assert!(matches!(*first, LayoutShape::Leaf { .. }));
                assert!(matches!(*second, LayoutShape::Leaf { .. }));
            }
            _ => panic!("expected a Split shape"),
        }
    }

    /// Validates: layout-and-docking Req 14.10, 14.11 -- snapshot -> restore
    /// round-trips the tree SHAPE (structure + direction + leaf count) and the
    /// focused leaf, distributing the current store tabs across the leaves.
    #[test]
    fn layout_snapshot_restore_round_trips_shape() {
        let runtime = Runtime::new().expect("runtime");
        // Build a depth-2 split: SPLIT (2 leaves), then SPLIT again (3 leaves).
        let mut mgr = mgr_with_titled(&runtime, 3);
        mgr.split_focused(SplitDirection::Horizontal, &runtime);
        mgr.split_focused(SplitDirection::Vertical, &runtime);
        let snap = mgr.layout_snapshot().expect("split -> snapshot");
        let leaves_before = mgr.leaf_ids().len();

        // A fresh manager with the same number of store tabs, then restore.
        let mut restored = mgr_with_titled(&runtime, mgr.len());
        assert!(!restored.is_split(), "starts unsplit");
        restored.restore_layout(&snap);

        assert!(restored.is_split(), "restore rebuilds the split");
        assert_eq!(
            restored.leaf_ids().len(),
            leaves_before,
            "restore reproduces the same number of leaves"
        );
        // The restored snapshot equals the original snapshot's shape (round-trip).
        let snap2 = restored.layout_snapshot().expect("restored -> snapshot");
        assert_eq!(snap2.shape, snap.shape, "tree shape round-trips");
    }

    /// Validates: layout-and-docking Req 14.13 -- restore is self-consistent when
    /// the store has MORE tabs than the saved layout's total count: no tab is
    /// lost (leftover tabs land in a leaf), and no leaf references a missing tab.
    #[test]
    fn restore_layout_reconciles_extra_store_tabs() {
        let runtime = Runtime::new().expect("runtime");
        // Snapshot from a 2-leaf split with 2 tabs total (1 each).
        let mut src = mgr_with_titled(&runtime, 1);
        src.split_focused(SplitDirection::Horizontal, &runtime); // leaf0=[T0], leaf1=[POM]
        let snap = src.layout_snapshot().expect("snapshot");

        // Restore into a manager with MORE tabs than the layout describes.
        let mut restored = mgr_with_titled(&runtime, 5); // 5 store tabs
        let before = restored.len();
        restored.restore_layout(&snap);
        // Every store tab is still present across the leaves (none lost).
        let placed: usize = restored
            .leaf_ids()
            .iter()
            .map(|id| restored.leaf_tab_store_indices(*id).len())
            .sum();
        assert_eq!(placed, before, "no store tab lost on restore");
        assert_eq!(restored.len(), before, "store tab count unchanged");
    }

    /// Validates: layout-and-docking Req 14.12 -- restoring into a single-tab
    /// store is a no-op (cannot split one tab into two regions).
    #[test]
    fn restore_layout_noop_with_single_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mut src = mgr_with_titled(&runtime, 2);
        src.split_focused(SplitDirection::Horizontal, &runtime);
        let snap = src.layout_snapshot().expect("snapshot");

        let mut restored = mgr_with_titled(&runtime, 1); // only one tab
        restored.restore_layout(&snap);
        assert!(!restored.is_split(), "cannot restore a split into one tab");
    }
}
