//! `TopicKey` — typed identifier for help topics.
//!
//! Each help topic is addressed by a key following the `"<namespace>:<identifier>"`
//! pattern, or a bare keyword for special topics like `"index"` and `"getting_started"`.

use std::fmt;
use std::str::FromStr;

use crate::error::HelpError;

/// The category (namespace prefix) of a `TopicKey`.
///
/// Extracted from the portion before the colon in a topic key string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TopicCategory {
    /// Primary command help: `"cmd:<NAME>"`.
    Command,
    /// Line command help: `"line:<NAME>"`.
    LineCommand,
    /// Editor mode help: `"mode:<NAME>"`.
    Mode,
    /// Feature help: `"feature:<NAME>"`.
    Feature,
    /// Configuration key help: `"config:<KEY>"`.
    Config,
    /// Macro API function help: `"api:<NAME>"`.
    Api,
    /// The top-level help index.
    Index,
    /// The getting started guide.
    GettingStarted,
}

/// A typed identifier for a help topic.
///
/// Determines the lookup key in the registry. Format is either
/// `"<namespace>:<identifier>"` or a bare keyword for special topics.
///
/// # Examples
///
/// ```
/// use ff_help::TopicKey;
///
/// let key = TopicKey::command("CHANGE");
/// assert_eq!(key.as_str(), "cmd:CHANGE");
///
/// let index = TopicKey::index();
/// assert_eq!(index.as_str(), "index");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicKey(String);

impl TopicKey {
    /// Create a command help topic key: `"cmd:CHANGE"`, `"cmd:FIND"`.
    pub fn command(name: &str) -> Self {
        Self(format!("cmd:{name}"))
    }

    /// Create a line command help topic key: `"line:CC"`, `"line:D"`.
    pub fn line_command(name: &str) -> Self {
        Self(format!("line:{name}"))
    }

    /// Create a mode help topic key: `"mode:hex"`, `"mode:edit"`.
    pub fn mode(name: &str) -> Self {
        Self(format!("mode:{name}"))
    }

    /// Create a feature help topic key: `"feature:undo"`, `"feature:macros"`.
    pub fn feature(name: &str) -> Self {
        Self(format!("feature:{name}"))
    }

    /// Create a config key help topic key: `"config:help_panel_position"`.
    pub fn config(name: &str) -> Self {
        Self(format!("config:{name}"))
    }

    /// Create a macro API function help topic key: `"api:cursor_line"`.
    pub fn api_function(name: &str) -> Self {
        Self(format!("api:{name}"))
    }

    /// The Help Index topic key.
    pub fn index() -> Self {
        Self("index".to_string())
    }

    /// The Getting Started topic key.
    pub fn getting_started() -> Self {
        Self("getting_started".to_string())
    }

    /// The line command index topic key.
    pub fn line_index() -> Self {
        Self("line:index".to_string())
    }

    /// Returns the raw string value of this topic key.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the namespace prefix (e.g., `"cmd"`, `"line"`, `"mode"`).
    ///
    /// Returns `None` for bare keywords like `"index"` or `"getting_started"`.
    pub fn namespace(&self) -> Option<&str> {
        self.0.split_once(':').map(|(ns, _)| ns)
    }

    /// Returns the identifier portion after the colon.
    ///
    /// For bare keywords, returns the entire string.
    pub fn identifier(&self) -> &str {
        self.0.split_once(':').map_or(self.0.as_str(), |(_, id)| id)
    }

    /// Extracts the `TopicCategory` from this key.
    pub fn category(&self) -> TopicCategory {
        match self.0.as_str() {
            "index" => TopicCategory::Index,
            "getting_started" => TopicCategory::GettingStarted,
            s if s.starts_with("cmd:") => TopicCategory::Command,
            s if s.starts_with("line:") => TopicCategory::LineCommand,
            s if s.starts_with("mode:") => TopicCategory::Mode,
            s if s.starts_with("feature:") => TopicCategory::Feature,
            s if s.starts_with("config:") => TopicCategory::Config,
            s if s.starts_with("api:") => TopicCategory::Api,
            _ => TopicCategory::Index, // fallback
        }
    }
}

impl fmt::Display for TopicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TopicKey {
    type Err = HelpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Special bare keywords
        match s {
            "index" | "getting_started" => return Ok(Self(s.to_string())),
            _ => {}
        }

        // Must contain a colon with valid prefix and non-empty identifier
        let (prefix, identifier) = s
            .split_once(':')
            .ok_or_else(|| HelpError::InvalidTopicKey { raw: s.to_string() })?;

        if identifier.is_empty() {
            return Err(HelpError::InvalidTopicKey { raw: s.to_string() });
        }

