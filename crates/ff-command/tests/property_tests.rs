//! Property-based tests for ff-command.
//!
//! These tests validate invariants that must hold across all valid inputs
//! using the proptest framework.

use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

use ff_command::{
    CommandHandler, CommandHistory, CommandId, CommandMetadata, CommandParams, CommandRegistry,
    CommandResult, ExecutionContext, KeyChord, KeyCode, Modifiers, ParamValue, ShortcutBinding,
    ShortcutRegistry, UndoManager,
};

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Generates valid command ID strings.
fn valid_id_strategy() -> impl Strategy<Value = String> {
    // Generate a valid segment: starts with lowercase letter, followed by
    // lowercase letters, digits, underscores.
    let segment = "[a-z][a-z0-9_]{0,10}";
    // 1 to 3 segments joined by dots
    proptest::collection::vec(segment, 1..=3).prop_map(|segments| segments.join("."))
}

/// Generates arbitrary strings that may or may not be valid IDs.
fn arbitrary_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty string
        Just(String::new()),
        // Valid-ish strings
        valid_id_strategy(),
        // Strings with uppercase
        "[A-Za-z0-9._]{1,20}",
        // Strings with spaces
        "[a-z .]{1,10}",
        // Strings with special chars
        "[a-z0-9!@#$%^&*]{1,10}",
        // Leading/trailing dots
        "\\.[a-z]{1,5}",
        "[a-z]{1,5}\\.",
        // Consecutive dots
        "[a-z]{1,3}\\.\\.[a-z]{1,3}",
    ]
}

/// Generates a valid CommandId value.
#[allow(dead_code)]
fn command_id_strategy() -> impl Strategy<Value = CommandId> {
    valid_id_strategy().prop_map(|s| CommandId::new(s).unwrap())
}

/// Generates a KeyCode value.
fn key_code_strategy() -> impl Strategy<Value = KeyCode> {
    prop_oneof![
        Just(KeyCode::A),
        Just(KeyCode::B),
        Just(KeyCode::C),
        Just(KeyCode::D),
        Just(KeyCode::E),
        Just(KeyCode::F),
        Just(KeyCode::G),
        Just(KeyCode::H),
        Just(KeyCode::I),
        Just(KeyCode::J),
        Just(KeyCode::K),
        Just(KeyCode::L),
        Just(KeyCode::M),
        Just(KeyCode::N),
        Just(KeyCode::O),
        Just(KeyCode::P),
        Just(KeyCode::Q),
        Just(KeyCode::R),
        Just(KeyCode::T),
        Just(KeyCode::U),
        Just(KeyCode::F2),
        Just(KeyCode::F3),
        Just(KeyCode::F4),
        Just(KeyCode::F5),
        Just(KeyCode::F6),
        Just(KeyCode::F7),
        Just(KeyCode::F8),
        Just(KeyCode::F9),
        Just(KeyCode::F10),
        Just(KeyCode::F11),
        Just(KeyCode::F12),
    ]
}

/// Generates a non-reserved shortcut binding (avoids reserved keys).
fn non_reserved_binding_strategy() -> impl Strategy<Value = ShortcutBinding> {
    // Use Alt modifier combinations — no reserved shortcuts use Alt alone
    let modifiers = prop_oneof![
        Just(Modifiers {
            ctrl: false,
            alt: true,
            shift: false,
            super_key: false
        }),
        Just(Modifiers {
            ctrl: true,
            alt: true,
            shift: false,
            super_key: false
        }),
        Just(Modifiers {
            ctrl: false,
            alt: true,
            shift: true,
            super_key: false
        }),
    ];

    (modifiers, key_code_strategy()).prop_map(|(m, k)| ShortcutBinding::Single(KeyChord::new(m, k)))
}

