//! Integration tests for the ff-keys crate.
//!
//! These tests exercise cross-component interactions and end-to-end flows.

use std::collections::HashSet;
use std::fs;

use ff_keys::{
    CommandHistory, FunctionKey, HistoryStore, KeyBinding, KeyLabelBarModel, KeyMap,
    KeyMapResolver, ModifiedKey, RetrieveResult, RetrieveState, DEFAULT_EXCLUDED_COMMANDS,
};
use tempfile::TempDir;

// ─── Integration Test 1: Global key map load and function key dispatch ──────

#[test]
fn global_key_map_load_and_dispatch_end_to_end() {
    // Validates: Requirement 1.1, 3.1
    let toml_str = r#"
        F3 = "END"
        F5 = "FIND 'ERROR' ALL"
        F7 = { command = "UP MAX", label = "UP" }
        F12 = "RETRIEVE"
    "#;
    let table: toml::Table = toml_str.parse().unwrap();
    let (global_map, warnings) = KeyMap::from_toml_table(&table, "global");
    assert!(warnings.is_empty());

    let resolver = KeyMapResolver::new(global_map);

    // Simulate dispatching F5
    let active = resolver.active_key_map();
    let binding = active.get_plain(FunctionKey::F5).unwrap();
    assert_eq!(binding.command(), "FIND 'ERROR' ALL");

    // Unassigned key produces no action
    assert!(active.get_plain(FunctionKey::F4).is_none());
}

// ─── Integration Test 2: Profile key map fully replaces global ──────────────

#[test]
fn profile_key_map_fully_replaces_global() {
    // Validates: Requirement 2.2, 2.5
    let mut global = KeyMap::empty("global");
    global.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
    global.set(ModifiedKey::plain(FunctionKey::F5), KeyBinding::new("FIND"));
    global.set(
        ModifiedKey::plain(FunctionKey::F7),
        KeyBinding::with_label("UP MAX", "UP"),
    );

    let mut profile = KeyMap::empty("cobol");
    profile.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
    profile.set(
        ModifiedKey::plain(FunctionKey::F10),
        KeyBinding::with_label("MACRO cobol_check", "CHECK"),
    );

    let mut resolver = KeyMapResolver::new(global);
    resolver.set_profile_key_map(Some("cobol"), Some(profile));

    // Profile keys work
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F3)
            .unwrap()
            .command(),
        "END"
    );
    assert_eq!(
        resolver
            .active_key_map()
            .get_plain(FunctionKey::F10)
            .unwrap()
            .command(),
        "MACRO cobol_check"
    );

    // Global-only keys are NOT inherited
    assert!(resolver
        .active_key_map()
        .get_plain(FunctionKey::F5)
        .is_none());
    assert!(resolver
        .active_key_map()
        .get_plain(FunctionKey::F7)
        .is_none());
}

// ─── Integration Test 3: RETRIEVE cycles through history and resets ─────────

#[test]
fn retrieve_cycles_through_history_and_resets() {
    // Validates: Requirement 5.1–5.5
    let mut history = CommandHistory::new(200);
    history.add("SAVE");
    history.add("FIND 'ERROR'");
    history.add("CHANGE 'foo' 'bar'");

    let mut state = RetrieveState::new();

    // First RETRIEVE gets most recent
    assert_eq!(
        state.retrieve(&history, ""),
        RetrieveResult::Recalled {
            command: "CHANGE 'foo' 'bar'".to_string()
        }
    );

    // Second gets next older
    assert_eq!(
        state.retrieve(&history, ""),
        RetrieveResult::Recalled {
            command: "FIND 'ERROR'".to_string()
        }
    );

    // Third gets oldest
    assert_eq!(
        state.retrieve(&history, ""),
        RetrieveResult::Recalled {
            command: "SAVE".to_string()
        }
    );

    // Fourth hits end
    assert_eq!(state.retrieve(&history, ""), RetrieveResult::NoOlderHistory);

    // Reset (simulating non-RETRIEVE command submission)
    state.reset();

    // RETRIEVE starts from most recent again
    assert_eq!(
        state.retrieve(&history, ""),
        RetrieveResult::Recalled {
            command: "CHANGE 'foo' 'bar'".to_string()
        }
    );
}

