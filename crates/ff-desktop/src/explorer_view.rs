//! Modern, NavModel-backed File Explorer view (CR-NR-060 Slice A).
//!
//! This module builds the File Explorer interaction + rendering on top of the
//! canonical `ff-file-tree` model (via [`crate::nav_model::NavModel`]), replacing
//! the legacy path-string / `std::fs` inline explorer. Identity is `NodeId`
//! throughout (ADR-002 D1); selection, cursor, and anchor are `NodeId`-keyed.
//!
//! The egui-free interaction core (selection model, visible-row flattening,
//! keyboard reduction, open classification) lives here and is unit-tested; the
//! egui rendering is a thin layer over it.
//!
//! Validates: Requirement 24.1, 24.2, 24.6, 24.9 (file-tree-panel Req 8/19/20)

use std::collections::HashSet;

use eframe::egui;
use ff_file_tree::keyboard::{
    first_visible_node, last_visible_node, next_visible_node, prev_visible_node, type_ahead_jump,
};
use ff_file_tree::{FileCategory, NodeId, TreeAction};
use ff_theme::{ColourRGBA, ThemePalette};
use ff_vfs::ResourceUri;

use crate::nav_model::NavModel;

/// Indentation (logical px) applied per tree depth level.
const INDENT_PER_LEVEL: f32 = 16.0;
/// Left padding before the first indent level.
const ROW_LEFT_PAD: f32 = 6.0;

/// Keyboard/mouse selection state for the File Explorer, keyed entirely on
/// `NodeId` (never on path strings). Mirrors the cursor-vs-selection model of
/// file-tree-panel Requirement 20.
///
/// Validates: Requirement 24.2 (Req 20.1-20.13)
#[derive(Debug, Default, Clone)]
pub struct ExplorerSelection {
    /// The node with the keyboard cursor (focus ring), independent of selection.
    pub cursor: Option<NodeId>,
    /// The set of selected nodes.
    pub selected: HashSet<NodeId>,
    /// The anchor node for range selection (Shift+Arrow).
    pub anchor: Option<NodeId>,
}

impl ExplorerSelection {
    /// Clear the selection set but keep the cursor as a single-node selection
    /// (Req 20.12: Escape clears selection, cursor remains).
    pub fn clear_to_cursor(&mut self) {
        self.selected.clear();
        if let Some(c) = self.cursor {
            self.selected.insert(c);
        }
        self.anchor = None;
    }

    /// Move the cursor to `id` without changing the selection (plain Arrow /
    /// Ctrl+Arrow -- Req 20.4, 20.9).
    pub fn move_cursor(&mut self, id: NodeId) {
        self.cursor = Some(id);
    }

    /// Move the cursor to `id` and add it to the selection, setting the anchor
    /// on first extension (Shift+Arrow -- Req 20.6, 20.7).
    pub fn extend_to(&mut self, id: NodeId) {
        if self.anchor.is_none() {
            self.anchor = self.cursor.or(Some(id));
        }
        self.cursor = Some(id);
        self.selected.insert(id);
    }

    /// Toggle membership of the cursor node in the selection (Ctrl+Space --
    /// Req 20.10).
    pub fn toggle_cursor(&mut self) {
        if let Some(c) = self.cursor {
            if !self.selected.remove(&c) {
                self.selected.insert(c);
            }
        }
    }

    /// Ctrl+click: move the cursor to `id` and toggle its membership in the
    /// selection, leaving the rest of the selection intact (Req 19.3).
    pub fn ctrl_click(&mut self, id: NodeId) {
        self.cursor = Some(id);
        if !self.selected.remove(&id) {
            self.selected.insert(id);
        }
        self.anchor = Some(id);
    }

    /// Set a single-node selection + cursor (plain click / initial focus).
    pub fn select_single(&mut self, id: NodeId) {
        self.cursor = Some(id);
        self.selected.clear();
        self.selected.insert(id);
        self.anchor = Some(id);
    }

    /// True if `id` is in the selection set.
    pub fn is_selected(&self, id: NodeId) -> bool {
        self.selected.contains(&id)
    }
}

/// A row in the flattened, display-ordered visible tree, carrying enough for the
/// renderer: node id, depth (for indent), label, whether expandable/expanded.
///
/// Validates: Requirement 24.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleRow {
    pub id: NodeId,
    pub depth: u32,
    pub label: String,
    pub expandable: bool,
    pub expanded: bool,
}

/// Flatten the model's visible nodes (respecting expansion, filter, hidden-file
/// rule) into display-ordered rows for the renderer.
///
/// Validates: Requirement 24.1, 24.6
pub fn visible_rows(model: &NavModel) -> Vec<VisibleRow> {
    model
        .tree
        .visible_nodes_iter()
        .into_iter()
        .map(|n| VisibleRow {
            id: n.id,
            depth: n.depth,
            label: n.label.clone(),
            expandable: n.node_type.is_expandable(),
            expanded: n.expanded,
        })
        .collect()
}

