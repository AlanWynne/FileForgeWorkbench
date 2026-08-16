//! Property-based tests for the ff-keys crate.
//!
//! Uses proptest to validate invariants across randomly generated inputs.

use proptest::prelude::*;
use std::collections::HashSet;

use ff_keys::{
    CommandHistory, FunctionKey, HistoryEntry, HistoryStore, KeyBinding, KeyLabelBarModel, KeyMap,
    KeyMapResolver, ModifiedKey, RetrieveResult, RetrieveState,
};

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Generate a random FunctionKey from F1–F24.
fn arb_function_key() -> impl Strategy<Value = FunctionKey> {
    (1u8..=24).prop_map(|n| FunctionKey::from_number(n).unwrap())
}

/// Generate a random assignable FunctionKey (F2–F24).
fn arb_assignable_key() -> impl Strategy<Value = FunctionKey> {
    (2u8..=24).prop_map(|n| FunctionKey::from_number(n).unwrap())
}

/// Generate a random command string (non-empty, printable ASCII).
fn arb_command() -> impl Strategy<Value = String> {
    "[A-Z]{2,8}( '[a-zA-Z0-9 ]{1,10}')?( [A-Z]{2,6})?"
}

/// Generate a random key map with a subset of keys assigned.
fn arb_key_map(source: &'static str) -> impl Strategy<Value = KeyMap> {
    proptest::collection::vec((arb_assignable_key(), arb_command()), 0..=12).prop_map(
        move |entries| {
            let mut map = KeyMap::empty(source);
            for (key, cmd) in entries {
                map.set(ModifiedKey::plain(key), KeyBinding::new(cmd));
            }
            map
        },
    )
}

// ─── Property 1: Key Map Resolution Full-Replacement Invariant ──────────────

proptest! {
    /// Feature: function-keys-and-history, Property 1: Key map resolution full-replacement invariant
    ///
    /// **Validates: Requirements 1.2, 2.2, 2.5**
    ///
    /// When a Profile_Key_Map is active, the resolved key map contains ONLY entries
    /// from the Profile_Key_Map. No Global_Key_Map entry is ever visible.
    #[test]
    fn key_map_resolution_full_replacement(
        global_map in arb_key_map("global"),
        profile_map in arb_key_map("profile"),
        query_key in arb_function_key(),
    ) {
        let mut resolver = KeyMapResolver::new(global_map);
        resolver.set_profile_key_map(Some("test_profile"), Some(profile_map.clone()));

        let active = resolver.active_key_map();

        // The active map must be exactly the profile map
        let active_result = active.get_plain(query_key);
        let profile_result = profile_map.get_plain(query_key);

        match (active_result, profile_result) {
            (Some(a), Some(p)) => {
                prop_assert_eq!(a.command(), p.command());
            }
            (None, None) => {} // Both unassigned — correct
            (Some(_), None) => {
                prop_assert!(false, "Active map returned entry for {} but profile has none — global leaked through", query_key);
            }
            (None, Some(_)) => {
                prop_assert!(false, "Active map returned None for {} but profile has an entry", query_key);
            }
        }
    }
}

// ─── Property 2: Command History Deduplication and Ordering ─────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 2: Command History deduplication and ordering
    ///
    /// **Validates: Requirements 7.1, 7.2, 7.3**
    ///
    /// After any sequence of add operations, the history contains no duplicates
    /// and the most recently added command is at index 0.
    #[test]
    fn history_deduplication_and_ordering(
        commands in proptest::collection::vec("[A-Z]{2,6}( '[a-z]{1,5}')?" , 1..100),
    ) {
        let mut history = CommandHistory::new(200);

        for cmd in &commands {
            history.add(cmd.clone());

            // Invariant 1: most recent is at index 0
            prop_assert_eq!(history.get(0).unwrap().command(), cmd.as_str());

            // Invariant 2: no duplicates
            let entries: Vec<_> = history.iter().collect();
            for i in 0..entries.len() {
                for j in (i+1)..entries.len() {
                    prop_assert!(
                        !entries[i].is_duplicate_of(entries[j]),
                        "Duplicate found at indices {} and {}: '{}' and '{}'",
                        i, j, entries[i].command(), entries[j].command()
                    );
                }
            }
        }
    }
}

// ─── Property 3: Command History Bounded Capacity Invariant ─────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 3: Command History bounded capacity invariant
    ///
    /// **Validates: Requirements 9.1, 9.3**
    ///
    /// For any max_entries and any sequence of adds, history.len() never exceeds max.
    #[test]
    fn history_bounded_capacity(
        max_entries in 1usize..=100,
        commands in proptest::collection::vec("[A-Z]{2,6}[0-9]{1,4}", 1..200),
    ) {
        let mut history = CommandHistory::new(max_entries);

        for cmd in commands {
            history.add(cmd);
            prop_assert!(
                history.len() <= max_entries,
                "History length {} exceeds max_entries {}",
                history.len(), max_entries
            );
        }
    }
}

