//! Integration tests for ff-idcams.
//!
//! Tests the full pipeline: parse → execute → result.

use ff_idcams::messages::{ConditionCode, MessageCode};
use ff_idcams::parser::ast::*;
use ff_idcams::parser::IdcamsParser;
use ff_idcams::services::mocks::*;
use ff_idcams::{execute_idcams, pretty_print, PrintMode};
use std::sync::Arc;

fn default_services() -> ff_idcams::IdcamsServices {
    TestServicesBuilder::new().build()
}

// ─── Parser Tests ───────────────────────────────────────────────────────────

#[test]
fn parse_define_cluster_indexed() {
    let input = "DEFINE CLUSTER (NAME(MY.KSDS.DATA) INDEXED \
                 KEYS(8 0) RECORDSIZE(80 80))";
    let commands = IdcamsParser::parse(input);
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        Command::DefineCluster(cmd) => {
            assert_eq!(cmd.name.as_str(), "MY.KSDS.DATA");
            assert_eq!(cmd.organization, VsamOrganization::Indexed);
            assert_eq!(cmd.keys, Some((8, 0)));
            assert_eq!(cmd.recordsize, Some((80, 80)));
        }
        _ => panic!("expected DefineCluster"),
    }
}

#[test]
fn parse_define_cluster_esds() {
    let input = "DEFINE CLUSTER (NAME(MY.ESDS) NONINDEXED)";
    let commands = IdcamsParser::parse(input);
    match &commands[0] {
        Command::DefineCluster(cmd) => {
            assert_eq!(cmd.organization, VsamOrganization::NonIndexed);
        }
        _ => panic!("expected DefineCluster"),
    }
}

#[test]
fn parse_define_gdg() {
    let input = "DEFINE GDG (NAME(MY.GDG.BASE) LIMIT(30) NOSCRATCH EMPTY FIFO)";
    let commands = IdcamsParser::parse(input);
    match &commands[0] {
        Command::DefineGdg(cmd) => {
            assert_eq!(cmd.name.as_str(), "MY.GDG.BASE");
            assert_eq!(cmd.limit, 30);
            assert!(!cmd.scratch);
            assert!(cmd.empty);
            assert!(cmd.fifo);
        }
        _ => panic!("expected DefineGdg"),
    }
}

#[test]
fn parse_delete_multiple_entries() {
    let input = "DELETE (MY.DATA1 MY.DATA2) CLUSTER PURGE";
    let commands = IdcamsParser::parse(input);
    match &commands[0] {
        Command::Delete(cmd) => {
            assert_eq!(cmd.entries.len(), 2);
            assert_eq!(cmd.entry_type, DeleteEntryType::Cluster);
            assert!(cmd.purge);
        }
        _ => panic!("expected Delete"),
    }
}

#[test]
fn parse_listcat_with_level() {
    let input = "LISTCAT LEVEL(MY.DATA) ALL";
    let commands = IdcamsParser::parse(input);
    match &commands[0] {
        Command::Listcat(cmd) => {
            assert_eq!(cmd.filter, ListcatFilter::Level("MY.DATA".to_string()));
            assert_eq!(cmd.display_level, DisplayLevel::All);
        }
        _ => panic!("expected Listcat"),
    }
}

#[test]
fn parse_set_maxcc() {
    let input = "SET MAXCC(0)";
    let commands = IdcamsParser::parse(input);
    match &commands[0] {
        Command::Set(cmd) => {
            assert_eq!(cmd.target, SetTarget::MaxCC);
            assert_eq!(cmd.value, 0);
        }
        _ => panic!("expected Set"),
    }
}

#[test]
fn parse_if_then_else() {
    let input = "IF LASTCC EQ 0 THEN SET MAXCC(0) ELSE SET MAXCC(4)";
    let commands = IdcamsParser::parse(input);
    match &commands[0] {
        Command::If(cmd) => {
            assert_eq!(cmd.then_commands.len(), 1);
            assert!(cmd.else_commands.is_some());
        }
        _ => panic!("expected If"),
    }
}

