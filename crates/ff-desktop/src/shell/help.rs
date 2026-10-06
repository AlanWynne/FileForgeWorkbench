//! Shell-side HELP / F1 wiring (context-help Requirement 18, 19; CR-NR-097).
//!
//! The `ff-help` engine resolves a `Topic_Key`; this module DISPLAYS it. The
//! shell owns a single `Arc<HelpTopicRegistry>` (loaded once at startup) and a
//! `HelpContextPanel`; `open_help` resolves the request against that registry,
//! opens the Help Context on success, generates the dynamic topics (index,
//! function keys) at display time, and on a miss records the miss + shows the
//! index with a message (never only a command-line error).
//!
//! Validates: context-help Requirement 18.1-18.7, 19.1-19.4 (CR-NR-097).

use ff_help::{
    resolve_help_argument, ContextDetector, DynamicContentGenerator, EditorContext, EditorMode,
    FunctionKeyBinding, HelpAction, HelpTopic, KeyMapAccess, TopicKey, TopicSource,
};
use ff_keys::FunctionKey;

use super::WorkbenchShell;
use crate::tab_state::TabKind;

/// Adapter exposing the shell's active key map to `ff-help`'s dynamic
/// function-key topic generator (Req 15, 18.3). Built per generation from the
/// active `KeyMapResolver` map; not retained.
struct ShellKeyMapAccess {
    bindings: Vec<FunctionKeyBinding>,
    profile: Option<String>,
}

impl ShellKeyMapAccess {
    fn from_shell(shell: &WorkbenchShell) -> Self {
        let map = shell.key_map_resolver.active_key_map();
        let bindings = FunctionKey::ALL
            .iter()
            .filter_map(|&fk| {
                map.get_plain(fk).map(|binding| FunctionKeyBinding {
                    key: fk.to_string(),
                    command_id: binding.command().to_string(),
                    label: binding.display_label().to_string(),
                })
            })
            .collect();
        Self {
            bindings,
            profile: shell
                .key_map_resolver
                .active_profile_name()
                .map(|s| s.to_string()),
        }
    }
}

impl KeyMapAccess for ShellKeyMapAccess {
    fn function_key_bindings(&self) -> Vec<FunctionKeyBinding> {
        self.bindings.clone()
    }
    fn active_profile_name(&self) -> Option<String> {
        self.profile.clone()
    }
}

impl WorkbenchShell {
    /// Handle the HELP command / F1 activation. `arg` is the text after `HELP`
    /// (empty for bare `HELP` or F1). Routes through the single command path;
    /// never recorded in history or undo (Req 18.7).
    ///
    /// Validates: context-help Requirement 18.2-18.4, 19.1-19.4 (CR-NR-097).
    pub(super) fn open_help(&mut self, arg: &str) {
        let arg_trimmed = arg.trim();

        // `HELP MISSING` -- diagnostics report (Req 19.3).
        if arg_trimmed.eq_ignore_ascii_case("MISSING") {
            self.show_help_missing_report();
            self.open_error = None;
            return;
        }

        // Resolve the requested Topic_Key.
        let key = if arg_trimmed.is_empty() {
            // Bare HELP / F1: context detection (CR-NR-079 -- a focused menu
            // option resolves that option's command topic).
            match self.resolve_f1_topic() {
                Some(k) => k,
                None => {
                    // HELP OFF-equivalent never reached here; bare with no context
                    // resolves to the index inside resolve_f1_topic.
                    TopicKey::index()
                }
            }
        } else {
            match resolve_help_argument(arg_trimmed) {
                HelpAction::ShowTopic(k) => k,
                HelpAction::Close => {
                    self.close_help();
                    self.open_error = None;
                    return;
                }
                HelpAction::UnrecognisedTopic(name) => {
                    self.show_help_index_with_message(&format!("No help available for: {name}"));
                    self.open_error = None;
                    return;
                }
            }
        };

        self.display_help_topic(&key);
        self.open_error = None;
    }

    /// Resolve the F1 context topic from the Cursor_Context snapshot + command
    /// line (mirrors the pre-CR-NR-097 handler, minus the throwaway registry).
    fn resolve_f1_topic(&self) -> Option<TopicKey> {
        let cc = self
            .cursor_context_snapshot
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default();
        let is_menu_option = cc
            .focused_identity
            .as_deref()
            .map(|id| id != "command-line")
            .unwrap_or(false);
        let command_line_text = if is_menu_option {
            cc.focused_identity.clone().unwrap_or_default()
        } else {
            self.command_text.clone()
        };
        let ctx = EditorContext {
            command_line_text,
            command_line_has_focus: true,
            prefix_area_text: None,
            prefix_area_has_focus: false,
            active_mode: EditorMode::Edit,
            help_panel_open: false,
            current_help_topic: None,
        };
        Some(ContextDetector::resolve(&ctx))
    }

