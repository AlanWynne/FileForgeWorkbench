//! `CompletionEngine` — the central orchestrator for the completion subsystem.
//!
//! Manages the lifecycle: trigger → provide → filter → rank → navigate → accept/dismiss.

use std::sync::Arc;

use crate::config::{CompletionConfig, TriggerMode};
use crate::context::{CompletionContext, CompletionContextBuilder, CompletionField};
use crate::list::{CompletionItem, CompletionList};
use crate::navigation::SelectionState;
use crate::positioning::{
    compute_popup_position, FieldRect, PopupBounds, PopupConfig, ViewportRect,
};
use crate::provider::ProviderRegistry;

/// Actions the shell forwards to the `CompletionEngine` when the popup is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationAction {
    /// Move selection down by one item.
    Down,
    /// Move selection up by one item.
    Up,
    /// Move selection down by one page.
    PageDown,
    /// Move selection up by one page.
    PageUp,
    /// Accept the current selection (Tab behaviour).
    AcceptTab,
    /// Accept and execute (Enter behaviour).
    AcceptEnter,
    /// Dismiss without accepting.
    Dismiss,
}

/// The result of processing a navigation action or text change.
///
/// Tells the shell what happened so it can update the command field and popup.
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionAction {
    /// The popup state was updated (re-render needed).
    PopupUpdated,
    /// A candidate was accepted — the shell should perform this text insertion.
    Accept {
        /// Text to insert, replacing the prefix at [anchor_offset..cursor_offset].
        insert_text: String,
        /// Whether to append a trailing space after insertion.
        trailing_space: bool,
        /// Whether to submit the command (Enter with no further args expected).
        submit: bool,
    },
    /// The popup was dismissed (hide it).
    Dismissed,
    /// No change — the event was not consumed by completion.
    NotConsumed,
}

/// An active completion session — created when the popup is triggered.
#[derive(Debug, Clone)]
struct CompletionSession {
    /// The context at trigger time.
    context: CompletionContext,
    /// The filtered/ranked item list.
    list: CompletionList,
    /// Navigation state.
    selection: SelectionState,
    /// The anchor position for popup placement.
    anchor_offset: usize,
}

/// The central orchestrator for the completion subsystem.
///
/// Manages trigger logic, provider invocation, filtering, navigation,
/// and accept/dismiss lifecycle.
pub struct CompletionEngine {
    /// Configuration for completion behaviour.
    config: CompletionConfig,
    /// The provider registry.
    registry: Arc<ProviderRegistry>,
    /// The active session, if any.
    session: Option<CompletionSession>,
    /// Character counter for auto-trigger.
    typed_char_count: usize,
    /// Viewport dimensions for popup positioning.
    viewport: ViewportRect,
    /// Command field rectangle.
    field_rect: FieldRect,
}

