//! # Per-Context ScreenProvider model builders (Stream SCRM C.2 / C.3)
//!
//! One `*_screen_model()` method per core Workspace Context in the
//! master-screen-directive retrofit matrix (Requirement 5). Each builds a
//! logical [`ScreenModel`] from the Context's OWN state struct (its panel state
//! on the shell), NOT from egui widgets (screen-snapshot-scrm Requirement 2.2).
//!
//! The dispatch seam (`shell/screen_provider.rs`) routes each `KindTag` here,
//! replacing the Phase 1A documented opt-out with a real model so SNAPSHOT /
//! CAPTURE work on every Context (master-screen-directive Requirement 4.1, 4.2).
//! A faithful-but-reasonable model is intended for Phase 2A: the Context title
//! plus the key fields / rows / messages the Context shows. ANSI colour is a
//! later enhancement (fields keep the default colour).
//!
//! Validates: master-screen-directive Requirement 4.1, 4.2, 4.4;
//! screen-snapshot-scrm Requirement 2.1, 2.2.

use ff_screen_model::{Field, ScreenModel};

use super::WorkbenchShell;
use crate::tab_state::KindTag;

/// The descriptive display title for the active Context's Kind, used as the
/// screen-model title (the on-screen Title_Line). Falls back to the cached tab
/// title when the Kind is not a built-in panel Kind.
fn active_display_title(shell: &WorkbenchShell) -> String {
    let tab = shell.tabs.active_tab();
    crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)
        .display_title()
        .to_string()
}

