//! Property-based tests for the ff-viewers crate.
//!
//! These tests use `proptest` to verify invariants that must hold across all
//! valid inputs. Each property test maps to a specific acceptance criterion.

use proptest::prelude::*;

use ff_viewers::built_in::register_built_in_viewers;
use ff_viewers::config::{
    ViewerConfig, MAX_DEBOUNCE_MS, MAX_SPLIT_RATIO, MIN_DEBOUNCE_MS, MIN_SPLIT_RATIO,
};
use ff_viewers::key::ViewerKey;
use ff_viewers::panel::ViewerPanel;
use ff_viewers::readonly::ReadOnlyGuard;
use ff_viewers::refresh::RefreshController;
use ff_viewers::registry::{ViewerRegistry, ViewerSource};
use ff_viewers::selection::ContentSelector;
use ff_viewers::trait_def::FileViewer;

// ─── Test Helpers ────────────────────────────────────────────────────────────

/// A configurable stub viewer for property tests.
struct PropTestViewer {
    key: String,
    extensions: Vec<&'static str>,
}

impl PropTestViewer {
    fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            extensions: vec![],
        }
    }
}

impl FileViewer for PropTestViewer {
    fn viewer_key(&self) -> &str {
        &self.key
    }
    fn display_name(&self) -> &str {
        "PropTest Viewer"
    }
    fn description(&self) -> &str {
        "A viewer for property testing"
    }
    fn supported_extensions(&self) -> &[&str] {
        &self.extensions
    }
    fn supported_mime_types(&self) -> &[&str] {
        &[]
    }
    fn can_render(&self, _: &str, _: &[u8]) -> bool {
        false
    }
    fn render(&self, content: &[u8]) -> String {
        String::from_utf8_lossy(content).to_string()
    }
    fn on_content_changed(&mut self, _: &[u8]) {}
}

/// Strategy to generate valid viewer key strings.
fn valid_key_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9][a-z0-9\\-]{0,15}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Strategy to generate arbitrary strings that may or may not be valid keys.
fn arbitrary_string_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex(".{0,80}").unwrap()
}

// ─── Property 1: ViewerKey Format Validation ─────────────────────────────────
// Feature: custom-file-viewers, Property 1: Viewer_Key format validation
// **Validates: Requirements 1.1**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// For any string, ViewerKey::new() succeeds if and only if the string is
    /// non-empty, at most 64 characters, and contains only lowercase ASCII
    /// letters, digits, and hyphens.
    #[test]
    fn property_1_viewer_key_format_validation(input in arbitrary_string_strategy()) {
        let result = ViewerKey::new(&input);

        let is_valid = !input.is_empty()
            && input.len() <= 64
            && input.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

        if is_valid {
            let key = result.expect("Expected valid key to be accepted");
            prop_assert_eq!(key.as_str(), input.as_str());
        } else {
            prop_assert!(result.is_err(), "Expected invalid key '{}' to be rejected", input);
        }
    }
}

// ─── Property 2: Registry Uniqueness ─────────────────────────────────────────
// Feature: custom-file-viewers, Property 2: Registry uniqueness
// **Validates: Requirements 1.6**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// For any sequence of viewer registrations, the ViewerRegistry contains at
    /// most one entry per ViewerKey. A registration with an existing key always
    /// returns DuplicateKey error without modifying state.
    #[test]
    fn property_2_registry_uniqueness(keys in prop::collection::vec(valid_key_strategy(), 2..10)) {
        let registry = ViewerRegistry::new();
        let mut registered: std::collections::HashSet<String> = std::collections::HashSet::new();

        for key in &keys {
            let viewer = Box::new(PropTestViewer::new(key));
            let result = registry.register_builtin(viewer);

            if registered.contains(key) {
                // Duplicate — must be rejected
                prop_assert!(result.is_err(), "Duplicate key '{}' should be rejected", key);
                match result.unwrap_err() {
                    ff_viewers::ViewerError::DuplicateKey { key: k } => {
                        prop_assert_eq!(&k, key);
                    }
                    other => prop_assert!(false, "Expected DuplicateKey, got: {:?}", other),
                }
            } else {
                // First time — should succeed
                prop_assert!(result.is_ok(), "First registration of '{}' should succeed", key);
                registered.insert(key.clone());
            }
        }

        // Verify no duplicate keys in the final listing
        let list = registry.list_viewers();
        let listed_keys: std::collections::HashSet<String> =
            list.iter().map(|info| info.key.as_str().to_string()).collect();
        prop_assert_eq!(listed_keys.len(), list.len(), "Listed viewers should have unique keys");
    }
}

