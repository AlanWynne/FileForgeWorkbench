//! Architectural Compliance Tests for the Dataset Ownership Model.
//!
//! These tests verify that the dependency direction rules defined in ADR-001
//! and the Dataset Ownership Model governance document are respected by all
//! dataset-related crates in the workspace.
//!
//! Run with: `cargo test -p ff-governance-tests --test architecture_compliance`

use ff_governance_tests::compliance::{
    check_compliance, crate_cargo_toml, crate_exists, parse_cargo_toml_dependencies,
    PROHIBITED_DEPENDENCIES,
};

// ─── Individual Crate Tests ─────────────────────────────────────────────────

// Validates: Requirement 2 AC 3; Requirement 7 AC 3
#[test]
fn vfs_has_no_domain_dependencies() {
    let cargo_toml_path = crate_cargo_toml("ff-vfs");
    assert!(
        cargo_toml_path.exists(),
        "ff-vfs crate must exist at {}",
        cargo_toml_path.display()
    );

    let deps = parse_cargo_toml_dependencies(&cargo_toml_path);

    assert!(
        !deps.contains_key("ff-idcams"),
        "ff-vfs must not depend on ff-idcams (Requirement 2 AC 3)"
    );
    assert!(
        !deps.contains_key("ff-dataset-catalog"),
        "ff-vfs must not depend on ff-dataset-catalog (Requirement 2 AC 3)"
    );
    assert!(
        !deps.contains_key("ff-dsalloc"),
        "ff-vfs must not depend on ff-dsalloc (Requirement 2 AC 3)"
    );
    assert!(
        !deps.contains_key("ff-vsam-services"),
        "ff-vfs must not depend on ff-vsam-services (Requirement 2 AC 3)"
    );
}

// Validates: Requirement 3 AC 3; Requirement 7 AC 3
#[test]
fn dataset_catalog_has_no_upstream_dependencies() {
    let cargo_toml_path = crate_cargo_toml("ff-dataset-catalog");
    if !cargo_toml_path.exists() {
        // Crate not yet created — test will pass once it exists
        eprintln!(
            "SKIPPED: ff-dataset-catalog does not exist yet at {}",
            cargo_toml_path.display()
        );
        return;
    }

    let deps = parse_cargo_toml_dependencies(&cargo_toml_path);

    assert!(
        !deps.contains_key("ff-idcams"),
        "ff-dataset-catalog must not depend on ff-idcams (Requirement 3 AC 3)"
    );
    assert!(
        !deps.contains_key("ff-dsalloc"),
        "ff-dataset-catalog must not depend on ff-dsalloc (Requirement 3 AC 3)"
    );
}

// Validates: Requirement 5 AC 3; Requirement 7 AC 3
#[test]
fn vsam_services_has_no_upstream_dependencies() {
    let cargo_toml_path = crate_cargo_toml("ff-vsam-services");
    if !cargo_toml_path.exists() {
        eprintln!(
            "SKIPPED: ff-vsam-services does not exist yet at {}",
            cargo_toml_path.display()
        );
        return;
    }

    let deps = parse_cargo_toml_dependencies(&cargo_toml_path);

    assert!(
        !deps.contains_key("ff-idcams"),
        "ff-vsam-services must not depend on ff-idcams (Requirement 5 AC 3)"
    );
    assert!(
        !deps.contains_key("ff-dsalloc"),
        "ff-vsam-services must not depend on ff-dsalloc (Requirement 5 AC 3)"
    );
}

// Validates: Requirement 7 AC 5
#[test]
fn dataset_allocator_has_no_idcams_dependency() {
    let cargo_toml_path = crate_cargo_toml("ff-dsalloc");
    assert!(
        cargo_toml_path.exists(),
        "ff-dsalloc crate must exist at {}",
        cargo_toml_path.display()
    );

    let deps = parse_cargo_toml_dependencies(&cargo_toml_path);

    assert!(
        !deps.contains_key("ff-idcams"),
        "ff-dsalloc must not depend on ff-idcams — allocation is a lower-level service \
         that IDCAMS orchestrates, not the reverse (Requirement 7 AC 5)"
    );
}

