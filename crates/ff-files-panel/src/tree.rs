//! # Files Panel tree render
//!
//! The top-level `render` entry point plus the left-side catalog tree
//! (three collapsible section headers, keyboard focus, context menus).
//!
//! Validates: Requirement 1.1-1.7, 4.1, 4.3

use ff_catalog_registry::{CatalogType, VirtualCatalog};

use crate::content::render_content_area;
use crate::resolve::FilesPanelState;
use crate::state::{ContentAreaState, FilesPanelAction};

// === Render =================================================================

/// Render the Files Panel into `ui`.
///
/// Returns a `FilesPanelAction` indicating any deferred action the shell must
/// perform after this frame (e.g. return to POM, open a dialog).
///
/// Validates: Requirement 1.1-1.7
pub fn render(ui: &mut egui::Ui, state: &mut FilesPanelState) -> FilesPanelAction {
    let mut action = FilesPanelAction::None;

    ui.vertical(|ui| {
        // === Title bar ======================================================
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("FileForge Workbench \u{2014} Virtual File Catalogs")
                    .monospace()
                    .strong(),
            );
        });
        ui.separator();

        // === Toolbar -- Req 1.3 =============================================
        ui.horizontal(|ui| {
            if ui.button("New Catalog").clicked() {
                action = FilesPanelAction::NewCatalog;
            }
            ui.separator();
            ui.button("Open").clicked();
            ui.button("Refresh").clicked();
            ui.button("Properties").clicked();
            ui.separator();
            ui.label("Filter:");
            ui.add(
                egui::TextEdit::singleline(&mut state.filter)
                    .desired_width(160.0)
                    .hint_text("name\u{2026}")
                    .font(egui::TextStyle::Monospace),
            );
        });
        ui.separator();

        // === Command field -- Req 1.6, 1.7 ==================================
        ui.horizontal(|ui| {
            ui.label("Command ===>");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.command)
                    .id(egui::Id::new("files_panel_cmd"))
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );
            if resp.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !state.command.is_empty()
            {
                let cmd = state.command.trim().to_uppercase();
                state.command.clear();
                if cmd == "END" || cmd == "F3" {
                    action = FilesPanelAction::ReturnToPom;
                }
            }
        });
        ui.separator();

        // === Split layout: left tree | right content ========================
        // Req 1.2 -- left catalog tree + right content area
        let tree_action = std::cell::Cell::new(FilesPanelAction::None);
        ui.columns(2, |cols| {
            egui::ScrollArea::vertical()
                .id_salt("files_tree")
                .show(&mut cols[0], |ui| {
                    if let Some(a) = render_catalog_tree(ui, state) {
                        tree_action.set(a);
                    }
                });

            egui::ScrollArea::vertical()
                .id_salt("files_content")
                .show(&mut cols[1], |ui| {
                    if let Some(a) = render_content_area(ui, state) {
                        tree_action.set(a);
                    }
                });
        });
        let tree_action = tree_action.into_inner();
        if tree_action != FilesPanelAction::None {
            action = tree_action;
        }
    });

    // F3 key also triggers return-to-POM -- Req 1.7
    if ui.input(|i| i.key_pressed(egui::Key::F3)) {
        action = FilesPanelAction::ReturnToPom;
    }

    action
}