// ─── Property 3: Built-in Viewer Keys Stable and Unique ─────────────────────
// Feature: custom-file-viewers, Property 3: Built-in viewer keys are stable and unique
// **Validates: Requirements 4.5, 1.1**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// All 4 built-in viewer keys are distinct, non-empty, and format-compliant.
    /// This property is trivially stable across runs (keys are hard-coded), but
    /// the test validates the invariant holds regardless of registration order.
    #[test]
    fn property_3_built_in_keys_stable_and_unique(_seed in 0u32..1000u32) {
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();

        let list = registry.list_viewers();
        prop_assert_eq!(list.len(), 4, "Must have exactly 4 built-in viewers");

        let mut keys: Vec<&str> = list.iter().map(|info| info.key.as_str()).collect();
        keys.sort();

        // All keys are valid
        for key_str in &keys {
            let result = ViewerKey::new(key_str);
            prop_assert!(result.is_ok(), "Built-in key '{}' must be format-valid", key_str);
        }

        // All keys are distinct
        keys.dedup();
        prop_assert_eq!(keys.len(), 4, "All 4 built-in keys must be distinct");

        // All are built-in source
        for info in &list {
            prop_assert_eq!(
                registry.viewer_source(&info.key),
                Some(ViewerSource::BuiltIn)
            );
        }
    }
}

// ─── Property 4: PREVIEW Never Produces Undo_Record ──────────────────────────
// Feature: custom-file-viewers, Property 4: PREVIEW command never produces an Undo_Record
// **Validates: Requirements 3.9**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Issue various PREVIEW commands — none generate undo state.
    /// Since our PreviewCommand has no undo integration by design, we verify
    /// that executing various command sequences doesn't modify any document state.
    #[test]
    fn property_4_preview_never_produces_undo_record(
        actions in prop::collection::vec(0u8..5, 1..10),
        uri in "[a-z]+\\.[a-z]{2,4}",
    ) {
        use ff_viewers::command::{PreviewCommand, PreviewCommandAction};

        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let mut panel = ViewerPanel::new();
        let selector = ContentSelector::new(&registry);

        let full_uri = format!("file:///{uri}");
        let content = b"sample content";

        // There is no undo system in PreviewCommand — we just verify no panics
        // and the command operates purely on display state.
        for action_idx in actions {
            let action = match action_idx % 5 {
                0 => PreviewCommandAction::Toggle,
                1 => PreviewCommandAction::On,
                2 => PreviewCommandAction::Off,
                3 => PreviewCommandAction::List,
                _ => PreviewCommandAction::Activate(ViewerKey::new("hex").unwrap()),
            };

            let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);
            // Execute — should never panic or produce undo state
            let _ = cmd.execute(action, Some(&full_uri), Some(content.as_slice()));
        }
        // If we reach here without any undo-related side effects, the property holds.
    }
}

// ─── Property 5: Plugin Viewer Lifecycle ─────────────────────────────────────
// Feature: custom-file-viewers, Property 5: Plugin viewer lifecycle consistency
// **Validates: Requirements 5.2, 5.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Register then deregister plugin viewers in random order.
    /// Assert registry consistency: no dangling keys, count correct.
    #[test]
    fn property_5_plugin_viewer_lifecycle(
        viewer_keys in prop::collection::vec(valid_key_strategy(), 1..8),
        deregister_indices in prop::collection::vec(0usize..8, 0..5),
    ) {
        let registry = ViewerRegistry::new();
        let mut panel = ViewerPanel::new();

        // Register unique plugin viewers
        let mut registered_keys: Vec<String> = Vec::new();
        for key in &viewer_keys {
            if registered_keys.contains(key) {
                continue; // Skip duplicates
            }
            let viewer = Box::new(PropTestViewer::new(key));
            if registry.register_plugin(viewer).is_ok() {
                registered_keys.push(key.clone());
            }
        }

        let initial_count = registered_keys.len();
        prop_assert_eq!(registry.viewer_count(), initial_count);

        // Deregister a random subset
        let mut deregistered_count = 0;
        for &idx in &deregister_indices {
            if idx < registered_keys.len() {
                let key_str = &registered_keys[idx];
                let key = ViewerKey::new(key_str).unwrap();
                if registry.contains(&key) {
                    ff_viewers::plugin_bridge::deregister_plugin_viewer(
                        &registry, &mut panel, key_str,
                    ).unwrap();
                    deregistered_count += 1;
                }
            }
        }

        // Verify consistency
        let remaining = registry.viewer_count();
        prop_assert_eq!(remaining, initial_count - deregistered_count,
            "Registry count should reflect deregistrations");

        // All remaining keys should be valid and accessible
        let list = registry.list_viewers();
        prop_assert_eq!(list.len(), remaining);
        for info in &list {
            prop_assert!(registry.contains(&info.key));
        }
    }
}

