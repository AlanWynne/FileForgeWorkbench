//! Command_Line_Outcome: what the `Command ===>` field should contain after an
//! invocation runs.
//!
//! A command (native, or -- in a future slice -- a Lua/REXX/external command)
//! returns a `CommandLineOutcome`; the shell applies it at the single dispatch
//! decision point on BOTH the Enter and key-forward paths. When a command
//! returns nothing, the framework applies a default (clear on success, restore
//! the executed text on error).
//!
//! The outcome has a serialisable [`OutcomeData`] shape (`{ action, text? }`)
//! with a total, lossless round-trip to and from the native enum, so a non-Rust
//! producer able to emit a key/value object can drive the command line through
//! the same mechanism (bridges are a later slice).
//!
//! Validates: command-framework Requirement 13 (CR-CH-033)

use serde::{Deserialize, Serialize};

/// What the `Command ===>` field should contain after an invocation.
///
/// Validates: command-framework Requirement 13 (glossary: Command_Line_Outcome)
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommandLineOutcome {
    /// Empty the field (the default when a command succeeds).
    Clear,
    /// Put back exactly what was executed -- for correction / retry (the default
    /// when a resolved command reports an error, and also the effect for an
    /// unresolved command so a typo remains editable).
    Restore,
    /// Put arbitrary text back: RETRIEVE recall, or a future suggested prompt.
    Set(String),
    /// Do not touch the field: keep whatever the command itself left in it
    /// (e.g. RETRIEVE LIST clears it and opens the picker).
    Leave,
}

/// The serialisable representation of a [`CommandLineOutcome`]: a tagged object
/// `{ "action": "clear" | "restore" | "set" | "leave", "text": <only for set> }`.
/// This is the documented public boundary a future language bridge targets; it
/// contains no Rust-specific construct, so any producer that can emit a
/// key/value object can conform.
///
/// Validates: command-framework Requirement 13.5, 13.6 (Outcome_Data_Shape)
// Slice 2 (CR-CH-033): the serialisable contract exists ahead of its runtime
// consumers (the Lua / REXX / External bridges of Slice 3+, gated per engine).
// It is exercised now only by the round-trip tests; not dead, just early.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OutcomeData {
    /// One of `clear`, `restore`, `set`, `leave` (matched case-insensitively).
    pub action: String,
    /// Present only when `action == "set"`; the text to place in the field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

// Slice 2 mapping: the (de)serialisation exists ahead of the bridge consumers
// (Slice 3+); exercised now by the round-trip tests only.
#[allow(dead_code)]
impl CommandLineOutcome {
    /// Serialise to the [`OutcomeData`] shape (total: every variant maps to
    /// exactly one shape).
    ///
    /// Validates: command-framework Requirement 13.5
    pub(crate) fn to_data(&self) -> OutcomeData {
        match self {
            CommandLineOutcome::Clear => OutcomeData {
                action: "clear".to_string(),
                text: None,
            },
            CommandLineOutcome::Restore => OutcomeData {
                action: "restore".to_string(),
                text: None,
            },
            CommandLineOutcome::Set(t) => OutcomeData {
                action: "set".to_string(),
                text: Some(t.clone()),
            },
            CommandLineOutcome::Leave => OutcomeData {
                action: "leave".to_string(),
                text: None,
            },
        }
    }

    /// Deserialise from the [`OutcomeData`] shape. An unknown action, or `set`
    /// without `text`, maps to `None` (the caller substitutes the framework
    /// default); this never panics.
    ///
    /// Validates: command-framework Requirement 13.5
    pub(crate) fn from_data(data: &OutcomeData) -> Option<CommandLineOutcome> {
        match data.action.trim().to_ascii_lowercase().as_str() {
            "clear" => Some(CommandLineOutcome::Clear),
            "restore" => Some(CommandLineOutcome::Restore),
            "leave" => Some(CommandLineOutcome::Leave),
            "set" => data.text.clone().map(CommandLineOutcome::Set),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: command-framework Requirement 13.5 -- every native variant
    // round-trips losslessly through the OutcomeData shape.
    #[test]
    fn outcome_data_shape_round_trips_all_variants() {
        for outcome in [
            CommandLineOutcome::Clear,
            CommandLineOutcome::Restore,
            CommandLineOutcome::Set("FIND foo".to_string()),
            CommandLineOutcome::Set(String::new()),
            CommandLineOutcome::Leave,
        ] {
            let data = outcome.to_data();
            let back = CommandLineOutcome::from_data(&data);
            assert_eq!(back, Some(outcome.clone()), "round-trip must be lossless");
        }
    }

    // Validates: command-framework Requirement 13.5 -- the action strings are the
    // documented tags.
    #[test]
    fn outcome_data_action_tags_are_stable() {
        assert_eq!(CommandLineOutcome::Clear.to_data().action, "clear");
        assert_eq!(CommandLineOutcome::Restore.to_data().action, "restore");
        assert_eq!(CommandLineOutcome::Leave.to_data().action, "leave");
        let set = CommandLineOutcome::Set("x".to_string()).to_data();
        assert_eq!(set.action, "set");
        assert_eq!(set.text.as_deref(), Some("x"));
    }

    // Validates: command-framework Requirement 13.5 -- `set` REQUIRES `text`; a
    // `set` shape with no text is invalid and maps to None (caller -> default).
    #[test]
    fn outcome_data_shape_set_requires_text() {
        let no_text = OutcomeData {
            action: "set".to_string(),
            text: None,
        };
        assert_eq!(CommandLineOutcome::from_data(&no_text), None);
    }

    // Validates: command-framework Requirement 13.5 -- an unknown action maps to
    // None (never panics); the caller substitutes the framework default.
    #[test]
    fn invalid_shape_maps_to_default() {
        let bad = OutcomeData {
            action: "explode".to_string(),
            text: None,
        };
        assert_eq!(CommandLineOutcome::from_data(&bad), None);
    }

    // Validates: command-framework Requirement 13.5, 13.6 -- action matching is
    // case-insensitive and whitespace-tolerant, so a bridge need not normalise.
    #[test]
    fn outcome_data_action_is_case_insensitive() {
        let d = OutcomeData {
            action: "  CLEAR ".to_string(),
            text: None,
        };
        assert_eq!(
            CommandLineOutcome::from_data(&d),
            Some(CommandLineOutcome::Clear)
        );
    }

    // Validates: command-framework Requirement 13.5, 13.6 -- the shape survives a
    // real serialisation round-trip (TOML here; the same holds for any format a
    // bridge uses), proving it is a genuine cross-language data contract.
    #[test]
    fn outcome_data_survives_toml_round_trip() {
        let data = CommandLineOutcome::Set("LOCATE 42".to_string()).to_data();
        let text = toml::to_string(&data).expect("serialise");
        let parsed: OutcomeData = toml::from_str(&text).expect("parse");
        assert_eq!(parsed, data);
        assert_eq!(
            CommandLineOutcome::from_data(&parsed),
            Some(CommandLineOutcome::Set("LOCATE 42".to_string()))
        );
    }
}
