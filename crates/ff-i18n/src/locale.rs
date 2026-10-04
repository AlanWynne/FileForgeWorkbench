//! The [`Locale`] type: a thin wrapper over a BCP-47 language identifier.
//!
//! A `Locale` is parsed from the `ui.locale` configuration string and selects
//! which catalogues are active (localization Req 1.1, 2.3). English (`en`) is
//! the always-present Identity_Base and is exposed as a constant so callers can
//! name the base without re-parsing (localization Req 2.5).

use std::fmt;
use std::str::FromStr;

use unic_langid::LanguageIdentifier;

/// The BCP-47 identifier string for the English Identity_Base locale.
pub const IDENTITY_BASE_TAG: &str = "en";

/// A BCP-47 language identifier selecting which catalogues are active.
///
/// Wraps [`unic_langid::LanguageIdentifier`] so the rest of the crate works in
/// terms of a small, documented type rather than exposing `unic-langid`
/// directly at every call site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    inner: LanguageIdentifier,
}

impl Locale {
    /// The English Identity_Base locale (`en`).
    ///
    /// This locale is always present; every other locale resolves against it as
    /// the fallback identity base (localization Req 2.5, 3.2).
    pub fn identity_base() -> Self {
        // IDENTITY_BASE_TAG is a compile-time constant known to be a valid
        // identifier, so parsing it cannot fail; falling back to the default
        // identifier keeps this constructor infallible without an unwrap.
        let inner = IDENTITY_BASE_TAG
            .parse::<LanguageIdentifier>()
            .unwrap_or_default();
        Self { inner }
    }

    /// Parse a `Locale` from the `ui.locale` configuration string.
    ///
    /// # Errors
    ///
    /// Returns the `unic-langid` parse error when `tag` is not a well-formed
    /// BCP-47 language identifier. Callers that cannot tolerate a parse failure
    /// (the defence-in-depth path) should fall back to [`Locale::identity_base`]
    /// and log a WARN (localization Req 1.6).
    pub fn parse(tag: &str) -> Result<Self, unic_langid::LanguageIdentifierError> {
        let inner = tag.parse::<LanguageIdentifier>()?;
        Ok(Self { inner })
    }

    /// Return the canonical BCP-47 tag string for this locale (e.g. `en`).
    pub fn tag(&self) -> String {
        self.inner.to_string()
    }

    /// Return the underlying `unic-langid` identifier, as required when building
    /// a Fluent bundle keyed by this locale.
    pub fn lang_id(&self) -> &LanguageIdentifier {
        &self.inner
    }

    /// Whether this locale is the English Identity_Base.
    pub fn is_identity_base(&self) -> bool {
        self.inner == Self::identity_base().inner
    }
}

impl FromStr for Locale {
    type Err = unic_langid::LanguageIdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
