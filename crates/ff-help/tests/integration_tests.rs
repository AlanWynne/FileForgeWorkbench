//! Integration tests for ff-help — end-to-end help scenarios.

use std::path::PathBuf;
use std::sync::Arc;

use ff_help::{
    resolve_help_argument, ContentParser, ContextDetector, EditorContext, EditorMode, HelpAction,
    HelpConfig, HelpPanelModel, HelpPluginBridge, HelpSearch, HelpTopic, HelpTopicRegistry,
    TopicKey, TopicSource,
};

/// Helper to create a populated registry for integration tests.
fn test_registry() -> Arc<HelpTopicRegistry> {
    let registry = Arc::new(HelpTopicRegistry::new());

    // Command topics
    for cmd in &["FIND", "CHANGE", "SAVE", "CANCEL", "EXCLUDE"] {
        registry.register_file_topic(HelpTopic::new(
            TopicKey::command(cmd),
            format!("{cmd} Command"),
            format!("Help for the {cmd} command.\n\n## Syntax\n\n`{cmd} args`"),
            TopicSource::FileBased {
                file_path: PathBuf::from("commands.help.md"),
            },
        ));
    }

    // Line command topics
    for lc in &["D", "CC", "MM", "I"] {
        registry.register_file_topic(HelpTopic::new(
            TopicKey::line_command(lc),
            format!("{lc} Line Command"),
            format!("Help for the {lc} line command."),
            TopicSource::FileBased {
                file_path: PathBuf::from("lines.help.md"),
            },
        ));
    }

    // Mode topics
    registry.register_file_topic(HelpTopic::new(
        TopicKey::mode("hex"),
        "Hex Mode".to_string(),
        "Hexadecimal display mode.".to_string(),
        TopicSource::FileBased {
            file_path: PathBuf::from("modes.help.md"),
        },
    ));

    // Index topic
    registry.register_file_topic(HelpTopic::new(
        TopicKey::index(),
        "Help Index".to_string(),
        "Welcome to FileForgeWorkbench Help.".to_string(),
        TopicSource::FileBased {
            file_path: PathBuf::from("index.help.md"),
        },
    ));

    registry
}

// ─── Integration Test: F1 with command in command field ─────────────────────

// Validates: Requirement 1.2 — F1 with command resolves and displays correct topic
#[test]
fn f1_with_command_in_field_displays_command_topic() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    let ctx = EditorContext {
        command_line_text: "FIND 'text'".to_string(),
        command_line_has_focus: true,
        prefix_area_text: None,
        prefix_area_has_focus: false,
        active_mode: EditorMode::Edit,
        help_panel_open: false,
        current_help_topic: None,
    };

    let topic_key = ContextDetector::resolve(&ctx);
    assert_eq!(topic_key, TopicKey::command("FIND"));

    panel.show_topic(&topic_key).unwrap();
    assert!(panel.is_open());
    assert_eq!(panel.current_topic().unwrap().title(), "FIND Command");
}

// ─── Integration Test: F1 with empty command field opens Help Index ─────────

// Validates: Requirement 1.3 — F1 with empty command field opens Help Index
#[test]
fn f1_with_empty_field_opens_help_index() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    let ctx = EditorContext {
        command_line_text: String::new(),
        command_line_has_focus: true,
        prefix_area_text: None,
        prefix_area_has_focus: false,
        active_mode: EditorMode::Edit,
        help_panel_open: false,
        current_help_topic: None,
    };

    let topic_key = ContextDetector::resolve(&ctx);
    assert_eq!(topic_key, TopicKey::index());

    panel.show_topic(&topic_key).unwrap();
    assert!(panel.is_open());
    assert_eq!(panel.current_topic().unwrap().title(), "Help Index");
}

// ─── Integration Test: F1 with line command in prefix area ──────────────────

// Validates: Requirement 1.4 — F1 with line command displays line command help
#[test]
fn f1_with_line_command_displays_line_help() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    let ctx = EditorContext {
        command_line_text: String::new(),
        command_line_has_focus: false,
        prefix_area_text: Some("cc".to_string()),
        prefix_area_has_focus: true,
        active_mode: EditorMode::Edit,
        help_panel_open: false,
        current_help_topic: None,
    };

    let topic_key = ContextDetector::resolve(&ctx);
    assert_eq!(topic_key, TopicKey::line_command("CC"));

    panel.show_topic(&topic_key).unwrap();
    assert!(panel.is_open());
    assert_eq!(panel.current_topic().unwrap().title(), "CC Line Command");
}

// ─── Integration Test: HELP CHANGE opens Help Panel ─────────────────────────

// Validates: Requirement 13.2 — HELP CHANGE opens Panel with cmd:CHANGE
#[test]
fn help_command_with_argument_opens_topic() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    let action = resolve_help_argument("CHANGE");
    match action {
        HelpAction::ShowTopic(key) => {
            assert_eq!(key, TopicKey::command("CHANGE"));
            panel.show_topic(&key).unwrap();
            assert!(panel.is_open());
            assert_eq!(panel.current_topic().unwrap().title(), "CHANGE Command");
        }
        _ => panic!("Expected ShowTopic action"),
    }
}

// ─── Integration Test: HELP OFF closes panel ────────────────────────────────

