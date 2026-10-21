# English Identity_Base keys for the Virtual Catalog New/Edit/Delete dialogs
# (CR-NR-103, Localisation Phase 2 Slice 2, FEAT-catalog-dialog).
#
# These values are the EXACT current English literals from crate
# ff-catalog-dialog (new_dialog.rs, edit_dialog.rs, delete_dialog.rs, lib.rs),
# byte-for-byte, ASCII only. Render code resolves each displayed string through
# the ff_i18n::t()/t_args() seam keyed by these ids (localization Req 4.1, 10.1).
# Command verbs, enum logic keys, egui id_salt values, and the Home-catalog
# comparison string are NOT localised (Req 4.2, 4.4). English base only --
# locale overlays layer on top later.
#
# Several field-label values carry SIGNIFICANT trailing (or leading) spaces used
# for column alignment in the dialogs. Fluent trims surrounding whitespace from a
# plain value, so those are preserved with a quoted string-literal placeable
# ({ "  " }) to keep the rendered text byte-identical to the original literal.

# --- Catalog type display labels (shared New + Edit; Req 4.1) ---------------
# DISPLAY text for the type combo; distinct from the CatalogType enum variants
# used in match arms, which are NOT localised (Req 4.4).
catalog-dialog-type-mainframe = Mainframe
catalog-dialog-type-posix = POSIX
catalog-dialog-type-native = Native

# --- Shared buttons (reused across the three dialogs; flat namespace) -------
catalog-dialog-btn-ok = OK
catalog-dialog-btn-cancel = Cancel
catalog-dialog-btn-save = Save

# --- New Catalog dialog -----------------------------------------------------
catalog-dialog-new-title = New Catalog
catalog-dialog-new-label-type = Catalog Type:
catalog-dialog-new-label-name = Catalog Name:{ "  " }
catalog-dialog-new-label-description = Description:{ "   " }
catalog-dialog-new-label-repository-path = Repository Path:
catalog-dialog-new-label-default-hlq = Default HLQ:{ "    " }
catalog-dialog-new-label-root-directory = Root Directory:{ " " }
catalog-dialog-new-label-mount-point = Mount Point:{ "    " }
catalog-dialog-new-label-root-path = Root Path:{ "      " }
catalog-dialog-new-chk-auto-mount = Auto-mount on startup
catalog-dialog-new-chk-read-only = Read-Only
catalog-dialog-new-chk-create-repository = Create repository now

# New Catalog validation messages (Req 3.3-3.8).
catalog-dialog-new-err-name-format = Catalog Name must be 1-32 characters (alphanumeric, hyphen, underscore).
catalog-dialog-new-err-duplicate = A catalog named '{ $name }' already exists.
catalog-dialog-new-err-description-length = Description must be 120 characters or fewer.
catalog-dialog-new-err-repository-path-required = Repository Path is required for Mainframe catalogs.
catalog-dialog-new-err-root-directory-required = Root Directory is required for POSIX catalogs.
catalog-dialog-new-err-root-path-required = Root Path is required for Native catalogs.
catalog-dialog-new-err-create-repository-failed = Failed to create repository: { $error }

# --- Edit Catalog dialog ----------------------------------------------------
catalog-dialog-edit-title = Edit Catalog
catalog-dialog-edit-label-name = Catalog Name:{ "  " }
catalog-dialog-edit-label-type = Catalog Type:{ "  " }
catalog-dialog-edit-label-repository-path = Repository Path:
catalog-dialog-edit-label-description = Description:{ "   " }
catalog-dialog-edit-label-default-hlq = Default HLQ:{ "   " }
catalog-dialog-edit-chk-auto-mount = Auto-mount on startup
catalog-dialog-edit-chk-read-only = Read-Only
catalog-dialog-edit-err-description-length = Description must be 120 characters or fewer.

# --- Delete Catalog dialog --------------------------------------------------
catalog-dialog-delete-title = Delete Catalog
catalog-dialog-delete-confirm = Delete catalog "{ $name }"? This will unmount it.
catalog-dialog-delete-optional-files = Optionally delete all backing files.
catalog-dialog-delete-btn-catalog-only = Delete Catalog Only
catalog-dialog-delete-btn-catalog-and-files = Delete Catalog and Files
catalog-dialog-delete-err-home-protected = The Home catalog cannot be deleted. Rename or edit it instead.
catalog-dialog-delete-err-delete-failed = Failed to delete '{ $path }': { $error }