impl CompletionEngine {
    /// Creates a new engine with the given configuration and provider registry.
    pub fn new(config: CompletionConfig, registry: Arc<ProviderRegistry>) -> Self {
        Self {
            config,
            registry,
            session: None,
            typed_char_count: 0,
            viewport: ViewportRect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
            field_rect: FieldRect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 30.0,
            },
        }
    }

    /// Returns true if the popup is currently visible (session is active).
    pub fn is_active(&self) -> bool {
        self.session.is_some()
    }

    /// Returns the current selection index, if a session is active.
    pub fn selected_index(&self) -> Option<usize> {
        self.session.as_ref().map(|s| s.selection.selected_index())
    }

    /// Returns the current filtered list items, if a session is active.
    pub fn items(&self) -> Option<&[CompletionItem]> {
        self.session.as_ref().map(|s| s.list.items())
    }

    /// Returns the number of filtered items in the current session.
    pub fn item_count(&self) -> usize {
        self.session.as_ref().map(|s| s.list.len()).unwrap_or(0)
    }

    /// Notify the engine that text changed in the active field.
    ///
    /// Evaluates whether to trigger, re-filter, or dismiss based on
    /// the current state and configuration.
    pub fn on_text_changed(
        &mut self,
        field: CompletionField,
        text: &str,
        cursor_offset: usize,
    ) -> CompletionAction {
        // If we have an active session, re-filter
        if let Some(ref mut session) = self.session {
            // Check dismiss conditions
            if self.config.cancel_at_start_pos && cursor_offset <= session.anchor_offset {
                self.session = None;
                return CompletionAction::Dismissed;
            }

            // Re-filter with new prefix
            let prefix = &text[session.anchor_offset..cursor_offset.min(text.len())];
            session.list.filter(prefix);
            session.context.prefix = prefix.to_string();
            session.context.cursor_offset = cursor_offset;

            // Auto-hide if empty
            if self.config.auto_hide && session.list.is_empty() {
                self.session = None;
                return CompletionAction::Dismissed;
            }

            // Reset selection
            session.selection.reset(session.list.len());

            // Choose-single: auto-accept lone match
            if self.config.choose_single && session.list.len() == 1 {
                let item = session.list.get(0).unwrap();
                let insert_text = item.candidate.insert_text.clone();
                self.session = None;
                return CompletionAction::Accept {
                    insert_text,
                    trailing_space: true,
                    submit: false,
                };
            }

            return CompletionAction::PopupUpdated;
        }

        // No active session — check if we should auto-trigger
        if self.should_auto_trigger(text, cursor_offset) {
            self.typed_char_count += 1;
            if self.typed_char_count >= self.config.auto_trigger_chars as usize {
                return self.trigger(field, text, cursor_offset);
            }
        } else {
            self.typed_char_count = 0;
        }

        CompletionAction::NotConsumed
    }

    /// Notify the engine that the user explicitly triggered completion (Ctrl+Space).
    pub fn on_manual_trigger(
        &mut self,
        field: CompletionField,
        text: &str,
        cursor_offset: usize,
    ) -> CompletionAction {
        // Manual trigger always works regardless of trigger mode
        self.trigger(field, text, cursor_offset)
    }

    /// Process a navigation action while the popup is visible.
    pub fn on_navigation(&mut self, action: NavigationAction) -> CompletionAction {
        let session = match self.session.as_mut() {
            Some(s) => s,
            None => return CompletionAction::NotConsumed,
        };

        match action {
            NavigationAction::Down => {
                session.selection.move_down();
                CompletionAction::PopupUpdated
            }
            NavigationAction::Up => {
                session.selection.move_up();
                CompletionAction::PopupUpdated
            }
            NavigationAction::PageDown => {
                session.selection.page_down();
                CompletionAction::PopupUpdated
            }
            NavigationAction::PageUp => {
                session.selection.page_up();
                CompletionAction::PopupUpdated
            }
            NavigationAction::AcceptTab => self.accept_selected(false),
            NavigationAction::AcceptEnter => self.accept_selected(true),
            NavigationAction::Dismiss => {
                self.session = None;
                CompletionAction::Dismissed
            }
        }
    }

    /// Notify the engine that focus left the command field.
    pub fn on_focus_lost(&mut self) -> CompletionAction {
        if self.session.is_some() {
            self.session = None;
            CompletionAction::Dismissed
        } else {
            CompletionAction::NotConsumed
        }
    }

    /// Notify the engine that the command was submitted.
    pub fn on_command_submit(&mut self) -> CompletionAction {
        if self.session.is_some() {
            self.session = None;
            CompletionAction::Dismissed
        } else {
            CompletionAction::NotConsumed
        }
    }

    /// Handles a stop character — dismisses the popup.
    pub fn on_stop_char(&mut self, _ch: char) -> CompletionAction {
        if self.session.is_some() {
            self.session = None;
            CompletionAction::Dismissed
        } else {
            CompletionAction::NotConsumed
        }
    }

    /// Handles a fill-up character — accepts selection and inserts the char.
    pub fn on_fill_up_char(&mut self, _ch: char) -> CompletionAction {
        if self.session.is_some() {
            self.accept_selected(false)
        } else {
            CompletionAction::NotConsumed
        }
    }

    /// Update configuration (called on hot-reload notification).
    pub fn update_config(&mut self, config: CompletionConfig) {
        self.config = config;
    }

    /// Set the viewport dimensions for popup positioning calculations.
    pub fn set_viewport(&mut self, viewport: ViewportRect) {
        self.viewport = viewport;
    }

    /// Set the command field rectangle for popup anchor computation.
    pub fn set_field_rect(&mut self, rect: FieldRect) {
        self.field_rect = rect;
    }

    /// Returns a reference to the current configuration.
    pub fn config(&self) -> &CompletionConfig {
        &self.config
    }

    /// Computes popup bounds for the current session.
    pub fn popup_bounds(&self) -> Option<PopupBounds> {
        let session = self.session.as_ref()?;
        let config = PopupConfig {
            max_items: self.config.popup_max_items as usize,
            max_width: self.config.popup_max_width as f32,
            item_height: 20.0,
        };
        let longest_label = session
            .list
            .items()
            .iter()
            .map(|item| item.candidate.label.len() as f32 * 8.0) // approximate char width
            .fold(0.0_f32, f32::max);

        Some(compute_popup_position(
            self.field_rect.x,
            &self.field_rect,
            session.list.len(),
            longest_label,
            &config,
            &self.viewport,
        ))
    }

    /// Triggers completion, gathering candidates from applicable providers.
    fn trigger(
        &mut self,
        field: CompletionField,
        text: &str,
        cursor_offset: usize,
    ) -> CompletionAction {
        // Build context
        let (prefix, anchor_offset, command_name, argument_index) =
            parse_field_context(field, text, cursor_offset);

        let context = CompletionContextBuilder::new()
            .field(field)
            .field_text(text)
            .cursor_offset(cursor_offset)
            .prefix(&prefix)
            .anchor_offset(anchor_offset)
            .build();

        let context = if let Some(cmd) = command_name {
            CompletionContext {
                command_name: Some(cmd),
                argument_index,
                ..context
            }
        } else {
            context
        };

        // Get candidates from providers
        let candidates = self.registry.provide_candidates(&context);

        if candidates.is_empty() {
            return CompletionAction::NotConsumed;
        }

        // Build list
        let mut list = CompletionList::new(
            candidates,
            self.config.matching_mode,
            self.config.case_sensitive,
        );
        list.filter(&prefix);

        if list.is_empty() && self.config.auto_hide {
            return CompletionAction::NotConsumed;
        }

        // Choose-single
        if self.config.choose_single && list.len() == 1 {
            let item = list.get(0).unwrap();
            let insert_text = item.candidate.insert_text.clone();
            return CompletionAction::Accept {
                insert_text,
                trailing_space: true,
                submit: false,
            };
        }

        // Create session
        let selection = SelectionState::new(
            list.len(),
            self.config.popup_max_items as usize,
            self.config.wrap_navigation,
        );

        self.session = Some(CompletionSession {
            context,
            list,
            selection,
            anchor_offset,
        });

        self.typed_char_count = 0;
        CompletionAction::PopupUpdated
    }

    /// Accepts the currently selected candidate.
    fn accept_selected(&mut self, submit: bool) -> CompletionAction {
        let session = match self.session.take() {
            Some(s) => s,
            None => return CompletionAction::NotConsumed,
        };

        let index = session.selection.selected_index();
        let item = match session.list.get(index) {
            Some(item) => item,
            None => return CompletionAction::Dismissed,
        };

        CompletionAction::Accept {
            insert_text: item.candidate.insert_text.clone(),
            trailing_space: true,
            submit,
        }
    }

    /// Determines if auto-triggering should be evaluated.
    fn should_auto_trigger(&self, text: &str, cursor_offset: usize) -> bool {
        match self.config.trigger_mode {
            TriggerMode::Manual => false,
            TriggerMode::Automatic | TriggerMode::Both => {
                cursor_offset > 0 && cursor_offset <= text.len()
            }
        }
    }
}