        match prefix {
            "cmd" | "line" | "mode" | "feature" | "config" | "api" => Ok(Self(s.to_string())),
            _ => Err(HelpError::InvalidTopicKey { raw: s.to_string() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 5.2 — TopicKey parsing with valid prefixes
    #[test]
    fn topic_key_command_constructor_formats_correctly() {
        let key = TopicKey::command("FIND");
        assert_eq!(key.as_str(), "cmd:FIND");
        assert_eq!(key.namespace(), Some("cmd"));
        assert_eq!(key.identifier(), "FIND");
        assert_eq!(key.category(), TopicCategory::Command);
    }

    // Validates: Requirement 5.2 — TopicKey parsing with valid prefixes
    #[test]
    fn topic_key_line_command_constructor_formats_correctly() {
        let key = TopicKey::line_command("CC");
        assert_eq!(key.as_str(), "line:CC");
        assert_eq!(key.namespace(), Some("line"));
        assert_eq!(key.identifier(), "CC");
        assert_eq!(key.category(), TopicCategory::LineCommand);
    }

    // Validates: Requirement 5.2 — TopicKey parsing with valid prefixes
    #[test]
    fn topic_key_mode_constructor_formats_correctly() {
        let key = TopicKey::mode("hex");
        assert_eq!(key.as_str(), "mode:hex");
        assert_eq!(key.category(), TopicCategory::Mode);
    }

    // Validates: Requirement 5.2 — TopicKey parsing with valid prefixes
    #[test]
    fn topic_key_feature_constructor_formats_correctly() {
        let key = TopicKey::feature("undo");
        assert_eq!(key.as_str(), "feature:undo");
        assert_eq!(key.category(), TopicCategory::Feature);
    }

    // Validates: Requirement 5.2 — TopicKey parsing with valid prefixes
    #[test]
    fn topic_key_config_constructor_formats_correctly() {
        let key = TopicKey::config("help_panel_position");
        assert_eq!(key.as_str(), "config:help_panel_position");
        assert_eq!(key.category(), TopicCategory::Config);
    }

    // Validates: Requirement 5.2 — TopicKey parsing with valid prefixes
    #[test]
    fn topic_key_api_function_constructor_formats_correctly() {
        let key = TopicKey::api_function("cursor_line");
        assert_eq!(key.as_str(), "api:cursor_line");
        assert_eq!(key.category(), TopicCategory::Api);
    }

    // Validates: Requirement 5.2 — Special topic keys
    #[test]
    fn topic_key_index_is_bare_keyword() {
        let key = TopicKey::index();
        assert_eq!(key.as_str(), "index");
        assert_eq!(key.namespace(), None);
        assert_eq!(key.identifier(), "index");
        assert_eq!(key.category(), TopicCategory::Index);
    }

    // Validates: Requirement 5.2 — Special topic keys
    #[test]
    fn topic_key_getting_started_is_bare_keyword() {
        let key = TopicKey::getting_started();
        assert_eq!(key.as_str(), "getting_started");
        assert_eq!(key.namespace(), None);
        assert_eq!(key.category(), TopicCategory::GettingStarted);
    }

    // Validates: Requirement 6.1 — FromStr parsing accepts valid formats
    #[test]
    fn topic_key_from_str_valid_prefixes() {
        assert!("cmd:FIND".parse::<TopicKey>().is_ok());
        assert!("line:CC".parse::<TopicKey>().is_ok());
        assert!("mode:hex".parse::<TopicKey>().is_ok());
        assert!("feature:undo".parse::<TopicKey>().is_ok());
        assert!("config:theme".parse::<TopicKey>().is_ok());
        assert!("api:cursor_line".parse::<TopicKey>().is_ok());
        assert!("index".parse::<TopicKey>().is_ok());
        assert!("getting_started".parse::<TopicKey>().is_ok());
    }

    // Validates: Requirement 6.1 — FromStr rejects invalid formats
    #[test]
    fn topic_key_from_str_rejects_invalid() {
        assert!("".parse::<TopicKey>().is_err());
        assert!("cmd:".parse::<TopicKey>().is_err());
        assert!("unknown:foo".parse::<TopicKey>().is_err());
        assert!("noprefix".parse::<TopicKey>().is_err());
        assert!(":empty_prefix".parse::<TopicKey>().is_err());
    }

    // Validates: Requirement 5.2 — Display impl
    #[test]
    fn topic_key_display_shows_raw_value() {
        let key = TopicKey::command("CHANGE");
        assert_eq!(format!("{key}"), "cmd:CHANGE");
    }
}
