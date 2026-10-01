//! # Files Panel content-area render
//!
//! The right-side content area: SQLite-backed entry population for Mainframe
//! catalogs, breadcrumb navigation, content filter, sortable column headers,
//! and the entry rows (double-click to open / navigate).
//!
//! Validates: Requirement 10.1-10.6, 13.3

use ff_catalog_registry::CatalogType;

use crate::resolve::FilesPanelState;
use crate::state::{ContentAreaState, ContentEntry, FilesPanelAction, SortColumn, SortDir};

/// Render the right-side content area.
///
/// Validates: Requirement 10.1-10.6, 13.3
pub(crate) fn render_content_area(
    ui: &mut egui::Ui,
    state: &mut FilesPanelState,
) -> Option<FilesPanelAction> {
    let mut action: Option<FilesPanelAction> = None;

    if state.content.selected_catalog.is_none() {
        ui.label(
            egui::RichText::new("Select a catalog to browse its contents.")
                .monospace()
                .weak(),
        );
        return None;
    }

    // Req 13.2 -- populate entries from SQLite for Mainframe catalogs;
    // fall back to load_entries_from_datasets for non-Mainframe catalogs.
    if let Some(cat) = state.content.selected_catalog.clone() {
        let is_mainframe = state
            .registry
            .get_by_name(&cat)
            .map(|c| c.catalog_type == CatalogType::Mainframe)
            .unwrap_or(false);
        if is_mainframe {
            let entries = state
                .registry
                .list_datasets(&cat)
                .unwrap_or_default()
                .into_iter()
                .map(|d| {
                    let is_container = matches!(
                        d.dsorg,
                        ff_dscatalog::dataset::Dsorg::PO | ff_dscatalog::dataset::Dsorg::GDG
                    );
                    ContentEntry {
                        name: d.dsn.as_str().to_string(),
                        entry_type: d.dsorg.to_string(),
                        size: String::new(),
                        modified: d.modified.unwrap_or_default(),
                        is_container,
                    }
                })
                .collect();
            state.content.entries = entries;
        } else {
            state.content.entries.clear();
        }
    }

    // === Breadcrumb bar -- Req 10.5 =========================================
    ui.horizontal(|ui| {
        let cat_name = state.content.selected_catalog.clone().unwrap_or_default();
        // Root segment (catalog name)
        if ui
            .selectable_label(false, egui::RichText::new(&cat_name).monospace())
            .clicked()
        {
            state.content.navigate_to_segment(0);
        }
        let segments = state.content.path_segments.clone();
        for (i, seg) in segments.iter().enumerate() {
            ui.label(egui::RichText::new(" / ").monospace().weak());
            if ui
                .selectable_label(false, egui::RichText::new(seg).monospace())
                .clicked()
            {
                state.content.navigate_to_segment(i + 1);
            }
        }
    });
    ui.separator();

    // === Content filter -- Req 10.6 =========================================
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.add(
            egui::TextEdit::singleline(&mut state.content.content_filter)
                .desired_width(200.0)
                .hint_text("name\u{2026}")
                .font(egui::TextStyle::Monospace),
        );
    });
    ui.separator();

    // === Column headers -- Req 10.2 =========================================
    ui.horizontal(|ui| {
        let col_btn = |ui: &mut egui::Ui,
                       label: &str,
                       col: SortColumn,
                       content: &mut ContentAreaState|
         -> bool {
            let indicator = if content.sort_col == col {
                match content.sort_dir {
                    SortDir::Ascending => " \u{25b2}",
                    SortDir::Descending => " \u{25bc}",
                }
            } else {
                ""
            };
            ui.button(
                egui::RichText::new(format!("{label}{indicator}"))
                    .monospace()
                    .strong(),
            )
            .clicked()
                && {
                    content.toggle_sort(col);
                    true
                }
        };
        col_btn(ui, "Name", SortColumn::Name, &mut state.content);
        col_btn(ui, "Type", SortColumn::Type, &mut state.content);
        col_btn(ui, "Size", SortColumn::Size, &mut state.content);
        col_btn(ui, "Modified", SortColumn::Modified, &mut state.content);
    });
    ui.separator();

    // === Entry rows -- Req 10.1, 10.3, 10.4 =================================
    let visible: Vec<ContentEntry> = state
        .content
        .visible_entries()
        .into_iter()
        .cloned()
        .collect();

    if visible.is_empty() {
        ui.label(
            egui::RichText::new("No entries to display.")
                .monospace()
                .weak(),
        );
    } else {
        for entry in &visible {
            let icon = if entry.is_container {
                "\u{1f4c1}"
            } else {
                "\u{1f4c4}"
            };
            let row_text = format!(
                "{icon} {:<40} {:<12} {:<10} {}",
                entry.name, entry.entry_type, entry.size, entry.modified
            );
            let resp = ui.selectable_label(false, egui::RichText::new(row_text).monospace());
            if resp.double_clicked() {
                if entry.is_container {
                    state.content.push_path(entry.name.clone());
                    action = Some(FilesPanelAction::NavigateInto(entry.name.clone()));
                } else {
                    action = Some(FilesPanelAction::OpenFile(entry.name.clone()));
                }
            }
        }
    }

    action
}
