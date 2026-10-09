//! The DEFINE VOLUME service and the command CONTRACT (Requirement 10).
//!
//! `VolumeService` owns a `VolumeRegistry` and implements `define_volume`
//! (Requirement 10.1, 10.3) and the dataset -> DatasetVolume -> Volume -> locator
//! resolution seam that the future `DatasetAccess` contract (RC.B.6) consumes.
//!
//! The command CONTRACT is DATA ONLY: command-name consts plus parameter
//! descriptors. There is NO `ff-command` dispatch wiring and NO
//! `WorkspaceContext` here -- those are built in RC.C.10 and resolve through the
//! single command-dispatch path when wired (Requirement 10.2, 10.5).

use crate::dataset_volume::{DatasetVolumeSet, ResolvedVolume};
use crate::error::VolumeError;
use crate::volume::{AccessMode, Volser, Volume, VolumeId, VolumeRegistry, VolumeStatus};

// === Command CONTRACT (data only) ===================================================

/// The `DEFINE VOLUME` FFWB-extension command name (Requirement 10.1). Resolves
/// through the single command-dispatch path once wired in RC.C.10
/// (Requirement 10.2); this crate exposes the name only.
pub const DEFINE_VOLUME_COMMAND: &str = "DEFINE VOLUME";

/// The volume-listing command name (Requirement 10.4). Dispatch wired in
/// RC.C.10.
pub const VOLUME_LISTING_COMMAND: &str = "LISTVOL";

/// The VTOC-view command name (Requirement 8.5, 10.5). Surfaced as a
/// `WorkspaceContext` via `render_workspace_context` when wired in RC.C.10.
pub const VTOC_COMMAND: &str = "VTOC";

/// Descriptor for the `DEFINE VOLUME` command parameters (Requirement 10.1).
/// Data only -- the dispatch arm in RC.C.10 reads this contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefineVolumeParams {
    /// Whether the VOLSER/name parameter is required (always true).
    pub volser_required: bool,
    /// Whether a path parameter is accepted.
    pub accepts_path: bool,
    /// Whether a capacity parameter is accepted.
    pub accepts_capacity: bool,
    /// Whether a status parameter is accepted.
    pub accepts_status: bool,
}

impl DefineVolumeParams {
    /// The canonical `DEFINE VOLUME` parameter contract (Requirement 10.1).
    pub const CONTRACT: Self = Self {
        volser_required: true,
        accepts_path: true,
        accepts_capacity: true,
        accepts_status: true,
    };
}

/// Descriptor for the volume-listing command parameters (Requirement 10.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeListingParams {
    /// Whether a VOLSER filter is accepted (optional).
    pub accepts_volser_filter: bool,
}

impl VolumeListingParams {
    /// The canonical listing parameter contract.
    pub const CONTRACT: Self = Self {
        accepts_volser_filter: true,
    };
}

/// Descriptor for the VTOC command parameters (Requirement 8.5, 10.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VtocParams {
    /// Whether a VOLSER argument is required to pick the Volume.
    pub volser_required: bool,
}

impl VtocParams {
    /// The canonical VTOC parameter contract.
    pub const CONTRACT: Self = Self {
        volser_required: true,
    };
}

// === DefineVolumeRequest ===================================================

/// A request to register a host directory as an emulated Volume
/// (Requirement 10.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefineVolumeRequest {
    /// The VOLSER (also used as the display name if none is given).
    pub volser: String,
    /// Optional display name.
    pub display_name: Option<String>,
    /// The host directory path (today's Repository root).
    pub path: String,
    /// Total capacity in tracks.
    pub capacity_tracks: u64,
    /// Initial status.
    pub status: VolumeStatus,
}

// === VolumeService ===================================================

/// Owns the `VolumeRegistry` and provides the DEFINE VOLUME service plus the
/// resolution seam the future `DatasetAccess` contract consumes (RC.B.6).
#[derive(Debug, Default)]
pub struct VolumeService {
    registry: VolumeRegistry,
    next_id: u64,
}