/// Render the left-side catalog tree with three collapsible section headers.
/// Returns `Some(action)` if a context menu item was clicked, else `None`.
///
/// Validates: Requirement 1.4, 1.5, 4.1, 4.3
fn render_catalog_tree(ui: &mut egui::Ui, state: &mut FilesPanelState) -> Option<FilesPanelAction> {
    let filter = state.filter.to_lowercase();
    let mut action: Option<FilesPanelAction> = None;

    // Collect all visible catalog names across all sections in order.
    let all_visible: Vec<String> = [
        state.registry.list_by_type(CatalogType::Mainframe),
        state.registry.list_by_type(CatalogType::Posix),
        state.registry.list_by_type(CatalogType::Native),
    ]
    .into_iter()
    .flatten()
    .filter(|c| filter.is_empty() || c.name.to_lowercase().contains(&filter))
    .map(|c| c.name.clone())
    .collect();

    // Tab-into-tree: move focus to the first catalog.
    if state.tree_focus_requested {
        state.tree_focus_requested = false;
        state.focused_catalog = all_visible.first().cloned();
    }

    // Arrow-key navigation within the tree when a catalog has focus.
    if state.focused_catalog.is_some() {
        let down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
        let up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
        if down || up {
            if let Some(ref cur) = state.focused_catalog.clone() {
                let pos = all_visible.iter().position(|n| n == cur);
                state.focused_catalog = match (down, pos) {
                    (true, Some(i)) => all_visible.get(i + 1).cloned().or(Some(cur.clone())),
                    (false, Some(0)) | (false, None) => Some(cur.clone()),
                    (false, Some(i)) => all_visible.get(i - 1).cloned(),
                    _ => Some(cur.clone()),
                };
            }
        }
        // Enter selects the focused catalog.
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(ref name) = state.focused_catalog.clone() {
                state.content.selected_catalog = Some(name.clone());
                state.content.path_segments.clear();
                state.content.content_filter.clear();
            }
        }
        // Tab out of tree clears focus.
        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            state.focused_catalog = None;
        }
    }

    // Snapshot open-state before borrowing state.content via sec_ctx.
    let mf_open = state.sections.mainframe_open;
    let px_open = state.sections.posix_open;
    let nat_open = state.sections.native_open;
    let mf_entries = state.registry.list_by_type(CatalogType::Mainframe);
    let px_entries = state.registry.list_by_type(CatalogType::Posix);
    let nat_entries = state.registry.list_by_type(CatalogType::Native);
    let native_header = format!(
        "Native Catalogs ({})",
        FilesPanelState::native_platform_label()
    );
    let focused = state.focused_catalog.clone();

    {
        let mut sec_ctx = SectionCtx {
            content: &mut state.content,
            action: &mut action,
            focused: &focused,
        };
        render_section(
            ui,
            egui::RichText::new("Mainframe Catalogs")
                .monospace()
                .strong(),
            mf_open,
            mf_entries,
            &filter,
            &mut sec_ctx,
        );
        render_section(
            ui,
            egui::RichText::new("POSIX Catalogs").monospace().strong(),
            px_open,
            px_entries,
            &filter,
            &mut sec_ctx,
        );
        render_section(
            ui,
            egui::RichText::new(native_header).monospace().strong(),
            nat_open,
            nat_entries,
            &filter,
            &mut sec_ctx,
        );
    } // sec_ctx dropped here

    state.sections.mainframe_open = true;
    state.sections.posix_open = true;
    state.sections.native_open = true;

    action
}

/// Mutable context threaded through `render_section` to avoid exceeding the
/// 7-argument Clippy limit.
struct SectionCtx<'a> {
    content: &'a mut ContentAreaState,
    action: &'a mut Option<FilesPanelAction>,
    focused: &'a Option<String>,
}

/// Render one collapsible section of the catalog tree.
///
/// Validates: Requirement 1.4, 1.5, 4.1, 4.3, 10.1
fn render_section(
    ui: &mut egui::Ui,
    header: egui::RichText,
    default_open: bool,
    entries: Vec<&VirtualCatalog>,
    filter: &str,
    ctx: &mut SectionCtx<'_>,
) {
    egui::CollapsingHeader::new(header)
        .default_open(default_open)
        .show(ui, |ui| {
            let visible: Vec<_> = entries
                .iter()
                .filter(|c| filter.is_empty() || c.name.to_lowercase().contains(filter))
                .collect();
            if visible.is_empty() {
                ui.label(
                    egui::RichText::new(
                        "  No catalogs defined \u{2014} click New Catalog to create one",
                    )
                    .monospace()
                    .weak(),
                );
            } else {
                for cat in visible.iter() {
                    let is_selected = ctx
                        .content
                        .selected_catalog
                        .as_deref()
                        .is_some_and(|s| s == cat.name);
                    let is_focused = ctx.focused.as_deref().is_some_and(|f| f == cat.name);
                    let label_text =
                        egui::RichText::new(format!("  \u{1f4c1} {}", cat.name)).monospace();
                    let resp = ui.selectable_label(is_selected, label_text);
                    // Validates: Requirement 20.1 -- draw focus highlight on keyboard-focused row.
                    if is_focused {
                        ui.painter().rect_stroke(
                            resp.rect,
                            2.0,
                            egui::Stroke::new(1.5_f32, ui.visuals().selection.stroke.color),
                            egui::StrokeKind::Inside,
                        );
                    }
                    // Left-click selects catalog -- Req 10.1
                    if resp.clicked() {
                        ctx.content.selected_catalog = Some(cat.name.clone());
                        ctx.content.path_segments.clear();
                        ctx.content.content_filter.clear();
                    }
                    // Right-click context menu -- Req 4.1, 4.3, 5.1
                    resp.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            *ctx.action = Some(FilesPanelAction::EditCatalog(cat.name.clone()));
                            ui.close();
                        }
                        if ui.button("Allocate Dataset").clicked() {
                            *ctx.action = Some(FilesPanelAction::AllocateDataset(cat.name.clone()));
                            ui.close();
                        }
                        if ui.button("Delete Catalog").clicked() {
                            *ctx.action = Some(FilesPanelAction::DeleteCatalog(cat.name.clone()));
                            ui.close();
                        }
                    });
                }
            }
        });
}
