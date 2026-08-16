//! Integration tests for ff-completion.
//!
//! These tests exercise the full completion lifecycle: trigger → filter → navigate → accept.

use std::sync::Arc;

use ff_completion::candidate::{CompletionCandidate, CompletionKind};
use ff_completion::config::CompletionConfig;
use ff_completion::context::{CompletionContext, CompletionField};
use ff_completion::engine::{CompletionAction, CompletionEngine, NavigationAction};
use ff_completion::error::CompletionError;
use ff_completion::provider::{create_default_registry, CompletionProvider, ProviderRegistry};

// ─── Test helpers ───────────────────────────────────────────────────────────

struct StaticProvider {
    id: String,
    applicable_field: CompletionField,
    needs_command: bool,
    candidates: Vec<CompletionCandidate>,
}

impl CompletionProvider for StaticProvider {
    fn id(&self) -> &str {
        &self.id
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        if context.field != self.applicable_field {
            return false;
        }
        if self.needs_command {
            context.command_name.is_some()
        } else {
            context.command_name.is_none()
        }
    }

    fn provide_candidates(
        &self,
        _context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        Ok(self.candidates.clone())
    }
}

fn command_candidates() -> Vec<CompletionCandidate> {
    vec![
        CompletionCandidate::new("FIND", "FIND", CompletionKind::Command)
            .with_detail("search")
            .with_description("Find text in document"),
        CompletionCandidate::new("FILTER", "FILTER", CompletionKind::Command)
            .with_detail("view")
            .with_description("Filter displayed lines"),
        CompletionCandidate::new("FILE.SAVE", "FILE.SAVE", CompletionKind::Command)
            .with_detail("file")
            .with_description("Save current document"),
        CompletionCandidate::new("SAVE", "SAVE", CompletionKind::Command)
            .with_detail("file")
            .with_description("Save current document"),
        CompletionCandidate::new("SORT", "SORT", CompletionKind::Command)
            .with_detail("edit")
            .with_description("Sort lines"),
        CompletionCandidate::new("SUBMIT", "SUBMIT", CompletionKind::Command)
            .with_detail("file")
            .with_description("Submit changes"),
    ]
}

fn keyword_candidates() -> Vec<CompletionCandidate> {
    vec![
        CompletionCandidate::new("CHARS", "CHARS", CompletionKind::Keyword),
        CompletionCandidate::new("PREFIX", "PREFIX", CompletionKind::Keyword),
        CompletionCandidate::new("SUFFIX", "SUFFIX", CompletionKind::Keyword),
        CompletionCandidate::new("WORD", "WORD", CompletionKind::Keyword),
    ]
}

fn make_engine_with_providers() -> CompletionEngine {
    let registry = Arc::new(ProviderRegistry::new());
    let _ = registry.register(Box::new(StaticProvider {
        id: "commands".to_string(),
        applicable_field: CompletionField::PrimaryCommand,
        needs_command: false,
        candidates: command_candidates(),
    }));
    let _ = registry.register(Box::new(StaticProvider {
        id: "keywords".to_string(),
        applicable_field: CompletionField::PrimaryCommand,
        needs_command: true,
        candidates: keyword_candidates(),
    }));
    CompletionEngine::new(CompletionConfig::default(), registry)
}

// ─── Integration Test: Full Command Name Completion Flow ────────────────────

// Validates: Requirements 1.1, 1.2, 1.5, 1.6
#[test]
fn full_command_name_completion_flow() {
    let mut engine = make_engine_with_providers();

    // 1. Trigger completion
    let action = engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
    assert_eq!(action, CompletionAction::PopupUpdated);
    assert!(engine.is_active());

    // 2. Verify filtered results
    let items = engine.items().unwrap();
    assert_eq!(items.len(), 3); // FIND, FILTER, FILE.SAVE
    let labels: Vec<&str> = items.iter().map(|i| i.candidate.label.as_str()).collect();
    assert!(labels.contains(&"FIND"));
    assert!(labels.contains(&"FILTER"));
    assert!(labels.contains(&"FILE.SAVE"));

    // 3. Type more to narrow
    let action = engine.on_text_changed(CompletionField::PrimaryCommand, "FIN", 3);
    assert_eq!(action, CompletionAction::PopupUpdated);
    let items = engine.items().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].candidate.label, "FIND");

    // 4. Accept the candidate
    let action = engine.on_navigation(NavigationAction::AcceptTab);
    match action {
        CompletionAction::Accept {
            insert_text,
            trailing_space,
            submit,
        } => {
            assert_eq!(insert_text, "FIND");
            assert!(trailing_space);
            assert!(!submit);
        }
        _ => panic!("Expected Accept"),
    }
    assert!(!engine.is_active());
}

// ─── Integration Test: Argument Completion with Keyword Provider ────────────