/// Generates a ParamValue for round-trip testing.
fn param_value_strategy() -> impl Strategy<Value = ParamValue> {
    let leaf = prop_oneof![
        any::<bool>().prop_map(ParamValue::Boolean),
        (-10000i64..10000i64).prop_map(ParamValue::Integer),
        (-1000.0f64..1000.0f64)
            .prop_filter("must be finite", |f| f.is_finite())
            .prop_map(ParamValue::Float),
        "[a-zA-Z0-9 _]{0,50}".prop_map(|s| ParamValue::String(s)),
    ];
    leaf
}

// ─── Test Helpers ───────────────────────────────────────────────────────────

struct NoopHandler;

impl CommandHandler for NoopHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        CommandResult::Ok
    }
}

fn make_meta(cat: &str) -> CommandMetadata {
    CommandMetadata::builder("Test", "A test command")
        .category(cat)
        .build()
}

// ─── Property Tests ─────────────────────────────────────────────────────────

proptest! {
    /// Feature: command-framework, Property 1: CommandId Validation
    ///
    /// **Validates: Requirement 1.1**
    ///
    /// For any string, `CommandId::new(s)` succeeds if and only if `s` is non-empty,
    /// contains only lowercase ASCII letters, digits, dots, and underscores,
    /// and does not start or end with a dot or contain consecutive dots.
    #[test]
    fn command_id_validation_property(s in arbitrary_string_strategy()) {
        let result = CommandId::new(&s);

        let expected_valid = !s.is_empty()
            && !s.starts_with('.')
            && !s.ends_with('.')
            && !s.contains("..")
            && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_');

        prop_assert_eq!(
            result.is_some(),
            expected_valid,
            "CommandId::new({:?}) returned {:?}, expected valid={}",
            s, result, expected_valid
        );

        // Round-trip: if valid, as_str() == original
        if let Some(id) = result {
            prop_assert_eq!(id.as_str(), s.as_str());
        }
    }

    /// Feature: command-framework, Property 2: Registry Duplicate Rejection
    ///
    /// **Validates: Requirement 1.2**
    ///
    /// For any sequence of register() calls, the registry contains exactly one
    /// entry per unique CommandId. Duplicate registrations return error.
    #[test]
    fn registry_duplicate_rejection_property(
        ids in proptest::collection::vec(valid_id_strategy(), 10..50)
    ) {
        let registry = CommandRegistry::new();
        let mut registered: HashSet<String> = HashSet::new();

        for id_str in &ids {
            let id = CommandId::new(id_str).unwrap();
            let result = registry.register(id, make_meta("test"), Box::new(NoopHandler));

            if registered.contains(id_str) {
                // Duplicate — should fail
                prop_assert!(result.is_err(), "Expected error for duplicate ID {:?}", id_str);
            } else {
                // First registration — should succeed
                prop_assert!(result.is_ok(), "Expected success for new ID {:?}", id_str);
                registered.insert(id_str.clone());
            }
        }

        // Registry count matches unique IDs
        prop_assert_eq!(registry.count(), registered.len());
    }

    /// Feature: command-framework, Property 3: Shortcut Conflict Detection
    ///
    /// **Validates: Requirement 5.4**
    ///
    /// For any set of shortcut bindings, no two distinct CommandIds can be bound
    /// to the same chord sequence.
    #[test]
    fn shortcut_conflict_detection_property(
        bindings in proptest::collection::vec(
            (non_reserved_binding_strategy(), valid_id_strategy()),
            10..30
        )
    ) {
        let registry = ShortcutRegistry::new();
        let mut bound: HashMap<ShortcutBinding, String> = HashMap::new();

        for (binding, id_str) in &bindings {
            let id = CommandId::new(id_str).unwrap();
            let result = registry.register(binding.clone(), id);

            if let Some(existing_id) = bound.get(binding) {
                if existing_id != id_str {
                    // Conflict with a different command — should fail
                    prop_assert!(result.is_err());
                } else {
                    // Same command re-registering same binding — also a conflict
                    prop_assert!(result.is_err());
                }
            } else {
                // New binding — should succeed
                prop_assert!(result.is_ok(), "Expected success for binding {:?}", binding);
                bound.insert(binding.clone(), id_str.clone());
            }
        }
    }

    /// Feature: command-framework, Property 4: Undo/Redo Stack Integrity
    ///
    /// **Validates: Requirement 4.2, 4.5, 4.6, 4.7**
    ///
    /// For any sequence of undoable command executions and undo/redo invocations,
    /// the stacks maintain correct LIFO semantics.
    #[test]
    fn undo_redo_stack_integrity_property(
        ops in proptest::collection::vec(0..3u8, 5..30)
    ) {
        use ff_command::DefaultUndoManager;

        let manager = DefaultUndoManager::new();
        let ctx = ExecutionContext::empty();
        let mut model_undo_depth: usize = 0;
        let mut model_redo_depth: usize = 0;

        for op in ops {
            match op {
                0 => {
                    // Execute undoable command
                    let record = make_mock_undo_record();
                    manager.push_undo(record);
                    model_undo_depth += 1;
                    // New command clears redo
                    manager.clear_redo();
                    model_redo_depth = 0;
                }
                1 => {
                    // Undo
                    if model_undo_depth > 0 {
                        let result = manager.perform_undo(&ctx);
                        prop_assert!(result.is_ok());
                        model_undo_depth -= 1;
                        model_redo_depth += 1;
                    } else {
                        let result = manager.perform_undo(&ctx);
                        prop_assert!(result.is_err());
                    }
                }
                _ => {
                    // Redo
                    if model_redo_depth > 0 {
                        let result = manager.perform_redo(&ctx);
                        prop_assert!(result.is_ok());
                        model_redo_depth -= 1;
                        model_undo_depth += 1;
                    } else {
                        let result = manager.perform_redo(&ctx);
                        prop_assert!(result.is_err());
                    }
                }
            }

            prop_assert_eq!(manager.undo_depth(), model_undo_depth);
            prop_assert_eq!(manager.redo_depth(), model_redo_depth);
        }
    }

    /// Feature: command-framework, Property 5: History FIFO Eviction
    ///
    /// **Validates: Requirement 7.4**
    ///
    /// When the command history reaches its maximum depth, adding a new entry
    /// evicts the oldest entry. The history size never exceeds the configured maximum.
    #[test]
    fn history_fifo_eviction_property(
        max_depth in 10usize..100,
        num_commands in 10usize..200
    ) {
        let history = CommandHistory::new(max_depth);

        for i in 0..num_commands {
            let id_str = format!("test.cmd{}", i);
            let id = CommandId::new(&id_str).unwrap();
            let params = CommandParams::new().with("index", i as i64);
            history.record(&id, &params);

            // Invariant: size never exceeds max_depth
            prop_assert!(history.len() <= max_depth);
        }

        // After all insertions, check final state
        if num_commands >= max_depth {
            prop_assert_eq!(history.len(), max_depth);

            // Verify entries are the most recent ones
            let entries = history.last_n(max_depth);
            prop_assert_eq!(entries.len(), max_depth);

            // Most recent entry should be the last command recorded
            let expected_last_id = format!("test.cmd{}", num_commands - 1);
            prop_assert_eq!(&entries[0].command_id, &expected_last_id);
        }
    }

    /// Feature: command-framework, Property 6: History Depth Clamping
    ///
    /// **Validates: Requirement 7.3**
    ///
    /// For any configured history_depth value, the effective depth is clamped to
    /// [10, 10000]. Values within range are unchanged.
    #[test]
    fn history_depth_clamping_property(value in -1000i64..20000) {
        let clamped = CommandHistory::clamp_depth_i64(value);

        // Always within bounds
        prop_assert!(clamped >= 10);
        prop_assert!(clamped <= 10000);

        // Values within range are unchanged
        if value >= 10 && value <= 10000 {
            prop_assert_eq!(clamped, value as usize);
        }

        // Values below minimum are clamped to 10
        if value < 10 {
            prop_assert_eq!(clamped, 10);
        }

        // Values above maximum are clamped to 10000
        if value > 10000 {
            prop_assert_eq!(clamped, 10000);
        }
    }

    /// Feature: command-framework, Property 7: CommandParams Round-Trip Conversion
    ///
    /// **Validates: Requirement 6.2, 6.3**
    ///
    /// For any CommandParams containing valid typed values, converting to a Lua
    /// table representation and back produces an equivalent CommandParams.
    #[test]
    fn command_params_round_trip_property(
        entries in proptest::collection::hash_map(
            "[a-z]{1,10}",
            param_value_strategy(),
            0..10
        )
    ) {
        use ff_command::scripting::{command_params_to_lua, lua_params_to_params, param_value_to_lua};
        use ff_command::LuaValue;

        let mut params = CommandParams::new();
        for (key, value) in &entries {
            params.insert(key.clone(), value.clone());
        }

        // Convert to Lua and back
        let lua_params = command_params_to_lua(&params);
        let round_tripped = lua_params_to_params(lua_params).unwrap();

        // Verify all original entries are preserved
        for (key, value) in &entries {
            let original_lua = param_value_to_lua(value);
            let rt_value = round_tripped.get(key);

            match (value, rt_value) {
                (ParamValue::String(s), Some(ParamValue::String(rt_s))) => {
                    prop_assert_eq!(s, rt_s);
                }
                (ParamValue::Integer(i), Some(ParamValue::Integer(rt_i))) => {
                    prop_assert_eq!(i, rt_i);
                }
                (ParamValue::Float(f), Some(ParamValue::Float(rt_f))) => {
                    // Float comparison with epsilon
                    prop_assert!((f - rt_f).abs() < 1e-10,
                        "Float mismatch: {} vs {}", f, rt_f);
                }
                (ParamValue::Boolean(b), Some(ParamValue::Boolean(rt_b))) => {
                    prop_assert_eq!(b, rt_b);
                }
                _ => {
                    // For non-nil values, round-trip should preserve type
                    if !matches!(original_lua, LuaValue::Nil) {
                        prop_assert!(
                            rt_value.is_some(),
                            "Key {:?} lost during round-trip", key
                        );
                    }
                }
            }
        }
    }

    /// Feature: command-framework, Property 8: Reserved Shortcut Immutability
    ///
    /// **Validates: Requirement 5.3, 5.5**
    ///
    /// For any reserved shortcut, any attempt to register a binding for that chord
    /// (regardless of the target CommandId) shall be rejected.
    #[test]
    fn reserved_shortcut_immutability_property(
        cmd_id in valid_id_strategy()
    ) {
        use ff_command::shortcut::reserved::reserved_shortcuts;

        let registry = ShortcutRegistry::new();
        let id = CommandId::new(&cmd_id).unwrap();

        // Every reserved shortcut must reject registration
        for reserved in reserved_shortcuts() {
            let result = registry.register(reserved.binding.clone(), id.clone());
            prop_assert!(
                result.is_err(),
                "Expected error when registering reserved shortcut {:?}",
                reserved.binding
            );

            // is_reserved must return true
            prop_assert!(registry.is_reserved(&reserved.binding));
        }
    }
}

// ─── Helper for undo/redo property test ─────────────────────────────────────

use ff_command::{CommandError, UndoRecord};

#[derive(Debug)]
struct MockRecord {
    cmd_id: CommandId,
}

impl UndoRecord for MockRecord {
    fn undo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
        Ok(())
    }
    fn redo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
        Ok(())
    }
    fn description(&self) -> &str {
        "mock"
    }
    fn command_id(&self) -> &CommandId {
        &self.cmd_id
    }
}

fn make_mock_undo_record() -> Box<dyn UndoRecord> {
    Box::new(MockRecord {
        cmd_id: CommandId::new("test.mock").unwrap(),
    })
}