    /// Display a resolved topic in the Help Context. Dynamic topics (index,
    /// function keys) are generated at display time; a missing non-dynamic topic
    /// records a miss and falls back to the index with a message.
    ///
    /// Validates: context-help Requirement 18.2, 18.3, 18.4, 19.1, 19.2.
    fn display_help_topic(&mut self, key: &TopicKey) {
        // Dynamic topics (Req 18.3): generated, not file-based.
        if key == &TopicKey::index() {
            let topic = DynamicContentGenerator::generate_index(&self.help_registry, app_version());
            self.open_help_context_with(topic);
            return;
        }
        if key == &TopicKey::feature("function_keys") {
            let topic = self.generate_function_keys_topic();
            self.open_help_context_with(topic);
            return;
        }

        // File-based / runtime topic (Req 18.2).
        if self.help_registry.contains(key) {
            self.open_help_context();
            self.help_context_panel.show(key);
            return;
        }

        // Miss (Req 18.4, 19.1, 19.2): record + WARN + index-with-message.
        self.record_help_miss(key);
        let label = help_label(key);
        self.show_help_index_with_message(&format!(
            "Help not yet available for {label} [topic-key: {}]",
            key.as_str()
        ));
    }

    /// Record a missing-topic event: WARN log + increment the session tally.
    /// Performs NO writes to any project document (Req 19.5).
    ///
    /// Validates: context-help Requirement 19.1, 19.2, 19.5.
    fn record_help_miss(&mut self, key: &TopicKey) {
        let label = help_label(key);
        ff_logging::log_warn!(
            "[help] lookup: topic not found -- {} ({})",
            key.as_str(),
            label
        );
        *self
            .help_missing_tally
            .entry(key.as_str().to_string())
            .or_insert(0) += 1;
    }

    /// The Help Context report for `HELP MISSING` (Req 19.3, 19.4): each tallied
    /// key with its count, classified EXPECTED (in the shipped/promised set) vs
    /// UNEXPECTED (a coverage gap for triage).
    fn show_help_missing_report(&mut self) {
        let mut rows: Vec<(String, u32)> = self
            .help_missing_tally
            .iter()
            .map(|(k, n)| (k.clone(), *n))
            .collect();
        rows.sort();

        let mut body = String::from(
            "# Missing Help Topics\n\n\
             Topics requested this session that have no authored content. This is a\n\
             diagnostic report; it does not modify any project files.\n\n",
        );
        if rows.is_empty() {
            body.push_str("No missing topics recorded this session.\n");
        } else {
            body.push_str("| Topic Key | Requests | Class |\n|---|---|---|\n");
            for (k, n) in &rows {
                let class = if is_expected_topic(k) {
                    "EXPECTED"
                } else {
                    "UNEXPECTED"
                };
                body.push_str(&format!("| `{k}` | {n} | {class} |\n"));
            }
            body.push_str(
                "\nEXPECTED topics are planned content (authoring backlog). UNEXPECTED\n\
                 topics are coverage gaps -- candidates for deliberate triage.\n",
            );
        }

        let topic = HelpTopic::new(
            TopicKey::feature("help_missing"),
            "Missing Help Topics".to_string(),
            body,
            TopicSource::CommandRegistry {
                command_id: "HELP MISSING".to_string(),
            },
        );
        self.open_help_context_with(topic);
    }

    /// Show the Help Index with a leading message (Req 18.4, 13.7).
    fn show_help_index_with_message(&mut self, message: &str) {
        let index = DynamicContentGenerator::generate_index(&self.help_registry, app_version());
        let body = format!("> {message}\n\n{}", index.body());
        let topic = HelpTopic::new(
            TopicKey::index(),
            index.title().to_string(),
            body,
            TopicSource::CommandRegistry {
                command_id: "HELP".to_string(),
            },
        );
        self.open_help_context_with(topic);
    }

    /// Generate the function-key topic from the active key map (Req 18.3, 15).
    fn generate_function_keys_topic(&self) -> HelpTopic {
        let accessor = ShellKeyMapAccess::from_shell(self);
        DynamicContentGenerator::generate_function_keys(&accessor)
    }

    /// Enter the Help Context on the active tab as a PUSH navigation (F1/HELP is
    /// a navigation, CR-NR-097 / B079): the previous Context descriptor is
    /// pushed onto the tab's Navigation_Stack so END/F3 pops back to it rather
    /// than closing the Workspace (and, when last, exiting the app).
    ///
    /// When the active tab is ALREADY the Help Context (navigating between help
    /// topics, e.g. index -> a command topic), this is a no-op transform in
    /// place: the previous non-Help Context stays a single level down, so one
    /// END/F3 returns to it rather than walking a stack of Help-on-Help frames.
    ///
    /// Validates: menu-workspace Requirement 14.2, 14.4 (via B079).
    fn enter_help_context(&mut self) {
        if self.tabs.active_tab().kind.tag() == crate::tab_state::KindTag::HelpContext {
            // Already in Help: re-render in place, do not push another frame.
            self.set_active_tab_context(TabKind::HelpContext, "[HELP]");
            return;
        }
        let current = self.descriptor_for_current_context();
        self.tabs.active_tab_mut().nav_stack.push(current);
        self.set_active_tab_context(TabKind::HelpContext, "[HELP]");
    }