// Validates: Requirements 2.1, 2.4
#[test]
fn argument_completion_with_keyword_provider() {
    let mut engine = make_engine_with_providers();

    // Trigger in argument position (after "FIND ")
    let action = engine.on_manual_trigger(CompletionField::PrimaryCommand, "FIND CH", 7);
    assert_eq!(action, CompletionAction::PopupUpdated);

    let items = engine.items().unwrap();
    // Should have keyword candidates that match "CH"
    let labels: Vec<&str> = items.iter().map(|i| i.candidate.label.as_str()).collect();
    assert!(labels.contains(&"CHARS"));
}

// ─── Integration Test: Line Command Completion ──────────────────────────────

// Validates: Requirements 7.1, 7.2, 7.3
#[test]
fn line_command_completion_in_prefix_area() {
    let registry = create_default_registry();
    let mut engine = CompletionEngine::new(CompletionConfig::default(), registry);

    // Trigger in prefix area
    let action = engine.on_manual_trigger(CompletionField::PrefixArea, "C", 1);
    assert_eq!(action, CompletionAction::PopupUpdated);

    let items = engine.items().unwrap();
    // Should have line command candidates starting with C
    let labels: Vec<&str> = items.iter().map(|i| i.candidate.label.as_str()).collect();
    assert!(labels.contains(&"C"));
    assert!(labels.contains(&"CC"));

    // D should not be in the list (filtered by prefix "C")
    assert!(!labels.contains(&"D"));
}

// ─── Integration Test: Dynamic Re-filter ────────────────────────────────────

// Validates: Requirement 1.6
#[test]
fn dynamic_refilter_as_user_types() {
    let mut engine = make_engine_with_providers();

    engine.on_manual_trigger(CompletionField::PrimaryCommand, "S", 1);
    let count1 = engine.item_count();
    assert!(count1 >= 2); // SAVE, SORT, SUBMIT

    engine.on_text_changed(CompletionField::PrimaryCommand, "SA", 2);
    let count2 = engine.item_count();
    assert!(count2 <= count1); // narrowed

    engine.on_text_changed(CompletionField::PrimaryCommand, "SAV", 3);
    let count3 = engine.item_count();
    assert!(count3 <= count2);
    assert_eq!(count3, 1); // only SAVE
}

// ─── Integration Test: Dismiss Behaviours ───────────────────────────────────

// Validates: Requirements 5.1, 5.2, 5.4
#[test]
fn dismiss_behaviours() {
    let mut engine = make_engine_with_providers();

    // Escape dismisses
    engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
    assert!(engine.is_active());
    let action = engine.on_navigation(NavigationAction::Dismiss);
    assert_eq!(action, CompletionAction::Dismissed);
    assert!(!engine.is_active());

    // Focus loss dismisses
    engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
    assert!(engine.is_active());
    let action = engine.on_focus_lost();
    assert_eq!(action, CompletionAction::Dismissed);
    assert!(!engine.is_active());

    // Empty matches auto-hide
    engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
    assert!(engine.is_active());
    let action = engine.on_text_changed(CompletionField::PrimaryCommand, "FIZZ", 4);
    assert_eq!(action, CompletionAction::Dismissed);
    assert!(!engine.is_active());
}

// ─── Integration Test: Plugin Provider Registration and Merging ─────────────

// Validates: Requirements 10.2, 10.6
#[test]
fn plugin_provider_registration_and_merging() {
    let registry = Arc::new(ProviderRegistry::new());

    // Register two providers that both apply
    let _ = registry.register(Box::new(StaticProvider {
        id: "builtin".to_string(),
        applicable_field: CompletionField::PrimaryCommand,
        needs_command: false,
        candidates: vec![CompletionCandidate::new(
            "FIND",
            "FIND",
            CompletionKind::Command,
        )],
    }));
    let _ = registry.register(Box::new(StaticProvider {
        id: "plugin".to_string(),
        applicable_field: CompletionField::PrimaryCommand,
        needs_command: false,
        candidates: vec![CompletionCandidate::new(
            "FINDALL",
            "FINDALL",
            CompletionKind::Plugin,
        )],
    }));

    let mut engine = CompletionEngine::new(CompletionConfig::default(), registry);
    engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);

    let items = engine.items().unwrap();
    let labels: Vec<&str> = items.iter().map(|i| i.candidate.label.as_str()).collect();
    // Both providers' candidates are merged
    assert!(labels.contains(&"FIND"));
    assert!(labels.contains(&"FINDALL"));
}

// ─── Integration Test: Config Hot-Reload ────────────────────────────────────

// Validates: Requirement 9.6
#[test]
fn config_hot_reload_updates_behaviour() {
    let mut engine = make_engine_with_providers();

    // Initially wrap is true
    engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
    assert!(engine.config().wrap_navigation);

    // Simulate hot-reload
    let new_config = CompletionConfig {
        wrap_navigation: false,
        ..Default::default()
    };
    engine.update_config(new_config);
    assert!(!engine.config().wrap_navigation);
}

// ─── Integration Test: Thread Safety ────────────────────────────────────────

// Validates: Design Principle 4 (Non-blocking)
#[test]
fn engine_is_send_and_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<CompletionEngine>();
    assert_sync::<CompletionEngine>();
    assert_send::<ProviderRegistry>();
    assert_sync::<ProviderRegistry>();
}