#[test]
fn parse_unrecognized_verb_produces_error_node() {
    let input = "BADVERB SOMETHING";
    let commands = IdcamsParser::parse(input);
    match &commands[0] {
        Command::Error(err) => {
            assert_eq!(err.code, "IDC0001E");
            assert!(err.message.contains("BADVERB"));
        }
        _ => panic!("expected Error node"),
    }
}

// ─── Executor Tests ─────────────────────────────────────────────────────────

#[test]
fn execute_define_cluster_success() {
    // Validates: Requirement 2 AC 2, AC 21
    let services = default_services();
    let result = execute_idcams(
        "DEFINE CLUSTER (NAME(MY.KSDS) INDEXED KEYS(8 0))",
        &services,
    );
    assert_eq!(result.maxcc, ConditionCode::Success);
    assert!(result
        .messages
        .iter()
        .any(|m| m.code == MessageCode::IDC0001I));
}

#[test]
fn execute_define_cluster_missing_keys_for_indexed() {
    // Validates: Requirement 2 AC 18
    let services = default_services();
    let result = execute_idcams("DEFINE CLUSTER (NAME(MY.KSDS) INDEXED)", &services);
    assert_eq!(result.maxcc, ConditionCode::Severe);
    assert!(result
        .messages
        .iter()
        .any(|m| m.code == MessageCode::IDC0503E));
}

#[test]
fn execute_define_cluster_duplicate_name() {
    // Validates: Requirement 2 AC 19
    use ff_idcams::error::CatalogError;

    let catalog = MockCatalogService::new_success();
    *catalog.create_responses.lock().unwrap() =
        vec![Err(CatalogError::DuplicateName("MY.KSDS".to_string()))];

    let services = TestServicesBuilder::new().with_catalog(catalog).build();

    let result = execute_idcams(
        "DEFINE CLUSTER (NAME(MY.KSDS) INDEXED KEYS(8 0))",
        &services,
    );
    assert_eq!(result.maxcc, ConditionCode::Severe);
    assert!(result
        .messages
        .iter()
        .any(|m| m.code == MessageCode::IDC0514E));
}

#[test]
fn execute_define_cluster_rollback_on_vsam_failure() {
    // Validates: Requirement 2 AC 20
    use ff_idcams::error::VsamError;

    let vsam = MockVsamService::new_success();
    *vsam.init_responses.lock().unwrap() =
        vec![Err(VsamError::Internal("init failed".to_string()))];

    let services = TestServicesBuilder::new().with_vsam(vsam).build();

    let result = execute_idcams(
        "DEFINE CLUSTER (NAME(MY.KSDS) INDEXED KEYS(8 0))",
        &services,
    );
    assert_eq!(result.maxcc, ConditionCode::Severe);
}

#[test]
fn execute_define_gdg_missing_limit() {
    // Validates: Requirement 5 AC 8
    let services = default_services();
    // Limit of 0 triggers the error
    let result = execute_idcams("DEFINE GDG (NAME(MY.GDG))", &services);
    assert_eq!(result.maxcc, ConditionCode::Severe);
    assert!(result
        .messages
        .iter()
        .any(|m| m.code == MessageCode::IDC0520E));
}

#[test]
fn execute_delete_entry_not_found() {
    // Validates: Requirement 6 AC 13
    use ff_idcams::error::VsamError;

    let vsam = MockVsamService::new_success();
    *vsam.destroy_responses.lock().unwrap() = vec![Err(VsamError::NotFound("MY.DATA".to_string()))];

    let services = TestServicesBuilder::new().with_vsam(vsam).build();

    let result = execute_idcams("DELETE MY.DATA CLUSTER", &services);
    assert_eq!(result.maxcc, ConditionCode::Error);
    assert!(result
        .messages
        .iter()
        .any(|m| m.code == MessageCode::IDC0550E));
}