impl WorkbenchShell {
    /// Config Panel (Settings / CONFIG) -- the flat config-key browser. The
    /// model carries the active filter and namespace scope plus each staged
    /// working edit, so a capture reflects what the user is editing. ANSI
    /// colour for changed/error keys is a later enhancement (Req note).
    pub(super) fn config_panel_screen_model(&self) -> ScreenModel {
        let p = &self.config_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        if let Some(ns) = &p.namespace_filter {
            model = model.with_field(Field::new("Namespace", ns.clone()));
        }
        model = model.with_field(Field::new("Filter", p.filter.clone()));
        model = model.with_field(Field::new(
            "Unsaved edits",
            if p.is_dirty() { "yes" } else { "no" }.to_string(),
        ));
        for key in p.working.keys() {
            model = model.with_field(Field::new(key.clone(), "(staged)"));
        }
        if let Some(err) = &p.persist_error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Theme Editor -- lists the editable colour tokens as label/value pairs.
    pub(super) fn theme_editor_screen_model(&self) -> ScreenModel {
        use crate::theme_editor_panel::EditableToken;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        for token in EditableToken::all() {
            model = model.with_field(Field::new(token.label().to_string(), String::new()));
        }
        model
    }

    /// Menus Editor -- the selected menu plus the list of editable menus.
    pub(super) fn menus_editor_screen_model(&self) -> ScreenModel {
        let p = &self.menus_editor_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model = model.with_field(Field::new(
            "Selected",
            p.selected.clone().unwrap_or_default(),
        ));
        for name in &p.available {
            model = model.with_field(Field::new("Menu", name.clone()));
        }
        if let Some(err) = &p.error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Keys Editor -- the selected kind plus one row per function key showing
    /// the Base-layer command.
    pub(super) fn keys_editor_screen_model(&self) -> ScreenModel {
        let p = &self.keys_editor_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model = model.with_field(Field::new(
            "Kind",
            p.selected_kind.clone().unwrap_or_default(),
        ));
        for row in &p.rows {
            model = model.with_field(Field::new(
                row.key.display_name().to_string(),
                row.commands[0].clone(),
            ));
        }
        if let Some(err) = &p.error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Kinds Editor -- the selected Kind plus the list of available Kinds.
    pub(super) fn kinds_editor_screen_model(&self) -> ScreenModel {
        let p = &self.kinds_editor_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model = model.with_field(Field::new(
            "Selected",
            p.selected.clone().unwrap_or_default(),
        ));
        for name in &p.kind_names {
            model = model.with_field(Field::new("Kind", name.clone()));
        }
        if let Some(err) = &p.error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Alias Editor -- one row per alias (surface -> root, with CE + template).
    pub(super) fn alias_editor_screen_model(&self) -> ScreenModel {
        let p = &self.alias_editor_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        for row in &p.rows {
            let value = if row.command_template.is_empty() {
                row.canonical.clone()
            } else {
                format!("{} {}", row.canonical, row.command_template)
            };
            let label = if row.surface.is_empty() {
                "(alias)".to_string()
            } else {
                row.surface.clone()
            };
            model = model.with_field(Field::new(label, value));
        }
        if let Some(err) = &p.error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Command Configurator -- one row per command definition (id -> label).
    pub(super) fn command_configurator_screen_model(&self) -> ScreenModel {
        let p = &self.command_configurator_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        for def in p.displayed(&self.command_store.definitions) {
            model = model.with_field(Field::new(def.id.clone(), def.label.clone()));
        }
        if let Some(err) = &p.error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Catalog Explorer (Virtual Catalog Manager) -- PARKED in the core build
    /// (core-001 strip: the catalog domain is removed). The Context renders an
    /// "unavailable in core" notice; the model records exactly that so the
    /// capture is faithful rather than empty.
    pub(super) fn files_panel_screen_model(&self) -> ScreenModel {
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model
            .messages
            .push("Catalog Explorer is unavailable in the core build.".to_string());
        model
    }

    /// File Explorer -- PARKED in the core build (core-001 strip). The model
    /// records the "unavailable in core" notice the Context renders.
    pub(super) fn file_explorer_screen_model(&self) -> ScreenModel {
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model
            .messages
            .push("File Explorer is unavailable in the core build.".to_string());
        model
    }

    /// Search Results -- the query plus one row per matched file with its hit
    /// count.
    pub(super) fn search_results_screen_model(&self) -> ScreenModel {
        let p = &self.search_results_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model = model.with_field(Field::new("Query", p.query.clone()));
        for fm in &p.results {
            model = model.with_field(Field::new(
                fm.file_path.clone(),
                format!("{} match(es)", fm.matches.len()),
            ));
        }
        if let Some(err) = &p.regex_error {
            model.messages.push(err.clone());
        }
        model
    }

    /// Plugin (Extension) Manager -- one row per discovered plugin (name ->
    /// state).
    pub(super) fn plugin_manager_screen_model(&self) -> ScreenModel {
        let p = &self.plugin_manager_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        for (name, state) in &p.plugins {
            model = model.with_field(Field::new(name.clone(), format!("{state:?}")));
        }
        model
    }

    /// Event Log -- the active filters plus the current notification entries.
    pub(super) fn event_log_screen_model(&self) -> ScreenModel {
        let p = &self.event_log_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model = model.with_field(Field::new(
            "Level filter",
            match p.level_filter {
                Some(l) => format!("{l:?}"),
                None => "all".to_string(),
            },
        ));
        model = model.with_field(Field::new("Text filter", p.text_filter.clone()));
        let queue = self
            .notification_queue
            .lock()
            .expect("notification queue lock");
        for entry in queue.entries() {
            model
                .messages
                .push(format!("[{}] {}", entry.level.label(), entry.title));
        }
        model
    }

    /// Macro Library -- the filter plus one row per discovered macro.
    pub(super) fn macro_library_screen_model(&self) -> ScreenModel {
        let p = &self.macro_library_panel;
        let mut model = ScreenModel::new(active_display_title(self));
        model.command_line = self.command_text.clone();
        model = model.with_field(Field::new("Filter", p.filter.clone()));
        for entry in &p.entries {
            model = model.with_field(Field::new(entry.name.clone(), entry.dir.clone()));
        }
        model
    }

    /// Help Context -- the current topic title and body (the on-screen help
    /// text), or a note when no topic is shown.
    pub(super) fn help_context_screen_model(&self) -> ScreenModel {
        // Help / SCRM viewer are transient Contexts, not configurable Kinds, so
        // they use an explicit title rather than the Kind display-title path.
        let mut model = ScreenModel::new("Help".to_string());
        model.command_line = self.command_text.clone();
        match self.help.context_panel.current_topic_text() {
            Some((title, body)) => {
                model = model.with_field(Field::new("Topic", title));
                for line in body.lines() {
                    model.messages.push(line.to_string());
                }
            }
            None => model.messages.push("(no help topic shown)".to_string()),
        }
        model
    }

    /// SCRM Replay viewer -- the replay position plus the current capture's
    /// screen text as free-standing messages. Decision (master-directive Req 5
    /// "decide in SCRM stream"): the ScrmViewer HAS logical content (the replay
    /// position + the current captured screen), so it gets a real PROVIDER, not
    /// a documented opt-out.
    pub(super) fn scrm_viewer_screen_model(&self) -> ScreenModel {
        let p = &self.scrm_viewer;
        let mut model = ScreenModel::new("Screen Replay".to_string());
        model.command_line = self.command_text.clone();
        let (count, position) = p.replay_summary();
        model = model.with_field(Field::new(
            "Capture",
            if count == 0 {
                "0 / 0".to_string()
            } else {
                format!("{} / {}", position + 1, count)
            },
        ));
        for line in p.current_screen_snapshot_text().lines() {
            model.messages.push(line.to_string());
        }
        model
    }
}

/// A compile-time reminder that every retrofitted `KindTag` has a model builder
/// above. If a new Context Kind is added, extend this match AND the dispatch in
/// `screen_provider.rs` (master-screen-directive Req 4.4: a model is the
/// default, opt-out is the rare documented exception).
#[allow(dead_code)]
fn retrofit_matrix_is_covered(tag: KindTag) -> bool {
    matches!(
        tag,
        KindTag::ConfigPanel
            | KindTag::ThemeEditor
            | KindTag::MenusEditor
            | KindTag::KeysEditor
            | KindTag::KindsEditor
            | KindTag::AliasEditor
            | KindTag::CommandConfigurator
            | KindTag::FilesPanel
            | KindTag::FileExplorerPanel
            | KindTag::SearchResults
            | KindTag::PluginManager
            | KindTag::EventLog
            | KindTag::MacroLibrary
            | KindTag::HelpContext
            | KindTag::ScrmViewer
            | KindTag::MenuWorkspace
    )
}
