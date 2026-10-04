//! Named placeable arguments for argument-bearing message lookups.
//!
//! [`MessageArgs`] is a small, GUI-free builder over Fluent's `FluentArgs` so
//! callers supply named placeables (`{ $arg }`) without constructing Fluent
//! types directly (localization Req 3.1, 9.1). A command verb or other stable
//! identifier embedded in a message is supplied as an ordinary string argument
//! and is NEVER translated (localization Req 9.2).

use fluent_bundle::{FluentArgs, FluentValue};

/// A set of named arguments passed to [`crate::t_args`].
///
/// Build it fluently:
///
/// ```
/// use ff_i18n::MessageArgs;
/// let args = MessageArgs::new().set("verb", "LOCK OFF");
/// ```
#[derive(Debug, Default, Clone)]
pub struct MessageArgs {
    entries: Vec<(String, ArgValue)>,
}

/// The value of a single named argument.
///
/// Kept deliberately small (string or number) because the catalogue seam only
/// needs to interpolate text and numeric runtime values; richer Fluent values
/// are not part of the first-gate scope.
#[derive(Debug, Clone)]
enum ArgValue {
    /// A text value (including a non-translated embedded verb).
    Text(String),
    /// A numeric value (for plural/number placeables).
    Number(f64),
}

impl MessageArgs {
    /// Create an empty argument set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a named text argument, returning `self` for chaining.
    ///
    /// Use this for interpolated prose values and for an embedded command verb
    /// that must stay English (localization Req 9.2).
    pub fn set(mut self, name: &str, value: impl Into<String>) -> Self {
        self.entries
            .push((name.to_string(), ArgValue::Text(value.into())));
        self
    }

    /// Set a named numeric argument, returning `self` for chaining.
    pub fn set_number(mut self, name: &str, value: f64) -> Self {
        self.entries
            .push((name.to_string(), ArgValue::Number(value)));
        self
    }

    /// Whether no arguments have been supplied.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Convert into a borrowed `FluentArgs` for pattern formatting.
    pub(crate) fn to_fluent_args(&self) -> FluentArgs<'_> {
        let mut args = FluentArgs::new();
        for (name, value) in &self.entries {
            match value {
                ArgValue::Text(text) => args.set(name.as_str(), FluentValue::from(text.as_str())),
                ArgValue::Number(number) => args.set(name.as_str(), FluentValue::from(*number)),
            }
        }
        args
    }
}
