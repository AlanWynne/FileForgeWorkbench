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

    /// Alias table: the alias editor's REAL columns
    /// `[Act, CE, Command, Alias, Parameters, Description]` with the current
    /// working rows' real values (A2, command-configurator Req 7.2). `Command` is
    /// the root canonical verb, `Alias` the surface name, `Description` the
    /// CR-CH-075 optional human-readable note.
    pub(super) fn alias_table(&self) -> Table {
        let rows = self
            .alias_editor_panel
            .rows
            .iter()
            .map(|r| {
                vec![
                    r.act.clone(),
                    r.environment.label().to_string(),
                    r.canonical.clone(),
                    r.surface.clone(),
                    r.command_template.clone(),
                    r.description.clone(),
                ]
            })
            .collect();
        Table {
            caption: Some("Aliases".to_string()),
            headers: vec![
                "Act".to_string(),
                "CE".to_string(),
                "Command".to_string(),
                "Alias".to_string(),
                "Parameters".to_string(),
                "Description".to_string(),
            ],
            rows,
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
