//! Property-based tests for `ModifiedKey` and `KeyMap` binding isolation.
//!
//! Covers Task 30 (Phase AN.5):
//!   30.1 — All 96 `ModifiedKey` TOML names round-trip through `parse()`
//!   30.2 — `get_plain()` always returns the `None`-modifier entry regardless of other modifier bindings
//!   30.3 — `KeyMap::from_toml_table` with mixed modifier entries produces exactly the expected set

use ff_keys::{FunctionKey, KeyBinding, KeyMap, KeyModifier, ModifiedKey};
use proptest::prelude::*;

// === Strategies =============================================================

fn arb_function_key() -> impl Strategy<Value = FunctionKey> {
    (1u8..=24).prop_map(|n| FunctionKey::from_number(n).unwrap())
}

fn arb_modifier() -> impl Strategy<Value = KeyModifier> {
    prop_oneof![
        Just(KeyModifier::None),
        Just(KeyModifier::Shift),
        Just(KeyModifier::Ctrl),
        Just(KeyModifier::Alt),
    ]
}

fn arb_modified_key() -> impl Strategy<Value = ModifiedKey> {
    (arb_function_key(), arb_modifier()).prop_map(|(key, modifier)| ModifiedKey { key, modifier })
}

fn arb_command() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("END".to_string()),
        Just("FIND".to_string()),
        Just("SAVE".to_string()),
        Just("UP MAX".to_string()),
        Just("DOWN MAX".to_string()),
        Just("RETRIEVE".to_string()),
        Just("SWAP".to_string()),
        Just("COPY".to_string()),
        Just("MOVE".to_string()),
    ]
}

// === PBT 30.1 — All 96 ModifiedKey TOML names round-trip ===================

// Feature: function-keys-and-history, Property 1: ModifiedKey TOML round-trip
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// All 96 `ModifiedKey` values produce a `toml_name()` that parses back
    /// to the original `ModifiedKey`.
    ///
    /// Validates: Requirement 20.11, 20.12
    #[test]
    fn modified_key_toml_name_always_round_trips(mk in arb_modified_key()) {
        // Feature: function-keys-and-history, Property 1: ModifiedKey TOML round-trip
        let name = mk.toml_name();
        let parsed = ModifiedKey::parse(&name);
        prop_assert_eq!(
            parsed,
            Some(mk),
            "round-trip failed for toml_name={}", name
        );
    }
}

// === PBT 30.2 — get_plain() unaffected by modifier bindings ================

// Feature: function-keys-and-history, Property 2: plain binding isolation
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// For any `KeyMap`, `get_plain(F)` always returns the `None`-modifier entry
    /// regardless of what Shift/Ctrl/Alt entries exist for the same key.
    ///
    /// Validates: Requirement 20.9, 20.12
    #[test]
    fn get_plain_unaffected_by_modifier_bindings(
        key in arb_function_key(),
        plain_cmd in arb_command(),
        shift_cmd in arb_command(),
        ctrl_cmd in arb_command(),
        alt_cmd in arb_command(),
    ) {
        // Feature: function-keys-and-history, Property 2: plain binding isolation
        let mut map = KeyMap::empty("test");
        map.set(ModifiedKey::plain(key), KeyBinding::new(plain_cmd.clone()));
        map.set(ModifiedKey::shift(key), KeyBinding::new(shift_cmd));
        map.set(ModifiedKey::ctrl(key),  KeyBinding::new(ctrl_cmd));
        map.set(ModifiedKey::alt(key),   KeyBinding::new(alt_cmd));

        // get_plain must return exactly the plain binding
        let result = map.get_plain(key);
        prop_assert!(result.is_some(), "get_plain returned None after setting plain binding");
        prop_assert_eq!(
            result.unwrap().command(),
            plain_cmd.as_str(),
            "get_plain returned wrong command"
        );
    }
}

// === PBT 30.3 — from_toml_table produces exactly the expected entries =======

// Feature: function-keys-and-history, Property 3: from_toml_table no cross-contamination
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// `KeyMap::from_toml_table` with mixed modifier entries produces exactly
    /// the expected set of `ModifiedKey` entries with no cross-contamination.
    ///
    /// Validates: Requirement 20.11, 20.12
    #[test]
    fn from_toml_table_mixed_modifiers_no_cross_contamination(
        key_num in 1u8..=24,
        plain_cmd in arb_command(),
        shift_cmd in arb_command(),
    ) {
        // Feature: function-keys-and-history, Property 3: from_toml_table no cross-contamination
        let key = FunctionKey::from_number(key_num).unwrap();
        let n = key_num;

        // Build a TOML string with plain and shift bindings for the same key
        let toml_str = format!(
            "F{n} = \"{plain_cmd}\"\nSF{n} = \"{shift_cmd}\"\n"
        );
        let table: toml::Table = toml_str.parse().expect("valid toml");
        let (map, warnings) = KeyMap::from_toml_table(&table, "test");

        prop_assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        prop_assert_eq!(map.len(), 2);

        // Plain binding is correct
        let plain = map.get_plain(key);
        prop_assert!(plain.is_some(), "plain binding missing");
        prop_assert_eq!(plain.unwrap().command(), plain_cmd.as_str());

        // Shift binding is correct and independent
        let shift = map.get(ModifiedKey::shift(key));
        prop_assert!(shift.is_some(), "shift binding missing");
        prop_assert_eq!(shift.unwrap().command(), shift_cmd.as_str());

        // Ctrl and Alt are absent (not set)
        prop_assert!(map.get(ModifiedKey::ctrl(key)).is_none(),  "ctrl should be absent");
        prop_assert!(map.get(ModifiedKey::alt(key)).is_none(),   "alt should be absent");
    }
}