// Validates: Requirement 13.8 — HELP OFF closes an open Help Panel
#[test]
fn help_off_closes_open_panel() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    // Open the panel first
    panel.show_topic(&TopicKey::index()).unwrap();
    assert!(panel.is_open());

    let action = resolve_help_argument("OFF");
    assert_eq!(action, HelpAction::Close);

    panel.close();
    assert!(!panel.is_open());
}

// ─── Integration Test: Navigation across multiple topics ────────────────────

// Validates: Requirement 3.1, 3.2 — Navigation back/forward across topics
#[test]
fn navigation_back_forward_across_topics() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    // Visit 3 topics
    panel.show_topic(&TopicKey::index()).unwrap();
    panel.show_topic(&TopicKey::command("FIND")).unwrap();
    panel.show_topic(&TopicKey::command("CHANGE")).unwrap();

    assert_eq!(panel.current_topic().unwrap().title(), "CHANGE Command");

    // Navigate back
    panel.navigate_back().unwrap();
    assert_eq!(panel.current_topic().unwrap().title(), "FIND Command");

    panel.navigate_back().unwrap();
    assert_eq!(panel.current_topic().unwrap().title(), "Help Index");

    // Navigate forward
    panel.navigate_forward().unwrap();
    assert_eq!(panel.current_topic().unwrap().title(), "FIND Command");
}

// ─── Integration Test: Search returns ranked results ────────────────────────

// Validates: Requirement 4.1, 4.4 — Search query returns ranked results
#[test]
fn search_returns_ranked_results() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    panel.show_topic(&TopicKey::index()).unwrap();
    panel.search("find");

    let results = panel.search_results();
    assert!(!results.is_empty());
    // The FIND command topic should rank highest (title match)
    assert_eq!(results[0].key, TopicKey::command("FIND"));
}

// ─── Integration Test: Plugin topic lifecycle ───────────────────────────────

// Validates: Requirement 6.1, 6.6 — Plugin registers/deregisters topics
#[test]
fn plugin_registers_and_deregisters_topics() {
    let registry = Arc::new(HelpTopicRegistry::new());
    let bridge = HelpPluginBridge::new(registry.clone());

    let key = TopicKey::feature("custom_plugin_feature");
    bridge.register_topic(
        "test_plugin",
        key.clone(),
        "Custom Feature".to_string(),
        "This is plugin-contributed help.".to_string(),
    );

    // Topic should be available
    assert!(registry.contains(&key));
    let topic = registry.get(&key).unwrap();
    assert_eq!(topic.title(), "Custom Feature");

    // After deregistration, topic is gone
    bridge.deregister_all("test_plugin");
    assert!(!registry.contains(&key));
}

// ─── Integration Test: Help Panel toggle ────────────────────────────────────

// Validates: Requirement 1.6 — Toggle: same topic closes, different navigates
#[test]
fn help_panel_toggle_behavior() {
    let registry = test_registry();
    let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

    let find_key = TopicKey::command("FIND");
    let change_key = TopicKey::command("CHANGE");

    // First toggle opens
    panel.toggle(&find_key).unwrap();
    assert!(panel.is_open());
    assert_eq!(panel.current_topic_key(), Some(&find_key));

    // Same topic toggle closes
    panel.toggle(&find_key).unwrap();
    assert!(!panel.is_open());

    // Toggle again opens
    panel.toggle(&find_key).unwrap();
    assert!(panel.is_open());

    // Different topic navigates (stays open)
    panel.toggle(&change_key).unwrap();
    assert!(panel.is_open());
    assert_eq!(panel.current_topic_key(), Some(&change_key));
}

// ─── Integration Test: Command metadata registration ────────────────────────

// Validates: Requirement 6.3 — Command registered with help_text creates topic
#[test]
fn command_with_help_text_creates_accessible_topic() {
    let registry = Arc::new(HelpTopicRegistry::new());

    // Simulate command registration with help_text
    registry.register_from_command_metadata(
        "SORT",
        "Sorts lines in the current file.",
        "SORT [ASC|DESC] [COL start end]",
    );

    // The topic should be accessible
    let key = TopicKey::command("SORT");
    assert!(registry.contains(&key));

    let topic = registry.get(&key).unwrap();
    assert!(topic.body().contains("Sorts lines"));
    assert!(topic.body().contains("SORT [ASC|DESC]"));
}

// ─── Integration Test: Content loading from temp directory ──────────────────

// Validates: Requirement 5.1, 5.4 — Content loading from .help.md files
#[test]
fn content_loading_from_help_files() {
    let content = "\
<!-- TOPIC: cmd:LOCATE -->
<!-- TITLE: LOCATE Command -->
## Syntax

`LOCATE label`

## Description

Navigates to a labelled line.
";

    let path = PathBuf::from("commands.help.md");
    let topics = ContentParser::parse_file(&path, content).unwrap();

    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].key(), &TopicKey::command("LOCATE"));
    assert_eq!(topics[0].title(), "LOCATE Command");
    assert!(topics[0].body().contains("LOCATE label"));
}

// ─── Integration Test: Search no results ────────────────────────────────────

// Validates: Requirement 4.5 — No results for unmatched query
#[test]
fn search_no_results_for_unmatched_query() {
    let search = HelpSearch::new();
    let topics = vec![HelpTopic::new(
        TopicKey::command("FIND"),
        "FIND Command".to_string(),
        "Searches for text.".to_string(),
        TopicSource::FileBased {
            file_path: PathBuf::from("t.help.md"),
        },
    )];

    let results = search.query(&topics, "zzz_nonexistent");
    assert!(results.is_empty());
}