// ─── Integration Test 4: History persistence across startup/shutdown ────────

#[test]
fn history_persistence_across_startup_shutdown() {
    // Validates: Requirement 6.1–6.3
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("command_history.toml");
    let store = HistoryStore::new(path);

    // Simulate session 1: build history and save
    let mut history = CommandHistory::new(200);
    history.add("SAVE");
    history.add("FIND 'ERROR' ALL");
    history.add("CHANGE 'foo' 'bar' ALL");
    store.save(&history).unwrap();

    // Simulate session 2: load history
    let (loaded, warnings) = store.load(200);
    assert!(warnings.is_empty());
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded.get(0).unwrap().command(), "CHANGE 'foo' 'bar' ALL");
    assert_eq!(loaded.get(1).unwrap().command(), "FIND 'ERROR' ALL");
    assert_eq!(loaded.get(2).unwrap().command(), "SAVE");
}

// ─── Integration Test 5: Corrupt History Store graceful degradation ─────────

#[test]
fn corrupt_history_store_graceful_degradation() {
    // Validates: Requirement 6.5, 6.6
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("command_history.toml");
    let store = HistoryStore::new(path.clone());

    // Write garbage to the file
    fs::write(&path, "{{{{this is not valid TOML at all!!!!").unwrap();

    let (history, warnings) = store.load(200);
    assert!(history.is_empty());
    assert!(!warnings.is_empty());
}

// ─── Integration Test 6: Hot-reload of global_key_map updates label bar ─────

#[test]
fn hot_reload_updates_label_bar() {
    // Validates: Requirement 4.6, 11.7
    let mut global = KeyMap::empty("global");
    global.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
    global.set(ModifiedKey::plain(FunctionKey::F5), KeyBinding::new("FIND"));

    let mut resolver = KeyMapResolver::new(global);
    let mut label_bar = KeyLabelBarModel::from_key_map(resolver.active_key_map());

    assert_eq!(
        label_bar
            .slot_for(FunctionKey::F3)
            .unwrap()
            .label
            .as_deref(),
        Some("END")
    );

    // Simulate hot-reload: new global map
    let mut new_global = KeyMap::empty("global");
    new_global.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("QUIT"));
    new_global.set(ModifiedKey::plain(FunctionKey::F9), KeyBinding::new("SWAP"));
    resolver.set_global_key_map(new_global);

    // Update label bar
    label_bar.update(resolver.active_key_map());

    assert_eq!(
        label_bar
            .slot_for(FunctionKey::F3)
            .unwrap()
            .label
            .as_deref(),
        Some("QUIT")
    );
    assert_eq!(
        label_bar
            .slot_for(FunctionKey::F9)
            .unwrap()
            .label
            .as_deref(),
        Some("SWAP")
    );
    // F5 no longer assigned
    assert_eq!(label_bar.slot_for(FunctionKey::F5).unwrap().label, None);
}

// ─── Integration Test 7: max_history_entries enforcement and hot-reload trim ─

#[test]
fn max_history_entries_enforcement_and_trim() {
    // Validates: Requirement 9.1, 9.3, 11.7
    let mut history = CommandHistory::new(5);

    for i in 0..10 {
        history.add(format!("CMD{}", i));
    }
    assert_eq!(history.len(), 5);
    assert_eq!(history.get(0).unwrap().command(), "CMD9");
    assert_eq!(history.get(4).unwrap().command(), "CMD5");

    // Hot-reload: reduce max
    history.set_max_entries(3);
    assert_eq!(history.len(), 3);
    assert_eq!(history.get(0).unwrap().command(), "CMD9");
    assert_eq!(history.get(2).unwrap().command(), "CMD7");
}