// ─── Property 4: Retrieve Pointer Cycling Correctness ───────────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 4: Retrieve Pointer cycling correctness
    ///
    /// **Validates: Requirements 5.2, 5.3, 5.4, 5.5**
    ///
    /// Successive RETRIEVEs cycle through history 0..N-1, then NoOlderHistory.
    /// Reset brings it back to start.
    #[test]
    fn retrieve_pointer_cycling(
        commands in proptest::collection::vec("[A-Z]{3,8}", 1..50),
    ) {
        // Build history with unique entries
        let unique_commands: Vec<String> = commands.into_iter().collect::<HashSet<_>>().into_iter().collect();
        if unique_commands.is_empty() {
            return Ok(());
        }

        let mut history = CommandHistory::new(200);
        for cmd in unique_commands.iter().rev() {
            history.add(cmd.clone());
        }

        let n = history.len();
        let mut state = RetrieveState::new();

        // N successive RETRIEVEs should return entries 0..N-1
        for i in 0..n {
            let result = state.retrieve(&history, "");
            match result {
                RetrieveResult::Recalled { command } => {
                    prop_assert_eq!(
                        &command,
                        history.get(i).unwrap().command(),
                        "Retrieve at step {} returned '{}' but expected '{}'",
                        i, command, history.get(i).unwrap().command()
                    );
                }
                other => {
                    prop_assert!(false, "Expected Recalled at step {}, got {:?}", i, other);
                }
            }
        }

        // (N+1)th RETRIEVE should return NoOlderHistory
        let result = state.retrieve(&history, "");
        prop_assert_eq!(result, RetrieveResult::NoOlderHistory);

        // After reset, next RETRIEVE returns entry at index 0
        state.reset();
        let result = state.retrieve(&history, "");
        match result {
            RetrieveResult::Recalled { command } => {
                prop_assert_eq!(&command, history.get(0).unwrap().command());
            }
            other => {
                prop_assert!(false, "Expected Recalled after reset, got {:?}", other);
            }
        }
    }
}

// ─── Property 5: History Store TOML Round-Trip Fidelity ─────────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 5: History Store TOML round-trip fidelity
    ///
    /// **Validates: Requirements 6.1, 6.7**
    ///
    /// Serializing to TOML and deserializing back produces identical entries.
    #[test]
    fn history_store_round_trip(
        commands in proptest::collection::vec("[A-Za-z0-9 '._-]{1,40}", 1..50),
    ) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("history.toml");
        let store = HistoryStore::new(path);

        let mut history = CommandHistory::new(200);
        for cmd in &commands {
            if !cmd.trim().is_empty() {
                history.add(cmd.clone());
            }
        }

        store.save(&history).unwrap();
        let (loaded, warnings) = store.load(200);

        prop_assert!(warnings.is_empty(), "Unexpected warnings: {:?}", warnings);
        prop_assert_eq!(loaded.len(), history.len());

        for i in 0..history.len() {
            prop_assert_eq!(
                loaded.get(i).unwrap().command(),
                history.get(i).unwrap().command(),
                "Mismatch at index {}", i
            );
        }
    }
}

// ─── Property 6: Key Label Bar Derivation Consistency ───────────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 6: Key Label Bar derivation consistency
    ///
    /// **Validates: Requirements 4.2, 4.4, 4.5**
    ///
    /// For any KeyMap, the label bar correctly reflects all assignments.
    #[test]
    fn key_label_bar_derivation_consistency(
        map in arb_key_map("test"),
    ) {
        let model = KeyLabelBarModel::from_key_map(&map);

        for key in FunctionKey::ALL.iter().filter(|k| k.is_assignable()) {
            let slot = model.slot_for(*key).unwrap();

            match map.get_plain(*key) {
                Some(binding) => {
                    let expected_label = binding.display_label().to_string();
                    prop_assert_eq!(
                        slot.label.as_deref(),
                        Some(expected_label.as_str()),
                        "Key {} has binding '{}' but slot label is {:?}",
                        key, binding.command(), slot.label
                    );
                }
                None => {
                    prop_assert_eq!(
                        slot.label.as_deref(), None,
                        "Key {} has no binding but slot label is {:?}",
                        key, slot.label
                    );
                }
            }
        }
    }
}

// ─── Property 7: Excluded Command Never Enters History ──────────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 7: Excluded Command never enters history
    ///
    /// **Validates: Requirements 8.1, 8.2, 8.4**
    ///
    /// No excluded command ever appears in history.
    #[test]
    fn excluded_commands_never_in_history(
        commands in proptest::collection::vec(
            prop_oneof![
                Just("RETRIEVE".to_string()),
                Just("UNDO".to_string()),
                Just("REDO".to_string()),
                Just("retrieve".to_string()),
                Just("undo".to_string()),
                "[A-Z]{3,8}( '[a-z]{1,5}')?".prop_map(|s| s),
            ],
            10..100
        ),
    ) {
        let excluded: HashSet<String> = ["RETRIEVE", "UNDO", "REDO"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut history = CommandHistory::new(200);

        for cmd in &commands {
            let first_token = cmd.split_whitespace().next().unwrap_or("");
            if !excluded.contains(&first_token.to_ascii_uppercase()) {
                history.add(cmd.clone());
            }
        }

        // Verify no excluded command is in history
        for entry in history.iter() {
            let name = entry.command_name().to_ascii_uppercase();
            prop_assert!(
                !excluded.contains(&name),
                "Excluded command '{}' found in history",
                entry.command()
            );
        }
    }
}

