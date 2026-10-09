//! DELETE and ALTER command handlers.

use ff_dscatalog::{
    AccessIntent, CatalogError, DatasetAttributes, Dsorg, Recfm, StepOutcome, VsamError,
};

use crate::executor::context::ExecutionState;
use crate::messages::{ConditionCode, MessageCode};
use crate::parser::ast::*;
use crate::services::IdcamsServices;

/// Thread the record-aware `DatasetAccess::dispose` into a DELETE where the DSN
/// resolves to a handle (Requirement 28.5). Best-effort: a DSN that was not
/// allocated through the access layer resolves to `NotFound`, which is ignored
/// -- the authoritative catalog removal still flows through
/// `CatalogService::delete_dataset`. This keeps behaviour and condition codes
/// unchanged (Requirement 28.7) while routing physical teardown through the
/// single record-aware contract when a handle exists.
fn dispose_via_access(services: &IdcamsServices, dsn: &str) {
    if let Ok(handle) = services.access.resolve(dsn, AccessIntent::Update) {
        let _ = services.access.dispose(handle, StepOutcome::Delete);
    }
}

/// Executes DELETE command.
///
/// VSAM teardown flows through the reconciled `VsamService::destroy_dataset` and
/// `DatasetAccess::dispose`; the catalog entry removal flows through the
/// reconciled `CatalogService::delete_dataset` (Requirement 28.5).
pub fn execute_delete(cmd: DeleteCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    for entry in &cmd.entries {
        let dsn = entry.as_str();
        match cmd.entry_type {
            DeleteEntryType::Cluster | DeleteEntryType::AlternateIndex => {
                match services.vsam.destroy_dataset(dsn) {
                    Ok(()) => {
                        dispose_via_access(services, dsn);
                        if let Err(e) = services.catalog.delete_dataset(dsn) {
                            state.emit_message(
                                MessageCode::IDC0700W,
                                &format!(
                                    "VSAM DESTROYED BUT CATALOG DELETE FAILED FOR {entry}: {e}"
                                ),
                            );
                            state.set_lastcc(ConditionCode::Severe);
                            continue;
                        }
                    }
                    Err(VsamError::DatasetNotFound { .. }) => {
                        state.emit_message(
                            MessageCode::IDC0550E,
                            &format!("ENTRY {entry} NOT FOUND"),
                        );
                        state.set_lastcc(ConditionCode::Error);
                        continue;
                    }
                    Err(e) => {
                        state.emit_message(
                            MessageCode::IDC0551E,
                            &format!("DELETE FAILED FOR {entry}: {e}"),
                        );
                        state.set_lastcc(ConditionCode::Severe);
                        continue;
                    }
                }
            }
            DeleteEntryType::Path => {
                // PATH has no reconciled VSAM trait method; remove the catalog
                // entry directly (orchestration-only, Requirement 28.5).
                if let Err(e) = services.catalog.delete_dataset(dsn) {
                    match e {
                        CatalogError::DatasetNotFound { .. } => {
                            state.emit_message(
                                MessageCode::IDC0550E,
                                &format!("PATH {entry} NOT FOUND: {e}"),
                            );
                            state.set_lastcc(ConditionCode::Error);
                        }
                        _ => {
                            state.emit_message(
                                MessageCode::IDC0700W,
                                &format!("PATH CATALOG REMOVE FAILED: {e}"),
                            );
                            state.set_lastcc(ConditionCode::Severe);
                        }
                    }
                    continue;
                }
            }
            DeleteEntryType::Gdg | DeleteEntryType::NonVsam | DeleteEntryType::UserCatalog => {
                // The reconciled catalog models GDG-base removal as a plain
                // delete_dataset of the base (no delete_gdg_base method);
                // documented orchestration mapping (Requirement 28.5).
                if let Err(e) = services.catalog.delete_dataset(dsn) {
                    match e {
                        CatalogError::DatasetNotFound { .. } => {
                            state.emit_message(
                                MessageCode::IDC0550E,
                                &format!("ENTRY {entry} NOT FOUND"),
                            );
                            state.set_lastcc(ConditionCode::Error);
                        }
                        _ => {
                            state.emit_message(
                                MessageCode::IDC0551E,
                                &format!("DELETE FAILED: {e}"),
                            );
                            state.set_lastcc(ConditionCode::Severe);
                        }
                    }
                    continue;
                }
            }
        }

        // Success for this entry.
        state.emit_message(MessageCode::IDC0002I, &format!("ENTRY {entry} DELETED"));
        state.set_lastcc(ConditionCode::Success);
    }
}

/// Executes ALTER command.
///
/// Rename flows through `CatalogService::rename_dataset`; attribute updates
/// through `CatalogService::update_dataset` (Requirement 28.3).
pub fn execute_alter(cmd: AlterCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let entry = cmd.entry_name.as_str();

    // Handle NEWNAME separately.
    if let Some(ref new_name) = cmd.newname {
        match services.catalog.rename_dataset(entry, new_name.as_str()) {
            Ok(()) => {}
            Err(CatalogError::DatasetNotFound { .. }) => {
                state.emit_message(
                    MessageCode::IDC0560E,
                    &format!("ENTRY {} NOT FOUND", cmd.entry_name),
                );
                state.set_lastcc(ConditionCode::Error);
                return;
            }
            Err(e) => {
                state.emit_message(MessageCode::IDC0561E, &format!("RENAME FAILED: {e}"));
                state.set_lastcc(ConditionCode::Severe);
                return;
            }
        }
    }

    let attrs = DatasetAttributes {
        recfm: Some(Recfm::VB),
        lrecl: cmd.recordsize.map(|(_, max)| max),
        blksize: cmd.bufferspace,
        dsorg: Some(Dsorg::PS),
        volser: cmd.add_volumes.first().cloned(),
    };

    match services.catalog.update_dataset(entry, attrs) {
        Ok(()) => {
            state.emit_message(
                MessageCode::IDC0003I,
                &format!("ENTRY {} ALTERED", cmd.entry_name),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::DatasetNotFound { .. }) => {
            state.emit_message(
                MessageCode::IDC0560E,
                &format!("ENTRY {} NOT FOUND", cmd.entry_name),
            );
            state.set_lastcc(ConditionCode::Error);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0561E, &format!("ALTER FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}
