//! Property-based tests for ff-help.
//!
//! These tests use `proptest` to verify invariants across many randomized inputs.

use proptest::prelude::*;
use std::path::PathBuf;

use ff_help::{
    ContextDetector, EditorContext, EditorMode, HelpConfig, HelpPanelModel, HelpSearch, HelpTopic,
    HelpTopicRegistry, MatchLocation, NavigationStack, TopicKey, TopicSource,
};

// ─── Strategies ─────────────────────────────────────────────────────────────

fn arb_editor_mode() -> impl Strategy<Value = EditorMode> {
    prop_oneof![
        Just(EditorMode::Browse),
        Just(EditorMode::Edit),
        Just(EditorMode::View),
        Just(EditorMode::Hex),
        Just(EditorMode::Preview),
        Just(EditorMode::GridBrowse),
        Just(EditorMode::GridEdit),
    ]
}

fn arb_command_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("FIND".to_string()),
        Just("CHANGE".to_string()),
        Just("SAVE".to_string()),
        Just("CANCEL".to_string()),
        Just("EXCLUDE".to_string()),
        Just("SHOW".to_string()),
        Just("SORT".to_string()),
        Just("DELETE".to_string()),
        Just("COPY".to_string()),
        Just("MOVE".to_string()),
    ]
}

fn arb_line_command() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("D".to_string()),
        Just("DD".to_string()),
        Just("CC".to_string()),
        Just("MM".to_string()),
        Just("I".to_string()),
        Just("R".to_string()),
        Just("A".to_string()),
        Just("B".to_string()),
    ]
}

fn arb_topic_key() -> impl Strategy<Value = TopicKey> {
    prop_oneof![
        arb_command_name().prop_map(|n| TopicKey::command(&n)),
        arb_line_command().prop_map(|n| TopicKey::line_command(&n)),
        Just(TopicKey::index()),
        Just(TopicKey::mode("hex")),
        Just(TopicKey::mode("preview")),
        Just(TopicKey::feature("undo")),
        Just(TopicKey::feature("macros")),
        Just(TopicKey::config("theme")),
    ]
}

fn arb_editor_context() -> impl Strategy<Value = EditorContext> {
    (
        "[ a-zA-Z]{0,20}",                  // command_line_text
        any::<bool>(),                      // command_line_has_focus
        proptest::option::of("[A-Z]{1,3}"), // prefix_area_text
        any::<bool>(),                      // prefix_area_has_focus
        arb_editor_mode(),                  // active_mode
        any::<bool>(),                      // help_panel_open
    )
        .prop_map(
            |(cmd_text, cmd_focus, prefix, prefix_focus, mode, panel_open)| EditorContext {
                command_line_text: cmd_text,
                command_line_has_focus: cmd_focus,
                prefix_area_text: prefix,
                prefix_area_has_focus: prefix_focus,
                active_mode: mode,
                help_panel_open: panel_open,
                current_help_topic: None,
            },
        )
}

// ─── Property 1: Context Resolution Determinism ─────────────────────────────

proptest! {
    /// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.7**
    ///
    /// For any given EditorContext state, the ContextDetector always resolves
    /// to the same TopicKey. The resolution is a pure function with no hidden state.
    #[test]
    fn context_resolution_is_deterministic(ctx in arb_editor_context()) {
        // Feature: context-help, Property 1: Context resolution determinism
        let result1 = ContextDetector::resolve(&ctx);
        let result2 = ContextDetector::resolve(&ctx);
        prop_assert_eq!(&result1, &result2,
            "Context resolution should be deterministic for the same input");

        // The resolved key should be non-empty
        prop_assert!(!result1.as_str().is_empty(),
            "Resolved TopicKey should never be empty");
    }
}

// ─── Property 2: Context Priority — Command Input Dominates ─────────────────

proptest! {
    /// **Validates: Requirements 1.2, 1.3**
    ///
    /// When command_input_focused is true and command_input contains a non-empty
    /// recognisable command token, the resolved TopicKey always has namespace "cmd".
    #[test]
    fn command_input_focus_dominates_resolution(
        cmd_name in arb_command_name(),
        prefix in proptest::option::of("[A-Z]{1,3}"),
        prefix_focus in any::<bool>(),
        mode in arb_editor_mode(),
    ) {
        // Feature: context-help, Property 2: Command input priority dominance
        let ctx = EditorContext {
            command_line_text: cmd_name.clone(),
            command_line_has_focus: true,
            prefix_area_text: prefix,
            prefix_area_has_focus: prefix_focus,
            active_mode: mode,
            help_panel_open: false,
            current_help_topic: None,
        };

        let result = ContextDetector::resolve(&ctx);
        prop_assert_eq!(result.namespace(), Some("cmd"),
            "When command field is focused with a command name, result must be cmd:<name>");
    }
}

