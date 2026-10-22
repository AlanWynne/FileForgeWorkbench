# English Identity_Base keys for the Dataset Allocation dialog
# (CR-NR-103, Localisation Phase 2 Slice 2, task 7.2, crate ff-dataset-alloc-dialog).
#
# These values are the EXACT current English literals from crate
# ff-dataset-alloc-dialog (render.rs, validate.rs), byte-for-byte, ASCII only.
# Render code resolves each displayed string through the ff_i18n::t()/t_args()
# seam keyed by these ids (localization Req 4.1, 10.1). The DSORG/RECFM mnemonic
# labels (PS/PO/PDSE/GDG, FB/F/VB/V/U) are technical mainframe format codes that
# are identical across languages; they stay as code literals and are NOT
# localised (Req 4.2, 4.4), mirroring how the CatalogType match arms stayed
# unlocalised in the catalog dialog. English base only -- locale overlays layer
# on top later.
#
# Several field-label values carry SIGNIFICANT trailing (or leading) spaces used
# for column alignment in the dialog. Fluent trims surrounding whitespace from a
# plain value, so those are preserved with a quoted string-literal placeable
# ({ "  " }) to keep the rendered text byte-identical to the original literal.

# --- Shared buttons ---------------------------------------------------------
dataset-alloc-btn-allocate = Allocate
dataset-alloc-btn-cancel = Cancel

# --- Allocate Dataset dialog labels -----------------------------------------
dataset-alloc-title = Allocate Dataset
dataset-alloc-label-name = Dataset Name:{ "      " }
dataset-alloc-label-dsorg = Dataset Org (DSORG):
dataset-alloc-label-recfm = Record Format:{ "     " }
dataset-alloc-label-lrecl = LRECL:{ "             " }
dataset-alloc-label-blksize = { "  " }Block Size:
dataset-alloc-label-dir-blocks = Directory Blocks:{ "  " }
dataset-alloc-label-gdg-limit = GDG Limit (1-255):{ " " }
dataset-alloc-label-description = Description:{ "       " }
dataset-alloc-chk-scratch = Scratch on roll-off

# --- Validation messages (Req 5.3, 5.8, 5.9) --------------------------------
dataset-alloc-err-name-required = Dataset Name is required.
dataset-alloc-err-lrecl-integer = LRECL must be a positive integer.
dataset-alloc-err-lrecl-range = LRECL must be between 1 and 32760 (got { $lrecl }).
dataset-alloc-err-blksize-integer = Block Size must be a non-negative integer.
dataset-alloc-err-blksize-range = Block Size ({ $blksize }) must be >= LRECL ({ $lrecl }), or 0 for system-determined.
dataset-alloc-err-dir-blocks-integer = Directory Blocks must be a positive integer.
dataset-alloc-err-gdg-limit-integer = GDG Limit must be an integer between 1 and 255.
dataset-alloc-err-gdg-limit-range = GDG Limit must be between 1 and 255 (got { $limit }).
dataset-alloc-err-duplicate = Dataset '{ $name }' already exists in this catalog.
