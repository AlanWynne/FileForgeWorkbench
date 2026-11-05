//! DISPLAY-TIME inherited short-description for alias rows and command listings
//! (CR-NR-119, command-configurator Requirement 9, context-help Requirement 22).
//!
//! An alias row whose own Description is EMPTY DISPLAYS the one-line description
//! of its CANONICAL root command, obtained through the single
//! `HelpTopicRegistry::short_description(cmd:<ROOT>)` lookup (Req 9.1). The
//! inherited value is applied at DISPLAY time ONLY and is NEVER written into the
//! persisted alias row: the stored Description stays the user's own value (empty
//! for the `X -> EXIT` seed), so the seed write / Reset-bare rebuild is
//! UNCHANGED and does not depend on help-registry readiness (Req 9.2). A missing
//! `cmd:<ROOT>` topic degrades to an EMPTY string -- never a panic (Req 9.4).
//!
//! The command palette / command configurator listings reuse the SAME lookup for
//! a home-grown FFCMD root command's one-liner rather than a duplicated literal
//! (Req 9.3), so there is one description per command.
//!
//! Validates: command-configurator Requirement 9.1, 9.2, 9.3, 9.4, 9.5;
//! context-help Requirement 22.2, 22.3.

use std::collections::HashMap;

use ff_help::{HelpTopicRegistry, TopicKey};

/// The inherited one-line description of a root command, through the single
/// `short_description(cmd:<ROOT>)` lookup keyed by the UPPERCASED canonical root
/// (context-help Req 22.2). Returns an empty string when the root is blank or no
/// `cmd:<ROOT>` topic is loaded (Req 9.4 -- graceful fallback, never a panic).
///
/// Validates: command-configurator Requirement 9.3; context-help Requirement 22.2
pub(crate) fn inherited_root_description(canonical: &str, registry: &HelpTopicRegistry) -> String {
    let upper = canonical.trim().to_uppercase();
    if upper.is_empty() {
        return String::new();
    }
    registry
        .short_description(&TopicKey::command(&upper))
        .unwrap_or_default()
}

/// Build a `UPPERCASED canonical -> one-line description` lookup for every
/// DISTINCT non-empty `canonical` among `canonicals`, through the single
/// registry lookup (Req 9.1/9.3). Roots with no loaded `cmd:<ROOT>` topic are
/// OMITTED (the caller then shows empty, Req 9.4). Computed ONCE per load so the
/// pure render reads the map without touching the registry.
///
/// Validates: command-configurator Requirement 9.1, 9.3
pub(crate) fn inherited_description_map<'a, I>(
    canonicals: I,
    registry: &HelpTopicRegistry,
) -> HashMap<String, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut map = HashMap::new();
    for canonical in canonicals {
        let upper = canonical.trim().to_uppercase();
        if upper.is_empty() || map.contains_key(&upper) {
            continue;
        }
        if let Some(desc) = registry.short_description(&TopicKey::command(&upper)) {
            map.insert(upper, desc);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_help::{HelpTopic, TopicSource};
    use std::path::PathBuf;

    /// A `cmd:<ROOT>` topic whose `## Description` first paragraph is `desc`.
    fn cmd_topic(root: &str, desc: &str) -> HelpTopic {
        HelpTopic::new(
            TopicKey::command(root),
            format!("{root} Command"),
            format!("## Description\n\n{desc}\n\n## See Also\n"),
            TopicSource::FileBased {
                file_path: PathBuf::from("test.help.md"),
            },
        )
    }

    fn registry_with(topics: Vec<HelpTopic>) -> HelpTopicRegistry {
        let r = HelpTopicRegistry::new();
        r.load_file_topics(topics);
        r
    }

    // Validates: command-configurator Requirement 9.1, 9.5 -- the inherited
    // one-liner for the EXIT root (the X -> EXIT seed's canonical) is the
    // cmd:EXIT Description first paragraph.
    #[test]
    fn inherited_one_liner_for_exit_root() {
        let registry = registry_with(vec![cmd_topic("EXIT", "Exit the application.")]);
        assert_eq!(
            inherited_root_description("EXIT", &registry),
            "Exit the application."
        );
    }

    // Validates: command-configurator Requirement 9.4 -- a root with no
    // cmd:<ROOT> topic yields an empty inherited description (graceful fallback,
    // never a panic).
    #[test]
    fn missing_topic_falls_back_to_empty() {
        let registry = registry_with(vec![cmd_topic("EXIT", "Exit the application.")]);
        assert_eq!(inherited_root_description("NOSUCHROOT", &registry), "");
    }

    // Validates: command-configurator Requirement 9.4 -- a blank canonical yields
    // an empty inherited description (no lookup, never a panic).
    #[test]
    fn blank_canonical_yields_empty() {
        let registry = registry_with(vec![cmd_topic("EXIT", "Exit the application.")]);
        assert_eq!(inherited_root_description("", &registry), "");
        assert_eq!(inherited_root_description("   ", &registry), "");
    }

    // Validates: context-help Requirement 22.2 -- the inherited lookup is keyed by
    // the UPPERCASED canonical, so a lowercase row root still resolves.
    #[test]
    fn inherited_lookup_is_case_insensitive_on_canonical() {
        let registry = registry_with(vec![cmd_topic("EXIT", "Exit the application.")]);
        assert_eq!(
            inherited_root_description("exit", &registry),
            "Exit the application."
        );
    }

    // Validates: command-configurator Requirement 9.1, 9.3 -- the map carries one
    // entry per distinct non-empty root that has a topic, keyed uppercase, and
    // OMITS roots with no topic (the caller shows empty).
    #[test]
    fn map_carries_one_entry_per_root_with_topic() {
        let registry = registry_with(vec![
            cmd_topic("EXIT", "Exit the application."),
            cmd_topic("EDIT", "Open a dataset for editing."),
        ]);
        let map = inherited_description_map(["exit", "EDIT", "edit", "", "NOTOPIC"], &registry);
        assert_eq!(
            map.get("EXIT").map(String::as_str),
            Some("Exit the application.")
        );
        assert_eq!(
            map.get("EDIT").map(String::as_str),
            Some("Open a dataset for editing.")
        );
        assert!(
            !map.contains_key("NOTOPIC"),
            "a root with no topic is omitted"
        );
        assert_eq!(map.len(), 2, "distinct roots with a topic only");
    }
}