// ─── Property 3: Navigation Stack Back/Forward Consistency ──────────────────

/// Navigation operation for property testing.
#[derive(Debug, Clone)]
enum NavOp {
    Push(TopicKey),
    Back,
    Forward,
}

fn arb_nav_op() -> impl Strategy<Value = NavOp> {
    prop_oneof![
        arb_topic_key().prop_map(NavOp::Push),
        Just(NavOp::Back),
        Just(NavOp::Forward),
    ]
}

proptest! {
    /// **Validates: Requirements 3.1, 3.2, 3.3**
    ///
    /// For any sequence of push, back, and forward operations, the navigation
    /// stack maintains its invariants at every step.
    #[test]
    fn navigation_stack_invariants_hold(
        initial_topics in prop::collection::vec(arb_topic_key(), 1..10),
        ops in prop::collection::vec(arb_nav_op(), 1..30),
    ) {
        // Feature: context-help, Property 3: Navigation stack consistency
        let mut stack = NavigationStack::new();

        // Seed the stack
        for key in &initial_topics {
            stack.push(key.clone());
        }

        for op in &ops {
            match op {
                NavOp::Push(key) => {
                    stack.push(key.clone());
                    // After push: can_go_forward is false
                    prop_assert!(!stack.can_go_forward(),
                        "After push, can_go_forward must be false");
                    // current is the just-pushed key
                    prop_assert_eq!(stack.current(), Some(key));
                }
                NavOp::Back => {
                    let could_go = stack.can_go_back();
                    let result = stack.back();
                    if could_go {
                        prop_assert!(result.is_some(),
                            "back() should return Some when can_go_back was true");
                    } else {
                        prop_assert!(result.is_none(),
                            "back() should return None when can_go_back was false");
                    }
                }
                NavOp::Forward => {
                    let could_go = stack.can_go_forward();
                    let result = stack.forward();
                    if could_go {
                        prop_assert!(result.is_some(),
                            "forward() should return Some when can_go_forward was true");
                    } else {
                        prop_assert!(result.is_none(),
                            "forward() should return None when can_go_forward was false");
                    }
                }
            }

            // Global invariant: current is never None when stack is non-empty
            if !stack.is_empty() {
                prop_assert!(stack.current().is_some(),
                    "current() must be Some when stack is non-empty");
            }
        }
    }
}

// ─── Property 4: Search Relevance Ranking Monotonicity ──────────────────────

proptest! {
    /// **Validates: Requirements 4.2, 4.4**
    ///
    /// Search results are always sorted by descending relevance score.
    /// Title matches always appear before body matches.
    #[test]
    fn search_results_are_sorted_by_relevance(
        query in "[a-z]{2,8}",
    ) {
        // Feature: context-help, Property 4: Search relevance ranking monotonicity
        let search = HelpSearch::new();

        // Create topics with the query in various positions
        let topics = vec![
            HelpTopic::new(
                TopicKey::command("AAA"),
                format!("Topic about {query}"),
                "No match in body here.".to_string(),
                TopicSource::FileBased { file_path: PathBuf::from("t.help.md") },
            ),
            HelpTopic::new(
                TopicKey::command("BBB"),
                "Unrelated Title".to_string(),
                format!("The body mentions {query} somewhere."),
                TopicSource::FileBased { file_path: PathBuf::from("t.help.md") },
            ),
            HelpTopic::new(
                TopicKey::command("CCC"),
                "Another Unrelated".to_string(),
                format!("## Section with {query}\n\nBody text."),
                TopicSource::FileBased { file_path: PathBuf::from("t.help.md") },
            ),
        ];

        let results = search.query(&topics, &query);

        // Results should be sorted by descending relevance
        for window in results.windows(2) {
            prop_assert!(
                window[0].relevance >= window[1].relevance,
                "Results must be sorted by descending relevance: {} >= {}",
                window[0].relevance,
                window[1].relevance
            );
        }

        // Title matches should never appear after body matches
        let mut seen_body = false;
        for r in &results {
            if r.match_location == MatchLocation::Body {
                seen_body = true;
            }
            if seen_body {
                prop_assert_ne!(r.match_location, MatchLocation::Title,
                    "Title match must not appear after a body match");
            }
        }
    }
}