/// Parses the field text to determine prefix, anchor, and command context.
fn parse_field_context(
    field: CompletionField,
    text: &str,
    cursor_offset: usize,
) -> (String, usize, Option<String>, Option<usize>) {
    if field == CompletionField::PrefixArea {
        // In prefix area, the entire text is the prefix
        return (text[..cursor_offset].to_string(), 0, None, None);
    }

    // Primary command field: determine if we're in command or argument position
    let before_cursor = &text[..cursor_offset];
    let tokens: Vec<&str> = before_cursor.split_whitespace().collect();

    if tokens.is_empty() {
        return (String::new(), 0, None, None);
    }

    if tokens.len() == 1 && !before_cursor.ends_with(' ') {
        // Still typing the command name
        let prefix = tokens[0].to_string();
        return (prefix, 0, None, None);
    }

    // In argument position
    let command_name = tokens[0].to_uppercase();
    let arg_index = tokens.len() - 2; // 0-indexed argument

    // Find the anchor of the current token
    let last_space = before_cursor.rfind(' ').unwrap_or(0);
    let anchor = if before_cursor.ends_with(' ') {
        cursor_offset
    } else {
        last_space + 1
    };

    let prefix = text[anchor..cursor_offset].to_string();

    (prefix, anchor, Some(command_name), Some(arg_index))
}