// ─── Integration Test 8: Excluded command not recorded via function key ──────

#[test]
fn function_key_dispatches_excluded_command_without_history() {
    // Validates: Requirement 3.6, 8.1, 8.2
    let excluded: HashSet<String> = DEFAULT_EXCLUDED_COMMANDS
        .iter()
        .map(|s| s.to_string())
        .collect();

    let mut global = KeyMap::empty("global");
    global.set(
        ModifiedKey::plain(FunctionKey::F12),
        KeyBinding::new("RETRIEVE"),
    );
    global.set(ModifiedKey::plain(FunctionKey::F5), KeyBinding::new("FIND"));

    let resolver = KeyMapResolver::new(global);
    let mut history = CommandHistory::new(200);

    // Simulate dispatching keys
    for key in &[FunctionKey::F5, FunctionKey::F12, FunctionKey::F5] {
        if let Some(binding) = resolver.active_key_map().get_plain(*key) {
            let cmd = binding.command();
            let first_token = cmd.split_whitespace().next().unwrap_or("");
            if !excluded.contains(&first_token.to_ascii_uppercase()) {
                history.add(cmd);
            }
        }
    }

    // History should only contain FIND (not RETRIEVE)
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().command(), "FIND");
}

// ─── Integration Test 9: History Dropdown selection updates Retrieve Pointer ─

#[test]
fn history_dropdown_selection_updates_retrieve_pointer() {
    // Validates: Requirement 10.3, 10.4
    let mut history = CommandHistory::new(200);
    history.add("CMD1");
    history.add("CMD2");
    history.add("CMD3");
    history.add("CMD4");

    let mut state = RetrieveState::new();

    // Simulate dropdown selection of index 2 ("CMD2")
    state.set_position(2);

    // Next RETRIEVE should continue from index 3 (CMD1)
    let result = state.retrieve(&history, "");
    assert_eq!(
        result,
        RetrieveResult::Recalled {
            command: "CMD1".to_string()
        }
    );
}

// ─── Integration Test 10: Profile switch triggers recomputation and label bar ─

#[test]
fn profile_switch_triggers_recomputation_and_label_bar_update() {
    // Validates: Requirement 2.6, 4.6
    let mut global = KeyMap::empty("global");
    global.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
    global.set(ModifiedKey::plain(FunctionKey::F5), KeyBinding::new("FIND"));

    let mut resolver = KeyMapResolver::new(global);
    let mut label_bar = KeyLabelBarModel::from_key_map(resolver.active_key_map());

    // Initially: global map active
    assert_eq!(
        label_bar
            .slot_for(FunctionKey::F5)
            .unwrap()
            .label
            .as_deref(),
        Some("FIND")
    );

    // Switch to COBOL profile
    let mut cobol = KeyMap::empty("cobol");
    cobol.set(
        ModifiedKey::plain(FunctionKey::F10),
        KeyBinding::with_label("MACRO cobol_check", "CHECK"),
    );
    resolver.set_profile_key_map(Some("cobol"), Some(cobol));
    label_bar.update(resolver.active_key_map());

    // F5 should now be blank (not inherited from global)
    assert_eq!(label_bar.slot_for(FunctionKey::F5).unwrap().label, None);
    // F10 should show CHECK
    assert_eq!(
        label_bar
            .slot_for(FunctionKey::F10)
            .unwrap()
            .label
            .as_deref(),
        Some("CHECK")
    );

    // Switch back to no profile
    resolver.set_profile_key_map(None, None);
    label_bar.update(resolver.active_key_map());

    // Back to global: F5 = FIND, F10 = blank
    assert_eq!(
        label_bar
            .slot_for(FunctionKey::F5)
            .unwrap()
            .label
            .as_deref(),
        Some("FIND")
    );
    assert_eq!(label_bar.slot_for(FunctionKey::F10).unwrap().label, None);
}