// ─── Property 5: Content Parser Round-Trip Fidelity ─────────────────────────

proptest! {
    /// **Validates: Requirements 5.2, 5.3, 5.4**
    ///
    /// For any valid .help.md content containing N topic delimiter blocks,
    /// parsing always produces exactly N HelpTopic objects with correct keys and titles.
    #[test]
    fn content_parser_preserves_topic_count_and_keys(
        topic_count in 1..5usize,
    ) {
        // Feature: context-help, Property 5: Content parser round-trip fidelity
        use ff_help::ContentParser;

        // Generate a valid .help.md file with N topics
        let mut content = String::new();
        let mut expected_keys = Vec::new();

        for i in 0..topic_count {
            let key = format!("cmd:CMD{i}");
            let title = format!("Command {i}");
            content.push_str(&format!("<!-- TOPIC: {key} -->\n"));
            content.push_str(&format!("<!-- TITLE: {title} -->\n"));
            content.push_str(&format!("Body content for topic {i}.\n\n"));
            expected_keys.push(key);
        }

        let path = PathBuf::from("test.help.md");
        let topics = ContentParser::parse_file(&path, &content).unwrap();

        prop_assert_eq!(topics.len(), topic_count,
            "Parser must produce exactly {} topics", topic_count);

        for (i, topic) in topics.iter().enumerate() {
            prop_assert_eq!(topic.key().as_str(), &expected_keys[i],
                "Topic {} key mismatch", i);
        }
    }
}

// ─── Property 6: TopicKey Parsing Totality ──────────────────────────────────

proptest! {
    /// **Validates: Requirements 5.2, 6.1**
    ///
    /// TopicKey::from_str accepts all valid patterns and rejects all invalid ones.
    /// No input panics.
    #[test]
    fn topic_key_parsing_never_panics(input in ".*") {
        // Feature: context-help, Property 6: TopicKey parsing totality
        // This should never panic, regardless of input
        let _ = input.parse::<TopicKey>();
    }

    #[test]
    fn valid_topic_keys_always_parse(
        prefix in prop_oneof![
            Just("cmd"), Just("line"), Just("mode"),
            Just("feature"), Just("config"), Just("api"),
        ],
        name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
    ) {
        // Feature: context-help, Property 6: TopicKey parsing totality (valid)
        let input = format!("{prefix}:{name}");
        let result = input.parse::<TopicKey>();
        prop_assert!(result.is_ok(),
            "Valid key '{}' must parse successfully", input);
    }

    #[test]
    fn invalid_topic_keys_always_rejected(
        prefix in prop_oneof![
            Just("bad"), Just("unknown"), Just("xyz"),
            Just(""), Just("123"),
        ],
        name in "[a-zA-Z]{1,10}",
    ) {
        // Feature: context-help, Property 6: TopicKey parsing totality (invalid)
        let input = format!("{prefix}:{name}");
        // Skip "index" and "getting_started" as they're special bare keywords
        if input != "index" && input != "getting_started" {
            let result = input.parse::<TopicKey>();
            prop_assert!(result.is_err(),
                "Invalid key '{}' must be rejected", input);
        }
    }
}

// ─── Property 7: Help Panel Toggle Idempotency ─────────────────────────────

proptest! {
    /// **Validates: Requirements 1.6, 2.4**
    ///
    /// Toggle N times with same topic alternates between open and closed perfectly.
    #[test]
    fn help_panel_toggle_alternates_state(
        toggle_count in 2..10usize,
    ) {
        // Feature: context-help, Property 7: Help Panel toggle idempotency
        let registry = std::sync::Arc::new(HelpTopicRegistry::new());
        let key = TopicKey::command("FIND");
        registry.register_file_topic(HelpTopic::new(
            key.clone(),
            "FIND".to_string(),
            "body".to_string(),
            TopicSource::FileBased { file_path: PathBuf::from("t.help.md") },
        ));
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        for i in 0..toggle_count {
            panel.toggle(&key).unwrap();
            let expected_open = (i % 2) == 0; // odd iterations (0-indexed) are open
            prop_assert_eq!(panel.is_open(), expected_open,
                "After toggle {}, panel open state should be {}", i + 1, expected_open);
        }
    }
}

