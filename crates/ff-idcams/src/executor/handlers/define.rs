//! DEFINE command handlers (CLUSTER / ALTERNATEINDEX / PATH / GDG).

use ff_dscatalog::{
    CatalogError, DatasetAttributes, Dsorg, KeyField, Recfm, VsamError, VsamParams, VsamType,
};

use crate::executor::context::ExecutionState;
use crate::messages::{ConditionCode, MessageCode};
use crate::parser::ast::*;
use crate::services::IdcamsServices;

/// Maps the parsed VSAM organization to the reconciled `VsamType`.
fn vsam_type_for(org: VsamOrganization) -> VsamType {
    match org {
        VsamOrganization::Indexed => VsamType::Ksds,
        VsamOrganization::NonIndexed => VsamType::Esds,
        VsamOrganization::Numbered => VsamType::Rrds,
        VsamOrganization::Linear => VsamType::Lds,
    }
}

/// Executes DEFINE CLUSTER.
///
/// Routes dataset creation through the reconciled `CatalogService::create_dataset`
/// and `VsamService::initialize_dataset` (Requirement 28.3); on VSAM init
/// failure the catalog entry is rolled back via `delete_dataset`
/// (Requirement 22 atomic execution).
pub fn execute_define_cluster(
    cmd: DefineClusterCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    // Validate: INDEXED requires KEYS.
    if cmd.organization == VsamOrganization::Indexed && cmd.keys.is_none() {
        state.emit_message(
            MessageCode::IDC0503E,
            &format!("KEYS PARAMETER REQUIRED FOR INDEXED CLUSTER {}", cmd.name),
        );
        state.set_lastcc(ConditionCode::Severe);
        return;
    }

    // Step 1: create the catalog entry.
    let attrs = DatasetAttributes {
        recfm: Some(Recfm::VB),
        lrecl: cmd.recordsize.map(|(_, max)| max),
        blksize: None,
        dsorg: Some(Dsorg::PS),
        volser: cmd.volumes.first().cloned(),
    };

    if let Err(e) = services.catalog.create_dataset(cmd.name.as_str(), attrs) {
        match e {
            CatalogError::DuplicateDataset { .. } => {
                state.emit_message(
                    MessageCode::IDC0514E,
                    &format!("ENTRY {} ALREADY EXISTS", cmd.name),
                );
            }
            other => {
                state.emit_message(MessageCode::IDC0514E, &format!("CATALOG ERROR: {other}"));
            }
        }
        state.set_lastcc(ConditionCode::Severe);
        return;
    }

    // Step 2: initialize the VSAM dataset.
    let vtype = vsam_type_for(cmd.organization);
    let params = VsamParams {
        key_length: cmd.keys.map(|(len, _)| len),
        key_offset: cmd.keys.map(|(_, off)| off as u16),
        record_length: cmd.recordsize.map(|(_, max)| max),
        slot_size: None,
    };

    if let Err(e) = services
        .vsam
        .initialize_dataset(cmd.name.as_str(), vtype, params)
    {
        // Rollback: remove the catalog entry.
        let _ = services.catalog.delete_dataset(cmd.name.as_str());
        state.emit_message(
            MessageCode::IDC0514E,
            &format!("VSAM INITIALIZATION FAILED: {e}"),
        );
        state.set_lastcc(ConditionCode::Severe);
        return;
    }

    state.emit_message(
        MessageCode::IDC0001I,
        &format!("ENTRY {} DEFINED", cmd.name),
    );
    state.set_lastcc(ConditionCode::Success);
}

/// Executes DEFINE ALTERNATEINDEX.
///
/// Routes through the reconciled `VsamService::define_aix` (Requirement 28.3).
pub fn execute_define_aix(
    cmd: DefineAixCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    let (key_len, key_off) = cmd.keys;
    let key_field = KeyField {
        offset: key_off as u16,
        length: key_len,
        unique: cmd.uniquekey,
    };

    match services
        .vsam
        .define_aix(cmd.relate.as_str(), cmd.name.as_str(), key_field)
    {
        Ok(()) => {
            state.emit_message(MessageCode::IDC0001I, &format!("AIX {} DEFINED", cmd.name));
            state.set_lastcc(ConditionCode::Success);
        }
        Err(VsamError::DatasetNotFound { dsn }) => {
            state.emit_message(
                MessageCode::IDC0510E,
                &format!("BASE CLUSTER {dsn} NOT FOUND"),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(VsamError::UnsupportedOperation { .. }) => {
            state.emit_message(
                MessageCode::IDC0511E,
                &format!("RELATE TARGET {} IS NOT A VSAM CLUSTER", cmd.relate),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0510E, &format!("DEFINE AIX FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes DEFINE PATH.
///
/// `ff-dscatalog` models no PATH trait method, so PATH is performed as an
/// orchestration-only action: the path is registered as a catalog entry via the
/// reconciled `CatalogService::create_dataset`. Behaviour, output, and condition
/// codes are unchanged from the pre-repoint PATH handler (Requirement 28.7).
pub fn execute_define_path(
    cmd: DefinePathCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    // Orchestration-local: no ff-dscatalog PATH trait method. The PATH is a
    // named catalog entry pointing at the AIX (cmd.pathentry); registering it
    // through the reconciled catalog keeps IDCAMS a thin orchestrator.
    let attrs = DatasetAttributes {
        dsorg: Some(Dsorg::PS),
        ..DatasetAttributes::default()
    };
    match services.catalog.create_dataset(cmd.name.as_str(), attrs) {
        Ok(_) => {
            state.emit_message(MessageCode::IDC0001I, &format!("PATH {} DEFINED", cmd.name));
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::DatasetNotFound { .. }) => {
            state.emit_message(
                MessageCode::IDC0512E,
                &format!("PATHENTRY {} NOT FOUND OR NOT AN AIX", cmd.pathentry),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0512E, &format!("DEFINE PATH FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes DEFINE GDG.
///
/// Routes through the reconciled `CatalogService::create_gdg_base`
/// (Requirement 28.3). The reconciled trait carries only `(dsn, limit, scratch)`;
/// the EMPTY / FIFO flags have no reconciled trait parameter and stay
/// IDCAMS-local (parsed, validated, but not forwarded) -- documented here.
pub fn execute_define_gdg(
    cmd: DefineGdgCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    // Validate: LIMIT required.
    if cmd.limit == 0 {
        state.emit_message(
            MessageCode::IDC0520E,
            &format!("LIMIT PARAMETER REQUIRED FOR GDG {}", cmd.name),
        );
        state.set_lastcc(ConditionCode::Severe);
        return;
    }

    match services
        .catalog
        .create_gdg_base(cmd.name.as_str(), cmd.limit, cmd.scratch)
    {
        Ok(()) => {
            state.emit_message(
                MessageCode::IDC0001I,
                &format!("GDG BASE {} DEFINED", cmd.name),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::DuplicateDataset { .. }) => {
            state.emit_message(
                MessageCode::IDC0514E,
                &format!("ENTRY {} ALREADY EXISTS", cmd.name),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0520E, &format!("DEFINE GDG FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}