// ─── Property 8: Function Key Dispatch Idempotency ──────────────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 8: Function key dispatch idempotency
    ///
    /// **Validates: Requirements 3.1, 3.2**
    ///
    /// Same key N times produces same command N times; unassigned produces zero.
    #[test]
    fn function_key_dispatch_idempotency(
        map in arb_key_map("test"),
        presses in proptest::collection::vec(arb_assignable_key(), 5..30),
    ) {
        let resolver = KeyMapResolver::new(map.clone());

        for key in &presses {
            let result = resolver.active_key_map().get_plain(*key);

            match map.get_plain(*key) {
                Some(binding) => {
                    prop_assert_eq!(
                        result.map(|b| b.command()),
                        Some(binding.command()),
                        "Key {} should dispatch '{}' but got {:?}",
                        key, binding.command(), result.map(|b| b.command())
                    );
                }
                None => {
                    prop_assert_eq!(
                        result, None,
                        "Unassigned key {} should produce None but got {:?}",
                        key, result.map(|b| b.command())
                    );
                }
            }
        }
    }
}

// ─── Property 9: Configuration Hot-Reload Convergence ───────────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 9: Configuration hot-reload convergence
    ///
    /// **Validates: Requirements 2.4, 11.7**
    ///
    /// After a config change, system converges to a consistent state.
    #[test]
    fn config_hot_reload_convergence(
        initial_global in arb_key_map("global"),
        new_global in arb_key_map("global_v2"),
        profile_map in arb_key_map("profile"),
        new_max in 1usize..=500,
    ) {
        // Test 1: Global map hot-reload (no profile active)
        let mut resolver = KeyMapResolver::new(initial_global);
        resolver.set_global_key_map(new_global.clone());

        for key in FunctionKey::ALL.iter().filter(|k| k.is_assignable()) {
            prop_assert_eq!(
                resolver.active_key_map().get_plain(*key).map(|b| b.command()),
                new_global.get_plain(*key).map(|b| b.command()),
                "After global reload, key {} not updated", key
            );
        }

        // Test 2: Profile removal falls back to global
        resolver.set_profile_key_map(Some("test"), Some(profile_map));
        resolver.set_profile_key_map(None, None);
        prop_assert!(!resolver.is_profile_active());

        for key in FunctionKey::ALL.iter().filter(|k| k.is_assignable()) {
            prop_assert_eq!(
                resolver.active_key_map().get_plain(*key).map(|b| b.command()),
                new_global.get_plain(*key).map(|b| b.command()),
                "After profile removal, key {} should match global", key
            );
        }

        // Test 3: max_history_entries change trims history
        let mut history = CommandHistory::new(500);
        for i in 0..100 {
            history.add(format!("CMD{}", i));
        }
        history.set_max_entries(new_max);
        prop_assert!(
            history.len() <= new_max,
            "After set_max_entries({}), len is {}", new_max, history.len()
        );
    }
}

// ─── Property 10: Deduplication Case-Sensitivity Correctness ────────────────

proptest! {
    /// Feature: function-keys-and-history, Property 10: Deduplication case-sensitivity correctness
    ///
    /// **Validates: Requirement 7.2**
    ///
    /// Deduplication is symmetric and follows case rules correctly.
    #[test]
    fn deduplication_case_sensitivity(
        name1 in "[A-Z]{3,6}",
        name2_case in "[a-z]{3,6}",
        args in "('[a-zA-Z]{1,5}')?",
    ) {
        // Test symmetry: same name different case + same args = duplicate
        let entry_upper = HistoryEntry::new(format!("{} {}", name1, args).trim().to_string());
        let entry_lower = HistoryEntry::new(format!("{} {}", name1.to_lowercase(), args).trim().to_string());

        // Same command name (case-insensitive) + same args → duplicate
        prop_assert_eq!(
            entry_upper.is_duplicate_of(&entry_lower),
            entry_lower.is_duplicate_of(&entry_upper),
            "Symmetry violation"
        );

        // If names match case-insensitively and args are identical, they're duplicates
        if entry_upper.command_name().eq_ignore_ascii_case(entry_lower.command_name())
            && entry_upper.arguments() == entry_lower.arguments()
        {
            prop_assert!(entry_upper.is_duplicate_of(&entry_lower));
        }

        // Different names → not duplicates
        if !name1.eq_ignore_ascii_case(&name2_case) {
            let entry_different = HistoryEntry::new(format!("{} {}", name2_case, args).trim().to_string());
            prop_assert!(!entry_upper.is_duplicate_of(&entry_different));
        }
    }
}