/// Build an indented ASCII text tree of the selected nodes, in display order,
/// for the OS clipboard (Req 19.5/19.6). Keyed on `NodeId`/`VisibleRow` (not on
/// path strings) so it is independent of the legacy inline tree.
///
/// Rules (mirroring the retired legacy `build_text_tree`):
/// - The shallowest selected node sits at indent level 0.
/// - Each additional depth level adds two spaces of indentation.
/// - Expandable (directory/container) nodes are prefixed with `[DIR] `.
/// - When a selected node's parent is also selected, a `|-- ` connector is used.
///
/// Returns an empty string when nothing is selected.
///
/// Validates: Requirement 24.2 (file-tree-panel Req 19.5, 19.6)
pub fn build_selection_text_tree(model: &NavModel, selected: &HashSet<NodeId>) -> String {
    if selected.is_empty() {
        return String::new();
    }
    // Selected rows in display order.
    let rows: Vec<VisibleRow> = visible_rows(model)
        .into_iter()
        .filter(|r| selected.contains(&r.id))
        .collect();
    if rows.is_empty() {
        return String::new();
    }
    let min_depth = rows.iter().map(|r| r.depth).min().unwrap_or(0);
    let mut lines = Vec::with_capacity(rows.len());
    for row in &rows {
        let rel_depth = row.depth.saturating_sub(min_depth) as usize;
        // Does this node's parent (in the model) also appear in the selection?
        let parent_selected = model
            .tree
            .get_node(row.id)
            .map(|n| n.parent)
            .map(|p| selected.contains(&p))
            .unwrap_or(false);
        let indent = if rel_depth == 0 {
            String::new()
        } else if parent_selected {
            format!("{}|-- ", "  ".repeat(rel_depth - 1))
        } else {
            "  ".repeat(rel_depth)
        };
        let prefix = if row.expandable { "[DIR] " } else { "" };
        lines.push(format!("{indent}{prefix}{}", row.label));
    }
    lines.join("\n")
}

/// The set of keyboard gestures the explorer understands, expressed
/// independently of egui so the reducer is unit-testable. Modifier state is
/// captured explicitly (Req 20.6/20.9/20.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerKey {
    Down { shift: bool, ctrl: bool },
    Up { shift: bool, ctrl: bool },
    Right,
    Left,
    Enter,
    Escape,
    CtrlSpace,
    Home,
    End,
}

/// Outcome of reducing a keyboard gesture over the model + selection: an
/// optional side effect the caller must perform (expand/collapse/open a node).
///
/// Validates: Requirement 24.2 (Req 20)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerEffect {
    None,
    Expand(NodeId),
    Collapse(NodeId),
    Open(NodeId),
    /// Copy the node's resource path to the OS clipboard (Req 16 Copy Path).
    CopyPath(NodeId),
    /// Copy the given pre-built text (an indented tree of the multi-selection)
    /// to the OS clipboard (Req 19.5/19.6 copy-as-text-tree).
    CopySelection(String),
    /// Reveal the node in the OS file manager (Req 16 Reveal in Explorer).
    Reveal(NodeId),
    /// Begin renaming the node (opens the rename dialog) (Req 16 Rename).
    Rename(NodeId),
    /// Begin deleting the node (opens the confirmation dialog) (Req 16 Delete).
    Delete(NodeId),
    /// Begin creating a new child (opens the name dialog). `anchor` is the
    /// right-clicked node; the shell resolves the containing directory (the
    /// anchor itself if it is a directory, else its parent). `is_dir` chooses
    /// folder vs file (Req 16 New File / New Folder).
    NewChild {
        anchor: NodeId,
        is_dir: bool,
    },
    /// Mark the current selection (or the given node) for a file copy -- the
    /// shell records the source URIs in its file clipboard (Req 21.1).
    MarkCopy(NodeId),
    /// Paste the file clipboard into the directory resolved from `anchor` (the
    /// anchor if it is a directory, else its parent) (Req 21.2/21.3).
    Paste(NodeId),
}

/// Resolve the paste target directory node from an anchor node: the anchor
/// itself when it is a directory/container, otherwise its parent (Req 21.2).
/// Returns `None` if the anchor has no usable parent.
///
/// Validates: Requirement 24.2 (file-tree-panel Req 21.2)
pub fn paste_target(model: &NavModel, anchor: NodeId) -> Option<NodeId> {
    let node = model.tree.get_node(anchor)?;
    if node.node_type.is_expandable() {
        Some(anchor)
    } else if node.parent != NodeId::ROOT {
        Some(node.parent)
    } else {
        None
    }
}

