//! FFCMD root-command help enforcement (CR-NR-119, context-help Req 23).
//!
//! The single-source-of-truth rule: EVERY FFCMD root command in the enforced
//! scope -- the union `EnvironmentRegistry::root_commands("FFCMD")` enumerates --
//! MUST have a conforming `cmd:<ROOT>` help topic shipped under the `help/` tree,
//! and that topic's `## Description` section MUST yield a valid one-line
//! description (non-empty, within the Req 21.4 length limit).
//!
//! This test reads the ACTUAL shipped `help/` tree (resolved relative to this
//! crate) and loads it with the same `ff_help::ContentLoader` pipeline the shell
//! uses, so authoring a new FFCMD root without its help file breaks the suite
//! (Req 23.2, 23.3).
//!
//! Scope boundaries (Req 23.4, 23.5): FFEDIT roots are NOT asserted here (they
//! conform later, on the roadmap); extension commands are NEVER asserted; and no
//! registration-time validation of extension help exists -- this is an
//! authoring-time test, not a runtime gate.

use ff_help::{ContentLoader, HelpTopicRegistry, TopicKey};

use super::environment_registry::EnvironmentRegistry;

/// Resolve the repository `help/` directory relative to this crate.
///
/// `ff-desktop` lives at `crates/ff-desktop`, so the shipped content is at
/// `../../help` from `CARGO_MANIFEST_DIR`.
fn shipped_help_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("help")
}

/// Load the shipped English help content into a registry, the same way the
/// shell's `build_help_registry_for_locale` does (ContentLoader + file topics).
fn load_shipped_registry() -> HelpTopicRegistry {
    let dir = shipped_help_dir();
    assert!(
        dir.is_dir(),
        "shipped help directory must exist at {}",
        dir.display()
    );
    let loader = ContentLoader::new(vec![dir]);
    let result = loader.load_all().expect("shipped help must load");
    assert!(
        result.warnings.is_empty(),
        "shipped help must parse without warnings: {:?}",
        result.warnings
    );
    let registry = HelpTopicRegistry::new();
    registry.load_file_topics(result.topics);
    registry
}

// Validates: context-help Requirement 23.1, 23.2, 23.3 -- every FFCMD root in the
// enforced scope has a conforming cmd:<ROOT> topic with a valid one-liner.
#[test]
fn every_ffcmd_root_has_conforming_help_topic_and_one_liner() {
    let registry = load_shipped_registry();
    let roots = EnvironmentRegistry::root_commands("FFCMD");
    assert!(
        !roots.is_empty(),
        "root_commands(FFCMD) must enumerate the built-in FFCMD root set"
    );

    let mut missing_topic: Vec<String> = Vec::new();
    let mut missing_one_liner: Vec<String> = Vec::new();

    for root in &roots {
        let key = TopicKey::command(root);
        if !registry.contains(&key) {
            missing_topic.push(root.clone());
            continue;
        }
        // Req 21.3/21.4: the derived one-liner must be a valid non-empty string.
        if registry.short_description(&key).is_none() {
            missing_one_liner.push(root.clone());
        }
    }

    assert!(
        missing_topic.is_empty(),
        "FFCMD roots without a cmd:<ROOT> help topic (Req 23.2a): {missing_topic:?}"
    );
    assert!(
        missing_one_liner.is_empty(),
        "FFCMD roots whose cmd:<ROOT> topic has no valid one-line description \
         (Req 23.2b / 21.3-21.4): {missing_one_liner:?}"
    );
}

// Validates: context-help Requirement 23.4 -- FFEDIT roots are OUT of the
// enforced scope: this test enumerates ONLY the FFCMD environment, never FFEDIT.
#[test]
fn ffedit_roots_are_not_enforced_by_this_check() {
    let ffedit = EnvironmentRegistry::root_commands("FFEDIT");
    let ffcmd = EnvironmentRegistry::root_commands("FFCMD");
    // FFEDIT has its own canonical verbs; this enforcement reads the FFCMD set
    // only. A canonical FFEDIT-only verb (e.g. the in-buffer renumber verbs)
    // must NOT be part of the enforced FFCMD scope.
    assert!(
        ffedit.iter().any(|v| !ffcmd.contains(v)),
        "FFEDIT must expose at least one canonical verb outside the enforced \
         FFCMD scope (sanity: the two root sets are distinct)"
    );
}