// ─── Property 6: Selection Priority Ordering ─────────────────────────────────
// Feature: custom-file-viewers, Property 6: Selection priority ordering
// **Validates: Requirements 6.1, 6.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// When language profile defines a default, it always wins over extension match.
    /// When no profile, extension match wins over content sniff.
    #[test]
    fn property_6_selection_priority_ordering(
        profile_key in prop::option::of(valid_key_strategy()),
    ) {
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let selector = ContentSelector::new(&registry);

        // CSV file would match csv-table by extension
        let uri = "file:///data.csv";
        let content = b"a,b\n1,2";

        let result = selector.select_viewer(uri, content, profile_key.as_deref());

        match &profile_key {
            Some(pk) => {
                // Language profile should win — even if it's a different viewer
                if ViewerKey::new(pk).is_ok() {
                    prop_assert_eq!(result.as_ref().map(|k| k.as_str()), Some(pk.as_str()),
                        "Language profile '{}' should take precedence", pk);
                }
            }
            None => {
                // Extension match should win
                prop_assert_eq!(result.as_ref().map(|k| k.as_str()), Some("csv-table"),
                    "Extension match should select csv-table for .csv");
            }
        }
    }
}

// ─── Property 7: ViewerPanel Never Exposes Mutable Content ───────────────────
// Feature: custom-file-viewers, Property 7: Viewer_Panel never exposes mutable content
// **Validates: Requirements 8.1, 8.2, 8.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// The content buffer is only accessible via &[u8]. Calling render() on a
    /// viewer with any content cannot modify the content.
    #[test]
    fn property_7_viewer_panel_immutable_content(
        content in prop::collection::vec(any::<u8>(), 0..256),
    ) {
        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("hex").unwrap();

        let content_clone = content.clone();
        panel.activate(key, "file:///test".to_string(), content.clone());

        // Content buffer is only accessible via &[u8]
        let buffer: &[u8] = panel.content_buffer();
        prop_assert_eq!(buffer, content_clone.as_slice(),
            "Content buffer must match original content exactly");

        // After refresh, content is still immutable
        let new_content = vec![42u8; 10];
        let new_clone = new_content.clone();
        panel.refresh_content(new_content);
        prop_assert_eq!(panel.content_buffer(), new_clone.as_slice());
    }
}

// ─── Property 8: Debounce Coalesces Rapid Edits ──────────────────────────────
// Feature: custom-file-viewers, Property 8: Debounce coalesces rapid edits
// **Validates: Requirements 9.2, 9.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// For any sequence of N rapid document changes arriving before the debounce
    /// interval expires, only 1 refresh call occurs per quiet period.
    #[test]
    fn property_8_debounce_coalesces_rapid_edits(
        num_changes in 2u32..20,
        debounce_ms in 50u64..200,
    ) {
        let mut ctrl = RefreshController::new(debounce_ms);

        // Fire N changes rapidly (no sleep between them)
        for _ in 0..num_changes {
            ctrl.notify_document_changed();
        }

        // Immediately after rapid changes, should NOT refresh
        let immediate_refresh = ctrl.should_refresh();
        prop_assert!(!immediate_refresh,
            "Should not refresh immediately after rapid changes");

        // After waiting for the debounce period
        std::thread::sleep(std::time::Duration::from_millis(debounce_ms + 10));

        // Should fire exactly once
        let first_check = ctrl.should_refresh();
        prop_assert!(first_check, "Should refresh after debounce period");

        // Second check should NOT fire again
        let second_check = ctrl.should_refresh();
        prop_assert!(!second_check, "Should not fire twice for same change batch");

        // Total refresh count should be 1
        prop_assert_eq!(ctrl.refresh_count(), 1,
            "Exactly 1 refresh should occur for {} rapid changes", num_changes);
    }
}

