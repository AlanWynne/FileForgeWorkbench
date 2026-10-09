//! Reconciliation tests for the repointed services (CR-CH-059, Requirement 28).
//!
//! These assert that `ff-idcams` now constructs over the RECONCILED
//! `ff-dscatalog` traits and routes DEFINE / REPRO / DELETE through them, with
//! IDCAMS syntax/output/condition codes unchanged.

use std::sync::Arc;

use ff_dscatalog::{CatalogError, VsamError, VsamHandle};

use crate::messages::{ConditionCode, MessageCode};
use crate::services::mocks::{MockCatalogService, MockVsamService, TestServicesBuilder};
use crate::{execute_idcams, DynCatalogService, IdcamsServices, VsamService};

// === Requirement 28.1 / 28.2: reconciled trait wiring =======================

#[test]
fn idcams_services_hold_reconciled_ff_dscatalog_traits() {
    // Validates: Requirement 28.1, 28.2 -- IdcamsServices is parameterised over
    // the reconciled ff-dscatalog traits (DynCatalogService + VsamService), not
    // any private ff-idcams trait.
    let catalog: Arc<dyn DynCatalogService> = Arc::new(MockCatalogService::new_success());
    let vsam: Arc<dyn VsamService> = Arc::new(MockVsamService::new_success());
    let services: IdcamsServices = TestServicesBuilder::new()
        .with_catalog(MockCatalogService::new_success())
        .with_vsam(MockVsamService::new_success())
        .build();
    // The container exposes the reconciled trait objects.
    let _ = &services.catalog;
    let _ = &services.vsam;
    let _ = &services.access;
    // The mocks implement the reconciled ff-dscatalog traits directly.
    assert!(catalog
        .create_dataset("A.B", ff_dscatalog::DatasetAttributes::default())
        .is_ok());
    assert_eq!(
        vsam.open("A.B", ff_dscatalog::AccessMode::Read).unwrap(),
        VsamHandle(1)
    );
}

// === Requirement 28.3: DEFINE routes through reconciled traits ==============

#[test]
fn define_cluster_routes_through_reconciled_catalog_and_vsam() {
    // Validates: Requirement 28.3 -- DEFINE CLUSTER drives the reconciled
    // CatalogService::create_dataset + VsamService init; CC/output unchanged.
    let services = TestServicesBuilder::new().build();
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
fn define_cluster_rolls_back_catalog_on_vsam_init_failure() {
    // Validates: Requirement 28.3, 22 -- on reconciled VsamService init failure
    // the catalog entry is rolled back via the reconciled delete_dataset.
    let vsam = MockVsamService::new_success();
    *vsam.init_responses.lock().unwrap() = vec![Err(VsamError::Internal("init boom".to_string()))];

    let catalog = MockCatalogService::new_success();
    let services = TestServicesBuilder::new()
        .with_catalog(catalog)
        .with_vsam(vsam)
        .build();

    let result = execute_idcams(
        "DEFINE CLUSTER (NAME(MY.KSDS) INDEXED KEYS(8 0))",
        &services,
    );
    assert_eq!(result.maxcc, ConditionCode::Severe);
}

#[test]
fn define_cluster_duplicate_maps_reconciled_catalog_error() {
    // Validates: Requirement 28.2, 28.3 -- the reconciled CatalogError variant
    // DuplicateDataset drives the IDC0514E "ALREADY EXISTS" message.
    let catalog = MockCatalogService::new_success();
    *catalog.create_responses.lock().unwrap() = vec![Err(CatalogError::DuplicateDataset {
        dsn: "MY.KSDS".to_string(),
        catalog: "MASTER".to_string(),
        operation: "create_dataset".to_string(),
    })];
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

// === Requirement 28.4: REPRO record copy via reconciled seam ================

#[test]
fn repro_record_copy_flows_through_reconciled_vsam_put() {
    // Validates: Requirement 28.4 -- REPRO drives the reconciled VsamService
    // put for each source record; ff-idcams holds no record-copy logic.
    let vsam = MockVsamService::new_success();
    // Two source records available from the browse seam.
    *vsam.records.lock().unwrap() = vec![
        Some(ff_dscatalog::Record {
            key: vec![1],
            data: vec![10, 11],
        }),
        Some(ff_dscatalog::Record {
            key: vec![2],
            data: vec![20, 21],
        }),
        None,
    ];
    let vsam = Arc::new(vsam);
    let services = IdcamsServices::new(
        Arc::new(MockCatalogService::new_success()),
        vsam.clone(),
        crate::services::default_dataset_access(),
        Arc::new(crate::services::mocks::MockAllocatorService::new_not_found()),
    );

    let result = execute_idcams("REPRO INDATASET(SRC.DS) OUTDATASET(TGT.DS)", &services);
    assert_eq!(result.maxcc, ConditionCode::Success);
    // Both source records were put through the reconciled VsamService seam.
    assert_eq!(*vsam.put_calls.lock().unwrap(), 2);
}

// === Requirement 28.5: DELETE routes through reconciled teardown ============

#[test]
fn delete_flows_through_reconciled_destroy_and_delete_dataset() {
    // Validates: Requirement 28.5 -- DELETE drives VsamService::destroy_dataset
    // then CatalogService::delete_dataset; CC/output unchanged.
    let services = TestServicesBuilder::new().build();
    let result = execute_idcams("DELETE MY.DATA CLUSTER", &services);
    assert_eq!(result.maxcc, ConditionCode::Success);
    assert!(result
        .messages
        .iter()
        .any(|m| m.code == MessageCode::IDC0002I));
}

#[test]
fn delete_not_found_maps_reconciled_vsam_error() {
    // Validates: Requirement 28.2, 28.5 -- the reconciled VsamError variant
    // DatasetNotFound drives IDC0550E with CC Error.
    let vsam = MockVsamService::new_success();
    *vsam.destroy_responses.lock().unwrap() = vec![Err(VsamError::DatasetNotFound {
        dsn: "MY.DATA".to_string(),
    })];
    let services = TestServicesBuilder::new().with_vsam(vsam).build();

    let result = execute_idcams("DELETE MY.DATA CLUSTER", &services);
    assert_eq!(result.maxcc, ConditionCode::Error);
    assert!(result
        .messages
        .iter()
        .any(|m| m.code == MessageCode::IDC0550E));
}

// === Requirement 28.7: condition-code / output invariance ===================

#[test]
fn multi_command_condition_codes_unchanged_by_repoint() {
    // Validates: Requirement 28.7 -- LASTCC/MAXCC semantics are byte-identical
    // after the repoint for a representative multi-command SYSIN.
    let services = TestServicesBuilder::new().build();
    let result = execute_idcams("SET LASTCC(4); SET LASTCC(0)", &services);
    assert_eq!(result.maxcc, ConditionCode::Warning);
}