/// Reduce a keyboard gesture over the model and selection, returning the side
/// effect (if any) the caller must apply. This is the NodeId-native equivalent
/// of the legacy path-string keyboard handler, preserving Req 8 + Req 20.
///
/// Validates: Requirement 24.2 (Req 8.1-8.11, Req 20.4-20.12)
pub fn reduce_key(
    model: &NavModel,
    sel: &mut ExplorerSelection,
    key: ExplorerKey,
) -> ExplorerEffect {
    match key {
        ExplorerKey::Down { shift, ctrl } => {
            if let Some(next) = cursor_step(model, sel, next_visible_node, first_visible_node) {
                apply_move(sel, next, shift, ctrl);
            }
            ExplorerEffect::None
        }
        ExplorerKey::Up { shift, ctrl } => {
            if let Some(prev) = cursor_step(model, sel, prev_visible_node, last_visible_node) {
                apply_move(sel, prev, shift, ctrl);
            }
            ExplorerEffect::None
        }
        ExplorerKey::Right => {
            if let Some(c) = sel.cursor {
                if let Some(node) = model.tree.get_node(c) {
                    if node.node_type.is_expandable() && !node.expanded {
                        return ExplorerEffect::Expand(c);
                    }
                }
            }
            ExplorerEffect::None
        }
        ExplorerKey::Left => {
            if let Some(c) = sel.cursor {
                if let Some(node) = model.tree.get_node(c) {
                    if node.node_type.is_expandable() && node.expanded {
                        return ExplorerEffect::Collapse(c);
                    }
                    // Move cursor to parent (Req 8.6 / 20.5).
                    if node.parent != NodeId::ROOT {
                        sel.move_cursor(node.parent);
                    }
                }
            }
            ExplorerEffect::None
        }
        ExplorerKey::Enter => {
            if let Some(c) = sel.cursor {
                if let Some(node) = model.tree.get_node(c) {
                    if node.node_type.is_expandable() {
                        return if node.expanded {
                            ExplorerEffect::Collapse(c)
                        } else {
                            ExplorerEffect::Expand(c)
                        };
                    }
                    return ExplorerEffect::Open(c);
                }
            }
            ExplorerEffect::None
        }
        ExplorerKey::CtrlSpace => {
            sel.toggle_cursor();
            ExplorerEffect::None
        }
        ExplorerKey::Escape => {
            sel.clear_to_cursor();
            ExplorerEffect::None
        }
        ExplorerKey::Home => {
            if let Some(first) = first_visible_node(&model.tree) {
                sel.select_single(first);
            }
            ExplorerEffect::None
        }
        ExplorerKey::End => {
            if let Some(last) = last_visible_node(&model.tree) {
                sel.select_single(last);
            }
            ExplorerEffect::None
        }
    }
}

/// Compute the next cursor target for an Up/Down step, falling back to the
/// first/last visible node when there is no current cursor.
fn cursor_step(
    model: &NavModel,
    sel: &ExplorerSelection,
    step: fn(&ff_file_tree::TreeState, NodeId) -> Option<NodeId>,
    fallback: fn(&ff_file_tree::TreeState) -> Option<NodeId>,
) -> Option<NodeId> {
    match sel.cursor {
        Some(c) => step(&model.tree, c).or(Some(c)),
        None => fallback(&model.tree),
    }
}

fn apply_move(sel: &mut ExplorerSelection, target: NodeId, shift: bool, ctrl: bool) {
    if shift {
        sel.extend_to(target);
    } else if ctrl {
        sel.move_cursor(target);
    } else {
        sel.select_single(target);
    }
}

/// The first visible node in display order, if any (used by Tab focus-transfer
/// to land the cursor on entry). Req 20.1.
pub fn first_row_id(model: &NavModel) -> Option<NodeId> {
    first_visible_node(&model.tree)
}

/// The next visible node after `cursor` in display order, if any (used by Tab to
/// advance the cursor within the tree; `None` means past the last row). Req 20.1.
pub fn next_row_id(model: &NavModel, cursor: NodeId) -> Option<NodeId> {
    next_visible_node(&model.tree, cursor)
}

/// Type-ahead jump helper exposed for the renderer (Req 8.12).
// Type-ahead search is not yet wired to egui key input; retained + tested for a
// later update.
#[allow(dead_code)]
pub fn type_ahead(model: &NavModel, current: NodeId, prefix: &str) -> Option<NodeId> {
    type_ahead_jump(&model.tree, current, prefix)
}

/// Translate a `ff-file-tree` `TreeAction` (from its `KeyboardHandler`) into an
/// `ExplorerEffect` against the selection, for callers that prefer to reuse the
/// model's own handler rather than `reduce_key`.
///
/// Validates: Requirement 24.11 (reuse the canonical model)
// Alternative reducer that adapts the model's own KeyboardHandler TreeActions;
// the shell currently uses `reduce_key` directly. Retained as the documented
// bridge for callers that prefer the model's handler.
#[allow(dead_code)]
pub fn effect_from_action(
    model: &NavModel,
    sel: &mut ExplorerSelection,
    action: TreeAction,
) -> ExplorerEffect {
    match action {
        TreeAction::SelectNext => {
            if let Some(c) = sel.cursor {
                if let Some(n) = next_visible_node(&model.tree, c) {
                    sel.select_single(n);
                }
            }
            ExplorerEffect::None
        }
        TreeAction::SelectPrevious => {
            if let Some(c) = sel.cursor {
                if let Some(n) = prev_visible_node(&model.tree, c) {
                    sel.select_single(n);
                }
            }
            ExplorerEffect::None
        }
        TreeAction::Expand(id) | TreeAction::ToggleExpand(id) => ExplorerEffect::Expand(id),
        TreeAction::Collapse(id) => ExplorerEffect::Collapse(id),
        TreeAction::Open(id) => ExplorerEffect::Open(id),
        TreeAction::SelectFirstChild(id) | TreeAction::SelectParent(id) => {
            sel.select_single(id);
            ExplorerEffect::None
        }
        TreeAction::SelectFirst => {
            if let Some(f) = first_visible_node(&model.tree) {
                sel.select_single(f);
            }
            ExplorerEffect::None
        }
        TreeAction::SelectLast => {
            if let Some(l) = last_visible_node(&model.tree) {
                sel.select_single(l);
            }
            ExplorerEffect::None
        }
        TreeAction::TypeAheadJump(prefix) => {
            if let Some(c) = sel.cursor {
                if let Some(t) = type_ahead_jump(&model.tree, c, &prefix) {
                    sel.select_single(t);
                }
            }
            ExplorerEffect::None
        }
        TreeAction::Delete(_) | TreeAction::Rename(_) => ExplorerEffect::None,
    }
}