// Validates: Requirement 6 AC 3; Requirement 7 AC 3
#[test]
fn idcams_has_no_storage_engine_dependencies() {
    let cargo_toml_path = crate_cargo_toml("ff-idcams");
    if !cargo_toml_path.exists() {
        eprintln!(
            "SKIPPED: ff-idcams does not exist yet at {}",
            cargo_toml_path.display()
        );
        return;
    }

    let deps = parse_cargo_toml_dependencies(&cargo_toml_path);

    // Direct storage engine dependencies that MUST NOT appear
    assert!(
        !deps.contains_key("rusqlite"),
        "ff-idcams must not depend on rusqlite — all catalog access flows through \
         CatalogService trait (Requirement 6 AC 3)"
    );
    assert!(
        !deps.contains_key("rocksdb"),
        "ff-idcams must not depend on rocksdb (Requirement 6 AC 3)"
    );
    assert!(
        !deps.contains_key("lmdb"),
        "ff-idcams must not depend on lmdb (Requirement 6 AC 3)"
    );
    assert!(
        !deps.contains_key("lmdb-rkv"),
        "ff-idcams must not depend on lmdb-rkv (Requirement 6 AC 3)"
    );
    assert!(
        !deps.contains_key("sled"),
        "ff-idcams must not depend on sled (Requirement 6 AC 3)"
    );
}

// ─── Full Compliance Check ──────────────────────────────────────────────────

// Validates: Requirement 18 AC 1, AC 2, AC 5
#[test]
fn full_dependency_compliance_check() {
    let result = check_compliance();

    if !result.passed {
        let violation_report: String = result
            .violations
            .iter()
            .enumerate()
            .map(|(i, v)| format!("\n  {}. {}", i + 1, v))
            .collect();

        panic!(
            "ARCHITECTURAL COMPLIANCE FAILURE\n\
             ================================\n\
             {} violation(s) detected across {} crates ({} rules checked):\n{}\n\n\
             These violations break the Dataset Ownership Model (ADR-001).\n\
             Fix each violation by removing the prohibited dependency and using \
             the appropriate trait interface instead.",
            result.violations.len(),
            result.crates_checked,
            result.rules_checked,
            violation_report
        );
    }

    eprintln!(
        "✅ All {} dependency direction rules pass across {} crates.",
        result.rules_checked, result.crates_checked
    );
}

// ─── Rule Coverage Tests ────────────────────────────────────────────────────

// Validates: Requirement 18 AC 3
#[test]
fn prohibited_rules_cover_all_dataset_crates() {
    // Verify that all dataset-related crates have at least one prohibition rule
    let dataset_crates = &[
        "ff-vfs",
        "ff-dataset-catalog",
        "ff-dsalloc",
        "ff-vsam-services",
        "ff-idcams",
    ];

    for crate_name in dataset_crates {
        // Each crate should either have a rule as the source (it's prohibited from
        // depending on something) or as a target (something is prohibited from depending on it)
        let has_rule = PROHIBITED_DEPENDENCIES
            .iter()
            .any(|r| r.crate_name == *crate_name || r.prohibited_dependency == *crate_name);

        assert!(
            has_rule,
            "Dataset-related crate '{}' has no dependency direction rules — \
             update PROHIBITED_DEPENDENCIES to include it (Requirement 18 AC 3)",
            crate_name
        );
    }
}

// Validates: Requirement 12 AC 3
#[test]
fn dataset_allocator_has_no_rusqlite_dependency() {
    let cargo_toml_path = crate_cargo_toml("ff-dsalloc");
    if !cargo_toml_path.exists() {
        return;
    }

    let deps = parse_cargo_toml_dependencies(&cargo_toml_path);

    assert!(
        !deps.contains_key("rusqlite"),
        "ff-dsalloc must not depend on rusqlite — all catalog access flows through \
         CatalogService trait (Requirement 12 AC 3)"
    );
}

// ─── Violation Reporting Quality ────────────────────────────────────────────

// Validates: Requirement 18 AC 1, AC 2, AC 5
#[test]
fn violation_reports_include_fix_instructions() {
    // Verify that every rule produces a meaningful fix instruction when violated
    for rule in PROHIBITED_DEPENDENCIES {
        assert!(
            !rule.reason.is_empty(),
            "Rule ({} → {}) must have a non-empty reason",
            rule.crate_name,
            rule.prohibited_dependency
        );
        assert!(
            !rule.requirement_ref.is_empty(),
            "Rule ({} → {}) must reference a governance requirement",
            rule.crate_name,
            rule.prohibited_dependency
        );
    }
}

// ─── Crate Existence Verification ──────────────────────────────────────────

#[test]
fn all_governed_crates_exist() {
    // Verify that the fundamental dataset-related crates exist.
    // This test ensures that governance rules can be checked.
    let required_crates = &[
        ("ff-vfs", true),
        ("ff-dataset-catalog", true),
        ("ff-vsam-services", true),
        ("ff-dsalloc", true),
        ("ff-idcams", true),
    ];

    for (crate_name, required) in required_crates {
        if *required {
            assert!(
                crate_exists(crate_name),
                "Governed crate '{}' must exist for compliance testing. \
                 Create it as a stub if implementation is not yet complete.",
                crate_name
            );
        }
    }
}
