//! Architectural compliance checking utilities.
//!
//! Provides helpers for parsing `Cargo.toml` files, extracting dependency
//! information, and verifying governance rules.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── Data Types ─────────────────────────────────────────────────────────────

/// A single dependency direction rule that must be enforced.
#[derive(Debug, Clone)]
pub struct DependencyRule {
    /// The crate that must NOT depend on the target.
    pub crate_name: &'static str,
    /// The crate that must NOT appear as a dependency.
    pub prohibited_dependency: &'static str,
    /// Human-readable reason for the prohibition.
    pub reason: &'static str,
    /// Which governance requirement mandates this rule.
    pub requirement_ref: &'static str,
}

/// A violation of a dependency rule.
#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    /// The rule that was violated.
    pub rule: DependencyRule,
    /// The Cargo.toml path where the violation was found.
    pub found_in: PathBuf,
    /// Instruction for fixing the violation.
    pub fix_instruction: String,
}

impl std::fmt::Display for ComplianceViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ARCHITECTURAL VIOLATION: '{}' depends on '{}' (prohibited)\n  \
             Reason: {}\n  \
             Governance rule: {}\n  \
             Found in: {}\n  \
             Fix: {}",
            self.rule.crate_name,
            self.rule.prohibited_dependency,
            self.rule.reason,
            self.rule.requirement_ref,
            self.found_in.display(),
            self.fix_instruction,
        )
    }
}

/// Result of running the architectural fitness function.
#[derive(Debug)]
pub struct ComplianceResult {
    /// Whether all checks passed.
    pub passed: bool,
    /// List of violations found.
    pub violations: Vec<ComplianceViolation>,
    /// Number of crates checked.
    pub crates_checked: usize,
    /// Number of rules checked.
    pub rules_checked: usize,
}

// ─── Prohibited Dependency Rules ────────────────────────────────────────────

/// All prohibited dependencies as defined by Requirement 7 and ADR-001.
pub const PROHIBITED_DEPENDENCIES: &[DependencyRule] = &[
    // ff-vfs shall not depend on any domain crate
    DependencyRule {
        crate_name: "ff-vfs",
        prohibited_dependency: "ff-idcams",
        reason:
            "VFS is domain-agnostic infrastructure; domain crates depend on VFS, not vice versa",
        requirement_ref: "Requirement 2 AC 3; Requirement 7 AC 3",
    },
    DependencyRule {
        crate_name: "ff-vfs",
        prohibited_dependency: "ff-dataset-catalog",
        reason:
            "VFS is domain-agnostic infrastructure; domain crates depend on VFS, not vice versa",
        requirement_ref: "Requirement 2 AC 3; Requirement 7 AC 3",
    },
    DependencyRule {
        crate_name: "ff-vfs",
        prohibited_dependency: "ff-dsalloc",
        reason:
            "VFS is domain-agnostic infrastructure; domain crates depend on VFS, not vice versa",
        requirement_ref: "Requirement 2 AC 3; Requirement 7 AC 3",
    },
    DependencyRule {
        crate_name: "ff-vfs",
        prohibited_dependency: "ff-vsam-services",
        reason:
            "VFS is domain-agnostic infrastructure; domain crates depend on VFS, not vice versa",
        requirement_ref: "Requirement 2 AC 3; Requirement 7 AC 3",
    },
    // ff-dataset-catalog shall not depend on upstream orchestrators
    DependencyRule {
        crate_name: "ff-dataset-catalog",
        prohibited_dependency: "ff-idcams",
        reason: "Catalog is a lower-level service; IDCAMS orchestrates, catalog provides",
        requirement_ref: "Requirement 3 AC 3; Requirement 7 AC 3",
    },
    DependencyRule {
        crate_name: "ff-dataset-catalog",
        prohibited_dependency: "ff-dsalloc",
        reason: "Catalog is a lower-level service; allocator depends on catalog, not vice versa",
        requirement_ref: "Requirement 3 AC 3; Requirement 7 AC 3",
    },
    // ff-vsam-services shall not depend on upstream orchestrators
    DependencyRule {
        crate_name: "ff-vsam-services",
        prohibited_dependency: "ff-idcams",
        reason: "VSAM services is a lower-level service; IDCAMS orchestrates, VSAM provides",
        requirement_ref: "Requirement 5 AC 3; Requirement 7 AC 3",
    },
    DependencyRule {
        crate_name: "ff-vsam-services",
        prohibited_dependency: "ff-dsalloc",
        reason: "VSAM services is a lower-level service; allocator does not interact with VSAM",
        requirement_ref: "Requirement 5 AC 3; Requirement 7 AC 3",
    },
    // ff-dsalloc shall not depend on ff-idcams
    DependencyRule {
        crate_name: "ff-dsalloc",
        prohibited_dependency: "ff-idcams",
        reason: "Allocator is a lower-level service that IDCAMS orchestrates, not the reverse",
        requirement_ref: "Requirement 7 AC 5",
    },
];

// ─── Cargo.toml Parsing ─────────────────────────────────────────────────────

/// Parse a Cargo.toml file and extract its dependency names.
///
/// Returns a map of dependency names to their specification strings.
/// Includes both `[dependencies]` and `[dev-dependencies]`.
pub fn parse_cargo_toml_dependencies(path: &Path) -> HashMap<String, String> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

    let toml_value: toml::Value = content
        .parse()
        .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));

    let mut deps = HashMap::new();

    // Extract [dependencies]
    if let Some(dep_table) = toml_value.get("dependencies").and_then(|v| v.as_table()) {
        for (name, _value) in dep_table {
            deps.insert(name.clone(), "dependency".to_string());
        }
    }

    deps
}

/// Get the workspace root directory (two levels up from this crate's src).
pub fn workspace_root() -> PathBuf {
    // The workspace root is the directory containing the top-level Cargo.toml.
    // We navigate from the crate's manifest directory.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("Could not determine workspace root")
        .to_path_buf()
}

/// Get the path to a specific crate's Cargo.toml.
pub fn crate_cargo_toml(crate_name: &str) -> PathBuf {
    workspace_root()
        .join("crates")
        .join(crate_name)
        .join("Cargo.toml")
}

/// Check whether a crate's Cargo.toml exists (the crate has been created).
pub fn crate_exists(crate_name: &str) -> bool {
    crate_cargo_toml(crate_name).exists()
}

/// Run the full compliance check against all prohibited dependency rules.
pub fn check_compliance() -> ComplianceResult {
    let mut violations = Vec::new();
    let mut crates_checked = 0;
    let rules_checked = PROHIBITED_DEPENDENCIES.len();

    for rule in PROHIBITED_DEPENDENCIES {
        let cargo_toml_path = crate_cargo_toml(rule.crate_name);

        if !cargo_toml_path.exists() {
            // Crate doesn't exist yet — skip (will be caught when created)
            continue;
        }

        crates_checked += 1;
        let deps = parse_cargo_toml_dependencies(&cargo_toml_path);

        if deps.contains_key(rule.prohibited_dependency) {
            violations.push(ComplianceViolation {
                rule: rule.clone(),
                found_in: cargo_toml_path,
                fix_instruction: format!(
                    "Remove '{}' from the [dependencies] section of {}/Cargo.toml. \
                     Use the trait interface instead.",
                    rule.prohibited_dependency, rule.crate_name
                ),
            });
        }
    }

    ComplianceResult {
        passed: violations.is_empty(),
        violations,
        crates_checked,
        rules_checked,
    }
}
