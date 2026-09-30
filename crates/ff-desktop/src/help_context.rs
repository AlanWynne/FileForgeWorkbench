//! Help Context -- the Workspace Context that displays context-sensitive help.
//!
//! The `ff-help` engine (context detection, registry, search, navigation) already
//! exists; this shell-side Context RENDERS a resolved topic. It owns a
//! [`HelpPanelModel`] (the GUI-free state) and implements [`WorkspaceContext`] so
//! it is dispatched through the single `render_workspace_context` focus-latch
//! path (CR-NR-078, workspace-conformance). The FIRST interior focus stop is the
//! Help_Search field, which carries a stable [`egui::Id`] so the CR-CH-023
//! Boundary_Policy can latch the command-field -> first-interior Tab jump to a
//! real, non-phantom widget.
//!
//! Validates: context-help Requirement 18.2, 18.5 (CR-NR-097).

use std::sync::Arc;

use eframe::egui;
use ff_help::{HelpConfig, HelpPanelModel, HelpTopicRegistry, TopicKey};

use crate::shell::workspace_context::{InteriorFocus, ShellServices, WorkspaceContext};

/// Stable egui id of the Help_Search field -- the FIRST interior control of the
/// Help Context. Reported as `first_interior_id` so the Boundary_Policy latches
/// the command-field -> first-interior Tab jump to a real widget (no phantom
/// stop, workspace-conformance / B056-B059 class).
pub fn help_search_field_id() -> egui::Id {
    egui::Id::new("help_search_field")
}

/// Shell-side state for the Help Context.
///
/// Wraps the `ff-help` [`HelpPanelModel`]. Constructed once (with the shell-owned
/// registry) and reused; opening a topic mutates the model in place.
pub struct HelpContextPanel {
    /// The GUI-free help panel model (current topic, navigation, search).
    model: HelpPanelModel,
    /// The search field text buffer (bound to the Help_Search field).
    search_text: String,
}

impl HelpContextPanel {
    /// Create a Help Context panel over the shared registry.
    pub fn new(registry: Arc<HelpTopicRegistry>, config: HelpConfig) -> Self {
        Self {
            model: HelpPanelModel::new(registry, config),
            search_text: String::new(),
        }
    }

    /// Open (or navigate to) a topic for display. Returns whether the topic was
    /// shown; on error the caller falls back to the index-with-message path.
    pub fn show(&mut self, key: &TopicKey) -> bool {
        self.model.show_topic(key).is_ok()
    }

    /// Directly display an already-built topic body (used for dynamically
    /// generated topics that are not in the registry, e.g. the index or the
    /// function-key table).
    pub fn model_mut(&mut self) -> &mut HelpPanelModel {
        &mut self.model
    }

    /// Read access to the model (used by tests to assert the displayed topic).
    #[cfg(test)]
    pub fn model(&self) -> &HelpPanelModel {
        &self.model
    }
}

impl Default for HelpContextPanel {
    /// A placeholder panel over an empty registry. Used only as the transient
    /// value `std::mem::take` leaves behind during the shell's owned-panel swap;
    /// the real panel (with the shell-owned registry) is immediately put back.
    fn default() -> Self {
        Self::new(Arc::new(HelpTopicRegistry::new()), HelpConfig::default())
    }
}

impl WorkspaceContext for HelpContextPanel {
    fn render(&mut self, ui: &mut egui::Ui, _services: &mut ShellServices<'_>) -> InteriorFocus {
        // --- Search field (FIRST interior control, stable id) ---------------
        ui.horizontal(|ui| {
            ui.label("Search:");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.search_text)
                    .id(help_search_field_id())
                    .desired_width(240.0)
                    .hint_text("type to search help"),
            );
            if resp.changed() {
                self.model.search(&self.search_text);
            }
        });

        ui.separator();

        // --- Breadcrumb -----------------------------------------------------
        let crumb: Vec<String> = self
            .model
            .breadcrumb()
            .iter()
            .map(|b| b.label.clone())
            .collect();
        if !crumb.is_empty() {
            ui.label(crumb.join(" > "));
        }

        // --- Body -----------------------------------------------------------
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(topic) = self.model.current_topic() {
                    ui.heading(topic.title());
                    ui.add_space(4.0);
                    // Render the Markdown body as monospace text. A richer
                    // Markdown renderer is a later enhancement; the model carries
                    // raw Markdown by design (content is data, not code).
                    ui.label(egui::RichText::new(topic.body()).monospace());
                } else {
                    ui.label("No help topic is open.");
                }
            });

        // The Help_Search field is both first and last interior stop; egui-native
        // Tab walks any controls in between (workspace-conformance pattern).
        InteriorFocus::single(help_search_field_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_help::TopicSource;

    fn registry_with(key: TopicKey, title: &str, body: &str) -> Arc<HelpTopicRegistry> {
        let reg = HelpTopicRegistry::new();
        reg.register_file_topic(ff_help::HelpTopic::new(
            key,
            title.to_string(),
            body.to_string(),
            TopicSource::FileBased {
                file_path: std::path::PathBuf::from("t.help.md"),
            },
        ));
        Arc::new(reg)
    }

    // Validates: Requirement 18.2 -- showing a resolved topic opens the model.
    #[test]
    fn show_opens_resolved_topic() {
        let key = TopicKey::command("FIND");
        let reg = registry_with(key.clone(), "FIND", "Find body");
        let mut panel = HelpContextPanel::new(reg, HelpConfig::default());
        assert!(panel.show(&key));
        assert!(panel.model().is_open());
        assert_eq!(panel.model().current_topic_key(), Some(&key));
    }

    // Validates: Requirement 18.2 -- showing a missing topic reports false.
    #[test]
    fn show_missing_topic_returns_false() {
        let reg = Arc::new(HelpTopicRegistry::new());
        let mut panel = HelpContextPanel::new(reg, HelpConfig::default());
        assert!(!panel.show(&TopicKey::command("NOPE")));
    }
}