    /// Open the Help Context on the active tab without changing the displayed
    /// topic (the caller then calls `show`). Enters via the push-navigation seam.
    fn open_help_context(&mut self) {
        self.enter_help_context();
    }

    /// Open the Help Context displaying an already-built (dynamic) topic.
    fn open_help_context_with(&mut self, topic: HelpTopic) {
        self.enter_help_context();
        self.help_context_panel.model_mut().show_generated(topic);
    }

    /// Close the Help Context (HELP OFF): pop back to the Context Help was opened
    /// from. Because entering Help now pushes the previous Context onto the
    /// Navigation_Stack (B079), closing pops that frame -- returning to wherever
    /// the user pressed F1/HELP, not unconditionally to the POM. When the stack
    /// is empty (Help somehow rooted a tab), fall back to Home.
    ///
    /// Validates: context-help Requirement 13.8; menu-workspace Requirement 14.4.
    fn close_help(&mut self) {
        if self.tabs.active_tab().kind.tag() != crate::tab_state::KindTag::HelpContext {
            return;
        }
        if let Some(parent) = self.tabs.active_tab_mut().nav_stack.pop() {
            self.reconstruct_context(&parent);
        } else {
            self.set_active_tab_home();
        }
    }
}

#[cfg(test)]
impl WorkbenchShell {
    /// Test-only: replace the help registry with one preloaded with `topics`,
    /// rebuilding the Help Context panel over it. Lets full-shell tests exercise
    /// the display pipeline without depending on the shipped `help/` directory
    /// being present beside the test binary.
    pub(crate) fn seed_help_registry_for_test(&mut self, topics: Vec<HelpTopic>) {
        let registry = ff_help::HelpTopicRegistry::new();
        registry.load_file_topics(topics);
        let registry = std::sync::Arc::new(registry);
        self.help_context_panel = crate::help_context::HelpContextPanel::new(
            registry.clone(),
            ff_help::HelpConfig::default(),
        );
        self.help_registry = registry;
    }

    /// Test-only: read the current missing-topic tally count for a key.
    pub(crate) fn help_missing_count(&self, key: &str) -> u32 {
        self.help_missing_tally.get(key).copied().unwrap_or(0)
    }

    /// Test-only: the active Help Context panel (to assert the displayed topic).
    pub(crate) fn help_panel_for_test(&self) -> &crate::help_context::HelpContextPanel {
        &self.help_context_panel
    }
}

/// The application version string for the index footer (Req 12.4).
fn app_version() -> &'static str {
    concat!("FileForge Workbench ", env!("CARGO_PKG_VERSION"))
}

/// Human label for a Topic_Key in messages (mirrors ff-help's phrasing).
fn help_label(key: &TopicKey) -> String {
    match key.namespace() {
        Some("cmd") => format!("command \"{}\"", key.identifier()),
        Some("line") => format!("line command \"{}\"", key.identifier()),
        Some("mode") => format!("mode \"{}\"", key.identifier()),
        Some("feature") => format!("feature \"{}\"", key.identifier()),
        Some("config") => format!("config key \"{}\"", key.identifier()),
        Some("api") => format!("macro API \"{}\"", key.identifier()),
        _ => "this topic".to_string(),
    }
}

/// Whether a missed Topic_Key belongs to the shipped/promised content set
/// (Req 17). EXPECTED keys are authoring backlog; anything else is a coverage
/// gap. Kept in sync with the shipped-content test's promised set.
fn is_expected_topic(key: &str) -> bool {
    const EXPECTED_CMDS: &[&str] = &[
        "FIND", "RFIND", "CHANGE", "RCHANGE", "EXCLUDE", "SHOW", "RESET", "LOCATE", "SAVE",
        "CANCEL", "END", "UP", "DOWN", "TOP", "BOTTOM", "UNDO", "REDO", "HEX", "HELP", "KEYS",
        "POM", "SETTINGS", "FILES",
    ];
    const EXPECTED_LINES: &[&str] = &[
        "index", "D", "I", "R", "C", "M", "A", "X", "U", "shift", "COLS", "BNDS", "TABS", "MASK",
    ];
    const EXPECTED_MODES: &[&str] = &[
        "browse",
        "edit",
        "view",
        "hex",
        "preview",
        "grid_browse",
        "grid_edit",
    ];
    const EXPECTED_FEATURES: &[&str] = &[
        "undo",
        "macros",
        "command_history",
        "tabs",
        "docking",
        "configuration",
        "function_keys",
    ];
    if key == "index" || key == "getting_started" {
        return true;
    }
    if let Some((ns, id)) = key.split_once(':') {
        return match ns {
            "cmd" => EXPECTED_CMDS.contains(&id),
            "line" => EXPECTED_LINES.contains(&id),
            "mode" => EXPECTED_MODES.contains(&id),
            "feature" => EXPECTED_FEATURES.contains(&id),
            _ => false,
        };
    }
    false
}