// === Open resolution ====================================================

/// What the shell should do when a leaf node is opened.
///
/// Resolution is done here (NodeId -> ResourceUri -> classification); the shell
/// performs the actual open (editor tab via its existing open path, or the OS
/// default application) so that File Explorer I/O stays in one place.
///
/// Validates: Requirement 24.2, 24.9
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenTarget {
    /// Open this resource in a FFWB editor tab (Text / FfwbStructured).
    Editor(ResourceUri),
    /// Hand this resource to the OS default application (External).
    External(ResourceUri),
    /// Open a Mainframe dataset: resolve `dsn` in `catalog` to a physical path
    /// (creating the backing file if missing) then open it in the editor.
    Dataset { catalog: String, dsn: String },
    /// The node has no associated URI (e.g. a category header) -- nothing to open.
    None,
}

/// Split a `vfs://dataset/{catalog}/{DSN}` URI path into `(catalog, dsn)`.
/// The catalog is the first path segment; the DSN is the remainder (a DSN
/// contains no `/`). Returns `None` if either part is empty.
///
/// Validates: Requirement 24.9
pub fn split_dataset_uri_path(uri_path: &str) -> Option<(&str, &str)> {
    let trimmed = uri_path.trim_start_matches('/');
    let (catalog, dsn) = trimmed.split_once('/')?;
    if catalog.is_empty() || dsn.is_empty() {
        return None;
    }
    Some((catalog, dsn))
}

/// Resolve an opened node to an [`OpenTarget`]. Dataset-scheme nodes
/// (`vfs://dataset/{catalog}/{DSN}`) resolve to [`OpenTarget::Dataset`] so the
/// shell can map the DSN to its physical path via the catalog. Other schemes
/// classify the resource by its URI path (reusing `context_menu::classify_file`);
/// for local/posix the path is the on-disk path, other schemes fall through to
/// the classifier (external opens fall back gracefully).
///
/// Validates: Requirement 24.2, 24.9
pub fn resolve_open(model: &NavModel, id: NodeId) -> OpenTarget {
    let Some(uri) = model.uri_of(id) else {
        return OpenTarget::None;
    };
    if uri.scheme() == "dataset" {
        return match split_dataset_uri_path(uri.path()) {
            Some((catalog, dsn)) => OpenTarget::Dataset {
                catalog: catalog.to_string(),
                dsn: dsn.to_string(),
            },
            None => OpenTarget::None,
        };
    }
    match crate::context_menu::classify_file(uri.path()) {
        crate::context_menu::FileClass::Text | crate::context_menu::FileClass::FfwbStructured => {
            OpenTarget::Editor(uri.clone())
        }
        crate::context_menu::FileClass::External => OpenTarget::External(uri.clone()),
    }
}

// === Rendering (egui) ===================================================

/// Convert a theme `ColourRGBA` to an `egui::Color32` (local copy of the shell
/// helper, which is private to the shell module).
fn to_egui_color(c: ColourRGBA) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
}

/// Map a node's `FileCategory` to its theme palette colour (Requirement 24.7 --
/// presentation driven by the `file_tree.*` palette group).
fn category_colour(cat: FileCategory, palette: &ThemePalette) -> ColourRGBA {
    match cat {
        FileCategory::NonEditableBinary => palette.file_tree.binary,
        FileCategory::FileForgeStructured => palette.file_tree.structured,
        FileCategory::StandardText => palette.file_tree.text,
        FileCategory::Unknown => palette.file_tree.unknown,
        FileCategory::Directory => palette.file_tree.directory,
        FileCategory::SymbolicLink => palette.file_tree.symlink,
        // FileCategory is #[non_exhaustive]; fall back to the "unknown" colour.
        _ => palette.file_tree.unknown,
    }
}

/// A glyph prefix for a row, chosen by expandability/expansion state -- modern,
/// theme-neutral disclosure + type iconography (Requirement 24.7).
fn row_glyph(row: &VisibleRow) -> &'static str {
    if row.expandable {
        if row.expanded {
            "\u{25BC}" // down-pointing triangle (expanded)
        } else {
            "\u{25B6}" // right-pointing triangle (collapsed)
        }
    } else {
        "\u{2022}" // bullet (leaf)
    }
}

