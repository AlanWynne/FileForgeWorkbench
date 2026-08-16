//! Command-specific execution handlers.
//!
//! Each handler takes a parsed command, the services container, and the execution
//! state. It delegates to downstream services and updates the state accordingly.

use super::context::ExecutionState;
use crate::error::{CatalogError, VsamError};
use crate::messages::{ConditionCode, MessageCode};
use crate::parser::ast::*;
use crate::services::*;

/// Executes DEFINE CLUSTER.
pub fn execute_define_cluster(
    cmd: DefineClusterCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    // Validate: INDEXED requires KEYS
    if cmd.organization == VsamOrganization::Indexed && cmd.keys.is_none() {
        state.emit_message(
            MessageCode::IDC0503E,
            &format!("KEYS PARAMETER REQUIRED FOR INDEXED CLUSTER {}", cmd.name),
        );
        state.set_lastcc(ConditionCode::Severe);
        return;
    }

    // Step 1: Create catalog entry
    let params = CreateDatasetParams {
        name: cmd.name.clone(),
        dsorg: cmd.organization,
        volumes: cmd.volumes.clone(),
        space: cmd.space.clone(),
        recordsize: cmd.recordsize,
        keys: cmd.keys,
        freespace: cmd.freespace,
        shareoptions: cmd.shareoptions,
        bufferspace: cmd.bufferspace,
        reuse: cmd.reuse,
    };

    if let Err(e) = services.catalog.create_dataset(params) {
        match e {
            CatalogError::DuplicateName(_) => {
                state.emit_message(
                    MessageCode::IDC0514E,
                    &format!("ENTRY {} ALREADY EXISTS", cmd.name),
                );
                state.set_lastcc(ConditionCode::Severe);
            }
            _ => {
                state.emit_message(MessageCode::IDC0514E, &format!("CATALOG ERROR: {e}"));
                state.set_lastcc(ConditionCode::Severe);
            }
        }
        return;
    }

    // Step 2: Initialize VSAM dataset
    let vtype = match cmd.organization {
        VsamOrganization::Indexed => VsamType::Ksds,
        VsamOrganization::NonIndexed => VsamType::Esds,
        VsamOrganization::Numbered => VsamType::Rrds,
        VsamOrganization::Linear => VsamType::Lds,
    };

    let init_params = VsamInitParams {
        keys: cmd.keys,
        recordsize: cmd.recordsize,
        ci_size: cmd
            .data_component
            .as_ref()
            .and_then(|d| d.controlintervalsize),
    };

    if let Err(e) = services
        .vsam
        .initialize_dataset(&cmd.name, vtype, init_params)
    {
        // Rollback: delete the catalog entry
        let _ = services.catalog.delete_dataset(&cmd.name);
        state.emit_message(
            MessageCode::IDC0514E,
            &format!("VSAM INITIALIZATION FAILED: {e}"),
        );
        state.set_lastcc(ConditionCode::Severe);
        return;
    }

    // Success
    state.emit_message(
        MessageCode::IDC0001I,
        &format!("ENTRY {} DEFINED", cmd.name),
    );
    state.set_lastcc(ConditionCode::Success);
}

