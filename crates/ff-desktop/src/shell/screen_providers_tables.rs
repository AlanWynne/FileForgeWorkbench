//! # Editor-Context snapshot TABLES (Stream GROUP A, capture-fidelity A2)
//!
//! Table builders for the tabular editor Contexts, split out of
//! `shell/screen_providers.rs` for the 400-line rule. Each helper reads the
//! editor's OWN panel state (never egui) and returns an `ff_screen_model::Table`
//! with the editor's REAL columns and current values, so the Markdown / HTML /
//! text snapshot renders a proper aligned table after the box drawing (the
//! renderers already emit `model.tables`, findings 6 -- this is PROVIDER work
//! only). A1 (Theme hex values) feeds the Theme table from the working palette.
//!
//! Validates: screen-snapshot-scrm Requirement 27.8; master-screen-directive
//! Requirement 4.4a.

use ff_screen_model::Table;
use ff_theme::ThemePalette;

use super::WorkbenchShell;
use crate::theme_editor_panel::EditableToken;

impl WorkbenchShell {
    /// Theme table: one row per editable token `[Setting, Value]`, the Value
    /// being the token's REAL hex colour read from the Theme editor WORKING
    /// palette (A1: fixes the former blank-value bug, findings 1b). Falls back
    /// to the active running palette when no working copy is loaded.
    pub(super) fn theme_table(&self) -> Table {
        let palette: &ThemePalette = self
            .theme_editor_panel
            .working
            .as_ref()
            .unwrap_or(&self.palette);
        let rows = EditableToken::all()
            .iter()
            .map(|token| vec![token.label().to_string(), token.get(palette).to_hex()])
            .collect();
        Table {
            caption: Some("Theme settings".to_string()),
            headers: vec!["Setting".to_string(), "Value".to_string()],
            rows,
        }
    }

    /// Alias table: the DISPLAYED alias rows (in BACKING order, honouring the
    /// FILTER row) rendered through the shared ISPF snapshot conventions, THEN an
    /// always-present trailing SCRATCH row -- so the captured grid matches the
    /// live editor (CR-NR-116, command-configurator Req 7.16-7.18,
    /// ispf-table-editor Req 12.3/12.5).
    ///
    /// The FILTER row is NEVER a table row. CE + Command render as `[value v]`
    /// combo cells (CE uppercased via `env_name()`); the rest are text cells. An
    /// empty working copy yields ONLY the scratch row. The caption stays
    /// `Some("Aliases")` for the Markdown / HTML section heading.
    ///
    /// Validates: command-configurator Requirement 7.16, 7.17, 7.18
    pub(super) fn alias_table(&self) -> Table {
        use crate::alias_editor_panel::{
            alias_snapshot_cells, displayed_indices, AliasRowEdit, ALIAS_COLUMNS,
        };
        let p = &self.alias_editor_panel;
        // Displayed set in BACKING order (the filter is display-only, Req 5.15).
        let rows: Vec<Vec<String>> = displayed_indices(&p.rows, p.filter_ce, &p.filter_root)
            .into_iter()
            .map(|i| alias_snapshot_cells(&p.rows[i]))
            .collect();
        // The always-present trailing scratch row (the blank row, Req 12.5).
        let scratch = alias_snapshot_cells(&AliasRowEdit::default());
        let mut table =
            crate::shell::ispf_table_editor::snapshot_table(&ALIAS_COLUMNS, &rows, &scratch);
        table.caption = Some("Aliases".to_string());
        table
    }

    /// Alias Markdown section: the RAW alias values (no `[v]` wrapper, no `_`
    /// fill, no padding) with NO scratch row, for a Context's
    /// `markdown_sections` clipboard flavour (CR-NR-116, ispf-table-editor
    /// Req 12.3, screen-snapshot-scrm Req 30.40). The heading is the caption
    /// "Aliases".
    pub(super) fn alias_markdown_section(&self) -> ff_screen_model::MarkdownSection {
        use crate::alias_editor_panel::{alias_snapshot_cells, displayed_indices, ALIAS_COLUMNS};
        let p = &self.alias_editor_panel;
        let rows: Vec<Vec<String>> = displayed_indices(&p.rows, p.filter_ce, &p.filter_root)
            .into_iter()
            .map(|i| alias_snapshot_cells(&p.rows[i]))
            .collect();
        let table = crate::shell::ispf_table_editor::raw_table(&ALIAS_COLUMNS, &rows);
        ff_screen_model::MarkdownSection {
            heading: Some("Aliases".to_string()),
            table,
        }
    }

    /// Menu table: the currently-edited menu's REAL `MenuOption` columns
    /// `[Option, Command, Description, Group, On, Bar]` with real values (A2).
    /// Empty rows when no menu working copy is loaded.
    pub(super) fn menu_table(&self) -> Table {
        let rows = self
            .menus_editor_panel
            .working
            .as_ref()
            .map(|menu| {
                menu.options
                    .iter()
                    .map(|o| {
                        vec![
                            o.key.clone(),
                            o.command.clone(),
                            o.description.clone(),
                            o.group.clone().unwrap_or_default(),
                            if o.enabled { "yes" } else { "no" }.to_string(),
                            if o.show_in_menu_bar { "yes" } else { "no" }.to_string(),
                        ]
                    })
                    .collect()
            })
            .unwrap_or_default();
        Table {
            caption: Some("Menu options".to_string()),
            headers: vec![
                "Option".to_string(),
                "Command".to_string(),
                "Description".to_string(),
                "Group".to_string(),
                "On".to_string(),
                "Bar".to_string(),
            ],
            rows,
        }
    }

    /// Keys table: one row per edited function key `[Key, Base, Shift, Ctrl,
    /// Alt]` with the real per-modifier-layer command assignments (A2).
    pub(super) fn keys_table(&self) -> Table {
        let rows = self
            .keys_editor_panel
            .rows
            .iter()
            .map(|r| {
                vec![
                    r.key.display_name().to_string(),
                    r.commands[0].clone(),
                    r.commands[1].clone(),
                    r.commands[2].clone(),
                    r.commands[3].clone(),
                ]
            })
            .collect();
        Table {
            caption: Some("Key assignments".to_string()),
            headers: vec![
                "Key".to_string(),
                "Base".to_string(),
                "Shift".to_string(),
                "Ctrl".to_string(),
                "Alt".to_string(),
            ],
            rows,
        }
    }

    /// Kinds table: one row per available Workspace Kind `[Kind, Modelled on,
    /// Title, Menu bar, Key list]` with its real effective config values (A2).
    pub(super) fn kinds_table(&self) -> Table {
        let rows = self
            .kinds_editor_panel
            .kind_names
            .iter()
            .map(|name| {
                let cfg = self.kind_registry.effective(name);
                vec![
                    cfg.name.clone(),
                    cfg.modelled_on.to_tag(),
                    cfg.title.clone(),
                    cfg.menu_bar.clone().unwrap_or_default(),
                    cfg.key_list.clone().unwrap_or_default(),
                ]
            })
            .collect();
        Table {
            caption: Some("Workspace kinds".to_string()),
            headers: vec![
                "Kind".to_string(),
                "Modelled on".to_string(),
                "Title".to_string(),
                "Menu bar".to_string(),
                "Key list".to_string(),
            ],
            rows,
        }
    }
}
