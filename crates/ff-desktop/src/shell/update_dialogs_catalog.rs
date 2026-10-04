//! # Catalog Manager dialog cluster
//!
//! The Catalog Manager / Allocate Dataset dialog match arm rendered as the last
//! step of `render_overlays`. Extracted verbatim from `update_dialogs.rs` as part
//! of the file-size split; behaviour and order are unchanged -- `render_overlays`
//! calls this as its LAST statement, so the early `return`s inside the
//! AllocateDataset arm end the frame exactly as before.

use eframe::egui;

use crate::catalog_manager_dialog::{self, DeleteChoice, DialogOutcome};
use crate::dataset_alloc_dialog::{self, validate_for_catalog, AllocOutcome, Dsorg, Recfm};
use crate::files_panel;
use ff_dscatalog::{
    dataset::{
        AllocParams as DsAllocParams, Dsorg as DsDsorg, PartitionedSubtype, Recfm as DsRecfm,
    },
    dsn::Dsn,
    hierarchy::CatalogScope,
};

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the Catalog Manager dialog cluster (New/Edit/Delete catalog and
    /// Allocate Dataset). Extracted verbatim from `render_overlays`; it is the
    /// last statement there, so the early `return`s inside the AllocateDataset
    /// arm end the frame exactly as before.
    pub(super) fn render_catalog_dialogs(&mut self, ctx: &egui::Context) {
        // Catalog Manager Dialogs - Req 3.1-3.8, 4.1-4.5
        match &mut self.files_panel.dialog {
            files_panel::FilesDialogState::NewCatalog(ref mut form) => {
                let outcome =
                    catalog_manager_dialog::render(ctx, form, &mut self.files_panel.registry);
                if outcome == DialogOutcome::Confirmed {
                    // Persist immediately so a force-close does not lose the new catalog (B020).
                    if let Some(session) = &self.session {
                        session.save_catalog_registry(&self.files_panel.registry);
                    }
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                } else if outcome == DialogOutcome::Cancelled {
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::EditCatalog(ref mut form) => {
                let outcome =
                    catalog_manager_dialog::render_edit(ctx, form, &mut self.files_panel.registry);
                if outcome == DialogOutcome::Confirmed || outcome == DialogOutcome::Cancelled {
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::DeleteCatalog(ref confirm) => {
                let choice = catalog_manager_dialog::render_delete(ctx, confirm);
                if choice != DeleteChoice::Cancel {
                    let confirm_clone = confirm.clone();
                    if let Err(e) = catalog_manager_dialog::execute_delete(
                        &choice,
                        &confirm_clone,
                        &mut self.files_panel.registry,
                    ) {
                        self.open_error = Some(e);
                    } else {
                        // Persist immediately so a force-close does not lose the deletion (B020).
                        if let Some(session) = &self.session {
                            session.save_catalog_registry(&self.files_panel.registry);
                        }
                    }
                }
                self.files_panel.dialog = files_panel::FilesDialogState::None;
            }
            files_panel::FilesDialogState::AllocateDataset(ref mut form) => {
                let outcome = dataset_alloc_dialog::render(ctx, form);
                if outcome == AllocOutcome::Confirmed {
                    // Req 13.1 -- validate form (duplicate check deferred to SQLite uniqueness)
                    match validate_for_catalog(form, &[]) {
                        Ok(params) => {
                            if let Some(cat) = self.files_panel.pending_alloc_catalog.take() {
                                // Convert UI AllocParams -> ff-dscatalog AllocParams
                                let ds_params = ui_params_to_ds_params(params, form);
                                match ds_params {
                                    Ok(p) => {
                                        if let Err(e) = self.files_panel.registry.allocate(&cat, p)
                                        {
                                            form.error = Some(format!("Allocation failed: {e}"));
                                            self.files_panel.pending_alloc_catalog = Some(cat);
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        form.error = Some(e);
                                        self.files_panel.pending_alloc_catalog = Some(cat);
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            form.error = Some(e);
                            return;
                        }
                    }
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                } else if outcome == AllocOutcome::Cancelled {
                    self.files_panel.pending_alloc_catalog = None;
                    self.files_panel.dialog = files_panel::FilesDialogState::None;
                }
            }
            files_panel::FilesDialogState::None => {}
        }
    }
}

/// Convert the UI-layer `AllocParams` to the ff-dscatalog `AllocParams`.
///
/// Returns `Err` if the dataset name is not a valid DSN.
fn ui_params_to_ds_params(
    params: crate::dataset_alloc_dialog::AllocParams,
    form: &crate::dataset_alloc_dialog::AllocDatasetForm,
) -> Result<DsAllocParams, String> {
    let dsn = Dsn::parse(&params.dataset_name)
        .map_err(|_| format!("'{}': invalid dataset name", params.dataset_name))?;
    let dsorg = match params.dsorg {
        Dsorg::Ps => DsDsorg::PS,
        Dsorg::Po | Dsorg::Pdse => DsDsorg::PO,
        Dsorg::Gdg => DsDsorg::GDG,
    };
    let subtype = match params.dsorg {
        Dsorg::Pdse => Some(PartitionedSubtype::PDSE),
        Dsorg::Po => Some(PartitionedSubtype::PDS),
        _ => None,
    };
    let recfm = match params.recfm {
        Recfm::Fb => Some(DsRecfm::FB),
        Recfm::F => Some(DsRecfm::F),
        Recfm::Vb => Some(DsRecfm::VB),
        Recfm::V => Some(DsRecfm::V),
        Recfm::U => Some(DsRecfm::U),
    };
    let gdg_limit = params.gdg_limit.map(|n| n as u8);
    let gdg_scratch = if params.dsorg == Dsorg::Gdg {
        Some(form.scratch)
    } else {
        None
    };
    Ok(DsAllocParams {
        dsn,
        dsorg,
        recfm,
        lrecl: Some(params.lrecl),
        blksize: if params.blksize == 0 {
            None
        } else {
            Some(params.blksize)
        },
        dir_blocks: params.dir_blocks,
        gdg_limit,
        gdg_scratch,
        subtype,
        description: params.description,
        scope: CatalogScope::User,
    })
}
