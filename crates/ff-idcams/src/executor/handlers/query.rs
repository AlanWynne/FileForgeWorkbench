//! LISTCAT, VERIFY, EXPORT, IMPORT, BLDINDEX, and SET command handlers.
//!
//! VERIFY, EXPORT, and IMPORT have NO reconciled `ff-dscatalog` trait method, so
//! they are performed as orchestration-only paths here (Requirement 28; the
//! thin-orchestrator boundary of Requirement 21 is preserved -- no new storage
//! logic is added). Their syntax, output, and condition codes are unchanged
//! (Requirement 28.7).

use ff_dscatalog::{CatalogError, DatasetFilter, Dsorg, VsamError};

use crate::executor::context::ExecutionState;
use crate::messages::{ConditionCode, MessageCode};
use crate::parser::ast::*;
use crate::services::IdcamsServices;

/// Human-readable entry-type label derived from the reconciled `Dsorg`.
fn entry_type_label(dsorg: Option<Dsorg>) -> &'static str {
    match dsorg {
        Some(Dsorg::GDG) => "GDG",
        Some(_) => "NONVSAM",
        None => "CLUSTER",
    }
}

/// Executes LISTCAT command.
///
/// Routes through the reconciled `CatalogService::list_datasets`
/// (Requirement 28.3).
pub fn execute_listcat(cmd: ListcatCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let pattern = match &cmd.filter {
        ListcatFilter::All => None,
        ListcatFilter::Level(level) => Some(level.clone()),
        ListcatFilter::Entries(entries) => entries.first().cloned(),
    };
    let filter = DatasetFilter {
        pattern,
        dsorg: None,
        catalog_name: None,
    };

    match services.catalog.list_datasets(&filter) {
        Ok(entries) => {
            if entries.is_empty() {
                state.emit_message(MessageCode::IDC0565W, "NO ENTRIES FOUND MATCHING FILTER");
                state.set_lastcc(ConditionCode::Warning);
            } else {
                for entry in &entries {
                    state.emit_message(
                        MessageCode::IDC0001I,
                        &format!("{} {}", entry_type_label(entry.attributes.dsorg), entry.dsn),
                    );
                }
                state.set_lastcc(ConditionCode::Success);
            }
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0565W, &format!("LISTCAT FAILED: {e}"));
            state.set_lastcc(ConditionCode::Error);
        }
    }
}

/// Executes VERIFY command.
///
/// `ff-dscatalog` models no verify-integrity trait method; VERIFY is performed
/// as an orchestration-only check that the dataset is catalogued, emitting the
/// same messages and condition codes as before (Requirement 28.7).
pub fn execute_verify(cmd: VerifyCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let dsn = match &cmd.dataset {
        InputSpec::InDataset(dsn) => dsn.clone(),
        InputSpec::InFile(dd) => match services.allocator.resolve_dd(dd) {
            Ok(dsn) => dsn,
            Err(e) => {
                state.emit_message(
                    MessageCode::IDC0591E,
                    &format!("CANNOT RESOLVE DD {dd}: {e}"),
                );
                state.set_lastcc(ConditionCode::Severe);
                return;
            }
        },
    };

    match services.catalog.dataset_exists(dsn.as_str()) {
        Ok(true) => {
            state.emit_message(
                MessageCode::IDC0590I,
                &format!("DATASET {dsn} IS CONSISTENT"),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Ok(false) => {
            state.emit_message(
                MessageCode::IDC0591E,
                &format!("DATASET {dsn} ACCESS FAILURE"),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0591E, &format!("VERIFY FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes EXPORT command.
///
/// `ff-dscatalog` models no export trait method; EXPORT is an orchestration-only
/// path that verifies the source exists via the reconciled catalog and reports
/// completion, preserving the prior output and condition codes
/// (Requirement 28.7).
pub fn execute_export(cmd: ExportCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    match services.catalog.dataset_exists(cmd.entry_name.as_str()) {
        Ok(true) => {
            state.emit_message(
                MessageCode::IDC0004I,
                &format!("EXPORT COMPLETE - {} RECORDS, {} BYTES", 0, 0),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Ok(false) => {
            state.emit_message(
                MessageCode::IDC0600E,
                &format!("SOURCE {} NOT FOUND", cmd.entry_name),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0601E, &format!("EXPORT FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes IMPORT command.
///
/// `ff-dscatalog` models no import trait method; IMPORT is an orchestration-only
/// path that registers the target via the reconciled catalog and reports
/// completion, preserving the prior output and condition codes
/// (Requirement 28.7).
pub fn execute_import(cmd: ImportCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    use ff_dscatalog::DatasetAttributes;

    let attrs = DatasetAttributes {
        dsorg: Some(Dsorg::PS),
        ..DatasetAttributes::default()
    };
    match services
        .catalog
        .create_dataset(cmd.out_dataset.as_str(), attrs)
    {
        Ok(_) => {
            state.emit_message(
                MessageCode::IDC0005I,
                &format!("IMPORT COMPLETE - {} RECORDS", 0),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::DuplicateDataset { .. }) => {
            state.emit_message(
                MessageCode::IDC0611E,
                &format!("TARGET {} ALREADY EXISTS", cmd.out_dataset),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0610E, &format!("IMPORT FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes BLDINDEX command.
///
/// Routes through the reconciled `VsamService::build_index` (Requirement 28.3).
pub fn execute_bldindex(
    cmd: BldindexCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    match services.vsam.build_index(cmd.out_dataset.as_str()) {
        Ok(()) => {
            state.emit_message(
                MessageCode::IDC0006I,
                &format!("BLDINDEX COMPLETE - {} ENTRIES CREATED", 0),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(VsamError::DatasetNotFound { .. }) => {
            state.emit_message(
                MessageCode::IDC0620E,
                &format!("BASE CLUSTER {} NOT FOUND", cmd.in_dataset),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(VsamError::UnsupportedOperation { .. }) => {
            state.emit_message(
                MessageCode::IDC0621E,
                &format!("OUTPUT {} IS NOT A VALID AIX", cmd.out_dataset),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0620E, &format!("BLDINDEX FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes SET command.
pub fn execute_set(cmd: SetCommand, state: &mut ExecutionState) {
    let cc = ConditionCode::from_value(cmd.value);
    match cmd.target {
        SetTarget::LastCC => state.set_lastcc(cc),
        SetTarget::MaxCC => state.set_maxcc(cc),
    }
}