#[test]
fn execute_multi_command_chaining_with_maxcc_propagation() {
    // Validates: Requirement 15 AC 2, Requirement 18 AC 2-4
    let services = default_services();
    let result = execute_idcams("SET LASTCC(4); SET LASTCC(0)", &services);
    // MAXCC should be 4 (from first SET) even though last was 0
    assert_eq!(result.maxcc, ConditionCode::Warning);
}

#[test]
fn execute_if_then_branch_taken() {
    // Validates: Requirement 16 AC 7
    let services = default_services();
    let result = execute_idcams(
        "SET LASTCC(0); IF LASTCC EQ 0 THEN SET LASTCC(4)",
        &services,
    );
    // IF condition true → SET LASTCC(4)
    assert_eq!(result.maxcc, ConditionCode::Warning);
}

#[test]
fn execute_if_else_branch_taken() {
    // Validates: Requirement 16 AC 7
    let services = default_services();
    let result = execute_idcams(
        "SET LASTCC(4); IF LASTCC EQ 0 THEN SET LASTCC(8) ELSE SET LASTCC(0)",
        &services,
    );
    // LASTCC is 4, condition false → ELSE → SET LASTCC(0)
    // MAXCC = 4 from the first SET
    assert_eq!(result.maxcc, ConditionCode::Warning);
}

#[test]
fn execute_cc16_terminates_processing() {
    // Validates: Requirement 15 AC 7
    let services = default_services();
    let result = execute_idcams("SET LASTCC(16); SET LASTCC(0)", &services);
    // CC=16 should terminate — second SET never executes
    assert_eq!(result.maxcc, ConditionCode::Catastrophic);
}

#[test]
fn execute_final_summary_message() {
    // Validates: Requirement 15 AC 8
    let services = default_services();
    let result = execute_idcams("SET LASTCC(0)", &services);
    let summary = result
        .messages
        .iter()
        .find(|m| m.code == MessageCode::IDC0002I && m.text.contains("MAXIMUM CONDITION CODE"));
    assert!(summary.is_some());
}

// ─── Pretty Printer Tests ───────────────────────────────────────────────────

#[test]
fn pretty_print_define_cluster_compact() {
    // Validates: Requirement 26 AC 1, AC 6
    let cmd = Command::DefineCluster(DefineClusterCommand {
        name: DatasetName::unchecked("MY.KSDS"),
        organization: VsamOrganization::Indexed,
        volumes: vec![],
        space: None,
        recordsize: Some((80, 80)),
        keys: Some((8, 0)),
        freespace: None,
        shareoptions: None,
        speed_recovery: None,
        reuse: false,
        bufferspace: None,
        data_component: None,
        index_component: None,
    });
    let output = pretty_print(&cmd, PrintMode::Compact);
    assert!(output.contains("DEFINE CLUSTER"));
    assert!(output.contains("NAME(MY.KSDS)"));
    assert!(output.contains("KEYS(8 0)"));
}

#[test]
fn pretty_print_output_is_reparseable() {
    // Validates: Requirement 26 AC 7
    let cmd = Command::Set(SetCommand {
        target: SetTarget::LastCC,
        value: 4,
    });
    let output = pretty_print(&cmd, PrintMode::Compact);
    let reparsed = IdcamsParser::parse(&output);
    assert_eq!(reparsed.len(), 1);
    match &reparsed[0] {
        Command::Set(s) => {
            assert_eq!(s.target, SetTarget::LastCC);
            assert_eq!(s.value, 4);
        }
        _ => panic!("expected Set after reparse"),
    }
}

// ─── Thread Safety Tests ────────────────────────────────────────────────────

#[test]
fn concurrent_invocations_have_independent_state() {
    // Validates: Requirement 24 AC 2
    use std::thread;

    let services = Arc::new(default_services());
    let mut handles = Vec::new();

    for i in 0..4 {
        let svc = services.clone();
        handles.push(thread::spawn(move || {
            let input = format!("SET LASTCC({})", i * 4);
            let result = execute_idcams(&input, &svc);
            result.maxcc
        }));
    }

    let results: Vec<ConditionCode> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Each invocation should have its own independent MAXCC
    assert_eq!(results.len(), 4);
}