// Ensure CompletionEngine is Send + Sync
fn _assert_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<CompletionEngine>();
    assert_sync::<CompletionEngine>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate::{CompletionCandidate, CompletionKind};
    use crate::provider::{CompletionProvider, ProviderRegistry};

    struct MockProvider {
        id: String,
        candidates: Vec<CompletionCandidate>,
    }

    impl CompletionProvider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }
        fn is_applicable(&self, _ctx: &CompletionContext) -> bool {
            true
        }
        fn provide_candidates(
            &self,
            _ctx: &CompletionContext,
        ) -> Result<Vec<CompletionCandidate>, crate::error::CompletionError> {
            Ok(self.candidates.clone())
        }
    }

    fn test_engine() -> CompletionEngine {
        let registry = Arc::new(ProviderRegistry::new());
        let _ = registry.register(Box::new(MockProvider {
            id: "test".to_string(),
            candidates: vec![
                CompletionCandidate::new("FIND", "FIND", CompletionKind::Command),
                CompletionCandidate::new("FILTER", "FILTER", CompletionKind::Command),
                CompletionCandidate::new("FILE.SAVE", "FILE.SAVE", CompletionKind::Command),
                CompletionCandidate::new("SAVE", "SAVE", CompletionKind::Command),
                CompletionCandidate::new("SORT", "SORT", CompletionKind::Command),
            ],
        }));
        CompletionEngine::new(CompletionConfig::default(), registry)
    }

    // Validates: Requirement 9.2 (manual trigger)
    #[test]
    fn manual_trigger_activates_popup() {
        let mut engine = test_engine();
        let action = engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
        assert_eq!(action, CompletionAction::PopupUpdated);
        assert!(engine.is_active());
    }

    // Validates: Requirement 1.6 (dynamic re-filter)
    #[test]
    fn text_change_refilters_active_session() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
        assert_eq!(engine.item_count(), 3); // FIND, FILTER, FILE.SAVE

        let action = engine.on_text_changed(CompletionField::PrimaryCommand, "FIN", 3);
        assert_eq!(action, CompletionAction::PopupUpdated);
        assert_eq!(engine.item_count(), 1); // FIND only
    }

    // Validates: Requirement 1.7, 5.4 (auto-hide on empty)
    #[test]
    fn auto_hide_dismisses_on_no_matches() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

        let action = engine.on_text_changed(CompletionField::PrimaryCommand, "FIZ", 3);
        assert_eq!(action, CompletionAction::Dismissed);
        assert!(!engine.is_active());
    }

    // Validates: Requirement 4.3 (Tab accepts)
    #[test]
    fn tab_accepts_highlighted_candidate() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

        let action = engine.on_navigation(NavigationAction::AcceptTab);
        match action {
            CompletionAction::Accept {
                insert_text,
                trailing_space,
                submit,
            } => {
                assert!(!insert_text.is_empty());
                assert!(trailing_space);
                assert!(!submit);
            }
            _ => panic!("Expected Accept action"),
        }
        assert!(!engine.is_active());
    }

    // Validates: Requirement 4.4 (Enter accepts and submits)
    #[test]
    fn enter_accepts_and_submits() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

        let action = engine.on_navigation(NavigationAction::AcceptEnter);
        match action {
            CompletionAction::Accept { submit, .. } => {
                assert!(submit);
            }
            _ => panic!("Expected Accept action"),
        }
    }

    // Validates: Requirement 4.5 (Escape dismisses)
    #[test]
    fn escape_dismisses_popup() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

        let action = engine.on_navigation(NavigationAction::Dismiss);
        assert_eq!(action, CompletionAction::Dismissed);
        assert!(!engine.is_active());
    }

    // Validates: Requirement 5.2 (focus loss dismisses)
    #[test]
    fn focus_loss_dismisses_popup() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

        let action = engine.on_focus_lost();
        assert_eq!(action, CompletionAction::Dismissed);
        assert!(!engine.is_active());
    }

    // Validates: Requirement 5.5 (command submit dismisses)
    #[test]
    fn command_submit_dismisses_popup() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

        let action = engine.on_command_submit();
        assert_eq!(action, CompletionAction::Dismissed);
    }

    // Validates: Requirement 5.3 (cursor retreat past anchor)
    #[test]
    fn cursor_retreat_past_anchor_dismisses() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

        // Cursor retreats to position 0 (before anchor)
        let action = engine.on_text_changed(CompletionField::PrimaryCommand, "", 0);
        assert_eq!(action, CompletionAction::Dismissed);
    }

    // Validates: Requirement 4.1 (navigation down)
    #[test]
    fn navigation_down_advances_selection() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
        assert_eq!(engine.selected_index(), Some(0));

        engine.on_navigation(NavigationAction::Down);
        assert_eq!(engine.selected_index(), Some(1));
    }

    // Validates: Requirement 4.2 (navigation up)
    #[test]
    fn navigation_up_retreats_selection() {
        let mut engine = test_engine();
        engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
        engine.on_navigation(NavigationAction::Down);
        engine.on_navigation(NavigationAction::Down);
        assert_eq!(engine.selected_index(), Some(2));

        engine.on_navigation(NavigationAction::Up);
        assert_eq!(engine.selected_index(), Some(1));
    }

    // Validates: Requirement 4.9 (choose_single auto-accept)
    #[test]
    fn choose_single_auto_accepts_lone_match() {
        let registry = Arc::new(ProviderRegistry::new());
        let _ = registry.register(Box::new(MockProvider {
            id: "test".to_string(),
            candidates: vec![CompletionCandidate::new(
                "UNIQUE_CMD",
                "UNIQUE_CMD",
                CompletionKind::Command,
            )],
        }));
        let config = CompletionConfig {
            choose_single: true,
            ..Default::default()
        };
        let mut engine = CompletionEngine::new(config, registry);

        let action = engine.on_manual_trigger(CompletionField::PrimaryCommand, "UNI", 3);
        match action {
            CompletionAction::Accept { insert_text, .. } => {
                assert_eq!(insert_text, "UNIQUE_CMD");
            }
            _ => panic!("Expected Accept action for choose_single"),
        }
        assert!(!engine.is_active());
    }

    // Validates: Requirement 10.5 (provider error isolation)
    #[test]
    fn provider_error_does_not_crash_engine() {
        struct FailingProvider;
        impl CompletionProvider for FailingProvider {
            fn id(&self) -> &str {
                "failing"
            }
            fn is_applicable(&self, _: &CompletionContext) -> bool {
                true
            }
            fn provide_candidates(
                &self,
                _: &CompletionContext,
            ) -> Result<Vec<CompletionCandidate>, crate::error::CompletionError> {
                Err(crate::error::CompletionError::Internal("boom".to_string()))
            }
        }

        let registry = Arc::new(ProviderRegistry::new());
        let _ = registry.register(Box::new(FailingProvider));
        let _ = registry.register(Box::new(MockProvider {
            id: "good".to_string(),
            candidates: vec![CompletionCandidate::new(
                "GOOD",
                "GOOD",
                CompletionKind::Command,
            )],
        }));

        let mut engine = CompletionEngine::new(CompletionConfig::default(), registry);
        let action = engine.on_manual_trigger(CompletionField::PrimaryCommand, "G", 1);
        assert_eq!(action, CompletionAction::PopupUpdated);
        assert_eq!(engine.item_count(), 1);
    }

    // Validates: Requirement 9.6 (config hot-reload)
    #[test]
    fn update_config_applies_new_settings() {
        let mut engine = test_engine();
        assert!(engine.config().wrap_navigation);

        let new_config = CompletionConfig {
            wrap_navigation: false,
            ..Default::default()
        };
        engine.update_config(new_config);
        assert!(!engine.config().wrap_navigation);
    }

    #[test]
    fn not_consumed_when_no_session_and_no_trigger() {
        let config = CompletionConfig {
            trigger_mode: TriggerMode::Manual,
            ..Default::default()
        };
        let registry = Arc::new(ProviderRegistry::new());
        let mut engine = CompletionEngine::new(config, registry);

        let action = engine.on_text_changed(CompletionField::PrimaryCommand, "F", 1);
        assert_eq!(action, CompletionAction::NotConsumed);
    }
}
