//! Shipped help content set tests (CR-NR-097, context-help Requirement 17).
//!
//! These validate the actual authored `.help.md` files under the repository
//! `help/` directory: every promised Topic_Key is present, there are no dangling
//! cross-references, and the files are ASCII.

use std::path::PathBuf;

use ff_help::{ContentLoader, HelpTopicRegistry, TopicKey};

/// Resolve the repository `help/` directory relative to this crate.
///
/// `ff-help` lives at `crates/ff-help`, so the shipped content is at
/// `../../help` from `CARGO_MANIFEST_DIR`.
fn shipped_help_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("help")
}

/// Load the shipped content into a registry.
fn load_shipped() -> HelpTopicRegistry {
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

/// Keys that are generated dynamically (never file-based) and therefore count as
/// present for cross-reference validation even though they are not in the files.
fn dynamic_keys() -> Vec<TopicKey> {
    vec![TopicKey::index(), TopicKey::feature("function_keys")]
}

// Validates: Requirement 17.2, 17.3, 17.4, 17.5, 17.6, 17.7 -- promised topics present.
#[test]
fn shipped_content_contains_every_promised_topic() {
    let registry = load_shipped();

    let mut promised: Vec<TopicKey> = Vec::new();
    // 17.2 getting_started (index is dynamic)
    promised.push(TopicKey::getting_started());
    // 17.3 primary commands
    for c in [
        "FIND", "RFIND", "CHANGE", "RCHANGE", "EXCLUDE", "SHOW", "RESET", "LOCATE", "SAVE",
        "CANCEL", "END", "UP", "DOWN", "TOP", "BOTTOM", "UNDO", "REDO", "HEX", "HELP", "KEYS",
        "POM", "SETTINGS", "FILES",
    ] {
        promised.push(TopicKey::command(c));
    }
    // 17.4 line commands + line:index
    promised.push(TopicKey::line_index());
    for l in [
        "D", "I", "R", "C", "M", "A", "X", "U", "shift", "COLS", "BNDS", "TABS", "MASK",
    ] {
        promised.push(TopicKey::line_command(l));
    }
    // 17.5 modes
    for m in [
        "browse",
        "edit",
        "view",
        "hex",
        "preview",
        "grid_browse",
        "grid_edit",
    ] {
        promised.push(TopicKey::mode(m));
    }
    // 17.6 + 17.7 features
    for f in [
        "undo",
        "macros",
        "command_history",
        "tabs",
        "docking",
        "configuration",
    ] {
        promised.push(TopicKey::feature(f));
    }

    let missing: Vec<String> = promised
        .iter()
        .filter(|k| !registry.contains(k))
        .map(|k| k.as_str().to_string())
        .collect();
    assert!(
        missing.is_empty(),
        "shipped help is missing promised topics: {missing:?}"
    );
}

// Validates: Requirement 17.8 -- no dangling cross-references.
#[test]
fn shipped_content_has_no_dangling_cross_references() {
    let registry = load_shipped();
    let dynamic = dynamic_keys();

    let mut dangling: Vec<String> = Vec::new();
    for topic in registry.all_topics() {
        for xref in topic.cross_references() {
            let known = registry.contains(&xref) || dynamic.contains(&xref);
            if !known {
                dangling.push(format!("{} -> {}", topic.key().as_str(), xref.as_str()));
            }
        }
    }
    assert!(
        dangling.is_empty(),
        "shipped help has dangling cross-references: {dangling:?}"
    );
}

// Validates: Requirement 17.9 -- shipped content files are ASCII.
#[test]
fn shipped_content_files_are_ascii() {
    let dir = shipped_help_dir();
    let mut offenders: Vec<String> = Vec::new();
    visit_help_files(&dir, &mut |path| {
        let bytes = std::fs::read(path).expect("read help file");
        if let Some(pos) = bytes.iter().position(|b| !b.is_ascii()) {
            offenders.push(format!(
                "{} (first non-ASCII byte at {})",
                path.display(),
                pos
            ));
        }
    });
    assert!(
        offenders.is_empty(),
        "shipped help files must be ASCII: {offenders:?}"
    );
}

/// Recursively visit every `.help.md` file under `dir`.
fn visit_help_files(dir: &std::path::Path, f: &mut dyn FnMut(&std::path::Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_help_files(&path, f);
        } else if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".help.md"))
            .unwrap_or(false)
        {
            f(&path);
        }
    }
}