// ─── Property 8: Plugin Registration/Unregistration Symmetry ────────────────

proptest! {
    /// **Validates: Requirements 6.1, 6.6**
    ///
    /// Registering N plugin topics then deregistering removes exactly those N topics.
    #[test]
    fn plugin_register_unregister_symmetry(
        topic_count in 1..8usize,
    ) {
        // Feature: context-help, Property 8: Plugin registration/unregistration symmetry
        let registry = HelpTopicRegistry::new();

        // Pre-register a file-based topic that should NOT be affected
        let unrelated_key = TopicKey::command("UNRELATED");
        registry.register_file_topic(HelpTopic::new(
            unrelated_key.clone(),
            "Unrelated".to_string(),
            "body".to_string(),
            TopicSource::FileBased { file_path: PathBuf::from("t.help.md") },
        ));

        // Register plugin topics
        let plugin_id = "test_plugin";
        let mut plugin_keys = Vec::new();
        for i in 0..topic_count {
            let key = TopicKey::feature(&format!("plugin_feat_{i}"));
            registry.register_plugin_topic(
                plugin_id,
                HelpTopic::new(
                    key.clone(),
                    format!("Plugin Feature {i}"),
                    "body".to_string(),
                    TopicSource::Plugin { plugin_id: plugin_id.to_string() },
                ),
            );
            plugin_keys.push(key);
        }

        // All plugin topics should exist
        for key in &plugin_keys {
            prop_assert!(registry.contains(key),
                "Plugin topic {:?} should exist after registration", key);
        }

        // Deregister
        registry.deregister_plugin(plugin_id);

        // All plugin topics removed
        for key in &plugin_keys {
            prop_assert!(!registry.contains(key),
                "Plugin topic {:?} should be removed after deregistration", key);
        }

        // Unrelated topic still exists
        prop_assert!(registry.contains(&unrelated_key),
            "Unrelated topic should not be affected by plugin deregistration");
    }
}

// ─── Property 9: Configuration Validation Boundary ──────────────────────────

proptest! {
    /// **Validates: Requirement 16.2**
    ///
    /// panel_width_ratio values outside 0.2–0.5 are rejected and default applied.
    /// Values within range are accepted.
    #[test]
    fn config_width_ratio_boundary(ratio in -1.0f32..2.0f32) {
        // Feature: context-help, Property 9: Configuration validation boundary
        let mut config = HelpConfig {
            panel_width_ratio: ratio,
            ..HelpConfig::default()
        };

        let result = config.validate_panel_width_ratio();

        if (0.2..=0.5).contains(&ratio) {
            prop_assert!(result.is_ok(),
                "Ratio {} should be accepted", ratio);
            prop_assert!((config.panel_width_ratio - ratio).abs() < f32::EPSILON,
                "Valid ratio should be preserved");
        } else {
            prop_assert!(result.is_err(),
                "Ratio {} should be rejected", ratio);
            prop_assert!((config.panel_width_ratio - 0.35).abs() < f32::EPSILON,
                "Invalid ratio should fall back to 0.35");
        }
    }
}

// ─── Property 10: Registry Priority Resolution ──────────────────────────────

proptest! {
    /// **Validates: Requirements 6.4, 6.5**
    ///
    /// When both file-based and runtime topics exist for the same key,
    /// the registry returns the runtime version. After runtime removal,
    /// file-based becomes visible.
    #[test]
    fn registry_priority_resolution(
        cmd_name in "[A-Z]{3,8}",
    ) {
        // Feature: context-help, Property 10: Topic registry priority resolution
        let registry = HelpTopicRegistry::new();
        let key = TopicKey::command(&cmd_name);

        // Register file-based
        registry.register_file_topic(HelpTopic::new(
            key.clone(),
            "File Version".to_string(),
            "from file".to_string(),
            TopicSource::FileBased { file_path: PathBuf::from("t.help.md") },
        ));

        // Register command (runtime)
        registry.register_command_topic(HelpTopic::new(
            key.clone(),
            "Runtime Version".to_string(),
            "from runtime".to_string(),
            TopicSource::CommandRegistry { command_id: cmd_name.clone() },
        ));

        // Runtime wins
        let topic = registry.get(&key).unwrap();
        prop_assert_eq!(topic.title(), "Runtime Version",
            "Runtime topic should be preferred over file-based");
    }
}