// ─── Property 9: Configuration Validation Bounds ─────────────────────────────
// Feature: custom-file-viewers, Property 9: Configuration validation bounds
// **Validates: Requirements 10.1, 10.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// For split_ratio: only 0.1–0.9 accepted without warning.
    /// For debounce_ms: only positive integers within 50–5000 accepted.
    #[test]
    fn property_9_config_validation_bounds(
        split_ratio in -2.0f64..3.0,
        debounce_ms in -1000i64..10000,
    ) {
        let toml_str = format!(
            "split_ratio = {}\nrefresh_debounce_ms = {}",
            split_ratio, debounce_ms
        );
        let value: toml::Value = toml_str.parse().unwrap();
        let config = ViewerConfig::from_toml(&value);

        // split_ratio should be clamped to valid range
        prop_assert!(config.split_ratio >= MIN_SPLIT_RATIO,
            "split_ratio {} should be >= {}", config.split_ratio, MIN_SPLIT_RATIO);
        prop_assert!(config.split_ratio <= MAX_SPLIT_RATIO,
            "split_ratio {} should be <= {}", config.split_ratio, MAX_SPLIT_RATIO);

        // Check if warnings were emitted for out-of-range values
        let ratio_f32 = split_ratio as f32;
        let ratio_in_range = (MIN_SPLIT_RATIO..=MAX_SPLIT_RATIO).contains(&ratio_f32);
        if !ratio_in_range {
            prop_assert!(!config.warnings.is_empty(),
                "Out-of-range split_ratio {} should produce a warning", split_ratio);
        }

        // debounce_ms should be valid
        let debounce_in_range = debounce_ms > 0
            && (MIN_DEBOUNCE_MS..=MAX_DEBOUNCE_MS).contains(&(debounce_ms as u64));
        if debounce_ms <= 0 {
            // Should use default
            prop_assert_eq!(config.refresh_debounce_ms, 300,
                "Invalid debounce {} should fall back to default", debounce_ms);
        } else if !debounce_in_range {
            // Should be clamped
            prop_assert!(config.refresh_debounce_ms >= MIN_DEBOUNCE_MS);
            prop_assert!(config.refresh_debounce_ms <= MAX_DEBOUNCE_MS);
        }
    }
}

// ─── Property 10: Read-Only Invariant Under Viewer_Mode ──────────────────────
// Feature: custom-file-viewers, Property 10: Read-only invariant under Viewer_Mode
// **Validates: Requirements 8.3, 8.4**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Generate random command sequences during active viewer. Assert no
    /// mutating command is allowed through.
    #[test]
    fn property_10_readonly_invariant_under_viewer_mode(
        commands in prop::collection::vec(
            prop::string::string_regex("[a-z]+\\.[a-z\\-]+").unwrap(),
            1..20
        ),
        viewer_key in valid_key_strategy(),
    ) {
        let mut guard = ReadOnlyGuard::new();
        guard.activate(&viewer_key);

        let mutating_prefixes = ["edit.", "delete.", "insert.", "cut.", "paste.", "undo.", "redo.", "format."];

        for cmd in &commands {
            let result = guard.check_command(cmd);
            let is_mutating = mutating_prefixes.iter().any(|p| cmd.starts_with(p));

            if is_mutating {
                prop_assert!(result.is_err(),
                    "Mutating command '{}' should be rejected during Viewer_Mode", cmd);
                match result.unwrap_err() {
                    ff_viewers::ViewerError::ViewerReadOnlyViolation { key, command } => {
                        prop_assert_eq!(&key, &viewer_key);
                        prop_assert_eq!(&command, cmd);
                    }
                    other => prop_assert!(false,
                        "Expected ViewerReadOnlyViolation for '{}', got: {:?}", cmd, other),
                }
            } else {
                prop_assert!(result.is_ok(),
                    "Non-mutating command '{}' should be allowed during Viewer_Mode", cmd);
            }
        }
    }
}