impl VolumeService {
    /// Creates an empty service.
    pub fn new() -> Self {
        Self {
            registry: VolumeRegistry::new(),
            next_id: 1,
        }
    }

    /// Returns a reference to the underlying registry.
    pub fn registry(&self) -> &VolumeRegistry {
        &self.registry
    }

    /// Registers a host directory as an emulated Volume (Requirement 10.1),
    /// rejecting a duplicate VOLSER (Requirement 10.3).
    ///
    /// # Errors
    /// `VolumeError::DuplicateVolser` when the VOLSER already exists, or any
    /// VOLSER validation error from `Volser::try_new`.
    pub fn define_volume(&mut self, request: DefineVolumeRequest) -> Result<VolumeId, VolumeError> {
        let volser = Volser::try_new(request.volser)?;
        let id = VolumeId(self.next_id);
        let mut volume = Volume::new(id, volser, request.path, request.capacity_tracks);
        if let Some(name) = request.display_name {
            volume = volume.with_display_name(name);
        }
        if request.status == VolumeStatus::Offline {
            volume.set_offline();
        }
        volume.set_access_mode(AccessMode::ReadWrite);
        self.registry.define(volume)?;
        self.next_id += 1;
        Ok(id)
    }

    /// Resolution seam for `DatasetAccess` (RC.B.6, NOT built here): resolves a
    /// dataset's Volumes in sequence order honouring the Online check
    /// (Requirement 2.4, 9.2). Documented here so RC.B.6 consumes a stable API.
    ///
    /// # Errors
    /// Propagates `DatasetVolumeSet::resolve` errors (`VolumeNotFound`,
    /// `VolumeOffline`).
    pub fn resolve_dataset(
        &self,
        volumes: &DatasetVolumeSet,
    ) -> Result<Vec<ResolvedVolume>, VolumeError> {
        volumes.resolve(&self.registry)
    }
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn request(volser: &str) -> DefineVolumeRequest {
        DefineVolumeRequest {
            volser: volser.to_string(),
            display_name: None,
            path: format!("/vol/{volser}"),
            capacity_tracks: 1000,
            status: VolumeStatus::Online,
        }
    }

    // Validates: Requirement 10.1, 10.3 -- define registers and rejects duplicate VOLSER
    #[test]
    fn define_volume_registers_and_rejects_duplicate_volser() {
        let mut svc = VolumeService::new();
        let id = svc.define_volume(request("VOL001")).expect("first");
        assert_eq!(id, VolumeId(1));
        let err = svc
            .define_volume(request("vol001"))
            .expect_err("duplicate (case-insensitive)");
        assert_eq!(
            err,
            VolumeError::DuplicateVolser {
                volser: "VOL001".to_string()
            }
        );
        assert_eq!(svc.registry().len(), 1);
    }

    // Validates: Requirement 10.1 -- define accepts name, path, capacity, status
    #[test]
    fn define_volume_accepts_name_path_capacity_status() {
        let mut svc = VolumeService::new();
        let mut req = request("VOL001");
        req.display_name = Some("Primary Pool".to_string());
        req.capacity_tracks = 5000;
        req.status = VolumeStatus::Offline;
        let id = svc.define_volume(req).expect("defined");
        let vol = svc.registry().find_by_id(id).expect("present");
        assert_eq!(vol.display_name(), Some("Primary Pool"));
        assert_eq!(vol.total_tracks(), 5000);
        assert_eq!(vol.status(), VolumeStatus::Offline);
        assert_eq!(vol.storage_uri(), "/vol/VOL001");
    }

    // Validates: Requirement 10.1 -- the DEFINE VOLUME command contract is exposed as data
    #[test]
    fn define_volume_command_contract_is_data_only() {
        assert_eq!(DEFINE_VOLUME_COMMAND, "DEFINE VOLUME");
        assert!(DefineVolumeParams::CONTRACT.volser_required);
        assert!(DefineVolumeParams::CONTRACT.accepts_capacity);
        assert!(VtocParams::CONTRACT.volser_required);
        assert!(VolumeListingParams::CONTRACT.accepts_volser_filter);
    }
}