/// Render the NavModel-backed tree as modern, indented, theme-coloured rows and
/// return the interaction effects the caller must apply (expand/collapse loads,
/// opens). Selection/cursor are updated in place on `sel`.
///
/// The renderer performs NO VFS I/O: expand/collapse and open are returned as
/// `ExplorerEffect`s so the shell (which owns the runtime + providers) performs
/// the async `list_via_provider` -> `apply_listing` load (Requirement 24.3/24.5).
///
/// Validates: Requirement 24.1, 24.2, 24.6, 24.7
pub fn render_tree(
    ui: &mut egui::Ui,
    model: &NavModel,
    sel: &mut ExplorerSelection,
    palette: &ThemePalette,
) -> Vec<ExplorerEffect> {
    let mut effects = Vec::new();
    let rows = visible_rows(model);
    let focus_stroke = egui::Stroke::new(1.5_f32, to_egui_color(palette.ui.focus_ring));

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in &rows {
                let category = model
                    .tree
                    .get_node(row.id)
                    .map(|n| n.category)
                    .unwrap_or(FileCategory::Unknown);
                let fg = to_egui_color(category_colour(category, palette));
                let indent = ROW_LEFT_PAD + (row.depth as f32) * INDENT_PER_LEVEL;
                let is_selected = sel.is_selected(row.id);
                let is_cursor = sel.cursor == Some(row.id);

                let label = format!("{} {}", row_glyph(row), row.label);
                let text = egui::RichText::new(label).monospace().color(fg);

                // Full-width selectable row with leading indent.
                let resp = ui
                    .horizontal(|ui| {
                        ui.add_space(indent);
                        ui.add(egui::SelectableLabel::new(is_selected, text))
                    })
                    .inner;

                // Cursor focus ring, distinct from the selection fill (Req 20.13).
                if is_cursor {
                    ui.painter().rect_stroke(
                        resp.rect.expand(1.0_f32),
                        2.0_f32,
                        focus_stroke,
                        egui::StrokeKind::Outside,
                    );
                }

                if resp.clicked() {
                    // Modifier-aware selection (Req 19.2/19.3): Shift extends a
                    // range from the anchor, Ctrl toggles the clicked node, plain
                    // click selects just this node.
                    let (ctrl, shift) = ui.input(|i| (i.modifiers.ctrl, i.modifiers.shift));
                    if shift {
                        sel.extend_to(row.id);
                    } else if ctrl {
                        sel.ctrl_click(row.id);
                    } else {
                        sel.select_single(row.id);
                    }
                }
                if resp.double_clicked() {
                    sel.select_single(row.id);
                    if row.expandable {
                        effects.push(if row.expanded {
                            ExplorerEffect::Collapse(row.id)
                        } else {
                            ExplorerEffect::Expand(row.id)
                        });
                    } else {
                        effects.push(ExplorerEffect::Open(row.id));
                    }
                }
                // Right-click context menu (Req 16). Selecting a row on
                // right-click mirrors the legacy panel so the acted-on node is
                // unambiguous.
                resp.context_menu(|ui| {
                    if let Some(eff) = context_menu_ui(ui, row) {
                        sel.select_single(row.id);
                        effects.push(eff);
                    }
                });
            }
        });

    effects
}

/// Render the right-click context menu for a tree row and return the chosen
/// effect (if any). Slice A wires the actions the NavModel can perform without
/// the legacy path-string state: Open (files), Copy Full Path, and Reveal in
/// the OS file manager. The mutating operations (Rename, Delete, New) are shown
/// disabled with a tooltip; they are ported with the full swap.
///
/// Validates: Requirement 24.2 (file-tree-panel Req 16.1-16.9)
fn context_menu_ui(ui: &mut egui::Ui, row: &VisibleRow) -> Option<ExplorerEffect> {
    let mut chosen = None;
    // Open: only meaningful for leaf (file/dataset) nodes; containers use
    // expand/collapse instead.
    if !row.expandable && ui.button("Open").clicked() {
        chosen = Some(ExplorerEffect::Open(row.id));
        ui.close_menu();
    }
    if ui.button("Copy").clicked() {
        // Mark the selection (or this node) for a file copy (Req 21.1).
        chosen = Some(ExplorerEffect::MarkCopy(row.id));
        ui.close_menu();
    }
    if ui.button("Paste").clicked() {
        // Paste the file clipboard into this node's directory (Req 21.2/21.3).
        chosen = Some(ExplorerEffect::Paste(row.id));
        ui.close_menu();
    }
    if ui.button("Copy Full Path").clicked() {
        chosen = Some(ExplorerEffect::CopyPath(row.id));
        ui.close_menu();
    }
    if ui.button("Reveal in File Manager").clicked() {
        chosen = Some(ExplorerEffect::Reveal(row.id));
        ui.close_menu();
    }
    ui.separator();
    if ui.button("Rename").clicked() {
        chosen = Some(ExplorerEffect::Rename(row.id));
        ui.close_menu();
    }
    if ui.button("Delete").clicked() {
        chosen = Some(ExplorerEffect::Delete(row.id));
        ui.close_menu();
    }
    if ui.button("New File").clicked() {
        chosen = Some(ExplorerEffect::NewChild {
            anchor: row.id,
            is_dir: false,
        });
        ui.close_menu();
    }
    if ui.button("New Folder").clicked() {
        chosen = Some(ExplorerEffect::NewChild {
            anchor: row.id,
            is_dir: true,
        });
        ui.close_menu();
    }
    chosen
}