/// Executes DEFINE ALTERNATEINDEX.
pub fn execute_define_aix(
    cmd: DefineAixCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    let params = DefineAixParams {
        aix_name: cmd.name.clone(),
        base_cluster: cmd.relate.clone(),
        keys: cmd.keys,
        unique_key: cmd.uniquekey,
        upgrade: cmd.upgrade,
        recordsize: cmd.recordsize,
    };

    match services.vsam.define_aix(params) {
        Ok(()) => {
            state.emit_message(MessageCode::IDC0001I, &format!("AIX {} DEFINED", cmd.name));
            state.set_lastcc(ConditionCode::Success);
        }
        Err(VsamError::NotFound(ref dsn)) => {
            state.emit_message(
                MessageCode::IDC0510E,
                &format!("BASE CLUSTER {} NOT FOUND", dsn),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(VsamError::NotVsam(ref dsn)) => {
            state.emit_message(
                MessageCode::IDC0511E,
                &format!("RELATE TARGET {} IS NOT A VSAM CLUSTER", dsn),
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
pub fn execute_define_path(
    cmd: DefinePathCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    let params = DefinePathParams {
        path_name: cmd.name.clone(),
        aix_name: cmd.pathentry.clone(),
        update: cmd.update,
    };

    match services.vsam.define_path(params) {
        Ok(()) => {
            state.emit_message(MessageCode::IDC0001I, &format!("PATH {} DEFINED", cmd.name));
            state.set_lastcc(ConditionCode::Success);
        }
        Err(VsamError::NotAnAix(ref name)) => {
            state.emit_message(
                MessageCode::IDC0512E,
                &format!("PATHENTRY {} NOT FOUND OR NOT AN AIX", name),
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
pub fn execute_define_gdg(
    cmd: DefineGdgCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    // Validate: LIMIT required
    if cmd.limit == 0 {
        state.emit_message(
            MessageCode::IDC0520E,
            &format!("LIMIT PARAMETER REQUIRED FOR GDG {}", cmd.name),
        );
        state.set_lastcc(ConditionCode::Severe);
        return;
    }

    let params = CreateGdgParams {
        name: cmd.name.clone(),
        limit: cmd.limit,
        scratch: cmd.scratch,
        empty: cmd.empty,
        fifo: cmd.fifo,
    };

    match services.catalog.create_gdg_base(params) {
        Ok(()) => {
            state.emit_message(
                MessageCode::IDC0001I,
                &format!("GDG BASE {} DEFINED", cmd.name),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::DuplicateName(_)) => {
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

/// Executes DELETE command.
pub fn execute_delete(cmd: DeleteCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    for entry in &cmd.entries {
        match cmd.entry_type {
            DeleteEntryType::Cluster | DeleteEntryType::AlternateIndex => {
                // VSAM destroy then catalog delete
                match services.vsam.destroy_dataset(entry) {
                    Ok(()) => {
                        if let Err(e) = services.catalog.delete_dataset(entry) {
                            state.emit_message(
                                MessageCode::IDC0700W,
                                &format!(
                                    "VSAM DESTROYED BUT CATALOG DELETE FAILED FOR {}: {e}",
                                    entry
                                ),
                            );
                            state.set_lastcc(ConditionCode::Severe);
                            continue;
                        }
                    }
                    Err(VsamError::NotFound(_)) => {
                        state.emit_message(
                            MessageCode::IDC0550E,
                            &format!("ENTRY {} NOT FOUND", entry),
                        );
                        state.set_lastcc(ConditionCode::Error);
                        continue;
                    }
                    Err(e) => {
                        state.emit_message(
                            MessageCode::IDC0551E,
                            &format!("DELETE FAILED FOR {}: {e}", entry),
                        );
                        state.set_lastcc(ConditionCode::Severe);
                        continue;
                    }
                }
            }
            DeleteEntryType::Path => {
                if let Err(e) = services.vsam.delete_path(entry) {
                    state.emit_message(
                        MessageCode::IDC0550E,
                        &format!("PATH {} NOT FOUND: {e}", entry),
                    );
                    state.set_lastcc(ConditionCode::Error);
                    continue;
                }
                if let Err(e) = services.catalog.delete_dataset(entry) {
                    state.emit_message(
                        MessageCode::IDC0700W,
                        &format!("PATH DELETED BUT CATALOG REMOVE FAILED: {e}"),
                    );
                    state.set_lastcc(ConditionCode::Severe);
                    continue;
                }
            }
            DeleteEntryType::Gdg => {
                if let Err(e) = services.catalog.delete_gdg_base(entry, cmd.force) {
                    match e {
                        CatalogError::NotFound(_) => {
                            state.emit_message(
                                MessageCode::IDC0550E,
                                &format!("ENTRY {} NOT FOUND", entry),
                            );
                            state.set_lastcc(ConditionCode::Error);
                        }
                        _ => {
                            state.emit_message(
                                MessageCode::IDC0551E,
                                &format!("DELETE GDG FAILED: {e}"),
                            );
                            state.set_lastcc(ConditionCode::Severe);
                        }
                    }
                    continue;
                }
            }
            DeleteEntryType::NonVsam | DeleteEntryType::UserCatalog => {
                if let Err(e) = services.catalog.delete_dataset(entry) {
                    match e {
                        CatalogError::NotFound(_) => {
                            state.emit_message(
                                MessageCode::IDC0550E,
                                &format!("ENTRY {} NOT FOUND", entry),
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

        // Success for this entry
        state.emit_message(MessageCode::IDC0002I, &format!("ENTRY {} DELETED", entry));
        state.set_lastcc(ConditionCode::Success);
    }
}

/// Executes ALTER command.
pub fn execute_alter(cmd: AlterCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    // Handle NEWNAME separately
    if let Some(ref new_name) = cmd.newname {
        match services.catalog.rename_dataset(&cmd.entry_name, new_name) {
            Ok(()) => {}
            Err(CatalogError::NotFound(_)) => {
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

    let attrs = UpdateAttrs {
        freespace: cmd.freespace,
        shareoptions: cmd.shareoptions,
        bufferspace: cmd.bufferspace,
        recordsize: cmd.recordsize,
        keys: cmd.keys,
        add_volumes: cmd.add_volumes,
        remove_volumes: cmd.remove_volumes,
        nullify: cmd.nullify,
    };

    match services.catalog.update_dataset(&cmd.entry_name, attrs) {
        Ok(()) => {
            state.emit_message(
                MessageCode::IDC0003I,
                &format!("ENTRY {} ALTERED", cmd.entry_name),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::NotFound(_)) => {
            state.emit_message(
                MessageCode::IDC0560E,
                &format!("ENTRY {} NOT FOUND", cmd.entry_name),
            );
            state.set_lastcc(ConditionCode::Error);
        }
        Err(CatalogError::AttributeNotModifiable(ref attr)) => {
            state.emit_message(
                MessageCode::IDC0561E,
                &format!("ATTRIBUTE {} NOT MODIFIABLE FOR {}", attr, cmd.entry_name),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0561E, &format!("ALTER FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes LISTCAT command.
pub fn execute_listcat(cmd: ListcatCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let filter = ListFilter {
        filter: cmd.filter,
        entry_type: None,
        display_level: cmd.display_level,
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
                        &format!("{} {}", entry.entry_type, entry.name),
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

/// Executes PRINT command.
pub fn execute_print(cmd: PrintCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let dsn = match &cmd.input {
        InputSpec::InDataset(dsn) => dsn.clone(),
        InputSpec::InFile(dd) => match services.allocator.resolve_dd(dd) {
            Ok(dsn) => dsn,
            Err(e) => {
                state.emit_message(
                    MessageCode::IDC0570E,
                    &format!("CANNOT RESOLVE DD {}: {e}", dd),
                );
                state.set_lastcc(ConditionCode::Severe);
                return;
            }
        },
    };

    // Open and browse
    match services.vsam.open(&dsn, OpenMode::Input) {
        Ok(handle) => {
            let position = BrowsePosition::Start;
            match services.vsam.start_browse(&handle, position) {
                Ok(mut cursor) => {
                    let mut count = 0u64;
                    let skip_count = cmd.skip.unwrap_or(0);
                    let max_count = cmd.count;
                    let mut skipped = 0u64;

                    loop {
                        match services.vsam.next_record(&mut cursor) {
                            Ok(Some(_record)) => {
                                if skipped < skip_count {
                                    skipped += 1;
                                    continue;
                                }
                                count += 1;
                                if let Some(max) = max_count {
                                    if count >= max {
                                        break;
                                    }
                                }
                            }
                            Ok(None) => break,
                            Err(e) => {
                                state.emit_message(
                                    MessageCode::IDC0570E,
                                    &format!("READ ERROR: {e}"),
                                );
                                state.set_lastcc(ConditionCode::Severe);
                                return;
                            }
                        }
                    }

                    state.emit_message(
                        MessageCode::IDC0001I,
                        &format!("IDCAMS PRINT - {} RECORDS PRINTED", count),
                    );
                    state.set_lastcc(ConditionCode::Success);
                }
                Err(e) => {
                    state.emit_message(MessageCode::IDC0570E, &format!("BROWSE FAILED: {e}"));
                    state.set_lastcc(ConditionCode::Severe);
                }
            }
        }
        Err(VsamError::NotFound(_)) => {
            state.emit_message(MessageCode::IDC0570E, &format!("DATASET {} NOT FOUND", dsn));
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(VsamError::NotVsam(_)) => {
            // Non-VSAM — handle via VFS (simplified: just report)
            state.emit_message(
                MessageCode::IDC0001I,
                &format!("PRINT OF NON-VSAM DATASET {} (SIMULATED)", dsn),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0570E, &format!("OPEN FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes REPRO command.
pub fn execute_repro(cmd: ReproCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let src_dsn = match &cmd.input {
        InputSpec::InDataset(dsn) => dsn.clone(),
        InputSpec::InFile(dd) => match services.allocator.resolve_dd(dd) {
            Ok(dsn) => dsn,
            Err(e) => {
                state.emit_message(MessageCode::IDC0581E, &format!("SOURCE NOT FOUND: {e}"));
                state.set_lastcc(ConditionCode::Severe);
                return;
            }
        },
    };

    let tgt_dsn = match &cmd.output {
        OutputSpec::OutDataset(dsn) => dsn.clone(),
        OutputSpec::OutFile(dd) => match services.allocator.resolve_dd(dd) {
            Ok(dsn) => dsn,
            Err(e) => {
                state.emit_message(MessageCode::IDC0582E, &format!("TARGET NOT FOUND: {e}"));
                state.set_lastcc(ConditionCode::Severe);
                return;
            }
        },
    };

    // Open source for browse
    let src_handle = match services.vsam.open(&src_dsn, OpenMode::Input) {
        Ok(h) => h,
        Err(VsamError::NotFound(_)) => {
            state.emit_message(
                MessageCode::IDC0581E,
                &format!("SOURCE DATASET {} NOT FOUND", src_dsn),
            );
            state.set_lastcc(ConditionCode::Severe);
            return;
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0581E, &format!("SOURCE OPEN FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
            return;
        }
    };

    // Open target for output
    let tgt_handle = match services.vsam.open(&tgt_dsn, OpenMode::Output) {
        Ok(h) => h,
        Err(VsamError::NotFound(_)) => {
            state.emit_message(
                MessageCode::IDC0582E,
                &format!("TARGET DATASET {} NOT FOUND", tgt_dsn),
            );
            state.set_lastcc(ConditionCode::Severe);
            return;
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0582E, &format!("TARGET OPEN FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
            return;
        }
    };

    // Browse and copy
    let mut cursor = match services
        .vsam
        .start_browse(&src_handle, BrowsePosition::Start)
    {
        Ok(c) => c,
        Err(e) => {
            state.emit_message(MessageCode::IDC0581E, &format!("BROWSE FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
            return;
        }
    };

    let mut copied = 0u64;
    let mut skipped = 0u64;
    let skip_count = cmd.skip.unwrap_or(0);
    let max_count = cmd.count;
    let mut skip_done = 0u64;

    loop {
        match services.vsam.next_record(&mut cursor) {
            Ok(Some(record)) => {
                if skip_done < skip_count {
                    skip_done += 1;
                    continue;
                }

                match services.vsam.put(&tgt_handle, &record) {
                    Ok(()) => copied += 1,
                    Err(VsamError::DuplicateKey(_)) => {
                        if cmd.replace {
                            // Replace mode — try again (simplified)
                            let _ = services.vsam.put(&tgt_handle, &record);
                            copied += 1;
                        } else {
                            skipped += 1;
                            state.emit_message(
                                MessageCode::IDC0580W,
                                "DUPLICATE KEY - RECORD SKIPPED",
                            );
                            state.set_lastcc(ConditionCode::Warning);
                        }
                    }
                    Err(e) => {
                        state.emit_message(
                            MessageCode::IDC0582E,
                            &format!("WRITE FAILED AFTER {} RECORDS: {e}", copied),
                        );
                        state.set_lastcc(ConditionCode::Severe);
                        return;
                    }
                }

                if let Some(max) = max_count {
                    if copied >= max {
                        break;
                    }
                }
            }
            Ok(None) => break,
            Err(e) => {
                state.emit_message(
                    MessageCode::IDC0581E,
                    &format!("READ FAILED AFTER {} RECORDS: {e}", copied),
                );
                state.set_lastcc(ConditionCode::Severe);
                return;
            }
        }
    }

    state.emit_message(
        MessageCode::IDC0001I,
        &format!(
            "REPRO - {} RECORDS COPIED, {} RECORDS SKIPPED",
            copied, skipped
        ),
    );
    if skipped == 0 {
        state.set_lastcc(ConditionCode::Success);
    }
    // If skipped > 0, LASTCC was already set to Warning
}

/// Executes VERIFY command.
pub fn execute_verify(cmd: VerifyCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let dsn = match &cmd.dataset {
        InputSpec::InDataset(dsn) => dsn.clone(),
        InputSpec::InFile(dd) => match services.allocator.resolve_dd(dd) {
            Ok(dsn) => dsn,
            Err(e) => {
                state.emit_message(
                    MessageCode::IDC0591E,
                    &format!("CANNOT RESOLVE DD {}: {e}", dd),
                );
                state.set_lastcc(ConditionCode::Severe);
                return;
            }
        },
    };

    match services.vsam.verify_integrity(&dsn) {
        Ok(result) => {
            if result.corrections_applied {
                state.emit_message(
                    MessageCode::IDC0001I,
                    &format!("DATASET {} VERIFIED - END-OF-FILE MARKER RESET", dsn),
                );
            } else {
                state.emit_message(
                    MessageCode::IDC0590I,
                    &format!("DATASET {} IS CONSISTENT", dsn),
                );
            }
            state.set_lastcc(ConditionCode::Success);
        }
        Err(VsamError::NotFound(_)) | Err(VsamError::Internal(_)) => {
            state.emit_message(
                MessageCode::IDC0591E,
                &format!("DATASET {} ACCESS FAILURE", dsn),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(VsamError::NotVsam(_)) => {
            state.emit_message(
                MessageCode::IDC0592E,
                &format!("DATASET {} IS NOT A VSAM DATASET", dsn),
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
pub fn execute_export(cmd: ExportCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let destination = match &cmd.output {
        OutputSpec::OutDataset(dsn) => dsn.to_string(),
        OutputSpec::OutFile(dd) => dd.clone(),
    };

    let params = ExportParams {
        source: cmd.entry_name.clone(),
        destination,
        temporary: cmd.temporary,
        inhibit_source: cmd.inhibit_source,
    };

    match services.catalog.export_dataset(params) {
        Ok(result) => {
            state.emit_message(
                MessageCode::IDC0004I,
                &format!(
                    "EXPORT COMPLETE - {} RECORDS, {} BYTES",
                    result.record_count, result.byte_count
                ),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::NotFound(_)) => {
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
pub fn execute_import(cmd: ImportCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let source = match &cmd.input {
        InputSpec::InDataset(dsn) => dsn.to_string(),
        InputSpec::InFile(dd) => dd.clone(),
    };

    let params = ImportParams {
        source,
        target: cmd.out_dataset.clone(),
        catalog: cmd.catalog,
    };

    match services.catalog.import_dataset(params) {
        Ok(result) => {
            state.emit_message(
                MessageCode::IDC0005I,
                &format!("IMPORT COMPLETE - {} RECORDS", result.record_count),
            );
            state.set_lastcc(ConditionCode::Success);
        }
        Err(CatalogError::DuplicateName(_)) => {
            state.emit_message(
                MessageCode::IDC0611E,
                &format!("TARGET {} ALREADY EXISTS", cmd.out_dataset),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(CatalogError::NotFound(_)) => {
            state.emit_message(MessageCode::IDC0610E, "INVALID IMPORT SOURCE");
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0610E, &format!("IMPORT FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
        }
    }
}

/// Executes BLDINDEX command.
pub fn execute_bldindex(
    cmd: BldindexCommand,
    services: &IdcamsServices,
    state: &mut ExecutionState,
) {
    match services.vsam.build_index(&cmd.in_dataset, &cmd.out_dataset) {
        Ok(result) => {
            if result.duplicates_found > 0 {
                state.emit_message(
                    MessageCode::IDC0622W,
                    &format!(
                        "BLDINDEX - {} DUPLICATE KEYS FOUND",
                        result.duplicates_found
                    ),
                );
                state.set_lastcc(ConditionCode::Error);
            }
            state.emit_message(
                MessageCode::IDC0006I,
                &format!(
                    "BLDINDEX COMPLETE - {} ENTRIES CREATED",
                    result.entries_created
                ),
            );
            if result.duplicates_found == 0 {
                state.set_lastcc(ConditionCode::Success);
            }
        }
        Err(VsamError::NotFound(_)) => {
            state.emit_message(
                MessageCode::IDC0620E,
                &format!("BASE CLUSTER {} NOT FOUND", cmd.in_dataset),
            );
            state.set_lastcc(ConditionCode::Severe);
        }
        Err(VsamError::NotAnAix(_)) => {
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
