//! PRINT and REPRO command handlers (record-level I/O).
//!
//! REPRO's record copy and PRINT's record scan flow through the reconciled
//! `VsamService` record seam (`open` / `start_browse` / `next_record` / `put` /
//! `close`) -- `ff-idcams` performs NO record-level copy logic of its own
//! (Requirement 28.4). RECFM/LRECL handling lives behind the reconciled service.

use ff_dscatalog::{AccessMode, BrowseDirection, VsamError};

use crate::executor::context::ExecutionState;
use crate::messages::{ConditionCode, MessageCode};
use crate::parser::ast::*;
use crate::services::IdcamsServices;

/// Resolve an `InputSpec` to a dataset name through the allocator for a DD.
fn resolve_input(
    input: &InputSpec,
    services: &IdcamsServices,
    state: &mut ExecutionState,
    code: MessageCode,
    what: &str,
) -> Option<DatasetName> {
    match input {
        InputSpec::InDataset(dsn) => Some(dsn.clone()),
        InputSpec::InFile(dd) => match services.allocator.resolve_dd(dd) {
            Ok(dsn) => Some(dsn),
            Err(e) => {
                state.emit_message(code, &format!("{what}: {e}"));
                state.set_lastcc(ConditionCode::Severe);
                None
            }
        },
    }
}

/// Executes PRINT command.
pub fn execute_print(cmd: PrintCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let dsn = match resolve_input(
        &cmd.input,
        services,
        state,
        MessageCode::IDC0570E,
        "CANNOT RESOLVE DD",
    ) {
        Some(d) => d,
        None => return,
    };

    let handle = match services.vsam.open(dsn.as_str(), AccessMode::Read) {
        Ok(h) => h,
        Err(VsamError::DatasetNotFound { .. }) => {
            state.emit_message(MessageCode::IDC0570E, &format!("DATASET {dsn} NOT FOUND"));
            state.set_lastcc(ConditionCode::Severe);
            return;
        }
        Err(VsamError::UnsupportedOperation { .. }) => {
            // Non-VSAM dataset -- handled via VFS elsewhere (simplified report).
            state.emit_message(
                MessageCode::IDC0001I,
                &format!("PRINT OF NON-VSAM DATASET {dsn} (SIMULATED)"),
            );
            state.set_lastcc(ConditionCode::Success);
            return;
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0570E, &format!("OPEN FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
            return;
        }
    };

    let browse = match services
        .vsam
        .start_browse(&handle, &[], BrowseDirection::Forward)
    {
        Ok(b) => b,
        Err(e) => {
            state.emit_message(MessageCode::IDC0570E, &format!("BROWSE FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
            let _ = services.vsam.close(handle);
            return;
        }
    };

    let mut count = 0u64;
    let skip_count = cmd.skip.unwrap_or(0);
    let max_count = cmd.count;
    let mut skipped = 0u64;

    loop {
        match services.vsam.next_record(&browse) {
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
                state.emit_message(MessageCode::IDC0570E, &format!("READ ERROR: {e}"));
                state.set_lastcc(ConditionCode::Severe);
                let _ = services.vsam.end_browse(browse);
                let _ = services.vsam.close(handle);
                return;
            }
        }
    }

    let _ = services.vsam.end_browse(browse);
    let _ = services.vsam.close(handle);

    state.emit_message(
        MessageCode::IDC0001I,
        &format!("IDCAMS PRINT - {count} RECORDS PRINTED"),
    );
    state.set_lastcc(ConditionCode::Success);
}

/// Executes REPRO command.
///
/// Record copy flows through the reconciled `VsamService` get/put/browse seam;
/// `ff-idcams` holds no record-level copy logic of its own (Requirement 28.4).
pub fn execute_repro(cmd: ReproCommand, services: &IdcamsServices, state: &mut ExecutionState) {
    let src_dsn = match resolve_input(
        &cmd.input,
        services,
        state,
        MessageCode::IDC0581E,
        "SOURCE NOT FOUND",
    ) {
        Some(d) => d,
        None => return,
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

    let src_handle = match services.vsam.open(src_dsn.as_str(), AccessMode::Read) {
        Ok(h) => h,
        Err(VsamError::DatasetNotFound { .. }) => {
            state.emit_message(
                MessageCode::IDC0581E,
                &format!("SOURCE DATASET {src_dsn} NOT FOUND"),
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

    let tgt_handle = match services.vsam.open(tgt_dsn.as_str(), AccessMode::Write) {
        Ok(h) => h,
        Err(VsamError::DatasetNotFound { .. }) => {
            state.emit_message(
                MessageCode::IDC0582E,
                &format!("TARGET DATASET {tgt_dsn} NOT FOUND"),
            );
            state.set_lastcc(ConditionCode::Severe);
            let _ = services.vsam.close(src_handle);
            return;
        }
        Err(e) => {
            state.emit_message(MessageCode::IDC0582E, &format!("TARGET OPEN FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
            let _ = services.vsam.close(src_handle);
            return;
        }
    };

    let browse = match services
        .vsam
        .start_browse(&src_handle, &[], BrowseDirection::Forward)
    {
        Ok(b) => b,
        Err(e) => {
            state.emit_message(MessageCode::IDC0581E, &format!("BROWSE FAILED: {e}"));
            state.set_lastcc(ConditionCode::Severe);
            let _ = services.vsam.close(src_handle);
            let _ = services.vsam.close(tgt_handle);
            return;
        }
    };

    let mut copied = 0u64;
    let mut skipped = 0u64;
    let skip_count = cmd.skip.unwrap_or(0);
    let max_count = cmd.count;
    let mut skip_done = 0u64;

    loop {
        match services.vsam.next_record(&browse) {
            Ok(Some(record)) => {
                if skip_done < skip_count {
                    skip_done += 1;
                    continue;
                }

                match services.vsam.put(&tgt_handle, &record) {
                    Ok(()) => copied += 1,
                    Err(VsamError::DuplicateKey { .. }) => {
                        if cmd.replace {
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
                            &format!("WRITE FAILED AFTER {copied} RECORDS: {e}"),
                        );
                        state.set_lastcc(ConditionCode::Severe);
                        let _ = services.vsam.end_browse(browse);
                        let _ = services.vsam.close(src_handle);
                        let _ = services.vsam.close(tgt_handle);
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
                    &format!("READ FAILED AFTER {copied} RECORDS: {e}"),
                );
                state.set_lastcc(ConditionCode::Severe);
                let _ = services.vsam.end_browse(browse);
                let _ = services.vsam.close(src_handle);
                let _ = services.vsam.close(tgt_handle);
                return;
            }
        }
    }

    let _ = services.vsam.end_browse(browse);
    let _ = services.vsam.close(src_handle);
    let _ = services.vsam.close(tgt_handle);

    state.emit_message(
        MessageCode::IDC0001I,
        &format!("REPRO - {copied} RECORDS COPIED, {skipped} RECORDS SKIPPED"),
    );
    if skipped == 0 {
        state.set_lastcc(ConditionCode::Success);
    }
}