/// Translate this frame's egui keyboard input into explorer gestures, run them
/// through the tested `reduce_key` reducer against the model + selection, and
/// return the resulting effects for the caller to apply. Call this once per
/// frame while the explorer has focus. Mirrors the legacy handler's key set
/// (Req 8 arrows/Enter/Home/End + Req 20 Shift/Ctrl selection semantics).
///
/// Validates: Requirement 24.2 (Req 8, Req 20.4-20.12)
pub fn keyboard_effects(
    ui: &egui::Ui,
    model: &NavModel,
    sel: &mut ExplorerSelection,
) -> Vec<ExplorerEffect> {
    let (ctrl, shift) = ui.input(|i| (i.modifiers.ctrl, i.modifiers.shift));
    let mut gestures: Vec<ExplorerKey> = Vec::new();
    ui.input(|i| {
        if i.key_pressed(egui::Key::ArrowDown) {
            gestures.push(ExplorerKey::Down { shift, ctrl });
        }
        if i.key_pressed(egui::Key::ArrowUp) {
            gestures.push(ExplorerKey::Up { shift, ctrl });
        }
        if i.key_pressed(egui::Key::ArrowRight) {
            gestures.push(ExplorerKey::Right);
        }
        if i.key_pressed(egui::Key::ArrowLeft) {
            gestures.push(ExplorerKey::Left);
        }
        if i.key_pressed(egui::Key::Enter) {
            gestures.push(ExplorerKey::Enter);
        }
        if i.key_pressed(egui::Key::Escape) {
            gestures.push(ExplorerKey::Escape);
        }
        if ctrl && i.key_pressed(egui::Key::Space) {
            gestures.push(ExplorerKey::CtrlSpace);
        }
        if i.key_pressed(egui::Key::Home) {
            gestures.push(ExplorerKey::Home);
        }
        if i.key_pressed(egui::Key::End) {
            gestures.push(ExplorerKey::End);
        }
    });

    // Ctrl+C over a non-empty selection copies an indented text tree of the
    // selected nodes to the OS clipboard (Req 19.5/19.6). Handled outside the
    // gesture reducer since it produces text, not a selection/navigation change.
    let copy_selection = ctrl && ui.input(|i| i.key_pressed(egui::Key::C));

    let mut effects = Vec::new();
    for g in gestures {
        let eff = reduce_key(model, sel, g);
        if eff != ExplorerEffect::None {
            effects.push(eff);
        }
    }
    if copy_selection && !sel.selected.is_empty() {
        let text = build_selection_text_tree(model, &sel.selected);
        if !text.is_empty() {
            effects.push(ExplorerEffect::CopySelection(text));
        }
    }
    effects
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a model: local root with dir "src" (containing "a.rs") and file "b.rs".
    fn model_with_tree() -> (NavModel, NodeId, NodeId, NodeId) {
        let mut m = NavModel::new();
        let local = m.tree.root_categories[0];
        m.set_uri(local, ResourceUri::new("local", "/root"));
        m.apply_listing(
            local,
            "local",
            &[
                ff_vfs::VfsEntry {
                    name: "src".into(),
                    entry_type: ff_vfs::VfsEntryType::Directory,
                    size: None,
                    modified: None,
                },
                ff_vfs::VfsEntry {
                    name: "b.rs".into(),
                    entry_type: ff_vfs::VfsEntryType::File,
                    size: None,
                    modified: None,
                },
            ],
        );
        let children = m.tree.get_node(local).unwrap().children.clone();
        let src = children
            .iter()
            .copied()
            .find(|&id| m.tree.get_node(id).unwrap().label == "src")
            .unwrap();
        let b = children
            .iter()
            .copied()
            .find(|&id| m.tree.get_node(id).unwrap().label == "b.rs")
            .unwrap();
        (m, local, src, b)
    }

    #[test]
    fn visible_rows_are_node_id_keyed_and_ordered() {
        // Validates: Requirement 24.1 -- rows come from the model, keyed on NodeId
        let (m, local, _src, _b) = model_with_tree();
        let rows = visible_rows(&m);
        // local is expanded (apply_children expands it), so its children are visible.
        assert!(rows
            .iter()
            .any(|r| r.id == local && r.label == "Local Files"));
        assert!(rows.iter().any(|r| r.label == "src" && r.expandable));
        assert!(rows.iter().any(|r| r.label == "b.rs" && !r.expandable));
    }

    #[test]
    fn select_single_sets_cursor_and_one_selection() {
        // Validates: Requirement 24.2 (Req 20 cursor vs selection)
        let (_m, local, _src, _b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.select_single(local);
        assert_eq!(sel.cursor, Some(local));
        assert!(sel.is_selected(local));
        assert_eq!(sel.selected.len(), 1);
    }

    #[test]
    fn shift_extend_accumulates_selection() {
        // Validates: Requirement 24.2 (Req 20.6, 20.7)
        let (m, _local, src, b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.select_single(src);
        // Shift+Down onto b accumulates.
        let _ = reduce_key(
            &m,
            &mut sel,
            ExplorerKey::Down {
                shift: true,
                ctrl: false,
            },
        );
        assert!(sel.is_selected(src));
        assert!(sel.is_selected(b));
        assert_eq!(sel.cursor, Some(b));
    }

    #[test]
    fn ctrl_arrow_moves_cursor_without_changing_selection() {
        // Validates: Requirement 24.2 (Req 20.9)
        let (m, _local, src, _b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.select_single(src);
        let before = sel.selected.clone();
        let _ = reduce_key(
            &m,
            &mut sel,
            ExplorerKey::Down {
                shift: false,
                ctrl: true,
            },
        );
        assert_eq!(sel.selected, before, "ctrl+arrow must not change selection");
        assert_ne!(sel.cursor, Some(src), "cursor should have moved");
    }

    #[test]
    fn ctrl_space_toggles_cursor_membership() {
        // Validates: Requirement 24.2 (Req 20.10)
        let (m, _local, src, _b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.move_cursor(src);
        let _ = reduce_key(&m, &mut sel, ExplorerKey::CtrlSpace);
        assert!(sel.is_selected(src));
        let _ = reduce_key(&m, &mut sel, ExplorerKey::CtrlSpace);
        assert!(!sel.is_selected(src));
    }

    #[test]
    fn escape_clears_to_cursor() {
        // Validates: Requirement 24.2 (Req 20.12)
        let (m, _local, src, b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.select_single(src);
        let _ = reduce_key(
            &m,
            &mut sel,
            ExplorerKey::Down {
                shift: true,
                ctrl: false,
            },
        );
        assert!(sel.selected.len() >= 2);
        let _ = reduce_key(&m, &mut sel, ExplorerKey::Escape);
        assert_eq!(sel.selected.len(), 1);
        assert!(sel.is_selected(b)); // cursor was on b after the shift-down
    }

    #[test]
    fn right_on_collapsed_dir_requests_expand() {
        // Validates: Requirement 24.2 (Req 8.3 / 20.5)
        let (mut m, _local, src, _b) = model_with_tree();
        // Ensure src is collapsed.
        if let Some(n) = m.tree.get_node_mut(src) {
            n.expanded = false;
        }
        let mut sel = ExplorerSelection::default();
        sel.move_cursor(src);
        let eff = reduce_key(&m, &mut sel, ExplorerKey::Right);
        assert_eq!(eff, ExplorerEffect::Expand(src));
    }

    #[test]
    fn enter_on_file_requests_open() {
        // Validates: Requirement 24.2 (Req 8.7)
        let (m, _local, _src, b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.move_cursor(b);
        let eff = reduce_key(&m, &mut sel, ExplorerKey::Enter);
        assert_eq!(eff, ExplorerEffect::Open(b));
    }

    #[test]
    fn enter_on_dir_toggles_expand() {
        // Validates: Requirement 24.2 (Req 8.8)
        // A freshly-listed child directory is collapsed (only the listed parent
        // is expanded by apply_children), so Enter on it requests Expand; Enter
        // again (once expanded) requests Collapse.
        let (mut m, _local, src, _b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.move_cursor(src);
        let eff = reduce_key(&m, &mut sel, ExplorerKey::Enter);
        assert_eq!(eff, ExplorerEffect::Expand(src));
        // Now expand it and confirm Enter collapses.
        m.tree.toggle_expand(src);
        let eff2 = reduce_key(&m, &mut sel, ExplorerKey::Enter);
        assert_eq!(eff2, ExplorerEffect::Collapse(src));
    }

    #[test]
    fn resolve_open_text_file_targets_editor() {
        // Validates: Requirement 24.2, 24.9 -- text/source resolves to Editor
        let (m, _local, _src, b) = model_with_tree();
        // b.rs already has a URI from apply_listing (child of /root).
        match resolve_open(&m, b) {
            OpenTarget::Editor(uri) => assert_eq!(uri.path(), "/root/b.rs"),
            other => panic!("expected Editor, got {other:?}"),
        }
        // A URI-less category node (Catalogs root, no set_uri) resolves to None.
        let catalogs = m.tree.root_categories[1];
        assert_eq!(resolve_open(&m, catalogs), OpenTarget::None);
    }

    #[test]
    fn resolve_open_binary_file_targets_external() {
        // Validates: Requirement 24.9 -- non-editable resolves to External
        let mut m = NavModel::new();
        let local = m.tree.root_categories[0];
        m.set_uri(local, ResourceUri::new("local", "/root"));
        m.apply_listing(
            local,
            "local",
            &[ff_vfs::VfsEntry {
                name: "image.png".into(),
                entry_type: ff_vfs::VfsEntryType::File,
                size: None,
                modified: None,
            }],
        );
        let png = m.tree.get_node(local).unwrap().children[0];
        match resolve_open(&m, png) {
            OpenTarget::External(uri) => assert_eq!(uri.path(), "/root/image.png"),
            other => panic!("expected External, got {other:?}"),
        }
    }

    #[test]
    fn split_dataset_uri_path_extracts_catalog_and_dsn() {
        // Validates: Requirement 24.9 -- dataset URI -> (catalog, dsn)
        assert_eq!(
            split_dataset_uri_path("/TESTING/TESTING.DATA"),
            Some(("TESTING", "TESTING.DATA"))
        );
        // Missing DSN or catalog is rejected.
        assert_eq!(split_dataset_uri_path("/TESTING"), None);
        assert_eq!(split_dataset_uri_path("/"), None);
    }

    #[test]
    fn resolve_open_dataset_node_targets_dataset() {
        // Validates: Requirement 24.9 -- a dataset-scheme node resolves to the
        // Dataset open target carrying the catalog + DSN for shell resolution.
        let mut m = NavModel::new();
        let catalogs = m.tree.root_categories[1];
        let cat = m.add_child(
            catalogs,
            "TESTING",
            ff_file_tree::NodeType::CatalogRoot,
            ResourceUri::new("catalog", "/TESTING"),
        );
        let ds = m.add_child(
            cat,
            "TESTING.DATA",
            ff_file_tree::NodeType::DatasetSequential,
            ResourceUri::new("dataset", "/TESTING/TESTING.DATA"),
        );
        match resolve_open(&m, ds) {
            OpenTarget::Dataset { catalog, dsn } => {
                assert_eq!(catalog, "TESTING");
                assert_eq!(dsn, "TESTING.DATA");
            }
            other => panic!("expected Dataset, got {other:?}"),
        }
    }

    #[test]
    fn paste_target_is_dir_itself_or_parent() {
        // Validates: Requirement 24.2 (Req 21.2) -- dir -> itself, file -> parent.
        let (m, local, src, b) = model_with_tree();
        // src is a directory -> target is src itself.
        assert_eq!(paste_target(&m, src), Some(src));
        // b.rs is a file whose parent is local -> target is local.
        assert_eq!(paste_target(&m, b), Some(local));
    }

    #[test]
    fn build_selection_text_tree_flat_uses_labels() {
        // Validates: Requirement 24.2 (Req 19.6) -- flat selection, one label/line.
        let (m, _local, src, b) = model_with_tree();
        // src is a dir (expandable), b.rs is a file.
        let mut sel = HashSet::new();
        sel.insert(src);
        sel.insert(b);
        let text = build_selection_text_tree(&m, &sel);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        // Both are direct children of the local root -> same depth -> no indent.
        assert!(lines.iter().any(|l| l == &"[DIR] src"));
        assert!(lines.iter().any(|l| l == &"b.rs"));
    }

    #[test]
    fn build_selection_text_tree_nested_uses_connector() {
        // Validates: Requirement 24.2 (Req 19.6) -- parent+child selected ->
        // child is indented with the |-- connector.
        let (mut m, local, src, _b) = model_with_tree();
        // Expand src and give it a child so we have a real depth-2 node.
        m.set_uri(src, ResourceUri::new("local", "/root/src"));
        m.apply_listing(
            src,
            "local",
            &[ff_vfs::VfsEntry {
                name: "a.rs".into(),
                entry_type: ff_vfs::VfsEntryType::File,
                size: None,
                modified: None,
            }],
        );
        let child = m.tree.get_node(src).unwrap().children[0];
        let mut sel = HashSet::new();
        sel.insert(src);
        sel.insert(child);
        let text = build_selection_text_tree(&m, &sel);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "[DIR] src");
        assert_eq!(lines[1], "|-- a.rs");
        let _ = local;
    }

    #[test]
    fn build_selection_text_tree_empty_selection_is_empty() {
        // Validates: Requirement 24.2 (Req 19.6) -- nothing selected -> empty.
        let (m, _local, _src, _b) = model_with_tree();
        assert_eq!(build_selection_text_tree(&m, &HashSet::new()), "");
    }

    #[test]
    fn ctrl_click_toggles_node_leaving_others_selected() {
        // Validates: Requirement 24.2 (Req 19.3) -- Ctrl+click toggles one node.
        let (_m, local, src, b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.select_single(src);
        // Ctrl+click b: adds b, keeps src.
        sel.ctrl_click(b);
        assert!(sel.is_selected(src));
        assert!(sel.is_selected(b));
        assert_eq!(sel.cursor, Some(b));
        // Ctrl+click b again: removes b, keeps src.
        sel.ctrl_click(b);
        assert!(sel.is_selected(src));
        assert!(!sel.is_selected(b));
        let _ = local;
    }

    #[test]
    fn shift_click_extends_range_from_anchor() {
        // Validates: Requirement 24.2 (Req 19.2) -- Shift+click extends selection.
        let (_m, _local, src, b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        sel.select_single(src); // anchor = src
        sel.extend_to(b);
        assert!(sel.is_selected(src));
        assert!(sel.is_selected(b));
        assert_eq!(sel.anchor, Some(src));
        assert_eq!(sel.cursor, Some(b));
    }

    #[test]
    fn first_row_id_is_the_first_visible_node() {
        // Validates: Requirement 24.9 (Req 20.1) -- Tab lands on the first row.
        let (m, local, _src, _b) = model_with_tree();
        assert_eq!(first_row_id(&m), Some(local));
    }

    #[test]
    fn next_row_id_advances_then_returns_none_past_last() {
        // Validates: Requirement 24.9 (Req 20.1) -- Tab advances, then exits.
        let (m, _local, _src, _b) = model_with_tree();
        let first = first_row_id(&m).expect("first");
        // Walking next_row_id from the first row must eventually yield None
        // (past the last visible row), and never loop forever.
        let mut cur = Some(first);
        let mut steps = 0;
        while let Some(c) = cur {
            cur = next_row_id(&m, c);
            steps += 1;
            assert!(steps < 100, "next_row_id must terminate");
        }
        assert!(steps >= 1, "at least one advance from the first row");
    }

    #[test]
    fn home_end_jump_to_first_last() {
        // Validates: Requirement 24.2 (Req 8.11)
        let (m, _local, _src, _b) = model_with_tree();
        let mut sel = ExplorerSelection::default();
        let _ = reduce_key(&m, &mut sel, ExplorerKey::End);
        let last = last_visible_node(&m.tree);
        assert_eq!(sel.cursor, last);
        let _ = reduce_key(&m, &mut sel, ExplorerKey::Home);
        let first = first_visible_node(&m.tree);
        assert_eq!(sel.cursor, first);
    }
}
